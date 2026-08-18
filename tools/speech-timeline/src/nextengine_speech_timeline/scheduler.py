from __future__ import annotations

from dataclasses import dataclass, field
from enum import IntEnum
import itertools
from queue import Empty, PriorityQueue
import threading
import time
from typing import Callable, Generic, Hashable, TypeVar


T = TypeVar("T")


class JobPriority(IntEnum):
    STARTUP = -1
    VOXTRAL_FINISH = 0
    VOXTRAL_PUSH = 1
    EMOTION_FINAL = 2
    EMOTION_STABLE = 3
    EMOTION_FAST = 4


class SchedulerOverloaded(RuntimeError):
    pass


class JobDiscarded(RuntimeError):
    pass


@dataclass(frozen=True)
class JobCompletion(Generic[T]):
    value: T | None
    error: BaseException | None
    queue_wait_ms: int
    inference_ms: int


@dataclass(order=True)
class _Job(Generic[T]):
    priority: int
    sequence: int
    generation: int = field(compare=False)
    function: Callable[[], T] = field(compare=False)
    callback: Callable[[JobCompletion[T]], None] | None = field(compare=False)
    is_current: Callable[[int], bool] = field(compare=False)
    coalesce_key: Hashable | None = field(compare=False, default=None)
    enqueued_ns: int = field(compare=False, default_factory=time.monotonic_ns)
    invalidated: bool = field(compare=False, default=False)
    stop: bool = field(compare=False, default=False)


@dataclass
class SchedulerMetrics:
    submitted: int = 0
    completed: int = 0
    failed: int = 0
    coalesced: int = 0
    stale: int = 0
    overloads: int = 0
    max_queue_depth: int = 0


