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
SCRIPT = SCRIPTS / "physical_sound_v37_f0_fresh_operator_freeze_v1.py"
PROFILE = (
    LAB / "profiles" / "physical-sound-v37-f0-fresh-query-surface-operator.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v37_f0_fresh_operator_freeze_v1 as f0


def artifact_tree(root: Path) -> dict[str, bytes]:
    return {
        path.relative_to(root).as_posix(): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class FreshOperatorFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = f0.load_profile(PROFILE)
        cls.dependencies, cls.documents = f0.load_dependencies(cls.profile)

    def test_profile_dependencies_authority_and_access_order_are_exact(self) -> None:
        self.assertEqual(f0.sha256_bytes(self.profile_data), f0.PROFILE_SHA256)
        self.assertEqual(self.profile["baseline_commit"], f0.BASELINE_COMMIT)
        self.assertEqual(self.profile["authority"], f0.EXPECTED_AUTHORITY)
        self.assertEqual(self.profile["access_order"], f0.EXPECTED_ACCESS_ORDER)
        self.assertTrue(set(f0.PRIOR_GENERATION_PATHS).issubset(self.documents))
        self.assertIn(f0.CONTRACT_PATH, self.dependencies)

    def test_fresh_identity_sets_are_disjoint_from_v32_through_v36(self) -> None:
        result = f0.validate_fresh_identities(self.profile, self.documents)
        self.assertEqual(
            result["contact_count_by_role"],
            {"development": 6, "method_holdout": 6, "train": 12},
        )
        self.assertEqual(result["fresh_geometry_cells"], 18)
        self.assertEqual(result["fresh_materials"], 3)
        self.assertEqual(result["fresh_surface_functions"], 14)
        self.assertEqual(result["fresh_operator_mixtures"], 8)
        self.assertEqual(result["fresh_truth_formula_ids"], 6)
        self.assertEqual(result["fresh_seeds"], 32)
        intersections = result["intersections_with_v32_through_v36"]
        self.assertIsInstance(intersections, dict)
        self.assertTrue(all(value == 0 for value in intersections.values()))

    def test_science_freezes_small_qso_controls_and_declared_diff(self) -> None:
        science = f0.validate_science(self.profile)
        self.assertEqual(science["candidate_parameter_count"], 12443)
        self.assertEqual(science["candidate_training_steps"], 96)
        self.assertEqual(science["graph_layers"], 2)
        self.assertEqual(science["latent_width"], 24)
        self.assertEqual(science["control_count"], 8)
        self.assertEqual(
            science["contact_truth_components"],
            [
                "qso-v37-local-anisotropic-integral-v1",
                "qso-v37-two-hop-canonical-graph-diffusion-v1",
                "qso-v37-global-low-rank-field-query-interaction-v1",
            ],
        )
        difference = f0.build_scientific_diff(self.profile, self.documents)
        self.assertEqual(tuple(difference["changed_paths"]), f0.EXPECTED_DIFF_PATHS)
        self.assertEqual(
            difference["unchanged_paths"],
            [
                "candidate.learned_axis",
                "inherited_branches",
                "truth.target_axis_count",
            ],
        )

    def test_role_commitments_close_disjoint_case_and_row_algebra(self) -> None:
        commitments = f0.build_role_commitments(self.profile)
        self.assertEqual(commitments["total_cases"], 1512)
        self.assertEqual(commitments["total_modal_rows"], 15120)
        self.assertTrue(
            all(
                value == 0
                for value in commitments[
                    "pairwise_physical_case_intersections"
                ].values()
            )
        )
        roles = commitments["roles"]
        self.assertEqual(roles["train"]["case_count"], 648)
        self.assertEqual(roles["development"]["modal_row_count"], 4320)
        self.assertEqual(roles["method_holdout"]["modal_row_count"], 4320)
        self.assertEqual(
            roles["development"]["strata"],
            {"contact-only": 108, "geometry-only": 216, "joint": 108},
        )

    def test_reused_contact_seed_and_material_physics_are_rejected(self) -> None:
        reused_contact = copy.deepcopy(self.profile)
        prior_v36 = self.documents[f0.V36_PATH]
        prior_contact = prior_v36["fresh_identity"]["contacts"]["development"][1]
        reused_contact["corpus"]["contacts"]["development"][1] = prior_contact
        with self.assertRaisesRegex(f0.F0FreezeError, "intersects V32-V36"):
            f0.validate_fresh_identities(reused_contact, self.documents)

        reused_seed = copy.deepcopy(self.profile)
        reused_seed["seeds"]["case_enumeration_seed"] = 3201
        with self.assertRaisesRegex(f0.F0FreezeError, "intersects V32-V36"):
            f0.validate_fresh_identities(reused_seed, self.documents)

        reused_material = copy.deepcopy(self.profile)
        prior_material = prior_v36["fresh_identity"]["materials"][0]
        current_material = reused_material["corpus"]["materials"][0]
        for key in (
            "density_kg_m3",
            "loss_rate_per_second",
            "poisson_ratio",
            "youngs_modulus_pa",
        ):
            current_material[key] = prior_material[key]
        with self.assertRaisesRegex(f0.F0FreezeError, "intersects V32-V36"):
            f0.validate_fresh_identities(reused_material, self.documents)

    def test_invalid_mixture_science_and_role_overlap_are_rejected(self) -> None:
        invalid_mixture = copy.deepcopy(self.profile)
        invalid_mixture["corpus"]["operator_mixtures"]["train"][0][
            "component_weights"
        ] = ["0.50", "0.30", "0.30"]
        with self.assertRaisesRegex(f0.F0FreezeError, "unit-sum"):
            f0.validate_fresh_identities(invalid_mixture, self.documents)

        invalid_science = copy.deepcopy(self.profile)
        invalid_science["science"]["candidate"]["parameter_count"] = 12444
        with self.assertRaisesRegex(f0.F0FreezeError, "parameter-count"):
            f0.validate_science(invalid_science)

        role_overlap = copy.deepcopy(self.profile)
        role_overlap["corpus"]["role_plan"]["method_holdout"] = copy.deepcopy(
            role_overlap["corpus"]["role_plan"]["development"]
        )
        role_overlap["corpus"]["surface_functions"]["method_holdout"] = copy.deepcopy(
            role_overlap["corpus"]["surface_functions"]["development"]
        )
        role_overlap["corpus"]["operator_mixtures"]["method_holdout"] = copy.deepcopy(
            role_overlap["corpus"]["operator_mixtures"]["development"]
        )
        with self.assertRaisesRegex(f0.F0FreezeError, "identity closure"):
            f0.build_role_commitments(role_overlap)

    def test_complete_cli_twice_is_exact_and_opens_no_values(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v37-f0-repeat-"
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
            self.assertEqual(artifact_tree(outputs[0]), artifact_tree(outputs[1]))
            self.assertEqual(
                set(artifact_tree(outputs[0])),
                {
                    "conformance.json",
                    "evidence.json",
                    "identity-commitments.json",
                    "report.json",
                    "scientific-diff.json",
                },
            )
            report = json.loads((outputs[0] / "report.json").read_text())
            conformance = json.loads((outputs[0] / "conformance.json").read_text())
            self.assertEqual(
                report["decision"], "F0_FRESH_QUERY_SURFACE_OPERATOR_FREEZE_PASS"
            )
            self.assertEqual(
                report["next_authorized_stage"],
                "V37-C0-zero-target-structural-and-cost-census",
            )
            self.assertEqual(report["access"], f0.ZERO_ACCESS)
            self.assertEqual(conformance["access"], f0.ZERO_ACCESS)
            self.assertFalse(report["official_values_opened"])

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v37-f0-failure-"
        ) as temporary:
            root = Path(temporary)
            corrupt_profile = copy.deepcopy(self.profile)
            corrupt_profile["science"]["candidate"]["graph_layers"] = 3
            corrupt_path = root / "corrupt.json"
            corrupt_path.write_bytes(f0.canonical_json(corrupt_profile))
            output = root / "output"
            with self.assertRaisesRegex(f0.F0FreezeError, "profile hash mismatch"):
                f0.run(corrupt_path, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v37-f0-")
                    for path in root.iterdir()
                )
            )
            output.mkdir()
            with self.assertRaisesRegex(f0.F0FreezeError, "fresh external path"):
                f0.run(PROFILE, output)


if __name__ == "__main__":
    unittest.main()
