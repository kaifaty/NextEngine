from __future__ import annotations

import json
import unittest
from pathlib import Path

from next_lab.progressive_kinodynamic_formulation import (
    _validate_profile,
    validate_descriptor_inventory,
)


class ProgressiveKinodynamicFormulationTests(unittest.TestCase):
    def test_tracked_profile_freezes_progressive_formulation_only(self) -> None:
        path = (
            Path(__file__).resolve().parents[1]
            / "profiles"
            / "humanoid-progressive-kinodynamic-formulation-r108.v1.json"
        )
        profile = json.loads(path.read_bytes())
        _validate_profile(profile)
        self.assertFalse(profile["scope"]["stage_execution_in_r108"])
        self.assertEqual(
            profile["stage_contracts"]["stage_0_model_identity"][
                "solver_runs"
            ],
            0,
        )
        self.assertEqual(profile["frozen_invariants"]["partial_reset"], "REPORT_ONLY")

    def test_descriptor_inventory_requires_every_dynamics_field(self) -> None:
        descriptor = _descriptor()
        inventory = validate_descriptor_inventory(descriptor)
        self.assertEqual(inventory["status"], "BASIC_FIELDS_PRESENT")
        self.assertEqual(inventory["physics_substeps_per_motor_tick"], 4)

        descriptor["actuators"][0].pop(
            "maximum_effort_rate_micronewton_metres_per_second"
        )
        with self.assertRaisesRegex(ValueError, "dynamics inventory"):
            validate_descriptor_inventory(descriptor)


def _descriptor() -> dict[str, object]:
    bodies = [
        {
            "body_id": f"body.{slot}",
            "body_slot": slot,
            "parent_body_slot": None if slot == 0 else slot - 1,
            "mass_microkilograms": 1,
            "center_of_mass_micrometres": [0, 0, 0],
            "authoritative_inertia_tensor_microkilogram_metre_squared": [
                1,
                0,
                0,
                1,
                0,
                1,
            ],
            "solver_principal_frame": {},
            "solver_principal_inertia_microkilogram_metre_squared": [1, 1, 1],
            "colliders": [],
        }
        for slot in range(24)
    ]
    joints = [
        {
            "joint_id": f"joint.{ordinal}",
            "dof_ordinal": ordinal,
            "parent_body_slot": ordinal,
            "child_body_slot": ordinal + 1,
            "parent_frame": {},
            "child_frame": {},
            "axis_q1_30": [1, 0, 0],
            "soft_limit_microradians": [-1, 1],
            "hard_limit_microradians": [-2, 2],
            "maximum_velocity_microradians_per_second": 1,
        }
        for ordinal in range(23)
    ]
    actuators = [
        {
            "actuator_id": f"actuator.{ordinal}",
            "joint_id": f"joint.{ordinal}",
            "dof_ordinal": ordinal,
            "stiffness_q16": 1,
            "damping_q16": 1,
            "effort_micronewton_metres": [-1, 1],
            "maximum_effort_rate_micronewton_metres_per_second": 1,
            "maximum_power_microwatts": 1,
            "maximum_positive_work_microjoules_per_motor_tick": 1,
            "target_delta_microradians_per_motor_tick": [-1, 1],
            "residual_scale_microradians": 1,
        }
        for ordinal in range(23)
    ]
    return {
        "schema_version": 1,
        "body_count": 24,
        "bodies": bodies,
        "joints": joints,
        "actuators": actuators,
        "action_width": 23,
        "motor_hz": 60,
        "physics_hz": 240,
    }


if __name__ == "__main__":
    unittest.main()
