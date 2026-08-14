from __future__ import annotations

import hashlib
import json
import math
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping, Sequence

import numpy as np
from numpy.typing import NDArray


REQUIRED_ARTIFACT_ARRAYS = (
    "center_of_mass_um",
    "contacts",
    "effector_position_um",
    "joint_position_urad",
    "joint_velocity_urad_s",
    "phase_u16",
    "reference_frame",
    "root_linear_velocity_um_s",
    "root_position_um",
    "root_quaternion_q1_30",
    "root_yaw_velocity_urad_s",
)


@dataclass(frozen=True)
class ContactPrototypeCase:
    ordinal: int
    source_case_ordinal: int
    clip_id: str
    split: str
    frame_first: int
    frame_last: int
    baseline_status: str
    baseline_reasons: tuple[str, ...]
    target_failure_categories: tuple[str, ...]
    artifact_path: Path
    artifact_sha256: str

    @property
    def horizon_motor_ticks(self) -> int:
        return self.frame_last - self.frame_first


@dataclass(frozen=True)
class AuthoredRootState:
    position_m: NDArray[np.float64]
    quaternion_wxyz: NDArray[np.float64]
    linear_velocity_world_m_s: NDArray[np.float64]
    angular_velocity_world_rad_s: NDArray[np.float64]
    linear_velocity_local_m_s: NDArray[np.float64]
    angular_velocity_local_degrees_s: NDArray[np.float64]


def complete_clip_probe_shape_is_valid(
    *,
    profile: Mapping[str, Any],
    prototype_manifest: Mapping[str, Any],
    cases: Sequence[ContactPrototypeCase],
) -> bool:
    """Validate the V9 complete-clip evidence required by the fresh probe."""

    scope = prototype_manifest.get("scope", {})
    complete_clips = prototype_manifest.get("complete_clips", ())
    case_records = prototype_manifest.get("cases", ())
    exact_slice_identity = prototype_manifest.get("exact_slice_identity", {})
    trajectory_closure = scope.get("projection", {}).get(
        "trajectory_closure", {}
    )
    partial = profile.get("execution", {}).get("indexed_partial_reset", {})
    acceptance = profile.get("bounded_acceptance", {})
    return (
        prototype_manifest.get("prototype_id")
        == "nextengine.humanoid-contact-manifold-prototype.v9"
        and scope.get("case_scope") == "all"
        and len(cases) == 17
        and len(case_records) == len(cases)
        and scope.get("failure_case_count") == 7
        and scope.get("control_case_count") == 10
        and trajectory_closure.get("algorithm_id")
        == "nextengine.dimensionless-contact-trajectory-qp.v9"
        and len(complete_clips) == 3
        and all(
            clip.get("solve_count") == 1
            and clip.get("projection_diagnostics", {}).get("status") == "PASS"
            and clip.get("projection_diagnostics", {}).get(
                "contact_point_deletion_count"
            )
            == 0
            for clip in complete_clips
        )
        and all(
            case.get("exact_complete_clip_slice_status") == "PASS"
            for case in case_records
        )
        and exact_slice_identity.get("status") == "PASS"
        and exact_slice_identity.get("disagreement_count") == 0
        and partial.get("enabled") is False
        and partial.get("evidence_role") == "report-only"
        and acceptance.get("acceptance_authority") == "fresh-scene"
        and acceptance.get("pass_gate_decision")
        == "PERMIT_FULL_V19_DATA_BUILD_ONLY"
        and acceptance.get("fail_gate_decision") == "STOP_AND_RESEARCH"
    )


