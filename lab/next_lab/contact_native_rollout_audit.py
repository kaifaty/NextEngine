from __future__ import annotations

import hashlib
import json
import math
from pathlib import Path
from typing import Any, Mapping, Sequence


AUDIT_ID = "nextengine.humanoid-contact-native-rollout-audit.v1"
CHECK_ID = "TRAIN-4-NATIVE-ROLLOUT-AUDIT"
SOURCE_ROLES = ("v7", "v9", "r100", "r101")


def build_native_rollout_audit(
    *,
    profile_path: Path,
    source_report_paths: Mapping[str, Path],
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Recompute the bounded R102 decision from immutable native traces."""

    profile_path = profile_path.resolve()
    tool_path = tool_path.resolve()
    if not profile_path.is_file() or not tool_path.is_file():
        raise FileNotFoundError("native-rollout audit input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    if set(source_report_paths) != set(SOURCE_ROLES):
        raise ValueError("native-rollout source roles are invalid")

    traces: dict[str, Mapping[str, Any]] = {}
    source_metrics: dict[str, dict[str, Any]] = {}
    source_identities: dict[str, dict[str, Any]] = {}
    for role in SOURCE_ROLES:
        source = _load_source(
            role=role,
            report_path=source_report_paths[role].resolve(),
            expected=profile["source"][role],
        )
        traces[role] = source["trace"]
        source_metrics[role] = native_trace_metrics(
            trace=source["trace"],
            phase_result=source["phase_result"],
            contact_pair_hard_limits=source["contact_pair_hard_limits"],
            measurement=profile["measurement"],
        )
        source_identities[role] = source["identities"]

    comparisons = native_trace_comparisons(
        traces=traces,
        measurement=profile["measurement"],
    )
    findings = native_rollout_findings(
        source_metrics=source_metrics,
        comparisons=comparisons,
    )
    observed = {
        "source_metrics": source_metrics,
        "comparisons": comparisons,
        "findings": findings,
    }
    _assert_expected(observed, profile["expected"], path="expected")

    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "audit_id": profile["audit_id"],
        "scope": profile["scope"],
        "method": {
            "measurement": profile["measurement"],
            "source_semantics": (
                "read-only recomputation from complete physical-substep "
                "traces; no simulator execution or trajectory mutation"
            ),
        },
        "identities": {
            "profile_sha256": sha256(profile_path),
            "tool_sha256": sha256(tool_path),
            "audit_module_sha256": sha256(Path(__file__).resolve()),
            "sources": source_identities,
        },
        "source_metrics": source_metrics,
        "comparisons": comparisons,
        "findings": findings,
        "future_candidate_contract": profile["future_candidate_contract"],
        "bounded_acceptance": {
            "candidate_status": "NOT_EVALUATED",
            "candidate_search": profile["decision"]["candidate_search"],
            "all_17": profile["decision"]["all_17"],
            "full_v19": profile["decision"]["full_v19"],
            "training": profile["decision"]["training"],
        },
        "physx_runs": 0,
        "candidate_evaluations": 0,
        "trajectory_mutations": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def native_trace_metrics(
    *,
    trace: Mapping[str, Any],
    phase_result: Mapping[str, Any],
    contact_pair_hard_limits: Sequence[int],
    measurement: Mapping[str, Any],
) -> dict[str, Any]:
    """Extract exact plant metrics used by the R102 stopping decision."""

    action_ids = tuple(str(value) for value in trace["action_channel_ids"])
    pair_ids = tuple(str(value) for value in trace["contact_pair_ids"])
    left_roll = _index(action_ids, measurement["left_roll_action_id"])
    right_pitch = _index(action_ids, measurement["right_pitch_action_id"])
    remote_pair = _index(pair_ids, measurement["remote_contact_pair_id"])
    samples = trace["samples"]
    _validate_trace_samples(
        trace, action_count=len(action_ids), pair_count=len(pair_ids)
    )

    comparison_tick = int(measurement["remote_comparison_motor_tick"])
    remote_episode = [
        int(row["episode_contact_pair_maximum_impulse_micronewton_seconds"][
            remote_pair
        ])
        for row in samples
    ]
    remote_tick = [
        int(row["motor_tick_contact_pair_maximum_impulse_micronewton_seconds"][
            remote_pair
        ])
        for row in samples
        if int(row["motor_tick"]) == comparison_tick
    ]
    first_hard = next(
        (
            {
                "motor_tick": int(row["motor_tick"]),
                "physics_substep": int(row["physics_substep"]),
                "episode_impulse_micronewton_seconds": int(
                    row[
                        "episode_contact_pair_maximum_impulse_"
                        "micronewton_seconds"
                    ][remote_pair]
                ),
            }
            for row in samples
            if bool(row["hard_impact_by_contact_pair"][remote_pair])
        ),
        None,
    )
    violation = phase_result.get("first_required_safety_violation")
    requested = [
        int(row["requested_effort_micronewton_metres"][right_pitch])
        for row in samples
    ]
    published = [
        int(row["published_effort_micronewton_metres"][right_pitch])
        for row in samples
    ]
    initial_effort = [
        int(value) for value in samples[0]["requested_effort_micronewton_metres"]
    ]
    return {
        "status": str(phase_result["status"]),
        "terminal_motor_tick": (
            None if violation is None else int(violation["terminal_motor_tick"])
        ),
        "terminal_reasons": (
            [] if violation is None else list(violation["reasons"])
        ),
        "sample_count": len(samples),
        "trace_sha256": str(trace["trace_sha256"]),
        "maximum_left_roll_speed_microradians_per_second": max(
            abs(int(row["pre_physics_joint_velocity_microradians_per_second"][
                left_roll
            ]))
            for row in samples
        ),
        "minimum_right_pitch_position_microradians": min(
            int(row["pre_physics_joint_position_microradians"][right_pitch])
            for row in samples
        ),
        "maximum_right_pitch_speed_microradians_per_second": max(
            abs(int(row["pre_physics_joint_velocity_microradians_per_second"][
                right_pitch
            ]))
            for row in samples
        ),
        "maximum_right_pitch_effort_debt_micronewton_metres": max(
            abs(target - actual)
            for target, actual in zip(requested, published, strict=True)
        ),
        "remote_contact_pair_id": pair_ids[remote_pair],
        "remote_contact_hard_limit_micronewton_seconds": int(
            contact_pair_hard_limits[remote_pair]
        ),
        "remote_episode_maximum_impulse_micronewton_seconds": max(
            remote_episode
        ),
        "remote_comparison_tick_maximum_impulse_micronewton_seconds": max(
            remote_tick, default=0
        ),
        "first_remote_hard_impact": first_hard,
        "initial_requested_effort_sha256": hashlib.sha256(
            canonical_json(initial_effort)
        ).hexdigest(),
    }


def native_trace_comparisons(
    *,
    traces: Mapping[str, Mapping[str, Any]],
    measurement: Mapping[str, Any],
) -> dict[str, Any]:
    if set(traces) != set(SOURCE_ROLES):
        raise ValueError("native trace comparison roles are invalid")
    action_ids = tuple(str(value) for value in traces["v7"]["action_channel_ids"])
    if any(
        tuple(str(value) for value in traces[role]["action_channel_ids"])
        != action_ids
        for role in SOURCE_ROLES
    ):
        raise ValueError("native trace action layouts disagree")
    pair_ids = tuple(str(value) for value in traces["v7"]["contact_pair_ids"])
    if any(
        tuple(str(value) for value in traces[role]["contact_pair_ids"])
        != pair_ids
        for role in SOURCE_ROLES
    ):
        raise ValueError("native trace contact layouts disagree")
    prefix = int(measurement["comparison_prefix_substeps"])
    left_roll = _index(action_ids, measurement["left_roll_action_id"])
    return {
        "r100_left_roll_to_v7_first_prefix": _velocity_rms(
            traces["r100"], traces["v7"], prefix, (left_roll,)
        ),
        "r100_left_roll_to_v9_first_prefix": _velocity_rms(
            traces["r100"], traces["v9"], prefix, (left_roll,)
        ),
        "r101_left_roll_to_v7_first_prefix": _velocity_rms(
            traces["r101"], traces["v7"], prefix, (left_roll,)
        ),
        "r101_left_roll_to_v9_first_prefix": _velocity_rms(
            traces["r101"], traces["v9"], prefix, (left_roll,)
        ),
        "r101_all_actions_to_v7_first_prefix": _velocity_rms(
            traces["r101"], traces["v7"], prefix, tuple(range(len(action_ids)))
        ),
        "r101_all_actions_to_v9_first_prefix": _velocity_rms(
            traces["r101"], traces["v9"], prefix, tuple(range(len(action_ids)))
        ),
        "r101_v9_applied_target_disagreement": _array_disagreement(
            traces["r101"],
            traces["v9"],
            sample_count=min(
                int(traces["r101"]["sample_count"]),
                int(traces["v9"]["sample_count"]),
            ),
            field="applied_target_microradians",
        ),
        "r101_v9_command_target_disagreement": _array_disagreement(
            traces["r101"],
            traces["v9"],
            sample_count=min(
                int(traces["r101"]["sample_count"]),
                int(traces["v9"]["sample_count"]),
            ),
            field="command_reference_target_microradians",
        ),
        "r101_v7_initial_requested_effort_disagreement": _array_disagreement(
            traces["r101"],
            traces["v7"],
            sample_count=1,
            field="requested_effort_micronewton_metres",
        ),
    }


def native_rollout_findings(
    *,
    source_metrics: Mapping[str, Mapping[str, Any]],
    comparisons: Mapping[str, Mapping[str, Any]],
) -> dict[str, Any]:
    r100 = source_metrics["r100"]
    r101 = source_metrics["r101"]
    v9 = source_metrics["v9"]
    return {
        "r101_initial_requested_effort_matches_v7": comparisons[
            "r101_v7_initial_requested_effort_disagreement"
        ]["element_count"]
        == 0,
        "r101_applied_targets_match_v9": comparisons[
            "r101_v9_applied_target_disagreement"
        ]["element_count"]
        == 0,
        "r101_local_phase_is_closer_to_v7": comparisons[
            "r101_left_roll_to_v7_first_prefix"
        ]["rounded_rms_microradians_per_second"]
        < comparisons["r101_left_roll_to_v9_first_prefix"][
            "rounded_rms_microradians_per_second"
        ],
        "r101_tick5_remote_impulse_improves_r100": r101[
            "remote_comparison_tick_maximum_impulse_micronewton_seconds"
        ]
        < r100["remote_comparison_tick_maximum_impulse_micronewton_seconds"],
        "r101_tick5_remote_impulse_regresses_v9": r101[
            "remote_comparison_tick_maximum_impulse_micronewton_seconds"
        ]
        > v9["remote_comparison_tick_maximum_impulse_micronewton_seconds"],
        "r101_required_safety_closed": r101["status"] == "PASS",
        "boundary_state_is_sufficient": r101["status"] == "PASS",
        "manual_boundary_substitution_disposition": "EXHAUSTED",
    }


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _load_source(
    *,
    role: str,
    report_path: Path,
    expected: Mapping[str, Any],
) -> dict[str, Any]:
    if not report_path.is_file():
        raise FileNotFoundError(report_path)
    report = json.loads(report_path.read_bytes())
    workers = report.get("fresh_scene", {}).get("workers", ())
    phase_results = report.get("fresh_scene", {}).get("results", {}).get(
        "phase_results", ()
    )
    if (
        report.get("check") != "TRAIN-4-ISAAC-NATIVE-DYNAMICS-TRACE"
        or report.get("status") != "COMPLETE"
        or report.get("claim") != "OptimizerFreeReportOnlyNativeDynamicsTrace"
        or report.get("gate_decision") != "STOP_AND_RESEARCH"
        or report.get("manifest_sha256") != expected["manifest_sha256"]
        or sha256(report_path) != expected["manifest_file_sha256"]
        or report.get("identities", {}).get("probe_profile_sha256")
        != expected["probe_profile_sha256"]
        or report.get("optimizer_steps") != 0
        or report.get("training_runs") != 0
        or report.get("indexed_partial_reset", {}).get("status") != "NOT_RUN"
        or report.get("repository", {}).get("dirty") is not False
        or len(workers) != 1
        or len(phase_results) != 1
    ):
        raise ValueError(f"native-rollout {role} source report is invalid")
    inventory = workers[0]
    relative = Path(str(inventory["report_relative_path"]))
    worker_path = (report_path.parent / relative).resolve()
    if report_path.parent.resolve() not in worker_path.parents:
        raise ValueError("native-rollout worker path escapes its report root")
    if (
        not worker_path.is_file()
        or sha256(worker_path) != expected["worker_report_sha256"]
        or inventory.get("report_sha256") != expected["worker_report_sha256"]
        or inventory.get("case_ordinal") != expected["worker_case_ordinal"]
    ):
        raise ValueError(f"native-rollout {role} worker identity is invalid")
    worker = json.loads(worker_path.read_bytes())
    trace = worker.get("native_dynamics_trace", {})
    summary = inventory.get("native_dynamics_trace", {})
    phase_result = phase_results[0]
    if (
        worker.get("phase_result") != phase_result
        or trace.get("comparison_role") != expected["comparison_role"]
        or trace.get("trace_sha256") != expected["trace_sha256"]
        or trace.get("sample_count") != expected["sample_count"]
        or trace.get("coverage_complete") is not True
        or trace.get("evidence_role") != "report-only"
        or trace.get("enabled") is not True
        or summary.get("trace_sha256") != trace.get("trace_sha256")
        or summary.get("sample_count") != trace.get("sample_count")
        or _trace_sha256(trace) != trace.get("trace_sha256")
        or worker.get("optimizer_steps") != 0
        or worker.get("training_runs") != 0
    ):
        raise ValueError(f"native-rollout {role} trace is invalid")
    hard_limits = worker.get("contact_pair_hard_limits_micronewton_seconds", ())
    if len(hard_limits) != len(trace.get("contact_pair_ids", ())):
        raise ValueError("native-rollout contact limit layout is invalid")
    return {
        "trace": trace,
        "phase_result": phase_result,
        "contact_pair_hard_limits": hard_limits,
        "identities": {
            "manifest_sha256": report["manifest_sha256"],
            "manifest_file_sha256": sha256(report_path),
            "probe_profile_sha256": report["identities"][
                "probe_profile_sha256"
            ],
            "worker_report_sha256": sha256(worker_path),
            "trace_sha256": trace["trace_sha256"],
            "repository_commit": report["repository"]["commit"],
        },
    }


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    contract = profile.get("future_candidate_contract", {})
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("audit_id") != AUDIT_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim") != "OptimizerFreeReportOnlyNativeRolloutAudit"
        or set(source) != set(SOURCE_ROLES)
        or scope.get("input_roles") != list(SOURCE_ROLES)
        or scope.get("case_count") != 1
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("start_frame") != 238
        or scope.get("source_case_ordinal") != 7967
        or contract.get("status") != "FORMULATION_ONLY"
        or contract.get("candidate_search") != "NOT_AUTHORIZED"
        or contract.get("required_safety_priority")
        != "LEXICOGRAPHIC_REJECT_BEFORE_TRACKING"
        or contract.get("fresh_scene_required") is not True
        or contract.get("physical_substep_trace_required") is not True
        or contract.get("frame_0_state_mutable") is not False
        or contract.get("controller_semantics_mutable") is not False
        or contract.get("safety_limits_mutable") is not False
        or contract.get("reset_semantics_mutable") is not False
        or decision
        != {
            "complete": "STOP_AND_RESEARCH",
            "candidate_search": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        }
        or not isinstance(profile.get("expected"), Mapping)
    ):
        raise ValueError("native-rollout audit profile is invalid")
    for role, row in source.items():
        if (
            row.get("role") != role
            or not _sha256_shape(row.get("manifest_sha256"))
            or not _sha256_shape(row.get("manifest_file_sha256"))
            or not _sha256_shape(row.get("probe_profile_sha256"))
            or not _sha256_shape(row.get("worker_report_sha256"))
            or not _sha256_shape(row.get("trace_sha256"))
            or not isinstance(row.get("worker_case_ordinal"), int)
            or not isinstance(row.get("sample_count"), int)
            or row["sample_count"] <= 0
            or not row.get("comparison_role")
        ):
            raise ValueError(f"native-rollout {role} profile source is invalid")
    measurement = profile.get("measurement", {})
    if (
        measurement.get("comparison_prefix_substeps") != 20
        or measurement.get("remote_comparison_motor_tick") != 5
        or measurement.get("left_roll_action_id")
        != "actuator.left-ankle-roll"
        or measurement.get("right_pitch_action_id")
        != "actuator.right-ankle-pitch"
        or measurement.get("remote_contact_pair_id")
        != "ground:body.right-ankle-roll"
    ):
        raise ValueError("native-rollout audit measurement is invalid")


def _validate_trace_samples(
    trace: Mapping[str, Any], *, action_count: int, pair_count: int
) -> None:
    samples = trace.get("samples", ())
    expected = int(trace.get("expected_sample_count", -1))
    substeps = int(trace.get("physics_substeps_per_motor_tick", -1))
    if (
        len(samples) != expected
        or len(samples) != trace.get("sample_count")
        or expected
        != int(trace.get("observed_motor_ticks", -1)) * substeps
        or substeps <= 0
    ):
        raise ValueError("native-rollout trace coverage is invalid")
    action_fields = (
        "applied_target_microradians",
        "command_reference_target_microradians",
        "pre_physics_joint_position_microradians",
        "pre_physics_joint_velocity_microradians_per_second",
        "published_effort_micronewton_metres",
        "requested_effort_micronewton_metres",
    )
    pair_fields = (
        "episode_contact_pair_maximum_impulse_micronewton_seconds",
        "motor_tick_contact_pair_maximum_impulse_micronewton_seconds",
        "hard_impact_by_contact_pair",
    )
    for index, row in enumerate(samples):
        expected_tick = index // substeps + 1
        expected_substep = index % substeps
        if (
            row.get("motor_tick") != expected_tick
            or row.get("physics_substep") != expected_substep
            or any(len(row.get(field, ())) != action_count for field in action_fields)
            or any(len(row.get(field, ())) != pair_count for field in pair_fields)
        ):
            raise ValueError("native-rollout trace sample is invalid")


def _velocity_rms(
    first: Mapping[str, Any],
    second: Mapping[str, Any],
    sample_count: int,
    channels: Sequence[int],
) -> dict[str, int]:
    first_samples = first["samples"]
    second_samples = second["samples"]
    if sample_count <= 0 or min(len(first_samples), len(second_samples)) < sample_count:
        raise ValueError("native-rollout RMS prefix is invalid")
    differences = [
        int(left["pre_physics_joint_velocity_microradians_per_second"][channel])
        - int(right["pre_physics_joint_velocity_microradians_per_second"][channel])
        for left, right in zip(
            first_samples[:sample_count], second_samples[:sample_count], strict=True
        )
        for channel in channels
    ]
    squared = sum(value * value for value in differences)
    return {
        "sample_count": sample_count,
        "component_count": len(differences),
        "sum_squared_error": squared,
        "rounded_rms_microradians_per_second": _rounded_rms(
            squared, len(differences)
        ),
    }


def _array_disagreement(
    first: Mapping[str, Any],
    second: Mapping[str, Any],
    *,
    sample_count: int,
    field: str,
) -> dict[str, int]:
    count = 0
    maximum = 0
    for left, right in zip(
        first["samples"][:sample_count],
        second["samples"][:sample_count],
        strict=True,
    ):
        if len(left[field]) != len(right[field]):
            raise ValueError("native-rollout comparison layout is invalid")
        for left_value, right_value in zip(left[field], right[field], strict=True):
            delta = abs(int(left_value) - int(right_value))
            count += int(delta != 0)
            maximum = max(maximum, delta)
    return {
        "sample_count": sample_count,
        "element_count": count,
        "maximum_absolute_delta": maximum,
    }


def _rounded_rms(sum_squared_error: int, count: int) -> int:
    if sum_squared_error < 0 or count <= 0:
        raise ValueError("native-rollout RMS input is invalid")
    lower = math.isqrt(sum_squared_error // count)
    midpoint_four = count * (4 * lower * lower + 4 * lower + 1)
    return lower + int(4 * sum_squared_error >= midpoint_four)


def _trace_sha256(trace: Mapping[str, Any]) -> str:
    payload = dict(trace)
    payload.pop("trace_sha256", None)
    return hashlib.sha256(canonical_json(payload)).hexdigest()


def _assert_expected(actual: Any, expected: Any, *, path: str) -> None:
    if isinstance(expected, Mapping):
        if not isinstance(actual, Mapping):
            raise ValueError(f"{path} has the wrong type")
        for key, value in expected.items():
            if key not in actual:
                raise ValueError(f"{path}.{key} is absent")
            _assert_expected(actual[key], value, path=f"{path}.{key}")
        return
    if actual != expected:
        raise ValueError(f"{path} disagrees: {actual!r} != {expected!r}")


def _index(values: Sequence[str], selected: Any) -> int:
    try:
        return values.index(str(selected))
    except ValueError as error:
        raise ValueError(f"native-rollout identity is absent: {selected}") from error


def _sha256_shape(value: Any) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )
