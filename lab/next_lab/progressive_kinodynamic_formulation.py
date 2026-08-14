from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Mapping

from next_lab.contact_projected_direction_audit import (
    AUDIT_ID as R107_AUDIT_ID,
    CHECK_ID as R107_CHECK_ID,
    _validate_profile as _validate_r107_profile,
)
from next_lab.contact_target_knot_formulation import canonical_json, sha256


FORMULATION_ID = "nextengine.humanoid-progressive-kinodynamic-formulation.v1"
CHECK_ID = "TRAIN-4-PROGRESSIVE-KINODYNAMIC-FORMULATION"


def build_progressive_kinodynamic_formulation(
    *,
    profile_path: Path,
    r107_report_path: Path,
    r107_profile_path: Path,
    v9_profile_path: Path,
    descriptor_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R108's progressive formulation without running any stage."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r107_report_path,
            r107_profile_path,
            v9_profile_path,
            descriptor_path,
            tool_path,
        )
    )
    (
        profile_path,
        r107_report_path,
        r107_profile_path,
        v9_profile_path,
        descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("progressive-kinodynamic formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    r107 = validate_r107(
        report_path=r107_report_path,
        profile_path=r107_profile_path,
        expected=profile["source"]["r107"],
    )
    if (
        sha256(v9_profile_path)
        != profile["source"]["v9_profile_sha256"]
        or sha256(descriptor_path)
        != profile["source"]["descriptor_sha256"]
        or r107["identities"]["v9_profile_sha256"]
        != sha256(v9_profile_path)
        or r107["identities"]["descriptor_sha256"]
        != sha256(descriptor_path)
    ):
        raise ValueError("progressive-kinodynamic source identity differs")

    descriptor = json.loads(descriptor_path.read_bytes())
    descriptor_inventory = validate_descriptor_inventory(descriptor)
    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "formulation_id": FORMULATION_ID,
        "scope": profile["scope"],
        "research_basis": profile["research_basis"],
        "frozen_invariants": profile["frozen_invariants"],
        "objective_priority": profile["objective_priority"],
        "stage_contracts": profile["stage_contracts"],
        "stage_transitions": profile["stage_transitions"],
        "descriptor_inventory": descriptor_inventory,
        "source_failure": {
            "r107_gate_decision": r107["gate_decision"],
            "exact_status": r107["exact_offline_result"]["status"],
            "failure_reasons": r107["exact_offline_result"][
                "failure_reasons"
            ],
            "maximum_joint_velocity_basis_points": r107[
                "exact_offline_result"
            ]["metrics"]["maximum_joint_velocity_basis_points"],
            "maximum_allowed_joint_velocity_basis_points": r107["limits"][
                "maximum_joint_velocity_basis_points"
            ],
            "continuous_projection_maximum_normalized_violation": r107[
                "projection_reconstruction"
            ]["maximum_projected_normalized_constraint_violation"],
            "quantized_projection_maximum_normalized_violation": r107[
                "projection_reconstruction"
            ]["maximum_quantized_normalized_constraint_violation"],
            "candidate_artifact": r107["quantized_candidate"][
                "candidate_artifact"
            ],
        },
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r107_report_sha256": r107["report_sha256"],
            "r107_report_file_sha256": sha256(r107_report_path),
            "r107_profile_sha256": sha256(r107_profile_path),
            "r107_audit_module_sha256": r107["identities"][
                "audit_module_sha256"
            ],
            "r107_tool_sha256": r107["identities"]["tool_sha256"],
            "v9_profile_sha256": sha256(v9_profile_path),
            "descriptor_sha256": sha256(descriptor_path),
            "tool_sha256": sha256(tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
        },
        "bounded_acceptance": {
            "r109_dynamics_model_identity_preflight": "AUTHORIZED",
            "quantization_aware_kto_solve": "NOT_AUTHORIZED",
            "inverse_dynamics_solve": "NOT_AUTHORIZED",
            "kinodynamic_solve": "NOT_AUTHORIZED",
            "candidate_target_construction": "NOT_AUTHORIZED",
            "candidate_artifact": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
        "model_identity_preflights": 0,
        "kto_solves": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "candidate_target_constructions": 0,
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": 0,
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r107(
    *,
    report_path: Path,
    profile_path: Path,
    expected: Mapping[str, Any],
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R107 report file identity differs")
    if sha256(profile_path) != expected.get("profile_sha256"):
        raise ValueError("R107 profile identity differs")
    audit_profile = json.loads(profile_path.read_bytes())
    _validate_r107_profile(audit_profile)
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    exact = report.get("exact_offline_result", {})
    metrics = exact.get("metrics", {})
    projected = report.get("projection_reconstruction", {})
    candidate = report.get("quantized_candidate", {})
    bounded = report.get("bounded_acceptance", {})
    if (
        report.get("check") != R107_CHECK_ID
        or report.get("audit_id") != R107_AUDIT_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "SELECT_PROGRESSIVE_KINODYNAMIC_FORMULATION"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("identities", {}).get("profile_sha256")
        != expected.get("profile_sha256")
        or report.get("repository", {}).get("dirty") is not False
        or exact.get("status") != "FAIL"
        or exact.get("failure_reasons") != ["joint_velocity"]
        or metrics.get("active_contact_status") != "PASS"
        or metrics.get("maximum_joint_velocity_basis_points") != 2501
        or report.get("limits", {}).get("maximum_joint_velocity_basis_points")
        != 2500
        or metrics.get("minimum_collider_height_micrometres") != 49
        or metrics.get("maximum_soft_rom_violation_microradians") != 0
        or projected.get("maximum_projected_normalized_constraint_violation")
        != 3.81639164715e-17
        or projected.get("maximum_quantized_normalized_constraint_violation")
        != 0.000411892173461
        or candidate.get("candidate_artifact") != "NOT_EMITTED"
        or candidate.get("position_scope", {}).get("status") != "PASS"
        or bounded.get("progressive_kinodynamic_formulation") != "SELECTED"
        or bounded.get("r108_bounded_fresh_discriminator_formulation")
        != "NOT_AUTHORIZED"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "candidate_artifact",
                "candidate_search",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or report.get("projection_qp_solves") != 1
        or report.get("candidate_target_constructions") != 1
        or report.get("offline_candidate_evaluations") != 1
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "candidate_artifacts_built",
                "physx_runs",
                "candidate_evaluations",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R107 exact projected-direction contract differs")
    return report


def validate_descriptor_inventory(descriptor: Mapping[str, Any]) -> dict[str, Any]:
    bodies = descriptor.get("bodies", ())
    joints = descriptor.get("joints", ())
    actuators = descriptor.get("actuators", ())
    body_fields = {
        "body_id",
        "body_slot",
        "parent_body_slot",
        "mass_microkilograms",
        "center_of_mass_micrometres",
        "authoritative_inertia_tensor_microkilogram_metre_squared",
        "solver_principal_frame",
        "solver_principal_inertia_microkilogram_metre_squared",
        "colliders",
    }
    joint_fields = {
        "joint_id",
        "dof_ordinal",
        "parent_body_slot",
        "child_body_slot",
        "parent_frame",
        "child_frame",
        "axis_q1_30",
        "soft_limit_microradians",
        "hard_limit_microradians",
        "maximum_velocity_microradians_per_second",
    }
    actuator_fields = {
        "actuator_id",
        "joint_id",
        "dof_ordinal",
        "stiffness_q16",
        "damping_q16",
        "effort_micronewton_metres",
        "maximum_effort_rate_micronewton_metres_per_second",
        "maximum_power_microwatts",
        "maximum_positive_work_microjoules_per_motor_tick",
        "target_delta_microradians_per_motor_tick",
        "residual_scale_microradians",
    }
    if (
        descriptor.get("schema_version") != 1
        or descriptor.get("body_count") != 24
        or len(bodies) != 24
        or len(joints) != 23
        or len(actuators) != 23
        or descriptor.get("action_width") != 23
        or descriptor.get("motor_hz") != 60
        or descriptor.get("physics_hz") != 240
        or any(not body_fields.issubset(body) for body in bodies)
        or any(not joint_fields.issubset(joint) for joint in joints)
        or any(not actuator_fields.issubset(actuator) for actuator in actuators)
        or sorted(int(joint["dof_ordinal"]) for joint in joints)
        != list(range(23))
        or sorted(int(actuator["dof_ordinal"]) for actuator in actuators)
        != list(range(23))
    ):
        raise ValueError("descriptor dynamics inventory differs")
    collider_count = sum(len(body["colliders"]) for body in bodies)
    return {
        "status": "BASIC_FIELDS_PRESENT",
        "acceptance_authority": False,
        "body_count": len(bodies),
        "joint_count": len(joints),
        "actuator_count": len(actuators),
        "collider_count": collider_count,
        "motor_hz": int(descriptor["motor_hz"]),
        "physics_hz": int(descriptor["physics_hz"]),
        "physics_substeps_per_motor_tick": int(
            descriptor["physics_hz"] // descriptor["motor_hz"]
        ),
        "next_preflight_requirement": (
            "bind descriptor fields to the exact native/derived-USD gravity, "
            "material, joint-frame, drive-clipping and effort-slew semantics"
        ),
    }


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    invariants = profile.get("frozen_invariants", {})
    stages = profile.get("stage_contracts", {})
    transitions = profile.get("stage_transitions", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim")
        != "ProgressiveKinodynamicFormulationOnly"
        or scope.get("initial_clip_id") != "cmu16-walk-nominal-b"
        or scope.get("initial_source_case_ordinal") != 7967
        or scope.get("body_count") != 24
        or scope.get("joint_count") != 23
        or scope.get("actuator_count") != 23
        or scope.get("motor_hz") != 60
        or scope.get("physics_hz") != 240
        or scope.get("stage_execution_in_r108") is not False
        or invariants.get("controller")
        != "fixed zero-residual PD with zero desired velocity and descriptor gains/caps"
        or invariants.get("fresh_scene_authority") != "ADR-070"
        or invariants.get("partial_reset") != "REPORT_ONLY"
        or invariants.get("contact_schedule_changes") != "FORBIDDEN"
        or invariants.get("limit_changes") != "FORBIDDEN"
        or invariants.get("integer_emission")
        != "ties-to-even; exact emitted arrays are the offline gate"
        or tuple(stages) != (
            "stage_0_model_identity",
            "stage_1_quantization_aware_kto",
            "stage_2_fixed_pd_inverse_dynamics",
            "stage_3_full_kinodynamics",
        )
        or stages.get("stage_0_model_identity", {}).get("next_run_id")
        != "R109"
        or stages.get("stage_0_model_identity", {}).get("solver_runs") != 0
        or stages.get("stage_1_quantization_aware_kto", {}).get(
            "decision_variables"
        )
        != "floating-base q/v/a plus all 23 joint q/v/a at 60 Hz"
        or stages.get("stage_2_fixed_pd_inverse_dynamics", {}).get(
            "decision_variables"
        )
        != "240 Hz generalized acceleration, 23 actuator efforts and scheduled contact wrenches"
        or stages.get("stage_3_full_kinodynamics", {}).get(
            "decision_variables"
        )
        != "240 Hz floating-base state, fixed-PD effort state and scheduled contact wrenches with 60 Hz reference targets"
        or transitions.get("r108_complete")
        != "PERMIT_R109_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        or transitions.get("model_identity_fail") != "STOP_INVALID_MODEL_LINEAGE"
        or transitions.get("kto_exact_fail") != "STOP_AND_RESEARCH"
        or transitions.get("kinodynamic_exact_fail") != "STOP_AND_RESEARCH"
        or profile.get("decision", {})
        != {
            "complete": "PERMIT_R109_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        }
    ):
        raise ValueError("progressive-kinodynamic formulation profile is invalid")
