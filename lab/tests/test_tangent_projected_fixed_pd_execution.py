from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
from next_lab.fixed_pd_inverse_dynamics_execution import derive_fixed_pd_schedule
from next_lab.fixed_pd_inverse_dynamics_execution_formulation import (
    ACTUATOR_COUNT,
    COLLOCATION_COUNT,
    FRAME_COUNT,
    GENERALIZED_WIDTH,
)
from next_lab.tangent_projected_fixed_pd_execution import (
    ProjectedExecutionOutcome,
    _projected_controller_lineage_passes,
    _validate_profile,
    aggregate_projection_rows,
    derive_projected_fixed_pd_schedule,
    encode_solver_private_cache,
)

ROOT = Path(__file__).resolve().parents[2]
PROFILE = (
    ROOT / "lab/profiles/humanoid-tangent-projected-fixed-pd-execution-r130.v1.json"
)
DESCRIPTOR = ROOT / "lab/tests/fixtures/biomechanics_motor_mirror_v1.json"


class TangentProjectedFixedPdExecutionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.profile = json.loads(PROFILE.read_bytes())
        cls.descriptor = json.loads(DESCRIPTOR.read_bytes())

    def test_profile_freezes_one_r130_without_downstream_authority(self) -> None:
        _validate_profile(self.profile)
        expected = self.profile["expected_inventory"]
        self.assertEqual(expected["projection_systems"], 2640)
        self.assertEqual(
            expected["inverse_dynamics_singular_value_decompositions"], 3200
        )
        self.assertEqual(expected["force_gauge_interval_classifications"], 2316)
        self.assertEqual(
            self.profile["bounded_acceptance"][
                "additional_projected_fixed_pd_execution"
            ],
            "NOT_AUTHORIZED",
        )
        self.assertFalse(self.profile["scope"]["integration"])

    def test_zero_projected_velocity_reproduces_zero_velocity_pd_schedule(
        self,
    ) -> None:
        cache = _zero_cache()
        projected = np.zeros((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
        original = derive_fixed_pd_schedule(cache=cache, descriptor=self.descriptor)
        recomputed = derive_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=self.descriptor,
            projected_velocity=projected,
        )
        np.testing.assert_array_equal(
            recomputed.applied_target_microradians,
            original.applied_target_microradians,
        )
        np.testing.assert_array_equal(
            recomputed.applied_effort_micronewton_metres,
            original.applied_effort_micronewton_metres,
        )
        self.assertEqual(recomputed.audit["status"], "PASS")
        self.assertEqual(sum(recomputed.audit["activation_counts"].values()), 0)

    def test_projected_velocity_recomputes_damping_effort(self) -> None:
        cache = _zero_cache()
        projected = np.zeros((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
        projected[0, 6] = 0.1
        recomputed = derive_projected_fixed_pd_schedule(
            cache=cache,
            descriptor=self.descriptor,
            projected_velocity=projected,
        )
        self.assertNotEqual(recomputed.applied_effort_micronewton_metres[0, 0], 0.0)
        self.assertEqual(recomputed.audit["status"], "PASS")
        self.assertEqual(
            recomputed.audit["projected_velocity_float64_sha256"],
            _array_sha256(projected),
        )

    def test_controller_lineage_guard_is_exact(self) -> None:
        expected = self.profile["controller_lineage_contract"]
        audit = {
            "status": "PASS",
            "collocation_count": COLLOCATION_COUNT,
            "activation_counts": {
                "hard_rom_violation": expected["hard_rom_violation_count"],
                "target_soft_clamp": expected["target_soft_clamp_count"],
                "target_slew": expected["target_slew_count"],
            },
            "activated_dof_ordinals": {
                "hard_rom_violation": expected["hard_rom_violation_dof_ordinals"],
                "target_soft_clamp": expected["target_soft_clamp_dof_ordinals"],
                "target_slew": expected["target_slew_dof_ordinals"],
            },
            "maxima": {
                "hard_rom_excess_microradians": expected[
                    "maximum_hard_rom_excess_microradians"
                ],
                "target_lag_microradians": expected["maximum_target_lag_microradians"],
            },
        }
        self.assertTrue(_projected_controller_lineage_passes(self.profile, audit))
        audit["activation_counts"]["target_slew"] = 0
        self.assertFalse(_projected_controller_lineage_passes(self.profile, audit))

    def test_projected_controller_rejects_shape_and_nonfinite_input(self) -> None:
        cache = _zero_cache()
        with self.assertRaisesRegex(ValueError, "velocity differs"):
            derive_projected_fixed_pd_schedule(
                cache=cache,
                descriptor=self.descriptor,
                projected_velocity=np.zeros((1, GENERALIZED_WIDTH)),
            )
        projected = np.zeros((COLLOCATION_COUNT, GENERALIZED_WIDTH), dtype=np.float64)
        projected[0, 0] = np.nan
        with self.assertRaisesRegex(ValueError, "velocity differs"):
            derive_projected_fixed_pd_schedule(
                cache=cache,
                descriptor=self.descriptor,
                projected_velocity=projected,
            )

    def test_projection_aggregate_preserves_flight_and_flat_extrema(self) -> None:
        rows = [
            _projection_row(collocation=0, active=(), line=None, active_after=0.0),
            _projection_row(
                collocation=1,
                active=(0, 1),
                line=-2.0e-17,
                active_after=3.0e-16,
            ),
        ]
        projected = np.zeros((2, GENERALIZED_WIDTH), dtype=np.float64)
        delta = np.ones((2, GENERALIZED_WIDTH), dtype=np.float64)
        aggregate = aggregate_projection_rows(
            rows,
            projected_velocity=projected,
            delta_velocity=delta,
        )
        self.assertEqual(aggregate["flight_identity_collocations"], 1)
        self.assertEqual(aggregate["flat_foot_projection_collocations"], 1)
        self.assertEqual(
            aggregate["maximum_active_velocity_after_metres_per_second"],
            3.0e-16,
        )
        self.assertEqual(
            aggregate["maximum_absolute_projected_flat_rigid_line_compatibility"],
            2.0e-17,
        )

    def test_solver_private_cache_has_no_candidate_authority(self) -> None:
        arrays = {
            "projected_generalized_velocity": np.zeros((2, GENERALIZED_WIDTH)),
            "collocation_feasible": np.ones(2, dtype=np.uint8),
        }
        outcome = ProjectedExecutionOutcome(
            status="VALID_COMPLETE",
            feasibility="FEASIBLE",
            invalid_reason=None,
            projection_systems=1,
            projection_factorizations=1,
            projection_solves=1,
            controller_schedule_derivations=1,
            inverse_dynamics_system_assemblies=2,
            inverse_dynamics_singular_value_decompositions=2,
            inverse_dynamics_particular_solutions=2,
            gauge_interval_classifications=0,
            projection_rows=(),
            solver_rows=(),
            projection_aggregate={},
            solver_aggregate={},
            fixed_pd_schedule=None,
            arrays=arrays,
            resource_usage={},
            ordered_projection_system_sha256="0" * 64,
            ordered_reduced_system_sha256="1" * 64,
        )
        payload, identity = encode_solver_private_cache(
            outcome=outcome,
            profile_sha256="2" * 64,
            r129_report_sha256="3" * 64,
            r126_report_sha256="4" * 64,
        )
        self.assertIsNotNone(payload)
        self.assertFalse(identity["candidate_or_corpus_authority"])
        self.assertEqual(identity["status"], "EMITTED_SOLVER_PRIVATE_TRANSIENT")
        self.assertEqual(identity["metadata"]["feasibility"], "FEASIBLE")


def _zero_cache() -> dict[str, np.ndarray]:
    rotations = np.repeat(np.eye(3, dtype=np.float64)[np.newaxis], FRAME_COUNT, axis=0)
    return {
        "root_position_m": np.zeros((FRAME_COUNT, 3), dtype=np.float64),
        "root_orientation_delta_rad": np.zeros((FRAME_COUNT, 3), dtype=np.float64),
        "joint_position_rad": np.zeros((FRAME_COUNT, ACTUATOR_COUNT), dtype=np.float64),
        "velocity": np.zeros((FRAME_COUNT, GENERALIZED_WIDTH), dtype=np.float64),
        "acceleration": np.zeros((FRAME_COUNT, GENERALIZED_WIDTH), dtype=np.float64),
        "reference_root_rotation": rotations,
    }


def _projection_row(
    *,
    collocation: int,
    active: tuple[int, ...],
    line: float | None,
    active_after: float,
) -> dict[str, object]:
    fields = {
        "maximum_active_velocity_before_metres_per_second": 0.1,
        "maximum_active_velocity_after_metres_per_second": active_after,
        "maximum_generalized_velocity_correction": 0.2,
        "mass_metric_velocity_correction": 0.3,
        "scaled_kkt_residual": 1.0e-15,
        "closed_form_to_kkt_maximum_absolute": 1.0e-15,
        "projection_idempotence_error": 1.0e-15,
        "mass_orthogonality_relative_error": 1.0e-15,
        "kinetic_energy_increase_joules": -0.1,
        "gauge_velocity_effect_maximum_absolute": 1.0e-15,
    }
    flat = (
        None
        if line is None
        else {"projected_line_compatibility_metres_per_second_squared": line}
    )
    return {
        "status": "PASS",
        "collocation": collocation,
        "active_point_ordinals": list(active),
        "projection": fields,
        "flat_rigid_line_audit": flat,
    }


def _array_sha256(value: np.ndarray) -> str:
    import hashlib

    return hashlib.sha256(np.ascontiguousarray(value).tobytes()).hexdigest()


if __name__ == "__main__":
    unittest.main()
