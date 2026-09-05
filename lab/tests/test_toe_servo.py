import unittest

from lab.scripts.audit_toe_servo import effort, nearest_even


class ToeServoTest(unittest.TestCase):
    def test_signed_rounding_ties_and_non_ties(self):
        for value, expected in [(1, 0), (3, 2), (5, 2), (7, 4), (8, 4)]:
            self.assertEqual(nearest_even(value, 2), expected)
            self.assertEqual(nearest_even(-value, 2), -expected)
        self.assertEqual(nearest_even(8, 3), 3)
        self.assertEqual(nearest_even(-8, 3), -3)

    def test_rate_intersection_can_oppose_requested_sign(self):
        actuator = self.actuator()
        channel = {
            "target_urad": 0,
            "input_position_urad": 40_000,
            "input_velocity_urad_s": 0,
        }
        self.assertEqual(
            effort(actuator, channel, 1_000_000, 0), (-1_000_000, 500_000, 4, 0)
        )

    def test_power_work_and_infeasible_intersection(self):
        actuator = self.actuator()
        actuator["maximum_effort_rate_micronewton_metres_per_second"] = 24_000_000_000
        channel = {
            "target_urad": 1_000_000,
            "input_position_urad": 0,
            "input_velocity_urad_s": 8_000_000,
        }
        request, applied, flags, work = effort(actuator, channel, 0, 0)
        self.assertGreater(request, 12_000_000)
        self.assertEqual((applied, flags, work), (5_000_000, 50, 166_667))
        self.assertEqual(effort(actuator, channel, 0, 666_667)[1:], (0, 50, 666_667))
        with self.assertRaises(ValueError):
            effort(self.actuator(), channel, 1_000_000, 666_667)

    @staticmethod
    def actuator():
        return {
            "stiffness_q16": 25 * 65536,
            "damping_q16": 1311,
            "effort_micronewton_metres": [-12_000_000, 12_000_000],
            "maximum_effort_rate_micronewton_metres_per_second": 120_000_000,
            "maximum_power_microwatts": 40_000_000,
            "maximum_positive_work_microjoules_per_motor_tick": 666_667,
        }


if __name__ == "__main__":
    unittest.main()
