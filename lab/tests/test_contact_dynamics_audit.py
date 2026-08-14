from __future__ import annotations

import unittest

import numpy as np

from next_lab.contact_dynamics_audit import (
    fixed_pd_proxy_metrics,
    select_counterfactual_cases,
)


class ContactDynamicsAuditTests(unittest.TestCase):
    def test_fixed_pd_proxy_applies_reference_target_slew(self) -> None:
        descriptor = {
            "motor_hz": 60,
            "actuators": [
                _actuator(0, stiffness=65_536, damping=65_536),
                _actuator(1, stiffness=131_072, damping=0),
            ],
        }
        result = fixed_pd_proxy_metrics(
            joint_position_urad=np.asarray(
                ((0, 0), (20, 5), (40, 10)), dtype=np.int64
            ),
            joint_velocity_urad_s=np.asarray(
                ((100, 0), (200, 0), (300, 0)), dtype=np.int64
            ),
            reference_frame=np.asarray((25, 26, 27), dtype=np.int64),
            descriptor=descriptor,
        )

        self.assertEqual(result["target_slew_activation_count"], 2)
        self.assertEqual(
            result["initial_damping"]["utilization_basis_points"], 1000
        )
        self.assertEqual(
            result["maximum_damping"]["utilization_basis_points"], 3000
        )
        self.assertEqual(
            result["maximum_one_frame_lag_requested"][
                "absolute_effort_micronewton_metres"
            ],
            200,
        )
        self.assertEqual(
            result["maximum_one_frame_lag_effort_rate"][
                "utilization_basis_points"
            ],
            11000,
        )
        self.assertEqual(
            result["maximum_one_frame_lag_effort_rate"]["reference_frame"],
            27,
        )

    def test_counterfactual_selection_prefers_orthogonal_causes(self) -> None:
        records = [
            _case(
                2,
                reasons=("hard_impact",),
                early=True,
                derivative=False,
                geometry=True,
                acceleration=10_000,
                jerk=10_400,
            ),
            _case(
                7,
                reasons=("hard_impact",),
                early=True,
                derivative=True,
                geometry=True,
                acceleration=55_000,
                jerk=116_800,
            ),
            _case(
                10,
                reasons=("hard_rom",),
                early=False,
                derivative=True,
                geometry=False,
                acceleration=83_600,
                jerk=327_300,
            ),
            _case(
                14,
                reasons=("joint_velocity",),
                early=False,
                derivative=True,
                geometry=False,
                acceleration=82_800,
                jerk=320_100,
            ),
        ]

        self.assertEqual(
            select_counterfactual_cases(records, maximum_count=2), [2, 10]
        )


def _actuator(
    ordinal: int, *, stiffness: int, damping: int
) -> dict[str, object]:
    return {
        "dof_ordinal": ordinal,
        "joint_id": f"joint.{ordinal}",
        "stiffness_q16": stiffness,
        "damping_q16": damping,
        "effort_micronewton_metres": (-1000, 1000),
        "maximum_effort_rate_micronewton_metres_per_second": 6000,
        "target_delta_microradians_per_motor_tick": (-10, 10),
    }


def _case(
    ordinal: int,
    *,
    reasons: tuple[str, ...],
    early: bool,
    derivative: bool,
    geometry: bool,
    acceleration: int,
    jerk: int,
) -> dict[str, object]:
    return {
        "case_ordinal": ordinal,
        "classification": "new_control_regression",
        "r94": {
            "reasons": list(reasons),
            "early_hard_impact": early,
        },
        "comparison": {
            "derivative_amplified": derivative,
            "impacted_foot_geometry_flags": ([{"kind": "gap"}] if geometry else []),
            "joint_acceleration_v9_over_v7_basis_points": acceleration,
            "joint_jerk_v9_over_v7_basis_points": jerk,
        },
    }


if __name__ == "__main__":
    unittest.main()
