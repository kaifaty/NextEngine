from __future__ import annotations

import asyncio
from dataclasses import asdict, is_dataclass
from typing import Any, Awaitable, Callable, Coroutine, TypeVar

from .adapters.base import AudioWindow
from .adapters.voxtral_transcribe_cpp import TranscriberConfig, TranscriptRevision
from .audio import pcm16le_to_float32, pcm16le_to_float32_array
from .metrics import ModelJobMetric, ResourceMonitor
from .protocol import event
from .scheduler import (
    JobCompletion,
    JobDiscarded,
    JobPriority,
    ModelScheduler,
    SchedulerOverloaded,
)
from .session import SessionBounds, SessionError, SessionState, SpeechSession
from .timeline import AffectCadence, SpeechTimeline, TimelineSnapshot


T = TypeVar("T")


class SpeechTimelineRuntime:
    """Owns resident models and the single dedicated model worker."""

    def __init__(self, transcriber: Any, affect_analyzer: Any) -> None:
        self.transcriber = transcriber
        self.affect_analyzer = affect_analyzer
        self.scheduler = ModelScheduler()
        self._ready: dict[str, object] | None = None
        self._closed = False
        self.resources = ResourceMonitor()

    def start(self) -> dict[str, object]:
        if self._ready is not None:
            return self._ready

        def load_and_warm() -> dict[str, object]:
            transcriber_load = self.transcriber.load()
            transcriber_warmup = self.transcriber.warmup()
            affect_load = self.affect_analyzer.load()
            affect_warmup = self.affect_analyzer.warmup()
            return {
                "transcriber": _value(self.transcriber.capabilities()),
                "vocal_affect": _value(self.affect_analyzer.capabilities()),
                "load": {
                    "transcriber": _value(transcriber_load),
                    "vocal_affect": _value(affect_load),
                },
                "warmup": {
                    "transcriber": _value(transcriber_warmup),
                    "vocal_affect": _value(affect_warmup),
                },
            }

        self._ready = self.scheduler.call_blocking(load_and_warm)
        self.resources.start()
        self._ready["resources"] = self.resources.snapshot()
        return self._ready

    def close(self) -> None:
        if self._closed:
            return
        self._closed = True
        self.resources.stop()
        if hasattr(self.transcriber, "close"):
            self.scheduler.call_blocking(self.transcriber.close)
        self.scheduler.close()


class SpeechConnection:
    """One authenticated WebSocket connection and at most one utterance."""

    def __init__(
        self,
        runtime: SpeechTimelineRuntime,
        bounds: SessionBounds,
        claim: Callable[[SpeechConnection], Awaitable[bool]],
        release: Callable[[SpeechConnection], Awaitable[None]],
    ) -> None:
        self.runtime = runtime
        self.session = SpeechSession(bounds)
        self.timeline = SpeechTimeline()
        self.cadence = AffectCadence()
        self.events: asyncio.Queue[dict[str, object]] = asyncio.Queue(maxsize=64)
        self._claim = claim
        self._release = release
        self._transcriber_session: Any = None
        self._asr_tasks: set[asyncio.Task[None]] = set()
        self._emotion_tasks: set[asyncio.Task[None]] = set()
        self._released = False
        self._job_metrics: list[ModelJobMetric] = []
        self.terminal_ready = asyncio.Event()
        self._scheduler_baseline = asdict(self.runtime.scheduler.metrics)
        self._session_max_queue_depth = 0

    async def start(self, session_id: str, locale: str | None) -> None:
        if not await self._claim(self):
            raise SessionError("SERVICE_BUSY", "another speech session is active")
        try:
            generation = self.session.start(session_id, locale)
            self._transcriber_session = await self._execute(
                JobPriority.STARTUP,
                generation,
                lambda: self.runtime.transcriber.start(TranscriberConfig(language=locale)),
            )
        except SessionError:
            raise
        except Exception as error:
            self.session.fail()
            await self._release_once()
            raise SessionError("MODEL_FAILURE", _bounded_error(error)) from error
        await self.events.put(
            event(
                "session.started",
                session_id=session_id,
                generation=generation,
                sample_rate_hz=16_000,
                encoding="pcm_s16le",
                channels=1,
            )
        )

    async def append_pcm(self, payload: bytes) -> None:
        frame = self.session.append_pcm(payload)
        generation = self.session.generation
        self._spawn(
            self._asr_push(generation, frame.start_sample, frame.end_sample, frame.payload),
            self._asr_tasks,
        )
        pcm = self.session.pcm_bytes
        for request in self.cadence.advance(frame.end_sample):
            window = pcm[request.start_sample * 2 : request.end_sample * 2]
            self._spawn(
                self._affect_observe(
                    generation,
                    request.mode,
                    request.start_sample,
                    request.end_sample,
                    frame.sequence + 1,
                    window,
                ),
                self._emotion_tasks,
            )

    async def finish(self, session_id: str) -> None:
        first = self.session.begin_finish(session_id)
        if not first:
            return
        if self.session.total_samples == 0:
            await self._fail("EMPTY_UTTERANCE", "cannot finalize an empty utterance")
            return
        if self._asr_tasks:
            await asyncio.gather(*tuple(self._asr_tasks), return_exceptions=True)
        if self.session.state is SessionState.FAILED:
            return
        generation = self.session.generation
        pcm = self.session.pcm_bytes

        def finish_transcriber() -> TranscriptRevision:
            try:
                return self._transcriber_session.finish()
            finally:
                self._transcriber_session.close()

        final_transcript_task = asyncio.create_task(
            self._execute(
                JobPriority.VOXTRAL_FINISH,
                generation,
                finish_transcriber,
                audio_start_sample=0,
                audio_end_sample=self.session.total_samples,
            )
        )
        final_window = AffectCadence.final(self.session.total_samples)
        final_affect_task = asyncio.create_task(
            self._execute(
                JobPriority.EMOTION_FINAL,
                generation,
                lambda: self.runtime.affect_analyzer.observe(
                    AudioWindow(
                        samples=pcm16le_to_float32(pcm),
                        sample_rate_hz=16_000,
                        start_sample=final_window.start_sample,
                        end_sample=final_window.end_sample,
                        source_revision=self.session.total_samples,
                    )
                ),
                audio_start_sample=final_window.start_sample,
                audio_end_sample=final_window.end_sample,
            )
        )
        try:
            final_transcript, final_affect = await asyncio.gather(
                final_transcript_task, final_affect_task
            )
        except BaseException as error:
            await self._fail("MODEL_FAILURE", _bounded_error(error))
            return
        self._transcriber_session = None
        self.timeline.apply_transcript(
            text=final_transcript.full_text,
            stable_prefix=final_transcript.committed_text,
            final=True,
            timing_precision=final_transcript.timing_precision,
        )
        snapshot = self.timeline.apply_affect(final_affect)
        self.session.complete(session_id)
        self.runtime.scheduler.invalidate_generation(generation)
        self._cancel_tasks(self._emotion_tasks)
        await self.events.put(_timeline_event(session_id, snapshot))
        await self.events.put(
            event(
                "utterance.final",
                session_id=session_id,
                **self.timeline.utterance_final(),
                metrics=self.metrics_payload(),
            )
        )
        self.session.claim_terminal_event()
        self.terminal_ready.set()
        await self._release_once()

    async def cancel(self, session_id: str) -> None:
        if not self.session.cancel(session_id):
            return
        generation = self.session.generation
        self.runtime.scheduler.invalidate_generation(generation)
        self._cancel_tasks(self._asr_tasks)
        self._cancel_tasks(self._emotion_tasks)
        if self._transcriber_session is not None:
            try:
                await self._execute(
                    JobPriority.VOXTRAL_FINISH,
                    generation,
                    self._transcriber_session.cancel,
                    is_current=lambda _: True,
                )
            except BaseException:
                pass
            self._transcriber_session = None
        if self.session.claim_terminal_event():
            await self.events.put(event("session.cancelled", session_id=session_id))
        self.terminal_ready.set()
        await self._release_once()

    async def disconnect(self) -> None:
        if self.session.state in {SessionState.ACTIVE, SessionState.FINALIZING}:
            assert self.session.session_id is not None
            await self.cancel(self.session.session_id)
        else:
            await self._release_once()

    async def fail_input(self, code: str, detail: str) -> None:
        """Terminate an unrecoverable client-input fault exactly once."""
        await self._fail(code, detail)

    async def _asr_push(
        self, generation: int, start_sample: int, end_sample: int, pcm: bytes
    ) -> None:
        try:
            revision = await self._execute(
                JobPriority.VOXTRAL_PUSH,
                generation,
                lambda: self._transcriber_session.push_pcm(pcm16le_to_float32_array(pcm)),
                audio_start_sample=start_sample,
                audio_end_sample=end_sample,
            )
            if revision is not None:
                snapshot = self.timeline.apply_transcript(
                    text=revision.full_text,
                    stable_prefix=revision.committed_text,
                    final=False,
                    timing_precision=revision.timing_precision,
                )
                await self.events.put(_timeline_event(self.session.session_id, snapshot))
        except asyncio.CancelledError:
            raise
        except JobDiscarded:
            return
        except BaseException as error:
            await self._fail("SERVICE_OVERLOADED" if isinstance(error, SchedulerOverloaded) else "MODEL_FAILURE", _bounded_error(error))

    async def _affect_observe(
        self,
        generation: int,
        mode: str,
        start_sample: int,
        end_sample: int,
        source_revision: int,
        pcm: bytes,
    ) -> None:
        priority = (
            JobPriority.EMOTION_FAST if mode == "fast" else JobPriority.EMOTION_STABLE
        )
        try:
            observation = await self._execute(
                priority,
                generation,
                lambda: self.runtime.affect_analyzer.observe(
                    AudioWindow(
                        samples=pcm16le_to_float32(pcm),
                        sample_rate_hz=16_000,
                        start_sample=start_sample,
                        end_sample=end_sample,
                        source_revision=source_revision,
                    )
                ),
                coalesce_key=mode,
                audio_start_sample=start_sample,
                audio_end_sample=end_sample,
            )
            snapshot = self.timeline.apply_affect(observation)
            await self.events.put(_timeline_event(self.session.session_id, snapshot))
        except asyncio.CancelledError:
            raise
        except JobDiscarded:
            return
        except BaseException as error:
            await self._fail("MODEL_FAILURE", _bounded_error(error))

    async def _execute(
        self,
        priority: JobPriority,
        generation: int,
        function: Callable[[], T],
        *,
        is_current: Callable[[int], bool] | None = None,
        coalesce_key: object | None = None,
        audio_start_sample: int = 0,
        audio_end_sample: int = 0,
    ) -> T:
        loop = asyncio.get_running_loop()
        future: asyncio.Future[T] = loop.create_future()

        def resolve(completion: JobCompletion[T]) -> None:
            if future.cancelled() or future.done():
                return
            if not isinstance(completion.error, JobDiscarded):
                self._job_metrics.append(
                    ModelJobMetric(
                        session_generation=generation,
                        job_kind=priority.name.lower(),
                        audio_start_sample=audio_start_sample,
                        audio_end_sample=audio_end_sample,
                        queue_wait_ms=completion.queue_wait_ms,
                        inference_ms=completion.inference_ms,
                    )
                )
            if completion.error is not None:
                future.set_exception(completion.error)
            else:
                future.set_result(completion.value)  # type: ignore[arg-type]

        self.runtime.scheduler.submit(
            priority,
            generation,
            function,
            is_current=is_current or self.session.is_generation_current,
            callback=lambda completion: loop.call_soon_threadsafe(resolve, completion),
            coalesce_key=coalesce_key,
        )
        self._session_max_queue_depth = max(
            self._session_max_queue_depth,
            self.runtime.scheduler.live_queue_depth,
        )
        return await future

    async def _fail(self, code: str, detail: str) -> None:
        if not self.session.fail():
            return
        self.runtime.scheduler.invalidate_generation(self.session.generation)
        self._cancel_tasks(self._asr_tasks, exclude=asyncio.current_task())
        self._cancel_tasks(self._emotion_tasks, exclude=asyncio.current_task())
        if self._transcriber_session is not None:
            try:
                await self._execute(
                    JobPriority.VOXTRAL_FINISH,
                    self.session.generation,
                    self._transcriber_session.cancel,
                    is_current=lambda _: True,
                )
            except BaseException:
                pass
            self._transcriber_session = None
        await self.events.put(
            event("error", code=code, terminal=True, detail=detail[:512])
        )
        self.session.claim_terminal_event()
        self.terminal_ready.set()
        await self._release_once()

    def _spawn(self, coroutine: Coroutine[Any, Any, None], tasks: set[asyncio.Task[None]]) -> None:
        task = asyncio.create_task(coroutine)
        tasks.add(task)
        task.add_done_callback(tasks.discard)

    @staticmethod
    def _cancel_tasks(
        tasks: set[asyncio.Task[None]],
        *,
        exclude: asyncio.Task[Any] | None = None,
    ) -> None:
        for task in tuple(tasks):
            if task is not exclude:
                task.cancel()

    async def _release_once(self) -> None:
        if self._released:
            return
        self._released = True
        await self._release(self)

    def metrics_payload(self) -> dict[str, object]:
        scheduler_current = asdict(self.runtime.scheduler.metrics)
        scheduler_delta = {
            key: scheduler_current[key] - self._scheduler_baseline[key]
            for key in scheduler_current
            if key != "max_queue_depth"
        }
        scheduler_delta["max_queue_depth"] = self._session_max_queue_depth
        return {
            "audio_samples": self.session.total_samples,
            "jobs": [item.as_dict() for item in self._job_metrics],
            "scheduler": scheduler_delta,
            "model_load_count": {
                "transcriber": getattr(self.runtime.transcriber, "load_count", None),
                "vocal_affect": getattr(self.runtime.affect_analyzer, "load_count", None),
            },
            "resources": self.runtime.resources.snapshot(),
        }


