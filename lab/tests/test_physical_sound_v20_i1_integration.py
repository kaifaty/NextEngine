#!/usr/bin/env python3
"""Metadata-only and fail-closed tests for Physical Sound V20 I1."""

from __future__ import annotations

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

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v20_i1_common as common
import physical_sound_v20_i1_integration as integration

EXPERIMENT_ROOT = Path("/home/kaifaty/.codex/experiments/nextengine/physical-sound")
B0_ROOT = EXPERIMENT_ROOT / "v18-b0-deterministic-global-run-a"
F0_ROOT = EXPERIMENT_ROOT / "v19-f0-residual-harmonic-run-a"
M0B_ROOT = EXPERIMENT_ROOT / "v20-m0b-confound-resistant-run-a"


class PhysicalSoundV20I1IntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.certificate = common.load_m0b_certificate(M0B_ROOT)
        cls.dependencies = common.load_dependencies(B0_ROOT, F0_ROOT)

    def test_protocol_environment_lineage_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            common.tree_digest(self.certificate.file_map), common.M0B_TREE_DIGEST
        )
        self.assertTrue(self.certificate.report["single_run_pass"])
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_i1_metadata_root_and_identity_are_frozen_without_mesh_values(
        self,
    ) -> None:
        rows = common.i1_rows()
        self.assertEqual(len(rows), 24)
        self.assertEqual(
            common.sha256_bytes(common.canonical_json([row.record() for row in rows])),
            common.I1_ROW_ROOT,
        )
        self.assertEqual({row.halton_index for row in rows}, set(range(1701, 1713)))
        self.assertEqual(len({row.object_id for row in rows}), 24)
        self.assertEqual(len({row.physical_group_id for row in rows}), 12)
        self.assertTrue(all(row.role == "integration" for row in rows))
        self.assertTrue(all(row.object_id.startswith("v20-i1-") for row in rows))

    def test_primary_twin_grid_contract_is_exact(self) -> None:
        rows = common.i1_rows()
        for primary, twin in zip(rows[::2], rows[1::2], strict=True):
            self.assertTrue(primary.object_id.endswith("-primary"))
            self.assertTrue(twin.object_id.endswith("-twin"))
            self.assertEqual(primary.physical_group_id, twin.physical_group_id)
            self.assertEqual(twin.grid_u, primary.grid_u + 1)
            self.assertEqual(twin.grid_v, primary.grid_v - 1)
            self.assertEqual(primary.length_m, twin.length_m)
            self.assertEqual(primary.wall_m, twin.wall_m)

    def test_metadata_mutation_changes_frozen_root(self) -> None:
        rows = list(common.i1_rows())
        rows[0] = replace(rows[0], grid_u=rows[0].grid_u + 1)
        observed = common.sha256_bytes(
            common.canonical_json([row.record() for row in rows])
        )
        self.assertNotEqual(observed, common.I1_ROW_ROOT)

    def test_value_generation_requires_exact_clean_implementation_commit(self) -> None:
        with self.assertRaises(common.I1Error):
            common.generate_i1_objects("")
        with self.assertRaises(common.I1Error):
            common.generate_i1_objects("0" * 40)

    def test_control_set_and_thresholds_match_p0c(self) -> None:
        self.assertEqual(
            tuple(item.name for item in integration.control_specs()),
            integration.CONTROL_NAMES,
        )
        self.assertEqual(len(integration.CONTROL_NAMES), 11)
        self.assertEqual(integration.MRSC_THRESHOLD, 0.6860209112961239)
        self.assertEqual(integration.MCLM_THRESHOLD, 0.46162132517706367)
        self.assertEqual(integration.DSR_THRESHOLD, 0.14714357326302258)
        self.assertEqual(integration.TE_THRESHOLD, 0.09215386292722896)

    def test_actual_gate_boundary_is_inclusive(self) -> None:
        summary = {
            "combined": {
                "metrics": {
                    "decay_slope_residual": {"p95": integration.DSR_THRESHOLD},
                    "mean_centered_log_magnitude": {"p95": integration.MCLM_THRESHOLD},
                    "mrsc": {"p95": integration.MRSC_THRESHOLD},
                    "transient_energy": {"p95": integration.TE_THRESHOLD},
                }
            }
        }
        self.assertTrue(all(integration._actual_gates(summary).values()))
        summary["combined"]["metrics"]["mrsc"]["p95"] += 1.0e-12
        self.assertFalse(integration._actual_gates(summary)["mrsc"])

    def test_certificate_and_output_boundaries_fail_closed(self) -> None:
        with self.assertRaises(common.I1Error):
            common.load_m0b_certificate(common.repository_root())
        with self.assertRaises(common.I1Error):
            common.prepare_output(common.repository_root() / "forbidden-i1-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            staging, _target = common.prepare_output(Path(temporary) / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


if __name__ == "__main__":
    unittest.main()
