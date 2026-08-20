from __future__ import annotations

import asyncio
from collections import Counter
from dataclasses import dataclass
import hashlib
import json
import math
import os
from pathlib import Path
import tempfile
import time
import wave
from typing import Any

from .microphone_client import ReadyInfo, run_websocket_session
from .profile import REPOSITORY_ROOT
from .protocol import ASR_AUDIO_ROUTES, ASR_AUDIO_ROUTE_RAW


class BenchmarkError(RuntimeError):
    pass


@dataclass(frozen=True)
class AffectCalibrationClip:
    """One externally stored, hashed clip used for a held-out evaluation."""

    clip_id: str
    source_emotion: str
    expected_label: str | None
    path: Path
    content_hash: str
    samples: int


@dataclass(frozen=True)
class AffectCalibrationManifest:
    path: Path
    content_hash: str
    source: dict[str, object]
    clips: tuple[AffectCalibrationClip, ...]


def load_benchmark_wav(path: Path) -> tuple[bytes, int]:
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
        raise BenchmarkError("benchmark WAV must be outside the repository")
    try:
        with wave.open(str(resolved), "rb") as source:
            metadata = (
                source.getframerate(),
                source.getnchannels(),
                source.getsampwidth(),
                source.getcomptype(),
            )
            frames = source.getnframes()
            if metadata != (16_000, 1, 2, "NONE"):
                raise BenchmarkError(
                    "benchmark WAV must be uncompressed mono signed-16 PCM at 16000 Hz"
                )
            if not 0 < frames <= 480_000:
                raise BenchmarkError("benchmark WAV must contain between 1 sample and 30 seconds")
            pcm = source.readframes(frames)
    except (OSError, EOFError, wave.Error) as error:
        raise BenchmarkError(f"cannot read benchmark WAV: {error}") from error
    if len(pcm) != frames * 2:
        raise BenchmarkError("benchmark WAV ended before its declared sample count")
    return pcm, frames


def load_affect_calibration_manifest(path: Path) -> AffectCalibrationManifest:
    """Load a bounded external manifest without importing audio into the repository.

    The evaluator accepts only the compact manual-calibration manifest format.
    Every referenced WAV is required to remain under the manifest directory and
    to match its pinned output hash before the service sees any audio.
    """

    resolved = _require_external_regular_file(path, "calibration manifest")
    raw = _read_bounded_json(resolved, "calibration manifest", maximum_bytes=512 * 1024)
    if raw.get("schema_version") != 1:
        raise BenchmarkError("calibration manifest schema_version must be 1")
    if raw.get("kind") != "nextengine.speech-timeline.manual-affect-calibration-set":
        raise BenchmarkError("unsupported calibration manifest kind")
    source = raw.get("source")
    if not isinstance(source, dict):
        raise BenchmarkError("calibration manifest source must be an object")
    clips_raw = raw.get("clips")
    if not isinstance(clips_raw, list) or not 1 <= len(clips_raw) <= 500:
        raise BenchmarkError("calibration manifest must contain between 1 and 500 clips")
    root = resolved.parent
    clips: list[AffectCalibrationClip] = []
    seen_ids: set[str] = set()
    for index, item in enumerate(clips_raw):
        if not isinstance(item, dict):
            raise BenchmarkError(f"calibration clip {index} must be an object")
        clip_id = _bounded_string(item.get("clip_id"), f"calibration clip {index} id", 128)
        if clip_id in seen_ids:
            raise BenchmarkError(f"calibration manifest has duplicate clip id: {clip_id}")
        seen_ids.add(clip_id)
        source_emotion = _bounded_string(
            item.get("source_emotion"), f"calibration clip {clip_id} source_emotion", 64
        )
        expected = item.get("expected_emotion2vec_label")
        if expected is not None:
            expected = _bounded_string(
                expected, f"calibration clip {clip_id} expected label", 64
            )
        relative_path = _bounded_string(
            item.get("relative_audio_path"), f"calibration clip {clip_id} audio path", 512
        )
        candidate = (root / relative_path).resolve()
        if not candidate.is_relative_to(root):
            raise BenchmarkError(f"calibration clip {clip_id} audio path escapes manifest directory")
        _require_external_regular_file(candidate, f"calibration clip {clip_id} audio")
        expected_hash = _sha256_value(
            item.get("normalized_audio_sha256"),
            f"calibration clip {clip_id} normalized_audio_sha256",
        )
        actual_hash = f"sha256:{_sha256_file(candidate)}"
        if actual_hash != expected_hash:
            raise BenchmarkError(f"calibration clip {clip_id} audio SHA-256 does not match manifest")
        _, samples = load_benchmark_wav(candidate)
        clips.append(
            AffectCalibrationClip(
                clip_id=clip_id,
                source_emotion=source_emotion,
                expected_label=expected,
                path=candidate,
                content_hash=actual_hash,
                samples=samples,
            )
        )
    return AffectCalibrationManifest(
        path=resolved,
        content_hash=f"sha256:{_sha256_file(resolved)}",
        source=dict(source),
        clips=tuple(clips),
    )


