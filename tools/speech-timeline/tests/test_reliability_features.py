from __future__ import annotations

import asyncio
import math
import tracemalloc
import unittest

from nextengine_speech_timeline.metrics import ModelJobMetric
from nextengine_speech_timeline.protocol import MAX_JSON_BYTES, encode_event, event
from nextengine_speech_timeline.reliability_features import (
    FEATURE_KIND,
    FEATURE_SCHEMA,
    MAX_REVISIONS,
    ReliabilityFeatureError,
    RevisionObservation,
    TranscriptRevisionState,
    build_feature_payload,
    common_prefix_length,
    incomplete_feature_payload,
    normalized_churn,
)
from nextengine_speech_timeline.service import SpeechConnection
from nextengine_speech_timeline.session import SessionBounds

from test_websocket import FakeAffect, FakeTranscriber


BASE_NS = 1_000_000_000_000


def observe(
    state: TranscriptRevisionState,
    index: int,
    text: str,
    stable_prefix: str = "",
    *,
    final: bool = False,
    audio_end_sample: int | None = None,
) -> None:
    if audio_end_sample is None:
        audio_end_sample = (index + 1) * 1_280
    state.observe(
        RevisionObservation(
            full_text=text,
            stable_prefix=stable_prefix,
            final=final,
            audio_end_sample=audio_end_sample,
            wall_monotonic_ns=BASE_NS + index * 80_000_000,
        )
    )


def base_identities() -> dict[str, object]:
    return {
        "feature_schema": FEATURE_SCHEMA,
        "sample_rate_hz": 16_000,
        "asr_model": "fake-asr/1",
        "asr_adapter_id": "fake-transcribe/1",
        "asr_runtime_id": "fake-runtime",
        "audio_route": "raw",
    }


def silent_levels(samples: int = 1_280) -> tuple[dict[str, object], dict[str, object]]:
    level = {
        "samples": samples,
        "rms_dbfs": -60.0,
        "peak_dbfs": -40.0,
        "nonzero_ratio": 0.5,
        "clipping_ratio": 0.0,
    }
    return dict(level), dict(level)


def complete_payload(**overrides: object) -> dict[str, object]:
    raw, route = silent_levels()
    arguments: dict[str, object] = {
        "identities": base_identities(),
        "transcript": {
            "revisions_received": 2,
            "nonempty_revisions": 2,
            "first_text_audio_ms": 80.0,
            "first_text_wall_ms": 0.0,
            "last_change_audio_ms": 160.0,
            "last_change_wall_ms": 80.0,
            "cumulative_churn": 1.4,
            "max_churn": 1.0,
            "stable_prefix_ratio": 0.5,
            "final_to_previous_edit_distance": 0,
            "final_char_count": 9,
            "final_word_count": 2,
            "utterance_duration_ms": 160.0,
            "all_revisions_empty": False,
            "final_empty": False,
            "text_appeared_then_vanished": False,
            "final_matches_previous": True,
        },
        "raw_signal": raw,
        "route_signal": route,
        "noise_floor_dbfs": None,
        "noise_floor_source": "default_thresholds",
        "speech_samples": 640,
        "speech_ratio": 0.5,
        "vad_segment_count": 2,
        "ingress_frames": 2,
        "discontinuous_frames": 0,
        "route_sample_deficit": 0,
        "scheduler_overloads": 0,
        "asr_job_failures": 0,
        "preprocessor_active": False,
        "algorithmic_delay_ms": 0,
    }
    arguments.update(overrides)
    return build_feature_payload(**arguments)


class ChurnFormulaTests(unittest.TestCase):
    def test_documented_churn_boundaries(self) -> None:
        self.assertEqual(normalized_churn("", ""), 0.0)
        self.assertEqual(normalized_churn("", "привет"), 1.0)
        self.assertEqual(normalized_churn("привет", ""), 1.0)
        self.assertEqual(normalized_churn("привет", "привет"), 0.0)
        # " мир" is four units against the longer ten-character text.
        self.assertEqual(normalized_churn("привет", "привет мир"), 4 / 10)
        self.assertLessEqual(normalized_churn("абв", "яъэ"), 1.0)

    def test_common_prefix_length(self) -> None:
        self.assertEqual(common_prefix_length("привет мир", "привет"), 6)
        self.assertEqual(common_prefix_length("", "привет"), 0)


