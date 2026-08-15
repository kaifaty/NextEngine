from __future__ import annotations

import json
import unittest
from copy import deepcopy
from pathlib import Path

import numpy as np
from next_lab.exit_mode_owned_projected_schedule_execution import (
    ACTUATOR_COUNT,
    COLLOCATION_COUNT,
    CONTROLLER_CATEGORIES,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
    _record_controller_events,
    _validate_profile,
    audit_controller_events,
    derive_eventful_projected_fixed_pd_schedule,
)
from next_lab.tangent_projected_fixed_pd_execution import (
    derive_projected_fixed_pd_schedule,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/humanoid-exit-mode-owned-projected-schedule-r133.v1.json"
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class ExitModeOwnedProjectedScheduleExecutionTests(unittest.TestCase):
    def test_profile_authorizes_only_r134_formulation_on_pass(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        _validate_profile(profile)
        self.assertEqual(profile["scope"]["collocation_count"], 3200)
        self.assertEqual(profile["scope"]["maximum_projection_systems"], 2640)
        self.assertEqual(
            profile["bounded_acceptance"][
                "r134_projected_inverse_dynamics_formulation"
            ],
            "AUTHORIZED_ON_R133_PASS_ONLY",
        )
        self.assertEqual(
            profile["bounded_acceptance"]["inverse_dynamics_execution"],
            "NOT_AUTHORIZED",
        )

    def test_profile_rejects_missing_event_address_closure(self) -> None:
        profile = json.loads(PROFILE.read_bytes())
        changed = deepcopy(profile)
        changed["acceptance_contract"]["required_controller_event_address_closure"] = (
            False
        )
        with self.assertRaisesRegex(ValueError, "execution profile differs"):
            _validate_profile(changed)

    def test_controller_event_records_exact_row_and_dof(self) -> None:
        mask = np.zeros(ACTUATOR_COUNT, dtype=np.bool_)
        mask[[2, 11]] = True
        counts = {name: 0 for name in CONTROLLER_CATEGORIES}
        channels = {name: set() for name in CONTROLLER_CATEGORIES}
        events: list[dict[str, object]] = []
        values = np.arange(ACTUATOR_COUNT, dtype=np.float64)
        _record_controller_events(
            mask=mask,
            category="velocity_violation",
            counts=counts,
            channels=channels,
            events=events,
            interval=76,
            substep=3,
            stage="synthetic_velocity",
            details={"value": values},
        )
        self.assertEqual(counts["velocity_violation"], 2)
        self.assertEqual(channels["velocity_violation"], {2, 11})
        self.assertEqual([row["collocation"] for row in events], [307, 307])
        self.assertEqual([row["dof_ordinal"] for row in events], [2, 11])
        self.assertEqual([row["value"] for row in events], [2.0, 11.0])
        self.assertEqual(
            audit_controller_events(events=events, counts=counts)["status"],
            "PASS",
        )

    def test_event_audit_rejects_unaddressed_event(self) -> None:
        counts = {name: 0 for name in CONTROLLER_CATEGORIES}
        counts["target_slew"] = 1
        event = {
            "category": "target_slew",
            "stage": "synthetic",
            "collocation": 4,
            "interval": 1,
            "substep": 0,
            "dof_ordinal": 23,
        }
        audit = audit_controller_events(events=[event], counts=counts)
        self.assertEqual(audit["status"], "FAIL")
        self.assertFalse(audit["all_events_row_addressed"])

    def test_eventful_schedule_is_byte_exact_r130_semantics(self) -> None:
        descriptor = json.loads(DESCRIPTOR.read_bytes())
        cache = {
            "joint_position_rad": np.zeros(
                (FRAME_COUNT, ACTUATOR_COUNT), dtype=np.float64
            )
        }
        projected = np.zeros((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
        projected[0, 6 + 11] = 20.0
        expected = derive_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=descriptor,
            projected_velocity=projected,
        )
        actual = derive_eventful_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=descriptor,
            projected_velocity=projected,
        )
        np.testing.assert_array_equal(
            actual.schedule.applied_target_microradians,
            expected.applied_target_microradians,
        )
        np.testing.assert_array_equal(
            actual.schedule.applied_effort_micronewton_metres,
            expected.applied_effort_micronewton_metres,
        )
        self.assertEqual(actual.schedule.audit, expected.audit)
        self.assertEqual(actual.event_audit["status"], "PASS")
        self.assertEqual(
            actual.event_audit["category_counts"],
            expected.audit["activation_counts"],
        )


if __name__ == "__main__":
    unittest.main()
