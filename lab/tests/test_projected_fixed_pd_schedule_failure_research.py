from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

from next_lab.projected_fixed_pd_schedule_failure_research import (
    _descriptor_channel,
    _validate_profile,
    analyze_projection_hotspots,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-post-r130-projected-schedule-research.v1.json"


def _collocation(
    ordinal: int,
    *,
    correction: float,
    contact_modes: list[int],
    active_points: list[int],
) -> dict[str, object]:
    return {
        "collocation": ordinal,
        "interval": ordinal // 4,
        "substep": ordinal % 4,
        "contact_modes": contact_modes,
        "active_point_ordinals": active_points,
        "projection": {
            "maximum_generalized_velocity_correction": correction,
            "maximum_active_velocity_before_metres_per_second": correction / 10.0,
        },
    }


class ProjectedFixedPdScheduleFailureResearchTests(unittest.TestCase):
    def test_profile_is_report_only_and_blocks_every_execution(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertTrue(
            all(
                value == "NOT_AUTHORIZED"
                for value in profile["bounded_acceptance"].values()
            )
        )
        scope = profile["scope"]
        self.assertEqual(scope["research_audits"], 1)
        self.assertEqual(scope["state_projection_systems"], 0)
        self.assertEqual(scope["inverse_dynamics_system_assemblies"], 0)
        self.assertEqual(scope["kinodynamic_solves"], 0)
        self.assertFalse(scope["candidate_construction"])
        self.assertFalse(scope["training"])

    def test_profile_rejects_a_missing_execution_prohibition(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        incomplete = deepcopy(profile)
        del incomplete["bounded_acceptance"]["training"]
        with self.assertRaisesRegex(ValueError, "research profile differs"):
            _validate_profile(incomplete)

    def test_hotspot_audit_ranks_rows_and_detects_contact_exit(self) -> None:
        rows = [
            _collocation(
                ordinal,
                correction=0.1,
                contact_modes=[0, 2] if ordinal < 4 else [0, 0],
                active_points=[3] if ordinal < 4 else [],
            )
            for ordinal in range(12)
        ]
        rows[6]["projection"]["maximum_generalized_velocity_correction"] = 9.0
        rows[7]["projection"]["maximum_generalized_velocity_correction"] = 10.0
        audit = analyze_projection_hotspots(rows, hotspot_count=2)
        self.assertEqual(
            [row["collocation"] for row in audit["hotspots"]],
            [7, 6],
        )
        self.assertFalse(audit["all_hotspots_are_contact_exit_rows"])

        rows[4]["projection"] = {
            "maximum_generalized_velocity_correction": 0.1,
            "maximum_active_velocity_before_metres_per_second": 0.01,
        }
        rows[2]["projection"]["maximum_generalized_velocity_correction"] = 20.0
        rows[3]["projection"]["maximum_generalized_velocity_correction"] = 30.0
        audit = analyze_projection_hotspots(rows, hotspot_count=2)
        self.assertEqual(
            [row["collocation"] for row in audit["hotspots"]],
            [3, 2],
        )
        self.assertTrue(audit["all_hotspots_are_contact_exit_rows"])
        self.assertEqual(audit["distinct_hotspot_intervals"], [0])
        self.assertEqual(audit["hotspots"][0]["next_interval_contact_modes"], [0, 0])

    def test_hotspot_audit_rejects_noncanonical_order(self) -> None:
        rows = [
            _collocation(
                ordinal,
                correction=float(ordinal),
                contact_modes=[0, 2],
                active_points=[3],
            )
            for ordinal in range(8)
        ]
        rows[3]["collocation"] = 4
        with self.assertRaisesRegex(ValueError, "collocation order differs"):
            analyze_projection_hotspots(rows, hotspot_count=1)

    def test_descriptor_channel_joins_joint_and_actuator_by_ordinal(self) -> None:
        descriptor = {
            "joints": [
                {
                    "dof_ordinal": 11,
                    "joint_id": "joint.right-ankle-roll",
                    "anatomical_semantic_id": "anatomical-joint.ankle-roll",
                    "maximum_velocity_microradians_per_second": 8_000_000,
                    "hard_limit_microradians": [-349_066, 349_066],
                }
            ],
            "actuators": [
                {
                    "dof_ordinal": 11,
                    "joint_id": "joint.right-ankle-roll",
                    "actuator_id": "actuator.right-ankle-roll",
                    "stiffness_q16": 19_660_800,
                    "damping_q16": 1_966_080,
                    "effort_micronewton_metres": [-140_000_000, 140_000_000],
                    "maximum_effort_rate_micronewton_metres_per_second": 1_400_000_000,
                    "maximum_power_microwatts": 500_000_000,
                    "maximum_positive_work_microjoules_per_motor_tick": 8_333_333,
                }
            ],
        }
        channel = _descriptor_channel(descriptor, 11)
        self.assertEqual(channel["joint"]["joint_id"], "joint.right-ankle-roll")
        self.assertEqual(channel["actuator"]["damping_q16"], 1_966_080)


if __name__ == "__main__":
    unittest.main()
