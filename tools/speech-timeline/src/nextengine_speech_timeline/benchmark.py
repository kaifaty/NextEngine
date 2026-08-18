from __future__ import annotations

import asyncio
import json
import math
import os
from pathlib import Path
import tempfile
import time
import wave

from .microphone_client import ReadyInfo, run_websocket_session
from .profile import REPOSITORY_ROOT


class BenchmarkError(RuntimeError):
    pass


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


async def benchmark_service(
    ready: ReadyInfo,
    pcm: bytes,
    samples: int,
    *,
    mode: str,
    runs: int,
    chunk_ms: int,
) -> dict[str, object]:
    if mode not in {"paced", "unpaced"}:
        raise BenchmarkError("benchmark mode must be paced or unpaced")
    if not 1 <= runs <= 20:
        raise BenchmarkError("benchmark runs must be between 1 and 20")
    if not 20 <= chunk_ms <= 1_000:
        raise BenchmarkError("benchmark chunk-ms must be between 20 and 1000")
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
        if isinstance(jobs, list):
            for job in jobs:
                if not isinstance(job, dict):
                    continue
                queue_wait = job.get("queue_wait_ms")
                inference = job.get("inference_ms")
                if isinstance(queue_wait, (int, float)):
                    queue_waits.append(float(queue_wait))
                if isinstance(inference, (int, float)):
                    inference_times.append(float(inference))
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
        "run_configuration": {"chunk_ms": chunk_ms},
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
