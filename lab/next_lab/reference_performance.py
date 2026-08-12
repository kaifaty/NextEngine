from __future__ import annotations

import math
import os
import statistics
import subprocess
import threading
from dataclasses import dataclass
from time import monotonic_ns
from typing import Any, Callable, Mapping

import torch


THROUGHPUT_REPORT_SCHEMA_ID = "nextengine.training.reference-throughput-report.v1"


def resolve_performance_overrides(
    profile: Mapping[str, Any],
    *,
    iterations: int | None = None,
    num_envs: int | None = None,
    minibatches: int | None = None,
    learning_rate: float | None = None,
    evaluation_num_envs: int | None = None,
) -> tuple[dict[str, Any], dict[str, dict[str, Any]]]:
    """Resolve bounded report-only sweep values without changing profile identity."""
    document = dict(profile)
    execution = dict(document["execution"])
    ppo = dict(document["ppo"])
    evaluation = dict(document["evaluation"])
    overrides: dict[str, dict[str, Any]] = {}

    def replace(
        target: dict[str, Any], name: str, value: int | float | None
    ) -> None:
        if value is None:
            return
        overrides[name] = {"source": target[name], "resolved": value}
        target[name] = value

    if iterations is not None and (
        iterations <= 0 or iterations > int(execution["iterations"])
    ):
        raise ValueError(
            "iteration override must be positive and no larger than frozen budget"
        )
    if num_envs is not None and not 1 <= num_envs <= 4_096:
        raise ValueError("performance num_envs must be between 1 and 4096")
    if minibatches is not None and minibatches <= 0:
        raise ValueError("performance minibatches must be positive")
    if learning_rate is not None and (
        not math.isfinite(learning_rate) or not 0.0 < learning_rate <= 0.01
    ):
        raise ValueError("performance learning rate must be finite and in (0, 0.01]")

    replace(execution, "iterations", iterations)
    replace(execution, "num_envs", num_envs)
    replace(ppo, "minibatches", minibatches)
    replace(ppo, "learning_rate", learning_rate)
    if evaluation_num_envs is not None:
        if not 0 < evaluation_num_envs <= int(execution["num_envs"]):
            raise ValueError(
                "performance evaluation cohort must fit the resolved environment"
            )
        overrides["evaluation_num_envs"] = {
            "source": evaluation.get("num_envs", profile["execution"]["num_envs"]),
            "resolved": evaluation_num_envs,
        }
        overrides["evaluation_episode_matrix"] = {
            "source": evaluation.get("episode_matrix", "completion-order-v1"),
            "resolved": "fixed-vector-waves-v1",
        }
        overrides["reset_episode_sequence_before_training"] = {
            "source": execution.get("reset_episode_sequence_before_training", False),
            "resolved": True,
        }
        evaluation["num_envs"] = evaluation_num_envs
        evaluation["episode_matrix"] = "fixed-vector-waves-v1"
        execution["reset_episode_sequence_before_training"] = True
    batch_size = int(execution["num_envs"]) * int(
        execution["rollout_steps_per_env"]
    )
    if batch_size % int(ppo["minibatches"]) != 0:
        raise ValueError("resolved rollout batch must divide evenly into minibatches")
    document["execution"] = execution
    document["ppo"] = ppo
    document["evaluation"] = evaluation
    return document, overrides


@dataclass(frozen=True)
class PhaseTiming:
    iteration: int
    phase: str
    host_submission_nanoseconds: int
    cuda_milliseconds: float


