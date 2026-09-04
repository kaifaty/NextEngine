from __future__ import annotations

import unittest

from lab.scripts.walking_lift_return_discriminator import (
    PEAK_UM,
    Q16,
    cost_q16,
    cycle_controls,
    native_summary,
    target_um,
)
from lab.tests.test_cpu_walking_contact_audit import fixture


class LiftReturnTests(unittest.TestCase):
    def test_exact_peak_endpoints_and_bilateral_symmetry(self):
        for phase in range(72):
            value = target_um(120 + phase, True)
            self.assertEqual(value, target_um(192 + phase, True))
            self.assertEqual(value[::-1], target_um(156 + phase, True))
            self.assertTrue(all(0 <= h <= PEAK_UM for h in value))
            self.assertEqual(cost_q16(value, 120 + phase, True), 0)
        for phase in (0, 6, 30, 36, 42, 66, 71):
            self.assertEqual(target_um(120 + phase, True), (0, 0))
        self.assertEqual(target_um(138, True), (PEAK_UM, 0))
        self.assertEqual(target_um(174, True), (0, PEAK_UM))
        self.assertEqual(target_um(132, True), (PEAK_UM // 2, 0))

    def test_wrong_side_flight_and_heel_raise_are_not_lift(self):
        self.assertEqual(cost_q16((0, 0), 138, True), Q16)
        self.assertEqual(cost_q16((0, PEAK_UM), 138, True), 2 * Q16)
        self.assertEqual(cost_q16((PEAK_UM, PEAK_UM), 138, True), Q16)
        # Heel-only rocking keeps the minimum sole corner at zero.
        self.assertEqual(cost_q16((0, 0), 138, True), cost_q16((0, 0), 174, True))

    def test_return_is_required_not_persistent_lift(self):
        self.assertEqual(cost_q16((PEAK_UM, 0), 150, True), Q16)
        controls = cycle_controls()
        self.assertEqual(controls["exact_lift_and_return"]["sum_q16"], 0)
        ground = controls["grounded_or_heel_only"]["sum_q16"]
        self.assertGreater(ground, 0)
        for key in ("wrong_foot", "left_held_at_peak", "both_held_at_peak"):
            self.assertGreater(controls[key]["sum_q16"], ground)

    def test_stop_extreme_inputs_and_phase_validation(self):
        self.assertEqual(cost_q16((PEAK_UM, PEAK_UM), 0, False), 0)
        self.assertEqual(cost_q16((-(10**30), 10**30), 138, True), 2 * Q16)
        with self.assertRaises(ValueError):
            target_um(-1, True)
        with self.assertRaises(ValueError):
            cost_q16((0,), 138, True)

    def test_native_post_step_frame_uses_action_phase_and_actual_sole(self):
        trace, descriptor, _ = fixture()
        frame = trace["frames"][0][0]
        frame["tick"] = 139  # native post-step tick; the action saw tick 138
        grounded = native_summary([frame], descriptor)["records"][0]
        self.assertEqual(grounded["action_tick"], 138)
        self.assertEqual(grounded["target_um"], [PEAK_UM, 0])
        self.assertEqual(grounded["actual_um"], [0, 0])
        self.assertEqual(grounded["cost_q16"], Q16)
        frame["links"][1]["position_um"][1] += PEAK_UM
        lifted = native_summary([frame], descriptor)["records"][0]
        self.assertEqual(lifted["cost_q16"], 0)
        # Height is not contact authority: contact bits remain unchanged.
        self.assertEqual(frame["contact_flags"], [1, 1])


if __name__ == "__main__":
    unittest.main()
