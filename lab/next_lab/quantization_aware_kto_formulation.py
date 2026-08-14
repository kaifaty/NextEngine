from __future__ import annotations

import hashlib
import json
import sys
from collections import Counter
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
import osqp
import scipy
from numpy.typing import NDArray

from next_lab.clean_dynamics_model_identity_preflight import (
    CHECK_ID as R113_CHECK_ID,
)
from next_lab.clean_dynamics_model_identity_preflight import (
    PREFLIGHT_ID as R113_PREFLIGHT_ID,
)
from next_lab.clean_dynamics_model_identity_preflight import (
    _validate_profile as _validate_r113_profile,
)
from next_lab.contact_manifold import contact_point_mask
from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.contact_trajectory import (
    VELOCITY_SEMANTICS,
    hybrid_velocity_stencil,
)
from next_lab.motor_mirror import (
    validate_biomechanics_descriptor,
    validate_current_biomechanics_descriptor,
)
from next_lab.progressive_kinodynamic_formulation import (
    CHECK_ID as R108_CHECK_ID,
)
from next_lab.progressive_kinodynamic_formulation import (
    FORMULATION_ID as R108_FORMULATION_ID,
)
from next_lab.progressive_kinodynamic_formulation import (
    _validate_profile as _validate_r108_profile,
)

FORMULATION_ID = "nextengine.humanoid-quantization-aware-kto-execution-formulation.v1"
CHECK_ID = "TRAIN-4-QUANTIZATION-AWARE-KTO-EXECUTION-FORMULATION"

TRACKED_SOURCE_PATHS = {
    "contact_trajectory": "lab/next_lab/contact_trajectory.py",
    "contact_manifold": "lab/next_lab/contact_manifold.py",
    "motion_math": "lab/next_lab/motion_math.py",
    "motor_mirror": "lab/next_lab/motor_mirror.py",
}

EXPECTED_ARRAYS = {
    "center_of_mass_um": ((801, 3), "int64"),
    "contact_modes": ((801, 2), "uint8"),
    "contacts": ((801, 7), "uint8"),
    "effector_position_um": ((801, 6, 3), "int64"),
    "joint_position_urad": ((801, 23), "int64"),
    "joint_velocity_urad_s": ((801, 23), "int64"),
    "phase_u16": ((801,), "uint16"),
    "reference_frame": ((801,), "int64"),
    "root_linear_velocity_um_s": ((801, 3), "int64"),
    "root_position_um": ((801, 3), "int64"),
    "root_quaternion_q1_30": ((801, 4), "int64"),
    "root_yaw_urad": ((801,), "int64"),
    "root_yaw_velocity_urad_s": ((801,), "int64"),
}

KINEMATIC_DESCRIPTOR_FIELDS = (
    "body_schema_hash",
    "ordered_body_ids",
    "ordered_actuator_ids",
    "body_count",
    "action_width",
    "motor_hz",
    "physics_hz",
    "bodies",
    "joints",
    "actuators",
    "effectors",
    "collision_exclusions",
)


def tracked_source_paths(repository_root: Path) -> dict[str, Path]:
    root = repository_root.resolve()
    return {name: root / relative for name, relative in TRACKED_SOURCE_PATHS.items()}


