from __future__ import annotations

import copy
import json
import math
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = (
    LAB / "profiles" / "physical-sound-v34-f0-target-safe-spectral-recovery.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v34_d0_development_tournament_v1 as d0


class DevelopmentTournamentTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.overlay, cls.effective, cls.p0, cls.profile_data = d0.load_context(PROFILE)

    def test_dependencies_role_metadata_and_holdout_boundary_are_exact(self) -> None:
        metadata = d0.validate_role_metadata(self.effective)
        self.assertEqual(metadata["train"]["cases"], 648)
        self.assertEqual(metadata["development"]["cases"], 432)
        self.assertEqual(metadata["train"]["modal_rows"], 6480)
        self.assertEqual(metadata["development"]["modal_rows"], 4320)
        self.assertEqual(d0.ALLOWED_ROLES, ("train", "development"))
        self.assertEqual(d0.sha256_bytes(self.profile_data), d0.f0.PROFILE_SHA256)
        self.assertEqual(
            d0.validate_bound_file(d0.TERMINAL_OWNER_PATH, d0.TERMINAL_OWNER_SHA256)[
                "sha256"
            ],
            d0.TERMINAL_OWNER_SHA256,
        )

    def test_access_ledger_records_rows_models_and_never_holdout(self) -> None:
        ledger = d0.AccessLedger()
        self.assertEqual(ledger.as_dict()["access_phase"], "pre-access")
        ledger.record_row("train")
        ledger.record_row("development")
        ledger.record_model(1811)
        receipt = ledger.as_dict()
        self.assertEqual(receipt["access_phase"], "post-access")
        self.assertEqual(receipt["target_rows_accessed"], 2)
        self.assertEqual(receipt["train_target_rows"], 1)
        self.assertEqual(receipt["development_target_rows"], 1)
        self.assertEqual(receipt["method_holdout_target_rows"], 0)
        self.assertEqual(receipt["official_model_parameters_initialized"], 1811)
        with self.assertRaisesRegex(d0.D0TournamentError, "role access forbidden"):
            ledger.record_row("method_holdout")

    def test_v34_oracle_matches_frozen_expression_and_differs_from_v33(self) -> None:
        context = {
            "curvature_u": 0.12,
            "density": -0.2,
            "difference_u": 0.3,
            "difference_v": -0.1,
            "family": 1.0,
            "geometry": [0.2, -0.3, 0.4],
            "index_a": -0.5,
            "log_frequency": 0.25,
            "loss": 0.1,
            "ordinal": 0.6,
            "p1_contact": -0.4,
            "poisson": 0.2,
            "support": 1.0,
            "surface_a": 0.7,
            "surface_b": -0.25,
            "u": 0.35,
            "youngs": 0.45,
        }
        actual = d0.oracle_targets({"oracle_context": context})
        expected_decay = 0.20 * math.tanh(
            0.40 * context["loss"]
            + 0.30 * context["log_frequency"]
            + 0.17 * context["support"] * context["ordinal"]
            + 0.14 * context["youngs"] * context["density"]
            + 0.10 * context["poisson"] * context["index_a"]
        )
        self.assertEqual(actual[0], expected_decay)
        parent = d0.v33.i0.oracle_targets({"oracle_context": context})
        self.assertFalse(np.array_equal(actual, parent))
        self.assertTrue(np.all(np.isfinite(actual)))

    def test_post_access_fault_is_atomic_scientific_reject(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v34-d0-fault-"
        ) as temporary:
            output = Path(temporary) / "fault"
            ledger = d0.AccessLedger()
            ledger.record_row("train")
            report = d0.post_access_fault(
                output,
                ledger,
                RuntimeError("discarded-owner-fault"),
                "discarded-stage",
                {"max_output_bytes": 1_000_000},
            )
            self.assertEqual(report["decision"], "HardGateReject")
            self.assertEqual(report["status"], "Reject")
            self.assertTrue(output.is_dir())
            self.assertEqual(
                json.loads((output / "report.json").read_text())["decision"],
                "HardGateReject",
            )
            self.assertFalse((output / d0.terminal.FREEZE_NAME).exists())

    def test_pre_access_profile_fault_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v34-d0-pre-") as temporary:
            root = Path(temporary)
            bad = root / "bad-profile.json"
            profile = copy.deepcopy(self.overlay)
            profile["revision"] = 2
            bad.write_bytes(d0.canonical_json(profile))
            output = root / "must-not-exist"
            report = d0.run(bad, output)
            self.assertEqual(report["decision"], "ContractReject")
            self.assertFalse(report["output_published"])
            self.assertEqual(report["access"]["target_rows_accessed"], 0)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
