#!/usr/bin/env python3
"""Focused guards for the frozen Physical Sound V18 B0 oracle."""

from __future__ import annotations

import inspect
import math
import os
import sys
import tempfile
import unittest
from dataclasses import replace
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

import physical_sound_v18_b0_common as common  # noqa: E402
import physical_sound_v18_b0_model as model  # noqa: E402
import physical_sound_v18_b0_oracle as oracle  # noqa: E402


class PhysicalSoundV18B0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.train = common.generate_training_rows()
        cls.fresh = common.generate_fresh_rows()
        cls.development = tuple(
            row for row in cls.fresh if row.role == "development"
        )
        cls.interpolation = tuple(
            row for row in cls.fresh if row.role == "test-interpolation"
        )
        cls.implementation = common.implementation_hashes()
        cls.artifact = model.fit_ridge(
            tuple(model.training_example(row) for row in cls.train),
            common.PROTOCOL_SHA256,
            cls.implementation,
        )

    def test_frozen_counts_ranges_identity_and_v17_disjointness(self) -> None:
        self.assertEqual(len(self.train), 72)
        self.assertEqual(len(self.fresh), 72)
        self.assertEqual(
            {
                role: sum(row.role == role for row in self.fresh)
                for role in (
                    "development",
                    "test-interpolation",
                    "test-scale-transfer",
                )
            },
            {
                "development": 24,
                "test-interpolation": 24,
                "test-scale-transfer": 24,
            },
        )
        self.assertEqual(len({row.object_id for row in self.fresh}), 72)
        self.assertEqual(
            (min(row.length_m for row in self.fresh), max(row.length_m for row in self.fresh)),
            (0.178076171875, 0.5598828125),
        )
        self.assertEqual(min(row.aspect for row in self.fresh), 0.7036579789666209)
        self.assertEqual(max(row.aspect for row in self.fresh), 1.4904892546867856)
        self.assertEqual(min(row.slenderness for row in self.fresh), 0.00418048)
        self.assertEqual(max(row.slenderness for row in self.fresh), 0.00791168)
        self.assertEqual(min(row.wall_m for row in self.fresh), 0.000777103875)
        self.assertEqual(
            max(row.wall_m for row in self.fresh), 0.0041652315500000005
        )
        self.assertEqual(
            min(float(np.min(row.frequencies)) for row in self.fresh),
            45.74235857965094,
        )
        self.assertEqual(
            max(float(np.max(row.frequencies)) for row in self.fresh),
            2901.0848945873163,
        )
        self.assertEqual(
            min(float(np.min(np.diff(row.frequencies))) for row in self.fresh),
            27.53202041965831,
        )
        self.assertTrue(
            {common.row_signature(row) for row in self.fresh}.isdisjoint(
                common.v17_opened_g_signatures()
            )
        )

    def test_hidden_scale_decomposition_recomposes_every_truth_exactly(self) -> None:
        for row in self.train + self.fresh:
            material_index = common.MATERIAL_ORDER.index(row.material)
            topology_index = common.TOPOLOGY_ORDER.index(row.topology)
            support_index = common.SUPPORT_ORDER.index(row.support)
            mode = np.arange(common.MODE_COUNT, dtype=np.float64)
            aspect_log = math.log(row.aspect)
            ratio = (
                common.Q
                * common.TOPOLOGY_FREQUENCY[topology_index]
                * common.SUPPORT_FREQUENCY[support_index]
                * (
                    1.0
                    + 0.08 * abs(aspect_log)
                    + 0.012 * np.sin((mode + 1.0) * aspect_log)
                )
                * (
                    1.0
                    + 0.01
                    * material_index
                    * np.cos(0.7 * (mode + 1.0))
                )
            )
            multiplier = (
                (1.0 + 0.08 * mode)
                * common.TOPOLOGY_DAMPING[topology_index]
                * common.SUPPORT_DAMPING[support_index]
                * (1.0 + 0.06 * abs(aspect_log))
            )
            np.testing.assert_array_equal(ratio * row.frequency_scale, row.frequencies)
            np.testing.assert_array_equal(multiplier * row.damping_scale, row.damping)

    def test_feature_and_polynomial_orders_are_exact(self) -> None:
        inputs = model.input_from_row(self.train[0])
        base = model.base_features(inputs)
        self.assertEqual(base.shape, (11,))
        self.assertEqual(len(model.BASE_FEATURE_NAMES), 11)
        standardized = np.linspace(-1.0, 1.0, 11, dtype=np.float64)
        observed = model.polynomial_features(standardized)
        expected = np.asarray(
            [1.0]
            + standardized.tolist()
            + [
                standardized[left] * standardized[right]
                for left in range(11)
                for right in range(left, 11)
            ],
            dtype=np.float64,
        )
        np.testing.assert_array_equal(observed, expected)
        self.assertEqual(observed.shape, (78,))
        self.assertEqual(len(model.POLYNOMIAL_FEATURE_NAMES), 78)
        self.assertEqual(len(model.TARGET_NAMES), 16)

    def test_ridge_and_serialized_artifact_repeat_exactly(self) -> None:
        examples = tuple(model.training_example(row) for row in self.train)
        repeated = model.fit_ridge(
            examples, common.PROTOCOL_SHA256, self.implementation
        )
        first_files = model.encode_artifact(self.artifact)
        second_files = model.encode_artifact(repeated)
        self.assertEqual(first_files, second_files)
        decoded = model.decode_artifact(first_files)
        self.assertEqual(decoded.training_identity_root, self.artifact.training_identity_root)
        for row in self.train:
            expected = model.predict(self.artifact, model.input_from_row(row))
            observed = model.predict(decoded, model.input_from_row(row))
            np.testing.assert_array_equal(expected[0], observed[0])
            np.testing.assert_array_equal(expected[1], observed[1])

    def test_predictor_is_independent_of_fresh_truth(self) -> None:
        row = self.development[0]
        original_input = model.input_from_row(row)
        altered_truth = replace(
            row,
            frequencies=row.frequencies + 10_000.0,
            damping=row.damping + 1_000.0,
        )
        self.assertEqual(original_input, model.input_from_row(altered_truth))
        first = model.predict(self.artifact, original_input)
        second = model.predict(self.artifact, model.input_from_row(altered_truth))
        np.testing.assert_array_equal(first[0], second[0])
        np.testing.assert_array_equal(first[1], second[1])
        predictor_source = inspect.getsource(model.predict)
        self.assertNotIn("_truth", predictor_source)
        self.assertNotIn("frequencies", predictor_source)

    def test_identity_control_reaches_zero_hidden_scorer_metrics(self) -> None:
        predictions = tuple(
            oracle.Prediction(row.object_id, row.frequencies, row.damping)
            for row in self.fresh
        )
        metrics = oracle.evaluate_rows(self.fresh, predictions)
        for family in ("frequency_cents", "damping"):
            for value in metrics[family].values():
                self.assertEqual(value, 0.0)

    def test_static_ood_excludes_length_and_catches_slenderness_corruption(self) -> None:
        minimum, maximum, threshold = oracle._static_ood_setup(
            self.train, self.development
        )
        inputs = model.input_from_row(self.development[0])
        legal_length = replace(
            inputs,
            length_m=inputs.length_m * 2.0,
            wall_m=inputs.wall_m * 2.0,
            frequency_scale=inputs.frequency_scale / 2.0,
        )
        self.assertEqual(
            oracle._static_ood_score(inputs, minimum, maximum),
            oracle._static_ood_score(legal_length, minimum, maximum),
        )
        corruption = replace(inputs, slenderness=inputs.slenderness * 20.0)
        self.assertGreater(
            oracle._static_ood_score(corruption, minimum, maximum), threshold
        )

    def test_mutation_matrix_has_three_unique_cases_per_cell(self) -> None:
        identities = set()
        for row in self.interpolation:
            for mutation in oracle.MUTATION_ORDER:
                changed = oracle._mutated_input(row, mutation)
                identity = (
                    row.cell,
                    mutation,
                    changed.material,
                    changed.support,
                    changed.length_m,
                    changed.slenderness,
                )
                identities.add(identity)
                self.assertNotEqual(changed.record(), model.input_from_row(row).record())
        self.assertEqual(len(identities), 72)

    def test_model_decoder_rejects_corrupt_payloads(self) -> None:
        files = model.encode_artifact(self.artifact)
        corrupt = dict(files)
        corrupt["coefficients.npy"] = corrupt["coefficients.npy"][:-1]
        with self.assertRaises(common.B0Error):
            model.decode_artifact(corrupt)
        missing = dict(files)
        del missing["normalization.npy"]
        with self.assertRaises(common.B0Error):
            model.decode_artifact(missing)

    def test_output_boundary_access_ledger_and_canonical_bytes(self) -> None:
        value = np.arange(24, dtype=np.float64).reshape(3, 8)
        self.assertEqual(common.array_bytes(value), common.array_bytes(value.copy()))
        self.assertEqual(
            common.canonical_json({"b": value, "a": 1}),
            common.canonical_json({"a": 1, "b": value.copy()}),
        )
        with self.assertRaises(common.B0Error):
            common.prepare_output(common.repository_root() / "forbidden-b0-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            with self.assertRaises(common.B0Error):
                common.prepare_output(Path(temporary))
            symlink = Path(temporary) / "linked-output"
            symlink.symlink_to(common.EXPERIMENT_ROOT, target_is_directory=True)
            with self.assertRaises(common.B0Error):
                common.prepare_output(symlink)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertIn("opened_v17_g0_artifact_bytes_read", common.ZERO_ACCESS)

    def test_protocol_environment_and_implementation_are_pinned(self) -> None:
        environment = common.verify_protocol_and_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertEqual(set(self.implementation), set(common.IMPLEMENTATION_FILES))
        self.assertEqual(
            common.sha256_file(common.repository_root() / common.PROTOCOL_PATH),
            common.PROTOCOL_SHA256,
        )


if __name__ == "__main__":
    unittest.main()
