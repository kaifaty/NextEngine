from __future__ import annotations

import copy
import json
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
PROFILE = (
    LAB / "profiles" / "physical-sound-v33-f0-mode-local-spectral-residual.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v33_d0_development_tournament_v1 as d0


class DevelopmentTournamentOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = d0.load_profile(PROFILE)
        cls.p0_profile, _ = d0.i0.p1.p0.load_profile(
            ROOT / cls.profile["parent"]["p0_profile"]["path"]
        )

    def test_dependency_and_role_metadata_closure_are_exact_without_values(
        self,
    ) -> None:
        dependencies = d0.validate_dependencies(self.profile)
        self.assertIn("i0_owner", dependencies)
        self.assertIn("i0_result", dependencies)
        self.assertEqual(d0.sha256_bytes(self.profile_data), d0.PROFILE_SHA256)
        metadata = d0.validate_role_metadata(self.profile)
        self.assertEqual(metadata["train"]["cases"], 648)
        self.assertEqual(metadata["train"]["modal_rows"], 6480)
        self.assertEqual(metadata["development"]["cases"], 432)
        self.assertEqual(metadata["development"]["modal_rows"], 4320)
        self.assertEqual(
            {
                name: values["modal_rows"]
                for name, values in metadata["development"]["strata"].items()
            },
            {"contact-only": 1080, "geometry-only": 2160, "joint": 1080},
        )

    def test_fixture_projection_and_holdout_access_guard_are_exact(self) -> None:
        fixture, multipliers = d0.build_fixture(
            self.profile,
            self.p0_profile,
            family_index=2,
            material_index=1,
            geometry_cell_index=7,
            contact_set="development",
            contact_index=3,
        )
        self.assertEqual(fixture["support"], "simply-supported-both-ends")
        self.assertEqual(
            fixture["contact"], {"normal_impulse_ns": "1", "u": "0.63", "v": "0.91"}
        )
        self.assertEqual(multipliers, (1.18, 0.78, 0.88))
        with self.assertRaisesRegex(d0.D0TournamentError, "role access forbidden"):
            d0.build_role("method_holdout", self.profile, self.p0_profile)

    def test_metrics_ratio_and_serialization_are_stable(self) -> None:
        expected = np.asarray([[0.1, -0.1, 0.05], [0.2, 0.0, -0.05]], dtype=np.float64)
        predicted = np.asarray(
            [[0.11, -0.08, 0.04], [0.18, 0.01, -0.03]], dtype=np.float64
        )
        strata = {
            "geometry-only": np.asarray([0], dtype=np.int64),
            "joint": np.asarray([1], dtype=np.int64),
        }
        metrics = d0.metric_set(predicted, expected, strata)
        self.assertGreater(metrics["aggregate_normalized_rmse"], 0.0)
        self.assertEqual(set(metrics["contact_stratum_rmse"]), set(strata))
        self.assertEqual(d0.ratio(0.0, 0.0), (0.0, True))
        self.assertEqual(d0.ratio(1.0, 0.0), (0.0, False))
        encoded_a = d0.encode_predictions(["a", "b"], predicted)
        encoded_b = d0.encode_predictions(["a", "b"], predicted)
        self.assertEqual(encoded_a, encoded_b)
        self.assertTrue(encoded_a.startswith(d0.PREDICTIONS_MAGIC))

    def test_compare_runs_detects_exact_and_drift(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v33-d0-compare-"
        ) as temporary:
            root = Path(temporary)
            path_a = root / "a"
            path_b = root / "b"
            path_a.mkdir()
            path_b.mkdir()
            (path_a / "report.json").write_bytes(b"same")
            (path_b / "report.json").write_bytes(b"same")
            self.assertEqual(
                d0.compare_runs(path_a, path_b)["decision"], "REPEAT_EXACT"
            )
            (path_b / "report.json").write_bytes(b"different")
            self.assertEqual(
                d0.compare_runs(path_a, path_b)["decision"],
                "REJECT_NONDETERMINISTIC",
            )

    def test_corrupt_profile_and_occupied_output_reject_before_values(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v33-d0-failure-"
        ) as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["revision"] = 2
            bad.write_bytes(d0.canonical_json(profile))
            with self.assertRaisesRegex(
                d0.i0.f0.F0FreezeError, "profile hash mismatch"
            ):
                d0.load_profile(bad)
            output = root / "occupied"
            output.mkdir()
            with self.assertRaisesRegex(d0.D0TournamentError, "fresh external path"):
                d0.external_output(output)
            self.assertEqual(
                json.loads(d0.canonical_json({"b": 1, "a": 2})), {"a": 2, "b": 1}
            )


if __name__ == "__main__":
    unittest.main()
