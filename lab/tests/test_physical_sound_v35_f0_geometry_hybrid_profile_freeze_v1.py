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
SCRIPT = SCRIPTS / "physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v35-f0-geometry-conditioned-hybrid.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1 as f0


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class GeometryHybridProfileFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = f0.load_profile(PROFILE)
        cls.v32 = f0.load_declared_json(cls.profile, "v32_profile")
        cls.v33 = f0.load_declared_json(cls.profile, "v33_profile")
        cls.v34 = f0.load_declared_json(cls.profile, "v34_profile")
        cls.effective = f0.build_effective_profile(cls.profile, cls.v33)

    def test_profile_dependencies_authority_and_preserved_sections_are_exact(
        self,
    ) -> None:
        dependencies = f0.validate_dependencies(self.profile)
        self.assertEqual(
            set(dependencies), set(self.profile["parent"]) | {"protocol", "research"}
        )
        self.assertEqual(f0.sha256_bytes(self.profile_data), f0.PROFILE_SHA256)
        self.assertEqual(self.profile["baseline_commit"], f0.BASELINE_COMMIT)
        self.assertFalse(self.profile["authority"]["model_allowed_after_f0"])
        self.assertTrue(self.profile["authority"]["c0_allowed_after_f0"])
        self.assertFalse(self.profile["authority"]["b0_allowed_after_f0"])
        self.assertEqual(
            f0.validate_preserved_sections(self.profile, self.v33),
            self.profile["preserved_sha256"],
        )

    def test_fresh_roles_counts_and_witness_plan_are_target_free(self) -> None:
        result = f0.validate_freshness_counts_and_witness_plan(
            self.profile,
            self.effective,
            {"V32": self.v32, "V33": self.v33, "V34": self.v34},
        )
        self.assertEqual(result["total_cases"], 1512)
        self.assertEqual(result["total_modal_rows"], 15120)
        self.assertEqual(
            result["contact_count_by_set"],
            {"development": 6, "method_holdout": 6, "train": 12},
        )
        for role in ("development", "method_holdout"):
            self.assertEqual(
                result["predicted_nodal_rows"][role],
                {"contact-only": 180, "geometry-only": 180, "joint": 180},
            )
        self.assertEqual(self.profile["witness_contract"]["target_rows_allowed"], 0)

    def test_geometry_hybrid_controls_local_gate_and_heads_are_closed(self) -> None:
        result = f0.validate_method_freeze(self.profile, self.effective, self.v33)
        self.assertEqual(result["contact_input_count"], 35)
        self.assertEqual(result["candidate_parameter_count"], 1859)
        self.assertEqual(result["control_count"], 9)
        self.assertEqual(result["local_maximum_compatible_rows"], 216)
        self.assertEqual(result["local_weight_cap"], "0.80")
        self.assertEqual(result["contact_bandwidth_hex"], "0x1.e6d4df96cc6b2p-2")
        self.assertEqual(result["geometry_bandwidth_hex"], "0x1.ac6db4c237254p-1")

    def test_oracle_is_fresh_and_four_preflights_precede_target_access(self) -> None:
        result = f0.validate_oracle_and_access_order(
            self.profile,
            self.effective,
            {"V32": self.v32, "V33": self.v33, "V34": self.v34},
        )
        self.assertEqual(result["fresh_truth_expressions"], 3)
        self.assertEqual(result["first_target_stage"], "d0-train-materialize-and-fit")
        self.assertEqual(
            result["pretraining_stages"],
            [
                "f0-identity-freshness-and-static-freeze",
                "c0-signal-blind-witness-and-coverage-census",
                "b0-discarded-local-and-gate-conformance",
                "i0-discarded-whole-owner-terminal-path-proof",
            ],
        )

    def test_complete_cli_twice_is_exact_and_opens_no_values(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v35-f0-repeat-"
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
                {"conformance.json", "evidence.json", "report.json"},
            )
            report = json.loads((outputs[0] / "report.json").read_text())
            conformance = json.loads((outputs[0] / "conformance.json").read_text())
            self.assertEqual(
                report["decision"],
                "F0_TARGET_SAFE_GEOMETRY_HYBRID_PROFILE_FREEZE_PASS",
            )
            self.assertEqual(
                report["next_authorized_stage"],
                "V35-C0-signal-blind-witness-and-coverage-census",
            )
            self.assertFalse(report["official_values_opened"])
            self.assertEqual(report["access"], f0.ZERO_ACCESS)
            self.assertEqual(conformance["access"], f0.ZERO_ACCESS)
            self.assertEqual(conformance["gates"]["prior_generation_values"], "0 Exact")

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v35-f0-failure-"
        ) as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["method_overlay"]["coverage_gate"]["local_weight_cap"] = "1.00"
            bad.write_bytes(f0.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(f0.F0FreezeError, "profile hash mismatch"):
                f0.run(bad, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v35-f0-")
                    for path in root.iterdir()
                )
            )
            output.mkdir()
            with self.assertRaisesRegex(f0.F0FreezeError, "fresh external path"):
                f0.run(PROFILE, output)


if __name__ == "__main__":
    unittest.main()