def load_contact_prototype_cases(
    *,
    manifest_path: Path,
    source_audit_path: Path,
) -> tuple[dict[str, Any], dict[str, Any], tuple[ContactPrototypeCase, ...]]:
    """Load and hash-close the selected prototype cases against the source audit."""

    manifest_path = manifest_path.resolve()
    source_audit_path = source_audit_path.resolve()
    manifest = json.loads(manifest_path.read_bytes())
    source_audit = json.loads(source_audit_path.read_bytes())
    embedded_hash = manifest.get("manifest_sha256")
    without_hash = dict(manifest)
    without_hash.pop("manifest_sha256", None)
    if (
        manifest.get("check") != "TRAIN-4-CONTACT-MANIFOLD-PROTOTYPE-BUILD"
        or manifest.get("status") != "PASS"
        or manifest.get("optimizer_steps") != 0
        or manifest.get("training_runs") != 0
        or manifest.get("learned_policy_claim") is not False
        or not isinstance(embedded_hash, str)
        or hashlib.sha256(_canonical_json(without_hash)).hexdigest() != embedded_hash
        or manifest.get("identities", {}).get("source_audit_sha256")
        != _sha256(source_audit_path)
        or source_audit.get("check")
        != "TRAIN-4-ISAAC-EXHAUSTIVE-DYNAMIC-REFERENCE-FEASIBILITY"
    ):
        raise ValueError("contact prototype lineage is invalid")
    source_rows = source_audit.get("results", {}).get("phase_results", ())
    source_by_ordinal = {
        int(row["case_ordinal"]): row for row in source_rows
    }
    records = manifest.get("cases")
    if not isinstance(records, list) or not records:
        raise ValueError("contact prototype case inventory is empty")
    cases: list[ContactPrototypeCase] = []
    for ordinal, record in enumerate(records):
        source_ordinal = int(record["source_case_ordinal"])
        source = source_by_ordinal.get(source_ordinal)
        artifact = record.get("artifact", {})
        relative_path = artifact.get("relative_path")
        artifact_path = (
            manifest_path.parent / relative_path
            if isinstance(relative_path, str)
            else Path()
        )
        reasons = tuple(
            sorted(
                (source.get("first_required_safety_violation") or {}).get(
                    "reasons", ()
                )
            )
        ) if source is not None else ()
        if (
            source is None
            or source["clip_id"] != record.get("clip_id")
            or source["split"] != record.get("split")
            or int(source["start_frame"]) != int(record.get("frame_first", -1))
            or int(source["terminal_frame"]) != int(record.get("frame_last", -1))
            or source["status"] != record.get("baseline_status")
            or reasons != tuple(record.get("baseline_reasons", ()))
            or not artifact_path.is_file()
            or _sha256(artifact_path) != artifact.get("sha256")
            or artifact_path.stat().st_size != int(artifact.get("bytes", -1))
        ):
            raise ValueError(f"contact prototype case {ordinal} does not close")
        with np.load(artifact_path, allow_pickle=False) as arrays:
            missing = set(REQUIRED_ARTIFACT_ARRAYS) - set(arrays.files)
            expected_frames = int(record["frame_last"]) - int(
                record["frame_first"]
            ) + 1
            if (
                missing
                or any(len(arrays[name]) != expected_frames for name in REQUIRED_ARTIFACT_ARRAYS)
                or not np.array_equal(
                    arrays["reference_frame"],
                    np.arange(
                        int(record["frame_first"]),
                        int(record["frame_last"]) + 1,
                        dtype=np.int64,
                    ),
                )
            ):
                raise ValueError(f"contact prototype artifact {ordinal} is invalid")
        cases.append(
            ContactPrototypeCase(
                ordinal=ordinal,
                source_case_ordinal=source_ordinal,
                clip_id=str(record["clip_id"]),
                split=str(record["split"]),
                frame_first=int(record["frame_first"]),
                frame_last=int(record["frame_last"]),
                baseline_status=str(record["baseline_status"]),
                baseline_reasons=reasons,
                target_failure_categories=tuple(
                    record.get("target_failure_categories", ())
                ),
                artifact_path=artifact_path.resolve(),
                artifact_sha256=str(artifact["sha256"]),
            )
        )
    scope = manifest.get("scope", {})
    if (
        len(cases) != int(scope.get("case_count", -1))
        or tuple(case.ordinal for case in cases) != tuple(range(len(cases)))
        or len({case.clip_id for case in cases})
        != len(scope.get("clip_ids", ()))
        or len({case.horizon_motor_ticks for case in cases}) != 1
    ):
        raise ValueError("contact prototype scope differs from its cases")
    return manifest, source_audit, tuple(cases)