async def evaluate_affect_calibration(
    ready: ReadyInfo,
    manifest: AffectCalibrationManifest,
    *,
    mode: str,
    chunk_ms: int,
) -> dict[str, object]:
    """Replay held-out clips through the actual resident VAD/timeline path.

    This deliberately uses the public WebSocket client rather than calling the
    affect adapter directly. It therefore measures VAD admission, fixed sample
    clock windows, final smoothing, and the exact resident model lineage. No
    transcript or PCM is retained in the report.
    """

    if mode not in {"paced", "unpaced"}:
        raise BenchmarkError("calibration mode must be paced or unpaced")
    if not 20 <= chunk_ms <= 1_000:
        raise BenchmarkError("calibration chunk-ms must be between 20 and 1000")
    chunk_bytes = 16_000 * 2 * chunk_ms // 1_000
    rows: list[dict[str, object]] = []
    for index, clip in enumerate(manifest.clips, start=1):
        pcm, samples = load_benchmark_wav(clip.path)
        measurement: dict[str, float] = {}

        async def chunks() -> Any:
            measurement["capture_start_monotonic"] = time.monotonic()
            for offset in range(0, len(pcm), chunk_bytes):
                chunk = pcm[offset : offset + chunk_bytes]
                if mode == "paced":
                    await asyncio.sleep(len(chunk) / 2 / 16_000)
                yield chunk

        started = time.monotonic()
        try:
            final = await run_websocket_session(
                ready,
                chunks(),
                locale="ru",
                on_event=lambda _: None,
                session_id=f"affect-calibration-{index}",
                measurement=measurement,
            )
        except Exception as error:
            rows.append(
                {
                    "clip_id": clip.clip_id,
                    "source_emotion": clip.source_emotion,
                    "expected_label": clip.expected_label,
                    "audio": _clip_audio_payload(clip),
                    "status": "failed",
                    "error": _bounded_error(error),
                }
            )
            continue
        rows.append(
            _calibration_result_row(
                clip,
                final,
                wall_ms=(time.monotonic() - started) * 1000,
                finish_to_final_ms=_delta_ms(
                    measurement, "finish_send_monotonic", "final_monotonic"
                ),
            )
        )
    return {
        "schema_version": 1,
        "kind": "nextengine.speech-timeline.affect-calibration",
        "purpose": "held-out VAD/timeline behavior measurement; no model training, threshold fitting, PCM, or transcript retention",
        "mode": mode,
        "run_configuration": {"chunk_ms": chunk_ms},
        "input_manifest": {
            "path": str(manifest.path),
            "sha256": manifest.content_hash,
            "source": manifest.source,
            "clips_total": len(manifest.clips),
        },
        "service": {
            "protocol": ready.protocol,
            "models": ready.models or {},
            "model_identity": ready.model_identity or {},
        },
        "rows": rows,
        "summary": _calibration_summary(rows),
        "privacy": "audio_and_transcript_omitted",
    }


def _calibration_result_row(
    clip: AffectCalibrationClip,
    final: dict[str, object],
    *,
    wall_ms: float,
    finish_to_final_ms: float | None,
) -> dict[str, object]:
    metrics = final.get("metrics")
    metrics = metrics if isinstance(metrics, dict) else {}
    activity = metrics.get("vocal_activity")
    activity = activity if isinstance(activity, dict) else {}
    affect = metrics.get("vocal_affect")
    affect = affect if isinstance(affect, dict) else {}
    final_summary = final.get("vocal_expression_summary")
    final_summary = final_summary if isinstance(final_summary, dict) else {}
    final_label = _string_or_default(final.get("observed_vocal_expression"), "unknown")
    return {
        "clip_id": clip.clip_id,
        "source_emotion": clip.source_emotion,
        "expected_label": clip.expected_label,
        "audio": _clip_audio_payload(clip),
        "status": "complete",
        "result": {
            "speech_samples": _non_negative_int(activity.get("speech_samples")),
            "speech_ratio": _finite_float_or_none(activity.get("speech_ratio")),
            "raw_observation_count": _non_negative_int(affect.get("raw_observation_count")),
            "confirmed_segment_count": _non_negative_int(
                final_summary.get("confirmed_segment_count")
            ),
            "final_label": final_label,
            "final_label_source": _string_or_default(
                final.get("observed_vocal_expression_source"),
                "unreported",
            ),
            "evidence_samples": _non_negative_int(final_summary.get("evidence_samples")),
            "abstained": final_label in {"unknown", "other"},
            "exact_top1_match": (
                None if clip.expected_label is None else final_label == clip.expected_label
            ),
        },
        "timing": {
            "wall_ms": round(wall_ms, 3),
            "finish_to_final_ms": (
                None if finish_to_final_ms is None else round(finish_to_final_ms, 3)
            ),
        },
    }


def _calibration_summary(rows: list[dict[str, object]]) -> dict[str, object]:
    complete = [item for item in rows if item.get("status") == "complete"]
    failed = [item for item in rows if item.get("status") == "failed"]
    final_labels: Counter[str] = Counter()
    expected_rows: dict[str, list[dict[str, object]]] = {}
    source_rows: dict[str, list[dict[str, object]]] = {}
    speech_admitted = 0
    affect_observed = 0
    abstained = 0
    wall_ms: list[float] = []
    finish_to_final_ms: list[float] = []
    for row in complete:
        source = _string_or_default(row.get("source_emotion"), "unknown")
        source_rows.setdefault(source, []).append(row)
        expected = row.get("expected_label")
        if isinstance(expected, str):
            expected_rows.setdefault(expected, []).append(row)
        result = row.get("result")
        result = result if isinstance(result, dict) else {}
        label = _string_or_default(result.get("final_label"), "unknown")
        final_labels[label] += 1
        if _non_negative_int(result.get("speech_samples")) > 0:
            speech_admitted += 1
        if _non_negative_int(result.get("raw_observation_count")) > 0:
            affect_observed += 1
        if result.get("abstained") is True:
            abstained += 1
        timing = row.get("timing")
        timing = timing if isinstance(timing, dict) else {}
        wall = _finite_float_or_none(timing.get("wall_ms"))
        finish = _finite_float_or_none(timing.get("finish_to_final_ms"))
        if wall is not None:
            wall_ms.append(wall)
        if finish is not None:
            finish_to_final_ms.append(finish)
    per_expected = {}
    scored_total = 0
    scored_matches = 0
    for label in sorted(expected_rows):
        label_rows = expected_rows[label]
        matches = sum(
            item.get("result", {}).get("exact_top1_match") is True
            for item in label_rows
            if isinstance(item.get("result"), dict)
        )
        scored_total += len(label_rows)
        scored_matches += matches
        per_expected[label] = {
            "clips_complete": len(label_rows),
            "speech_admitted": sum(
                _non_negative_int(item.get("result", {}).get("speech_samples")) > 0
                for item in label_rows
                if isinstance(item.get("result"), dict)
            ),
            "affect_observed": sum(
                _non_negative_int(item.get("result", {}).get("raw_observation_count")) > 0
                for item in label_rows
                if isinstance(item.get("result"), dict)
            ),
            "final_abstentions": sum(
                item.get("result", {}).get("abstained") is True
                for item in label_rows
                if isinstance(item.get("result"), dict)
            ),
            "exact_top1_matches": matches,
            "exact_top1_accuracy": round(matches / len(label_rows), 6),
        }
    return {
        "clips_total": len(rows),
        "clips_complete": len(complete),
        "clips_failed": len(failed),
        "speech_admitted_clips": speech_admitted,
        "affect_observed_clips": affect_observed,
        "final_abstention_clips": abstained,
        "final_label_counts": dict(sorted(final_labels.items())),
        "scored_clips_complete": scored_total,
        "exact_top1_matches": scored_matches,
        "exact_top1_accuracy": (
            None if scored_total == 0 else round(scored_matches / scored_total, 6)
        ),
        "per_expected_label": per_expected,
        "per_source_emotion_final_labels": {
            source: dict(
                sorted(
                    Counter(
                        _string_or_default(
                            item.get("result", {}).get("final_label"), "unknown"
                        )
                        for item in source_rows[source]
                        if isinstance(item.get("result"), dict)
                    ).items()
                )
            )
            for source in sorted(source_rows)
        },
        "wall_ms": _percentiles(wall_ms),
        "finish_to_final_ms": _percentiles(finish_to_final_ms),
    }


