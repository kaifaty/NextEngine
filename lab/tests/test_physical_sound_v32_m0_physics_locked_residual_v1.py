from __future__ import annotations

import copy
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np


LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v32_m0_physics_locked_residual_v1.py"
PROFILE = LAB / "profiles" / "physical-sound-v32-m0-physics-locked-residual.v1.json"
P0_PROFILE = LAB / "profiles" / "physical-sound-v31-p0-causal-baseline.v1.json"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v32_m0_physics_locked_residual_v1 as m0  # noqa: E402


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class PhysicsLockedResidualConformanceTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = m0.load_profile(PROFILE)
        cls.p0_profile, _ = m0.p1.p0.load_profile(P0_PROFILE)

    def test_profile_dependencies_counts_and_model_shape_are_exact(self) -> None:
        dependencies = m0.validate_dependencies(self.profile)
        self.assertEqual(set(dependencies), set(self.profile["parent"]) | {"protocol", "research"})
        self.assertEqual(self.profile["corpus"]["counts"]["modal_rows"], 5760)
        self.assertEqual(
            sum(
                self.profile["corpus"]["counts"][role]["modal_rows"]
                for role in ("train", "development", "method_holdout")
            ),
            5760,
        )
        model = m0.ResidualModel(self.profile)
        self.assertEqual(m0.parameter_count(model), 1491)
        self.assertEqual(model.decay[0].in_features, 11)
        self.assertEqual(model.global_gain[0].in_features, 13)
        self.assertEqual(model.contact[0].in_features, 12)

    def test_oracle_features_and_composition_preserve_locked_physics(self) -> None:
        fixture = m0.smoke_fixture(self.profile, self.p0_profile)
        solution = m0.p1.solve_case("test-smoke", fixture, self.p0_profile)
        modes = solution["modal_document"]["modes"]
        rows = [m0.normalized_row(fixture, mode, (1.05, 0.95, 1.05)) for mode in modes]
        self.assertTrue(all(row["decay"].shape == (11,) for row in rows))
        self.assertTrue(all(row["global_gain"].shape == (13,) for row in rows))
        self.assertTrue(all(row["contact"].shape == (12,) for row in rows))
        targets = np.stack([m0.oracle_targets(row) for row in rows])
        self.assertTrue(np.all(np.abs(targets[:, 0]) <= 0.20))
        self.assertTrue(np.all(np.abs(targets[:, 1]) <= 0.16))
        self.assertTrue(np.all(np.abs(targets[:, 2]) <= 0.22))
        composed = m0.compose_modes(modes, targets)
        for original, corrected in zip(modes, composed, strict=True):
            self.assertEqual(corrected["frequency_hz"], original["frequency_hz"])
            self.assertGreater(corrected["decay_per_second"], 0.0)
            if original["contact_participation"] == 0.0:
                self.assertEqual(corrected["signed_gain"], 0.0)
            elif original["signed_gain"] != 0.0:
                self.assertEqual(
                    np.sign(corrected["signed_gain"]), np.sign(original["signed_gain"])
                )

    def test_complete_cli_twice_is_byte_exact_and_opens_no_official_role(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-m0-repeat-") as temporary:
            root = Path(temporary)
            outputs = [root / "a", root / "b"]
            stdout: list[bytes] = []
            environment = os.environ.copy()
            environment.update(
                {
                    "OMP_NUM_THREADS": "1",
                    "OPENBLAS_NUM_THREADS": "1",
                    "MKL_NUM_THREADS": "1",
                    "NUMEXPR_NUM_THREADS": "1",
                }
            )
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
                    env=environment,
                    check=False,
                    capture_output=True,
                )
                self.assertEqual(completed.returncode, 0, completed.stderr.decode())
                self.assertEqual(completed.stderr, b"")
                stdout.append(completed.stdout)
            self.assertEqual(all_files(outputs[0]), all_files(outputs[1]))
            self.assertEqual(stdout[0], stdout[1])
            self.assertEqual(len(all_files(outputs[0])), 3)
            report = json.loads((outputs[0] / "report.json").read_text())
            conformance = json.loads((outputs[0] / "conformance.json").read_text())
            self.assertEqual(report["decision"], "M0_OWNER_CONFORMANCE_PASS")
            self.assertFalse(report["official_candidate_values_opened"])
            self.assertEqual(conformance["access"], m0.ZERO_ACCESS)
            self.assertTrue(conformance["smoke"]["values_discarded"])

    def test_corrupt_profile_and_occupied_output_reject_atomically(self) -> None:
        with tempfile.TemporaryDirectory(prefix="nextengine-v32-m0-failure-") as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["revision"] = 2
            bad.write_bytes(m0.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(m0.M0ConformanceError, "profile hash mismatch"):
                m0.run(bad, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(path.name.startswith(".nextengine-v32-m0-") for path in root.iterdir())
            )
            output.mkdir()
            with self.assertRaisesRegex(m0.M0ConformanceError, "fresh external path"):
                m0.run(PROFILE, output)


if __name__ == "__main__":
    unittest.main()
