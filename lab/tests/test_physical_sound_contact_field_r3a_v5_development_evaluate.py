from __future__ import annotations

import sys
import unittest
from pathlib import Path

import numpy as np

SCRIPT_DIRECTORY = Path(__file__).resolve().parents[1] / "scripts"
sys.path.insert(0, str(SCRIPT_DIRECTORY))

import physical_sound_contact_field_r3a_v5_development_evaluate as development


class PhysicalSoundContactFieldR3AV5DevelopmentEvaluateTests(unittest.TestCase):
    def test_output_gain_matches_target_rms_as_float32(self) -> None:
        target = np.asarray([0.0, 0.5, -0.5, 0.25], dtype=np.float64)
        decoded = target * 0.2
        gain = development.matched_output_gain(target, decoded)
        self.assertEqual(gain.dtype, np.dtype("float32"))
        self.assertAlmostEqual(float(gain), 5.0)

    def test_selection_uses_smallest_passing_capacity(self) -> None:
        results = [
            {"capacity_id": "rvq-6kbps", "passed": False},
            {"capacity_id": "rvq-12kbps", "passed": True},
            {"capacity_id": "rvq-24kbps", "passed": True},
        ]
        self.assertEqual(
            development.select_smallest_passing(results), "rvq-12kbps"
        )
        self.assertIsNone(
            development.select_smallest_passing(
                [dict(item, passed=False) for item in results]
            )
        )


if __name__ == "__main__":
    unittest.main()
