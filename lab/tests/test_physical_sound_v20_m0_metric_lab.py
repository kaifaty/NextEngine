#!/usr/bin/env python3
"""Focused development-only tests for Physical Sound V20 M0."""

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

import physical_sound_v20_m0_common as common
import physical_sound_v20_m0_metric_lab as metric_lab

B0_ROOT = Path(
    "/home/kaifaty/.codex/experiments/nextengine/physical-sound/"
    "v18-b0-deterministic-global-run-a"
)
F0_ROOT = Path(
    "/home/kaifaty/.codex/experiments/nextengine/physical-sound/"
    "v19-f0-residual-harmonic-run-a"
)


class PhysicalSoundV20M0MetricLabTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dependencies = common.load_dependencies(B0_ROOT, F0_ROOT)
        cls.corpus = metric_lab.build_corpus(cls.dependencies)
        cls.truth = metric_lab.build_truth_cache(cls.corpus)
        cls.controls = metric_lab.control_specs()
        cls.by_name = {item.name: item for item in cls.controls}

    def test_protocol_environment_dependencies_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )
        self.assertEqual(
            common.tree_digest(self.dependencies.b0_file_map), common.B0_TREE_DIGEST
        )
        self.assertEqual(
            common.tree_digest(self.dependencies.f0_file_map), common.F0_TREE_DIGEST
        )

    def test_only_exact_development_views_and_queries_are_selected(self) -> None:
        self.assertEqual(len(self.corpus.objects), 24)
        self.assertEqual(len(self.corpus.case_records), metric_lab.CASE_COUNT)
        self.assertEqual(self.corpus.truth_gains.shape, (192, 8))
        self.assertTrue(
            all(item.row.role == "development" for item in self.corpus.objects)
        )
        for object_index, item in enumerate(self.corpus.objects):
            selected = metric_lab.selected_queries(item.accepted_query)
            observed = self.corpus.query_vertices[
                self.corpus.case_object_index == object_index
            ]
            np.testing.assert_array_equal(observed, selected)

    def test_query_selection_fails_closed_on_insufficient_or_duplicate_values(
        self,
    ) -> None:
        with self.assertRaises(common.M0Error):
            metric_lab.selected_queries(np.arange(7, dtype=np.int64))
        with self.assertRaises(common.M0Error):
            metric_lab.selected_queries(np.zeros(8, dtype=np.int64))

    def test_control_matrix_matches_frozen_protocol(self) -> None:
        self.assertEqual(len(self.controls), 58)
        self.assertEqual(len({item.name for item in self.controls}), 58)
        self.assertEqual(
            self.by_name["frequency-uniform-020-cents"].classification,
            "acceptable",
        )
        self.assertEqual(
            self.by_name["frequency-uniform-040-cents"].classification,
            "harmful",
        )
        self.assertEqual(
            self.by_name["onset-delay-32-samples"].classification,
            "diagnostic",
        )
        self.assertEqual(
            self.by_name["sample-zero-impulse-008"].classification,
            "harmful",
        )

    def test_stft_is_periodic_hann_and_right_padded(self) -> None:
        starts, sample_index, window = metric_lab._stft_plan(256, 64)
        self.assertEqual(int(starts[0]), 0)
        self.assertEqual(int(starts[-1]), 8_128)
        self.assertGreater(int(sample_index[-1, -1]), metric_lab.SAMPLE_COUNT - 1)
        self.assertEqual(float(window[0]), 0.0)
        self.assertNotEqual(float(window[-1]), 0.0)
        magnitude = metric_lab._stft_magnitude(self.truth.normalized[:2], 256, 64)
        self.assertEqual(magnitude.shape, (2, 128, 129))

    def test_identity_and_global_polarity_obey_metric_invariant(self) -> None:
        for name in ("identity", "global-polarity"):
            waveform, _frequency, _damping, _gain = metric_lab._render_control(
                self.corpus, self.truth, self.by_name[name], 0, 32
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

    def test_frequency_and_damping_harmful_controls_fail_physical_limits(
        self,
    ) -> None:
        frequency = self.by_name["frequency-alternating-090-cents"]
        _waveform, frequencies, damping, gains = metric_lab._render_control(
            self.corpus, self.truth, frequency, 0, 32
        )
        physical = metric_lab._physical_batch(
            self.corpus, frequencies, damping, gains, 0, 32
        )
        self.assertGreater(float(np.min(physical["frequency_cents_max"])), 60.0)

        decay = self.by_name["damping-positive-0p35"]
        _waveform, frequencies, damping, gains = metric_lab._render_control(
            self.corpus, self.truth, decay, 0, 32
        )
        physical = metric_lab._physical_batch(
            self.corpus, frequencies, damping, gains, 0, 32
        )
        self.assertGreater(float(np.min(physical["damping_relative_median"])), 0.20)

    def test_mode_removal_and_gain_sign_have_physical_owners(self) -> None:
        removed = self.by_name["mode-removal-2"]
        _waveform, frequencies, damping, gains = metric_lab._render_control(
            self.corpus, self.truth, removed, 0, 32
        )
        physical = metric_lab._physical_batch(
            self.corpus, frequencies, damping, gains, 0, 32
        )
        np.testing.assert_array_equal(physical["missing_mode_count"], np.full(32, 2))

        signed = self.by_name["alternating-gain-sign"]
        _waveform, frequencies, damping, gains = metric_lab._render_control(
            self.corpus, self.truth, signed, 0, 32
        )
        physical = metric_lab._physical_batch(
            self.corpus, frequencies, damping, gains, 0, 32
        )
        self.assertGreater(float(np.mean(physical["gain_nrmse"])), 0.40)

    def test_transient_harmful_controls_move_transient_metric(self) -> None:
        for name in ("onset-delay-64-samples", "sample-zero-impulse-008"):
            waveform, _frequency, _damping, _gain = metric_lab._render_control(
                self.corpus, self.truth, self.by_name[name], 0, 32
            )
            metrics = metric_lab._acoustic_batch(waveform, self.truth, 0, 32)
            self.assertGreater(float(np.min(metrics["transient_energy"])), 0.0)

    def test_legacy_attribution_is_exactly_reproduced_without_i1(self) -> None:
        legacy = metric_lab._legacy_attribution(self.corpus)
        self.assertTrue(legacy["pass"])
        self.assertLessEqual(
            legacy["maximum_absolute_delta"], metric_lab.IDENTITY_TOLERANCE
        )
        self.assertTrue(legacy["causal_ordering_reproduced"])

    def test_metric_serialization_roundtrip_is_exact(self) -> None:
        waveform, frequencies, damping, gains = metric_lab._render_control(
            self.corpus, self.truth, self.by_name["identity"], 0, 32
        )
        acoustic = metric_lab._acoustic_batch(waveform, self.truth, 0, 32)
        physical = metric_lab._physical_batch(
            self.corpus, frequencies, damping, gains, 0, 32
        )
        rows = []
        for index in range(32):
            rows.append(
                {
                    "case_index": index,
                    **{
                        name: int(value[index])
                        if name
                        in (
                            "decay_active_cells",
                            "decay_omitted_cells",
                            "missing_mode_count",
                        )
                        else float(value[index])
                        for name, value in {**acoustic, **physical}.items()
                    },
                }
            )
        self.assertTrue(metric_lab._metric_roundtrip_exact(rows))

    def test_output_and_dependency_boundaries_fail_closed(self) -> None:
        with self.assertRaises(common.M0Error):
            common.prepare_output(common.repository_root() / "forbidden-m0-output")
        with self.assertRaises(common.i0_common.I0Error):
            common.i0_common._validate_artifact_root(
                F0_ROOT, "0" * 64, set(self.dependencies.f0_file_map)
            )
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            root = Path(temporary)
            staging, _target = common.prepare_output(root / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


if __name__ == "__main__":
    unittest.main()
