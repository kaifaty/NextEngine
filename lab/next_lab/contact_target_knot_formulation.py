from __future__ import annotations

import hashlib
import itertools
import json
from pathlib import Path
from typing import Any, Mapping, Sequence

import numpy as np
from numpy.typing import NDArray


FORMULATION_ID = "nextengine.humanoid-contact-target-knot-formulation.v1"
CHECK_ID = "TRAIN-4-CONTACT-TARGET-KNOT-FORMULATION"
ARRAY_NAME = "joint_position_urad"
SOURCE_CASE_ORDINAL = 7967
FRAME_FIRST = 238
FRAME_LAST = 249
FRAME_COUNT = 12
ACTION_COUNT = 23
IMMUTABLE_FRAME_OFFSETS = (0, 1)
KNOT_FRAME_OFFSETS = (2, 6, 11)
COEFFICIENT_SCALE_BASIS_POINTS = 10_000
COEFFICIENT_GRID_BASIS_POINTS = (0, 5_000, 10_000)
JOINT_DOF_IDS = (
    "joint.left-hip-pitch",
    "joint.left-hip-roll",
    "joint.left-hip-yaw",
    "joint.left-knee",
    "joint.left-ankle-pitch",
    "joint.left-ankle-roll",
    "joint.right-hip-pitch",
    "joint.right-hip-roll",
    "joint.right-hip-yaw",
    "joint.right-knee",
    "joint.right-ankle-pitch",
    "joint.right-ankle-roll",
    "joint.torso-pitch",
    "joint.torso-roll",
    "joint.torso-yaw",
    "joint.left-shoulder-pitch",
    "joint.left-shoulder-roll",
    "joint.left-shoulder-yaw",
    "joint.left-elbow",
    "joint.right-shoulder-pitch",
    "joint.right-shoulder-roll",
    "joint.right-shoulder-yaw",
    "joint.right-elbow",
)


