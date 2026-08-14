from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
from typing import Any, Mapping

import numpy as np
from numpy.typing import NDArray

from next_lab import contact_trajectory
from next_lab.contact_feasible_direction_audit import (
    AUDIT_ID as R105_AUDIT_ID,
    CHECK_ID as R105_CHECK_ID,
    _support_columns,
    _validate_profile as _validate_r105_profile,
    build_public_limit_linearization,
    build_raw_basis,
    constraint_violations,
    maximum_constraint_violation,
    project_direction,
    projection_metrics,
)
from next_lab.contact_projected_direction_formulation import (
    CHECK_ID as R106_CHECK_ID,
    FORMULATION_ID as R106_FORMULATION_ID,
    SELECTED_BASIS_ID,
    SELECTED_COEFFICIENTS,
    _validate_profile as _validate_r106_profile,
    validate_r105,
)
from next_lab.contact_target_knot_formulation import (
    FRAME_FIRST,
    FRAME_LAST,
    SOURCE_CASE_ORDINAL,
    _load_arrays,
    _load_case,
    _load_complete_v9_clip,
    _load_manifest,
    _target_array,
    _validate_matched_case,
    apply_anchor,
    array_sha256,
    canonical_json,
    interpolate_coefficients,
    sha256,
)
from next_lab.contact_target_knot_preflight import (
    _load_artifact,
    _metrics,
    _validate_complete_arrays,
    _validate_r103,
    candidate_failure_reasons,
    worst_limit_utilization_basis_points,
)
from scripts.build_contact_manifold_prototype import (
    _tolerances,
    _trajectory_closure,
)


AUDIT_ID = "nextengine.humanoid-contact-projected-direction-exact-audit.v1"
CHECK_ID = "TRAIN-4-CONTACT-PROJECTED-DIRECTION-EXACT-OFFLINE-AUDIT"
SUPPORT_FRAME_FIRST = 240
SUPPORT_FRAME_LAST = 249
SUPPORT_FRAME_COUNT = SUPPORT_FRAME_LAST - SUPPORT_FRAME_FIRST + 1


