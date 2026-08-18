from __future__ import annotations

import threading
import unittest

from nextengine_speech_timeline.scheduler import (
    JobPriority,
    ModelScheduler,
    SchedulerOverloaded,
)


class SchedulerTests(unittest.TestCase):
    def test_priority_is_stable_and_asr_precedes_emotion(self) -> None:
        scheduler = ModelScheduler()
        order: list[str] = []
        current = lambda generation: generation == 1
        scheduler.submit(JobPriority.EMOTION_FAST, 1, lambda: order.append("emotion"), is_current=current)
        scheduler.submit(JobPriority.VOXTRAL_PUSH, 1, lambda: order.append("asr-1"), is_current=current)
        scheduler.submit(JobPriority.VOXTRAL_PUSH, 1, lambda: order.append("asr-2"), is_current=current)
        scheduler.submit(JobPriority.VOXTRAL_FINISH, 1, lambda: order.append("finish"), is_current=current)
        while scheduler.run_next():
            pass
        self.assertEqual(order, ["finish", "asr-1", "asr-2", "emotion"])

    def test_provisional_emotion_coalesces_and_stale_callback_is_discarded(self) -> None:
        scheduler = ModelScheduler(max_queue=4)
        calls: list[str] = []
        callbacks: list[str] = []
        current_generation = [1]
        current = lambda generation: generation == current_generation[0]
        scheduler.submit(
            JobPriority.EMOTION_FAST,
            1,
            lambda: calls.append("obsolete"),
            is_current=current,
            coalesce_key="fast",
        )
        scheduler.submit(
            JobPriority.EMOTION_FAST,
            1,
            lambda: calls.append("latest") or "value",
            is_current=current,
            callback=lambda completion: callbacks.append(str(completion.value)),
            coalesce_key="fast",
        )
        while scheduler.run_next():
            pass
        self.assertEqual(calls, ["latest"])
        self.assertEqual(callbacks, ["value"])
        self.assertEqual(scheduler.metrics.coalesced, 1)

        scheduler.submit(
            JobPriority.EMOTION_FINAL,
            1,
            lambda: current_generation.__setitem__(0, 2) or "late",
            is_current=current,
            callback=lambda completion: callbacks.append(str(completion.value)),
        )
        scheduler.run_next()
        self.assertEqual(callbacks, ["value"])
        self.assertGreaterEqual(scheduler.metrics.stale, 1)

    def test_third_pending_asr_chunk_is_overload(self) -> None:
        scheduler = ModelScheduler(max_pending_asr=2)
        for _ in range(2):
            scheduler.submit(JobPriority.VOXTRAL_PUSH, 1, lambda: None, is_current=lambda _: True)
        with self.assertRaises(SchedulerOverloaded):
            scheduler.submit(JobPriority.VOXTRAL_PUSH, 1, lambda: None, is_current=lambda _: True)

    def test_started_scheduler_uses_one_dedicated_worker(self) -> None:
        scheduler = ModelScheduler()
        completed = threading.Event()
        thread_names: list[str] = []
        scheduler.start()
        scheduler.submit(
            JobPriority.EMOTION_FINAL,
            1,
            lambda: thread_names.append(threading.current_thread().name),
            is_current=lambda _: True,
            callback=lambda _: completed.set(),
        )
        self.assertTrue(completed.wait(2))
        self.assertTrue(scheduler.wait_idle())
        scheduler.close()
        self.assertEqual(thread_names, ["nextengine-speech-model-worker"])


if __name__ == "__main__":
    unittest.main()
