from __future__ import annotations

import copy
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import numpy as np

LAB = Path(__file__).resolve().parents[1]
ROOT = LAB.parent
SCRIPTS = LAB / "scripts"
SCRIPT = SCRIPTS / "physical_sound_v33_i0_mode_local_spectral_owner_v1.py"
PROFILE = (
    LAB / "profiles" / "physical-sound-v33-f0-mode-local-spectral-residual.v1.json"
)
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v33_i0_mode_local_spectral_owner_v1 as i0


def all_files(root: Path) -> dict[str, bytes]:
    return {
        str(path.relative_to(root)): path.read_bytes()
        for path in sorted(root.rglob("*"))
        if path.is_file()
    }


class ModeLocalSpectralOwnerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile, cls.profile_data = i0.load_profile(PROFILE)
        cls.p0_profile, _ = i0.p1.p0.load_profile(
            ROOT / cls.profile["parent"]["p0_profile"]["path"]
        )

    def test_dependency_authority_and_nonofficial_isolation_are_exact(self) -> None:
        dependencies = i0.validate_dependencies(self.profile)
        self.assertIn("f0_owner", dependencies)
        self.assertIn("f0_result", dependencies)
        self.assertEqual(i0.sha256_bytes(self.profile_data), i0.PROFILE_SHA256)
        v32_profile = i0.f0.load_v32_profile(self.profile)
        self.assertEqual(
            i0.validate_nonofficial_isolation(self.profile, v32_profile),
            {"contacts": 6, "geometry_cells": 1, "materials": 1},
        )
        self.assertEqual(i0.ZERO_OFFICIAL_ACCESS["official_train_target_rows"], 0)
        self.assertEqual(i0.ZERO_OFFICIAL_ACCESS["development_target_rows"], 0)
        self.assertEqual(i0.ZERO_OFFICIAL_ACCESS["method_holdout_target_rows"], 0)

    def test_feature_lift_stencil_models_and_controls_are_complete(self) -> None:
        data = i0.build_nonofficial_data(self.profile, self.p0_profile)
        self.assertEqual(len(data.cases), 18)
        self.assertEqual(len(data.row_ids), 180)
        self.assertEqual(data.features["contact"].shape, (180, 32))
        self.assertEqual(data.raw_features["contact"].shape, (180, 12))
        self.assertEqual(data.features["decay"].shape, (180, 11))
        self.assertEqual(data.features["global_gain"].shape, (180, 13))
        self.assertEqual(data.targets.shape, (180, 3))
        boundary = i0.boundary_conformance(data, 0.0625)
        self.assertEqual(boundary["clamped_edge_probes"], 4)
        candidate = i0.ResidualModel(self.profile, 32, 3301)
        raw = i0.ResidualModel(self.profile, 12, 3302)
        self.assertEqual(i0.parameter_count(candidate), 1811)
        self.assertEqual(i0.parameter_count(raw), 1491)
        controls = i0.control_conformance(data)
        self.assertEqual(
            set(controls), {"identity", "nearest", "raw_ridge", "spectral_ridge"}
        )

    def test_nearest_tie_break_is_lexicographic(self) -> None:
        train_x = np.asarray([[0.0], [0.0], [2.0]], dtype=np.float64)
        train_y = np.asarray([7.0, 3.0, 9.0], dtype=np.float64)
        predicted = i0.predict_nearest(
            train_x,
            train_y,
            np.asarray([[0.0]], dtype=np.float64),
            ["row-z", "row-a", "row-m"],
        )
        np.testing.assert_array_equal(predicted, np.asarray([3.0]))

    def test_complete_cli_twice_is_exact_and_opens_no_official_values(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v33-i0-repeat-"
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
            self.assertEqual(report["decision"], "I0_OWNER_CONFORMANCE_PASS")
            self.assertFalse(report["official_values_opened"])
            for field, expected in i0.ZERO_OFFICIAL_ACCESS.items():
                self.assertEqual(conformance["access"][field], expected)
            self.assertTrue(conformance["hard"]["pass"])
            self.assertEqual(conformance["model"]["candidate_parameter_count"], 1811)

    def test_corrupt_occupied_and_late_failure_are_atomic(self) -> None:
        with tempfile.TemporaryDirectory(
            prefix="nextengine-v33-i0-failure-"
        ) as temporary:
            root = Path(temporary)
            bad = root / "bad.json"
            profile = copy.deepcopy(self.profile)
            profile["revision"] = 2
            bad.write_bytes(i0.canonical_json(profile))
            output = root / "output"
            with self.assertRaisesRegex(i0.f0.F0FreezeError, "profile hash mismatch"):
                i0.run(bad, output)
            self.assertFalse(output.exists())

            output.mkdir()
            with self.assertRaisesRegex(i0.I0ConformanceError, "fresh external path"):
                i0.run(PROFILE, output)
            output.rmdir()

            with (
                mock.patch.object(
                    i0, "build_conformance", side_effect=RuntimeError("late failure")
                ),
                self.assertRaisesRegex(RuntimeError, "late failure"),
            ):
                i0.run(PROFILE, output)
            self.assertFalse(output.exists())
            self.assertFalse(
                any(
                    path.name.startswith(".nextengine-v33-i0-")
                    for path in root.iterdir()
                )
            )


if __name__ == "__main__":
    unittest.main()