def build_projected_direction_exact_audit(
    *,
    profile_path: Path,
    source_audit_path: Path,
    r103_report_path: Path,
    r105_report_path: Path,
    r105_profile_path: Path,
    r106_report_path: Path,
    r106_profile_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
    v9_profile_path: Path,
    descriptor_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Run the one optimizer-free R107 projected-direction exact audit."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            source_audit_path,
            r103_report_path,
            r105_report_path,
            r105_profile_path,
            r106_report_path,
            r106_profile_path,
            v7_manifest_path,
            v9_manifest_path,
            v9_profile_path,
            descriptor_path,
            tool_path,
        )
    )
    (
        profile_path,
        source_audit_path,
        r103_report_path,
        r105_report_path,
        r105_profile_path,
        r106_report_path,
        r106_profile_path,
        v7_manifest_path,
        v9_manifest_path,
        v9_profile_path,
        descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("projected-direction exact-audit input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    if (
        sha256(source_audit_path) != profile["source"]["audit_sha256"]
        or sha256(v9_profile_path)
        != profile["source"]["v9_profile_sha256"]
        or sha256(descriptor_path)
        != profile["source"]["descriptor_sha256"]
    ):
        raise ValueError("projected-direction exact-audit source differs")

    r103 = _validate_r103(
        report_path=r103_report_path,
        expected=profile["source"]["r103"],
    )
    r105 = validate_r105(
        report_path=r105_report_path,
        expected=profile["source"]["r105"],
    )
    if sha256(r105_profile_path) != profile["source"]["r105"]["profile_sha256"]:
        raise ValueError("R105 profile identity differs")
    r105_profile = json.loads(r105_profile_path.read_bytes())
    _validate_r105_profile(r105_profile)
    if r105_profile["analysis"] != r105["analysis_contract"]:
        raise ValueError("R105 analysis contract differs")
    r106 = validate_r106(
        report_path=r106_report_path,
        profile_path=r106_profile_path,
        expected=profile["source"]["r106"],
    )
    if (
        r105["identities"]["r103_report_sha256"] != r103["report_sha256"]
        or r106["identities"]["r105_report_sha256"]
        != r105["report_sha256"]
        or r106["source_lineage"]["r103_report_sha256"]
        != r103["report_sha256"]
    ):
        raise ValueError("R103/R105/R106 lineage differs")

    v7_manifest = _load_manifest(
        path=v7_manifest_path,
        expected=profile["source"]["v7"],
        source_audit_path=source_audit_path,
    )
    v9_manifest = _load_manifest(
        path=v9_manifest_path,
        expected=profile["source"]["v9"],
        source_audit_path=source_audit_path,
    )
    v7_case, v7_artifact = _load_case(
        manifest=v7_manifest,
        manifest_path=v7_manifest_path,
        expected=profile["source"]["v7"],
    )
    v9_case, v9_artifact = _load_case(
        manifest=v9_manifest,
        manifest_path=v9_manifest_path,
        expected=profile["source"]["v9"],
    )
    _validate_matched_case(v7_case, v9_case)
    complete_record, complete_artifact = _load_complete_v9_clip(
        manifest=v9_manifest,
        manifest_path=v9_manifest_path,
        expected=profile["source"]["v9"],
    )
    if (
        v7_manifest["manifest_sha256"]
        != r106["source_lineage"]["v7_manifest_sha256"]
        or v9_manifest["manifest_sha256"]
        != r106["source_lineage"]["v9_manifest_sha256"]
        or sha256(complete_artifact)
        != r106["source_lineage"]["v9_complete_clip_artifact_sha256"]
    ):
        raise ValueError("R106 source artifact lineage differs")

    v7_target = _target_array(_load_arrays(v7_artifact))
    v9_target = _target_array(_load_arrays(v9_artifact))
    anchor_delta = v7_target - v9_target
    selected_r103 = next(
        (
            row
            for row in r103["candidate_lattice"]
            if tuple(row["knot_coefficients_basis_points"])
            == SELECTED_COEFFICIENTS
        ),
        None,
    )
    if selected_r103 is None:
        raise ValueError("R103 late anchor basis is absent")
    anchor_target = apply_anchor(
        v9_target,
        anchor_delta,
        interpolate_coefficients(SELECTED_COEFFICIENTS),
    )
    if array_sha256(anchor_target) != selected_r103["target_sha256"]:
        raise ValueError("R103 late anchor target identity differs")

    complete_arrays, complete_metadata = _load_artifact(complete_artifact)
    frame_count = int(profile["scope"]["complete_clip_frame_count"])
    _validate_complete_arrays(
        arrays=complete_arrays,
        metadata=complete_metadata,
        frame_count=frame_count,
        v9_target=v9_target,
    )
    descriptor = json.loads(descriptor_path.read_bytes())
    v9_profile = json.loads(v9_profile_path.read_bytes())
    tolerances = _tolerances(v9_profile["projection"])
    closure = _trajectory_closure(v9_profile["projection"])
    if closure is None:
        raise ValueError("V9 coupled-trajectory closure is absent")

    linearization = build_public_limit_linearization(
        descriptor=descriptor,
        effector_ids=tuple(complete_metadata["effector_ids"]),
        arrays=complete_arrays,
        tolerances=tolerances,
        closure=closure,
    )
    support_columns = _support_columns(
        frame_first=SUPPORT_FRAME_FIRST,
        frame_last=SUPPORT_FRAME_LAST,
        local_variable_count=linearization.local_variable_count,
    )
    local_matrix = linearization.matrix[:, support_columns].tocsc()
    relevant_rows = np.flatnonzero(np.diff(local_matrix.tocsr().indptr) > 0)
    local_matrix = local_matrix[relevant_rows].tocsc()
    local_lower = linearization.lower[relevant_rows]
    local_upper = linearization.upper[relevant_rows]
    local_categories = tuple(
        linearization.row_categories[int(row)] for row in relevant_rows
    )
    expected_linearization = r105["linearization"]
    if (
        len(support_columns)
        != expected_linearization["local_support_variable_count"]
        or len(relevant_rows)
        != expected_linearization["local_relevant_constraint_count"]
        or linearization.selected_dof_ordinals.tolist()
        != expected_linearization["selected_dof_ordinals"]
        or list(linearization.selected_joint_ids)
        != expected_linearization["selected_joint_ids"]
        or maximum_constraint_violation(
            matrix=local_matrix,
            lower=local_lower,
            upper=local_upper,
            value=np.zeros(len(support_columns), dtype=np.float64),
        )
        > float(r105["analysis_contract"]["feasibility_tolerance"])
    ):
        raise ValueError("R105 local linearization reconstruction differs")

    raw = build_raw_basis(
        coefficients=SELECTED_COEFFICIENTS,
        v9_target=v9_target,
        anchor_delta=anchor_delta,
        support_frame_first=SUPPORT_FRAME_FIRST,
        support_frame_last=SUPPORT_FRAME_LAST,
        local_variable_count=linearization.local_variable_count,
        local_scale=linearization.local_scale,
        selected_dof_ordinals=linearization.selected_dof_ordinals,
    )
    raw_violations = constraint_violations(
        matrix=local_matrix,
        lower=local_lower,
        upper=local_upper,
        value=raw,
        row_categories=local_categories,
        tolerance=float(r105["analysis_contract"]["feasibility_tolerance"]),
    )
    raw_facts = {
        "normalized_l2_norm": _finite_float(np.linalg.norm(raw)),
        "violating_row_count": raw_violations["row_count"],
        "violation_category_counts": raw_violations["category_counts"],
        "maximum_normalized_violation": raw_violations[
            "maximum_normalized_violation"
        ],
    }
    selected_r105 = next(
        row
        for row in r105["basis_results"]
        if row["basis_id"] == SELECTED_BASIS_ID
    )
    if raw_facts != selected_r105["raw_direction"]:
        raise ValueError("R105 raw late direction reconstruction differs")

    projected, projection_solve = project_direction(
        raw=raw,
        matrix=local_matrix,
        lower=local_lower,
        upper=local_upper,
        solver=r105["analysis_contract"]["solver"],
    )
    projected_metrics = projection_metrics(
        raw=raw,
        projected=projected,
        local_scale=linearization.local_scale,
        support_frame_count=SUPPORT_FRAME_COUNT,
    )
    projected_violation = _finite_float(
        maximum_constraint_violation(
            matrix=local_matrix,
            lower=local_lower,
            upper=local_upper,
            value=projected,
        )
    )
    if (
        projected_metrics != selected_r105["projection"]
        or projected_violation
        != selected_r105[
            "maximum_projected_normalized_constraint_violation"
        ]
        or projected_metrics != profile["expected_projection"]
    ):
        raise ValueError("R105 projected late direction identity differs")

    root_delta_um, joint_delta_urad = quantize_projected_direction(
        projected=projected,
        local_scale=linearization.local_scale,
        support_frame_count=SUPPORT_FRAME_COUNT,
    )
    candidate_root, candidate_joint = construct_candidate_positions(
        source_root=np.asarray(complete_arrays["root_position_um"]),
        source_joint=np.asarray(complete_arrays["joint_position_urad"]),
        root_delta_um=root_delta_um,
        joint_delta_urad=joint_delta_urad,
        selected_dof_ordinals=linearization.selected_dof_ordinals,
        support_frame_first=SUPPORT_FRAME_FIRST,
        support_frame_last=SUPPORT_FRAME_LAST,
    )
    immutable = validate_candidate_position_scope(
        source_root=np.asarray(complete_arrays["root_position_um"]),
        source_joint=np.asarray(complete_arrays["joint_position_urad"]),
        candidate_root=candidate_root,
        candidate_joint=candidate_joint,
        selected_dof_ordinals=linearization.selected_dof_ordinals,
        support_frame_first=SUPPORT_FRAME_FIRST,
        support_frame_last=SUPPORT_FRAME_LAST,
    )

    quantized_direction = quantized_normalized_direction(
        root_delta_um=root_delta_um,
        joint_delta_urad=joint_delta_urad,
        local_scale=linearization.local_scale,
    )
    quantized_linear_violation = _finite_float(
        maximum_constraint_violation(
            matrix=local_matrix,
            lower=local_lower,
            upper=local_upper,
            value=quantized_direction,
        )
    )
    frozen_names = (
        "root_quaternion_q1_30",
        "root_yaw_velocity_urad_s",
        "contacts",
        "contact_modes",
    )
    frozen_before = {
        name: array_identity(np.asarray(complete_arrays[name]))
        for name in frozen_names
    }

    active = contact_trajectory.contact_point_mask(
        complete_arrays["contact_modes"]
    )
    stencil_indices, stencil_coefficients = (
        contact_trajectory.hybrid_velocity_stencil(active)
    )

    def velocity(values: NDArray[np.float64]) -> NDArray[np.float64]:
        weights = stencil_coefficients.reshape(
            (frame_count, 2) + (1,) * (values.ndim - 1)
        )
        return np.sum(values[stencil_indices] * weights, axis=1)

    collider_floor = (
        closure.minimum_collider_height_micrometres / 1_000_000.0
    )
    (
        state,
        root_um,
        joint_um,
        effector_um,
        center_um,
        root_velocity_um,
        joint_velocity_um,
    ) = contact_trajectory._exact_state(
        descriptor=descriptor,
        effector_ids=tuple(complete_metadata["effector_ids"]),
        root_values=candidate_root.astype(np.float64) / 1_000_000.0,
        quaternion_q1_30=complete_arrays["root_quaternion_q1_30"],
        yaw_velocity_urad_s=complete_arrays["root_yaw_velocity_urad_s"],
        joint_values=candidate_joint.astype(np.float64) / 1_000_000.0,
        modes=complete_arrays["contact_modes"],
        velocity=velocity,
        collider_floor=collider_floor,
        tolerances=tolerances,
        closure=closure,
    )
    if not np.array_equal(root_um, candidate_root) or not np.array_equal(
        joint_um, candidate_joint
    ):
        raise AssertionError("exact audit changed quantized position inputs")
    frozen_after = {
        name: array_identity(np.asarray(complete_arrays[name]))
        for name in frozen_names
    }
    if frozen_before != frozen_after:
        raise AssertionError("exact audit mutated a frozen source array")
    if not (
        np.array_equal(
            root_um[FRAME_FIRST],
            complete_arrays["root_position_um"][FRAME_FIRST],
        )
        and np.array_equal(
            joint_um[FRAME_FIRST],
            complete_arrays["joint_position_urad"][FRAME_FIRST],
        )
        and np.array_equal(
            root_velocity_um[FRAME_FIRST],
            complete_arrays["root_linear_velocity_um_s"][FRAME_FIRST],
        )
        and np.array_equal(
            joint_velocity_um[FRAME_FIRST],
            complete_arrays["joint_velocity_urad_s"][FRAME_FIRST],
        )
    ):
        raise AssertionError("R107 changed the frozen frame-238 state")

    metrics = _metrics(state)
    reasons = candidate_failure_reasons(
        metrics=metrics,
        limits=profile["limits"],
    )
    exact_status = (
        "PASS" if state["status"] == "PASS" and not reasons else "FAIL"
    )
    decision = profile["decision"]["pass" if exact_status == "PASS" else "fail"]

    exact_arrays = {
        "root_position_um": root_um,
        "root_quaternion_q1_30": complete_arrays["root_quaternion_q1_30"],
        "root_yaw_velocity_urad_s": complete_arrays[
            "root_yaw_velocity_urad_s"
        ],
        "joint_position_urad": joint_um,
        "joint_velocity_urad_s": joint_velocity_um,
        "root_linear_velocity_um_s": root_velocity_um,
        "effector_position_um": effector_um,
        "center_of_mass_um": center_um,
        "contacts": complete_arrays["contacts"],
        "contact_modes": complete_arrays["contact_modes"],
    }
    selected_slice = selected_slice_identity(
        arrays=exact_arrays,
        frame_first=FRAME_FIRST,
        frame_last=FRAME_LAST,
    )
    if selected_slice["status"] != "PASS":
        raise AssertionError("R107 selected slice is not an exact complete slice")

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": decision,
        "audit_id": AUDIT_ID,
        "scope": profile["scope"],
        "method": profile["method"],
        "limits": profile["limits"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "source_audit_sha256": sha256(source_audit_path),
            "r103_report_sha256": r103["report_sha256"],
            "r103_report_file_sha256": sha256(r103_report_path),
            "r105_report_sha256": r105["report_sha256"],
            "r105_report_file_sha256": sha256(r105_report_path),
            "r105_profile_sha256": sha256(r105_profile_path),
            "r106_report_sha256": r106["report_sha256"],
            "r106_report_file_sha256": sha256(r106_report_path),
            "r106_profile_sha256": sha256(r106_profile_path),
            "v7_manifest_sha256": v7_manifest["manifest_sha256"],
            "v7_manifest_file_sha256": sha256(v7_manifest_path),
            "v7_case_artifact_sha256": sha256(v7_artifact),
            "v9_manifest_sha256": v9_manifest["manifest_sha256"],
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "v9_case_artifact_sha256": sha256(v9_artifact),
            "v9_complete_clip_artifact_sha256": sha256(complete_artifact),
            "v9_profile_sha256": sha256(v9_profile_path),
            "descriptor_sha256": sha256(descriptor_path),
            "contact_trajectory_module_sha256": sha256(
                Path(contact_trajectory.__file__).resolve()
            ),
            "r105_audit_module_sha256": r105["identities"][
                "audit_module_sha256"
            ],
            "r106_formulation_module_sha256": r106["identities"][
                "formulation_module_sha256"
            ],
            "tool_sha256": sha256(tool_path),
            "audit_module_sha256": sha256(Path(__file__).resolve()),
        },
        "source_records": {
            "clip_id": complete_record["clip_id"],
            "source_case_ordinal": SOURCE_CASE_ORDINAL,
            "frame_first": FRAME_FIRST,
            "frame_last": FRAME_LAST,
        },
        "projection_reconstruction": {
            "basis_id": SELECTED_BASIS_ID,
            "knot_coefficients_basis_points": list(SELECTED_COEFFICIENTS),
            "raw_direction": raw_facts,
            "projection": projected_metrics,
            "projection_solve": projection_solve,
            "maximum_projected_normalized_constraint_violation": (
                projected_violation
            ),
            "maximum_quantized_normalized_constraint_violation": (
                quantized_linear_violation
            ),
            "quantized_linear_proxy_acceptance_authority": False,
            "local_support_variable_count": len(support_columns),
            "local_relevant_constraint_count": len(relevant_rows),
            "selected_joint_ids": list(linearization.selected_joint_ids),
            "selected_dof_ordinals": (
                linearization.selected_dof_ordinals.tolist()
            ),
        },
        "quantized_candidate": {
            "construction": "IN_MEMORY_ONLY",
            "candidate_artifact": "NOT_EMITTED",
            "root_delta": array_identity(root_delta_um),
            "joint_delta": array_identity(joint_delta_urad),
            "changed_root_position_cell_count": int(
                np.count_nonzero(root_delta_um)
            ),
            "changed_joint_position_cell_count": int(
                np.count_nonzero(joint_delta_urad)
            ),
            "maximum_absolute_root_delta_micrometres": int(
                np.max(np.abs(root_delta_um), initial=0)
            ),
            "maximum_absolute_joint_delta_microradians": int(
                np.max(np.abs(joint_delta_urad), initial=0)
            ),
            "position_scope": immutable,
            "reference_target_equals_joint_position": True,
            "separate_control_target_array": "ABSENT",
        },
        "exact_offline_result": {
            "status": exact_status,
            "failure_reasons": reasons,
            "metrics": metrics,
            "worst_limit_utilization_basis_points": (
                worst_limit_utilization_basis_points(
                    metrics=metrics,
                    limits=profile["limits"],
                )
            ),
            "violating_collider_sample_count": int(
                state["violating_collider_sample_count"]
            ),
            "frame_238_state_matches_v9": True,
            "frozen_source_arrays": frozen_after,
            "dependent_change_facts": {
                "root_linear_velocity_um_s": change_facts(
                    root_velocity_um,
                    complete_arrays["root_linear_velocity_um_s"],
                ),
                "joint_velocity_urad_s": change_facts(
                    joint_velocity_um,
                    complete_arrays["joint_velocity_urad_s"],
                ),
                "effector_position_um": change_facts(
                    effector_um,
                    complete_arrays["effector_position_um"],
                ),
                "center_of_mass_um": change_facts(
                    center_um,
                    complete_arrays["center_of_mass_um"],
                ),
            },
            "selected_slice_identity": selected_slice,
        },
        "failure_disposition": profile["failure_disposition"],
        "bounded_acceptance": {
            "r108_bounded_fresh_discriminator_formulation": (
                "AUTHORIZED" if exact_status == "PASS" else "NOT_AUTHORIZED"
            ),
            "progressive_kinodynamic_formulation": (
                "NOT_SELECTED" if exact_status == "PASS" else "SELECTED"
            ),
            "candidate_artifact": "NOT_AUTHORIZED",
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
        "projection_qp_solves": 1,
        "candidate_target_constructions": 1,
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": 1,
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r106(
    *,
    report_path: Path,
    profile_path: Path,
    expected: Mapping[str, Any],
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R106 report file identity differs")
    if sha256(profile_path) != expected.get("profile_sha256"):
        raise ValueError("R106 profile identity differs")
    formulation_profile = json.loads(profile_path.read_bytes())
    _validate_r106_profile(formulation_profile)
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    bounded = report.get("bounded_acceptance", {})
    selected = report.get("selected_direction", {})
    if (
        report.get("check") != R106_CHECK_ID
        or report.get("formulation_id") != R106_FORMULATION_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R107_EXACT_OFFLINE_PROJECTED_DIRECTION_AUDIT_ONLY"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("identities", {}).get("profile_sha256")
        != expected.get("profile_sha256")
        or report.get("repository", {}).get("dirty") is not False
        or selected.get("basis_id") != SELECTED_BASIS_ID
        or tuple(selected.get("knot_coefficients_basis_points", ()))
        != SELECTED_COEFFICIENTS
        or report.get("reconstruction_contract")
        != formulation_profile["reconstruction"]
        or report.get("quantization_contract")
        != formulation_profile["quantization"]
        or report.get("future_exact_offline_audit")
        != formulation_profile["future_exact_offline_audit"]
        or bounded.get("r107_exact_offline_projected_direction_audit")
        != "AUTHORIZED"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "candidate_target_construction",
                "candidate_artifact",
                "candidate_search",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "projection_qp_solves",
                "candidate_target_constructions",
                "candidate_artifacts_built",
                "offline_candidate_evaluations",
                "physx_runs",
                "candidate_evaluations",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R106 projected-direction contract differs")
    return report


def quantize_projected_direction(
    *,
    projected: NDArray[np.float64],
    local_scale: NDArray[np.float64],
    support_frame_count: int,
) -> tuple[NDArray[np.int64], NDArray[np.int64]]:
    local_variable_count = len(local_scale)
    if (
        local_variable_count < 4
        or projected.shape != (support_frame_count * local_variable_count,)
    ):
        raise ValueError("projected-direction quantization shape differs")
    physical = projected.reshape(support_frame_count, local_variable_count)
    physical = physical * local_scale[None, :]
    root_delta_um = np.rint(physical[:, :3] * 1_000_000.0).astype(np.int64)
    joint_delta_urad = np.rint(physical[:, 3:] * 1_000_000.0).astype(
        np.int64
    )
    return root_delta_um, joint_delta_urad


def construct_candidate_positions(
    *,
    source_root: NDArray[Any],
    source_joint: NDArray[Any],
    root_delta_um: NDArray[np.int64],
    joint_delta_urad: NDArray[np.int64],
    selected_dof_ordinals: NDArray[np.int64],
    support_frame_first: int,
    support_frame_last: int,
) -> tuple[NDArray[np.int64], NDArray[np.int64]]:
    root = np.asarray(source_root, dtype=np.int64)
    joint = np.asarray(source_joint, dtype=np.int64)
    frame_count = support_frame_last - support_frame_first + 1
    if (
        root.ndim != 2
        or root.shape[1] != 3
        or joint.ndim != 2
        or joint.shape[0] != root.shape[0]
        or root_delta_um.shape != (frame_count, 3)
        or joint_delta_urad.shape
        != (frame_count, len(selected_dof_ordinals))
        or support_frame_first < 0
        or support_frame_last >= len(root)
        or len(set(int(value) for value in selected_dof_ordinals))
        != len(selected_dof_ordinals)
        or np.any(selected_dof_ordinals < 0)
        or np.any(selected_dof_ordinals >= joint.shape[1])
    ):
        raise ValueError("projected candidate position shape differs")
    candidate_root = root.copy()
    candidate_joint = joint.copy()
    interval = slice(support_frame_first, support_frame_last + 1)
    candidate_root[interval] += root_delta_um
    for local, ordinal in enumerate(selected_dof_ordinals):
        candidate_joint[interval, int(ordinal)] += joint_delta_urad[:, local]
    return candidate_root, candidate_joint


def validate_candidate_position_scope(
    *,
    source_root: NDArray[Any],
    source_joint: NDArray[Any],
    candidate_root: NDArray[Any],
    candidate_joint: NDArray[Any],
    selected_dof_ordinals: NDArray[np.int64],
    support_frame_first: int,
    support_frame_last: int,
) -> dict[str, Any]:
    source_root = np.asarray(source_root)
    source_joint = np.asarray(source_joint)
    candidate_root = np.asarray(candidate_root)
    candidate_joint = np.asarray(candidate_joint)
    if source_root.shape != candidate_root.shape or source_joint.shape != candidate_joint.shape:
        raise ValueError("candidate position scope shape differs")
    outside = np.ones(len(source_root), dtype=np.bool_)
    outside[support_frame_first : support_frame_last + 1] = False
    selected = {int(value) for value in selected_dof_ordinals}
    nonselected = np.asarray(
        [ordinal for ordinal in range(source_joint.shape[1]) if ordinal not in selected],
        dtype=np.int64,
    )
    if (
        not np.array_equal(candidate_root[outside], source_root[outside])
        or not np.array_equal(candidate_joint[outside], source_joint[outside])
        or not np.array_equal(
            candidate_joint[:, nonselected], source_joint[:, nonselected]
        )
    ):
        raise ValueError("candidate changed an immutable position input")
    return {
        "status": "PASS",
        "support_frame_first": support_frame_first,
        "support_frame_last": support_frame_last,
        "outside_support_root_position_exact": True,
        "outside_support_joint_position_exact": True,
        "all_nonselected_joint_positions_exact": True,
        "frame_238_root_joint_positions_exact": bool(
            support_frame_first > FRAME_FIRST
            and np.array_equal(candidate_root[FRAME_FIRST], source_root[FRAME_FIRST])
            and np.array_equal(candidate_joint[FRAME_FIRST], source_joint[FRAME_FIRST])
        ),
        "frame_239_root_joint_positions_exact": bool(
            support_frame_first > FRAME_FIRST + 1
            and np.array_equal(
                candidate_root[FRAME_FIRST + 1], source_root[FRAME_FIRST + 1]
            )
            and np.array_equal(
                candidate_joint[FRAME_FIRST + 1], source_joint[FRAME_FIRST + 1]
            )
        ),
    }


def quantized_normalized_direction(
    *,
    root_delta_um: NDArray[np.int64],
    joint_delta_urad: NDArray[np.int64],
    local_scale: NDArray[np.float64],
) -> NDArray[np.float64]:
    physical = np.concatenate((root_delta_um, joint_delta_urad), axis=1)
    physical = physical.astype(np.float64) / 1_000_000.0
    if physical.shape[1] != len(local_scale):
        raise ValueError("quantized direction scale shape differs")
    return (physical / local_scale[None, :]).reshape(-1)


def selected_slice_identity(
    *, arrays: Mapping[str, NDArray[Any]], frame_first: int, frame_last: int
) -> dict[str, Any]:
    frame_count = frame_last - frame_first + 1
    if not arrays or any(
        np.asarray(value).ndim == 0
        or len(np.asarray(value)) <= frame_last
        for value in arrays.values()
    ):
        return {"status": "FAIL", "array_count": len(arrays)}
    identities = {
        name: array_identity(np.asarray(value)[frame_first : frame_last + 1])
        for name, value in sorted(arrays.items())
    }
    return {
        "status": "PASS",
        "construction": "direct byte slices of the one exact complete-clip result",
        "frame_first": frame_first,
        "frame_last": frame_last,
        "frame_count": frame_count,
        "array_count": len(identities),
        "arrays": identities,
    }


def change_facts(
    actual: NDArray[Any], source: NDArray[Any]
) -> dict[str, Any]:
    actual = np.asarray(actual)
    source = np.asarray(source)
    if actual.shape != source.shape:
        raise ValueError("dependent array shape differs")
    changed = actual != source
    per_frame = changed.reshape(len(changed), -1).any(axis=1)
    return {
        "changed_element_count": int(np.count_nonzero(changed)),
        "changed_frame_count": int(np.count_nonzero(per_frame)),
        "changed_frames": np.flatnonzero(per_frame).astype(int).tolist(),
        "result": array_identity(actual),
    }


def array_identity(value: NDArray[Any]) -> dict[str, Any]:
    array = np.ascontiguousarray(value)
    return {
        "dtype": array.dtype.str,
        "shape": list(array.shape),
        "sha256": hashlib.sha256(array.tobytes(order="C")).hexdigest(),
    }


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    method = profile.get("method", {})
    limits = profile.get("limits", {})
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("audit_id") != AUDIT_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim")
        != "OptimizerFreeExactOfflineProjectedDirectionAudit"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("source_case_ordinal") != SOURCE_CASE_ORDINAL
        or scope.get("frame_first") != FRAME_FIRST
        or scope.get("frame_last") != FRAME_LAST
        or scope.get("support_frame_first") != SUPPORT_FRAME_FIRST
        or scope.get("support_frame_last") != SUPPORT_FRAME_LAST
        or scope.get("complete_clip_frame_count") != 801
        or scope.get("candidate_count") != 1
        or method.get("selected_basis_id") != SELECTED_BASIS_ID
        or tuple(method.get("knot_coefficients_basis_points", ()))
        != SELECTED_COEFFICIENTS
        or method.get("projection_qp_solve_count") != 1
        or method.get("candidate_artifact_policy")
        != "construct in memory only; emit exact metrics report, not artifact"
        or method.get("repair_or_grid_refinement") != "FORBIDDEN"
        or method.get("physx_runs") != 0
        or method.get("optimizer_steps") != 0
        or method.get("training_runs") != 0
        or limits
        != {
            "maximum_normal_residual_micrometres": 5000,
            "maximum_normal_step_micrometres": 1000,
            "maximum_tangential_step_micrometres": 2000,
            "maximum_analytic_normal_step_micrometres": 1000,
            "maximum_analytic_tangential_step_micrometres": 2000,
            "minimum_collider_height_micrometres": -2,
            "maximum_root_vertical_velocity_micrometres_per_second": 200060,
            "maximum_joint_velocity_basis_points": 2500,
            "maximum_soft_rom_violation_microradians": 0,
        }
        or profile.get("expected_projection")
        != {
            "normalized_l2_norm": 0.0887876379443,
            "normalized_distance_from_raw": 0.0143870590744,
            "anchor_component_basis_points": 9722,
            "cosine_similarity_basis_points": 9872,
            "quantized_nonzero_joint_cell_count": 45,
            "quantized_maximum_absolute_joint_correction_microradians": 5688,
            "quantized_maximum_root_correction_micrometres": 41,
        }
        or decision
        != {
            "pass": "PERMIT_R108_BOUNDED_FRESH_DISCRIMINATOR_FORMULATION_ONLY",
            "fail": "SELECT_PROGRESSIVE_KINODYNAMIC_FORMULATION",
        }
    ):
        raise ValueError("projected-direction exact-audit profile is invalid")


def _finite_float(value: float) -> float:
    if not math.isfinite(value):
        raise ValueError("non-finite projected-direction metric")
    return float(f"{value:.12g}")
