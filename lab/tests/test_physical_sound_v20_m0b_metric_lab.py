#!/usr/bin/env python3
"""Focused development-only tests for Physical Sound V20 M0b."""

from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path

for _thread_variable in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
):
    os.environ[_thread_variable] = "1"

import numpy as np

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v20_m0b_common as common
import physical_sound_v20_m0b_metric_lab as metric_lab

EXPERIMENT_ROOT = Path("/home/kaifaty/.codex/experiments/nextengine/physical-sound")
B0_ROOT = EXPERIMENT_ROOT / "v18-b0-deterministic-global-run-a"
F0_ROOT = EXPERIMENT_ROOT / "v19-f0-residual-harmonic-run-a"
M0_ROOT = EXPERIMENT_ROOT / "v20-m0-phase-consistent-run-a"


class PhysicalSoundV20M0bMetricLabTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.predecessor = common.load_predecessor(M0_ROOT)
        cls.dependencies = common.load_dependencies(B0_ROOT, F0_ROOT)
        cls.corpus = metric_lab.build_corpus(cls.dependencies)
        cls.truth = metric_lab.build_truth_cache(cls.corpus)
        cls.controls = metric_lab.control_specs()
        cls.by_name = {item.name: item for item in cls.controls}

    def test_protocol_environment_lineage_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )
        self.assertEqual(
            common.tree_digest(self.predecessor.file_map), common.M0_TREE_DIGEST
        )
        self.assertFalse(self.predecessor.report["single_run_pass"])

    def test_predecessor_root_and_member_mutations_fail_closed(self) -> None:
        with self.assertRaises(common.M0bError):
            common.load_predecessor(common.repository_root())
        with (
            tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary,
            self.assertRaises(common.M0bError),
        ):
            common.load_predecessor(Path(temporary))

    def test_exact_development_corpus_and_control_matrix_are_reused(self) -> None:
        self.assertEqual(len(self.corpus.objects), 24)
        self.assertEqual(len(self.corpus.case_records), 192)
        self.assertTrue(
            all(item.row.role == "development" for item in self.corpus.objects)
        )
        self.assertEqual(len(self.controls), 58)
        self.assertEqual(
            [item.record() for item in self.controls],
            [item.record() for item in metric_lab.m0_lab.control_specs()],
        )

    def test_centered_log_magnitude_removes_frame_level(self) -> None:
        rng = np.random.default_rng(7)
        magnitude = 1.0 + rng.random((2, 4, 17))
        left = metric_lab._centered_log_magnitude(magnitude)
        right = metric_lab._centered_log_magnitude(5.0 * magnitude)
        self.assertLess(float(np.max(np.abs(left - right))), 1.0e-7)
        raw = np.mean(
            np.abs(np.log(5.0 * magnitude + 1.0e-7) - np.log(magnitude + 1.0e-7))
        )
        self.assertGreater(raw, 1.0)

    def test_edc_slope_is_amplitude_invariant(self) -> None:
        truth_magnitude = self.truth.base.stft_magnitudes[512][:4]
        left, left_active = metric_lab._edc_slope(truth_magnitude)
        right, right_active = metric_lab._edc_slope(7.0 * truth_magnitude)
        np.testing.assert_array_equal(left_active, right_active)
        np.testing.assert_allclose(left, right, atol=1.0e-12, rtol=0.0)

    def test_identity_and_polarity_obey_successor_invariants(self) -> None:
        for name in ("identity", "global-polarity"):
            waveform, frequency, damping, gain = metric_lab.m0_lab._render_control(
                self.corpus, self.truth.base, self.by_name[name], 0, 32
            )
            metrics = metric_lab._acoustic_batch(waveform, self.truth, 0, 32)
            for metric in metric_lab.ACOUSTIC_METRICS:
                self.assertLessEqual(
                    float(np.max(metrics[metric])), metric_lab.IDENTITY_TOLERANCE
                )
            if name == "global-polarity":
                self.assertGreaterEqual(
                    float(np.min(metrics["full_waveform_nrmse"])), 1.9
                )
                physical = metric_lab.m0_lab._physical_batch(
                    self.corpus, frequency, damping, gain, 0, 32
                )
                self.assertGreater(float(np.mean(physical["gain_nrmse"])), 0.40)

    def test_selected_frequency_and_damping_steps_are_ordered(self) -> None:
        values = {}
        for name in (
            "frequency-uniform-020-cents",
            "frequency-uniform-040-cents",
            "damping-positive-0p08",
            "damping-positive-0p12",
        ):
            waveform, _frequency, _damping, _gain = metric_lab.m0_lab._render_control(
                self.corpus, self.truth.base, self.by_name[name], 0, 32
            )
            values[name] = metric_lab._acoustic_batch(waveform, self.truth, 0, 32)
        self.assertLess(
            float(
                np.quantile(
                    values["frequency-uniform-020-cents"][
                        "mean_centered_log_magnitude"
                    ],
                    0.95,
                )
            ),
            float(
                np.quantile(
                    values["frequency-uniform-040-cents"][
                        "mean_centered_log_magnitude"
                    ],
                    0.95,
                )
            ),
        )
        self.assertLess(
            float(
                np.quantile(
                    values["damping-positive-0p08"]["decay_slope_residual"],
                    0.95,
                )
            ),
            float(
                np.quantile(
                    values["damping-positive-0p12"]["decay_slope_residual"],
                    0.95,
                )
            ),
        )

    def test_raw_predecessor_metrics_reproduce_for_base_controls(self) -> None:
        metric_map = {
            "mrsc": "mrsc",
            "raw_mrlm": "mrlm",
            "absolute_decay_energy": "decay_energy",
            "transient_energy": "transient_energy",
        }
        predecessor = self.predecessor.report["control_summaries"]
        for name in ("identity", "b0-only", "f0-only", "combined"):
            observed = {metric: [] for metric in metric_map}
            for start in range(0, 192, 32):
                end = start + 32
                waveform, _frequency, _damping, _gain = (
                    metric_lab.m0_lab._render_control(
                        self.corpus, self.truth.base, self.by_name[name], start, end
                    )
                )
                metrics = metric_lab._acoustic_batch(waveform, self.truth, start, end)
                for metric in metric_map:
                    observed[metric].extend(metrics[metric].tolist())
            for current, old in metric_map.items():
                self.assertAlmostEqual(
                    float(np.quantile(observed[current], 0.95)),
                    predecessor[name]["metrics"][old]["p95"],
                    delta=metric_lab.IDENTITY_TOLERANCE,
                )

    def test_physical_owner_and_legacy_evidence_remain_unchanged(self) -> None:
        component = metric_lab.m0_lab._component_evidence(
            self.dependencies, self.corpus
        )
        legacy = metric_lab.m0_lab._legacy_attribution(self.corpus)
        self.assertTrue(component["pass"])
        self.assertTrue(legacy["pass"])
        self.assertEqual(legacy["maximum_absolute_delta"], 0.0)

    def test_serialization_and_output_boundary(self) -> None:
        rows = [
            {
                "decay_slope_residual": 0.125,
                "mean_centered_log_magnitude": 0.25,
                "schema": common.METRIC_SCHEMA,
            }
        ]
        self.assertTrue(metric_lab._metric_roundtrip_exact(rows))
        with self.assertRaises(common.M0bError):
            common.prepare_output(common.repository_root() / "forbidden-m0b-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            staging, _target = common.prepare_output(Path(temporary) / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


if __name__ == "__main__":
    unittest.main()
