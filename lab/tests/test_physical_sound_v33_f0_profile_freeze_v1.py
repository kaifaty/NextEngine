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
SCRIPT = SCRIPTS / "physical_sound_v33_f0_profile_freeze_v1.py"
PROFILE = (
    LAB / "profiles" / "physical-sound-v33-f0-mode-local-spectral-residual.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v33_f0_profile_freeze_v1 as f0


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class FreshRoleSpectralFreezeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = f0.load_profile(PROFILE)
        cls.v32_profile = f0.load_v32_profile(cls.profile)

    def test_profile_dependency_and_authority_closure_are_exact(self) -> None:
        dependencies = f0.validate_dependencies(self.profile)
        self.assertEqual(
            set(dependencies), set(self.profile["parent"]) | {"protocol", "research"}
        )
        self.assertEqual(f0.sha256_bytes(self.profile_data), f0.PROFILE_SHA256)
        self.assertEqual(self.profile["baseline_commit"], f0.BASELINE_COMMIT)
        self.assertFalse(self.profile["authority"]["real_signal_allowed"])
        self.assertFalse(self.profile["authority"]["network_allowed"])
        self.assertTrue(self.profile["authority"]["authored_fallback_required"])

    def test_fresh_roles_counts_features_and_models_are_exact(self) -> None:
        freshness = f0.validate_freshness_and_counts(self.profile, self.v32_profile)
        features = f0.validate_features_models_and_gates(self.profile, self.v32_profile)
        self.assertEqual(freshness["total_cases"], 1512)
        self.assertEqual(freshness["total_modal_rows"], 15120)
        self.assertEqual(freshness["unique_geometry_groups"], 90)
        self.assertEqual(
            freshness["contact_count_by_set"],
            {"development": 6, "method_holdout": 6, "train": 12},
        )
        self.assertEqual(features["contact_feature_count"], 32)
        self.assertEqual(features["fourier_feature_count"], 16)
        self.assertEqual(features["local_stencil_feature_count"], 4)
        self.assertEqual(features["candidate_parameter_count"], 1811)
        self.assertEqual(features["raw_mlp_parameter_count"], 1491)
        self.assertEqual(features["truth_expressions_distinct_from_v32"], 3)

    def test_complete_cli_twice_is_exact_and_opens_no_values(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v33-f0-repeat-"
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
                self.assertEqual(completed.returncode, 0, completed.stderr.decode())
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
            self.assertEqual(report["decision"], "F0_PROFILE_FREEZE_PASS")
            self.assertFalse(report["official_values_opened"])
            self.assertEqual(conformance["access"], f0.ZERO_ACCESS)
            self.assertEqual(conformance["freshness_and_counts"]["total_cases"], 1512)

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v33-f0-failure-"
        ) as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["revision"] = 2
            bad.write_bytes(f0.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(f0.F0FreezeError, "profile hash mismatch"):
                f0.run(bad, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v33-f0-")
                    for path in root.iterdir()
                )
            )
            output.mkdir()
            with self.assertRaisesRegex(f0.F0FreezeError, "fresh external path"):
                f0.run(PROFILE, output)


if __name__ == "__main__":
    unittest.main()
