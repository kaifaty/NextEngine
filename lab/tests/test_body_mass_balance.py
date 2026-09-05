import copy
import unittest

import numpy as np

from lab.scripts.audit_body_mass_balance import (
    carrier_com_counterfactual,
    center_of_mass,
    mass_positions,
    tilt_degrees,
)


class BodyMassBalanceTests(unittest.TestCase):
    def test_mass_weighted_center_is_not_mean_of_origins(self):
        rows = [
            {"mass": 1, "com": np.array([0, 1, 0])},
            {"mass": 3, "com": np.array([4, 3, 2])},
        ]
        np.testing.assert_array_equal(center_of_mass(rows), [3, 2.5, 1.5])
        np.testing.assert_array_equal(center_of_mass(rows[::-1]), [3, 2.5, 1.5])

    def test_body_rotation_applies_to_local_com_once(self):
        body = {
            "body_id": "body.pelvis",
            "body_token": 7,
            "mass_microkilograms": 2_000_000,
            "center_of_mass_micrometres": [1_000_000, 0, 0],
            "non_colliding_carrier": False,
        }
        frame = {
            "links": [
                {
                    "body_token": 7,
                    "position_um": [2_000_000, 3_000_000, 0],
                    "rotation_q1_30": [0, 0, 759250125, 759250125],
                }
            ]
        }
        before = copy.deepcopy((body, frame))
        row = mass_positions({"bodies": [body]}, frame)[0]
        np.testing.assert_allclose(row["com"], [2, 4, 0], atol=1e-14)
        self.assertEqual((body, frame), before)
        with self.assertRaisesRegex(ValueError, "token set"):
            mass_positions({"bodies": [body]}, {"links": []})

    def test_torso_tilt_can_differ_from_root_and_is_yaw_invariant(self):
        yaw = np.array([[0, 0, 1], [0, 1, 0], [-1, 0, 0]])
        tilt = np.array(
            [[1, 0, 0], [0, np.sqrt(3) / 2, -0.5], [0, 0.5, np.sqrt(3) / 2]]
        )
        self.assertEqual(tilt_degrees(yaw), 0)
        self.assertAlmostEqual(tilt_degrees(tilt), 30)
        self.assertAlmostEqual(tilt_degrees(yaw @ tilt), 30)

    def test_carrier_counterfactual_preserves_neutral_and_bounds_moment_shift(self):
        rows = [
            {
                "id": "body.torso-pitch",
                "mass": 0.25,
                "com": np.array([0.0, 1.0, 0.0]),
                "carrier": True,
            },
            {
                "id": "body.torso-yaw",
                "mass": 0.75,
                "com": np.array([0.0, 1.0, 0.0]),
                "carrier": False,
            },
        ]
        np.testing.assert_array_equal(carrier_com_counterfactual(rows), [0, 0, 0])
        rows[1]["com"][2] = 0.4
        np.testing.assert_array_equal(carrier_com_counterfactual(rows), [0, 0, 0.1])
        rows[0]["id"] = "unknown"
        with self.assertRaises(ValueError):
            carrier_com_counterfactual(rows)

    def test_bad_mass_rejects(self):
        for rows in (
            [],
            [{"mass": 0, "com": np.zeros(3)}],
            [{"mass": float("nan"), "com": np.zeros(3)}],
        ):
            with self.assertRaises(ValueError):
                center_of_mass(rows)


if __name__ == "__main__":
    unittest.main()