class ModelScheduler:
    """One stable-priority worker for both resident model adapters."""

    def __init__(self, *, max_queue: int = 16, max_pending_asr: int = 2) -> None:
        if max_queue <= 0 or max_pending_asr <= 0:
            raise ValueError("scheduler bounds must be positive")
        self.max_queue = max_queue
        self.max_pending_asr = max_pending_asr
        self.metrics = SchedulerMetrics()
        self._queue: PriorityQueue[_Job[object]] = PriorityQueue()
        self._sequences = itertools.count()
        self._lock = threading.RLock()
        self._pending_asr: dict[int, int] = {}
        self._coalesced: dict[tuple[int, Hashable], _Job[object]] = {}
        self._active = 0
        self._invalidated_pending = 0
        self._stopping = False
        self._thread: threading.Thread | None = None

    def start(self) -> None:
        with self._lock:
            if self._thread is not None:
                return
            self._thread = threading.Thread(
                target=self._worker,
                name="nextengine-speech-model-worker",
                daemon=True,
            )
            self._thread.start()

    @property
    def live_queue_depth(self) -> int:
        with self._lock:
            return self._live_queue_depth()

    def submit(
        self,
        priority: JobPriority,
        generation: int,
        function: Callable[[], T],
        *,
        is_current: Callable[[int], bool],
        callback: Callable[[JobCompletion[T]], None] | None = None,
        coalesce_key: Hashable | None = None,
    ) -> int:
        with self._lock:
            if self._stopping:
                raise SchedulerOverloaded("scheduler is stopping")
            is_asr_push = priority is JobPriority.VOXTRAL_PUSH
            pending_asr = self._pending_asr.get(generation, 0)
            if is_asr_push and pending_asr >= self.max_pending_asr:
                self.metrics.overloads += 1
                raise SchedulerOverloaded("ASR backlog exceeds two pending chunks")
            if self._live_queue_depth() >= self.max_queue:
                self.metrics.overloads += 1
                raise SchedulerOverloaded("model queue is full")
            if coalesce_key is not None:
                key = (generation, coalesce_key)
                previous = self._coalesced.get(key)
                if previous is not None:
                    previous.invalidated = True
                    self._invalidated_pending += 1
                    self.metrics.coalesced += 1
                    if previous.callback is not None:
                        previous.callback(
                            JobCompletion(
                                value=None,
                                error=JobDiscarded("provisional model job was coalesced"),
                                queue_wait_ms=0,
                                inference_ms=0,
                            )
                        )
            sequence = next(self._sequences)
            job: _Job[T] = _Job(
                int(priority), sequence, generation, function, callback, is_current, coalesce_key
            )
            if is_asr_push:
                self._pending_asr[generation] = pending_asr + 1
            if coalesce_key is not None:
                self._coalesced[(generation, coalesce_key)] = job  # type: ignore[assignment]
            self._queue.put(job)  # type: ignore[arg-type]
            self.metrics.submitted += 1
            self.metrics.max_queue_depth = max(
                self.metrics.max_queue_depth, self._live_queue_depth()
            )
            return sequence

    def call_blocking(self, function: Callable[[], T], timeout: float = 120.0) -> T:
        self.start()
        completed = threading.Event()
        result: list[JobCompletion[T]] = []
        self.submit(
            JobPriority.STARTUP,
            0,
            function,
            is_current=lambda _: True,
            callback=lambda completion: (result.append(completion), completed.set()),
        )
        if not completed.wait(timeout):
            raise TimeoutError("model worker call timed out")
        completion = result[0]
        if completion.error is not None:
            raise completion.error
        return completion.value  # type: ignore[return-value]

    def invalidate_generation(self, generation: int) -> None:
        with self._lock:
            for job in self._coalesced.values():
                if job.generation == generation and not job.invalidated:
                    job.invalidated = True
                    self._invalidated_pending += 1

    def run_next(self, *, block: bool = False, timeout: float | None = None) -> bool:
        try:
            item = self._queue.get(block=block, timeout=timeout)
        except Empty:
            return False
        if item.stop:
            self._queue.task_done()
            return False
        self._execute(item)
        self._queue.task_done()
        return True

    def wait_idle(self, timeout: float = 5.0) -> bool:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            with self._lock:
                if self._live_queue_depth() == 0 and self._active == 0:
                    return True
            time.sleep(0.001)
        return False

    def close(self) -> None:
        with self._lock:
            if self._stopping:
                return
            self._stopping = True
            thread = self._thread
            if thread is None:
                return
            self._queue.put(
                _Job(
                    999,
                    next(self._sequences),
                    -1,
                    lambda: None,
                    None,
                    lambda _: False,
                    stop=True,
                )
            )
        thread.join(timeout=5)
        if thread.is_alive():
            raise RuntimeError("model scheduler worker did not stop")

    def _worker(self) -> None:
        while True:
            item = self._queue.get()
            if item.stop:
                self._queue.task_done()
                return
            self._execute(item)
            self._queue.task_done()

    def _execute(self, job: _Job[object]) -> None:
        with self._lock:
            if job.invalidated:
                self._invalidated_pending = max(0, self._invalidated_pending - 1)
            self._remove_pending(job)
            if job.invalidated or not job.is_current(job.generation):
                self.metrics.stale += 1
                return
            self._active += 1
        started_ns = time.monotonic_ns()
        value: object | None = None
        error: BaseException | None = None
        try:
            value = job.function()
        except BaseException as caught:
            error = caught
        ended_ns = time.monotonic_ns()
        with self._lock:
            self._active -= 1
            current = not job.invalidated and job.is_current(job.generation)
            if not current:
                self.metrics.stale += 1
            elif error is None:
                self.metrics.completed += 1
            else:
                self.metrics.failed += 1
        if current and job.callback is not None:
            job.callback(
                JobCompletion(
                    value=value,
                    error=error,
                    queue_wait_ms=round((started_ns - job.enqueued_ns) / 1_000_000),
                    inference_ms=round((ended_ns - started_ns) / 1_000_000),
                )
            )

    def _remove_pending(self, job: _Job[object]) -> None:
        if job.priority == int(JobPriority.VOXTRAL_PUSH):
            remaining = self._pending_asr.get(job.generation, 0) - 1
            if remaining > 0:
                self._pending_asr[job.generation] = remaining
            else:
                self._pending_asr.pop(job.generation, None)
        if job.coalesce_key is not None:
            key = (job.generation, job.coalesce_key)
            if self._coalesced.get(key) is job:
                self._coalesced.pop(key, None)

    def _live_queue_depth(self) -> int:
        # PriorityQueue intentionally keeps invalidated coalesced entries until
        # their turn; capacity counts only useful pending work.
        return max(0, self._queue.qsize() - self._invalidated_pending)