class GoldenVectorTests(unittest.TestCase):
    def test_golden_two_partial_revisions(self) -> None:
        state = TranscriptRevisionState()
        observe(state, 0, "привет")
        observe(state, 1, "привет мир", "привет ")
        features = state.transcript_features(utterance_duration_ms=240.0)

        self.assertEqual(
            features,
            {
                "revisions_received": 2,
                "nonempty_revisions": 2,
                "first_text_audio_ms": 80.0,
                "first_text_wall_ms": 0.0,
                "last_change_audio_ms": 160.0,
                "last_change_wall_ms": 80.0,
                "cumulative_churn": round(1.0 + 4 / 10, 6),
                "max_churn": 1.0,
                "stable_prefix_ratio": round(common_prefix_length("привет", "привет мир") / 10, 6),
                "final_to_previous_edit_distance": None,
                "final_char_count": 9,
                "final_word_count": 2,
                "utterance_duration_ms": 240.0,
                "all_revisions_empty": False,
                "final_empty": False,
                "text_appeared_then_vanished": False,
                "final_matches_previous": False,
            },
        )

    def test_golden_full_sequence_with_final_revision(self) -> None:
        state = TranscriptRevisionState()
        observe(state, 0, "привет")
        observe(state, 1, "привет мир", "привет ")
        observe(state, 2, "привет мир!", "привет ")
        observe(state, 3, "привет мир", "привет мир", final=True, audio_end_sample=5120)
        features = state.transcript_features(utterance_duration_ms=320.0)

        self.assertEqual(features["revisions_received"], 4)
        self.assertEqual(features["nonempty_revisions"], 4)
        self.assertEqual(features["final_to_previous_edit_distance"], 0)
        self.assertTrue(features["final_matches_previous"])
        self.assertEqual(features["final_word_count"], 2)
        self.assertEqual(features["final_char_count"], 9)
        self.assertAlmostEqual(features["stable_prefix_ratio"], 1.0)
        self.assertEqual(features["first_text_audio_ms"], 80.0)
        self.assertEqual(features["first_text_wall_ms"], 0.0)
        # "привет мир!" normalizes to the same text as revision 1, so the
        # documented last-change markers stay on revision 1.
        self.assertEqual(features["last_change_audio_ms"], 160.0)
        self.assertEqual(features["last_change_wall_ms"], 80.0)
        # Final repeats the previous revision: churn stays at the partial sum.
        self.assertEqual(features["cumulative_churn"], round(1.0 + 4 / 10, 6))

    def test_empty_utterance_has_only_the_final_empty_revision(self) -> None:
        state = TranscriptRevisionState()
        observe(state, 0, "", "", final=True)
        features = state.transcript_features(utterance_duration_ms=80.0)

        self.assertEqual(features["revisions_received"], 1)
        self.assertEqual(features["nonempty_revisions"], 0)
        self.assertTrue(features["all_revisions_empty"])
        self.assertTrue(features["final_empty"])
        self.assertFalse(features["text_appeared_then_vanished"])
        self.assertIsNone(features["first_text_audio_ms"])
        self.assertIsNone(features["first_text_wall_ms"])
        self.assertEqual(features["final_to_previous_edit_distance"], 0)
        self.assertFalse(features["final_matches_previous"])

    def test_speech_present_but_final_transcript_empty(self) -> None:
        state = TranscriptRevisionState()
        observe(state, 0, "привет")
        observe(state, 1, "привет", "привет")
        observe(state, 2, "", "", final=True)
        features = state.transcript_features(utterance_duration_ms=240.0)

        self.assertTrue(features["text_appeared_then_vanished"])
        self.assertTrue(features["final_empty"])
        self.assertFalse(features["all_revisions_empty"])
        self.assertEqual(features["final_to_previous_edit_distance"], 6)
        self.assertFalse(features["final_matches_previous"])

    def test_transient_vanish_with_recovery_is_not_reported_as_vanish(self) -> None:
        state = TranscriptRevisionState()
        observe(state, 0, "привет")
        observe(state, 1, "")
        observe(state, 2, "снова текст", "", final=True)
        features = state.transcript_features(utterance_duration_ms=240.0)

        self.assertFalse(features["text_appeared_then_vanished"])
        self.assertEqual(features["nonempty_revisions"], 2)
        self.assertEqual(features["cumulative_churn"], 3.0)
        self.assertFalse(features["final_empty"])

    def test_highly_oscillating_partials_maximize_churn(self) -> None:
        state = TranscriptRevisionState()
        texts = ("кот", "пёс", "дом", "сыр")
        for index, text in enumerate(texts[:-1]):
            observe(state, index, text)
        observe(state, len(texts) - 1, texts[-1], final=True)
        features = state.transcript_features(utterance_duration_ms=320.0)

        self.assertEqual(features["max_churn"], 1.0)
        self.assertEqual(features["cumulative_churn"], 4.0)
        self.assertFalse(features["final_matches_previous"])
        self.assertEqual(features["final_to_previous_edit_distance"], 3)