def _clip_audio_payload(clip: AffectCalibrationClip) -> dict[str, object]:
    return {
        "content_hash": clip.content_hash,
        "sample_rate_hz": 16_000,
        "channels": 1,
        "samples": clip.samples,
        "duration_ms": round(clip.samples * 1_000 / 16_000),
    }


def _read_bounded_json(path: Path, name: str, *, maximum_bytes: int) -> dict[str, object]:
    try:
        raw = path.read_bytes()
    except OSError as error:
        raise BenchmarkError(f"cannot read {name}: {error}") from error
    if len(raw) > maximum_bytes:
        raise BenchmarkError(f"{name} exceeds {maximum_bytes} bytes")
    try:
        value = json.loads(raw)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise BenchmarkError(f"invalid {name} JSON: {error}") from error
    if not isinstance(value, dict):
        raise BenchmarkError(f"{name} must be a JSON object")
    return value


def _require_external_regular_file(path: Path, name: str) -> Path:
    resolved = path.expanduser().resolve()
    if resolved == REPOSITORY_ROOT or resolved.is_relative_to(REPOSITORY_ROOT):
        raise BenchmarkError(f"{name} must be outside the repository")
    try:
        metadata = resolved.lstat()
    except OSError as error:
        raise BenchmarkError(f"cannot inspect {name}: {error}") from error
    if not resolved.is_file() or metadata.st_mode & 0o170000 != 0o100000:
        raise BenchmarkError(f"{name} must be a regular file")
    return resolved


def _bounded_string(value: object, name: str, maximum_bytes: int) -> str:
    if not isinstance(value, str) or not value or len(value.encode("utf-8")) > maximum_bytes:
        raise BenchmarkError(f"{name} must be a bounded non-empty string")
    return value


def _sha256_value(value: object, name: str) -> str:
    result = _bounded_string(value, name, 71)
    if len(result) != 71 or not result.startswith("sha256:"):
        raise BenchmarkError(f"{name} must be sha256:<64 lowercase hex>")
    try:
        int(result[7:], 16)
    except ValueError as error:
        raise BenchmarkError(f"{name} must contain hexadecimal digits") from error
    if result.lower() != result:
        raise BenchmarkError(f"{name} must use lowercase hexadecimal digits")
    return result


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _non_negative_int(value: object) -> int:
    return value if isinstance(value, int) and not isinstance(value, bool) and value >= 0 else 0


def _finite_float_or_none(value: object) -> float | None:
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        numeric = float(value)
        if math.isfinite(numeric):
            return numeric
    return None


def _string_or_default(value: object, default: str) -> str:
    return value if isinstance(value, str) and value else default


def _bounded_error(error: Exception) -> str:
    return f"{type(error).__name__}: {str(error)[:256]}"