def build_quantization_aware_kto_formulation(
    *,
    profile_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    r108_report_path: Path,
    r108_profile_path: Path,
    descriptor_bytes: bytes,
    legacy_descriptor_path: Path,
    v9_profile_path: Path,
    v9_manifest_path: Path,
    v9_complete_clip_path: Path,
    v7_profile_path: Path,
    v7_manifest_path: Path,
    v7_case_path: Path,
    tracked_sources: Mapping[str, Path],
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R114's one-run KTO contract without executing a solver."""

    direct_paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r113_report_path,
            r113_profile_path,
            r108_report_path,
            r108_profile_path,
            legacy_descriptor_path,
            v9_profile_path,
            v9_manifest_path,
            v9_complete_clip_path,
            v7_profile_path,
            v7_manifest_path,
            v7_case_path,
            tool_path,
        )
    )
    (
        profile_path,
        r113_report_path,
        r113_profile_path,
        r108_report_path,
        r108_profile_path,
        legacy_descriptor_path,
        v9_profile_path,
        v9_manifest_path,
        v9_complete_clip_path,
        v7_profile_path,
        v7_manifest_path,
        v7_case_path,
        tool_path,
    ) = direct_paths
    sources = {name: path.resolve() for name, path in tracked_sources.items()}
    if any(not path.is_file() for path in (*direct_paths, *sources.values())):
        raise FileNotFoundError("R114 KTO formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r113 = validate_r113(
        profile=profile,
        report_path=r113_report_path,
        profile_path=r113_profile_path,
    )
    r108 = validate_r108(
        profile=profile,
        report_path=r108_report_path,
        profile_path=r108_profile_path,
    )
    source_identity = audit_source_identity(profile, sources)
    descriptor_lineage = audit_descriptor_lineage(
        profile=profile,
        descriptor_bytes=descriptor_bytes,
        legacy_descriptor_path=legacy_descriptor_path,
        v9_profile_path=v9_profile_path,
    )
    trajectory_lineage = audit_trajectory_lineage(
        profile=profile,
        v9_profile_path=v9_profile_path,
        v9_manifest_path=v9_manifest_path,
        v9_complete_clip_path=v9_complete_clip_path,
        v7_profile_path=v7_profile_path,
        v7_manifest_path=v7_manifest_path,
        v7_case_path=v7_case_path,
    )
    variable_inventory = build_variable_inventory(
        profile=profile,
        descriptor=descriptor_lineage["current_descriptor"],
        v9_profile=trajectory_lineage["v9_profile"],
    )
    validations = _validate_results(profile, validation_results)

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "formulation_id": FORMULATION_ID,
        "scope": profile["scope"],
        "frozen_invariants": profile["frozen_invariants"],
        "source_gates": {
            "r113_status": r113["status"],
            "r113_gate_decision": r113["gate_decision"],
            "r113_report_sha256": r113["report_sha256"],
            "r108_status": r108["status"],
            "r108_gate_decision": r108["gate_decision"],
            "r108_report_sha256": r108["report_sha256"],
        },
        "source_roles": profile["source_roles"],
        "descriptor_lineage": {
            key: value
            for key, value in descriptor_lineage.items()
            if key != "current_descriptor"
        },
        "trajectory_lineage": {
            key: value
            for key, value in trajectory_lineage.items()
            if key != "v9_profile"
        },
        "time_grid": profile["time_grid"],
        "variable_inventory": variable_inventory,
        "kinematic_equalities": profile["kinematic_equalities"],
        "hard_constraints": profile["hard_constraints"],
        "objective": profile["objective"],
        "quantization_and_emission": profile["quantization_and_emission"],
        "tracking_progress_gate": profile["tracking_progress_gate"],
        "exact_acceptance": profile["exact_acceptance"],
        "execution_budget": profile["execution_budget"],
        "failure_disposition": profile["failure_disposition"],
        "result_transitions": profile["result_transitions"],
        "validation_results": validations,
        "next_smallest_action": profile["decision"]["next_smallest_action"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "r108_report_file_sha256": sha256(r108_report_path),
            "r108_profile_sha256": sha256(r108_profile_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "legacy_descriptor_sha256": sha256(legacy_descriptor_path),
            "v9_profile_sha256": sha256(v9_profile_path),
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "v7_profile_sha256": sha256(v7_profile_path),
            "v7_manifest_file_sha256": sha256(v7_manifest_path),
            "v7_case_sha256": sha256(v7_case_path),
            "tracked_source_sha256": source_identity,
            "tool_sha256": sha256(tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "kto_execution_formulations": 1,
        "model_identity_preflights": 0,
        "solver_runs": 0,
        "kto_solves": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "candidate_target_constructions": 0,
        "candidate_artifacts_built": 0,
        "solver_private_warm_start_caches": 0,
        "offline_candidate_evaluations": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r113(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    profile_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r113"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R114 R113 source identity differs")
    _validate_r113_profile(json.loads(profile_path.read_bytes()))
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R113")
    bounded = report.get("bounded_acceptance", {})
    result = report.get("model_identity_result", {})
    if (
        report.get("check") != R113_CHECK_ID
        or report.get("preflight_id") != R113_PREFLIGHT_ID
        or report.get("status") != "PASS"
        or report.get("gate_decision")
        != "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY"
        or result.get("status") != "PASS"
        or result.get("blocking_reasons") != []
        or result.get("single_backend_neutral_dynamics_model_available") is not True
        or bounded.get("r114_quantization_aware_kto_formulation")
        != "AUTHORIZED_FORMULATION_ONLY"
        or bounded.get("quantization_aware_kto_solve") != "NOT_AUTHORIZED"
        or report.get("model_identity_preflights") != 1
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "solver_runs",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_target_constructions",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R114 R113 gate contract differs")
    return report


def validate_r108(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    profile_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r108"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R114 R108 source identity differs")
    source_profile = json.loads(profile_path.read_bytes())
    _validate_r108_profile(source_profile)
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R108")
    stages = report.get("stage_contracts", {})
    if (
        report.get("check") != R108_CHECK_ID
        or report.get("formulation_id") != R108_FORMULATION_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R109_DYNAMICS_MODEL_IDENTITY_PREFLIGHT_ONLY"
        or stages.get("stage_1_quantization_aware_kto", {}).get("decision_variables")
        != "floating-base q/v/a plus all 23 joint q/v/a at 60 Hz"
        or report.get("stage_transitions", {}).get("model_identity_pass")
        != "PERMIT_SEPARATE_BOUNDED_KTO_EXECUTION_FORMULATION_ONLY"
        or report.get("repository", {}).get("dirty") is not False
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "model_identity_preflights",
                "kto_solves",
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "candidate_target_constructions",
                "candidate_artifacts_built",
                "physx_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R114 R108 formulation contract differs")
    return report


def audit_source_identity(
    profile: Mapping[str, Any], tracked_sources: Mapping[str, Path]
) -> dict[str, str]:
    if set(tracked_sources) != set(TRACKED_SOURCE_PATHS):
        raise ValueError("R114 tracked source set differs")
    actual = {name: sha256(path) for name, path in tracked_sources.items()}
    if actual != profile["source"]["tracked_source_sha256"]:
        raise ValueError("R114 tracked source identity differs")
    trajectory = tracked_sources["contact_trajectory"].read_text(encoding="utf-8")
    motion_math = tracked_sources["motion_math"].read_text(encoding="utf-8")
    required_trajectory_tokens = (
        "def hybrid_velocity_stencil(",
        "np.rint(root_values * 1_000_000.0)",
        "np.rint(joint_values * 1_000_000.0)",
        "maximum_joint_velocity_basis_points",
        "minimum_collider_height_micrometres",
    )
    required_math_tokens = (
        "def matrix_to_quaternion(",
        "def quaternion_to_matrix(",
        "def q1_30(",
        "np.rint(np.clip(values, -1.0, 1.0)",
    )
    if any(token not in trajectory for token in required_trajectory_tokens) or any(
        token not in motion_math for token in required_math_tokens
    ):
        raise ValueError("R114 quantization or stencil source semantics differ")
    return actual


def audit_descriptor_lineage(
    *,
    profile: Mapping[str, Any],
    descriptor_bytes: bytes,
    legacy_descriptor_path: Path,
    v9_profile_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["current_descriptor"]
    if (
        hashlib.sha256(descriptor_bytes).hexdigest()
        != expected["mirror_v2_file_sha256"]
        or sha256(legacy_descriptor_path)
        != profile["source"]["legacy_kinematic_descriptor_sha256"]
    ):
        raise ValueError("R114 descriptor byte identity differs")
    current = json.loads(descriptor_bytes)
    legacy = json.loads(legacy_descriptor_path.read_bytes())
    validate_current_biomechanics_descriptor(current)
    validate_biomechanics_descriptor(legacy)
    if (
        current.get("body_schema_hash") != expected["body_schema_hash"]
        or current.get("compiled_descriptor_hash")
        != expected["compiled_descriptor_hash"]
        or current.get("material_lineage_hash") != expected["material_lineage_hash"]
        or any(
            current.get(key) != legacy.get(key) for key in KINEMATIC_DESCRIPTOR_FIELDS
        )
    ):
        raise ValueError("R114 current-to-legacy kinematic lineage differs")
    v9_profile = json.loads(v9_profile_path.read_bytes())
    if (
        v9_profile.get("source", {}).get("descriptor_sha256")
        != profile["source"]["legacy_kinematic_descriptor_sha256"]
    ):
        raise ValueError("R114 V9 descriptor lineage differs")
    kinematic_projection = {key: current[key] for key in KINEMATIC_DESCRIPTOR_FIELDS}
    return {
        "status": "EXACT_KINEMATIC_SUCCESSOR_IDENTITY",
        "current_schema_version": current["schema_version"],
        "legacy_schema_version": legacy["schema_version"],
        "body_schema_hash": current["body_schema_hash"],
        "compiled_descriptor_hash": current["compiled_descriptor_hash"],
        "material_lineage_hash": current["material_lineage_hash"],
        "kinematic_projection_sha256": hashlib.sha256(
            canonical_json(kinematic_projection)
        ).hexdigest(),
        "kinematic_fields_equal": True,
        "legacy_material_reinterpretation": False,
        "current_descriptor": current,
    }


def audit_trajectory_lineage(
    *,
    profile: Mapping[str, Any],
    v9_profile_path: Path,
    v9_manifest_path: Path,
    v9_complete_clip_path: Path,
    v7_profile_path: Path,
    v7_manifest_path: Path,
    v7_case_path: Path,
) -> dict[str, Any]:
    expected_v9 = profile["source"]["v9"]
    expected_v7 = profile["source"]["v7_prior"]
    identities = (
        (v9_profile_path, expected_v9["prototype_profile_sha256"]),
        (v9_manifest_path, expected_v9["prototype_manifest_file_sha256"]),
        (v9_complete_clip_path, expected_v9["complete_clip_artifact_sha256"]),
        (v7_profile_path, expected_v7["prototype_profile_sha256"]),
        (v7_manifest_path, expected_v7["prototype_manifest_file_sha256"]),
        (v7_case_path, expected_v7["case_artifact_sha256"]),
    )
    if any(sha256(path) != expected for path, expected in identities):
        raise ValueError("R114 trajectory byte identity differs")
    v9_profile = json.loads(v9_profile_path.read_bytes())
    v7_profile = json.loads(v7_profile_path.read_bytes())
    _validate_contact_profiles(profile, v9_profile, v7_profile)
    v9_manifest = _load_canonical_manifest(
        v9_manifest_path,
        expected_v9["prototype_id"],
        expected_v9["prototype_manifest_sha256"],
    )
    v7_manifest = _load_canonical_manifest(
        v7_manifest_path,
        expected_v7["prototype_id"],
        expected_v7["prototype_manifest_sha256"],
    )
    v9_row = _complete_clip_row(v9_manifest, expected_v9)
    v7_row = _prior_case_row(v7_manifest, expected_v7)
    v9_arrays, v9_metadata = _load_complete_arrays(v9_complete_clip_path)
    v7_arrays, v7_metadata = _load_prior_arrays(v7_case_path)
    _validate_artifact_metadata(v9_metadata, v7_metadata)
    stencil = audit_hybrid_stencil(
        profile,
        np.asarray(v9_arrays["contact_modes"], dtype=np.uint8),
    )
    prior = audit_tracking_prior(profile, v9_arrays, v7_arrays)
    return {
        "status": "EXACT_V9_COMPLETE_WITH_LOCAL_V7_PRIOR",
        "v9": {
            "manifest_sha256": v9_manifest["manifest_sha256"],
            "artifact_id": v9_metadata["artifact_id"],
            "artifact_sha256": v9_row["artifact"]["sha256"],
            "frame_count": 801,
            "source_clip_artifact_sha256": v9_metadata["source_clip_artifact_sha256"],
            "projection_status": v9_row["projection_diagnostics"]["status"],
        },
        "v7_prior": {
            "manifest_sha256": v7_manifest["manifest_sha256"],
            "artifact_id": v7_metadata["artifact_id"],
            "artifact_sha256": v7_row["artifact"]["sha256"],
            "frame_first": 238,
            "frame_last": 249,
            "baseline_status": v7_row["baseline_status"],
            "complete_trajectory_role": "FORBIDDEN",
        },
        "hybrid_stencil": stencil,
        "tracking_prior": prior,
        "v9_profile": v9_profile,
    }


def audit_hybrid_stencil(
    profile: Mapping[str, Any], contact_modes: NDArray[np.uint8]
) -> dict[str, Any]:
    active = contact_point_mask(contact_modes)
    indices, coefficients = hybrid_velocity_stencil(active)
    kinds = []
    for frame, (index, coefficient) in enumerate(
        zip(indices, coefficients, strict=True)
    ):
        pair = tuple(int(value) for value in index)
        weights = tuple(float(value) for value in coefficient)
        if pair == (frame, frame + 1) and weights == (-60.0, 60.0):
            kinds.append("forward")
        elif pair == (frame - 1, frame) and weights == (-60.0, 60.0):
            kinds.append("backward")
        elif pair == (frame - 1, frame + 1) and weights == (-30.0, 30.0):
            kinds.append("centered")
        else:
            raise ValueError("R114 hybrid stencil contains an unknown row")
    digest = hashlib.sha256()
    digest.update(indices.astype("<i8", copy=False).tobytes())
    digest.update(coefficients.astype("<f8", copy=False).tobytes())
    counts = dict(sorted(Counter(kinds).items()))
    expected = profile["kinematic_equalities"]
    if (
        VELOCITY_SEMANTICS != expected["stencil_semantics"]
        or digest.hexdigest() != expected["stencil_sha256"]
        or counts != expected["expected_stencil_counts"]
    ):
        raise ValueError("R114 hybrid velocity stencil differs")
    return {
        "status": "EXACT_FIXED_60_HZ_STENCIL",
        "sha256": digest.hexdigest(),
        "row_counts": counts,
        "entry_precedence": True,
        "contact_schedule_mutable": False,
    }


def audit_tracking_prior(
    profile: Mapping[str, Any],
    v9_arrays: Mapping[str, NDArray[Any]],
    v7_arrays: Mapping[str, NDArray[Any]],
) -> dict[str, Any]:
    frames = np.asarray(v7_arrays["reference_frame"], dtype=np.int64)
    expected_frames = np.arange(238, 250, dtype=np.int64)
    immutable_names = ("contact_modes", "contacts", "root_quaternion_q1_30")
    if not np.array_equal(frames, expected_frames) or any(
        not np.array_equal(v9_arrays[name][frames], v7_arrays[name])
        for name in immutable_names
    ):
        raise ValueError("R114 V7 prior is not a matched V9 slice")
    v9_joint = np.asarray(v9_arrays["joint_position_urad"])[frames]
    v7_joint = np.asarray(v7_arrays["joint_position_urad"])
    delta = v7_joint - v9_joint
    differing = int(np.count_nonzero(delta))
    l1 = sum(abs(int(value)) for value in delta.ravel())
    squared = sum(int(value) ** 2 for value in delta.ravel())
    expected = profile["tracking_progress_gate"]
    if (
        differing != expected["expected_differing_cell_count"]
        or l1 != expected["v9_to_v7_l1_distance_microradians"]
        or squared != expected["v9_to_v7_squared_l2_distance_microradians_squared"]
    ):
        raise ValueError("R114 V7 tracking-prior metric differs")
    return {
        "status": "MATCHED_PASSING_LOCAL_PRIOR_ONLY",
        "frame_first": int(frames[0]),
        "frame_last": int(frames[-1]),
        "differing_joint_position_cell_count": differing,
        "v9_to_v7_l1_distance_microradians": l1,
        "v9_to_v7_squared_l2_distance_microradians_squared": squared,
        "root_position_is_tracking_prior": False,
        "root_orientation_is_tracking_prior": False,
        "joint_position_is_tracking_prior": True,
        "independently_admissible_complete_clip": False,
    }


def classify_tracking_progress(
    *,
    v9_joint_position_urad: NDArray[np.int64],
    v7_joint_position_urad: NDArray[np.int64],
    emitted_joint_position_urad: NDArray[np.int64],
) -> dict[str, Any]:
    """Apply R114's exact-integer, nonzero V7 progress predicate."""

    if (
        v9_joint_position_urad.shape != (12, 23)
        or v7_joint_position_urad.shape != v9_joint_position_urad.shape
        or emitted_joint_position_urad.shape != v9_joint_position_urad.shape
        or any(
            array.dtype != np.int64
            for array in (
                v9_joint_position_urad,
                v7_joint_position_urad,
                emitted_joint_position_urad,
            )
        )
    ):
        raise ValueError("R114 tracking-progress array contract differs")
    prior_direction = v7_joint_position_urad - v9_joint_position_urad
    emitted_direction = emitted_joint_position_urad - v9_joint_position_urad
    baseline_squared_distance = sum(
        int(value) ** 2 for value in prior_direction.ravel()
    )
    emitted_squared_distance = sum(
        int(value) ** 2
        for value in (v7_joint_position_urad - emitted_joint_position_urad).ravel()
    )
    directional_dot = sum(
        int(prior) * int(emitted)
        for prior, emitted in zip(
            prior_direction.ravel(), emitted_direction.ravel(), strict=True
        )
    )
    changed = int(np.count_nonzero(emitted_direction))
    passed = (
        changed > 0
        and directional_dot > 0
        and emitted_squared_distance < baseline_squared_distance
    )
    return {
        "status": "PASS" if passed else "FAIL",
        "changed_joint_position_cell_count": changed,
        "directional_dot_microradians_squared": directional_dot,
        "baseline_squared_distance_microradians_squared": (baseline_squared_distance),
        "emitted_squared_distance_microradians_squared": emitted_squared_distance,
        "strict_distance_decrease": (
            emitted_squared_distance < baseline_squared_distance
        ),
        "strict_positive_directional_dot": directional_dot > 0,
    }


def build_variable_inventory(
    *,
    profile: Mapping[str, Any],
    descriptor: Mapping[str, Any],
    v9_profile: Mapping[str, Any],
) -> dict[str, Any]:
    variables = profile["decision_variables"]
    frame_count = profile["scope"]["frame_count"]
    per_knot = (
        variables["configuration_scalar_count_per_knot"]
        + variables["velocity_scalar_count_per_knot"]
        + variables["acceleration_scalar_count_per_knot"]
    )
    if (
        per_knot != variables["scalar_count_per_knot"]
        or frame_count * per_knot != variables["total_scalar_count"]
    ):
        raise ValueError("R114 variable inventory arithmetic differs")
    v9_bounds = v9_profile["projection"]["trajectory_closure"][
        "joint_bounds_microradians"
    ]
    joints = sorted(descriptor["joints"], key=lambda row: row["dof_ordinal"])
    effective_limits = []
    for joint in joints:
        source_minimum, source_maximum = (
            int(value) for value in joint["soft_limit_microradians"]
        )
        local = v9_bounds.get(joint["joint_id"], [source_minimum, source_maximum])
        minimum = max(source_minimum, int(local[0]))
        maximum = min(source_maximum, int(local[1]))
        maximum_velocity = (
            int(joint["maximum_velocity_microradians_per_second"]) * 2500
        ) // 10_000
        if minimum > maximum or maximum_velocity <= 0:
            raise ValueError("R114 effective joint limit is empty")
        effective_limits.append(
            {
                "joint_id": joint["joint_id"],
                "dof_ordinal": int(joint["dof_ordinal"]),
                "position_microradians": [minimum, maximum],
                "maximum_absolute_velocity_microradians_per_second": (maximum_velocity),
                "v9_stricter_position_bound": [minimum, maximum]
                != [source_minimum, source_maximum],
            }
        )
    if [row["dof_ordinal"] for row in effective_limits] != list(range(23)):
        raise ValueError("R114 joint variable order differs")
    return {
        **variables,
        "status": "COMPLETE_FLOATING_BASE_AND_23_DOF_QVA",
        "joint_order": [row["joint_id"] for row in effective_limits],
        "effective_joint_limits": effective_limits,
        "v9_stricter_joint_bound_count": sum(
            row["v9_stricter_position_bound"] for row in effective_limits
        ),
    }


def _validate_contact_profiles(
    profile: Mapping[str, Any],
    v9: Mapping[str, Any],
    v7: Mapping[str, Any],
) -> None:
    closure = v9.get("projection", {}).get("trajectory_closure", {})
    projection = v9.get("projection", {})
    if (
        v9.get("prototype_id") != profile["source"]["v9"]["prototype_id"]
        or v9.get("status") != "FrozenResearchOnly"
        or v7.get("prototype_id") != profile["source"]["v7_prior"]["prototype_id"]
        or v7.get("status") != "FrozenResearchOnly"
        or closure.get("velocity_semantics") != VELOCITY_SEMANTICS
        or closure.get("minimum_collider_height_micrometres") != -2
        or closure.get("maximum_root_vertical_velocity_micrometres_per_second")
        != 200060
        or closure.get("joint_velocity_limit_basis_points") != 2500
        or closure.get("post_root_velocity_closure") is not False
        or closure.get("post_joint_velocity_projection") is not False
        or closure.get("all_collider_samples_constrained") is not True
        or projection.get("maximum_normal_residual_micrometres") != 5000
        or projection.get("maximum_normal_step_micrometres") != 1000
        or projection.get("maximum_tangential_step_micrometres") != 2000
    ):
        raise ValueError("R114 V7/V9 contact profile semantics differ")


def _load_canonical_manifest(
    path: Path, prototype_id: str, expected_hash: str
) -> dict[str, Any]:
    manifest = json.loads(path.read_bytes())
    embedded = manifest.get("manifest_sha256")
    without_hash = dict(manifest)
    without_hash.pop("manifest_sha256", None)
    if (
        manifest.get("schema_version") != 1
        or manifest.get("prototype_id") != prototype_id
        or manifest.get("status") != "PASS"
        or embedded != expected_hash
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError("R114 contact manifest contract differs")
    return manifest


def _complete_clip_row(
    manifest: Mapping[str, Any], expected: Mapping[str, Any]
) -> Mapping[str, Any]:
    rows = [
        row
        for row in manifest.get("complete_clips", ())
        if row.get("clip_id") == "cmu16-walk-nominal-b"
    ]
    if (
        len(rows) != 1
        or rows[0].get("frame_first") != 0
        or rows[0].get("frame_last") != 800
        or rows[0].get("artifact", {}).get("sha256")
        != expected["complete_clip_artifact_sha256"]
        or rows[0].get("projection_diagnostics", {}).get("status") != "PASS"
        or rows[0].get("projection_diagnostics", {}).get("contact_point_deletion_count")
        != 0
    ):
        raise ValueError("R114 V9 complete-clip row differs")
    return rows[0]


def _prior_case_row(
    manifest: Mapping[str, Any], expected: Mapping[str, Any]
) -> Mapping[str, Any]:
    rows = [
        row
        for row in manifest.get("cases", ())
        if row.get("clip_id") == "cmu16-walk-nominal-b"
        and row.get("frame_first") == 238
        and row.get("frame_last") == 249
    ]
    if (
        len(rows) != 1
        or rows[0].get("source_case_ordinal") != 7967
        or rows[0].get("baseline_status") != "PASS"
        or rows[0].get("baseline_reasons") != []
        or rows[0].get("target_failure_categories") != []
        or rows[0].get("artifact", {}).get("sha256") != expected["case_artifact_sha256"]
    ):
        raise ValueError("R114 V7 prior case differs")
    return rows[0]


def _load_complete_arrays(
    path: Path,
) -> tuple[dict[str, NDArray[Any]], dict[str, Any]]:
    with np.load(path, allow_pickle=False) as source:
        arrays = {name: np.asarray(source[name]).copy() for name in source.files}
    metadata = json.loads(arrays.pop("metadata_json_utf8").tobytes().decode("utf-8"))
    actual = {name: (array.shape, str(array.dtype)) for name, array in arrays.items()}
    if actual != EXPECTED_ARRAYS:
        raise ValueError("R114 V9 complete-clip array schema differs")
    return arrays, metadata


def _load_prior_arrays(
    path: Path,
) -> tuple[dict[str, NDArray[Any]], dict[str, Any]]:
    with np.load(path, allow_pickle=False) as source:
        arrays = {name: np.asarray(source[name]).copy() for name in source.files}
    metadata = json.loads(arrays.pop("metadata_json_utf8").tobytes().decode("utf-8"))
    expected = {
        name: ((12, *shape[1:]), dtype)
        for name, (shape, dtype) in EXPECTED_ARRAYS.items()
    }
    if {
        name: (array.shape, str(array.dtype)) for name, array in arrays.items()
    } != expected:
        raise ValueError("R114 V7 prior array schema differs")
    return arrays, metadata


def _validate_artifact_metadata(v9: Mapping[str, Any], v7: Mapping[str, Any]) -> None:
    if (
        v9.get("artifact_id") != "cmu16-walk-nominal-b--complete"
        or v9.get("clip_id") != "cmu16-walk-nominal-b"
        or v9.get("frame_first") != 0
        or v9.get("frame_last") != 800
        or v7.get("artifact_id") != "cmu16-walk-nominal-b--start-0238"
        or v7.get("clip_id") != v9.get("clip_id")
        or v7.get("frame_first") != 238
        or v7.get("frame_last") != 249
        or v7.get("source_case_ordinal") != 7967
        or v7.get("baseline_status") != "PASS"
        or v7.get("source_clip_artifact_sha256")
        != v9.get("source_clip_artifact_sha256")
        or v7.get("effector_ids") != v9.get("effector_ids")
    ):
        raise ValueError("R114 V7/V9 artifact metadata differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected_ids = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected_ids or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R114 validation result differs")
    return normalized


def _validate_repository(repository: Mapping[str, Any]) -> None:
    commit = repository.get("commit")
    if (
        not isinstance(commit, str)
        or len(commit) != 40
        or any(character not in "0123456789abcdef" for character in commit)
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R114 requires a clean repository")


def _validate_canonical_report(
    report: Mapping[str, Any], expected_hash: str, label: str
) -> None:
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected_hash
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError(f"R114 {label} canonical report identity differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    invariants = profile.get("frozen_invariants", {})
    roles = profile.get("source_roles", {})
    variables = profile.get("decision_variables", {})
    equalities = profile.get("kinematic_equalities", {})
    constraints = profile.get("hard_constraints", {})
    progress = profile.get("tracking_progress_gate", {})
    budget = profile.get("execution_budget", {})
    transitions = profile.get("result_transitions", {})
    bounded = profile.get("bounded_acceptance", {})
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "QuantizationAwareKtoExecutionFormulationOnly"
        or scope.get("run_id") != "R114"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("source_case_ordinal") != 7967
        or (scope.get("frame_first"), scope.get("frame_last"), scope.get("frame_count"))
        != (0, 800, 801)
        or (
            scope.get("v7_prior_frame_first"),
            scope.get("v7_prior_frame_last"),
            scope.get("v7_prior_frame_count"),
        )
        != (238, 249, 12)
        or (scope.get("body_count"), scope.get("joint_count")) != (24, 23)
        or (scope.get("motor_hz"), scope.get("physics_hz")) != (60, 240)
        or scope.get("kto_execution_formulations") != 1
        or scope.get("kto_solves") != 0
        or scope.get("candidate_construction") is not False
        or scope.get("warm_start_cache_construction") is not False
        or scope.get("physx_execution") is not False
        or scope.get("training") is not False
        or invariants.get("controller")
        != "fixed zero-residual PD with zero desired velocity and descriptor gains/caps"
        or invariants.get("fresh_scene_authority") != "ADR-070"
        or invariants.get("partial_reset") != "REPORT_ONLY"
        or invariants.get("contact_schedule_changes") != "FORBIDDEN"
        or invariants.get("contact_point_deletion") != "FORBIDDEN"
        or invariants.get("limit_changes") != "FORBIDDEN"
        or invariants.get("rounding_repair") != "FORBIDDEN"
        or invariants.get("coefficient_sweep") != "FORBIDDEN"
        or "never a complete trajectory" not in roles.get("v7_prior", "")
        or variables.get("scalar_count_per_knot") != 87
        or variables.get("total_scalar_count") != 69687
        or variables.get("independent_contact_force_variables") != 0
        or variables.get("independent_effort_variables") != 0
        or equalities.get("stencil_semantics") != VELOCITY_SEMANTICS
        or equalities.get("expected_stencil_counts")
        != {"forward": 10, "backward": 10, "centered": 781}
        or "minimum height -2 micrometres" not in constraints.get("colliders", "")
        or "200060" not in constraints.get("root_vertical_velocity", "")
        or progress.get("expected_differing_cell_count") != 61
        or progress.get("v9_to_v7_l1_distance_microradians") != 527631
        or progress.get("v9_to_v7_squared_l2_distance_microradians_squared")
        != 17142428727
        or budget.get("kto_process_count") != 1
        or budget.get("kto_solve_count") != 1
        or budget.get("maximum_major_iterations") != 12
        or budget.get("maximum_qp_solves") != 12
        or budget.get("maximum_osqp_iterations_per_subproblem") != 100000
        or budget.get("osqp_absolute_tolerance") != 0.00001
        or budget.get("osqp_relative_tolerance") != 0.00001
        or budget.get("osqp_polishing_enabled") is not True
        or budget.get("osqp_adaptive_rho_enabled") is not True
        or budget.get("finite_difference_root_translation_metres") != 0.00001
        or budget.get("finite_difference_root_orientation_radians") != 0.0001
        or budget.get("finite_difference_joint_position_radians") != 0.0001
        or budget.get("maximum_root_translation_step_metres") != 0.02
        or budget.get("maximum_root_orientation_step_radians") != 0.1
        or budget.get("maximum_joint_position_step_radians") != 0.1
        or budget.get("maximum_exact_emission_audits") != 72
        or budget.get("randomized_restart_count") != 0
        or transitions.get("r114_complete")
        != "PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY"
        or transitions.get("r115_exact_fail") != "STOP_AND_RESEARCH"
        or bounded.get("r115_quantization_aware_kto_execution")
        != "AUTHORIZED_ONE_IN_MEMORY_SOLVE_ONLY"
        or bounded.get("r115_solver_private_warm_start_cache")
        != "AUTHORIZED_ONLY_ON_EXACT_PASS"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_kto_solve",
                "inverse_dynamics_solve",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or decision.get("complete")
        != "PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY"
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != ("ruff_check", "ruff_format", "lab_full", "motor", "host_check")
        or profile.get("quantization_and_emission", {}).get(
            "persisted_candidate_artifact"
        )
        != "FORBIDDEN"
        or "external transient R115 cache"
        not in profile.get("quantization_and_emission", {}).get(
            "solver_private_warm_start_cache", ""
        )
        or profile.get("objective", {}).get("weight_or_scale_search") != "FORBIDDEN"
        or profile.get("objective", {}).get("normalization", {}).get("root_translation")
        != "divide displacement by the frozen V9 root-variable scale 0.02 metres"
        or profile.get("exact_acceptance", {}).get("required_status") != "PASS"
        or profile.get("failure_disposition", {}).get(
            "infeasible_or_resource_exhausted"
        )
        != "STOP_AND_RESEARCH"
        or profile.get("execution_budget", {}).get("runtime_versions")
        != {
            "python": f"{sys.version_info.major}.{sys.version_info.minor}",
            "numpy": np.__version__,
            "scipy": scipy.__version__,
            "osqp": osqp.__version__,
        }
    ):
        raise ValueError("R114 KTO formulation profile differs")