def load_case_arrays(case: ContactPrototypeCase) -> dict[str, NDArray[Any]]:
    with np.load(case.artifact_path, allow_pickle=False) as source:
        return {
            name: np.array(source[name], copy=True)
            for name in source.files
            if name != "metadata_json_utf8"
        }


def overlay_bounded_reference_window(
    *,
    source_values: Any,
    projected_by_environment: Any,
    selected_environment_ids: Any,
    selected_frames: Any,
    episode_start_frames: Any,
    where: Any,
) -> Any:
    """Overlay a bounded projection without inventing lookahead frames.

    Contact-manifold artifacts contain only the measured episode interval,
    while policy observations can request later reference frames. Values in
    the interval come from the projection; values outside it retain the
    hash-closed source-corpus reference supplied by ``source_values``.

    The operation is array-library agnostic so its indexed semantics can be
    covered with NumPy and executed on resident Torch tensors.
    """

    window_length = int(projected_by_environment.shape[1])
    if window_length <= 0:
        raise ValueError("projected reference window is empty")
    relative = selected_frames - episode_start_frames
    bounded_relative = relative.clip(0, window_length - 1)
    projected_values = projected_by_environment[
        selected_environment_ids, bounded_relative
    ]
    if source_values.shape != projected_values.shape:
        raise ValueError("source and projected reference values differ in shape")
    mask_shape = tuple(relative.shape) + (1,) * (
        projected_values.ndim - relative.ndim
    )
    valid = ((relative >= 0) & (relative < window_length)).reshape(mask_shape)
    return where(valid, projected_values, source_values)


def minimum_normalized_quaternion_dot_q1_30(
    actual_wxyz: NDArray[np.floating[Any]],
    expected_wxyz: NDArray[np.floating[Any]],
) -> int:
    """Return the bounded absolute-dot metric used by the reference tracker."""

    actual = np.asarray(actual_wxyz, dtype=np.float64)
    expected = np.asarray(expected_wxyz, dtype=np.float64)
    if actual.shape != expected.shape or actual.ndim != 2 or actual.shape[1] != 4:
        raise ValueError("quaternion comparison shape mismatch")
    actual_norm = np.linalg.norm(actual, axis=1, keepdims=True)
    expected_norm = np.linalg.norm(expected, axis=1, keepdims=True)
    if (
        not np.all(np.isfinite(actual))
        or not np.all(np.isfinite(expected))
        or np.any(actual_norm <= 1.0e-12)
        or np.any(expected_norm <= 1.0e-12)
    ):
        raise ValueError("quaternion comparison contains an invalid value")
    dot = np.abs(
        np.sum((actual / actual_norm) * (expected / expected_norm), axis=1)
    )
    return int(np.floor(np.min(np.clip(dot, 0.0, 1.0)) * (1 << 30)))


def authored_root_state(arrays: Mapping[str, NDArray[Any]]) -> AuthoredRootState:
    position = engine_to_isaac_vector(
        np.asarray(arrays["root_position_um"][0], dtype=np.float64)
        / 1_000_000.0
    )
    quaternion = engine_xyzw_to_isaac_wxyz(
        np.asarray(arrays["root_quaternion_q1_30"][0], dtype=np.float64)
        / float(1 << 30)
    )
    quaternion /= np.linalg.norm(quaternion)
    linear_world = engine_to_isaac_vector(
        np.asarray(arrays["root_linear_velocity_um_s"][0], dtype=np.float64)
        / 1_000_000.0
    )
    yaw = float(arrays["root_yaw_velocity_urad_s"][0]) / 1_000_000.0
    angular_world = engine_to_isaac_vector(
        np.asarray((0.0, yaw, 0.0), dtype=np.float64)
    )
    linear_local = rotate_world_to_local_wxyz(linear_world, quaternion)
    angular_local = rotate_world_to_local_wxyz(angular_world, quaternion)
    return AuthoredRootState(
        position_m=position,
        quaternion_wxyz=quaternion,
        linear_velocity_world_m_s=linear_world,
        angular_velocity_world_rad_s=angular_world,
        linear_velocity_local_m_s=linear_local,
        angular_velocity_local_degrees_s=np.rad2deg(angular_local),
    )