async def benchmark_service(
    ready: ReadyInfo,
    pcm: bytes,
    samples: int,
    *,
    mode: str,
    runs: int,
    chunk_ms: int,
    asr_audio_route: str = ASR_AUDIO_ROUTE_RAW,
    asr_model: str | None = None,
    asr_delay_ms: int | None = None,
) -> dict[str, object]:
    if mode not in {"paced", "unpaced"}:
        raise BenchmarkError("benchmark mode must be paced or unpaced")
    if not 1 <= runs <= 20:
        raise BenchmarkError("benchmark runs must be between 1 and 20")
    if not 20 <= chunk_ms <= 1_000:
        raise BenchmarkError("benchmark chunk-ms must be between 20 and 1000")
    if asr_audio_route not in ASR_AUDIO_ROUTES:
        raise BenchmarkError(
            f"benchmark ASR audio route must be one of {sorted(ASR_AUDIO_ROUTES)}"
        )
    chunk_bytes = 16_000 * 2 * chunk_ms // 1_000
    audio_seconds = samples / 16_000
    run_reports = []
    queue_waits: list[float] = []
    inference_times: list[float] = []
    finish_latencies: list[float] = []
    first_update_latencies: list[float] = []
    capture_to_first_update_latencies: list[float] = []
    capture_to_first_transcript_latencies: list[float] = []
    capture_to_first_affect_latencies: list[float] = []
    wall_times: list[float] = []
    model_busy_rtfs: list[float] = []

    for index in range(runs):
        measurement: dict[str, float] = {}

        async def chunks():
            measurement["capture_start_monotonic"] = time.monotonic()
            for offset in range(0, len(pcm), chunk_bytes):
                chunk = pcm[offset : offset + chunk_bytes]
                if mode == "paced":
                    await asyncio.sleep(len(chunk) / 2 / 16_000)
                yield chunk

        started = time.monotonic()
        final = await run_websocket_session(
            ready,
            chunks(),
            locale="ru",
            on_event=lambda _: None,
            session_id=f"benchmark-{index + 1}",
            measurement=measurement,
            asr_audio_route=asr_audio_route,
            asr_model=asr_model,
            asr_delay_ms=asr_delay_ms,
        )
        wall_ms = (time.monotonic() - started) * 1000
        wall_times.append(wall_ms)
        finish_ms = _delta_ms(measurement, "finish_send_monotonic", "final_monotonic")
        first_ms = _delta_ms(
            measurement, "first_chunk_send_monotonic", "first_update_monotonic"
        )
        capture_first_ms = _delta_ms(
            measurement, "capture_start_monotonic", "first_update_monotonic"
        )
        capture_transcript_ms = _delta_ms(
            measurement,
            "capture_start_monotonic",
            "first_transcript_update_monotonic",
        )
        capture_affect_ms = _delta_ms(
            measurement,
            "capture_start_monotonic",
            "first_affect_update_monotonic",
        )
        if finish_ms is not None:
            finish_latencies.append(finish_ms)
        if first_ms is not None:
            first_update_latencies.append(first_ms)
        if capture_first_ms is not None:
            capture_to_first_update_latencies.append(capture_first_ms)
        if capture_transcript_ms is not None:
            capture_to_first_transcript_latencies.append(capture_transcript_ms)
        if capture_affect_ms is not None:
            capture_to_first_affect_latencies.append(capture_affect_ms)
        metrics = final.get("metrics")
        jobs = metrics.get("jobs", []) if isinstance(metrics, dict) else []
        jobs_truncated = isinstance(metrics, dict) and metrics.get("jobs_truncated") is True
        if isinstance(jobs, list) and not jobs_truncated:
            for job in jobs:
                if not isinstance(job, dict):
                    continue
                queue_wait = job.get("queue_wait_ms")
                inference = job.get("inference_ms")
                if isinstance(queue_wait, (int, float)):
                    queue_waits.append(float(queue_wait))
                if isinstance(inference, (int, float)):
                    inference_times.append(float(inference))
        if jobs_truncated and isinstance(metrics, dict):
            run_inference_ms = 0.0
            summary = metrics.get("job_summary")
            if isinstance(summary, dict):
                for item in summary.values():
                    if not isinstance(item, dict):
                        continue
                    total = item.get("inference_total_ms")
                    if isinstance(total, (int, float)):
                        run_inference_ms += float(total)
                    for key in ("queue_wait_p50_ms", "queue_wait_p95_ms"):
                        value = item.get(key)
                        if isinstance(value, (int, float)):
                            queue_waits.append(float(value))
                    for key in ("inference_p50_ms", "inference_p95_ms"):
                        value = item.get(key)
                        if isinstance(value, (int, float)):
                            inference_times.append(float(value))
        else:
            run_inference_ms = sum(
                float(job.get("inference_ms", 0))
                for job in jobs
                if isinstance(job, dict) and isinstance(job.get("inference_ms"), (int, float))
            )
        model_busy_rtf = run_inference_ms / 1000 / audio_seconds
        model_busy_rtfs.append(model_busy_rtf)
        run_reports.append(
            {
                "run": index + 1,
                "wall_ms": round(wall_ms, 3),
                "end_to_end_rtf": round(wall_ms / 1000 / audio_seconds, 6),
                "model_worker_busy_rtf": round(model_busy_rtf, 6),
                "finish_to_final_ms": None if finish_ms is None else round(finish_ms, 3),
                "first_chunk_to_first_update_ms": None if first_ms is None else round(first_ms, 3),
                "capture_start_to_first_update_ms": (
                    None if capture_first_ms is None else round(capture_first_ms, 3)
                ),
                "capture_start_to_first_transcript_ms": (
                    None
                    if capture_transcript_ms is None
                    else round(capture_transcript_ms, 3)
                ),
                "capture_start_to_first_affect_ms": (
                    None if capture_affect_ms is None else round(capture_affect_ms, 3)
                ),
                "metrics": metrics if isinstance(metrics, dict) else {},
            }
        )
    return {
        "schema_version": 1,
        "kind": "nextengine.speech-timeline.benchmark",
        "mode": mode,
        "audio": {
            "sample_rate_hz": 16_000,
            "channels": 1,
            "encoding": "pcm_s16le",
            "samples": samples,
            "duration_ms": round(audio_seconds * 1000),
        },
        "run_configuration": {
            "chunk_ms": chunk_ms,
            "asr_audio_route": asr_audio_route,
            "asr_model": asr_model or "service_default",
            "asr_delay_ms": asr_delay_ms,
        },
        "service": {
            "protocol": ready.protocol,
            "models": ready.models or {},
            "model_identity": ready.model_identity or {},
        },
        "runs": run_reports,
        "summary": {
            "wall_ms": _percentiles(wall_times),
            "end_to_end_rtf_p95": round(_percentile(wall_times, 95) / 1000 / audio_seconds, 6),
            "model_worker_busy_rtf_p95": round(_percentile(model_busy_rtfs, 95), 6),
            "finish_to_final_ms": _percentiles(finish_latencies),
            "first_chunk_to_first_update_ms": _percentiles(first_update_latencies),
            "capture_start_to_first_update_ms": _percentiles(
                capture_to_first_update_latencies
            ),
            "capture_start_to_first_transcript_ms": _percentiles(
                capture_to_first_transcript_latencies
            ),
            "capture_start_to_first_affect_ms": _percentiles(
                capture_to_first_affect_latencies
            ),
            "model_queue_wait_ms": _percentiles(queue_waits),
            "model_inference_ms": _percentiles(inference_times),
        },
        "privacy": "audio_and_transcript_omitted",
    }