class ReferenceThroughputRecorder:
    """Collect report-only phase timings without changing training metrics."""

    def __init__(self, *, warmup_iterations: int) -> None:
        if warmup_iterations < 0:
            raise ValueError("throughput warmup iterations must be non-negative")
        self.warmup_iterations = warmup_iterations
        self._pending: list[tuple[int, str, int, Any, Any]] = []
        self._open: tuple[int, str, int, Any] | None = None
        self._measurement_started_ns: int | None = None

    def begin(self, iteration: int, phase: str) -> None:
        if iteration <= 0 or phase not in {"rollout", "update"}:
            raise ValueError("invalid reference throughput phase")
        if self._open is not None:
            raise RuntimeError("reference throughput phase already open")
        start_event = torch.cuda.Event(enable_timing=True)
        start_event.record()
        now = monotonic_ns()
        if iteration > self.warmup_iterations and self._measurement_started_ns is None:
            self._measurement_started_ns = now
        self._open = (iteration, phase, now, start_event)

    def end(self, iteration: int, phase: str) -> None:
        if self._open is None or self._open[:2] != (iteration, phase):
            raise RuntimeError("reference throughput phase does not match open phase")
        open_iteration, open_phase, started_ns, start_event = self._open
        end_event = torch.cuda.Event(enable_timing=True)
        end_event.record()
        self._pending.append(
            (open_iteration, open_phase, monotonic_ns() - started_ns, start_event, end_event)
        )
        self._open = None

    def finalize(
        self,
        *,
        num_envs: int,
        rollout_steps_per_env: int,
        telemetry: list[Mapping[str, int | float | str]],
    ) -> dict[str, Any]:
        if self._open is not None:
            raise RuntimeError("cannot finalize an open reference throughput phase")
        if self._measurement_started_ns is None:
            raise RuntimeError("throughput run has no measured iteration")
        torch.cuda.synchronize()
        elapsed_ns = monotonic_ns() - self._measurement_started_ns
        timings = [
            PhaseTiming(
                iteration=iteration,
                phase=phase,
                host_submission_nanoseconds=host_ns,
                cuda_milliseconds=float(start.elapsed_time(end)),
            )
            for iteration, phase, host_ns, start, end in self._pending
            if iteration > self.warmup_iterations
        ]
        return build_throughput_report(
            timings=timings,
            elapsed_nanoseconds=elapsed_ns,
            num_envs=num_envs,
            rollout_steps_per_env=rollout_steps_per_env,
            warmup_iterations=self.warmup_iterations,
            telemetry=telemetry,
        )


