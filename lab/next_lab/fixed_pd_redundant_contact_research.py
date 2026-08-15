from __future__ import annotations

import hashlib
import json
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

RESEARCH_ID = "nextengine.humanoid-post-r123-redundant-contact-research.v1"
CHECK_ID = "TRAIN-4-POST-R123-REDUNDANT-CONTACT-RESEARCH"


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_redundant_contact_research(
    *,
    profile_path: Path,
    r123_report_path: Path,
    r123_profile_path: Path,
    r123_execution_module_path: Path,
    r123_tool_path: Path,
    r113_report_path: Path,
    r113_profile_path: Path,
    validation_results: Sequence[Mapping[str, str]],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Hash-close the R123 geometry failure without rebuilding a local system."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r123_report_path,
            r123_profile_path,
            r123_execution_module_path,
            r123_tool_path,
            r113_report_path,
            r113_profile_path,
            tool_path,
        )
    )
    (
        profile_path,
        r123_report_path,
        r123_profile_path,
        r123_execution_module_path,
        r123_tool_path,
        r113_report_path,
        r113_profile_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("post-R123 research input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_repository(repository)
    r123 = validate_r123(
        profile=profile,
        report_path=r123_report_path,
        execution_profile_path=r123_profile_path,
        execution_module_path=r123_execution_module_path,
        execution_tool_path=r123_tool_path,
    )
    r113 = validate_r113(
        profile=profile,
        report_path=r113_report_path,
        identity_profile_path=r113_profile_path,
    )
    validations = _validate_results(profile, validation_results)

    first = r123["solver_result"]["collocations"][0]
    points = r113["point_contact_force_identity"]["application_point_identity"]
    geometry = analyze_redundant_pair(
        points=points,
        active_point_ordinals=tuple(first["active_point_ordinals"]),
    )
    maximum_condition = float(
        profile["discriminators"]["maximum_scaled_condition_number"]
    )
    observed_condition = float(first["singular_value_condition_number"])
    discriminators = {
        "r123_is_clean_single_invalid_execution": True,
        "first_failure_precedes_every_local_solve": (
            first["collocation"] == 0
            and r123["solver_result"]["local_system_solves"] == 0
        ),
        "first_failure_is_ill_conditioned_flat_foot": (
            first["invalid_reason"] == "SCALED_LOCAL_SYSTEM_ILL_CONDITIONED"
            and first["contact_modes"] == [0, 3]
            and first["active_point_ordinals"] == [2, 3]
            and observed_condition > maximum_condition
        ),
        "active_points_share_one_rigid_body": geometry["same_body"],
        "active_points_are_distinct": geometry["separation_is_nonzero"],
        "nonzero_internal_force_has_zero_resultant_force": geometry[
            "resultant_force_is_exact_zero"
        ],
        "nonzero_internal_force_has_zero_resultant_moment": geometry[
            "resultant_moment_is_exact_zero"
        ],
        "two_point_sticking_rank_is_at_most_five": (
            geometry["constraint_rank_upper_bound"] == 5
        ),
        "research_reconstructs_or_solves_no_local_system": True,
    }
    confirmed = all(discriminators.values())
    finding = (
        "CONFIRMED_REDUNDANT_FLAT_FOOT_FORCE_GAUGE"
        if confirmed
        else "INCONCLUSIVE_RETAIN_STOP_WITHOUT_EXECUTION"
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
        "r123_result": {
            "status": r123["status"],
            "gate_decision": r123["gate_decision"],
            "report_sha256": r123["report_sha256"],
            "invalid_reason": r123["solver_result"]["invalid_reason"],
            "first_collocation": first,
            "singular_value_decompositions": r123["solver_result"][
                "singular_value_decompositions"
            ],
            "local_system_solves": r123["solver_result"]["local_system_solves"],
        },
        "exact_geometry_audit": geometry,
        "condition_audit": {
            "maximum_scaled_condition_number": maximum_condition,
            "observed_scaled_condition_number": observed_condition,
            "observed_to_maximum_ratio": observed_condition / maximum_condition,
            "interpretation": "floating-point manifestation of an exact same-body two-point force gauge; scaling cannot restore rank",
        },
        "discriminators": discriminators,
        "hypothesis_disposition": profile["hypothesis_disposition"],
        "research_basis": profile["research_basis"],
        "repair_alternatives": profile["repair_alternatives"],
        "interpretation": {
            "supported": "the R121/R123 square equality system assumes six independent same-body flat-foot point constraints and a unique six-component force split, but the exact map has a nonzero internal-force null direction",
            "not_supported": "R123 provides no evidence that the fixed-PD schedule is dynamically feasible or infeasible because no local solve or friction-cone classification occurred",
            "required_successor_scope": "preserve both point locations, rigid sticking and individual cones while testing existence over the force gauge; any deterministic witness selection is secondary to feasibility",
        },
        "next_smallest_action": profile["next_smallest_action"],
        "validation_results": validations,
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r123_report_file_sha256": sha256(r123_report_path),
            "r123_execution_profile_sha256": sha256(r123_profile_path),
            "r123_execution_module_sha256": sha256(r123_execution_module_path),
            "r123_execution_tool_sha256": sha256(r123_tool_path),
            "r113_report_file_sha256": sha256(r113_report_path),
            "r113_profile_sha256": sha256(r113_profile_path),
            "research_module_sha256": sha256(Path(__file__).resolve()),
            "tool_sha256": sha256(tool_path),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "research_audits": 1,
        "local_system_reconstructions": 0,
        "singular_value_decompositions": 0,
        "local_system_solves": 0,
        "solver_runs": 0,
        "additional_inverse_dynamics_execution_runs": 0,
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "candidate_artifacts_built": 0,
        "solver_private_caches_built": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def validate_r123(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    execution_profile_path: Path,
    execution_module_path: Path,
    execution_tool_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r123"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(execution_profile_path) != expected["execution_profile_sha256"]
        or sha256(execution_module_path) != expected["execution_module_sha256"]
        or sha256(execution_tool_path) != expected["tool_sha256"]
    ):
        raise ValueError("post-R123 source file identity differs")
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R123")
    solver = report.get("solver_result", {})
    rows = solver.get("collocations", [])
    first = rows[0] if len(rows) == 1 else {}
    identities = report.get("identities", {})
    if (
        report.get("status") != "INVALID"
        or report.get("gate_decision") != "STOP_INVALID_EVIDENCE_WITHOUT_RESTART"
        or report.get("result_transition") != "STOP_R123_WITHOUT_RETRY"
        or report.get("inverse_dynamics_execution_runs") != 1
        or report.get("solver_runs") != 1
        or report.get("local_system_solves") != 0
        or solver.get("status") != "INVALID"
        or solver.get("feasibility") is not None
        or solver.get("invalid_reason") != "SCALED_LOCAL_SYSTEM_ILL_CONDITIONED"
        or solver.get("local_system_solves") != 0
        or solver.get("singular_value_decompositions") != 1
        or first.get("collocation") != 0
        or first.get("interval") != 0
        or first.get("substep") != 0
        or first.get("contact_modes") != [0, 3]
        or first.get("active_point_ordinals") != [2, 3]
        or first.get("invalid_reason") != "SCALED_LOCAL_SYSTEM_ILL_CONDITIONED"
        or not isinstance(first.get("singular_value_condition_number"), float)
        or first["singular_value_condition_number"] <= 1.0e12
        or report.get("solver_private_cache", {}).get("status")
        != "NOT_EMITTED_INVALID_EXECUTION"
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "kinodynamic_solves",
                "candidate_artifacts_built",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or identities.get("profile_sha256") != expected["execution_profile_sha256"]
        or identities.get("execution_module_sha256")
        != expected["execution_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or report.get("bounded_acceptance", {}).get("r123_retry") != "NOT_AUTHORIZED"
        or report.get("bounded_acceptance", {}).get("r124_formulation")
        != "AUTHORIZED_REPORT_ONLY_ON_VALID_R123_COMPLETION"
        or any(row.get("status") != "PASS" for row in report["validation_results"])
    ):
        raise ValueError("post-R123 source report contract differs")
    return report


def validate_r113(
    *,
    profile: Mapping[str, Any],
    report_path: Path,
    identity_profile_path: Path,
) -> dict[str, Any]:
    expected = profile["source"]["r113"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(identity_profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("post-R123 R113 source file identity differs")
    report = json.loads(report_path.read_bytes())
    _validate_canonical_report(report, expected["report_sha256"], "R113")
    point_identity = report.get("point_contact_force_identity", {})
    points = point_identity.get("application_point_identity", [])
    by_ordinal = {
        point.get("point_ordinal"): point
        for point in points
        if isinstance(point, Mapping)
    }
    expected_pair = profile["discriminators"]["expected_active_pair"]
    if (
        report.get("status") != "PASS"
        or point_identity.get("model_identity_satisfied") is not True
        or len(points) != 4
        or any(
            by_ordinal.get(row["point_ordinal"], {}).get("body_id") != row["body_id"]
            or by_ordinal[row["point_ordinal"]].get("local_translation_micrometres")
            != row["local_translation_micrometres"]
            for row in expected_pair
        )
    ):
        raise ValueError("post-R123 R113 point identity differs")
    return report


def analyze_redundant_pair(
    *,
    points: Sequence[Mapping[str, Any]],
    active_point_ordinals: Sequence[int],
) -> dict[str, Any]:
    if len(active_point_ordinals) != 2 or len(set(active_point_ordinals)) != 2:
        raise ValueError("redundant-contact audit requires two distinct ordinals")
    by_ordinal = {int(point["point_ordinal"]): point for point in points}
    try:
        heel = by_ordinal[int(active_point_ordinals[0])]
        forefoot = by_ordinal[int(active_point_ordinals[1])]
    except KeyError as error:
        raise ValueError("redundant-contact audit point is absent") from error
    heel_position = _integer_vector(heel["local_translation_micrometres"])
    forefoot_position = _integer_vector(forefoot["local_translation_micrometres"])
    separation = _subtract(forefoot_position, heel_position)
    opposite = tuple(-value for value in separation)
    resultant_force = _add(separation, opposite)
    heel_moment = _cross(heel_position, separation)
    forefoot_moment = _cross(forefoot_position, opposite)
    resultant_moment = _add(heel_moment, forefoot_moment)
    separation_squared = sum(value * value for value in separation)
    same_body = heel.get("body_id") == forefoot.get("body_id")
    exact_gauge = (
        same_body
        and separation_squared > 0
        and resultant_force == (0, 0, 0)
        and resultant_moment == (0, 0, 0)
    )
    return {
        "active_point_ordinals": [int(value) for value in active_point_ordinals],
        "first_point": {
            "body_id": heel.get("body_id"),
            "effector_id": heel.get("effector_id"),
            "local_translation_micrometres": list(heel_position),
        },
        "second_point": {
            "body_id": forefoot.get("body_id"),
            "effector_id": forefoot.get("effector_id"),
            "local_translation_micrometres": list(forefoot_position),
        },
        "same_body": same_body,
        "separation_micrometres": list(separation),
        "separation_squared_micrometres_squared": separation_squared,
        "separation_is_nonzero": separation_squared > 0,
        "internal_force_pair_in_common_body_frame": {
            "first_force_proportional_to": list(separation),
            "second_force_proportional_to": list(opposite),
            "resultant_force": list(resultant_force),
            "first_moment": list(heel_moment),
            "second_moment": list(forefoot_moment),
            "resultant_moment": list(resultant_moment),
            "common_world_rotation_preserves_zero_wrench": True,
        },
        "resultant_force_is_exact_zero": resultant_force == (0, 0, 0),
        "resultant_moment_is_exact_zero": resultant_moment == (0, 0, 0),
        "point_force_multiplier_dimension": 6,
        "minimum_force_nullity": 1 if exact_gauge else 0,
        "constraint_rank_upper_bound": 5 if exact_gauge else None,
        "exact_internal_force_gauge_confirmed": exact_gauge,
    }


def _integer_vector(value: Any) -> tuple[int, int, int]:
    if (
        not isinstance(value, list)
        or len(value) != 3
        or any(not isinstance(component, int) for component in value)
    ):
        raise ValueError("point translation must be an integer 3-vector")
    return (value[0], value[1], value[2])


def _add(
    left: tuple[int, int, int], right: tuple[int, int, int]
) -> tuple[int, int, int]:
    return tuple(a + b for a, b in zip(left, right, strict=True))  # type: ignore[return-value]


def _subtract(
    left: tuple[int, int, int], right: tuple[int, int, int]
) -> tuple[int, int, int]:
    return tuple(a - b for a, b in zip(left, right, strict=True))  # type: ignore[return-value]


def _cross(
    left: tuple[int, int, int], right: tuple[int, int, int]
) -> tuple[int, int, int]:
    return (
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    )


def _validate_canonical_report(
    report: Mapping[str, Any], expected_sha256: str, source_name: str
) -> None:
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected_sha256
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
    ):
        raise ValueError(f"post-R123 {source_name} canonical identity differs")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("post-R123 research validation differs")
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
        raise ValueError("post-R123 research requires a clean repository")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    method = profile.get("method", {})
    discriminators = profile.get("discriminators", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("research_id") != RESEARCH_ID
        or profile.get("status") != "FrozenReportOnly"
        or profile.get("claim") != "PostR123RedundantContactGeometryAuditOnly"
        or scope.get("research_cycle_id") != "R123-RC1"
        or scope.get("source_r123_execution_runs") != 1
        or scope.get("research_audits") != 1
        or scope.get("local_system_reconstructions") != 0
        or scope.get("local_system_solves") != 0
        or method.get("solver_or_factorization_policy") != "FORBIDDEN"
        or discriminators.get("maximum_scaled_condition_number") != 1.0e12
        or [
            row.get("point_ordinal")
            for row in discriminators.get("expected_active_pair", [])
        ]
        != [2, 3]
        or profile.get("decision", {}).get("confirmed")
        != "PERMIT_SEPARATE_REPORT_ONLY_R125_REDUNDANT_CONTACT_FEASIBILITY_FORMULATION_ONLY"
        or profile.get("decision", {}).get("inconclusive")
        != "STOP_AND_RESEARCH_WITHOUT_EXECUTION"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "r123_retry",
                "r124_formulation",
                "r125_execution",
                "additional_kto_solve",
                "additional_inverse_dynamics_solve",
                "kinodynamic_solve",
                "candidate_artifact",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != (
            "ruff_check",
            "ruff_format",
            "solver_free_import",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("post-R123 research profile differs")
