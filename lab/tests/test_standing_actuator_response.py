import copy
import unittest

import numpy as np

from lab.scripts.audit_standing_actuator_response import spectrum, summarize


class ActuatorSpectrumTests(unittest.TestCase):
    def test_identifies_24_hz_in_microstep_series(self):
        values = 3 + 2 * np.sin(2 * np.pi * 24 * np.arange(1920) / 240)
        result = spectrum(values)
        self.assertEqual(result["peak_hz"], 24)
        self.assertAlmostEqual(result["power_fraction_20_to_30_hz"], 1)
        self.assertAlmostEqual(result["power_fraction_above_30_hz"], 0)

    def test_identifies_alternating_step_velocity(self):
        result = spectrum((-1.0) ** np.arange(1920))
        self.assertEqual(result["peak_hz"], 120)
        self.assertAlmostEqual(result["power_fraction_above_30_hz"], 1)

    def test_constant_has_no_oscillation_peak(self):
        self.assertIsNone(spectrum(np.ones(1920))["peak_hz"])

    def test_invalid_series_fails(self):
        for values in ([], [1], [1, float("nan")], [[1, 2]]):
            with self.subTest(values=values), self.assertRaises(ValueError):
                spectrum(values)

    def test_summary_validates_order_and_contiguity(self):
        descriptor = {
            "physics_hz": 240,
            "actuators": [{"actuator_id": "a", "dof_ordinal": 0}],
        }
        trace = {
            "schema_version": 9,
            "ordered_actuator_ids": ["a"],
            "actuators": descriptor["actuators"],
            "physics_substeps": 2,
            "substep_samples": [
                {
                    "physics_substep": i,
                    "joints": [
                        {"ordinal": 0, "position_urad": i, "velocity_urad_s": 0}
                    ],
                    "applied_efforts_by_dof_unm": [0],
                }
                for i in (1, 2)
            ],
            "body_schema_hash": "diagnostic-fixture",
            "compiled_descriptor_hash": "diagnostic-fixture",
            "reason": "fixture",
        }
        result = summarize(descriptor, trace)
        self.assertEqual(result["channels"][0]["position_range_rad"], [1e-6, 2e-6])
        for key, value in (("physics_substeps", 3), ("ordered_actuator_ids", ["b"])):
            invalid = copy.deepcopy(trace)
            invalid[key] = value
            with self.assertRaises(ValueError):
                summarize(descriptor, invalid)
        invalid = copy.deepcopy(trace)
        invalid["substep_samples"][1]["physics_substep"] = 3
        with self.assertRaises(ValueError):
            summarize(descriptor, invalid)
        invalid = copy.deepcopy(trace)
        invalid["actuators"][0]["dof_ordinal"] = 1
        with self.assertRaises(ValueError):
            summarize(descriptor, invalid)


if __name__ == "__main__":
    unittest.main()
