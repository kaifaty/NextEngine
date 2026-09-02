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
PROFILE = LAB / "profiles" / "physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v35_d0_development_tournament_v1 as d0


class DevelopmentTournamentTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        (
            cls.overlay,
            cls.effective,
            cls.p0,
            cls.profile_data,
            cls.dependencies,
        ) = d0.load_context(PROFILE)

    def test_dependencies_roles_method_shape_and_holdout_boundary_are_exact(
        self,
    ) -> None:
        metadata = d0.validate_role_metadata(self.effective)
        self.assertEqual(metadata["train"]["cases"], 648)
        self.assertEqual(metadata["development"]["cases"], 432)
        self.assertEqual(metadata["train"]["modal_rows"], 6480)
        self.assertEqual(metadata["development"]["modal_rows"], 4320)
        self.assertEqual(d0.ALLOWED_ROLES, ("train", "development"))
        self.assertEqual(d0.sha256_bytes(self.profile_data), d0.b0.f0.PROFILE_SHA256)
        self.assertEqual(
            set(self.dependencies),
            {
                "b0_owner",
                "c0_owner",
                "i0_owner",
                "i0_result",
                "terminal_owner",
                "training_primitives",
            },
        )
        method = self.effective["method_overlay"]
        self.assertEqual(method["candidate"]["model"]["parameter_count"], 1859)
        self.assertEqual(len(method["controls"]), 9)
        self.assertEqual(len(method["ablations"]), 3)

    def test_access_ledger_never_opens_holdout_or_prior_values(self) -> None:
        ledger = d0.AccessLedger()
        self.assertEqual(ledger.as_dict()["access_phase"], "pre-access")
        ledger.record_row("train")
        ledger.record_row("development")
        for kind in (
            "candidate",
            "raw_mlp",
            "v34_shaped_spectral_mlp",
            "without_explicit_geometry",
        ):
            ledger.record_model(d0.model_spec(self.effective, kind)[2])
        receipt = ledger.as_dict()
        self.assertEqual(receipt["target_rows_accessed"], 2)
        self.assertEqual(receipt["train_target_rows"], 1)
        self.assertEqual(receipt["development_target_rows"], 1)
        self.assertEqual(receipt["method_holdout_target_rows"], 0)
        self.assertEqual(receipt["official_model_parameters_initialized"], 6972)
        self.assertEqual(receipt["prior_generation_target_values_read"], 0)
        with self.assertRaisesRegex(d0.D0TournamentError, "role access forbidden"):
            ledger.record_row("method_holdout")

    def test_v35_oracle_matches_frozen_geometry_expression(self) -> None:
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
            "v": 0.65,
            "youngs": 0.45,
        }
        actual = d0.oracle_targets({"oracle_context": context})
        expected_contact = 0.22 * math.tanh(
            0.21 * context["p1_contact"]
            + 0.14 * context["difference_u"]
            - 0.12 * context["difference_v"]
            + 0.16 * context["surface_a"]
            + 0.15 * context["surface_b"]
            + 0.13 * context["geometry"][0] * context["p1_contact"]
            + 0.12 * context["geometry"][1] * (2.0 * context["u"] - 1.0)
            + 0.10 * context["geometry"][2] * (2.0 * context["v"] - 1.0)
            + 0.08 * context["ordinal"] * context["support"]
            + 0.06 * context["log_frequency"] * context["curvature_u"]
        )
        self.assertEqual(actual[2], expected_contact)
        self.assertTrue(np.all(np.isfinite(actual)))

    def test_geometry_ablation_preserves_group_and_removes_only_distance_fields(
        self,
    ) -> None:
        source = d0.b0.discarded_row(
            family=0,
            ordinal=2,
            material=d0.b0.DISCARDED_MATERIALS[0],
            geometry=d0.b0.DISCARDED_GEOMETRY[1],
            contact=d0.b0.DISCARDED_CONTACTS[2],
            trace_id="discarded/d0-ablation",
        )
        ablated = d0.without_geometry_row(source)
        self.assertEqual(ablated.causal_key[:4], source.causal_key[:4])
        self.assertEqual(ablated.causal_key[4:7], (0.0, 0.0, 0.0))
        self.assertEqual(ablated.causal_key[7:], source.causal_key[7:])
        self.assertEqual(ablated.group_key, source.group_key)
        self.assertEqual(ablated.target, source.target)

    def test_discarded_training_executes_all_four_frozen_model_paths(self) -> None:
        effective = copy.deepcopy(self.effective)
        effective["training"]["steps"] = 2
        count = 12
        rng = np.random.Generator(np.random.PCG64(35004))
        features = {
            "decay": rng.normal(0.0, 0.2, size=(count, 11)),
            "global_gain": rng.normal(0.0, 0.2, size=(count, 13)),
            "contact": rng.normal(0.0, 0.2, size=(count, 35)),
        }
        role = d0.RoleData(
            role="discarded",
            row_ids=[f"discarded/d0/o{index:02d}" for index in range(count)],
            features=features,
            no_geometry_features={
                "decay": np.array(features["decay"], copy=True),
                "global_gain": np.array(features["global_gain"], copy=True),
                "contact": np.array(features["contact"][:, :32], copy=True),
            },
            raw_features={
                "decay": np.array(features["decay"], copy=True),
                "global_gain": np.array(features["global_gain"], copy=True),
                "contact": np.array(features["contact"][:, :12], copy=True),
            },
            targets=rng.uniform(-0.1, 0.1, size=(count, 3)),
            cases=[],
            strata={},
            local_rows=[],
            geometry_keys=[],
            contact_keys=[],
            case_count=0,
            role_local_geometry_groups=0,
        )
        support = d0.HybridSupport(
            local=np.linspace(-0.05, 0.05, count),
            local_weight=np.full(count, 0.8),
            normalized_squared_distance=np.zeros(count),
            contributor_minimum=215,
            contributor_maximum=215,
            ood_count=0,
        )
        d0.primitives.i0.configure_torch()
        for kind, selected_support in (
            ("candidate", support),
            ("raw_mlp", None),
            ("v34_shaped_spectral_mlp", None),
            ("without_explicit_geometry", support),
        ):
            model, receipt = d0.train_model(role, effective, kind, selected_support)
            predicted, neural = d0.model_predictions(
                model, role, kind, selected_support
            )
            self.assertEqual(receipt["steps"], 2)
            self.assertEqual(predicted.shape, (count, 3))
            self.assertEqual(neural.shape, (count, 3))
            self.assertTrue(np.all(np.isfinite(predicted)))
            self.assertTrue(d0.encode_weights(model).startswith(d0.WEIGHTS_MAGIC))

    def test_serialization_and_post_access_fault_are_atomic(self) -> None:
        fitted = {
            "raw_ridge": {
                "contact": {"beta": np.asarray([1.0, -2.0]), "intercept": 0.5}
            }
        }
        encoded = d0.encode_ridges(fitted)
        self.assertTrue(encoded.startswith(d0.RIDGE_MAGIC))
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v35-d0-fault-"
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
            self.assertEqual(
                json.loads((output / "report.json").read_text())["decision"],
                "HardGateReject",
            )
            self.assertFalse((output / d0.terminal.FREEZE_NAME).exists())

    def test_pre_access_profile_fault_publishes_nothing(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v35-d0-pre-") as temporary:
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
