from __future__ import annotations

import copy
import hashlib
import json
import tempfile
import unittest
from pathlib import Path

import numpy as np
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    _validate_profile,
    affine_sample,
    audit_contact_schedule,
    audit_fixed_pd_schedule,
    audit_future_execution_budget,
    audit_system_inventory,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT
    / "lab/profiles/humanoid-fixed-pd-inverse-dynamics-execution-formulation-r121.v1.json"
)


class FixedPdInverseDynamicsExecutionFormulationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.profile = json.loads(PROFILE.read_bytes())

    def test_profile_authorizes_only_report_only_r122_conformance(self) -> None:
        _validate_profile(self.profile)
        self.assertEqual(self.profile["scope"]["inverse_dynamics_solves"], 0)
        self.assertEqual(
            self.profile["decision"]["complete"],
            "PERMIT_R122_FIXED_PD_INVERSE_DYNAMICS_IMPLEMENTATION_CONFORMANCE_ONLY",
        )
        self.assertEqual(
            self.profile["bounded_acceptance"]["inverse_dynamics_solve"],
            "NOT_AUTHORIZED",
        )
        self.assertFalse(
            self.profile["implementation_conformance_contract"]["execution_in_r122"]
        )

    def test_system_is_3200_independent_square_64_by_64_blocks(self) -> None:
        inventory = audit_system_inventory(
            active_point_collocations=4956,
            inactive_point_collocations=7844,
        )
        self.assertEqual(inventory["physics_collocation_count"], 3200)
        self.assertEqual(inventory["local_unknown_count"], 64)
        self.assertEqual(inventory["total_decision_scalar_count"], 204800)
        self.assertEqual(inventory["total_equality_row_count"], 204800)
        self.assertEqual(inventory["friction_second_order_cone_count"], 4956)

    def test_affine_lift_samples_only_preintegration_abscissae(self) -> None:
        left = np.asarray([0.0, 4.0], dtype=np.float64)
        right = np.asarray([4.0, 0.0], dtype=np.float64)
        np.testing.assert_array_equal(affine_sample(left, right, 0), left)
        np.testing.assert_array_equal(
            affine_sample(left, right, 2), np.asarray([2.0, 2.0])
        )
        np.testing.assert_array_equal(
            affine_sample(left, right, 3), np.asarray([3.0, 1.0])
        )
        with self.assertRaisesRegex(ValueError, "affine sample"):
            affine_sample(left, right, 4)

    def test_contact_schedule_repeats_modes_four_times_without_mutation(self) -> None:
        modes = np.zeros((801, 2), dtype=np.uint8)
        r120 = {
            "solver_result": {
                "accepted_exact_result": {
                    "emitted_hashes": {
                        "arrays": {
                            "contact_modes": {
                                "sha256": hashlib.sha256(modes.tobytes()).hexdigest(),
                                "shape": [801, 2],
                                "dtype": "uint8",
                            }
                        }
                    }
                }
            }
        }
        points = {
            "ordered_points": [
                {"point_ordinal": 0, "side_index": 0, "active_mode_bits": [1, 3]},
                {"point_ordinal": 1, "side_index": 0, "active_mode_bits": [2, 3]},
                {"point_ordinal": 2, "side_index": 1, "active_mode_bits": [1, 3]},
                {"point_ordinal": 3, "side_index": 1, "active_mode_bits": [2, 3]},
            ]
        }
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "clip.npz"
            np.savez(path, contact_modes=modes)
            report = audit_contact_schedule(
                path,
                expected_sha256=_sha256(path),
                point_identity=points,
                r120=r120,
            )
        self.assertEqual(report["motor_active_point_count"], 0)
        self.assertEqual(report["active_point_collocation_count"], 0)
        self.assertEqual(report["inactive_point_collocation_count"], 12800)
        self.assertTrue(report["last_frame_is_endpoint_only"])

    def test_zero_state_fixed_pd_schedule_has_no_hidden_effort(self) -> None:
        cache = {
            "joint_position_rad": np.zeros((801, 23), dtype=np.float64),
            "velocity": np.zeros((801, 29), dtype=np.float64),
        }
        report = audit_fixed_pd_schedule(
            cache=cache, descriptor=_synthetic_descriptor()
        )
        self.assertEqual(report["status"], "PASS")
        self.assertTrue(
            all(value == 0 for value in report["activation_counts"].values())
        )
        self.assertEqual(
            report["maxima"]["absolute_applied_effort_micronewton_metres"], 0.0
        )

    def test_future_budget_is_one_bounded_nonrestartable_process(self) -> None:
        audit = audit_future_execution_budget(
            self.profile["future_single_execution_budget"]
        )
        self.assertEqual(audit["status"], "PASS")
        self.assertTrue(audit["one_process"])
        self.assertEqual(audit["maximum_local_system_solves"], 3200)
        self.assertEqual(audit["maximum_local_unknown_count"], 64)
        self.assertEqual(audit["restart_count"], 0)

    def test_profile_rejects_stage_2_integration_or_direct_solve(self) -> None:
        invalid = copy.deepcopy(self.profile)
        invalid["state_lift_contract"]["discrete_integration_constraint"] = (
            "ENFORCED_IN_STAGE_2"
        )
        with self.assertRaisesRegex(ValueError, "profile differs"):
            _validate_profile(invalid)
        invalid = copy.deepcopy(self.profile)
        invalid["bounded_acceptance"]["inverse_dynamics_solve"] = "AUTHORIZED"
        with self.assertRaisesRegex(ValueError, "profile differs"):
            _validate_profile(invalid)


def _synthetic_descriptor() -> dict[str, object]:
    joints = []
    actuators = []
    for ordinal in range(23):
        joint_id = f"joint.{ordinal}"
        joints.append(
            {
                "joint_id": joint_id,
                "dof_ordinal": ordinal,
                "hard_limit_microradians": [-1_000_000, 1_000_000],
                "soft_limit_microradians": [-900_000, 900_000],
                "maximum_velocity_microradians_per_second": 10_000_000,
            }
        )
        actuators.append(
            {
                "actuator_id": f"actuator.{ordinal}",
                "joint_id": joint_id,
                "dof_ordinal": ordinal,
                "stiffness_q16": 80 * 65_536,
                "damping_q16": 4 * 65_536,
                "effort_micronewton_metres": [-150_000_000, 150_000_000],
                "maximum_effort_rate_micronewton_metres_per_second": 6_000_000_000,
                "maximum_power_microwatts": 800_000_000,
                "maximum_positive_work_microjoules_per_motor_tick": 13_333_333,
                "target_delta_microradians_per_motor_tick": [-80_000, 80_000],
            }
        )
    return {"joints": joints, "actuators": actuators}


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    unittest.main()