class NvidiaSmiTelemetrySampler:
    """Sample GPU telemetry out of band from the CUDA training stream."""

    def __init__(
        self,
        *,
        device_index: int,
        interval_seconds: float = 0.5,
        runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
    ) -> None:
        if device_index < 0 or interval_seconds <= 0.0:
            raise ValueError("invalid GPU telemetry sampling configuration")
        self.device_index = device_index
        self.interval_seconds = interval_seconds
        self._runner = runner
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self.records: list[dict[str, int | float | str]] = []
        self.error: str | None = None

    def __enter__(self) -> "NvidiaSmiTelemetrySampler":
        if self._thread is not None:
            raise RuntimeError("GPU telemetry sampler already started")
        self._thread = threading.Thread(target=self._sample_until_stopped, daemon=True)
        self._thread.start()
        return self

    def __exit__(self, *_: object) -> None:
        self._stop.set()
        if self._thread is not None:
            self._thread.join(timeout=max(2.0, self.interval_seconds * 4.0))
        if self._thread is not None and self._thread.is_alive():
            self.error = "nvidia-smi telemetry sampler did not stop"

    def _sample_until_stopped(self) -> None:
        while not self._stop.is_set():
            try:
                result = self._runner(
                    [
                        "nvidia-smi",
                        f"--id={self.device_index}",
                        "--query-gpu=utilization.gpu,utilization.memory,memory.used,"
                        "power.draw,temperature.gpu,clocks.current.sm,pstate",
                        "--format=csv,noheader,nounits",
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                )
                record = parse_gpu_telemetry_csv(result.stdout)
                record["monotonic_nanoseconds"] = monotonic_ns()
                self.records.append(record)
            except (OSError, subprocess.SubprocessError, ValueError) as error:
                self.error = str(error)
                return
            self._stop.wait(self.interval_seconds)


def assert_no_competing_training_process(
    *,
    device_index: int,
    current_pid: int | None = None,
    runner: Callable[..., subprocess.CompletedProcess[str]] = subprocess.run,
) -> None:
    current_pid = os.getpid() if current_pid is None else current_pid
    result = runner(
        [
            "nvidia-smi",
            f"--id={device_index}",
            "--query-compute-apps=pid,process_name",
            "--format=csv,noheader,nounits",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    competing: list[str] = []
    for line in result.stdout.splitlines():
        fields = [field.strip() for field in line.split(",", maxsplit=1)]
        if len(fields) != 2:
            continue
        try:
            pid = int(fields[0])
        except ValueError:
            continue
        process = fields[1].lower()
        if pid != current_pid and any(
            marker in process for marker in ("python", "isaac", "kit", "train")
        ):
            competing.append(f"{pid}:{fields[1]}")
    if competing:
        raise RuntimeError(
            "reference performance evidence requires an exclusive training GPU: "
            + ", ".join(sorted(competing))
        )


def parse_gpu_telemetry_csv(value: str) -> dict[str, int | float | str]:
    fields = [field.strip() for field in value.strip().split(",")]
    if len(fields) != 7:
        raise ValueError("unexpected nvidia-smi telemetry response")
    try:
        return {
            "gpu_utilization_percent": int(fields[0]),
            "memory_utilization_percent": int(fields[1]),
            "memory_used_mib": int(fields[2]),
            "power_watts": float(fields[3]),
            "temperature_celsius": int(fields[4]),
            "sm_clock_mhz": int(fields[5]),
            "pstate": fields[6],
        }
    except ValueError as error:
        raise ValueError("invalid nvidia-smi telemetry response") from error


def build_throughput_report(
    *,
    timings: list[PhaseTiming],
    elapsed_nanoseconds: int,
    num_envs: int,
    rollout_steps_per_env: int,
    warmup_iterations: int,
    telemetry: list[Mapping[str, int | float | str]],
) -> dict[str, Any]:
    if elapsed_nanoseconds <= 0 or num_envs <= 0 or rollout_steps_per_env <= 0:
        raise ValueError("invalid reference throughput measurement")
    iterations = sorted({timing.iteration for timing in timings})
    if not iterations:
        raise ValueError("reference throughput report has no measured iteration")
    for iteration in iterations:
        phases = [timing.phase for timing in timings if timing.iteration == iteration]
        if phases != ["rollout", "update"]:
            raise ValueError("reference throughput iteration has incomplete phases")
    measured_samples = len(iterations) * num_envs * rollout_steps_per_env
    elapsed_seconds = elapsed_nanoseconds / 1_000_000_000.0
    phase_summary = {
        phase: _summarize_values(
            [timing.cuda_milliseconds for timing in timings if timing.phase == phase]
        )
        for phase in ("rollout", "update")
    }
    telemetry_summary: dict[str, Any] = {"sample_count": len(telemetry)}
    for field in (
        "gpu_utilization_percent",
        "memory_utilization_percent",
        "memory_used_mib",
        "power_watts",
        "temperature_celsius",
        "sm_clock_mhz",
    ):
        values = [float(record[field]) for record in telemetry if field in record]
        if values:
            telemetry_summary[field] = _summarize_values(values)
    return {
        "schema_version": 1,
        "schema_id": THROUGHPUT_REPORT_SCHEMA_ID,
        "claim": "PerformanceEvidenceOnly",
        "warmup_iterations": warmup_iterations,
        "measured_iterations": len(iterations),
        "measured_samples": measured_samples,
        "elapsed_seconds": elapsed_seconds,
        "samples_per_second": measured_samples / elapsed_seconds,
        "phase_cuda_milliseconds": phase_summary,
        "host_submission_milliseconds": {
            phase: _summarize_values(
                [
                    timing.host_submission_nanoseconds / 1_000_000.0
                    for timing in timings
                    if timing.phase == phase
                ]
            )
            for phase in ("rollout", "update")
        },
        "gpu_telemetry": telemetry_summary,
    }


def _summarize_values(values: list[float]) -> dict[str, float]:
    if not values:
        raise ValueError("cannot summarize an empty performance sample")
    return {
        "mean": statistics.fmean(values),
        "median": statistics.median(values),
        "minimum": min(values),
        "maximum": max(values),
    }