def build_fresh_scene_usda(
    *,
    base_usd_path: Path,
    descriptor: Mapping[str, Any],
    arrays: Mapping[str, NDArray[Any]],
    source_case_ordinal: int,
    artifact_sha256: str,
) -> str:
    """Compose an articulation state before PhysX scene creation.

    JointStateAPI values and rigid-body angular velocity are authored in
    degrees.  The floating articulation's rigid-body velocity is authored as
    center-of-mass velocity in the world frame, as required by PhysX.
    """

    base = base_usd_path.resolve()
    if not base.is_file() or not re.fullmatch(r"[0-9a-f]{64}", artifact_sha256):
        raise ValueError("fresh-scene USD source identity is invalid")
    joints = sorted(descriptor.get("joints", ()), key=lambda item: int(item["dof_ordinal"]))
    joint_position = np.asarray(arrays["joint_position_urad"][0], dtype=np.float64)
    joint_velocity = np.asarray(
        arrays["joint_velocity_urad_s"][0], dtype=np.float64
    )
    if (
        len(joints) != len(joint_position)
        or joint_velocity.shape != joint_position.shape
        or tuple(int(joint["dof_ordinal"]) for joint in joints)
        != tuple(range(len(joints)))
    ):
        raise ValueError("fresh-scene joint order does not close")
    root = authored_root_state(arrays)
    root_body = next(
        (
            body
            for body in descriptor.get("bodies", ())
            if body.get("parent_body_slot") is None
        ),
        None,
    )
    if root_body is None:
        raise ValueError("fresh-scene articulation root body is absent")
    center_of_mass_local = engine_to_isaac_vector(
        np.asarray(
            root_body["center_of_mass_micrometres"], dtype=np.float64
        )
        / 1_000_000.0
    )
    center_of_mass_world_offset = rotate_local_to_world_wxyz(
        center_of_mass_local, root.quaternion_wxyz
    )
    center_of_mass_velocity_world = (
        root.linear_velocity_world_m_s
        + np.cross(
            root.angular_velocity_world_rad_s,
            center_of_mass_world_offset,
        )
    )
    position = _tuple(root.position_m, precision=17)
    quaternion = _tuple(root.quaternion_wxyz, precision=9)
    linear_com_world = _tuple(center_of_mass_velocity_world, precision=9)
    angular_world = _tuple(
        np.rad2deg(root.angular_velocity_world_rad_s), precision=9
    )
    joint_blocks = []
    for joint, position_urad, velocity_urad_s in zip(
        joints, joint_position, joint_velocity, strict=True
    ):
        prim = _prim(str(joint["joint_id"]))
        position_degrees = math.degrees(position_urad / 1_000_000.0)
        velocity_degrees = math.degrees(velocity_urad_s / 1_000_000.0)
        joint_blocks.append(
            "\n".join(
                (
                    f'        over "{prim}" (',
                    '            prepend apiSchemas = ["PhysicsJointStateAPI:angular"]',
                    "        )",
                    "        {",
                    f"            float state:angular:physics:position = {_number(position_degrees, 9)}",
                    f"            float state:angular:physics:velocity = {_number(velocity_degrees, 9)}",
                    "        }",
                )
            )
        )
    escaped_base = str(base).replace("@", "@@")
    return (
        "#usda 1.0\n"
        "(\n"
        '    defaultPrim = "Humanoid"\n'
        "    metersPerUnit = 1\n"
        '    upAxis = "Z"\n'
        ")\n\n"
        'def Xform "Humanoid" (\n'
        f"    prepend references = @{escaped_base}@</Humanoid>\n"
        ")\n"
        "{\n"
        f"    custom int nextengine:freshSceneSourceCaseOrdinal = {source_case_ordinal}\n"
        f'    custom string nextengine:freshSceneArtifactSha256 = "{artifact_sha256}"\n'
        '    over "Bodies"\n'
        "    {\n"
        '        over "body_pelvis"\n'
        "        {\n"
        f"            double3 xformOp:translate = {position}\n"
        f"            quatf xformOp:orient = {quaternion}\n"
        f"            vector3f physics:velocity = {linear_com_world}\n"
        f"            vector3f physics:angularVelocity = {angular_world}\n"
        "        }\n"
        "    }\n"
        '    over "Joints"\n'
        "    {\n"
        + "\n".join(joint_blocks)
        + "\n    }\n"
        "}\n"
    )


