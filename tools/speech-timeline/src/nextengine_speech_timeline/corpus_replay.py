"""Replay a hash-closed prepared reliability corpus through the resident path.

This is the R1 replay runner of ``DEV-SPEECH-RELIABILITY-001``.  It reads an
explicitly prepared external index only, verifies every clip hash against its
pinned SHA-256 and WAV contract, streams each clip exactly once through the
production-shaped resident WebSocket service using the real 16 kHz / 80 ms
ingress parameters plus an explicit finish, and writes typed results,
bounded revision traces and the service-side feature payloads to an explicit
external directory.

Fail-closed rules:

- The recipe must be fully hash-closed before any audio is touched.
- The live service must advertise exactly the ASR adapter identity pinned by
  the recipe for the selected model; a mismatch aborts before the first clip.
- Each clip gets exactly one attempt.  Overload, turn overflow, timeout,
  no-speech, speech-but-empty and technical failures are recorded as typed
  outcomes; nothing is retried to green.
- Audio, transcripts, traces, features and reports stay outside this
  repository.  Output files are replaced atomically with private (0600)
  permissions.  The run summary carries hashes, identities and counts but no
  transcript or audio content.
"""

from __future__ import annotations

import asyncio
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import time
from typing import Callable

import wave

from .microphone_client import ClientError, ReadyInfo, run_websocket_session
from .reliability import (
    DatasetRecipe,
    ReliabilityCorpusError,
    _atomic_write,
    _canonical_json,
    _external_directory,
    load_dataset_recipe,
    score_transcript,
    verify_indexed_audio,
)


RESULT_KIND = "nextengine.speech-reliability.replay-result"
REPORT_KIND = "nextengine.speech-reliability.replay-report"
TRACE_KIND = "speech-reliability.transcript-revision-trace-v0"

MAX_PREPARED_INDEX_BYTES = 256 * 1024 * 1024
MAX_REPLAY_ROWS = 10_000
MAX_TRACE_REVISIONS_PER_UTTERANCE = 512
DEFAULT_TIMEOUT_SECONDS = 120.0

OUTCOME_COMPLETED = "completed"
OUTCOME_NO_SPEECH = "no_speech"
OUTCOME_SPEECH_BUT_EMPTY = "speech_but_empty"
OUTCOME_OVERLOAD = "scheduler_overload"
OUTCOME_TURN_TOO_LARGE = "turn_too_large"
OUTCOME_TIMEOUT = "timeout"
OUTCOME_TECHNICAL_FAILURE = "technical_failure"


class ReliabilityReplayError(RuntimeError):
    """A stable fail-closed corpus replay error."""


@dataclass(frozen=True)
class ReplayRow:
    source_id: str
    clip_id: str
    split: str
    audio_sha256: str
    samples: int
    normalized_reference: str
    pcm: bytes


