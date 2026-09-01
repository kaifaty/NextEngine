#!/usr/bin/env python3
"""Focused guards for the frozen Physical Sound V17 G0 oracle."""

from __future__ import annotations

import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path

import numpy as np
import torch

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v17_g0_common as common  # noqa: E402
import physical_sound_v17_g0_model as neural  # noqa: E402
import physical_sound_v17_g0_oracle as oracle  # noqa: E402


class PhysicalSoundV17G0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.corpus = common.generate_corpus()
        cls.train = tuple(row for row in cls.corpus if row.role == "train")
        cls.development = tuple(
            row for row in cls.corpus if row.role == "development"
        )

    def test_corpus_has_frozen_counts_ranges_and_unique_identity(self) -> None:
        self.assertEqual(len(self.corpus), 144)
        self.assertEqual(
            {
                role: sum(row.role == role for row in self.corpus)
                for role in (
                    "train",
                    "development",
                    "test-interpolation",
                    "test-scale-transfer",
                )
            },
            {
                "train": 72,
                "development": 24,
                "test-interpolation": 24,
                "test-scale-transfer": 24,
            },
        )
        self.assertEqual(len({row.object_id for row in self.corpus}), 144)
        self.assertEqual(
            (min(row.length_m for row in self.corpus), max(row.length_m for row in self.corpus)),
            (0.17548828125, 0.558359375),
        )
        self.assertAlmostEqual(min(row.aspect for row in self.corpus), 0.7043895747599451)
        self.assertAlmostEqual(max(row.aspect for row in self.corpus), 1.4912208504801097)
        self.assertEqual(
            (min(row.slenderness for row in self.corpus), max(row.slenderness for row in self.corpus)),
            (0.004032, 0.007968),
        )
        for row in self.corpus:
            self.assertTrue(common.hard_validate(row.frequencies, row.damping))

    def test_no_complete_v16_row_is_reused(self) -> None:
        # (material, topology, support, L, aspect, wall) from frozen V16 L0.
        v16_rows = {
            ("Steel", "Plate", "Free", 0.320, 1.30, 0.0040),
            ("Steel", "Cylinder", "BaseClamped", 0.280, 1.10, 0.0030),
            ("Steel", "Bowl", "Free", 0.240, 0.85, 0.0035),
            ("Wood", "Plate", "BaseClamped", 0.380, 1.50, 0.0120),
            ("Wood", "Cylinder", "Free", 0.340, 1.00, 0.0090),
            ("Wood", "Bowl", "BaseClamped", 0.300, 0.90, 0.0100),
            ("Glass", "Plate", "Free", 0.220, 1.20, 0.0040),
            ("Glass", "Cylinder", "BaseClamped", 0.260, 0.85, 0.0030),
            ("Glass", "Bowl", "Free", 0.200, 1.05, 0.0025),
            ("Steel", "Cylinder", "Free", 0.310, 0.92, 0.0032),
            ("Wood", "Bowl", "Free", 0.330, 1.10, 0.0110),
            ("Glass", "Plate", "BaseClamped", 0.240, 1.40, 0.0032),
            ("Steel", "Plate", "BaseClamped", 0.290, 0.80, 0.0045),
            ("Steel", "Bowl", "BaseClamped", 0.270, 1.20, 0.0038),
            ("Wood", "Plate", "Free", 0.350, 1.00, 0.0105),
            ("Wood", "Cylinder", "BaseClamped", 0.310, 1.25, 0.0085),
            ("Glass", "Cylinder", "Free", 0.230, 1.15, 0.0028),
            ("Glass", "Bowl", "BaseClamped", 0.210, 0.95, 0.0030),
        }
        observed = {
            (
                row.material,
                row.topology,
                row.support,
                row.length_m,
                row.aspect,
                row.wall_m,
            )
            for row in self.corpus
        }
        self.assertTrue(observed.isdisjoint(v16_rows))

    def test_hidden_scale_decomposition_recomposes_truth_exactly(self) -> None:
        for row in self.corpus:
            mode = np.arange(common.MODE_COUNT, dtype=np.float64)
            aspect_log = np.log(row.aspect)
            frequency_ratio = (
                common.Q
                * common.TOPOLOGY_FREQUENCY[row.topology_index]
                * common.SUPPORT_FREQUENCY[row.support_index]
                * (
                    1.0
                    + 0.08 * abs(aspect_log)
                    + 0.012 * np.sin((mode + 1.0) * aspect_log)
                )
                * (
                    1.0
                    + 0.01
                    * row.material_index
                    * np.cos(0.7 * (mode + 1.0))
                )
            )
            damping_multiplier = (
                (1.0 + 0.08 * mode)
                * common.TOPOLOGY_DAMPING[row.topology_index]
                * common.SUPPORT_DAMPING[row.support_index]
                * (1.0 + 0.06 * abs(aspect_log))
            )
            np.testing.assert_array_equal(
                frequency_ratio * row.frequency_scale, row.frequencies
            )
            np.testing.assert_array_equal(
                damping_multiplier * row.damping_scale, row.damping
            )

    def test_candidate_excludes_length_and_raw_family_includes_dimensions(self) -> None:
        row = self.train[0]
        scale_changed = replace(
            row,
            length_m=row.length_m * 1.7,
            wall_m=row.wall_m * 1.7,
        )
        np.testing.assert_array_equal(
            neural.candidate_features(row), neural.candidate_features(scale_changed)
        )
        self.assertFalse(
            np.array_equal(neural.raw_features(row), neural.raw_features(scale_changed))
        )
        self.assertEqual(neural.candidate_features(row).shape, (11,))
        self.assertEqual(neural.raw_features(row).shape, (16,))

    def test_model_contract_is_finite_and_has_frozen_shapes(self) -> None:
        statistics = neural.fit_statistics(self.train, "scale_separated")
        neural.configure_determinism(common.SEEDS[0])
        model = neural.GlobalHead(11).to(dtype=torch.float64)
        frequency, damping = neural.predict_member(model, self.train[0], statistics)
        self.assertEqual(frequency.shape, (common.MODE_COUNT,))
        self.assertEqual(damping.shape, (common.MODE_COUNT,))
        self.assertTrue(common.hard_validate(frequency, damping))

    def test_short_training_repeats_byte_exactly(self) -> None:
        statistics = neural.fit_statistics(self.train, "scale_separated")
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

    def test_controls_do_not_read_target_row_truth(self) -> None:
        controls = oracle.Controls(self.train)
        target = self.development[0]
        original = controls.predict(target)
        altered = replace(
            target,
            frequencies=target.frequencies + 10_000.0,
            damping=target.damping + 1_000.0,
        )
        observed = controls.predict(altered)
        for name in oracle.CONTROL_ORDER:
            np.testing.assert_array_equal(original[name][0], observed[name][0])
            np.testing.assert_array_equal(original[name][1], observed[name][1])

    def test_static_ood_ignores_legal_length_transfer_and_catches_corruption(self) -> None:
        minimum, maximum, threshold = oracle._static_ood_setup(
            self.train, self.development
        )
        row = self.development[0]
        legal_scale_transfer = replace(
            row,
            length_m=row.length_m * 2.0,
            wall_m=row.wall_m * 2.0,
        )
        self.assertEqual(
            oracle._static_ood_score(row, minimum, maximum),
            oracle._static_ood_score(legal_scale_transfer, minimum, maximum),
        )
        corrupted = replace(row, slenderness=row.slenderness * 20.0)
        self.assertGreater(
            oracle._static_ood_score(corrupted, minimum, maximum), threshold
        )

    def test_mutation_matrix_has_three_unique_cases_per_interpolation_cell(self) -> None:
        interpolation = tuple(
            row for row in self.corpus if row.role == "test-interpolation"
        )
        identities = set()
        for row in interpolation:
            for mutation in oracle.MUTATION_ORDER:
                mutated = oracle._mutated_row(row, mutation)
                identity = (
                    row.cell,
                    mutation,
                    mutated.material,
                    mutated.support,
                    mutated.length_m,
                    mutated.slenderness,
                )
                identities.add(identity)
                self.assertNotEqual(
                    (
                        mutated.material,
                        mutated.support,
                        mutated.length_m,
                        mutated.slenderness,
                    ),
                    (row.material, row.support, row.length_m, row.slenderness),
                )
        self.assertEqual(len(identities), 72)

    def test_serialization_output_boundary_and_access_ledger(self) -> None:
        value = np.arange(24, dtype=np.float64).reshape(3, 8)
        self.assertEqual(common.array_bytes(value), common.array_bytes(value.copy()))
        self.assertEqual(
            common.canonical_json({"b": value, "a": 1}),
            common.canonical_json({"a": 1, "b": value.copy()}),
        )
        with self.assertRaises(common.G0Error):
            common.prepare_output(common.repository_root() / "forbidden-g0-output")
        experiment_root = Path(
            "/home/kaifaty/.codex/experiments/nextengine/physical-sound"
        )
        experiment_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=experiment_root) as temporary:
            with self.assertRaises(common.G0Error):
                common.prepare_output(Path(temporary))
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertNotIn("path", common.ZERO_ACCESS)
        self.assertNotIn("bytes", common.ZERO_ACCESS)


if __name__ == "__main__":
    unittest.main()