def build_target_knot_formulation(
    *,
    profile_path: Path,
    source_audit_path: Path,
    r102_audit_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
    descriptor_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Close the read-only R103 reference-target knot formulation."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            source_audit_path,
            r102_audit_path,
            v7_manifest_path,
            v9_manifest_path,
            descriptor_path,
            tool_path,
        )
    )
    (
        profile_path,
        source_audit_path,
        r102_audit_path,
        v7_manifest_path,
        v9_manifest_path,
        descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("target-knot formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    _validate_r102(
        report_path=r102_audit_path,
        expected=profile["source"]["r102"],
    )
    if sha256(source_audit_path) != profile["source"]["audit_sha256"]:
        raise ValueError("source audit identity differs")
    _validate_descriptor(
        path=descriptor_path,
        expected_sha256=profile["source"]["descriptor_sha256"],
    )

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
    if any(
        manifest.get("identities", {}).get("descriptor_sha256")
        != sha256(descriptor_path)
        for manifest in (v7_manifest, v9_manifest)
    ):
        raise ValueError("source manifest descriptor identity differs")
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

    v7_arrays = _load_arrays(v7_artifact)
    v9_arrays = _load_arrays(v9_artifact)
    v7_target = _target_array(v7_arrays)
    v9_target = _target_array(v9_arrays)
    _validate_reference_frames(v7_arrays, v9_arrays)

    complete_record, complete_artifact = _load_complete_v9_clip(
        manifest=v9_manifest,
        manifest_path=v9_manifest_path,
        expected=profile["source"]["v9"],
    )
    complete_arrays = _load_arrays(complete_artifact)
    complete_target = np.asarray(complete_arrays[ARRAY_NAME], dtype=np.int64)
    if (
        complete_target.shape
        != (int(profile["scope"]["complete_clip_frame_count"]), ACTION_COUNT)
        or not np.array_equal(
            complete_target[FRAME_FIRST : FRAME_LAST + 1], v9_target
        )
    ):
        raise ValueError("V9 complete clip does not contain the exact case slice")

    anchor_delta = v7_target - v9_target
    anchor = _anchor_facts(v7_target, v9_target, anchor_delta)
    if anchor != profile["expected_anchor"]:
        raise ValueError("predeclared target anchor differs")

    lattice = []
    for coefficients in itertools.product(
        COEFFICIENT_GRID_BASIS_POINTS, repeat=len(KNOT_FRAME_OFFSETS)
    ):
        alpha = interpolate_coefficients(coefficients)
        candidate = apply_anchor(v9_target, anchor_delta, alpha)
        _validate_candidate(candidate, v9_target, v7_target, alpha)
        lattice.append(
            {
                "candidate_id": candidate_id(coefficients),
                "knot_coefficients_basis_points": list(coefficients),
                "interpolated_coefficients_basis_points": alpha.tolist(),
                "target_sha256": array_sha256(candidate),
                "changed_target_element_count": int(
                    np.count_nonzero(candidate != v9_target)
                ),
                "maximum_absolute_target_correction_microradians": int(
                    np.max(np.abs(candidate - v9_target))
                ),
            }
        )
    if len(lattice) != 27 or len({row["target_sha256"] for row in lattice}) != 27:
        raise AssertionError("target-knot lattice is not a unique 27-point grid")

    prefix_lattice = _prefix_lattice(v9_target, anchor_delta)
    formulation = dict(profile["formulation"])
    formulation["candidate_lattice_count"] = len(lattice)
    formulation["stage_1_prefix_count"] = len(prefix_lattice)
    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "formulation_id": FORMULATION_ID,
        "scope": profile["scope"],
        "reference_semantics": profile["reference_semantics"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "source_audit_sha256": sha256(source_audit_path),
            "descriptor_sha256": sha256(descriptor_path),
            "r102_report_sha256": profile["source"]["r102"][
                "report_sha256"
            ],
            "r102_report_file_sha256": sha256(r102_audit_path),
            "v7_manifest_sha256": v7_manifest["manifest_sha256"],
            "v7_manifest_file_sha256": sha256(v7_manifest_path),
            "v7_case_artifact_sha256": sha256(v7_artifact),
            "v9_manifest_sha256": v9_manifest["manifest_sha256"],
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "v9_case_artifact_sha256": sha256(v9_artifact),
            "v9_complete_clip_artifact_sha256": sha256(complete_artifact),
            "tool_sha256": sha256(tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
        },
        "source_records": {
            "v7_case": _case_identity(v7_case),
            "v9_case": _case_identity(v9_case),
            "v9_complete_clip": {
                "clip_id": complete_record["clip_id"],
                "frame_first": int(complete_record["frame_first"]),
                "frame_last": int(complete_record["frame_last"]),
            },
        },
        "anchor": anchor,
        "formulation": formulation,
        "stage_1_prefix_lattice": prefix_lattice,
        "candidate_lattice": lattice,
        "offline_preflight_contract": profile["offline_preflight"],
        "conditional_native_contract": profile["conditional_native_evaluation"],
        "bounded_acceptance": {
            "candidate_artifact_status": "NOT_BUILT",
            "offline_candidate_preflight": profile["decision"][
                "offline_candidate_preflight"
            ],
            "candidate_search": profile["decision"]["candidate_search"],
            "physx": profile["decision"]["physx"],
            "all_17": profile["decision"]["all_17"],
            "full_v19": profile["decision"]["full_v19"],
            "training": profile["decision"]["training"],
        },
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


def interpolate_coefficients(
    knot_coefficients: Sequence[int],
) -> NDArray[np.int64]:
    if (
        len(knot_coefficients) != len(KNOT_FRAME_OFFSETS)
        or any(value not in COEFFICIENT_GRID_BASIS_POINTS for value in knot_coefficients)
    ):
        raise ValueError("target-knot coefficients are outside the frozen grid")
    result = np.zeros(FRAME_COUNT, dtype=np.int64)
    for segment in range(len(KNOT_FRAME_OFFSETS) - 1):
        start = KNOT_FRAME_OFFSETS[segment]
        end = KNOT_FRAME_OFFSETS[segment + 1]
        left = int(knot_coefficients[segment])
        right = int(knot_coefficients[segment + 1])
        width = end - start
        for frame in range(start, end + 1):
            numerator = left * (end - frame) + right * (frame - start)
            result[frame] = round_divide_ties_to_even(numerator, width)
    return result


def apply_anchor(
    baseline: NDArray[np.int64],
    anchor_delta: NDArray[np.int64],
    coefficients: NDArray[np.int64],
) -> NDArray[np.int64]:
    baseline = np.asarray(baseline, dtype=np.int64)
    anchor_delta = np.asarray(anchor_delta, dtype=np.int64)
    coefficients = np.asarray(coefficients, dtype=np.int64)
    if (
        baseline.shape != (FRAME_COUNT, ACTION_COUNT)
        or anchor_delta.shape != baseline.shape
        or coefficients.shape != (FRAME_COUNT,)
    ):
        raise ValueError("target-knot array shape differs")
    candidate = baseline.copy()
    for frame in range(FRAME_COUNT):
        for action in range(ACTION_COUNT):
            correction = round_divide_ties_to_even(
                int(anchor_delta[frame, action]) * int(coefficients[frame]),
                COEFFICIENT_SCALE_BASIS_POINTS,
            )
            candidate[frame, action] += correction
    return candidate


def round_divide_ties_to_even(numerator: int, denominator: int) -> int:
    if denominator <= 0:
        raise ValueError("rounding denominator must be positive")
    sign = -1 if numerator < 0 else 1
    quotient, remainder = divmod(abs(numerator), denominator)
    doubled = remainder * 2
    if doubled > denominator or (doubled == denominator and quotient % 2 == 1):
        quotient += 1
    return sign * quotient


def candidate_id(coefficients: Sequence[int]) -> str:
    return "-".join(
        f"a{frame:02d}-{int(value):05d}"
        for frame, value in zip(KNOT_FRAME_OFFSETS, coefficients, strict=True)
    )


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def array_sha256(value: NDArray[Any]) -> str:
    canonical = np.asarray(value, dtype="<i8")
    return hashlib.sha256(canonical.tobytes(order="C")).hexdigest()


def _validate_profile(profile: Mapping[str, Any]) -> None:
    formulation = profile.get("formulation", {})
    semantics = profile.get("reference_semantics", {})
    decision = profile.get("decision", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim")
        != "OptimizerFreeReferenceTargetKnotFormulationOnly"
        or profile.get("scope", {}).get("source_case_ordinal")
        != SOURCE_CASE_ORDINAL
        or profile.get("scope", {}).get("frame_first") != FRAME_FIRST
        or profile.get("scope", {}).get("frame_last") != FRAME_LAST
        or tuple(formulation.get("joint_dof_ids", ())) != JOINT_DOF_IDS
        or tuple(formulation.get("immutable_frame_offsets", ()))
        != IMMUTABLE_FRAME_OFFSETS
        or tuple(formulation.get("knot_frame_offsets", ()))
        != KNOT_FRAME_OFFSETS
        or formulation.get("coefficient_scale_basis_points")
        != COEFFICIENT_SCALE_BASIS_POINTS
        or tuple(formulation.get("coefficient_grid_basis_points", ()))
        != COEFFICIENT_GRID_BASIS_POINTS
        or formulation.get("interpolation")
        != "piecewise-linear coefficient with integer ties-to-even rounding"
        or semantics.get("candidate_target_equals_reference_joint_position")
        is not True
        or semantics.get("separate_control_target_array_allowed") is not False
        or semantics.get("realized_rollout_state_can_replace_reference") is not False
        or semantics.get("frame_0_state_mutable") is not False
        or semantics.get("controller_semantics_mutable") is not False
        or decision
        != {
            "complete": "PERMIT_R104_OFFLINE_LATTICE_PREFLIGHT_ONLY",
            "offline_candidate_preflight": "AUTHORIZED",
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        }
    ):
        raise ValueError("target-knot formulation profile is invalid")


def _validate_r102(*, report_path: Path, expected: Mapping[str, Any]) -> None:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R102 report file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        report.get("audit_id") != expected.get("audit_id")
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("status") != "COMPLETE"
        or report.get("findings", {}).get("boundary_state_is_sufficient")
        is not False
        or report.get("findings", {}).get(
            "manual_boundary_substitution_disposition"
        )
        != "EXHAUSTED"
        or report.get("future_candidate_contract", {}).get("status")
        != "FORMULATION_ONLY"
        or report.get("bounded_acceptance", {}).get("candidate_search")
        != "NOT_AUTHORIZED"
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "physx_runs",
                "candidate_evaluations",
                "trajectory_mutations",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R102 audit contract differs")


def _load_manifest(
    *,
    path: Path,
    expected: Mapping[str, Any],
    source_audit_path: Path,
) -> dict[str, Any]:
    if sha256(path) != expected.get("manifest_file_sha256"):
        raise ValueError("source manifest file identity differs")
    manifest = json.loads(path.read_bytes())
    embedded = manifest.get("manifest_sha256")
    without_hash = dict(manifest)
    without_hash.pop("manifest_sha256", None)
    if (
        manifest.get("prototype_id") != expected.get("prototype_id")
        or embedded != expected.get("manifest_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or manifest.get("status") != "PASS"
        or manifest.get("scope", {}).get("case_scope") != "all"
        or manifest.get("identities", {}).get("source_audit_sha256")
        != sha256(source_audit_path)
        or manifest.get("identities", {}).get("prototype_profile_sha256")
        != expected.get("prototype_profile_sha256")
        or manifest.get("optimizer_steps") != 0
        or manifest.get("training_runs") != 0
    ):
        raise ValueError("source manifest contract differs")
    return manifest


def _load_case(
    *,
    manifest: Mapping[str, Any],
    manifest_path: Path,
    expected: Mapping[str, Any],
) -> tuple[Mapping[str, Any], Path]:
    matches = [
        row
        for row in manifest.get("cases", ())
        if row.get("source_case_ordinal") == SOURCE_CASE_ORDINAL
    ]
    if len(matches) != 1:
        raise ValueError("matched source case is absent or ambiguous")
    record = matches[0]
    artifact = (manifest_path.parent / record["artifact"]["relative_path"]).resolve()
    if (
        record.get("clip_id") != "cmu16-walk-nominal-b"
        or record.get("frame_first") != FRAME_FIRST
        or record.get("frame_last") != FRAME_LAST
        or record.get("baseline_status") != "PASS"
        or record.get("baseline_reasons") != []
        or record.get("target_failure_categories") != []
        or record.get("artifact", {}).get("sha256")
        != expected.get("case_artifact_sha256")
        or not artifact.is_file()
        or sha256(artifact) != expected.get("case_artifact_sha256")
    ):
        raise ValueError("matched source case differs")
    return record, artifact


def _load_complete_v9_clip(
    *,
    manifest: Mapping[str, Any],
    manifest_path: Path,
    expected: Mapping[str, Any],
) -> tuple[Mapping[str, Any], Path]:
    matches = [
        row
        for row in manifest.get("complete_clips", ())
        if row.get("clip_id") == "cmu16-walk-nominal-b"
    ]
    if len(matches) != 1:
        raise ValueError("V9 complete clip is absent or ambiguous")
    record = matches[0]
    artifact = (manifest_path.parent / record["artifact"]["relative_path"]).resolve()
    if (
        record.get("frame_first") != 0
        or record.get("frame_last") != 800
        or record.get("solve_count") != 1
        or record.get("projection_diagnostics", {}).get("status") != "PASS"
        or record.get("artifact", {}).get("sha256")
        != expected.get("complete_clip_artifact_sha256")
        or not artifact.is_file()
        or sha256(artifact) != expected.get("complete_clip_artifact_sha256")
    ):
        raise ValueError("V9 complete clip differs")
    return record, artifact


def _load_arrays(path: Path) -> dict[str, NDArray[Any]]:
    with np.load(path, allow_pickle=False) as source:
        return {name: np.array(source[name], copy=True) for name in source.files}


def _target_array(arrays: Mapping[str, NDArray[Any]]) -> NDArray[np.int64]:
    target = np.asarray(arrays.get(ARRAY_NAME), dtype=np.int64)
    if target.shape != (FRAME_COUNT, ACTION_COUNT):
        raise ValueError("source target array shape differs")
    return target


def _validate_reference_frames(
    v7_arrays: Mapping[str, NDArray[Any]],
    v9_arrays: Mapping[str, NDArray[Any]],
) -> None:
    expected = np.arange(FRAME_FIRST, FRAME_LAST + 1, dtype=np.int64)
    if (
        not np.array_equal(v7_arrays.get("reference_frame"), expected)
        or not np.array_equal(v9_arrays.get("reference_frame"), expected)
    ):
        raise ValueError("reference frame identity differs")


def _validate_matched_case(
    v7_case: Mapping[str, Any], v9_case: Mapping[str, Any]
) -> None:
    keys = ("clip_id", "frame_first", "frame_last", "source_case_ordinal")
    if any(v7_case.get(key) != v9_case.get(key) for key in keys):
        raise ValueError("V7/V9 matched case identities disagree")


def _anchor_facts(
    v7_target: NDArray[np.int64],
    v9_target: NDArray[np.int64],
    delta: NDArray[np.int64],
) -> dict[str, Any]:
    mutable = delta[KNOT_FRAME_OFFSETS[0] :]
    return {
        "v7_target_sha256": array_sha256(v7_target),
        "v9_target_sha256": array_sha256(v9_target),
        "full_delta_sha256": array_sha256(delta),
        "mutable_delta_sha256": array_sha256(mutable),
        "full_changed_element_count": int(np.count_nonzero(delta)),
        "mutable_changed_element_count": int(np.count_nonzero(mutable)),
        "mutable_support_dof_ordinals": [
            int(value) for value in np.flatnonzero(np.any(mutable != 0, axis=0))
        ],
        "maximum_absolute_mutable_delta_microradians": int(
            np.max(np.abs(mutable))
        ),
    }


def _validate_candidate(
    candidate: NDArray[np.int64],
    baseline: NDArray[np.int64],
    anchor: NDArray[np.int64],
    coefficients: NDArray[np.int64],
) -> None:
    if (
        not np.array_equal(candidate[:2], baseline[:2])
        or np.any(coefficients < 0)
        or np.any(coefficients > COEFFICIENT_SCALE_BASIS_POINTS)
        or np.any(candidate < np.minimum(baseline, anchor))
        or np.any(candidate > np.maximum(baseline, anchor))
    ):
        raise AssertionError("target-knot candidate violates the convex anchor")


def _validate_descriptor(*, path: Path, expected_sha256: str) -> None:
    if sha256(path) != expected_sha256:
        raise ValueError("descriptor file identity differs")
    descriptor = json.loads(path.read_bytes())
    joints = sorted(
        descriptor.get("joints", ()), key=lambda row: int(row["dof_ordinal"])
    )
    if (
        len(joints) != ACTION_COUNT
        or tuple(int(row["dof_ordinal"]) for row in joints)
        != tuple(range(ACTION_COUNT))
        or tuple(str(row["joint_id"]) for row in joints) != JOINT_DOF_IDS
    ):
        raise ValueError("descriptor joint-DoF layout differs")


def _prefix_lattice(
    baseline: NDArray[np.int64], anchor_delta: NDArray[np.int64]
) -> list[dict[str, Any]]:
    rows = []
    for left, middle in itertools.product(
        COEFFICIENT_GRID_BASIS_POINTS, repeat=2
    ):
        coefficients = interpolate_coefficients((left, middle, middle))
        candidate = apply_anchor(baseline, anchor_delta, coefficients)
        prefix = candidate[: KNOT_FRAME_OFFSETS[1] + 1]
        rows.append(
            {
                "prefix_id": candidate_id((left, middle, middle)).rsplit("-", 2)[0],
                "knot_coefficients_basis_points": [left, middle],
                "horizon_last_frame_offset": KNOT_FRAME_OFFSETS[1],
                "target_prefix_sha256": array_sha256(prefix),
            }
        )
    return rows


def _case_identity(record: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "clip_id": record["clip_id"],
        "source_case_ordinal": int(record["source_case_ordinal"]),
        "frame_first": int(record["frame_first"]),
        "frame_last": int(record["frame_last"]),
    }
