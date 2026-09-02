from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v36_f0_fresh_role_science_freeze_v1.py"
PROFILE = (
    LAB / "profiles" / "physical-sound-v36-f0-fresh-role-unchanged-science.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v36_f0_fresh_role_science_freeze_v1 as f0


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class FreshRoleScienceFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = f0.load_profile(PROFILE)
        cls.v35_profile = f0.load_dependency_json(cls.profile, "v35_f0_profile")
        cls.v32 = f0.load_dependency_json(cls.profile, "v32_profile")
        cls.v33 = f0.load_dependency_json(cls.profile, "v33_profile")
        cls.v34 = f0.load_dependency_json(cls.profile, "v34_profile")
        cls.parent, cls.current = f0.build_experiments(cls.profile, cls.v35_profile)

    def test_profile_dependencies_authority_and_owner_contract_are_exact(self) -> None:
        dependencies = f0.validate_dependencies(self.profile)
        self.assertEqual(set(dependencies), set(self.profile["parent"]) | {"protocol"})
        self.assertEqual(f0.sha256_bytes(self.profile_data), f0.PROFILE_SHA256)
        self.assertEqual(self.profile["baseline_commit"], f0.BASELINE_COMMIT)
        self.assertEqual(self.profile["authority"], f0.EXPECTED_AUTHORITY)
        contract = f0.validate_access_and_owner_contract(self.profile)
        self.assertEqual(
            contract["contract_schema"],
            "nextengine.experimental-physical-sound-v36-owner-contract.v1",
        )
        self.assertEqual(
            contract["post_access_outcomes"],
            ["Pass", "MetricReject", "HardGateReject", "ResourceReject", "OwnerFault"],
        )

    def test_scientific_projection_is_exact_and_model_seeds_are_unchanged(self) -> None:
        result = f0.validate_unchanged_science(self.profile, self.parent, self.current)
        self.assertEqual(
            result["status"],
            "SEMANTICALLY_EXACT_TO_V35_OUTSIDE_IDENTITY_PATHS",
        )
        self.assertEqual(result["changed_truth_expressions"], 3)
        self.assertEqual(set(result["model_seeds"].values()), {3301, 3302})
        self.assertEqual(
            f0.scientific_projection(self.parent),
            f0.scientific_projection(self.current),
        )

    def test_scientific_or_truth_structure_drift_is_rejected(self) -> None:
        method_drift = copy.deepcopy(self.current)
        method_drift["method_overlay"]["coverage_gate"]["local_weight_cap"] = "0.81"
        with self.assertRaisesRegex(f0.F0FreezeError, "scientific projection"):
            f0.validate_unchanged_science(self.profile, self.parent, method_drift)

        truth_structure_drift = copy.deepcopy(self.current)
        truth_structure_drift["oracle"]["contact"]["bound"] = "0.23"
        with self.assertRaisesRegex(f0.F0FreezeError, "scientific projection"):
            f0.validate_unchanged_science(
                self.profile, self.parent, truth_structure_drift
            )

    def test_fresh_identity_sets_are_disjoint_from_v32_through_v35(self) -> None:
        result = f0.validate_fresh_identities(
            self.profile,
            self.current,
            {
                "V32": self.v32,
                "V33": self.v33,
                "V34": self.v34,
                "V35": self.v35_profile,
            },
        )
        self.assertEqual(
            result["contact_count_by_role"],
            {"development": 6, "method_holdout": 6, "train": 12},
        )
        self.assertEqual(result["fresh_geometry_cells"], 10)
        self.assertEqual(result["fresh_materials"], 3)
        self.assertEqual(result["fresh_truth_expressions"], 3)
        self.assertEqual(result["fresh_identity_seeds"], 10)
        for key, value in result.items():
            if key.startswith("prior_"):
                self.assertEqual(value, 0)

    def test_reused_contact_or_seed_is_rejected(self) -> None:
        reused_contact = copy.deepcopy(self.current)
        reused_contact["corpus"]["contacts"]["development"][1] = ["0.31", "0.19"]
        with self.assertRaisesRegex(f0.F0FreezeError, "contact reused"):
            f0.validate_fresh_identities(
                self.profile,
                reused_contact,
                {
                    "V32": self.v32,
                    "V33": self.v33,
                    "V34": self.v34,
                    "V35": self.v35_profile,
                },
            )

        reused_seed_profile = copy.deepcopy(self.profile)
        reused_seed_profile["fresh_identity"]["seed_identity"]["case_enumeration"] = (
            35001
        )
        with self.assertRaisesRegex(f0.F0FreezeError, "seed reused"):
            f0.validate_fresh_identities(
                reused_seed_profile,
                self.current,
                {
                    "V32": self.v32,
                    "V33": self.v33,
                    "V34": self.v34,
                    "V35": self.v35_profile,
                },
            )

    def test_role_commitments_close_complete_disjoint_algebra(self) -> None:
        result = f0.build_role_commitments(self.profile, self.current)
        self.assertEqual(result["total_cases"], 1512)
        self.assertEqual(result["total_modal_rows"], 15120)
        self.assertEqual(
            result["pairwise_role_case_intersections"],
            {
                "development_method_holdout": 0,
                "train_development": 0,
                "train_method_holdout": 0,
            },
        )
        self.assertEqual(result["roles"]["train"]["case_count"], 648)
        self.assertEqual(result["roles"]["development"]["modal_row_count"], 4320)
        self.assertEqual(result["roles"]["method_holdout"]["modal_row_count"], 4320)

    def test_complete_cli_twice_is_exact_and_opens_no_values(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-f0-repeat-"
        ) as temporary:
            root = Path(temporary)
            outputs = [root / "a", root / "b"]
            stdout: list[bytes] = []
            for output in outputs:
                completed = subprocess.run(
                    [
                        sys.executable,
                        str(SCRIPT),
                        "--profile",
                        str(PROFILE),
                        "--output",
                        str(output),
                    ],
                    cwd=ROOT,
                    check=False,
                    capture_output=True,
                )
                self.assertEqual(completed.returncode, 0, completed.stdout.decode())
                self.assertEqual(completed.stderr, b"")
                stdout.append(completed.stdout)
            self.assertEqual(stdout[0], stdout[1])
            self.assertEqual(all_files(outputs[0]), all_files(outputs[1]))
            self.assertEqual(
                set(all_files(outputs[0])),
                {
                    "conformance.json",
                    "evidence.json",
                    "identity-commitments.json",
                    "report.json",
                },
            )
            report = json.loads((outputs[0] / "report.json").read_text())
            conformance = json.loads((outputs[0] / "conformance.json").read_text())
            self.assertEqual(
                report["decision"],
                "F0_FRESH_ROLE_UNCHANGED_SCIENCE_FREEZE_PASS",
            )
            self.assertEqual(
                report["next_authorized_stage"],
                "V36-C0-fresh-zero-target-structural-census",
            )
            self.assertFalse(report["official_values_opened"])
            self.assertEqual(report["access"], f0.ZERO_ACCESS)
            self.assertEqual(conformance["access"], f0.ZERO_ACCESS)
            self.assertEqual(
                conformance["gates"]["science_semantically_exact_to_v35"], "Pass"
            )

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v36-f0-failure-"
        ) as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["fresh_identity"]["materials"][0]["density_kg_m3"] = "3051"
            bad.write_bytes(f0.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(f0.F0FreezeError, "profile hash mismatch"):
                f0.run(bad, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v36-f0-")
                    for path in root.iterdir()
                )
            )
            output.mkdir()
            with self.assertRaisesRegex(f0.F0FreezeError, "fresh external path"):
                f0.run(PROFILE, output)


if __name__ == "__main__":
    unittest.main()
