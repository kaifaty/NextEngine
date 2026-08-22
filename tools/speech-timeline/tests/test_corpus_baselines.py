from __future__ import annotations

import json
import math
from pathlib import Path
import stat
import tempfile
import unittest

import numpy as np

from nextengine_speech_timeline.corpus_baselines import (
    FEATURE_NAMES,
    ReliabilityBaselineError,
    brier_score,
    expected_calibration_error,
    fit_baselines,
    fit_logistic_l2,
    logistic_predict,
    load_training_table,
    stability_rule_predict,
)
from nextengine_speech_timeline.profile import REPOSITORY_ROOT


def _payload(*, speech_ratio: float, final_empty: bool = False,
             revisions: int = 1, vanished: bool = False) -> dict:
    return {
        "completeness": "complete",
        "transcript": {
            "revisions_received": revisions,
            "nonempty_revisions": revisions,
            "cumulative_churn": 1.0,
            "max_churn": 1.0,
            "stable_prefix_ratio": 0.5,
            "final_to_previous_edit_distance": 0,
            "final_char_count": 3,
            "final_word_count": 1,
            "utterance_duration_ms": 1000.0,
            "first_text_audio_ms": 80.0,
            "last_change_audio_ms": 160.0,
            "all_revisions_empty": False,
            "final_empty": final_empty,
            "text_appeared_then_vanished": vanished,
            "final_matches_previous": True,
        },
        "acoustic": {
            "noise_floor_dbfs": None,
            "noise_floor_source": "default_thresholds",
            "speech_samples": int(16_000 * speech_ratio),
            "speech_ratio": speech_ratio,
            "vad_segment_count": 2,
            "raw": {"samples": 16_000, "rms_dbfs": -30.0, "peak_dbfs": -12.0,
                    "nonzero_ratio": 0.9, "clipping_ratio": 0.0},
            "asr_route": {"samples": 16_000, "rms_dbfs": -30.0, "peak_dbfs": -12.0,
                          "nonzero_ratio": 0.9, "clipping_ratio": 0.0},
            "route_minus_raw_rms_dbfs": 0.0,
            "route_minus_raw_peak_dbfs": 0.0,
        },
        "runtime": {
            "ingress_frames": 13,
            "discontinuous_frames": 0,
            "route_sample_deficit": 0,
            "scheduler_overloads": 0,
            "asr_job_failures": 0,
            "preprocessor_active": False,
            "algorithmic_delay_ms": 0,
        },
        "identity": {"feature_schema": "speech-reliability-features-v0"},
    }


def _result_row(*, split: str, exact: bool, payload: dict) -> dict:
    return {
        "kind": "nextengine.speech-reliability.replay-result",
        "schema_version": 0,
        "clip_id": "c",
        "source_id": "s",
        "split": split,
        "audio_sha256": "sha256:" + "0" * 64,
        "samples": 16_000,
        "duration_ms": 1000,
        "outcome": "completed" if not payload["transcript"]["final_empty"] else "completed",
        "error_code": None,
        "detail": None,
        "timing": {},
        "scores": {"exact_match": exact, "wer": 0.0 if exact else 0.5, "cer": 0.0},
        "final_text_empty": payload["transcript"]["final_empty"],
        "speech_samples": 8_000,
        "feature_payload": payload,
        "trace_revisions": 1,
        "trace_truncated_revisions": 0,
    }


class CorpusBaselinesTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def _write_results(self, name: str, rows: list[dict]) -> Path:
        path = self.root / name
        path.write_text(
            "".join(json.dumps(row) + "\n" for row in rows), encoding="utf-8"
        )
        return path

    def test_separable_features_give_perfect_logistic_fit(self) -> None:
        rng = np.random.default_rng(11)
        rows = []
        for index in range(120):
            positive = index % 2 == 0
            ratio = (0.6 if positive else 0.05) + float(rng.uniform(-0.01, 0.01))
            rows.append(
                _result_row(
                    split="train",
                    exact=positive,
                    payload=_payload(speech_ratio=max(0.0, min(1.0, ratio))),
                )
            )
        path = self._write_results("results.jsonl", rows)
        x, y, codes, excluded = load_training_table([path])

        self.assertEqual(x.shape, (120, len(FEATURE_NAMES)))
        self.assertEqual(int(codes.sum()), 0)  # all train
        self.assertEqual(excluded["held_out_refused"], 0)

        mean, std, weights = fit_logistic_l2(x, y)
        p = logistic_predict(mean, std, weights, x)
        self.assertGreater(brier_score(y, np.full_like(y, y.mean())) - brier_score(y, p), 0.15)

    def test_held_out_rows_are_refused_and_counted(self) -> None:
        rows = [
            _result_row(split="train", exact=True,
                        payload=_payload(speech_ratio=0.5)),
            _result_row(split="held_out", exact=False,
                        payload=_payload(speech_ratio=0.5)),
        ]
        path = self._write_results("mixed.jsonl", rows)
        _x, y, codes, excluded = load_training_table([path])
        self.assertEqual(y.size, 1)
        self.assertEqual(excluded["held_out_refused"], 1)

    def test_incomplete_payloads_and_failures_are_excluded(self) -> None:
        broken = _result_row(split="train", exact=True,
                             payload=_payload(speech_ratio=0.5))
        broken["feature_payload"]["completeness"] = "incomplete"
        failed = _result_row(split="train", exact=True,
                             payload=_payload(speech_ratio=0.5))
        failed["outcome"] = "technical_failure"
        good = _result_row(split="train", exact=True,
                           payload=_payload(speech_ratio=0.5))
        path = self._write_results("mixed.jsonl", [broken, failed, good])
        _x, y, codes, excluded = load_training_table([path])
        self.assertEqual(y.size, 1)
        self.assertEqual(excluded["incomplete_payload"], 1)
        self.assertEqual(excluded["not_completed"], 1)

    def test_stability_rule_flags_empty_final_as_unreliable(self) -> None:
        good = np.zeros((1, len(FEATURE_NAMES)))
        by = {name: good[0][i] for i, name in enumerate(FEATURE_NAMES)}
        by.update({"revisions_received": 1, "speech_ratio": 0.6})
        good[0] = [by[name] for name in FEATURE_NAMES]

        empty = good.copy()
        empty[0][FEATURE_NAMES.index("final_empty")] = 1.0

        p = stability_rule_predict(np.vstack([good, empty]))
        self.assertEqual(p[0], 0.95)
        self.assertEqual(p[1], 0.05)

    def test_constant_and_ece_math(self) -> None:
        y = np.array([1.0, 0.0, 1.0, 0.0])
        p = np.full(4, 0.5)
        self.assertAlmostEqual(brier_score(y, p), 0.25)
        # Perfectly calibrated two-bin predictor has zero ECE.
        self.assertAlmostEqual(expected_calibration_error(y, p), 0.0, places=9)

    def test_fit_baselines_publishes_private_candidates_and_report(self) -> None:
        rows = []
        rng = np.random.default_rng(3)
        for index in range(60):
            positive = index % 2 == 0
            rows.append(
                _result_row(
                    split="train",
                    exact=positive,
                    payload=_payload(speech_ratio=0.7 if positive else 0.1),
                )
            )
        for index in range(20):
            positive = index % 2 == 0
            rows.append(
                _result_row(
                    split="calibration",
                    exact=positive,
                    payload=_payload(speech_ratio=0.7 if positive else 0.1),
                )
            )
        results = self._write_results("results.jsonl", rows)
        out_dir = self.root / "candidates"

        report = fit_baselines([results], out_dir=out_dir, lineage={"run": "fixture"})

        self.assertEqual(report["status"], "complete")
        self.assertEqual(report["split_counts"], {"train": 60, "calibration": 20})
        logistic_cal = report["baselines"]["logistic_l2"]["calibration"]
        self.assertIsInstance(logistic_cal, dict)
        self.assertLess(logistic_cal["brier"], 0.01)  # separable fixture under L2
        gate = report["gate_min_10_percent_brier_improvement"]
        self.assertTrue(gate["logistic_l2"])
        for name in ("constant", "stability_rule", "logistic_l2"):
            candidate = out_dir / f"{name.replace('_', '-')}.candidate.json"
            self.assertEqual(stat.S_IMODE(candidate.stat().st_mode), 0o600)
            payload = json.loads(candidate.read_text(encoding="utf-8"))
            self.assertIn("lineage", payload)
        report_path = out_dir / "baseline-report.json"
        self.assertEqual(stat.S_IMODE(report_path.stat().st_mode), 0o600)

    def test_repository_outputs_are_refused(self) -> None:
        inside = REPOSITORY_ROOT / "tools" / "speech-timeline"
        with self.assertRaisesRegex(ReliabilityBaselineError, "external"):
            fit_baselines([], out_dir=inside)


if __name__ == "__main__":
    unittest.main()
