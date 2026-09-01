#!/usr/bin/env python3
"""Focused guards for the Physical Sound V16 L0 known-truth oracle."""

from __future__ import annotations

import copy
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np
import torch

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v16_l0_common as common  # noqa: E402
import physical_sound_v16_l0_model as neural  # noqa: E402


class PhysicalSoundV16L0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.corpus = common.generate_corpus()
        cls.train = tuple(item for item in cls.corpus if item.spec.role == "train")

    def test_corpus_is_role_disjoint_and_truth_is_bounded(self) -> None:
        self.assertEqual(len(self.corpus), 18)
        self.assertEqual(
            [sum(item.spec.role == role for item in self.corpus) for role in ("train", "development", "test")],
            [9, 3, 6],
        )
        identities = {item.mesh_sha256 for item in self.corpus}
        self.assertEqual(len(identities), 18)
        for item in self.corpus:
            self.assertGreaterEqual(item.frequencies[0], 40.0)
            self.assertLessEqual(item.frequencies[-1], 12_000.0)
            self.assertTrue(np.all(np.diff(item.frequencies) > 0.0))
            self.assertTrue(np.all((item.damping > 0.0) & (item.damping <= 128.0)))
            self.assertTrue(np.isfinite(item.gains).all())
            self.assertGreaterEqual(item.context_indices.size, 20)
            self.assertGreaterEqual(item.query_indices.size, 40)

    def test_frequency_formula_matches_protocol(self) -> None:
        item = self.corpus[0]
        spec = item.spec
        material = common.MATERIALS[spec.material]
        base = 0.5 * material["wave_speed"] * spec.wall_m / spec.length_m**2
        expected = (
            base
            * common.Q
            * common.TOPOLOGY_FACTORS[spec.surface]
            * (1.0 + 0.12 * abs(np.log(spec.aspect)))
            * common.SUPPORT_FREQUENCY_FACTORS[spec.support]
        )
        np.testing.assert_array_equal(item.frequencies, expected)

    def test_surface_topology_and_context_split_are_repeatable(self) -> None:
        for item in self.corpus:
            repeated = common.generate_object(item.spec)
            np.testing.assert_array_equal(item.vertices, repeated.vertices)
            np.testing.assert_array_equal(item.faces, repeated.faces)
            np.testing.assert_array_equal(item.context_indices, repeated.context_indices)
            self.assertEqual(item.mesh_sha256, repeated.mesh_sha256)
            expected_faces = 2 * (item.spec.grid_v - 1) * (
                item.spec.grid_u if item.spec.surface != "Plate" else item.spec.grid_u - 1
            )
            self.assertEqual(item.faces.shape, (expected_faces, 3))

    def test_controls_never_read_query_gain_truth(self) -> None:
        item = self.corpus[-1]
        original = common.gain_controls(item)
        mutated = copy.deepcopy(item)
        mutated.gains[mutated.query_indices] += 100.0
        observed = common.gain_controls(mutated)
        for name in original:
            np.testing.assert_array_equal(original[name], observed[name])

    def test_model_contract_is_finite_and_has_frozen_shapes(self) -> None:
        statistics = neural.fit_training_statistics(self.train, True)
        neural.configure_determinism(common.SEEDS[0])
        model = neural.StructuredModalField(
            statistics.static_normalizer.mean.size,
            statistics.local_normalizer.mean.size,
        ).to(dtype=torch.float64)
        result = neural.predict_member(model, self.train[0], statistics)
        self.assertEqual(result["frequencies"].shape, (8,))
        self.assertEqual(result["damping"].shape, (8,))
        self.assertEqual(
            result["gains"].shape,
            (self.train[0].query_indices.size, common.MODE_COUNT),
        )
        self.assertTrue(common.hard_validate_modes(result["frequencies"], result["damping"]))

    def test_short_train_control_repeats_exactly(self) -> None:
        statistics = neural.fit_training_statistics(self.train, False)
        first, _ = neural.train_member(
            self.train, statistics, common.SEEDS[0], updates=2
        )
        second, _ = neural.train_member(
            self.train, statistics, common.SEEDS[0], updates=2
        )
        self.assertEqual(
            common.model_parameter_bytes(first.state_dict()),
            common.model_parameter_bytes(second.state_dict()),
        )

    def test_ood_development_calibration_can_reject_coverage_collapse(self) -> None:
        development = np.asarray(
            [[0.1, 0.0, 0.1], [0.2, 0.0, 0.2]], dtype=np.float64
        )
        collapse = np.asarray([[0.1, 0.0, 0.6]], dtype=np.float64)
        score, maxima, threshold = common.calibrate_ood(development, collapse)
        np.testing.assert_array_equal(maxima, np.asarray([0.2, 0.0, 0.2]))
        self.assertEqual(threshold, 1.25)
        self.assertGreater(score[0], threshold)

    def test_static_ood_uses_frozen_train_range_allowance(self) -> None:
        development = np.asarray([[0.2, 0.0, 0.2]], dtype=np.float64)
        tolerated = np.asarray([[0.1, 0.1, 0.1]], dtype=np.float64)
        score, _, threshold = common.calibrate_ood(development, tolerated)
        self.assertEqual(threshold, 1.25)
        self.assertLess(score[0], threshold)

    def test_exact_prediction_handles_analytic_nodal_queries(self) -> None:
        plate = next(
            item
            for item in self.corpus
            if item.spec.role == "test" and item.spec.surface == "Plate"
        )
        metrics = common.object_metrics(
            plate,
            plate.frequencies,
            plate.damping,
            plate.gains[plate.query_indices],
        )
        self.assertGreater(metrics["nodal_query_count"], 0)
        self.assertEqual(metrics["gain_nrmse"], 0.0)
        self.assertEqual(metrics["waveform_nrmse_mean"], 0.0)
        self.assertEqual(metrics["spectrum_rmse_db_mean"], 0.0)

    def test_corrupt_modes_fail_closed(self) -> None:
        item = self.corpus[0]
        self.assertTrue(common.hard_validate_modes(item.frequencies, item.damping))
        self.assertFalse(
            common.hard_validate_modes(item.frequencies[::-1], item.damping)
        )
        damping = item.damping.copy()
        damping[0] = -1.0
        self.assertFalse(common.hard_validate_modes(item.frequencies, damping))

    def test_serializers_are_byte_exact(self) -> None:
        value = np.arange(24, dtype=np.float64).reshape(3, 8)
        self.assertEqual(common.array_bytes(value), common.array_bytes(value.copy()))
        self.assertEqual(
            common.canonical_json({"b": value, "a": 1}),
            common.canonical_json({"a": 1, "b": value.copy()}),
        )

    def test_output_boundary_rejects_repo_and_existing_target(self) -> None:
        with self.assertRaises(common.L0Error):
            common.prepare_output(common.repository_root() / "forbidden-l0-output")
        experiment_root = Path(
            "/home/kaifaty/.codex/experiments/nextengine/physical-sound"
        )
        experiment_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=experiment_root) as temporary:
            with self.assertRaises(common.L0Error):
                common.prepare_output(Path(temporary))

    def test_access_ledger_and_authority_are_zero_by_construction(self) -> None:
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertNotIn("path", common.ZERO_ACCESS)
        self.assertNotIn("bytes", common.ZERO_ACCESS)


if __name__ == "__main__":
    unittest.main()
