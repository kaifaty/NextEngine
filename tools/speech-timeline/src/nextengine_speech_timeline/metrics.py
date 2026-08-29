from __future__ import annotations

from dataclasses import asdict, dataclass
import os
from pathlib import Path
import resource
import subprocess
import threading


@dataclass(frozen=True)
class ModelJobMetric:
    session_generation: int
    job_kind: str
    audio_start_sample: int
    audio_end_sample: int
    queue_wait_ms: int
    inference_ms: int

    def as_dict(self) -> dict[str, int | str]:
        return asdict(self)


class ResourceMonitor:
    """Low-rate metadata-only RSS/VRAM sampler for development evidence."""

    def __init__(self, interval_seconds: float = 0.5) -> None:
        self.interval_seconds = interval_seconds
        self._lock = threading.Lock()
        self._stop = threading.Event()
        self._thread: threading.Thread | None = None
        self._latest: dict[str, int | None] = {}
        self._peak_gpu_process_used_mib: int | None = None

    def start(self) -> None:
        if self._thread is not None:
            return
        self._sample()
        self._thread = threading.Thread(
            target=self._run,
            name="nextengine-speech-resource-monitor",
            daemon=True,
        )
        self._thread.start()

    def stop(self) -> None:
        if self._thread is None:
            return
        self._stop.set()
        self._thread.join(timeout=2)
        self._sample()
        self._thread = None

    def snapshot(self) -> dict[str, int | None]:
        with self._lock:
            return {
                **self._latest,
                "peak_gpu_process_used_mib": self._peak_gpu_process_used_mib,
            }

    def _run(self) -> None:
        while not self._stop.wait(self.interval_seconds):
            self._sample()

    def _sample(self) -> None:
        snapshot = resource_snapshot()
        gpu_used = snapshot.get("gpu_process_used_mib")
        with self._lock:
            self._latest = snapshot
            if isinstance(gpu_used, int):
                if self._peak_gpu_process_used_mib is None:
                    self._peak_gpu_process_used_mib = gpu_used
                else:
                    self._peak_gpu_process_used_mib = max(
                        self._peak_gpu_process_used_mib, gpu_used
                    )


def resource_snapshot() -> dict[str, int | None]:
    rss_bytes: int | None = None
    try:
        pages = int(Path("/proc/self/statm").read_text(encoding="ascii").split()[1])
        rss_bytes = pages * os.sysconf("SC_PAGE_SIZE")
    except (OSError, ValueError, IndexError):
        pass
    peak_rss_bytes = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024
    gpu_used: int | None = None
    gpu_total: int | None = None
    gpu_free: int | None = None
    try:
        applications = subprocess.run(
            [
                "nvidia-smi",
                "--query-compute-apps=pid,used_gpu_memory",
                "--format=csv,noheader,nounits",
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=2,
        )
        if applications.returncode == 0:
            matches = []
            for line in applications.stdout.splitlines():
                fields = [field.strip() for field in line.split(",")]
                if len(fields) == 2 and fields[0] == str(os.getpid()):
                    matches.append(int(fields[1]))
            if matches:
                gpu_used = sum(matches)
        device = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=memory.total,memory.free",
                "--format=csv,noheader,nounits",
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=2,
        )
        if device.returncode == 0 and device.stdout.strip():
            total, free = [int(field.strip()) for field in device.stdout.splitlines()[0].split(",")]
            gpu_total, gpu_free = total, free
    except (OSError, ValueError, subprocess.TimeoutExpired):
        pass
    return {
        "process_rss_bytes": rss_bytes,
        "process_peak_rss_bytes": peak_rss_bytes,
        "gpu_process_used_mib": gpu_used,
        "gpu_total_mib": gpu_total,
        "gpu_free_mib": gpu_free,
    }
