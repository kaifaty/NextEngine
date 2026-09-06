from __future__ import annotations

import copy
import hashlib
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parents[2]
SCRIPTS = ROOT / "lab" / "scripts"
sys.path.insert(0, str(SCRIPTS))

import physical_sound_v44_g0b1c0_noise_robust_target_v1 as g0b1c0  # noqa: E402

PROFILE = ROOT / g0b1c0.PROFILE_PATH
PROTOCOL = ROOT / g0b1c0.PROTOCOL_PATH
SCRIPT = ROOT / g0b1c0.OWNER_PATH


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


class NoiseRobustTargetTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.protocol_bytes, cls.protocol = g0b1c0.read_canonical_json(
            PROTOCOL, "protocol"
        )
        cls.profile_bytes, cls.profile = g0b1c0.read_canonical_json(PROFILE, "profile")
        cls.outputs = g0b1c0.build_outputs(cls.profile, cls.protocol)

    def test_tracked_protocol_and_profile_are_canonical_and_bind_owner(self) -> None:
        g0b1c0.validate_protocol(self.protocol)
        g0b1c0.validate_profile(self.profile)
        self.assertEqual(self.protocol_bytes, g0b1c0.canonical_json(self.protocol))
        self.assertEqual(self.profile_bytes, g0b1c0.canonical_json(self.profile))
        owner = next(
            item
            for item in self.profile["dependency_bindings"]
            if item["path"] == g0b1c0.OWNER_PATH
        )
        self.assertEqual(SCRIPT.stat().st_size, owner["bytes"])
        self.assertEqual(sha256(SCRIPT.read_bytes()), owner["sha256"])

    def test_synthetic_controls_admit_target_without_external_values(self) -> None:
        report = self.outputs["report.json"]
        self.assertEqual("NoiseRobustTargetSyntheticAdmitted", report["decision"])
        for gate in (
            "clean_mode_recovery",
            "frequency_shift_sensitivity",
            "mixture_stability",
            "noise_rejection",
            "resource_bounds",
            "scale_invariance",
        ):
            self.assertTrue(report["gates"][gate])
        counters = self.outputs["access-ledger.json"]["counters"]
        for counter in g0b1c0.FORBIDDEN_COUNTERS:
            self.assertEqual(0, counters[counter])

    def test_every_target_has_fixed_dimension_and_closed_partitions(self) -> None:
        rows = self.outputs["audit.json"]["synthetic_targets"]
        self.assertEqual(4, len(rows))
        for row in rows:
            target = row["target"]
            self.assertEqual(75, len(g0b1c0.target_vector(target)))
            self.assertEqual(24, len(target["tonal_excess_energy_ppm_by_band"]))
            self.assertEqual(24, len(target["residual_energy_ppm_by_band"]))
            self.assertEqual(24, len(target["qualified_peak_count_by_band"]))
            self.assertEqual(
                1_000_000,
                sum(target["tonal_excess_energy_ppm_by_band"])
                + sum(target["residual_energy_ppm_by_band"]),
            )
            self.assertEqual(
                target["qualified_peak_count"],
                sum(target["qualified_peak_count_by_band"]),
            )

    def test_noise_and_clean_controls_separate(self) -> None:
        metrics = self.outputs["audit.json"]["metrics"]
        self.assertEqual(3, metrics["clean_qualified_peak_count"])
        self.assertGreaterEqual(metrics["clean_tonal_excess_energy_ppm"], 950_000)
        self.assertEqual(0, metrics["noise_qualified_peak_count"])
        self.assertEqual(0, metrics["noise_tonal_excess_energy_ppm"])
        self.assertEqual(1_000_000, metrics["noise_residual_energy_ppm"])

    def test_positive_power_of_two_scales_are_target_identical(self) -> None:
        fixture = next(
            row
            for row in self.protocol["synthetic_fixtures"]
            if row["fixture_id"] == "modal_plus_noise"
        )
        samples = g0b1c0.synthesize_fixture(fixture, self.protocol["algorithm"])
        baseline = g0b1c0.canonical_json(g0b1c0.extract_target(samples, self.protocol))
        for factor in (0.5, 2.0):
            self.assertEqual(
                baseline,
                g0b1c0.canonical_json(
                    g0b1c0.extract_target(samples * factor, self.protocol)
                ),
            )

    def test_nonfinite_silent_and_protocol_mutations_fail_closed(self) -> None:
        count = self.protocol["algorithm"]["sample_count"]
        with self.assertRaises(g0b1c0.SyntheticTargetError):
            g0b1c0.extract_target(np.zeros(count, dtype=np.float64), self.protocol)
        invalid = np.zeros(count, dtype=np.float64)
        invalid[0] = np.nan
        with self.assertRaises(g0b1c0.SyntheticTargetError):
            g0b1c0.extract_target(invalid, self.protocol)
        mutated = copy.deepcopy(self.protocol)
        mutated["algorithm"]["local_log_power_median_bins"] = 30
        with self.assertRaises(g0b1c0.SyntheticTargetError):
            g0b1c0.validate_protocol(mutated)

    def test_repeated_build_is_byte_identical(self) -> None:
        repeated = g0b1c0.build_outputs(self.profile, self.protocol)
        self.assertEqual(
            g0b1c0.canonical_json(self.outputs), g0b1c0.canonical_json(repeated)
        )

    def test_output_guard_atomic_rejection_and_network_imports(self) -> None:
        with self.assertRaisesRegex(g0b1c0.SyntheticTargetError, "outside"):
            g0b1c0.prepare_output(ROOT / "forbidden-g0b1c0-output")
        with tempfile.TemporaryDirectory(dir="/tmp") as temporary:
            output = Path(temporary) / "published"
            g0b1c0.run(PROFILE, PROTOCOL, output)
            self.assertEqual(
                {
                    "access-ledger.json",
                    "audit.json",
                    "profile.json",
                    "protocol.json",
                    "report.json",
                },
                {path.name for path in output.iterdir()},
            )
            with self.assertRaises(g0b1c0.SyntheticTargetError):
                g0b1c0.run(PROFILE, PROTOCOL, output)
        source = SCRIPT.read_text()
        for forbidden in (
            "import requests",
            "import torch",
            "import urllib",
            "from urllib",
        ):
            self.assertNotIn(forbidden, source)


if __name__ == "__main__":
    unittest.main()