class PayloadValidationTests(unittest.TestCase):
    def test_complete_payload_shape_and_identity(self) -> None:
        payload = complete_payload()

        self.assertEqual(payload["kind"], FEATURE_KIND)
        self.assertEqual(payload["completeness"], "complete")
        self.assertIsNone(payload["invalid_reason"])
        self.assertEqual(payload["identity"]["asr_adapter_id"], "fake-transcribe/1")
        self.assertEqual(payload["acoustic"]["route_minus_raw_rms_dbfs"], 0.0)
        self.assertFalse(payload["runtime"]["preprocessor_active"])

    def test_unknown_schema_version_fails_closed(self) -> None:
        identities = base_identities()
        identities["feature_schema"] = "speech-reliability-features-v9"
        with self.assertRaisesRegex(ReliabilityFeatureError, "UNKNOWN_FEATURE_SCHEMA"):
            complete_payload(identities=identities)

    def test_missing_adapter_identity_fails_closed(self) -> None:
        identities = base_identities()
        del identities["asr_adapter_id"]
        with self.assertRaisesRegex(ReliabilityFeatureError, "IDENTITY_MISMATCH"):
            complete_payload(identities=identities)

    def test_expected_identity_mismatch_fails_closed(self) -> None:
        with self.assertRaisesRegex(ReliabilityFeatureError, "IDENTITY_MISMATCH"):
            complete_payload(expected_adapter_id="other-transcribe/1")

    def test_wrong_sample_rate_fails_closed(self) -> None:
        identities = base_identities()
        identities["sample_rate_hz"] = 48_000
        with self.assertRaisesRegex(ReliabilityFeatureError, "IDENTITY_MISMATCH"):
            complete_payload(identities=identities)

    def test_non_finite_level_fails_closed(self) -> None:
        _, route = silent_levels()
        route["rms_dbfs"] = float("nan")
        with self.assertRaisesRegex(ReliabilityFeatureError, "NON_FINITE_FEATURE"):
            complete_payload(route_signal=route)

    def test_out_of_range_ratio_fails_closed(self) -> None:
        raw, _ = silent_levels()
        raw["clipping_ratio"] = 1.5
        with self.assertRaisesRegex(ReliabilityFeatureError, "OUT_OF_RANGE_FEATURE"):
            complete_payload(raw_signal=raw)

    def test_infinite_transcript_feature_fails_closed_but_none_stays_optional(self) -> None:
        transcript = dict(complete_payload()["transcript"])
        transcript["cumulative_churn"] = float("inf")
        with self.assertRaisesRegex(ReliabilityFeatureError, "NON_FINITE_FEATURE"):
            complete_payload(transcript=transcript)

        optional = dict(transcript)
        optional["cumulative_churn"] = 1.0
        optional["first_text_audio_ms"] = None
        payload = complete_payload(transcript=optional)
        self.assertIsNone(payload["transcript"]["first_text_audio_ms"])

    def test_infinite_noise_floor_fails_closed(self) -> None:
        with self.assertRaisesRegex(ReliabilityFeatureError, "NON_FINITE_FEATURE"):
            complete_payload(noise_floor_dbfs=float("inf"))

    def test_negative_counts_fail_closed(self) -> None:
        with self.assertRaisesRegex(ReliabilityFeatureError, "OUT_OF_RANGE"):
            complete_payload(ingress_frames=-1)

    def test_empty_route_branch_pins_documented_floor(self) -> None:
        empty = {"samples": 0, "rms_dbfs": None, "peak_dbfs": None}
        payload = complete_payload(route_signal=dict(empty))
        self.assertEqual(payload["acoustic"]["asr_route"]["rms_dbfs"], -120.0)
        self.assertEqual(payload["acoustic"]["asr_route"]["clipping_ratio"], 0.0)

    def test_incomplete_helper_carries_typed_reason(self) -> None:
        payload = incomplete_feature_payload("FEATURE_CAPTURE_FAILED", "boom")
        self.assertEqual(payload["completeness"], "incomplete")
        self.assertEqual(payload["invalid_reason"], "FEATURE_CAPTURE_FAILED")