def compare_reset_paths(
    *,
    fresh_rows: Sequence[Mapping[str, Any]],
    partial_rows: Sequence[Mapping[str, Any]],
    maximum_first_tick_impulse_delta: int,
) -> dict[str, Any]:
    if (
        maximum_first_tick_impulse_delta < 0
        or len(fresh_rows) != len(partial_rows)
        or not fresh_rows
    ):
        raise ValueError("reset comparison inputs are invalid")
    cases = []
    all_outcomes_equal = True
    maximum_delta = 0
    within_impulse_tolerance = True
    for fresh, partial in zip(fresh_rows, partial_rows, strict=True):
        identity = (fresh["clip_id"], int(fresh["start_frame"]))
        if identity != (partial["clip_id"], int(partial["start_frame"])):
            raise ValueError("reset comparison case order differs")
        fresh_outcome = _outcome(fresh)
        partial_outcome = _outcome(partial)
        outcomes_equal = fresh_outcome == partial_outcome
        fresh_impulse = tuple(int(value) for value in fresh["first_tick_impulses"])
        partial_impulse = tuple(int(value) for value in partial["first_tick_impulses"])
        if len(fresh_impulse) != len(partial_impulse):
            raise ValueError("reset comparison impulse width differs")
        delta = max(
            (abs(left - right) for left, right in zip(fresh_impulse, partial_impulse, strict=True)),
            default=0,
        )
        maximum_delta = max(maximum_delta, delta)
        all_outcomes_equal &= outcomes_equal
        within_impulse_tolerance &= delta <= maximum_first_tick_impulse_delta
        cases.append(
            {
                "case_ordinal": int(fresh["case_ordinal"]),
                "clip_id": identity[0],
                "start_frame": identity[1],
                "fresh_outcome": fresh_outcome,
                "partial_outcome": partial_outcome,
                "outcomes_equal": outcomes_equal,
                "maximum_first_tick_impulse_delta_micronewton_seconds": delta,
                "first_tick_impulse_within_tolerance": (
                    delta <= maximum_first_tick_impulse_delta
                ),
            }
        )
    equivalent = all_outcomes_equal and within_impulse_tolerance
    return {
        "status": "PASS" if equivalent else "FAIL",
        "equivalent_within_frozen_bounds": equivalent,
        "all_outcomes_reasons_and_terminal_ticks_equal": all_outcomes_equal,
        "maximum_first_tick_impulse_delta_micronewton_seconds": maximum_delta,
        "maximum_allowed_first_tick_impulse_delta_micronewton_seconds": (
            maximum_first_tick_impulse_delta
        ),
        "all_first_tick_impulses_within_tolerance": within_impulse_tolerance,
        "cases": cases,
    }


