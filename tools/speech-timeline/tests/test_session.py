from __future__ import annotations

import unittest

from nextengine_speech_timeline.session import (
    SessionBounds,
    SessionError,
    SessionState,
    SpeechSession,
)


def active_session(bounds: SessionBounds | None = None) -> SpeechSession:
    session = SpeechSession(bounds)
    session.authenticate()
    session.start("turn-1", "ru")
    return session


class SessionTests(unittest.TestCase):
    def test_pcm_frames_advance_one_monotonic_sample_clock(self) -> None:
        session = active_session()
        first = session.append_pcm(b"\0\0" * 4)
        second = session.append_pcm(b"\0\0" * 3)
        self.assertEqual((first.sequence, first.start_sample, first.end_sample), (0, 0, 4))
        self.assertEqual((second.sequence, second.start_sample, second.end_sample), (1, 4, 7))
        self.assertEqual(session.total_samples, 7)

    def test_frame_and_turn_bounds_fail_before_mutation(self) -> None:
        session = active_session(SessionBounds(max_frame_bytes=8, max_turn_bytes=12))
        for payload, code in ((b"", "EMPTY_PCM"), (b"x", "MISALIGNED_PCM"), (b"x" * 10, "FRAME_TOO_LARGE")):
            with self.subTest(code=code), self.assertRaises(SessionError) as caught:
                session.append_pcm(payload)
            self.assertEqual(caught.exception.code, code)
        session.append_pcm(b"x" * 8)
        with self.assertRaises(SessionError) as caught:
            session.append_pcm(b"x" * 6)
        self.assertEqual(caught.exception.code, "TURN_TOO_LARGE")
        self.assertEqual(session.total_samples, 4)

    def test_finish_is_idempotent_and_terminal_buffer_is_cleared(self) -> None:
        session = active_session()
        session.append_pcm(b"\0\0" * 8)
        self.assertTrue(session.begin_finish("turn-1"))
        self.assertFalse(session.begin_finish("turn-1"))
        self.assertTrue(session.complete("turn-1"))
        self.assertEqual(session.state, SessionState.FINAL)
        self.assertEqual(session.pcm_bytes, b"")
        self.assertEqual(session.total_samples, 8)
        self.assertTrue(session.claim_terminal_event())
        self.assertFalse(session.claim_terminal_event())
        with self.assertRaises(SessionError):
            session.append_pcm(b"\0\0")

    def test_duplicate_start_and_wrong_identity_are_rejected(self) -> None:
        session = active_session()
        with self.assertRaises(SessionError):
            session.start("turn-2", "ru")
        with self.assertRaises(SessionError) as caught:
            session.cancel("turn-2")
        self.assertEqual(caught.exception.code, "SESSION_MISMATCH")

    def test_pcm_window_copies_only_requested_range(self) -> None:
        session = active_session()
        session.append_pcm(b"0123456789")
        self.assertEqual(session.pcm_window(1, 4), b"234567")
        metrics = session.copy_metrics()
        self.assertEqual(metrics["window_copy_calls"], 1)
        self.assertEqual(metrics["window_copy_bytes"], 6)
        self.assertEqual(metrics["full_copy_bytes"], 0)
        self.assertEqual(metrics["peak_buffer_bytes"], 10)


if __name__ == "__main__":
    unittest.main()