class AccumulatorBoundTests(unittest.TestCase):
    def test_memory_does_not_grow_with_revision_count(self) -> None:
        state = TranscriptRevisionState()
        text = "повторяющийся фрагмент текста"
        for index in range(300):
            observe(state, index, f"{text} {index % 7}")

        tracemalloc.start()
        baseline = tracemalloc.get_traced_memory()[0]
        for index in range(300, 4_000):
            observe(state, index, f"{text} {index % 7}")
        current, _peak = tracemalloc.get_traced_memory()
        tracemalloc.stop()

        self.assertLess(current - baseline, 256 * 1024)
        self.assertEqual(state.revisions_received, 4_000)
        self.assertTrue(math.isfinite(state.cumulative_churn))
        self.assertGreaterEqual(state.max_churn, 0.0)

    def test_state_holds_no_history_containers(self) -> None:
        state = TranscriptRevisionState()
        for index in range(50):
            observe(state, index, f"текст {index}")
        for name, value in vars(state).items():
            self.assertNotIsInstance(value, (list, dict, set), name)

    def test_excessive_revision_count_fails_closed(self) -> None:
        limited = TranscriptRevisionState()
        limited.revisions_received = MAX_REVISIONS
        with self.assertRaisesRegex(ReliabilityFeatureError, "TOO_MANY_REVISIONS"):
            observe(limited, 0, "текст")

    def test_oversized_revision_text_fails_closed(self) -> None:
        state = TranscriptRevisionState()
        with self.assertRaisesRegex(ReliabilityFeatureError, "TRANSCRIPT_TOO_LARGE"):
            observe(state, 0, "ж" * 5_000)

    def test_negative_audio_clock_fails_closed(self) -> None:
        state = TranscriptRevisionState()
        with self.assertRaisesRegex(ReliabilityFeatureError, "INVALID_AUDIO_CLOCK"):
            observe(state, 0, "текст", audio_end_sample=-1)


class ServicePathTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        self.runtime = _started_runtime()

    def _connection(self) -> SpeechConnection:
        async def claim(_: object) -> bool:
            return True

        async def release(_: object) -> None:
            return None

        return SpeechConnection(self.runtime, SessionBounds(), claim, release)

    async def test_long_turn_metrics_with_many_revisions_stay_bounded(self) -> None:
        connection = self._connection()
        state = connection._reliability_transcript
        for index in range(600):
            observe(state, index, f"частичная ревизия номер {index % 11}")
        connection._job_metrics = [
            ModelJobMetric(1, "asr_push", index * 1_000, (index + 1) * 1_000, 3, 40)
            for index in range(1_000)
        ]

        metrics = connection.metrics_payload()
        features = metrics["recognition_reliability_features"]
        self.assertIsInstance(features, dict)
        self.assertIn("completeness", features)
        self.assertEqual(metrics["jobs_total"], 1_000)
        encoded = encode_event(
            event(
                "utterance.final",
                session_id="long",
                text="готово",
                timing_precision="utterance",
                observed_vocal_expression="unknown",
                alignment_grade="utterance",
                spans=[],
                metrics=metrics,
            )
        )
        self.assertLessEqual(len(encoded.encode("utf-8")), MAX_JSON_BYTES)

    async def test_identical_event_sequences_produce_identical_vectors(self) -> None:
        sequence = [
            ("привет", ""),
            ("привет мир", "привет "),
            ("", ""),
            ("привет снова", ""),
            ("привет снова и мир", "привет снова "),
        ]
        first = TranscriptRevisionState()
        second = TranscriptRevisionState()
        for index, (text, prefix) in enumerate(sequence):
            observation = RevisionObservation(
                full_text=text,
                stable_prefix=prefix,
                final=False,
                audio_end_sample=(index + 1) * 1_280,
                wall_monotonic_ns=BASE_NS + index * 80_000_000,
            )
            first.observe(observation)
            second.observe(observation)
        self.assertEqual(
            first.transcript_features(utterance_duration_ms=400.0),
            second.transcript_features(utterance_duration_ms=400.0),
        )

        third = TranscriptRevisionState()
        for index, (text, prefix) in enumerate(sequence[:-1]):
            observe(third, index, text, prefix)
        observe(third, len(sequence) - 1, "совсем другой финал", "", final=True)
        perturbed = third.transcript_features(utterance_duration_ms=400.0)
        baseline = first.transcript_features(utterance_duration_ms=400.0)
        differing_keys = {
            key
            for key in baseline
            if perturbed[key] != baseline[key]
        }
        self.assertTrue(differing_keys)
        self.assertIn("cumulative_churn", differing_keys)

    async def test_capture_fault_degrades_to_typed_incomplete_payload(self) -> None:
        connection = self._connection()
        connection._reliability_fault = ("TOO_MANY_REVISIONS", "too many")
        payload = connection._reliability_features(
            activity_segments=(), speech_samples=0
        )
        self.assertEqual(payload["completeness"], "incomplete")
        self.assertEqual(payload["invalid_reason"], "TOO_MANY_REVISIONS")

    async def test_features_from_live_connection_are_complete_and_labeled(self) -> None:
        connection = self._connection()
        observe(connection._reliability_transcript, 0, "промежуточный текст")
        payload = connection._reliability_features(
            activity_segments=(), speech_samples=0
        )

        self.assertEqual(payload["kind"], FEATURE_KIND)
        self.assertEqual(payload["completeness"], "complete")
        self.assertIsNone(payload["invalid_reason"])
        self.assertEqual(payload["identity"]["asr_model"], "default")
        self.assertEqual(payload["identity"]["asr_adapter_id"], "fake-transcribe/1")
        self.assertEqual(payload["identity"]["audio_route"], "raw")
        self.assertEqual(payload["identity"]["sample_rate_hz"], 16_000)
        self.assertEqual(payload["transcript"]["revisions_received"], 1)
        self.assertEqual(payload["acoustic"]["noise_floor_source"], "default_thresholds")


def _started_runtime() -> object:
    from nextengine_speech_timeline.service import SpeechTimelineRuntime

    transcriber = FakeTranscriber(adapter_id="fake-transcribe/1")
    runtime = SpeechTimelineRuntime(transcriber, FakeAffect())
    runtime._ready = {
        "transcribers": {
            runtime.default_transcriber: dict(transcriber.capabilities()),
        },
    }
    return runtime


if __name__ == "__main__":
    unittest.main()
