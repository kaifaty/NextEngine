#!/usr/bin/env python3
"""Metadata and algebraic-equivalence guards for Physical Sound V22 F1r."""

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

SCRIPTS = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v22_f1r_common as common
import physical_sound_v22_f1r_model as model
import physical_sound_v22_f1r_tournament as tournament


class PhysicalSoundV22F1rTests(unittest.TestCase):
    def test_protocol_environment_lineage_and_zero_access(self) -> None:
        environment = common.verify_protocol_environment()
        self.assertEqual(environment["libraries"], common.REQUIRED_ENVIRONMENT)
        self.assertTrue(all(value == 0 for value in common.ZERO_ACCESS.values()))
        self.assertEqual(
            set(common.implementation_hashes()), set(common.IMPLEMENTATION_FILES)
        )

    def test_fresh_role_roots_are_metadata_only_and_sealed(self) -> None:
        expected = {"train": 48, "development": 24, "test": 24, "integration": 24}
        for role, count in expected.items():
            rows = common.generate_rows(role)
            self.assertEqual(len(rows), count)
            self.assertEqual(
                common.sha256_bytes(
                    common.canonical_json([row.record() for row in rows])
                ),
                common.ROW_ROOTS[role],
            )
        self.assertEqual(
            common.sha256_bytes(common.canonical_json(common.row_ledger())),
            common.ROW_LEDGER_ROOT,
        )
        with self.assertRaises(common.F1Error):
            common.generate_objects("test", "0" * 40)
        with self.assertRaises(common.F1Error):
            common.generate_objects("integration", "0" * 40)

    def test_reference_and_batched_fixture_agree_before_f1_values(self) -> None:
        first = model.equivalence_report()
        second = model.equivalence_report()
        self.assertEqual(first, second)
        self.assertTrue(first["passed"])
        self.assertEqual(first["tolerance"], 5.0e-12)
        self.assertLessEqual(first["max_absolute_difference"], first["tolerance"])
        self.assertLessEqual(first["max_scaled_difference"], first["tolerance"])

    def test_scientific_constants_and_predecessor_evaluator_are_unchanged(self) -> None:
        self.assertEqual(model.UPDATES, 1_500)
        self.assertEqual(model.CANDIDATE_SEED, 210_001)
        self.assertEqual(model.PAIRED_HARMONIC_SEED, 210_002)
        self.assertEqual(model.PARAMETER_COUNT, 41_992)
        self.assertEqual(model.PARAMETER_BYTES, 335_936)
        self.assertEqual(
            [(value.order, value.regularization) for value in model.CANDIDATES],
            [
                (2, 1.0e-6),
                (2, 1.0e-4),
                (2, 1.0e-2),
                (3, 1.0e-6),
                (3, 1.0e-4),
                (3, 1.0e-2),
            ],
        )
        self.assertIsNot(tournament.predecessor.common, common)
        self.assertIsNot(tournament.predecessor.model, model)
        self.assertEqual(
            tournament.predecessor._candidate_gates.__module__,
            "physical_sound_v21_f1_tournament",
        )

    def test_external_output_boundary_remains_atomic(self) -> None:
        self.assertEqual(len(tournament.OUTPUT_FILES), 9)
        with self.assertRaises(common.F1Error):
            common.prepare_output(common.repository_root() / "forbidden-f1r-output")
        common.EXPERIMENT_ROOT.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=common.EXPERIMENT_ROOT) as temporary:
            staging, _target = common.prepare_output(Path(temporary) / "candidate")
            self.assertTrue(staging.is_dir())
            common.abandon_output(staging)


if __name__ == "__main__":
    unittest.main()
