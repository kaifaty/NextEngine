#!/usr/bin/env python3
"""Focused dependency/development-only guards for Physical Sound V19 I0."""

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

import physical_sound_v19_i0_common as common
import physical_sound_v19_i0_oracle as oracle

B0_ROOT = Path(
    "/home/kaifaty/.codex/experiments/nextengine/physical-sound/"
    "v18-b0-deterministic-global-run-a"
)
F0_ROOT = Path(
    "/home/kaifaty/.codex/experiments/nextengine/physical-sound/"
    "v19-f0-residual-harmonic-run-a"
)


class PhysicalSoundV19I0OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.dependencies = common.load_dependencies(B0_ROOT, F0_ROOT)
        cls.development = common.f0_common.generate_objects("development")

    def test_protocol_environment_implementation_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_integration_metadata_root_is_frozen_without_generating_values(
        self,
    ) -> None:
        rows = common.integration_rows()
        self.assertEqual(len(rows), 24)
        self.assertEqual(
            common.sha256_bytes(common.canonical_json([row.record() for row in rows])),
            common.f0_common.ROW_ROOTS["integration"],
        )
        self.assertEqual(
            common.ZERO_ACCESS[
                "integration_rows_generated_before_implementation_commit"
            ],
            0,
        )

    def test_i0_execution_is_fail_closed_after_development_rejection(self) -> None:
        self.assertFalse(common.I0_EXECUTION_AUTHORIZED)
        with self.assertRaisesRegex(common.I0Error, "closed by development control"):
            common.generate_integration_objects()
        with self.assertRaisesRegex(common.I0Error, "closed by development control"):
            oracle.run(B0_ROOT, F0_ROOT, common.EXPERIMENT_ROOT / "forbidden-v19-i0")

    def test_exact_b0_and_f0_dependency_trees_load(self) -> None:
        self.assertEqual(
            common.tree_digest(self.dependencies.b0_file_map), common.B0_TREE_DIGEST
        )
        self.assertEqual(
            common.tree_digest(self.dependencies.f0_file_map), common.F0_TREE_DIGEST
        )
        self.assertGreater(self.dependencies.b0_bytes_read, 0)
        self.assertGreater(self.dependencies.f0_bytes_read, 0)
        np.testing.assert_array_equal(
            self.dependencies.gain_scale,
            np.asarray(
                [
                    0.46931169032862163,
                    0.35531018511381657,
                    0.2687902069074183,
                    0.2415016122971205,
                    0.21932470070055649,
                    0.20081183274524714,
                    0.18138609453269658,
                    0.17905253481706596,
                ]
            ),
        )

    def test_dependency_file_set_and_tree_mutations_fail_closed(self) -> None:
        with self.assertRaises(common.I0Error):
            common._validate_artifact_root(
                F0_ROOT, "0" * 64, set(self.dependencies.f0_file_map)
            )
        with self.assertRaises(common.I0Error):
            common._validate_artifact_root(
                F0_ROOT, common.F0_TREE_DIGEST, {"model.npz"}
            )

    def test_loaded_models_predict_development_deterministically(self) -> None:
        item = self.development[0]
        modes = common.global_modes(self.dependencies.b0, item.row)
        self.assertTrue(
            oracle._hard_modes(modes.predicted_frequencies, modes.predicted_damping)
        )
        left = common.f0_model.predict_methods(
            self.dependencies.f0_model,
            self.dependencies.gain_scale,
            item,
        )["candidate"]
        right = common.f0_model.predict_methods(
            self.dependencies.f0_model,
            self.dependencies.gain_scale,
            item,
        )["candidate"]
        np.testing.assert_array_equal(left, right)

    def test_modal_renderer_and_spectrum_identity_are_exact(self) -> None:
        item = self.development[0]
        modes = common.global_modes(self.dependencies.b0, item.row)
        value = oracle._render_modal(
            modes.truth_frequencies,
            modes.truth_damping,
            item.gains[item.accepted_query[:4]],
        )
        self.assertEqual(value.shape, (4, oracle.SAMPLE_COUNT))
        np.testing.assert_allclose(
            oracle._spectrum_rmse(value, value), 0.0, atol=1.0e-12
        )

    def test_hard_mode_mutations_and_dependency_claim_reject(self) -> None:
        item = self.development[0]
        modes = common.global_modes(self.dependencies.b0, item.row)
        negative = modes.predicted_damping.copy()
        negative[0] *= -1.0
        self.assertFalse(oracle._hard_modes(modes.predicted_frequencies, negative))
        unordered = modes.predicted_frequencies.copy()
        unordered[1], unordered[2] = unordered[2], unordered[1]
        self.assertFalse(oracle._hard_modes(unordered, modes.predicted_damping))
        self.assertFalse(
            oracle._dependency_claim_valid("0" * 64, common.F0_TREE_DIGEST)
        )

    def test_all_development_hard_mutations_reject(self) -> None:
        modes = tuple(
            common.global_modes(self.dependencies.b0, item.row)
            for item in self.development
        )
        counts = oracle._hard_mutations(
            self.development,
            modes,
            oracle._fallback_records(self.development),
        )
        self.assertEqual(
            counts,
            {
                "dependency-hash": 12,
                "missing-fallback": 12,
                "negative-damping": 12,
                "unordered-frequency": 12,
            },
        )

    def test_development_attribution_does_not_open_i0(self) -> None:
        preview = oracle.development_preview(B0_ROOT, F0_ROOT)
        attribution = preview["waveform_attribution"]
        self.assertEqual(preview["integration_rows_generated"], 0)
        self.assertLess(
            attribution["field_only"]["early_waveform_nrmse"],
            attribution["global_only"]["early_waveform_nrmse"],
        )
        self.assertFalse(preview["gates"]["early_waveform"])
        self.assertFalse(preview["gates"]["envelope"])
        self.assertTrue(preview["gates"]["spectrum"])

    def test_fallback_completeness_is_exact_set_equality(self) -> None:
        rejected = {("object", 3), ("object", 7)}
        rows = [
            {"object_id": "object", "query_vertex": 3},
            {"object_id": "object", "query_vertex": 7},
        ]
        self.assertTrue(oracle._fallback_complete(rejected, rows))
        self.assertFalse(oracle._fallback_complete(rejected, rows[:-1]))
        self.assertFalse(oracle._fallback_complete(rejected, [*rows, rows[0]]))

    def test_endpoint_and_geometry_serialization_are_deterministic(self) -> None:
        item = self.development[0]
        modes = common.global_modes(self.dependencies.b0, item.row)
        prediction = common.f0_model.predict_methods(
            self.dependencies.f0_model,
            self.dependencies.gain_scale,
            item,
        )["candidate"]
        first = oracle._endpoint_npz((item,), (modes,), (prediction,))
        second = oracle._endpoint_npz((item,), (modes,), (prediction,))
        self.assertEqual(first, second)
        self.assertEqual(common.geometry_npz((item,)), common.geometry_npz((item,)))

    def test_output_boundary_rejects_escape_and_accepts_new_external_child(
        self,
    ) -> None:
        with self.assertRaises(common.I0Error):
            common.prepare_output(common.repository_root() / "forbidden-i0-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            root = Path(temporary)
            staging, _target = common.prepare_output(root / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


if __name__ == "__main__":
    unittest.main()
