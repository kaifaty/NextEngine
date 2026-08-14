from __future__ import annotations

import hashlib
import json
from collections import Counter
from pathlib import Path
from typing import Any, Callable, Mapping

import numpy as np
from numpy.typing import NDArray

from next_lab import contact_trajectory
from next_lab.contact_target_knot_formulation import (
    CHECK_ID as R103_CHECK_ID,
    FORMULATION_ID,
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
    canonical_json,
    interpolate_coefficients,
    sha256,
)
from scripts.build_contact_manifold_prototype import (
    _tolerances,
    _trajectory_closure,
)


PREFLIGHT_ID = "nextengine.humanoid-contact-target-knot-preflight.v1"
CHECK_ID = "TRAIN-4-CONTACT-TARGET-KNOT-OFFLINE-PREFLIGHT"
ZERO_CONTROL_ID = "a02-00000-a06-00000-a11-00000"


def build_target_knot_preflight(
    *,
    profile_path: Path,
    source_audit_path: Path,
    r103_report_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
    v9_profile_path: Path,
    descriptor_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
    progress: Callable[[dict[str, Any]], None] | None = None,
) -> dict[str, Any]:
    """Evaluate the frozen R103 lattice through the exact V9 offline gate."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            source_audit_path,
            r103_report_path,
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
        v7_manifest_path,
        v9_manifest_path,
        v9_profile_path,
        descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("target-knot preflight input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    r103 = _validate_r103(
        report_path=r103_report_path,
        expected=profile["source"]["r103"],
    )
    if sha256(source_audit_path) != profile["source"]["audit_sha256"]:
        raise ValueError("source audit identity differs")
    if sha256(v9_profile_path) != profile["source"]["v9_profile_sha256"]:
        raise ValueError("V9 profile identity differs")
    if sha256(descriptor_path) != profile["source"]["descriptor_sha256"]:
        raise ValueError("descriptor identity differs")

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

    v7_target = _target_array(_load_arrays(v7_artifact))
    v9_target = _target_array(_load_arrays(v9_artifact))
    anchor_delta = v7_target - v9_target
    complete_arrays, complete_metadata = _load_artifact(complete_artifact)
    descriptor = json.loads(descriptor_path.read_bytes())
    v9_profile = json.loads(v9_profile_path.read_bytes())
    tolerances = _tolerances(v9_profile["projection"])
    closure = _trajectory_closure(v9_profile["projection"])
    if closure is None:
        raise ValueError("V9 coupled-trajectory closure is absent")

    frame_count = int(profile["scope"]["complete_clip_frame_count"])
    _validate_complete_arrays(
        arrays=complete_arrays,
        metadata=complete_metadata,
        frame_count=frame_count,
        v9_target=v9_target,
    )
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

    root_values = (
        complete_arrays["root_position_um"].astype(np.float64) / 1_000_000.0
    )
    collider_floor = (
        closure.minimum_collider_height_micrometres / 1_000_000.0
    )
    result_rows = []
    for index, candidate_row in enumerate(r103["candidate_lattice"], 1):
        coefficients = tuple(
            int(value)
            for value in candidate_row["knot_coefficients_basis_points"]
        )
        alpha = interpolate_coefficients(coefficients)
        target = apply_anchor(v9_target, anchor_delta, alpha)
        if _array_sha256(target) != candidate_row["target_sha256"]:
            raise ValueError("R103 candidate target identity differs")
        candidate_joint = complete_arrays["joint_position_urad"].copy()
        candidate_joint[FRAME_FIRST : FRAME_LAST + 1] = target
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
            root_values=root_values,
            quaternion_q1_30=complete_arrays["root_quaternion_q1_30"],
            yaw_velocity_urad_s=complete_arrays[
                "root_yaw_velocity_urad_s"
            ],
            joint_values=candidate_joint.astype(np.float64) / 1_000_000.0,
            modes=complete_arrays["contact_modes"],
            velocity=velocity,
            collider_floor=collider_floor,
            tolerances=tolerances,
            closure=closure,
        )
        metrics = _metrics(state)
        reasons = candidate_failure_reasons(
            metrics=metrics,
            limits=profile["limits"],
        )
        exact_initial_state = (
            np.array_equal(
                joint_um[FRAME_FIRST],
                complete_arrays["joint_position_urad"][FRAME_FIRST],
            )
            and np.array_equal(
                joint_velocity_um[FRAME_FIRST],
                complete_arrays["joint_velocity_urad_s"][FRAME_FIRST],
            )
            and np.array_equal(root_um, complete_arrays["root_position_um"])
            and np.array_equal(
                root_velocity_um,
                complete_arrays["root_linear_velocity_um_s"],
            )
        )
        if not exact_initial_state:
            raise AssertionError("candidate changed the frozen initial state")
        row = {
            "candidate_ordinal": index - 1,
            "candidate_id": candidate_row["candidate_id"],
            "knot_coefficients_basis_points": list(coefficients),
            "target_sha256": candidate_row["target_sha256"],
            "status": "PASS" if not reasons and state["status"] == "PASS" else "FAIL",
            "failure_reasons": reasons,
            "metrics": metrics,
            "worst_limit_utilization_basis_points": (
                worst_limit_utilization_basis_points(
                    metrics=metrics, limits=profile["limits"]
                )
            ),
            "changed_target_element_count": int(
                candidate_row["changed_target_element_count"]
            ),
            "maximum_absolute_target_correction_microradians": int(
                candidate_row[
                    "maximum_absolute_target_correction_microradians"
                ]
            ),
            "frame_0_state_matches_v9": exact_initial_state,
            "candidate_artifact": "NOT_EMITTED",
        }
        if candidate_row["candidate_id"] == ZERO_CONTROL_ID:
            _validate_zero_control(
                row=row,
                expected=profile["expected_zero_control"],
                values=(
                    root_um,
                    joint_um,
                    effector_um,
                    center_um,
                    root_velocity_um,
                    joint_velocity_um,
                ),
                arrays=complete_arrays,
            )
        result_rows.append(row)
        if progress is not None:
            progress(
                {
                    "candidate_ordinal": index - 1,
                    "candidate_count": len(r103["candidate_lattice"]),
                    "candidate_id": row["candidate_id"],
                    "status": row["status"],
                    "failure_reasons": row["failure_reasons"],
                    "worst_limit_utilization_basis_points": row[
                        "worst_limit_utilization_basis_points"
                    ],
                }
            )

    summary = preflight_summary(result_rows)
    decision = (
        profile["decision"]["nonzero_pass"]
        if summary["nonzero_pass_count"] > 0
        else profile["decision"]["no_nonzero_pass"]
    )
    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": decision,
        "preflight_id": PREFLIGHT_ID,
        "scope": profile["scope"],
        "method": profile["method"],
        "limits": profile["limits"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "source_audit_sha256": sha256(source_audit_path),
            "r103_report_sha256": r103["report_sha256"],
            "r103_report_file_sha256": sha256(r103_report_path),
            "r103_profile_sha256": r103["identities"]["profile_sha256"],
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
            "tool_sha256": sha256(tool_path),
            "preflight_module_sha256": sha256(Path(__file__).resolve()),
        },
        "source_records": {
            "clip_id": complete_record["clip_id"],
            "source_case_ordinal": SOURCE_CASE_ORDINAL,
            "frame_first": FRAME_FIRST,
            "frame_last": FRAME_LAST,
        },
        "summary": summary,
        "candidates": result_rows,
        "bounded_acceptance": {
            "r105_native_candidate_evaluation": (
                "AUTHORIZED"
                if summary["nonzero_pass_count"] > 0
                else "NOT_AUTHORIZED"
            ),
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
        "candidate_target_constructions": len(result_rows),
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": len(result_rows),
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def candidate_failure_reasons(
    *, metrics: Mapping[str, int | str], limits: Mapping[str, int]
) -> list[str]:
    comparisons = (
        ("contact_normal_residual", "maximum_normal_residual_micrometres"),
        ("contact_finite_normal", "maximum_normal_step_micrometres"),
        (
            "contact_finite_tangential",
            "maximum_tangential_step_micrometres",
        ),
        (
            "contact_analytic_normal",
            "maximum_analytic_normal_step_micrometres",
        ),
        (
            "contact_analytic_tangential",
            "maximum_analytic_tangential_step_micrometres",
        ),
        (
            "root_vertical_velocity",
            "maximum_root_vertical_velocity_micrometres_per_second",
        ),
        ("joint_velocity", "maximum_joint_velocity_basis_points"),
    )
    reasons = [
        reason
        for reason, key in comparisons
        if int(metrics[key]) > int(limits[key])
    ]
    if int(metrics["minimum_collider_height_micrometres"]) < int(
        limits["minimum_collider_height_micrometres"]
    ):
        reasons.append("collider_floor")
    if int(metrics["maximum_soft_rom_violation_microradians"]) > 0:
        reasons.append("soft_rom")
    return reasons


def worst_limit_utilization_basis_points(
    *, metrics: Mapping[str, int | str], limits: Mapping[str, int]
) -> int:
    keys = (
        "maximum_normal_residual_micrometres",
        "maximum_normal_step_micrometres",
        "maximum_tangential_step_micrometres",
        "maximum_analytic_normal_step_micrometres",
        "maximum_analytic_tangential_step_micrometres",
        "maximum_root_vertical_velocity_micrometres_per_second",
        "maximum_joint_velocity_basis_points",
    )
    return max(
        _ceil_div(int(metrics[key]) * 10_000, int(limits[key])) for key in keys
    )


def preflight_summary(rows: list[Mapping[str, Any]]) -> dict[str, Any]:
    if len(rows) != 27 or rows[0].get("candidate_id") != ZERO_CONTROL_ID:
        raise ValueError("preflight candidate inventory differs")
    zero = rows[0]
    nonzero = rows[1:]
    passing = [row for row in rows if row["status"] == "PASS"]
    nonzero_passing = [row for row in nonzero if row["status"] == "PASS"]
    best_failed = min(
        (row for row in nonzero if row["status"] == "FAIL"),
        key=lambda row: (
            int(row["worst_limit_utilization_basis_points"]),
            int(row["maximum_absolute_target_correction_microradians"]),
            str(row["candidate_id"]),
        ),
        default=None,
    )
    reason_counts = Counter(
        reason for row in rows for reason in row["failure_reasons"]
    )
    return {
        "candidate_count": len(rows),
        "pass_count": len(passing),
        "fail_count": len(rows) - len(passing),
        "zero_control_status": zero["status"],
        "nonzero_pass_count": len(nonzero_passing),
        "nonzero_fail_count": len(nonzero) - len(nonzero_passing),
        "failure_reason_counts": dict(sorted(reason_counts.items())),
        "passing_candidate_ids": [row["candidate_id"] for row in passing],
        "best_failed_nonzero_candidate": (
            None
            if best_failed is None
            else {
                "candidate_id": best_failed["candidate_id"],
                "failure_reasons": best_failed["failure_reasons"],
                "worst_limit_utilization_basis_points": best_failed[
                    "worst_limit_utilization_basis_points"
                ],
                "metrics": best_failed["metrics"],
            }
        ),
    }


def _validate_profile(profile: Mapping[str, Any]) -> None:
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("preflight_id") != PREFLIGHT_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim") != "OptimizerFreeExactOfflineLatticePreflight"
        or profile.get("scope", {}).get("candidate_count") != 27
        or profile.get("scope", {}).get("source_case_ordinal")
        != SOURCE_CASE_ORDINAL
        or profile.get("method", {}).get("candidate_artifact_policy")
        != "do not emit any candidate artifact; report exact metrics only"
        or decision
        != {
            "nonzero_pass": "PERMIT_R105_BOUNDED_NATIVE_DISCRIMINATOR_ONLY",
            "no_nonzero_pass": "STOP_AND_RESEARCH",
        }
    ):
        raise ValueError("target-knot preflight profile is invalid")


def _validate_r103(
    *, report_path: Path, expected: Mapping[str, Any]
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R103 report file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        report.get("check") != R103_CHECK_ID
        or report.get("formulation_id") != FORMULATION_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R104_OFFLINE_LATTICE_PREFLIGHT_ONLY"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("identities", {}).get("profile_sha256")
        != expected.get("profile_sha256")
        or len(report.get("candidate_lattice", ())) != 27
        or report.get("bounded_acceptance", {}).get("candidate_search")
        != "NOT_AUTHORIZED"
        or report.get("bounded_acceptance", {}).get("physx")
        != "NOT_AUTHORIZED"
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "candidate_artifacts_built",
                "offline_candidate_evaluations",
                "physx_runs",
                "candidate_evaluations",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R103 formulation contract differs")
    return report


def _load_artifact(
    path: Path,
) -> tuple[dict[str, NDArray[Any]], dict[str, Any]]:
    with np.load(path, allow_pickle=False) as source:
        metadata = json.loads(source["metadata_json_utf8"].tobytes())
        arrays = {
            name: np.array(source[name], copy=True)
            for name in source.files
            if name != "metadata_json_utf8"
        }
    return arrays, metadata


def _validate_complete_arrays(
    *,
    arrays: Mapping[str, NDArray[Any]],
    metadata: Mapping[str, Any],
    frame_count: int,
    v9_target: NDArray[np.int64],
) -> None:
    required = {
        "root_position_um",
        "root_quaternion_q1_30",
        "root_yaw_velocity_urad_s",
        "joint_position_urad",
        "joint_velocity_urad_s",
        "root_linear_velocity_um_s",
        "effector_position_um",
        "center_of_mass_um",
        "contacts",
        "contact_modes",
    }
    if (
        not required.issubset(arrays)
        or metadata.get("clip_id") != "cmu16-walk-nominal-b"
        or len(metadata.get("effector_ids", ())) != 6
        or arrays["joint_position_urad"].shape != (frame_count, 23)
        or not np.array_equal(
            arrays["joint_position_urad"][FRAME_FIRST : FRAME_LAST + 1],
            v9_target,
        )
    ):
        raise ValueError("V9 complete-clip array contract differs")


def _metrics(state: Mapping[str, Any]) -> dict[str, int | str]:
    contact = state["contact"]
    return {
        "exact_state_status": str(state["status"]),
        "active_contact_status": str(contact["status"]),
        "collider_closure_status": str(state["collider_closure_status"]),
        "maximum_normal_residual_micrometres": int(
            contact["maximum_normal_residual_micrometres"]
        ),
        "maximum_normal_step_micrometres": int(
            contact["maximum_normal_step_micrometres"]
        ),
        "maximum_tangential_step_micrometres": int(
            contact["maximum_tangential_step_micrometres"]
        ),
        "maximum_analytic_normal_step_micrometres": int(
            contact["maximum_analytic_normal_step_micrometres"]
        ),
        "maximum_analytic_tangential_step_micrometres": int(
            contact["maximum_analytic_tangential_step_micrometres"]
        ),
        "minimum_collider_height_micrometres": int(
            state["minimum_collider_height_micrometres"]
        ),
        "maximum_root_vertical_velocity_micrometres_per_second": int(
            state["maximum_root_vertical_velocity_micrometres_per_second"]
        ),
        "maximum_joint_velocity_basis_points": int(
            state["maximum_joint_velocity_basis_points"]
        ),
        "maximum_soft_rom_violation_microradians": int(
            state["maximum_soft_rom_violation_microradians"]
        ),
    }


def _validate_zero_control(
    *,
    row: Mapping[str, Any],
    expected: Mapping[str, Any],
    values: tuple[NDArray[Any], ...],
    arrays: Mapping[str, NDArray[Any]],
) -> None:
    baseline = (
        arrays["root_position_um"],
        arrays["joint_position_urad"],
        arrays["effector_position_um"],
        arrays["center_of_mass_um"],
        arrays["root_linear_velocity_um_s"],
        arrays["joint_velocity_urad_s"],
    )
    if (
        row["status"] != "PASS"
        or row["failure_reasons"]
        or row["metrics"] != expected
        or any(not np.array_equal(actual, source) for actual, source in zip(values, baseline, strict=True))
    ):
        raise ValueError("zero-control V9 exact audit differs")


def _array_sha256(value: NDArray[Any]) -> str:
    canonical = np.asarray(value, dtype="<i8")
    return hashlib.sha256(canonical.tobytes(order="C")).hexdigest()


def _ceil_div(numerator: int, denominator: int) -> int:
    return -(-numerator // denominator)