def write_report(path: Path, report: dict[str, object]) -> None:
    destination = path.expanduser().resolve()
    if destination == REPOSITORY_ROOT or destination.is_relative_to(REPOSITORY_ROOT):
        raise BenchmarkError("benchmark report must be outside the repository")
    if not destination.parent.is_dir():
        raise BenchmarkError("benchmark report parent directory does not exist")
    encoded = json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True).encode("utf-8")
    descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{destination.name}.", suffix=".tmp", dir=destination.parent
    )
    temporary = Path(temporary_name)
    try:
        os.fchmod(descriptor, 0o600)
        with os.fdopen(descriptor, "wb") as stream:
            stream.write(encoded)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, destination)
    except BaseException:
        temporary.unlink(missing_ok=True)
        raise


def _delta_ms(values: dict[str, float], start: str, end: str) -> float | None:
    if start not in values or end not in values:
        return None
    return (values[end] - values[start]) * 1000


def _percentiles(values: list[float]) -> dict[str, float | None]:
    if not values:
        return {"p50": None, "p95": None}
    return {
        "p50": round(_percentile(values, 50), 3),
        "p95": round(_percentile(values, 95), 3),
    }


def _percentile(values: list[float], percentile: int) -> float:
    if not values:
        return math.nan
    ordered = sorted(values)
    index = max(0, math.ceil(percentile / 100 * len(ordered)) - 1)
    return ordered[index]
