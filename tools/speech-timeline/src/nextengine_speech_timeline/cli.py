from __future__ import annotations

import argparse
import asyncio
import json
from pathlib import Path
import sys
from typing import Sequence

from . import SERVICE_PROTOCOL
from .benchmark import (
    BenchmarkError,
    benchmark_service,
    load_benchmark_wav,
    write_report,
)
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
from .service import SpeechTimelineRuntime
from .transport_websocket import SpeechTimelineWebSocketService


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(prog="next-speech-timeline")
    root.add_argument("--version", action="version", version=SERVICE_PROTOCOL)
    commands = root.add_subparsers(dest="command", required=True)
    serve = commands.add_parser("serve", help="start the resident loopback service")
    serve.add_argument("--profile", type=Path, required=True)
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
    benchmark = commands.add_parser(
        "benchmark", help="stream an external WAV and write a content-free timing report"
    )
    benchmark.add_argument("--ready-file", type=Path, required=True)
    benchmark.add_argument("--audio", type=Path, required=True)
    benchmark.add_argument("--mode", choices=("paced", "unpaced"), default="paced")
    benchmark.add_argument("--runs", type=int, default=2)
    benchmark.add_argument("--chunk-ms", type=int, default=80)
    benchmark.add_argument("--out", type=Path, required=True)
    return root


def main(argv: Sequence[str] | None = None) -> int:
    arguments = parser().parse_args(argv)
    if arguments.command == "serve":
        try:
            profile = load_profile(arguments.profile)
            transcriber, affect = build_adapters(profile)
            runtime = SpeechTimelineRuntime(transcriber, affect)
            service = SpeechTimelineWebSocketService(
                runtime,
                ready_file=profile.service.ready_file,
                port=profile.service.port,
                bounds=profile.service.bounds,
                model_identity={
                    "voxtral_sha256": profile.voxtral.model_sha256,
                    "transcribe_revision": profile.voxtral.runtime_revision,
                    "emotion_model_id": profile.emotion.model_id,
                    "emotion_revision": profile.emotion.model_revision,
                    "emotion_classification": profile.emotion.classification,
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
    )
    if debug_wav is not None:
        print(f"saved debug WAV: {debug_wav}", flush=True)
    return 0
