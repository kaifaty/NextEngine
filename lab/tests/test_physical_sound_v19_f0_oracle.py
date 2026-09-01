#!/usr/bin/env python3
"""Focused train/development-only guards for Physical Sound V19 F0."""

from __future__ import annotations

import io
import math
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
import torch

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v19_f0_common as common  # noqa: E402
import physical_sound_v19_f0_model as model  # noqa: E402
import physical_sound_v19_f0_oracle as oracle  # noqa: E402


class PhysicalSoundV19F0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.train = common.generate_objects("train")
        cls.development = common.generate_objects("development")
        torch.manual_seed(model.SEED)
        cls.untrained = model.PriorNetwork().double().eval()
        cls.gain_scale = model.fit_gain_scale(cls.train)

    def test_protocol_environment_dependencies_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_all_role_metadata_roots_are_frozen_without_opening_values(self) -> None:
        expected_counts = {
            "train": 24,
            "development": 24,
            "test": 24,
            "integration": 24,
        }
        for role, count in expected_counts.items():
            rows = common.generate_rows(role)
            self.assertEqual(len(rows), count)
            self.assertEqual(
                common.sha256_bytes(
                    common.canonical_json([row.record() for row in rows])
                ),
                common.ROW_ROOTS[role],
            )
        with self.assertRaises(common.F0Error):
            common.generate_objects("integration")

    def test_train_and_development_full_record_roots_are_exact(self) -> None:
        self.assertEqual(len(self.train), 24)
        self.assertEqual(len(self.development), 24)
        self.assertEqual(
            common.sha256_bytes(
                common.canonical_json([item.identity_record() for item in self.train])
            ),
            common.TRAIN_RECORD_ROOT,
        )
        self.assertEqual(
            common.sha256_bytes(
                common.canonical_json(
                    [item.identity_record() for item in self.development]
                )
            ),
            common.DEVELOPMENT_RECORD_ROOT,
        )

    def test_context_budget_and_c0_fallback_are_exact(self) -> None:
        for item in self.train:
            self.assertEqual(
                item.context.size,
                max(16, math.ceil(item.mesh.vertex_count / 3)),
            )
            self.assertEqual(item.rejected_query.size, 0)
        for item in self.development:
            minimum = max(16, math.ceil(item.mesh.vertex_count / 8))
            expected = minimum + (2 if item.is_twin else 0)
            self.assertEqual(item.context.size, expected)
            summary = oracle._coverage_summary(item)
            self.assertTrue(summary["global_pass"])
            self.assertLessEqual(summary["local_false_ood_fraction"], 0.05)

    def test_geometry_normals_curvatures_and_rolled_seam_are_canonical(self) -> None:
        for item in self.development:
            np.testing.assert_allclose(
                np.linalg.norm(item.normals, axis=1), 1.0, atol=1.0e-12
            )
            self.assertTrue(np.isfinite(item.curvatures).all())
            self.assertEqual(
                item.gains.shape, (item.mesh.vertex_count, common.MODE_COUNT)
            )
        rolled = next(
            item
            for item in self.development
            if item.row.topology == "RolledSheet" and not item.is_twin
        )
        for v_index in range(rolled.row.grid_v):
            left = v_index * rolled.row.grid_u
            right = left + rolled.row.grid_u - 1
            np.testing.assert_allclose(
                rolled.mesh.vertices[left], rolled.mesh.vertices[right], atol=1.0e-15
            )
            self.assertGreater(rolled.analysis.all_pairs[left, right], 0.0)

    def test_prior_features_and_model_size_are_frozen(self) -> None:
        features = model.prior_features(self.development[0])
        self.assertEqual(features.shape, (self.development[0].mesh.vertex_count, 61))
        self.assertEqual(model.parameter_count(self.untrained), model.PARAMETER_COUNT)
        self.assertEqual(model.parameter_bytes(self.untrained), model.PARAMETER_BYTES)
        self.assertLessEqual(model.PARAMETER_BYTES, model.MODEL_BYTE_LIMIT)

    def test_candidate_is_finite_exact_on_context_and_masks_rejects(self) -> None:
        for item in self.development:
            prediction = model.predict_methods(self.untrained, self.gain_scale, item)[
                "candidate"
            ]
            np.testing.assert_array_equal(
                prediction[item.context], item.gains[item.context]
            )
            self.assertTrue(np.isfinite(prediction[_active(item)]).all())
            np.testing.assert_array_equal(
                prediction[item.rejected_query],
                np.zeros((item.rejected_query.size, common.MODE_COUNT)),
            )

    def test_model_npz_roundtrip_is_pickle_free_and_prediction_exact(self) -> None:
        payload = model.encode_model(self.untrained, self.gain_scale)
        repeated = model.encode_model(self.untrained, self.gain_scale)
        self.assertEqual(payload, repeated)
        decoded, scale = model.decode_model(payload)
        np.testing.assert_array_equal(scale, self.gain_scale)
        item = self.development[0]
        left = model.predict_methods(self.untrained, self.gain_scale, item)["candidate"]
        right = model.predict_methods(decoded, scale, item)["candidate"]
        np.testing.assert_array_equal(left, right)
        with np.load(io.BytesIO(payload), allow_pickle=False) as archive:
            self.assertIn("gain_scale", archive.files)

    def test_structural_mutations_reject_before_model_inference(self) -> None:
        rows = oracle._structural_metrics(self.development)
        self.assertEqual(len(rows), 60)
        self.assertTrue(all(row["pre_inference_reject"] for row in rows))
        self.assertTrue(all(row["reason"] == "OOD_CONTEXT_BUDGET" for row in rows))

    def test_serialization_and_output_boundary_are_deterministic(self) -> None:
        first = common.geometry_npz(self.development)
        second = common.geometry_npz(common.generate_objects("development"))
        self.assertEqual(first, second)
        with self.assertRaises(ValueError):
            common.canonical_json({"invalid": float("inf")})
        with self.assertRaises(common.coverage_common.C0Error):
            common.deterministic_npz(
                {"invalid": np.asarray([float("nan")], dtype=np.float64)}
            )
        with self.assertRaises(common.F0Error):
            common.prepare_output(common.repository_root() / "forbidden-f0-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            root = Path(temporary)
            staging, _target = common.prepare_output(root / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


def _active(item: common.FieldObject) -> np.ndarray:
    return np.unique(np.concatenate((item.context, item.accepted_query)))


if __name__ == "__main__":
    unittest.main()