def evaluate_bounded_acceptance(
    *,
    cases: Sequence[ContactPrototypeCase],
    rows: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    if len(cases) != len(rows) or not cases:
        raise ValueError("bounded acceptance inputs are invalid")
    target_categories = tuple(
        sorted(
            {
                category
                for case in cases
                for category in case.target_failure_categories
            }
        )
    )
    source_counts = {category: 0 for category in target_categories}
    result_counts = {category: 0 for category in target_categories}
    control_regressions = []
    new_reasons = []
    for case, row in zip(cases, rows, strict=True):
        if (
            case.ordinal != int(row["case_ordinal"])
            or case.clip_id != row["clip_id"]
            or case.frame_first != int(row["start_frame"])
        ):
            raise ValueError("bounded acceptance case identity differs")
        result_reasons = set(_outcome(row)["reasons"])
        source_reasons = set(case.baseline_reasons)
        for category in target_categories:
            source_counts[category] += category in source_reasons
            result_counts[category] += category in result_reasons
        if not case.target_failure_categories and row["status"] != "PASS":
            control_regressions.append(case.ordinal)
        introduced = sorted(result_reasons - source_reasons)
        if introduced:
            new_reasons.append(
                {"case_ordinal": case.ordinal, "reasons": introduced}
            )
    decreases = {
        category: result_counts[category] < source_counts[category]
        for category in target_categories
    }
    accepted = not control_regressions and not new_reasons and all(decreases.values())
    return {
        "status": "PASS" if accepted else "FAIL",
        "accepted_for_full_v19_build": accepted,
        "passing_control_regression_count": len(control_regressions),
        "passing_control_regression_case_ordinals": control_regressions,
        "new_required_safety_reason_count": len(new_reasons),
        "new_required_safety_reasons": new_reasons,
        "targeted_failure_category_source_counts": source_counts,
        "targeted_failure_category_result_counts": result_counts,
        "strict_decrease_by_targeted_failure_category": decreases,
    }


def engine_to_isaac_vector(value: NDArray[np.float64]) -> NDArray[np.float64]:
    value = np.asarray(value, dtype=np.float64)
    if value.shape != (3,):
        raise ValueError("engine vector must have three components")
    return np.asarray((value[0], -value[2], value[1]), dtype=np.float64)


def engine_xyzw_to_isaac_wxyz(value: NDArray[np.float64]) -> NDArray[np.float64]:
    value = np.asarray(value, dtype=np.float64)
    if value.shape != (4,):
        raise ValueError("engine quaternion must have four components")
    return np.asarray((value[3], value[0], -value[2], value[1]), dtype=np.float64)


def rotate_world_to_local_wxyz(
    vector: NDArray[np.float64], quaternion: NDArray[np.float64]
) -> NDArray[np.float64]:
    vector = np.asarray(vector, dtype=np.float64)
    quaternion = np.asarray(quaternion, dtype=np.float64)
    if vector.shape != (3,) or quaternion.shape != (4,):
        raise ValueError("root local rotation shape mismatch")
    quaternion = quaternion / np.linalg.norm(quaternion)
    w = quaternion[0]
    xyz = quaternion[1:]
    first = np.cross(xyz, vector)
    second = np.cross(xyz, first - w * vector)
    return vector + 2.0 * second


def rotate_local_to_world_wxyz(
    vector: NDArray[np.float64], quaternion: NDArray[np.float64]
) -> NDArray[np.float64]:
    vector = np.asarray(vector, dtype=np.float64)
    quaternion = np.asarray(quaternion, dtype=np.float64)
    if vector.shape != (3,) or quaternion.shape != (4,):
        raise ValueError("root world rotation shape mismatch")
    quaternion = quaternion / np.linalg.norm(quaternion)
    w = quaternion[0]
    xyz = quaternion[1:]
    first = np.cross(xyz, vector)
    second = np.cross(xyz, first + w * vector)
    return vector + 2.0 * second


def _outcome(row: Mapping[str, Any]) -> dict[str, Any]:
    violation = row.get("first_required_safety_violation")
    return {
        "status": row["status"],
        "reasons": sorted(violation.get("reasons", ())) if violation else [],
        "terminal_motor_tick": int(row["observed_motor_ticks"]),
    }


def _prim(identifier: str) -> str:
    return re.sub(r"[^A-Za-z0-9_]", "_", identifier)


def _number(value: float, precision: int) -> str:
    if abs(value) < 10.0 ** (-precision):
        return "0"
    output = format(float(value), f".{precision}g")
    return "0" if output in {"-0", "-0.0"} else output


def _tuple(value: NDArray[np.float64], *, precision: int) -> str:
    return "(" + ", ".join(_number(item, precision) for item in value) + ")"


def _canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