def _timeline_event(session_id: str | None, snapshot: TimelineSnapshot) -> dict[str, object]:
    return event(
        "speech_timeline.update",
        session_id=session_id,
        revision=snapshot.revision,
        transcript={
            "revision": snapshot.transcript.revision,
            "text": snapshot.transcript.text,
            "stable_prefix": snapshot.transcript.stable_prefix,
            "final": snapshot.transcript.final,
            "timing_precision": snapshot.transcript.timing_precision,
        },
        vocal_affect={
            "revision": snapshot.vocal_affect.revision,
            "replace_from_sample": snapshot.vocal_affect.replace_from_sample,
            "raw_observations": [
                {
                    "observation_id": item.observation_id,
                    "start_sample": item.start_sample,
                    "end_sample": item.end_sample,
                    "scores": dict(item.scores),
                    "top_label": item.top_label,
                }
                for item in snapshot.vocal_affect.raw_observations
            ],
            "segments": [asdict(item) for item in snapshot.vocal_affect.segments],
        },
        fusion={
            "revision": snapshot.fusion.revision,
            "alignment_grade": snapshot.fusion.alignment_grade,
            "spans": [],
            "observed_vocal_expression": snapshot.fusion.observed_vocal_expression,
        },
    )


def _value(value: object) -> object:
    if is_dataclass(value):
        return asdict(value)
    if isinstance(value, dict):
        return value
    return str(value)


def _bounded_error(error: BaseException) -> str:
    return f"model operation failed: {type(error).__name__}"