def read_prepared_rows(prepared_index_path: Path) -> tuple[list[dict[str, object]], str]:
    """Read the prepared JSONL index and return speech rows plus file hash."""

    resolved = prepared_index_path.expanduser().resolve()
    try:
        raw = resolved.read_bytes()
    except OSError as error:
        raise ReliabilityReplayError(f"cannot read prepared index: {error}") from error
    if not 0 < len(raw) <= MAX_PREPARED_INDEX_BYTES:
        raise ReliabilityReplayError(
            f"prepared index size must be between 1 and {MAX_PREPARED_INDEX_BYTES}"
        )
    digest = hashlib.sha256(raw).hexdigest()
    rows: list[dict[str, object]] = []
    for line_number, line in enumerate(raw.decode("utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ReliabilityReplayError(
                f"cannot decode prepared index line {line_number}: {error}"
            ) from error
        if not isinstance(value, dict):
            raise ReliabilityReplayError(
                f"prepared index line {line_number} must be an object"
            )
        if value.get("entry_kind") != "speech":
            # Augmentation assets share the prepared index but are never
            # replayed as utterances.
            continue
        samples = value.get("samples")
        if (
            not isinstance(value.get("clip_id"), str)
            or not isinstance(value.get("source_id"), str)
            or not isinstance(value.get("split"), str)
            or not isinstance(value.get("audio_sha256"), str)
            or not isinstance(value.get("normalized_reference"), str)
            or not isinstance(value.get("relative_audio_path"), str)
            or isinstance(samples, bool)
            or not isinstance(samples, int)
        ):
            raise ReliabilityReplayError(
                f"prepared index line {line_number} is not a speech entry "
                "with the expected fields"
            )
        rows.append(value)
    if not rows:
        raise ReliabilityReplayError("prepared index contains no speech rows")
    return rows, f"sha256:{digest}"


def load_closed_recipe(manifest_path: Path) -> DatasetRecipe:
    """Load a dataset recipe and refuse anything that is not hash closed."""

    recipe = load_dataset_recipe(manifest_path)
    if recipe.replay_profile["identity_status"] != "closed":
        raise ReliabilityReplayError(
            "REPLAY_IDENTITY_NOT_HASH_CLOSED",
            "recipe replay identity_status must be closed before replay",
        )
    unclosed = [
        source.source_id
        for source in recipe.sources
        if source.status != "closed" and source.rights.admission != "excluded"
    ]
    if unclosed:
        raise ReliabilityReplayError(
            "SOURCE_NOT_HASH_CLOSED",
            "admitted sources are not hash closed: " + ", ".join(sorted(unclosed)),
        )
    return recipe


def service_adapter_id(ready: ReadyInfo, asr_model: str) -> str:
    transcribers = (ready.models or {}).get("transcribers")
    capabilities = (
        transcribers.get(asr_model) if isinstance(transcribers, dict) else None
    )
    adapter_id = (
        capabilities.get("adapter_id") if isinstance(capabilities, dict) else None
    )
    if not isinstance(adapter_id, str) or not adapter_id:
        raise ReliabilityReplayError(
            "REPLAY_IDENTITY_MISMATCH",
            f"ready file does not advertise an adapter identity for {asr_model}",
        )
    return adapter_id


def check_service_identity(ready: ReadyInfo, recipe: DatasetRecipe, *, asr_model: str) -> None:
    """Abort before replay when live ASR identity differs from the recipe."""

    actual = service_adapter_id(ready, asr_model)
    pinned = str(recipe.replay_profile["asr_adapter_id"])
    if actual != pinned:
        raise ReliabilityReplayError(
            "REPLAY_IDENTITY_MISMATCH",
            f"recipe pins adapter {pinned!r} but the service advertises "
            f"{actual!r} for {asr_model}",
        )


def classify_outcome(*, final_text: str, speech_samples: int) -> str:
    if final_text.strip():
        return OUTCOME_COMPLETED
    if speech_samples > 0:
        return OUTCOME_SPEECH_BUT_EMPTY
    return OUTCOME_NO_SPEECH


def outcome_from_error_code(code: str) -> str:
    if code == "SERVICE_OVERLOADED":
        return OUTCOME_OVERLOAD
    if code == "TURN_TOO_LARGE":
        return OUTCOME_TURN_TOO_LARGE
    return OUTCOME_TECHNICAL_FAILURE


class _RevisionTraceCollector:
    """Bounded per-utterance collector of transcript revision metadata.

    ``observed_audio_end_sample`` approximates the delivered sample cursor as
    the number of PCM samples handed to the WebSocket client so far; it is a
    delivery marker, never an acoustic word timestamp.
    """

    def __init__(self) -> None:
        self.started_monotonic = time.monotonic()
        self.samples_sent = 0
        self.rows: list[dict[str, object]] = []
        self.truncated_revisions = 0

    def on_event(self, payload: dict[str, object]) -> None:
        if payload.get("type") != "speech_timeline.update":
            return
        transcript = payload.get("transcript")
        if not isinstance(transcript, dict):
            return
        text = transcript.get("text")
        stable_prefix = transcript.get("stable_prefix")
        final = transcript.get("final")
        if not isinstance(text, str) or not isinstance(stable_prefix, str):
            return
        if len(self.rows) >= MAX_TRACE_REVISIONS_PER_UTTERANCE:
            self.truncated_revisions += 1
            return
        self.rows.append(
            {
                "kind": TRACE_KIND,
                "schema_version": 0,
                "revision_index": len(self.rows) + 1,
                "observed_audio_end_sample": self.samples_sent,
                "service_elapsed_ms": round(
                    (time.monotonic() - self.started_monotonic) * 1000, 3
                ),
                "final": bool(final),
                "stable_prefix": stable_prefix[:4_096],
                "text": text[:4_096],
            }
        )


async def replay_prepared_clip(
    ready: ReadyInfo,
    row: ReplayRow,
    *,
    chunk_ms: int,
    paced: bool,
    asr_model: str,
    asr_delay_ms: int | None,
    asr_audio_route: str,
    timeout_seconds: float,
) -> tuple[dict[str, object], list[dict[str, object]]]:
    """Stream one verified clip through the resident path exactly once."""

    chunk_bytes = 16_000 * 2 * chunk_ms // 1_000
    measurement: dict[str, float] = {}
    trace = _RevisionTraceCollector()
    outcome: str | None = None
    error_code: str | None = None
    detail: str | None = None
    final: dict[str, object] | None = None

    async def chunks():
        for offset in range(0, len(row.pcm), chunk_bytes):
            chunk = row.pcm[offset : offset + chunk_bytes]
            trace.samples_sent += len(chunk) // 2
            yield chunk
            if paced:
                await asyncio.sleep(len(chunk) / 2 / 16_000)

    started = time.monotonic()
    try:
        final = await asyncio.wait_for(
            run_websocket_session(
                ready,
                chunks(),
                locale="ru",
                on_event=trace.on_event,
                session_id=f"reliability-{row.clip_id}",
                measurement=measurement,
                asr_model=asr_model,
                asr_delay_ms=asr_delay_ms,
                asr_audio_route=asr_audio_route,
                retain_diagnostic_audio=False,
            ),
            timeout=timeout_seconds,
        )
    except asyncio.TimeoutError:
        outcome = OUTCOME_TIMEOUT
        detail = f"replay exceeded {timeout_seconds:g}s"
    except ClientError as error:
        message = str(error)
        detail = f"{type(error).__name__}: {message[:256]}"
        if message.startswith("service failed: "):
            code = message.removeprefix("service failed: ").strip()
            error_code = code
            outcome = outcome_from_error_code(code)
        else:
            outcome = OUTCOME_TECHNICAL_FAILURE
    except Exception as error:
        outcome = OUTCOME_TECHNICAL_FAILURE
        error_code = None
        detail = f"{type(error).__name__}: {str(error)[:256]}"

    wall_ms = round((time.monotonic() - started) * 1000, 3)
    result: dict[str, object] = {
        "kind": RESULT_KIND,
        "schema_version": 0,
        "clip_id": row.clip_id,
        "source_id": row.source_id,
        "split": row.split,
        "audio_sha256": row.audio_sha256,
        "samples": row.samples,
        "duration_ms": round(row.samples * 1_000 / 16_000),
        "outcome": outcome,
        "error_code": error_code,
        "detail": detail,
        "timing": {"wall_ms": wall_ms},
        "scores": None,
        "final_text_empty": None,
        "speech_samples": None,
        "feature_payload": None,
        "trace_revisions": len(trace.rows),
        "trace_truncated_revisions": trace.truncated_revisions,
    }
    if final is None:
        return result, trace.rows

    metrics = final.get("metrics")
    metrics = metrics if isinstance(metrics, dict) else {}
    activity = metrics.get("vocal_activity")
    activity = activity if isinstance(activity, dict) else {}
    speech_value = activity.get("speech_samples")
    speech_samples = (
        speech_value
        if isinstance(speech_value, int) and not isinstance(speech_value, bool)
        else 0
    )
    final_text = final.get("text")
    final_text = final_text if isinstance(final_text, str) else ""
    outcome = classify_outcome(final_text=final_text, speech_samples=speech_samples)
    scores = None
    if outcome == OUTCOME_COMPLETED:
        try:
            scores = score_transcript(row.normalized_reference, final_text).as_dict()
        except ReliabilityCorpusError as error:
            scores = {
                "error": f"SCORE_REJECTED: {str(error)[:128]}",
                "exact_match": False,
                "wer": None,
                "cer": None,
            }
    finish_sent = measurement.get("finish_send_monotonic")
    final_at = measurement.get("final_monotonic")
    finish_to_final_ms = (
        round((final_at - finish_sent) * 1000, 3)
        if finish_sent is not None and final_at is not None
        else None
    )
    result.update(
        {
            "outcome": outcome,
            "error_code": None,
            "detail": None,
            "timing": {
                "wall_ms": wall_ms,
                "finish_to_final_ms": finish_to_final_ms,
            },
            "scores": scores,
            "final_text_empty": final_text == "",
            "speech_samples": speech_samples,
            "feature_payload": metrics.get("recognition_reliability_features"),
        }
    )
    return result, trace.rows


async def replay_corpus(
    ready: ReadyInfo,
    *,
    manifest_path: Path,
    store: Path,
    prepared_index_path: Path,
    out_dir: Path,
    asr_model: str,
    asr_audio_route: str,
    asr_delay_ms: int | None = None,
    mode: str = "paced",
    chunk_ms: int = 80,
    splits: tuple[str, ...] | None = None,
    limit: int | None = None,
    timeout_seconds: float = DEFAULT_TIMEOUT_SECONDS,
    log: Callable[[str], None] | None = None,
) -> dict[str, object]:
    """Run one deterministic single-route pass over the prepared index."""

    if mode not in {"paced", "unpaced"}:
        raise ReliabilityReplayError("mode must be paced or unpaced")
    if chunk_ms != 80:
        raise ReliabilityReplayError("corpus replay requires the production 80 ms chunk")
    if not asr_model:
        raise ReliabilityReplayError(
            "replay requires one exact --asr-model for the whole run"
        )
    if timeout_seconds <= 0:
        raise ReliabilityReplayError("timeout must be positive")

    store_root = _external_directory(store, "corpus store")
    recipe = load_closed_recipe(manifest_path)
    check_service_identity(ready, recipe, asr_model=asr_model)
    rows, prepared_hash = read_prepared_rows(prepared_index_path)
    selected = [
        row
        for row in rows
        if splits is None or row["split"] in splits
    ]
    if not selected:
        raise ReliabilityReplayError("no prepared rows match the requested splits")
    if limit is not None:
        if limit <= 0:
            raise ReliabilityReplayError("limit must be positive")
        selected = selected[:limit]
    if len(selected) > MAX_REPLAY_ROWS:
        raise ReliabilityReplayError(
            f"replay exceeds {MAX_REPLAY_ROWS} rows; narrow the selection"
        )

    results: list[dict[str, object]] = []
    traces: list[dict[str, object]] = []
    features: list[dict[str, object]] = []
    outcome_counts: dict[str, int] = {}
    for index, prepared_row in enumerate(selected, start=1):
        clip_id = str(prepared_row["clip_id"])
        source_id = str(prepared_row["source_id"])
        try:
            relative_path, audio_hash, samples = verify_indexed_audio(
                store_root, prepared_row, source_id, clip_id
            )
        except ReliabilityCorpusError as error:
            raise ReliabilityReplayError(str(error)) from error
        if audio_hash != str(prepared_row["audio_sha256"]) or samples != int(
            prepared_row["samples"]
        ):
            raise ReliabilityReplayError(
                f"prepared row drifted from verified audio: {source_id}/{clip_id}"
            )
        audio_path = store_root / relative_path
        try:
            with wave.open(str(audio_path), "rb") as source:
                pcm = source.readframes(samples)
        except (OSError, EOFError, wave.Error) as error:
            raise ReliabilityReplayError(
                f"cannot read verified audio {source_id}/{clip_id}: {error}"
            ) from error
        row = ReplayRow(
            source_id=source_id,
            clip_id=clip_id,
            split=str(prepared_row["split"]),
            audio_sha256=audio_hash,
            samples=samples,
            normalized_reference=str(prepared_row["normalized_reference"]),
            pcm=pcm,
        )
        if log is not None:
            log(f"[{index}/{len(selected)}] {source_id}/{clip_id}")
        result, trace_rows = await replay_prepared_clip(
            ready,
            row,
            chunk_ms=chunk_ms,
            paced=(mode == "paced"),
            asr_model=asr_model,
            asr_delay_ms=asr_delay_ms,
            asr_audio_route=asr_audio_route,
            timeout_seconds=timeout_seconds,
        )
        outcome = str(result["outcome"])
        outcome_counts[outcome] = outcome_counts.get(outcome, 0) + 1
        feature_payload = result.get("feature_payload")
        results.append(result)
        if isinstance(feature_payload, dict):
            features.append(feature_payload)
        for trace_row in trace_rows:
            enriched = dict(trace_row)
            enriched["clip_id"] = clip_id
            enriched["source_id"] = source_id
            traces.append(enriched)

    completed = sum(
        1 for item in results if item["outcome"] == OUTCOME_COMPLETED
    )
    exact_matches = sum(
        1
        for item in results
        if isinstance(item.get("scores"), dict)
        and item["scores"].get("exact_match") is True
    )
    report: dict[str, object] = {
        "schema_version": 0,
        "kind": REPORT_KIND,
        "status": "complete",
        "dataset_id": recipe.dataset_id,
        "dataset_revision": recipe.dataset_revision,
        "calibration_domain": recipe.calibration_domain,
        "recipe_hash": recipe.manifest_hash,
        "prepared_index_sha256": prepared_hash,
        "normalizer_profile": "ru-asr-normalize-v0",
        "run_configuration": {
            "mode": mode,
            "chunk_ms": chunk_ms,
            "sample_rate_hz": 16_000,
            "encoding": "pcm_s16le",
            "channels": 1,
            "explicit_finish": True,
            "retain_diagnostic_audio": False,
            "single_attempt_per_clip": True,
            "splits": sorted(splits) if splits else ["all"],
            "requested_limit": limit,
            "timeout_seconds": timeout_seconds,
        },
        "requested_identity": {
            "asr_adapter_id": str(recipe.replay_profile["asr_adapter_id"]),
            "model_id": str(recipe.replay_profile["model_id"]),
            "model_revision": str(recipe.replay_profile["model_revision"]),
            "runtime_revision": str(recipe.replay_profile["runtime_revision"]),
            "model_delay_ms": recipe.replay_profile["model_delay_ms"],
            "partial_decode_interval_ms": recipe.replay_profile[
                "partial_decode_interval_ms"
            ],
            "asr_audio_route": asr_audio_route,
            "selected_asr_model": asr_model,
        },
        "service": {
            "protocol": ready.protocol,
            "models": ready.models or {},
            "model_identity": ready.model_identity or {},
        },
        "counts": {
            "rows_selected": len(selected),
            "completed": completed,
            "exact_matches": exact_matches,
            "by_outcome": dict(sorted(outcome_counts.items())),
        },
        "privacy": "transcripts_confined_to_external_results_trace_and_features",
        "outputs": "explicit_external_directory",
    }
    written = write_replay_outputs(
        out_dir,
        report=report,
        results=results,
        traces=traces,
        features=features,
    )
    report["written_files"] = written
    return report


def write_replay_outputs(
    out_dir: Path,
    *,
    report: dict[str, object],
    results: list[dict[str, object]],
    traces: list[dict[str, object]],
    features: list[dict[str, object]],
) -> dict[str, str]:
    """Atomically publish private output files into the external directory."""

    directory = _external_directory(out_dir, "replay output directory")
    payloads = {
        "results.jsonl": b"".join(_canonical_json(row) + b"\n" for row in results),
        "features.jsonl": b"".join(_canonical_json(row) + b"\n" for row in features),
        "trace.jsonl": b"".join(_canonical_json(row) + b"\n" for row in traces),
        "report.json": json.dumps(
            report, ensure_ascii=False, indent=2, sort_keys=True
        ).encode("utf-8"),
    }
    written: dict[str, str] = {}
    for name, data in payloads.items():
        destination = directory / name
        _atomic_write(destination, data)
        written[name] = str(destination)
    return written
