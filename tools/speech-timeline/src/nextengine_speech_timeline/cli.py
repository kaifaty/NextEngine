from __future__ import annotations

import argparse
import asyncio
import json
import logging
import os
from pathlib import Path
import sys
from typing import Sequence

from . import SERVICE_PROTOCOL
from .benchmark import (
    BenchmarkError,
    benchmark_service,
    evaluate_affect_calibration,
    load_affect_calibration_manifest,
    load_benchmark_wav,
    write_report,
)
from .corpus_import import (
    ReliabilityImportError,
    import_common_voice,
    import_fleurs_ru,
    import_musan_noise,
    import_rirs,
)
from .corpus_replay import (
    DEFAULT_TIMEOUT_SECONDS,
    ReliabilityReplayError,
    replay_corpus,
)
from .diagnostic_audio import DiagnosticAudioStore
from .microphone_client import (
    ClientError,
    ReadyInfo,
    capture_module,
    load_ready_file,
    microphone_chunks,
    render_event,
    run_websocket_session,
    validate_debug_wav,
)
from .profile import ProfileError, build_adapters, load_profile
from .protocol import ASR_AUDIO_ROUTES, ASR_AUDIO_ROUTE_RAW
from .reliability import (
    ReliabilityCorpusError,
    dry_run_reliability_corpus,
    load_dataset_recipe,
    prepare_reliability_index,
    score_transcript,
    write_reliability_report,
)
from .service import SpeechTimelineRuntime
from .transport_websocket import SpeechTimelineWebSocketService


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="next-speech-timeline")
    root.add_argument("--version", action="version", version=SERVICE_PROTOCOL)
    commands = root.add_subparsers(dest="command", required=True)
    serve = commands.add_parser("serve", help="start the resident loopback service")
    serve.add_argument("--profile", type=Path, required=True)
    serve.add_argument(
        "--log-level",
        choices=("DEBUG", "INFO", "WARNING", "ERROR"),
        default=os.environ.get("NEXTENGINE_SPEECH_LOG_LEVEL", "INFO").upper(),
        help="diagnostic log verbosity (default: INFO)",
    )
    microphone = commands.add_parser(
        "microphone", help="stream the Linux microphone into a resident service"
    )
    microphone.add_argument("--ready-file", type=Path)
    microphone.add_argument("--device", default="default")
    microphone.add_argument("--chunk-ms", type=int, default=80)
    microphone.add_argument("--duration", type=float)
    microphone.add_argument("--locale", default="ru")
    microphone.add_argument("--probe-seconds", type=int, default=2)
    microphone.add_argument("--list-inputs", action="store_true")
    microphone.add_argument("--save-wav", type=Path)
    microphone.add_argument("--json", action="store_true", dest="json_output")
    microphone.add_argument(
        "--asr-model",
        help="resident ASR route ID advertised by the service (default: service default)",
    )
    microphone.add_argument(
        "--asr-delay-ms",
        type=int,
        choices=(480, 960, 2_400),
        help="lock a supported ASR delay for this utterance",
    )
    microphone.add_argument(
        "--asr-audio-route",
        choices=sorted(ASR_AUDIO_ROUTES),
        default=ASR_AUDIO_ROUTE_RAW,
        help="ASR input route; raw is the selected control (default: raw)",
    )
    benchmark = commands.add_parser(
        "benchmark", help="stream an external WAV and write a content-free timing report"
    )
    benchmark.add_argument("--ready-file", type=Path, required=True)
    benchmark.add_argument("--audio", type=Path, required=True)
    benchmark.add_argument("--mode", choices=("paced", "unpaced"), default="paced")
    benchmark.add_argument("--runs", type=int, default=2)
    benchmark.add_argument("--chunk-ms", type=int, default=80)
    benchmark.add_argument("--out", type=Path, required=True)
    benchmark.add_argument(
        "--asr-model",
        help="resident ASR route ID advertised by the service (default: service default)",
    )
    benchmark.add_argument(
        "--asr-delay-ms",
        type=int,
        choices=(480, 960, 2_400),
        help="lock a supported ASR delay for every benchmark turn",
    )
    benchmark.add_argument(
        "--asr-audio-route",
        choices=sorted(ASR_AUDIO_ROUTES),
        default=ASR_AUDIO_ROUTE_RAW,
        help="route the same benchmark WAV through raw or configured enhancement",
    )
    evaluate_affect = commands.add_parser(
        "evaluate-affect",
        help="replay an external held-out affect manifest through the resident service",
    )
    evaluate_affect.add_argument("--ready-file", type=Path, required=True)
    evaluate_affect.add_argument("--manifest", type=Path, required=True)
    evaluate_affect.add_argument(
        "--mode",
        choices=("paced", "unpaced"),
        default="unpaced",
        help="unpaced preserves sample-clock behavior without claiming live wall latency",
    )
    evaluate_affect.add_argument("--chunk-ms", type=int, default=80)
    evaluate_affect.add_argument("--out", type=Path, required=True)
    reliability = commands.add_parser(
        "reliability-corpus",
        help="validate and prepare an external public-data ASR reliability corpus",
    )
    reliability_commands = reliability.add_subparsers(
        dest="reliability_command", required=True
    )
    reliability_dry_run = reliability_commands.add_parser(
        "dry-run",
        help="validate a recipe and report missing closure without reading audio",
    )
    reliability_dry_run.add_argument("--manifest", type=Path, required=True)
    reliability_dry_run.add_argument("--store", type=Path, required=True)
    reliability_dry_run.add_argument("--out", type=Path, required=True)
    reliability_prepare = reliability_commands.add_parser(
        "prepare",
        help="verify external source indexes/audio and publish a prepared index",
    )
    reliability_prepare.add_argument("--manifest", type=Path, required=True)
    reliability_prepare.add_argument("--store", type=Path, required=True)
    reliability_prepare.add_argument("--out-index", type=Path, required=True)
    reliability_prepare.add_argument("--out", type=Path, required=True)
    reliability_score = reliability_commands.add_parser(
        "score",
        help="apply ru-asr-normalize-v0 and report deterministic WER/CER",
    )
    reliability_score.add_argument("--reference", required=True)
    reliability_score.add_argument("--hypothesis", required=True)
    reliability_replay = reliability_commands.add_parser(
        "replay",
        help="replay one prepared hash-closed corpus through the resident service",
    )
    reliability_replay.add_argument("--manifest", type=Path, required=True)
    reliability_replay.add_argument("--store", type=Path, required=True)
    reliability_replay.add_argument("--prepared-index", type=Path, required=True)
    reliability_replay.add_argument("--ready-file", type=Path, required=True)
    reliability_replay.add_argument("--out-dir", type=Path, required=True)
    reliability_replay.add_argument(
        "--asr-model",
        required=True,
        help="one exact resident ASR route ID for the whole run",
    )
    reliability_replay.add_argument(
        "--asr-delay-ms",
        type=int,
        choices=(480, 960, 2_400),
        help="lock a supported ASR delay for every replayed turn",
    )
    reliability_replay.add_argument(
        "--asr-audio-route",
        choices=sorted(ASR_AUDIO_ROUTES),
        default=ASR_AUDIO_ROUTE_RAW,
        help="single ASR audio route for the whole run (default: raw)",
    )
    reliability_replay.add_argument(
        "--mode",
        choices=("paced", "unpaced"),
        default="paced",
        help="paced preserves realtime ingress shape (default)",
    )
    reliability_replay.add_argument("--limit", type=int)
    reliability_replay.add_argument(
        "--split",
        action="append",
        choices=("train", "calibration", "held_out"),
        dest="splits",
        help="restrict replay to a split; repeatable (default: all)",
    )
    reliability_replay.add_argument(
        "--timeout-seconds",
        type=float,
        default=DEFAULT_TIMEOUT_SECONDS,
        help="per-clip session timeout before a typed failure",
    )
    import_fleurs = reliability_commands.add_parser(
        "import-fleurs",
        help="convert acquired FLEURS ru_ru into the store source index",
    )
    import_fleurs.add_argument("--store", type=Path, required=True)
    import_fleurs.add_argument("--fleurs-root", type=Path, required=True)
    import_fleurs.add_argument("--out-index", type=Path, required=True)
    import_fleurs.add_argument("--workers", type=int, default=8)
    import_musan = reliability_commands.add_parser(
        "import-musan-noise",
        help="index the MUSAN noise subset, converting non-conforming files",
    )
    import_musan.add_argument("--store", type=Path, required=True)
    import_musan.add_argument("--noise-root", type=Path, required=True)
    import_musan.add_argument("--out-index", type=Path, required=True)
    import_musan.add_argument("--workers", type=int, default=8)
    import_rirs_cmd = reliability_commands.add_parser(
        "import-rirs",
        help="reference contract-conforming RIRS_NOISES assets in an index",
    )
    import_rirs_cmd.add_argument("--store", type=Path, required=True)
    import_rirs_cmd.add_argument("--rirs-root", type=Path, required=True)
    import_rirs_cmd.add_argument("--out-index", type=Path, required=True)
    import_rirs_cmd.add_argument(
        "--include-pointsource-noises",
        action="store_true",
        help="also admit pointsource_noises as rir assets",
    )
    import_cv = reliability_commands.add_parser(
        "import-common-voice",
        help="convert a Common Voice release into a bounded speech index",
    )
    import_cv.add_argument("--store", type=Path, required=True)
    import_cv.add_argument("--cv-root", type=Path, required=True)
    import_cv.add_argument("--out-index", type=Path, required=True)
    import_cv.add_argument(
        "--kind",
        choices=("scripted", "spontaneous"),
        required=True,
        help="scripted reads validated.tsv over clips/, spontaneous reads ss-corpus TSV",
    )
    import_cv.add_argument(
        "--max-rows",
        type=int,
        default=60_000,
        help="deterministic file-order admission cap (default: 60000)",
    )
    import_cv.add_argument("--workers", type=int, default=12)
    return root


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parser().parse_args(argv)
    if arguments.command == "serve":
        logging.basicConfig(
            level=logging.WARNING,
            format="%(asctime)s %(levelname)s %(name)s %(message)s",
        )
        logging.getLogger("nextengine.speech_timeline").setLevel(
            getattr(logging, arguments.log_level)
        )
        try:
            profile = load_profile(arguments.profile)
            transcribers, default_transcriber, affect, audio_preprocessor = build_adapters(profile)
            runtime = SpeechTimelineRuntime(
                transcribers,
                affect,
                audio_preprocessor,
                default_transcriber=default_transcriber,
            )
            diagnostic_audio = (
                DiagnosticAudioStore(
                    profile.service.diagnostic_audio_root,
                    profile.service.diagnostic_audio_max_records or 5,
                )
                if profile.service.diagnostic_audio_root is not None
                else None
            )
            service = SpeechTimelineWebSocketService(
                runtime,
                ready_file=profile.service.ready_file,
                port=profile.service.port,
                bounds=profile.service.bounds,
                diagnostic_audio=diagnostic_audio,
                model_identity={
                    **(
                        {
                            "voxtral_sha256": profile.voxtral.model_sha256,
                            "transcribe_revision": profile.voxtral.runtime_revision,
                        }
                        if profile.voxtral is not None
                        else {}
                    ),
                    "default_asr_model": profile.default_asr_model,
                    **(
                        {
                            "gigaam_model_id": profile.gigaam.model_id,
                            "gigaam_revision": profile.gigaam.model_revision,
                            "gigaam_weights_sha256": profile.gigaam.weights_sha256,
                            "gigaam_modeling_sha256": profile.gigaam.modeling_sha256,
                            "gigaam_classification": profile.gigaam.classification,
                        }
                        if profile.gigaam is not None
                        else {}
                    ),
                    **(
                        {
                            "audio_enhancers": [
                                {
                                    "adapter_id": enhancer.adapter_id,
                                    "model_id": enhancer.model_id,
                                    "model_revision": enhancer.model_revision,
                                    "model_sha256": enhancer.model_sha256,
                                    "routing": enhancer.routing,
                                }
                                for enhancer in profile.audio_enhancers
                            ]
                        }
                        if profile.audio_enhancers
                        else {}
                    ),
                    **(
                        {
                            "gigastt_model_id": profile.gigastt.model_id,
                            "gigastt_model_revision": profile.gigastt.model_revision,
                            "gigastt_runtime_version": profile.gigastt.runtime_version,
                            "gigastt_runtime_revision": profile.gigastt.runtime_revision,
                            "gigastt_runtime_sha256": profile.gigastt.runtime_sha256,
                            "gigastt_encoder_sha256": profile.gigastt.encoder_sha256,
                            "gigastt_decoder_sha256": profile.gigastt.decoder_sha256,
                            "gigastt_joint_sha256": profile.gigastt.joint_sha256,
                            "gigastt_vocab_sha256": profile.gigastt.vocab_sha256,
                            "gigastt_classification": profile.gigastt.classification,
                        }
                        if profile.gigastt is not None
                        else {}
                    ),
                    **(
                        {
                            "nemotron_model_id": profile.nemotron.model_id,
                            "nemotron_revision": profile.nemotron.model_revision,
                            "nemotron_model_sha256": profile.nemotron.model_sha256,
                            "nemotron_runtime_revision": profile.nemotron.runtime_revision,
                            "nemotron_implementation_library_sha256": (
                                profile.nemotron.implementation_library_sha256
                            ),
                            "nemotron_abi_library_sha256": (
                                profile.nemotron.abi_library_sha256
                            ),
                            "nemotron_classification": profile.nemotron.classification,
                        }
                        if profile.nemotron is not None
                        else {}
                    ),
                    "emotion_adapter_id": profile.emotion.adapter_id,
                    "emotion_model_id": profile.emotion.model_id,
                    "emotion_revision": profile.emotion.model_revision,
                    "emotion_classification": profile.emotion.classification,
                    **(
                        {
                            "audio_preprocessor_adapter_id": profile.audio_preprocessor.adapter_id,
                            "audio_preprocessor_model_id": profile.audio_preprocessor.model_id,
                            "audio_preprocessor_revision": profile.audio_preprocessor.model_revision,
                            "audio_preprocessor_model_name": profile.audio_preprocessor.model_name,
                            "audio_preprocessor_model_sha256": profile.audio_preprocessor.model_sha256,
                            "audio_preprocessor_routing": profile.audio_preprocessor.routing,
                            "audio_preprocessor_gain": profile.audio_preprocessor.gain_config.as_dict(),
                            "audio_preprocessor_gain_placement": profile.audio_preprocessor.gain_placement,
                        }
                        if profile.audio_preprocessor is not None
                        else {}
                    ),
                    **(
                        {"emotion_weights_sha256": profile.emotion.weights_sha256}
                        if profile.emotion.weights_sha256 is not None
                        else {}
                    ),
                },
            )
            return asyncio.run(_serve(service, profile.service.ready_file))
        except ProfileError as error:
            print(
                json.dumps(
                    {"schema_version": 1, "status": "error", "error": str(error)},
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 2
        except KeyboardInterrupt:
            return 130
    if arguments.command == "microphone":
        try:
            capture = capture_module()
            if arguments.list_inputs:
                return capture.list_inputs()
            if arguments.ready_file is None:
                raise ClientError("--ready-file is required unless --list-inputs is used")
            if not 20 <= arguments.chunk_ms <= 1_000:
                raise ClientError("--chunk-ms must be between 20 and 1000")
            if arguments.duration is not None and not 0 < arguments.duration <= 30:
                raise ClientError("--duration must be greater than 0 and at most 30 seconds")
            if not 1 <= arguments.probe_seconds <= 10:
                raise ClientError("--probe-seconds must be between 1 and 10")
            debug_wav = validate_debug_wav(arguments.save_wav)
            try:
                capture.probe_microphone(
                    arguments.device,
                    seconds=arguments.probe_seconds,
                    threshold_dbfs=capture.DEFAULT_SILENCE_THRESHOLD_DBFS,
                )
            except SystemExit as error:
                raise ClientError(str(error)) from error
            ready = load_ready_file(arguments.ready_file)
            return asyncio.run(
                _microphone(
                    ready,
                    device=arguments.device,
                    chunk_ms=arguments.chunk_ms,
                    duration=arguments.duration,
                    locale=arguments.locale,
                    debug_wav=debug_wav,
                    json_output=arguments.json_output,
                    asr_audio_route=arguments.asr_audio_route,
                    asr_model=arguments.asr_model,
                    asr_delay_ms=arguments.asr_delay_ms,
                )
            )
        except ClientError as error:
            print(
                json.dumps(
                    {"schema_version": 1, "status": "error", "error": str(error)},
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 2
        except KeyboardInterrupt:
            return 130
    if arguments.command == "benchmark":
        try:
            ready = load_ready_file(arguments.ready_file)
            pcm, samples = load_benchmark_wav(arguments.audio)
            report = asyncio.run(
                benchmark_service(
                    ready,
                    pcm,
                    samples,
                    mode=arguments.mode,
                    runs=arguments.runs,
                    chunk_ms=arguments.chunk_ms,
                    asr_audio_route=arguments.asr_audio_route,
                    asr_model=arguments.asr_model,
                    asr_delay_ms=arguments.asr_delay_ms,
                )
            )
            write_report(arguments.out, report)
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "status": "complete",
                        "report": str(arguments.out.expanduser().resolve()),
                        "summary": report["summary"],
                    },
                    ensure_ascii=False,
                ),
                flush=True,
            )
            return 0
        except (BenchmarkError, ClientError) as error:
            print(
                json.dumps(
                    {"schema_version": 1, "status": "error", "error": str(error)},
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 2
        except KeyboardInterrupt:
            return 130
    if arguments.command == "evaluate-affect":
        try:
            ready = load_ready_file(arguments.ready_file)
            manifest = load_affect_calibration_manifest(arguments.manifest)
            report = asyncio.run(
                evaluate_affect_calibration(
                    ready,
                    manifest,
                    mode=arguments.mode,
                    chunk_ms=arguments.chunk_ms,
                )
            )
            write_report(arguments.out, report)
            print(
                json.dumps(
                    {
                        "schema_version": 1,
                        "status": "complete",
                        "report": str(arguments.out.expanduser().resolve()),
                        "summary": report["summary"],
                    },
                    ensure_ascii=False,
                ),
                flush=True,
            )
            return 0
        except (BenchmarkError, ClientError) as error:
            print(
                json.dumps(
                    {"schema_version": 1, "status": "error", "error": str(error)},
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 2
        except KeyboardInterrupt:
            return 130
    if arguments.command == "reliability-corpus":
        try:
            if arguments.reliability_command == "score":
                print(
                    json.dumps(
                        {
                            "schema_version": 0,
                            "status": "complete",
                            **score_transcript(
                                arguments.reference, arguments.hypothesis
                            ).as_dict(),
                        },
                        ensure_ascii=False,
                        sort_keys=True,
                    ),
                    flush=True,
                )
                return 0
            if arguments.reliability_command == "import-fleurs":
                report = import_fleurs_ru(
                    arguments.store,
                    fleurs_root=arguments.fleurs_root,
                    out_index=arguments.out_index,
                    workers=arguments.workers,
                )
                print(json.dumps({"schema_version": 0, "status": "complete",
                                  **report}, ensure_ascii=False, sort_keys=True),
                      flush=True)
                return 0
            if arguments.reliability_command == "import-musan-noise":
                report = import_musan_noise(
                    arguments.store,
                    noise_root=arguments.noise_root,
                    out_index=arguments.out_index,
                    workers=arguments.workers,
                )
                print(json.dumps({"schema_version": 0, "status": "complete",
                                  **report}, ensure_ascii=False, sort_keys=True),
                      flush=True)
                return 0
            if arguments.reliability_command == "import-common-voice":
                report = import_common_voice(
                    arguments.store,
                    cv_root=arguments.cv_root,
                    out_index=arguments.out_index,
                    kind=arguments.kind,
                    max_rows=arguments.max_rows,
                    workers=arguments.workers,
                )
                print(json.dumps({"schema_version": 0, "status": "complete",
                                  **report}, ensure_ascii=False, sort_keys=True),
                      flush=True)
                return 0
            if arguments.reliability_command == "import-rirs":
                subsets = ["simulated_rirs", "real_rirs_isotropic_noises"]
                if arguments.include_pointsource_noises:
                    subsets.append("pointsource_noises")
                report = import_rirs(
                    arguments.store,
                    rirs_root=arguments.rirs_root,
                    out_index=arguments.out_index,
                    subsets=tuple(subsets),
                )
                print(json.dumps({"schema_version": 0, "status": "complete",
                                  **report}, ensure_ascii=False, sort_keys=True),
                      flush=True)
                return 0
            if arguments.reliability_command == "replay":
                ready = load_ready_file(arguments.ready_file)
                report = asyncio.run(
                    replay_corpus(
                        ready,
                        manifest_path=arguments.manifest,
                        store=arguments.store,
                        prepared_index_path=arguments.prepared_index,
                        out_dir=arguments.out_dir,
                        asr_model=arguments.asr_model,
                        asr_audio_route=arguments.asr_audio_route,
                        asr_delay_ms=arguments.asr_delay_ms,
                        mode=arguments.mode,
                        splits=(
                            tuple(arguments.splits) if arguments.splits else None
                        ),
                        limit=arguments.limit,
                        timeout_seconds=arguments.timeout_seconds,
                    )
                )
                print(
                    json.dumps(
                        {
                            "schema_version": 0,
                            "status": report["status"],
                            "report": report["written_files"]["report.json"],
                            "counts": report["counts"],
                        },
                        ensure_ascii=False,
                        sort_keys=True,
                    ),
                    flush=True,
                )
                return 0
            recipe = load_dataset_recipe(arguments.manifest)
            if arguments.reliability_command == "dry-run":
                report = dry_run_reliability_corpus(recipe, arguments.store)
            elif arguments.reliability_command == "prepare":
                report = prepare_reliability_index(
                    recipe, arguments.store, arguments.out_index
                )
            else:
                return 2
            write_reliability_report(arguments.out, report)
            print(
                json.dumps(
                    {
                        "schema_version": 0,
                        "status": report["status"],
                        "report": str(arguments.out.expanduser().resolve()),
                        "dataset_id": report["dataset_id"],
                        "ready_for_prepare": report.get("ready_for_prepare"),
                        "samples_prepared": report.get("samples_prepared"),
                    },
                    ensure_ascii=False,
                    sort_keys=True,
                ),
                flush=True,
            )
            return 0
        except (
            ReliabilityCorpusError,
            ReliabilityReplayError,
            ReliabilityImportError,
            ClientError,
        ) as error:
            print(
                json.dumps(
                    {
                        "schema_version": 0,
                        "status": "error",
                        "error": str(error),
                    },
                    ensure_ascii=False,
                ),
                file=sys.stderr,
            )
            return 2
    return 2


async def _serve(service: SpeechTimelineWebSocketService, ready_file: Path) -> int:
    try:
        await service.start()
        print(
            json.dumps(
                {
                    "schema_version": 1,
                    "status": "ready",
                    "uri": service.uri,
                    "dashboard_uri": service.dashboard_uri,
                    "ready_file": str(ready_file),
                },
                ensure_ascii=False,
            ),
            flush=True,
        )
        await service.serve_forever()
    finally:
        await service.close()
    return 0


async def _microphone(
    ready: ReadyInfo,
    *,
    device: str,
    chunk_ms: int,
    duration: float | None,
    locale: str | None,
    debug_wav: Path | None,
    json_output: bool,
    asr_audio_route: str,
    asr_model: str | None,
    asr_delay_ms: int | None,
) -> int:
    chunks = microphone_chunks(
        device,
        chunk_ms=chunk_ms,
        duration=duration,
        save_wav=debug_wav,
    )
    await run_websocket_session(
        ready,
        chunks,
        locale=locale,
        on_event=lambda payload: render_event(payload, json_output=json_output),
        asr_audio_route=asr_audio_route,
        asr_model=asr_model,
        asr_delay_ms=asr_delay_ms,
    )
    if debug_wav is not None:
        print(f"saved debug WAV: {debug_wav}", flush=True)
    return 0
