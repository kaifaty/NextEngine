from __future__ import annotations

import hashlib
import json
import math
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray
from scipy import sparse

from next_lab import contact_manifold
from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.contact_trajectory import hybrid_velocity_stencil
from next_lab.motion_math import (
    matrix_to_quaternion,
    quaternion_to_matrix,
)
from next_lab.quantization_aware_kto_formulation import (
    _load_complete_arrays,
    _load_prior_arrays,
)
from next_lab.quantization_aware_kto_formulation import (
    _validate_profile as _validate_r114_profile,
)
from next_lab.quantization_aware_kto_solver import (
    ACCELERATION_OFFSET,
    BLOCK_WIDTH,
    FRAME_COUNT,
    Q_WIDTH,
    _build_problem,
    _effective_joint_limits,
    _initial_state,
    _linearize_geometry,
    _rotation_exp,
)

RESEARCH_ID = "nextengine.humanoid-post-r115-kto-linearization-research.v1"
CHECK_ID = "TRAIN-4-POST-R115-KTO-LINEARIZATION-RESEARCH"
SIDE_NAMES = ("left", "right")
POINT_NAMES = ("heel", "forefoot")


def build_kto_linearization_research(
    *,
    profile_path: Path,
    r115_report_path: Path,
    r114_profile_path: Path,
    execution_profile_path: Path,
    descriptor_bytes: bytes,
    v9_profile_path: Path,
    v9_complete_clip_path: Path,
    v7_case_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Audit the R115 analytic-contact model without running another solver."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r115_report_path,
            r114_profile_path,
            execution_profile_path,
            v9_profile_path,
            v9_complete_clip_path,
            v7_case_path,
            tool_path,
        )
    )
    (
        profile_path,
        r115_report_path,
        r114_profile_path,
        execution_profile_path,
        v9_profile_path,
        v9_complete_clip_path,
        v7_case_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("post-R115 research input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r115 = validate_r115(
        profile=profile,
        report_path=r115_report_path,
        execution_profile_path=execution_profile_path,
    )
    r114_profile = json.loads(r114_profile_path.read_bytes())
    _validate_r114_profile(r114_profile)
    _validate_source_files(
        profile=profile,
        r114_profile_path=r114_profile_path,
        descriptor_bytes=descriptor_bytes,
        v9_profile_path=v9_profile_path,
        v9_complete_clip_path=v9_complete_clip_path,
        v7_case_path=v7_case_path,
    )
    validations = _validate_results(profile, validation_results)

    descriptor = json.loads(descriptor_bytes)
    v9_profile = json.loads(v9_profile_path.read_bytes())
    v9_arrays, v9_metadata = _load_complete_arrays(v9_complete_clip_path)
    v7_arrays, _ = _load_prior_arrays(v7_case_path)
    active = contact_manifold.contact_point_mask(v9_arrays["contact_modes"])
    stencil_indices, stencil_coefficients = hybrid_velocity_stencil(active)
    reference_rotations = np.stack(
        [
            quaternion_to_matrix(value.astype(np.float64) / float(1 << 30))
            for value in v9_arrays["root_quaternion_q1_30"]
        ]
    )
    state = _initial_state(
        v9_arrays=v9_arrays,
        reference_rotations=reference_rotations,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
    )
    linearization = _linearize_geometry(
        descriptor=descriptor,
        state=state,
        reference_rotations=reference_rotations,
        effector_ids=tuple(v9_metadata["effector_ids"]),
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        r114_profile=r114_profile,
    )
    problem = _build_problem(
        descriptor=descriptor,
        state=state,
        v9_arrays=v9_arrays,
        v7_arrays=v7_arrays,
        active=active,
        stencil_indices=stencil_indices,
        stencil_coefficients=stencil_coefficients,
        linearization=linearization,
        effective_limits=_effective_joint_limits(descriptor, v9_profile),
        r114_profile=r114_profile,
    )

    dependency = analyze_analytic_row_dependencies(
        matrix=problem.constraints,
        category_counts=problem.constraint_categories,
    )
    exact_velocity_um_s = contact_manifold._analytic_active_point_velocities(
        descriptor=descriptor,
        root_positions=v9_arrays["root_position_um"].astype(np.float64) / 1_000_000.0,
        root_quaternions=v9_arrays["root_quaternion_q1_30"].astype(np.float64)
        / float(1 << 30),
        joint_positions=v9_arrays["joint_position_urad"].astype(np.float64)
        / 1_000_000.0,
        root_linear_velocity_um_s=v9_arrays["root_linear_velocity_um_s"],
        root_yaw_velocity_urad_s=v9_arrays["root_yaw_velocity_urad_s"],
        joint_velocity_urad_s=v9_arrays["joint_velocity_urad_s"],
        active=active,
        probe=float(profile["method"]["exact_kernel_probe_radians"]),
    )
    model_velocity_um_s = linearization.analytic_sole_velocity_m_s * 1_000_000.0
    kernel_identity = compare_velocity_models(
        model_velocity_um_s=model_velocity_um_s,
        exact_velocity_um_s=exact_velocity_um_s,
        active=active,
    )
    hotspot = tuple(int(value) for value in kernel_identity["exact_tangent_hotspot"])
    sensitivity = audit_hotspot_configuration_sensitivity(
        descriptor=descriptor,
        v9_arrays=v9_arrays,
        active=active,
        hotspot=hotspot,
        orientation_probe=float(
            profile["method"]["configuration_sensitivity_orientation_probe_radians"]
        ),
        joint_probe=float(
            profile["method"]["configuration_sensitivity_joint_probe_radians"]
        ),
        kernel_probe=float(profile["method"]["exact_kernel_probe_radians"]),
        qp_configuration_nonzero_count=dependency["configuration_nonzero_count"],
        nonzero_threshold=float(
            profile["discriminators"][
                "minimum_nonzero_configuration_sensitivity_um_s_per_rad"
            ]
        ),
    )
    fractions = analyze_r115_fractions(r115)
    discriminators = {
        "r115_is_valid_single_fail": True,
        "quarter_through_one_thirty_second_isolate_analytic_tangent": fractions[
            "small_fraction_isolation"
        ],
        "analytic_qp_rows_have_zero_configuration_nonzeros": dependency[
            "configuration_nonzero_count"
        ]
        == 0,
        "exact_kernel_has_nonzero_configuration_sensitivity": sensitivity[
            "nonzero_variable_count"
        ]
        > 0,
        "fraction_response_is_first_order_consistent": fractions[
            "analytic_tangent_affine_r_squared"
        ]
        >= float(profile["discriminators"]["minimum_affine_r_squared"]),
    }
    confirmed = all(discriminators.values())
    finding = (
        "CONFIRMED_ANALYTIC_CONTACT_LINEARIZATION_IDENTITY_GAP"
        if confirmed
        else "INCONCLUSIVE_RETAIN_STOP_AND_RESEARCH"
    )
    gate = profile["decision"]["confirmed" if confirmed else "inconclusive"]
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "research_id": RESEARCH_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "finding": finding,
        "gate_decision": gate,
        "scope": profile["scope"],
        "r115_result": {
            "status": r115["status"],
            "gate_decision": r115["gate_decision"],
            "report_sha256": r115["report_sha256"],
            "termination": r115["solver_result"]["termination"],
            "qp_status": r115["solver_result"]["qp"]["status"],
            "qp_solves": r115["qp_solves"],
            "exact_emission_audits": r115["in_memory_emitted_iterates"],
        },
        "fraction_analysis": fractions,
        "analytic_row_dependency_audit": dependency,
        "baseline_kernel_identity": kernel_identity,
        "hotspot_configuration_sensitivity": sensitivity,
        "discriminators": discriminators,
        "research_basis": profile["research_basis"],
        "interpretation": {
            "supported": "R115 rejected the frozen implementation direction because every positive audited fraction violated exact analytic tangential contact velocity; the QP analytic rows omit configuration derivatives that the exact kernel measurably has",
            "not_supported": "R115 does not prove that the frozen exact requirements or quantization-aware KTO are intrinsically infeasible",
            "required_successor_scope": "bind the exact contact-kernel function and its complete q/v derivative plus an explicit nonlinear-iterate acceptance/restoration rule before considering any new KTO execution",
        },
        "next_smallest_action": profile["next_smallest_action"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r115_report_file_sha256": sha256(r115_report_path),
            "r114_profile_sha256": sha256(r114_profile_path),
            "execution_profile_sha256": sha256(execution_profile_path),
            "current_descriptor_file_sha256": hashlib.sha256(
                descriptor_bytes
            ).hexdigest(),
            "v9_profile_sha256": sha256(v9_profile_path),
            "v9_complete_clip_sha256": sha256(v9_complete_clip_path),
            "v7_case_sha256": sha256(v7_case_path),
            "contact_manifold_module_sha256": sha256(
                Path(contact_manifold.__file__).resolve()
            ),
            "solver_module_sha256": sha256(
                Path(_build_problem.__code__.co_filename).resolve()
            ),
            "research_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "research_audits": 1,
        "solver_runs": 0,
        "qp_solves": 0,
        "kto_solves": 0,
        "candidate_artifacts_built": 0,
        "solver_private_warm_start_caches": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r115(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    execution_profile_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r115"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(execution_profile_path) != expected["execution_profile_sha256"]
    ):
        raise ValueError("post-R115 source file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    solver = report.get("solver_result", {})
    cache = report.get("solver_private_warm_start_cache", {})
    identities = report.get("identities", {})
    if (
        embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("status") != "FAIL"
        or report.get("gate_decision") != "STOP_AND_RESEARCH"
        or solver.get("termination") != "no_exact_progress_step"
        or solver.get("major_iterations") != 1
        or report.get("qp_solves") != 1
        or report.get("kto_solves") != 1
        or report.get("in_memory_emitted_iterates") != 6
        or cache.get("status") != "NOT_EMITTED"
        or report.get("candidate_artifacts_built") != 0
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "inverse_dynamics_solves",
                "kinodynamic_solves",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("execution_profile_sha256")
        != expected["execution_profile_sha256"]
        or identities.get("execution_module_sha256")
        != expected["execution_module_sha256"]
        or identities.get("solver_module_sha256") != expected["solver_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
    ):
        raise ValueError("post-R115 source report contract differs")
    return report


def analyze_r115_fractions(report: Mapping[str, Any]) -> dict[str, Any]:
    audits = report["solver_result"]["exact_audits"]
    expected = ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
    if tuple(row.get("fraction") for row in audits) != expected:
        raise ValueError("R115 line-search audit inventory differs")
    rows = []
    for row in audits:
        contact = row["contact"]
        progress = row["tracking_progress"]
        baseline = int(progress["baseline_squared_distance_microradians_squared"])
        emitted = int(progress["emitted_squared_distance_microradians_squared"])
        rows.append(
            {
                "fraction": row["fraction"],
                "failure_reasons": row["failure_reasons"],
                "contact_status": contact["status"],
                "maximum_finite_tangential_step_micrometres": contact[
                    "maximum_tangential_step_micrometres"
                ],
                "maximum_finite_normal_step_micrometres": contact[
                    "maximum_normal_step_micrometres"
                ],
                "maximum_analytic_tangential_step_micrometres": contact[
                    "maximum_analytic_tangential_step_micrometres"
                ],
                "maximum_analytic_normal_step_micrometres": contact[
                    "maximum_analytic_normal_step_micrometres"
                ],
                "minimum_collider_height_micrometres": row[
                    "minimum_collider_height_micrometres"
                ],
                "maximum_root_vertical_velocity_micrometres_per_second": row[
                    "maximum_root_vertical_velocity_micrometres_per_second"
                ],
                "maximum_joint_velocity_basis_points": row[
                    "maximum_joint_velocity_basis_points"
                ],
                "tracking_progress_status": progress["status"],
                "tracking_distance_improvement_basis_points": (
                    (baseline - emitted) * 10_000 // baseline
                ),
            }
        )
    small = rows[2:]
    isolated = all(
        row["failure_reasons"] == ["contact"]
        and row["maximum_finite_tangential_step_micrometres"] <= 2000
        and row["maximum_finite_normal_step_micrometres"] <= 1000
        and row["maximum_analytic_normal_step_micrometres"] <= 1000
        and row["maximum_analytic_tangential_step_micrometres"] > 2000
        and row["minimum_collider_height_micrometres"] >= -2
        and row["maximum_root_vertical_velocity_micrometres_per_second"] <= 200060
        and row["maximum_joint_velocity_basis_points"] <= 2500
        and row["tracking_progress_status"] == "PASS"
        for row in small
    )
    fraction_values = np.asarray([_parse_fraction(row["fraction"]) for row in rows])
    analytic_values = np.asarray(
        [row["maximum_analytic_tangential_step_micrometres"] for row in rows],
        dtype=np.float64,
    )
    slope, intercept = np.polyfit(fraction_values, analytic_values, 1)
    predicted = slope * fraction_values + intercept
    residual = float(np.sum((analytic_values - predicted) ** 2))
    total = float(np.sum((analytic_values - np.mean(analytic_values)) ** 2))
    r_squared = 1.0 - residual / total
    return {
        "rows": rows,
        "small_fraction_isolation": isolated,
        "analytic_tangent_affine_slope_micrometres": float(slope),
        "analytic_tangent_affine_intercept_micrometres": float(intercept),
        "analytic_tangent_affine_r_squared": r_squared,
        "one_thirty_second_analytic_excess_micrometres": (
            rows[-1]["maximum_analytic_tangential_step_micrometres"] - 2000
        ),
        "one_thirty_second_tracking_improvement_basis_points": rows[-1][
            "tracking_distance_improvement_basis_points"
        ],
    }


def analyze_analytic_row_dependencies(
    *, matrix: sparse.csc_matrix, category_counts: Mapping[str, int]
) -> dict[str, Any]:
    start = 0
    target: slice | None = None
    for category, count_value in category_counts.items():
        count = int(count_value)
        if category == "contact_analytic_velocity":
            target = slice(start, start + count)
        start += count
    if target is None or start != matrix.shape[0]:
        raise ValueError("analytic contact row inventory differs")
    local_columns = matrix[target].tocoo().col % BLOCK_WIDTH
    configuration = int(np.count_nonzero(local_columns < Q_WIDTH))
    velocity = int(
        np.count_nonzero(
            (local_columns >= Q_WIDTH) & (local_columns < ACCELERATION_OFFSET)
        )
    )
    acceleration = int(np.count_nonzero(local_columns >= ACCELERATION_OFFSET))
    return {
        "row_count": target.stop - target.start,
        "nonzero_count": len(local_columns),
        "configuration_nonzero_count": configuration,
        "velocity_nonzero_count": velocity,
        "acceleration_nonzero_count": acceleration,
        "configuration_dependency_modeled": configuration > 0,
        "velocity_dependency_modeled": velocity > 0,
    }


def compare_velocity_models(
    *,
    model_velocity_um_s: NDArray[np.float64],
    exact_velocity_um_s: NDArray[np.float64],
    active: NDArray[np.bool_],
) -> dict[str, Any]:
    if (
        model_velocity_um_s.shape != exact_velocity_um_s.shape
        or model_velocity_um_s.shape != (*active.shape, 3)
    ):
        raise ValueError("analytic velocity comparison shape differs")
    delta = (model_velocity_um_s - exact_velocity_um_s)[active]
    exact_tangent = np.linalg.norm(exact_velocity_um_s[..., (0, 2)], axis=-1) / 60.0
    model_tangent = np.linalg.norm(model_velocity_um_s[..., (0, 2)], axis=-1) / 60.0
    masked_exact = np.where(active, exact_tangent, -np.inf)
    masked_model = np.where(active, model_tangent, -np.inf)
    exact_hotspot = tuple(
        int(value)
        for value in np.unravel_index(int(np.argmax(masked_exact)), masked_exact.shape)
    )
    model_hotspot = tuple(
        int(value)
        for value in np.unravel_index(int(np.argmax(masked_model)), masked_model.shape)
    )
    return {
        "active_point_frame_count": int(np.sum(active)),
        "maximum_absolute_component_difference_micrometres_per_second": float(
            np.max(np.abs(delta))
        ),
        "root_mean_square_component_difference_micrometres_per_second": float(
            np.sqrt(np.mean(delta * delta))
        ),
        "differing_component_count_above_one_micrometre_per_second": int(
            np.count_nonzero(np.abs(delta) > 1.0)
        ),
        "exact_maximum_tangential_step_micrometres": math.ceil(
            float(masked_exact[exact_hotspot])
        ),
        "model_maximum_tangential_step_micrometres": math.ceil(
            float(masked_model[model_hotspot])
        ),
        "exact_tangent_hotspot": list(exact_hotspot),
        "exact_tangent_hotspot_id": _point_id(exact_hotspot),
        "model_tangent_hotspot": list(model_hotspot),
        "model_tangent_hotspot_id": _point_id(model_hotspot),
        "kernel_identity": (
            "MATCH" if np.max(np.abs(delta)) <= 1.0 else "MISMATCH_DIAGNOSTIC_ONLY"
        ),
    }


def audit_hotspot_configuration_sensitivity(
    *,
    descriptor: dict[str, Any],
    v9_arrays: Mapping[str, NDArray[Any]],
    active: NDArray[np.bool_],
    hotspot: tuple[int, int, int],
    orientation_probe: float,
    joint_probe: float,
    kernel_probe: float,
    qp_configuration_nonzero_count: int,
    nonzero_threshold: float,
) -> dict[str, Any]:
    frame, side, point = hotspot
    if not bool(active[frame, side, point]):
        raise ValueError("analytic tangent hotspot is inactive")
    root_position = (
        np.asarray(v9_arrays["root_position_um"][[frame]], dtype=np.float64)
        / 1_000_000.0
    )
    quaternion = np.asarray(
        v9_arrays["root_quaternion_q1_30"][[frame]], dtype=np.float64
    ) / float(1 << 30)
    joints = (
        np.asarray(v9_arrays["joint_position_urad"][[frame]], dtype=np.float64)
        / 1_000_000.0
    )
    root_velocity = np.asarray(v9_arrays["root_linear_velocity_um_s"][[frame]])
    yaw_velocity = np.asarray(v9_arrays["root_yaw_velocity_urad_s"][[frame]])
    joint_velocity = np.asarray(v9_arrays["joint_velocity_urad_s"][[frame]])
    local_active = np.zeros((1, 2, 2), dtype=np.bool_)
    local_active[0, side, point] = True

    def evaluate(
        candidate_quaternion: NDArray[np.float64],
        candidate_joints: NDArray[np.float64],
    ) -> NDArray[np.float64]:
        value = contact_manifold._analytic_active_point_velocities(
            descriptor=descriptor,
            root_positions=root_position,
            root_quaternions=candidate_quaternion,
            joint_positions=candidate_joints,
            root_linear_velocity_um_s=root_velocity,
            root_yaw_velocity_urad_s=yaw_velocity,
            joint_velocity_urad_s=joint_velocity,
            active=local_active,
            probe=kernel_probe,
        )
        return value[0, side, point]

    rows = []
    rotation = quaternion_to_matrix(quaternion[0])
    for axis, name in enumerate(("world-x", "world-y", "world-z")):
        offset = np.zeros(3, dtype=np.float64)
        offset[axis] = orientation_probe
        plus = quaternion.copy()
        minus = quaternion.copy()
        plus[0] = matrix_to_quaternion(_rotation_exp(offset) @ rotation)
        minus[0] = matrix_to_quaternion(_rotation_exp(-offset) @ rotation)
        derivative = (evaluate(plus, joints) - evaluate(minus, joints)) / (
            2.0 * orientation_probe
        )
        rows.append(_sensitivity_row(f"root-orientation-{name}", derivative))
    joint_ordinals = contact_manifold._leg_joint_ordinals(descriptor)[side]
    joint_by_ordinal = {
        int(row["dof_ordinal"]): row["joint_id"] for row in descriptor["joints"]
    }
    for ordinal_value in joint_ordinals:
        ordinal = int(ordinal_value)
        plus = joints.copy()
        minus = joints.copy()
        plus[0, ordinal] += joint_probe
        minus[0, ordinal] -= joint_probe
        derivative = (evaluate(quaternion, plus) - evaluate(quaternion, minus)) / (
            2.0 * joint_probe
        )
        rows.append(_sensitivity_row(joint_by_ordinal[ordinal], derivative))
    return {
        "frame": frame,
        "side": SIDE_NAMES[side],
        "point": POINT_NAMES[point],
        "point_id": _point_id(hotspot),
        "qp_configuration_nonzero_count_for_all_analytic_rows": (
            qp_configuration_nonzero_count
        ),
        "sensitivity_rows": rows,
        "maximum_tangential_sensitivity_micrometres_per_second_per_radian": max(
            row["tangential_norm_micrometres_per_second_per_radian"] for row in rows
        ),
        "nonzero_threshold_micrometres_per_second_per_radian": nonzero_threshold,
        "nonzero_variable_count": sum(
            row["tangential_norm_micrometres_per_second_per_radian"] > nonzero_threshold
            for row in rows
        ),
    }


def _sensitivity_row(name: str, derivative: NDArray[np.float64]) -> dict[str, Any]:
    return {
        "variable": name,
        "xyz_micrometres_per_second_per_radian": [float(value) for value in derivative],
        "tangential_norm_micrometres_per_second_per_radian": float(
            np.linalg.norm(derivative[[0, 2]])
        ),
    }


def _point_id(value: tuple[int, int, int]) -> str:
    frame, side, point = value
    return f"frame-{frame}:{SIDE_NAMES[side]}-{POINT_NAMES[point]}"


def _parse_fraction(value: str) -> float:
    if "/" not in value:
        return float(value)
    numerator, denominator = value.split("/", maxsplit=1)
    return int(numerator) / int(denominator)


def _validate_source_files(
    *,
    profile: Mapping[str, Any],
    r114_profile_path: Path,
    descriptor_bytes: bytes,
    v9_profile_path: Path,
    v9_complete_clip_path: Path,
    v7_case_path: Path,
) -> None:
    expected = profile["source"]
    if (
        sha256(r114_profile_path) != expected["r114_profile_sha256"]
        or hashlib.sha256(descriptor_bytes).hexdigest()
        != expected["current_descriptor_file_sha256"]
        or sha256(v9_profile_path) != expected["v9_profile_sha256"]
        or sha256(v9_complete_clip_path) != expected["v9_complete_clip_sha256"]
        or sha256(v7_case_path) != expected["v7_case_sha256"]
        or sha256(Path(contact_manifold.__file__).resolve())
        != expected["contact_manifold_module_sha256"]
        or sha256(Path(_build_problem.__code__.co_filename).resolve())
        != expected["r115"]["solver_module_sha256"]
    ):
        raise ValueError("post-R115 research source identity differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("post-R115 research validation differs")
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
        raise ValueError("post-R115 research requires a clean repository")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("research_id") != RESEARCH_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "PostR115AnalyticContactLinearizationAuditOnly"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("frame_count") != FRAME_COUNT
        or scope.get("r115_kto_solves") != 1
        or scope.get("research_kto_solves") != 0
        or scope.get("physx_scene_runs") != 0
        or profile.get("decision", {}).get("confirmed")
        != "PERMIT_REPORT_ONLY_KTO_LINEARIZATION_REPAIR_FORMULATION_ONLY"
        or profile.get("decision", {}).get("inconclusive") != "STOP_AND_RESEARCH"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "additional_kto_solve",
                "r116_inverse_dynamics_formulation",
                "inverse_dynamics_solve",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != ("ruff_check", "ruff_format", "lab_full", "motor", "host_check")
    ):
        raise ValueError("post-R115 research profile differs")
