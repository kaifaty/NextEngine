from __future__ import annotations

import ast
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
SCRIPT = SCRIPTS / "physical_sound_v34_h0_method_holdout_v1.py"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v34_h0_method_holdout_v1 as h0


class MethodHoldoutTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        _, cls.effective, _, _ = h0.d0.load_context(PROFILE)

    def test_owner_has_no_training_call_and_only_train_holdout_roles(self) -> None:
        tree = ast.parse(SCRIPT.read_bytes())
        calls = {
            node.func.attr
            for node in ast.walk(tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute)
        }
        self.assertNotIn("train_model", calls)
        self.assertEqual(h0.ALLOWED_ROLES, ("train", "method_holdout"))
        self.assertEqual(len(h0.D0_ARTIFACT_SHA256), 10)
        self.assertEqual(
            h0.D0_ARTIFACT_SHA256["candidate-weights.bin"],
            "831fd0a78d0d470f6ba54605d52eb0cff0b88aac483b1a35de4564f679fd80bd",
        )

    def test_weight_decoder_roundtrips_frozen_binary_format(self) -> None:
        model = h0.d0.v33.i0.ResidualModel(
            self.effective,
            self.effective["model"]["heads"]["contact"]["input_count"],
            self.effective["training"]["candidate_seed"],
        )
        data = h0.d0.encode_weights(model)
        decoded = h0.decode_weights(
            data,
            self.effective,
            self.effective["model"]["heads"]["contact"]["input_count"],
            self.effective["training"]["candidate_seed"],
        )
        for name, tensor in model.state_dict().items():
            self.assertTrue(
                h0.d0.v33.torch.equal(tensor, decoded.state_dict()[name]), name
            )
        with self.assertRaisesRegex(h0.H0HoldoutError, "magic"):
            h0.decode_weights(
                b"bad",
                self.effective,
                32,
                self.effective["training"]["candidate_seed"],
            )

    def test_access_ledger_forbids_development_and_training(self) -> None:
        ledger = h0.AccessLedger()
        ledger.record_row("train")
        ledger.record_row("method_holdout")
        ledger.model_parameters_loaded = 3302
        receipt = ledger.as_dict()
        self.assertEqual(receipt["disclosed_train_target_rows_rebuilt"], 1)
        self.assertEqual(receipt["method_holdout_target_rows"], 1)
        self.assertEqual(receipt["model_training_steps"], 0)
        self.assertEqual(receipt["model_parameters_loaded"], 3302)
        with self.assertRaisesRegex(h0.H0HoldoutError, "role access forbidden"):
            ledger.record_row("development")

    def test_holdout_gate_conjunction_passes_only_better_candidate(self) -> None:
        targets = np.zeros((3, 3), dtype=np.float64)
        role = h0.d0.v33.RoleData(
            role="method_holdout",
            row_ids=["geometry", "contact", "joint"],
            features={},
            raw_features={},
            targets=targets,
            cases=[],
            strata={
                "contact-only": np.asarray([1]),
                "geometry-only": np.asarray([0]),
                "joint": np.asarray([2]),
            },
            case_count=3,
            role_local_geometry_groups=3,
        )
        candidate = np.full((3, 3), 0.001, dtype=np.float64)
        controls = {
            name: np.full((3, 3), value, dtype=np.float64)
            for name, value in {
                "identity": 0.02,
                "nearest": 0.01,
                "raw_mlp": 0.015,
                "raw_ridge": 0.018,
                "spectral_ridge": 0.012,
            }.items()
        }
        result = h0.holdout_evaluation(role, candidate, controls, self.effective)
        self.assertTrue(result["pass"])
        self.assertTrue(all(result["gates"].values()))
        rejected = h0.holdout_evaluation(
            role, controls["identity"], controls, self.effective
        )
        self.assertFalse(rejected["pass"])

    def test_missing_d0_artifacts_reject_before_holdout_and_publish_nothing(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v34-h0-pre-") as temporary:
            root = Path(temporary)
            missing = root / "missing-d0"
            output = root / "must-not-exist"
            report = h0.run(PROFILE, missing, output)
            self.assertEqual(report["decision"], "ContractReject")
            self.assertEqual(report["access"]["target_rows_accessed"], 0)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
