from __future__ import annotations

import unittest

from nextengine_speech_timeline.adapters.base import AffectObservation
from nextengine_speech_timeline.timeline import AffectCadence, SpeechTimeline


def affect(start: int, end: int, angry: float) -> AffectObservation:
    return AffectObservation(
        model_id="emotion",
        model_revision="revision",
        start_sample=start,
        end_sample=end,
        source_revision=end,
        scores={"angry": angry, "neutral": 1.0 - angry},
        top_label="angry" if angry > 0.5 else "neutral",
        inference_elapsed_ms=1,
    )


class TimelineTests(unittest.TestCase):
    def test_cadence_switches_from_one_to_two_second_windows(self) -> None:
        cadence = AffectCadence()
        self.assertEqual(cadence.advance(15_999), [])
        first = cadence.advance(16_000)
        self.assertEqual([(item.mode, item.start_sample, item.end_sample) for item in first], [("fast", 0, 16_000)])
        requests = cadence.advance(32_000)
        self.assertEqual(
            [(item.mode, item.start_sample, item.end_sample) for item in requests],
            [
                ("fast", 4_000, 20_000),
                ("fast", 8_000, 24_000),
                ("fast", 12_000, 28_000),
                ("stable", 0, 32_000),
            ],
        )
        final = cadence.final(35_000)
        self.assertEqual((final.mode, final.start_sample, final.end_sample), ("final", 0, 35_000))

    def test_tracks_revision_independently_and_utterance_fusion_has_no_word_spans(self) -> None:
        timeline = SpeechTimeline()
        transcript = timeline.apply_transcript(
            text="Привет",
            stable_prefix="При",
            final=False,
            timing_precision="utterance",
        )
        self.assertEqual(transcript.transcript.revision, 1)
        self.assertEqual(transcript.vocal_affect.revision, 0)
        affected = timeline.apply_affect(affect(0, 16_000, 0.9))
        self.assertEqual(affected.transcript.revision, 1)
        self.assertEqual(affected.vocal_affect.revision, 1)
        timeline.apply_affect(affect(4_000, 20_000, 0.9))
        final = timeline.apply_transcript(
            text="Привет",
            stable_prefix="Привет",
            final=True,
            timing_precision="utterance",
        )
        utterance = timeline.utterance_final()
        self.assertEqual(utterance["alignment_grade"], "utterance")
        self.assertEqual(utterance["spans"], [])
        self.assertEqual(final.fusion.spans, ())
        self.assertTrue(final.fusion.observed_vocal_expression in {"unknown", "angry"})

    def test_affect_replacement_produces_non_overlapping_segments(self) -> None:
        timeline = SpeechTimeline()
        for observation in (
            affect(0, 16_000, 0.9),
            affect(4_000, 20_000, 0.9),
            affect(8_000, 24_000, 0.9),
        ):
            timeline.apply_affect(observation)
        segments = timeline.affect.segments
        self.assertTrue(segments)
        for left, right in zip(segments, segments[1:]):
            self.assertLessEqual(left.end_sample, right.start_sample)
        self.assertEqual(timeline.affect.replace_from_sample, 8_000)


if __name__ == "__main__":
    unittest.main()
