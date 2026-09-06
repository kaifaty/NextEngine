#!/usr/bin/env python3
"""Metadata-only guards for the frozen Physical Sound V21 F1a tournament."""

from __future__ import annotations

import os
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

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

import physical_sound_v21_f1_common as common
import physical_sound_v21_f1_model as model
import physical_sound_v21_f1_tournament as tournament


class PhysicalSoundV21F1MetadataTests(unittest.TestCase):
    def test_protocol_environment_lineage_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_all_role_roots_are_frozen_without_opening_meshes_or_values(self) -> None:
        expected_counts = {
            "train": 48,
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
        self.assertEqual(common.row_ledger()["test"]["row_count"], 24)
        with self.assertRaises(common.F1Error):
            common.generate_objects("test", "0" * 40)
        with self.assertRaises(common.F1Error):
            common.generate_objects("integration", "0" * 40)

    def test_analytic_prior_is_mesh_independent_and_has_61_channels(self) -> None:
        primary, twin = common.generate_rows("development")[:2]
        uv = np.asarray([(-0.75, -0.25), (0.0, 0.5), (0.75, 0.25)], dtype=np.float64)
        left = model.prior_features_at(primary, uv)
        right = model.prior_features_at(twin, uv)
        self.assertEqual(left.shape, (3, 61))
        np.testing.assert_array_equal(left, right)
        self.assertTrue(np.isfinite(left).all())

    def test_continuous_basis_identity_periodicity_and_penalty(self) -> None:
        endpoints = np.asarray([(-1.0, 0.0), (1.0, 0.0)], dtype=np.float64)
        cylinder, q2_penalty = model.basis_matrix("Cylinder", endpoints, 2)
        plate, _ = model.basis_matrix("Plate", endpoints, 2)
        _, q3_penalty = model.basis_matrix("Bowl", endpoints, 3)
        self.assertEqual(cylinder.shape, (2, 25))
        self.assertEqual(q2_penalty.shape, (25,))
        self.assertEqual(q3_penalty.shape, (49,))
        np.testing.assert_allclose(cylinder[0], cylinder[1], atol=1.0e-15)
        self.assertFalse(np.allclose(plate[0], plate[1], atol=1.0e-15))
        self.assertTrue(np.all(q2_penalty > 0.0))

    def test_request_identity_and_resource_corruptions_fail_closed(self) -> None:
        item = SimpleNamespace(row=common.generate_rows("development")[0])
        spec = model.CANDIDATES[0]
        size = (1 + 2 * spec.order) ** 2
        model.validate_request(item, spec, expected_role="development")
        corruptions = (
            {"declared_role": "train"},
            {"declared_basis_revision": "unknown"},
            {"declared_regularization": spec.regularization * 10.0},
            {
                "coefficient": np.full(
                    (size, common.MODE_COUNT), np.nan, dtype=np.float64
                )
            },
            {"available_model_bytes": model.PARAMETER_BYTES - 1},
        )
        for corruption in corruptions:
            with (
                self.subTest(corruption=tuple(corruption)),
                self.assertRaises(common.F1Error),
            ):
                model.validate_request(
                    item, spec, expected_role="development", **corruption
                )
        with self.assertRaises(common.F1Error):
            model.validate_request(
                item,
                model.CandidateSpec(
                    spec.candidate_id, spec.order, spec.regularization * 2.0
                ),
                expected_role="development",
            )

    def test_model_budget_candidate_order_and_roundtrip_are_exact(self) -> None:
        self.assertEqual(
            [spec.candidate_id for spec in model.CANDIDATES],
            [
                "continuous-q2-l1e-6",
                "continuous-q2-l1e-4",
                "continuous-q2-l1e-2",
                "continuous-q3-l1e-6",
                "continuous-q3-l1e-4",
                "continuous-q3-l1e-2",
            ],
        )
        torch.manual_seed(model.CANDIDATE_SEED)
        candidates = tuple(
            model.TrainingResult(
                model.PriorNetwork().double().eval(),
                np.ones(common.MODE_COUNT, dtype=np.float64),
                1.0,
                0.5,
                spec,
                model.CANDIDATE_SEED,
            )
            for spec in model.CANDIDATES
        )
        paired = model.TrainingResult(
            model.PriorNetwork().double().eval(),
            np.ones(common.MODE_COUNT, dtype=np.float64),
            1.0,
            0.5,
            None,
            model.PAIRED_HARMONIC_SEED,
        )
        self.assertEqual(model.parameter_count(candidates[0].model), 41_992)
        self.assertEqual(model.parameter_bytes(candidates[0].model), 335_936)
        payload = model.encode_models(candidates, paired)
        self.assertEqual(payload, model.encode_models(candidates, paired))
        decoded, decoded_paired, scale = model.decode_models(payload)
        self.assertEqual(len(decoded), 6)
        self.assertEqual(model.parameter_count(decoded_paired), model.PARAMETER_COUNT)
        np.testing.assert_array_equal(scale, np.ones(common.MODE_COUNT))

    def test_strict_topology_gradient_gate_is_point_five(self) -> None:
        rows = []
        for row in common.generate_rows("development"):
            rows.append(
                {
                    "edge_gradient_p99": 0.51,
                    "gain_nrmse": 0.10,
                    "object_id": row.object_id,
                    "topology": row.topology,
                }
            )
        methods = {
            name: [dict(value) for value in rows] for name in tournament.METHOD_ORDER
        }
        f0_rows = [
            {**value, "edge_gradient_p99": 0.60, "gain_nrmse": 0.20} for value in rows
        ]
        remesh = [
            {
                "gain_metric_drift": 0.01,
                "probe_disagreement_nrmse": 0.01,
            }
            for _ in range(12)
        ]
        f0_remesh = [{"gain_metric_drift": 0.02} for _ in range(12)]
        mutations = [
            {"mutation": mutation, "quality_reject": True}
            for mutation in tournament.MUTATION_ORDER
            for _ in range(12)
        ]
        structural = [{"pre_inference_reject": True} for _ in range(120)]
        coverage = [
            {
                "fallback_complete": True,
                "global_pass": True,
                "local_false_ood_fraction": 0.0,
            }
            for _ in range(24)
        ]
        gates = tournament._candidate_gates(
            methods,
            remesh,
            mutations,
            structural,
            f0_rows,
            f0_remesh,
            coverage,
            True,
        )
        self.assertFalse(gates["topology_gradient_strict"])

    def test_serialization_and_external_output_boundary(self) -> None:
        with self.assertRaises(ValueError):
            common.canonical_json({"invalid": float("inf")})
        self.assertEqual(
            tournament.OUTPUT_FILES,
            {
                "access-ledger.json",
                "corpus.json",
                "geometry.npz",
                "manifest.json",
                "metrics.jsonl",
                "models.npz",
                "predictions.npz",
                "report.json",
                "selection.json",
            },
        )
        with self.assertRaises(common.F1Error):
            common.prepare_output(common.repository_root() / "forbidden-f1-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            staging, _target = common.prepare_output(Path(temporary) / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


if __name__ == "__main__":
    unittest.main()
