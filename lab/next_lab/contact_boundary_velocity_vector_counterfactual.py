from __future__ import annotations

import hashlib
import json
from copy import deepcopy
from pathlib import Path
from typing import Any, Mapping

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_boundary_velocity_counterfactual import (
    CASE_ORDINAL,
    DIRECT_TARGET_ARRAY_NAME,
    DOF_ORDINAL,
    JOINT_ID,
    SOURCE_CASE_ORDINAL,
    V7_PROTOTYPE_ID,
    V9_PROTOTYPE_ID,
    canonical_json,
    sha256,
)
from next_lab.contact_manifold_physx import load_contact_prototype_cases


COUNTERFACTUAL_ID = (
    "nextengine.humanoid-contact-boundary-velocity-vector-counterfactual.v1"
)
COUNTERFACTUAL_CHECK = (
    "TRAIN-4-CONTACT-MANIFOLD-BOUNDARY-VELOCITY-VECTOR-COUNTERFACTUAL"
)
ARRAY_NAME = "joint_velocity_urad_s"
FRAME_OFFSET = 0
CHANGED_DOF_ORDINALS = (
    0,
    1,
    3,
    5,
    6,
    7,
    9,
    10,
    11,
    12,
    13,
    14,
    15,
    16,
    17,
    19,
    20,
    21,
)
V9_VECTOR_SHA256 = (
    "5b76f62975e26711bf2e27db9df43509037f9d49c966369172ff1a47d0decece"
)
V7_VECTOR_SHA256 = (
    "84a410295ea8f1d628ac5f8fd7048b7aa2a7ba4813e6457d66438a149d117a84"
)
DELTA_VECTOR_SHA256 = (
    "26b614cdcc3587aba8310f91b8c374a02f7ec030fad20759ff290281f2775cb0"
)


def build_boundary_velocity_vector_counterfactual(
    *,
    profile_path: Path,
    source_audit_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
    staging_directory: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Build the frame-0 all-joint velocity-vector R101 discriminator."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            source_audit_path,
            v7_manifest_path,
            v9_manifest_path,
            tool_path,
        )
    )
    (
        profile_path,
        source_audit_path,
        v7_manifest_path,
        v9_manifest_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths) or not staging_directory.is_dir():
        raise FileNotFoundError("boundary-velocity-vector input is absent")
    profile = json.loads(profile_path.read_bytes())
    _validate_profile(
        profile=profile,
        source_audit_path=source_audit_path,
        v7_manifest_path=v7_manifest_path,
        v9_manifest_path=v9_manifest_path,
    )
    v7_manifest, _, v7_cases = load_contact_prototype_cases(
        manifest_path=v7_manifest_path,
        source_audit_path=source_audit_path,
    )
    v9_manifest, _, v9_cases = load_contact_prototype_cases(
        manifest_path=v9_manifest_path,
        source_audit_path=source_audit_path,
    )
    _validate_manifest(
        manifest=v7_manifest,
        manifest_path=v7_manifest_path,
        cases=v7_cases,
        expected=profile["source"]["v7"],
        prototype_id=V7_PROTOTYPE_ID,
    )
    _validate_manifest(
        manifest=v9_manifest,
        manifest_path=v9_manifest_path,
        cases=v9_cases,
        expected=profile["source"]["v9"],
        prototype_id=V9_PROTOTYPE_ID,
    )
    shared_keys = (
        "source_audit_sha256",
        "corpus_manifest_sha256",
        "corpus_manifest_file_sha256",
        "descriptor_sha256",
    )
    if any(
        v7_manifest["identities"].get(key)
        != v9_manifest["identities"].get(key)
        for key in shared_keys
    ):
        raise ValueError("V7 and V9 source identities disagree")

    v7_case = v7_cases[CASE_ORDINAL]
    v9_case = v9_cases[CASE_ORDINAL]
    if (
        v7_case.source_case_ordinal != SOURCE_CASE_ORDINAL
        or v9_case.source_case_ordinal != SOURCE_CASE_ORDINAL
        or v7_case.baseline_status != "PASS"
        or v9_case.baseline_status != "PASS"
        or v7_case.baseline_reasons
        or v9_case.baseline_reasons
        or v7_case.target_failure_categories
        or v9_case.target_failure_categories
        or (v7_case.clip_id, v7_case.frame_first, v7_case.frame_last)
        != (v9_case.clip_id, v9_case.frame_first, v9_case.frame_last)
    ):
        raise ValueError("matched case-10 source identity is invalid")

    v7_arrays = _load_all_arrays(v7_case.artifact_path)
    v9_arrays = _load_all_arrays(v9_case.artifact_path)
    v7_vector = v7_arrays[ARRAY_NAME][FRAME_OFFSET]
    v9_vector = v9_arrays[ARRAY_NAME][FRAME_OFFSET]
    delta = v7_vector - v9_vector
    changed_ordinals = tuple(int(value) for value in np.flatnonzero(delta))
    if (
        set(v7_arrays) != set(v9_arrays)
        or not np.array_equal(
            v7_arrays[DIRECT_TARGET_ARRAY_NAME][:, DOF_ORDINAL],
            v9_arrays[DIRECT_TARGET_ARRAY_NAME][:, DOF_ORDINAL],
        )
        or changed_ordinals != CHANGED_DOF_ORDINALS
        or _vector_sha256(v9_vector) != V9_VECTOR_SHA256
        or _vector_sha256(v7_vector) != V7_VECTOR_SHA256
        or _vector_sha256(delta) != DELTA_VECTOR_SHA256
    ):
        raise ValueError("matched boundary-velocity vector differs")

    candidate = {name: np.array(value, copy=True) for name, value in v9_arrays.items()}
    candidate[ARRAY_NAME][FRAME_OFFSET] = v7_vector
    changed_by_array = {
        name: int(np.count_nonzero(candidate[name] != v9_arrays[name]))
        for name in candidate
    }
    if (
        sum(changed_by_array.values()) != len(CHANGED_DOF_ORDINALS)
        or changed_by_array[ARRAY_NAME] != len(CHANGED_DOF_ORDINALS)
    ):
        raise AssertionError("velocity-vector candidate changed outside frame 0")

    artifact_directory = staging_directory / "cases"
    artifact_directory.mkdir()
    artifact_path = artifact_directory / (
        "boundary-velocity-vector--cmu16-walk-nominal-b--start-0238.npz"
    )
    np.savez(artifact_path, **candidate)
    reloaded = _load_all_arrays(artifact_path)
    if set(reloaded) != set(candidate) or any(
        not np.array_equal(reloaded[name], value)
        for name, value in candidate.items()
    ):
        raise AssertionError("saved velocity-vector artifact is not exact")

    edit = {
        "array": ARRAY_NAME,
        "frame_offset": FRAME_OFFSET,
        "source_frame": v9_case.frame_first,
        "dof_scope": "all-ordered-joints",
        "changed_dof_ordinals": list(CHANGED_DOF_ORDINALS),
        "changed_dof_count": len(CHANGED_DOF_ORDINALS),
        "source_vector_sha256": V9_VECTOR_SHA256,
        "replacement_vector_sha256": V7_VECTOR_SHA256,
        "delta_vector_sha256": DELTA_VECTOR_SHA256,
    }
    record = deepcopy(v9_manifest["cases"][CASE_ORDINAL])
    record["artifact"] = {
        "relative_path": str(artifact_path.relative_to(staging_directory)),
        "sha256": sha256(artifact_path),
        "bytes": artifact_path.stat().st_size,
    }
    record["counterfactual_role"] = "boundary-velocity-vector-locality"
    record["counterfactual_edit"] = edit
    identities = v9_manifest["identities"]
    report = {
        "schema_version": 1,
        "check": COUNTERFACTUAL_CHECK,
        "status": "PASS",
        "claim": "FrameZeroJointVelocityVectorCounterfactualResearchOnly",
        "gate_decision": "PERMIT_EXACTLY_ONE_R101_FRESH_TRACE_ONLY",
        "prototype_id": COUNTERFACTUAL_ID,
        "scope": {
            "case_count": 1,
            "failure_case_count": 0,
            "control_case_count": 1,
            "case_scope": "single-boundary-velocity-vector-counterfactual",
            "projection_domain": "exact V9 case-10 artifact",
            "clip_ids": [v9_case.clip_id],
            "source_case_ordinal": SOURCE_CASE_ORDINAL,
            "source_v9_case_ordinal": CASE_ORDINAL,
            "source_v7_control_case_ordinal": CASE_ORDINAL,
        },
        "identities": {
            "source_audit_sha256": sha256(source_audit_path),
            "prototype_profile_sha256": sha256(profile_path),
            "corpus_manifest_sha256": identities["corpus_manifest_sha256"],
            "corpus_manifest_file_sha256": identities[
                "corpus_manifest_file_sha256"
            ],
            "descriptor_sha256": identities["descriptor_sha256"],
            "v7_manifest_sha256": v7_manifest["manifest_sha256"],
            "v7_manifest_file_sha256": sha256(v7_manifest_path),
            "v7_case_artifact_sha256": v7_case.artifact_sha256,
            "v9_manifest_sha256": v9_manifest["manifest_sha256"],
            "v9_manifest_file_sha256": sha256(v9_manifest_path),
            "v9_case_artifact_sha256": v9_case.artifact_sha256,
            "tool_sha256": sha256(tool_path),
            "builder_module_sha256": sha256(Path(__file__).resolve()),
        },
        "offline_verification": {
            "status": "PASS",
            "direct_target_array": DIRECT_TARGET_ARRAY_NAME,
            "direct_target_joint_id": JOINT_ID,
            "direct_target_disagreement_count": 0,
            "changed_array_element_count": len(CHANGED_DOF_ORDINALS),
            "changed_array_element_count_by_array": changed_by_array,
            **edit,
        },
        "cases": [record],
        "fresh_scene_runs": 0,
        "physx_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["manifest_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report


def _validate_profile(
    *,
    profile: Mapping[str, Any],
    source_audit_path: Path,
    v7_manifest_path: Path,
    v9_manifest_path: Path,
) -> None:
    source = profile.get("source", {})
    execution = profile.get("execution", {})
    edit = profile.get("edit", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("counterfactual_id") != COUNTERFACTUAL_ID
        or profile.get("status") != "FrozenResearchOnly"
        or source.get("audit_sha256") != sha256(source_audit_path)
        or source.get("v7", {}).get("manifest_file_sha256")
        != sha256(v7_manifest_path)
        or source.get("v9", {}).get("manifest_file_sha256")
        != sha256(v9_manifest_path)
        or edit
        != {
            "array": ARRAY_NAME,
            "frame_offset": FRAME_OFFSET,
            "dof_scope": "all-ordered-joints",
            "changed_dof_ordinals": list(CHANGED_DOF_ORDINALS),
            "changed_dof_count": len(CHANGED_DOF_ORDINALS),
            "source_vector_sha256": V9_VECTOR_SHA256,
            "replacement_vector_sha256": V7_VECTOR_SHA256,
            "delta_vector_sha256": DELTA_VECTOR_SHA256,
        }
        or execution
        != {
            "generated_case_count": 1,
            "fresh_scene_runs": 0,
            "physx_runs": 0,
            "indexed_partial_reset_enabled": False,
            "all_17_enabled": False,
            "optimizer_steps": 0,
            "training_runs": 0,
        }
    ):
        raise ValueError("boundary-velocity-vector profile is invalid")


def _validate_manifest(
    *,
    manifest: Mapping[str, Any],
    manifest_path: Path,
    cases: tuple[Any, ...],
    expected: Mapping[str, Any],
    prototype_id: str,
) -> None:
    if (
        manifest.get("prototype_id") != prototype_id
        or expected.get("prototype_id") != prototype_id
        or manifest.get("manifest_sha256") != expected.get("manifest_sha256")
        or sha256(manifest_path) != expected.get("manifest_file_sha256")
        or manifest.get("identities", {}).get("prototype_profile_sha256")
        != expected.get("prototype_profile_sha256")
        or len(cases) != 17
        or cases[CASE_ORDINAL].artifact_sha256
        != expected.get("case_artifact_sha256")
        or manifest.get("scope", {}).get("case_scope") != "all"
        or manifest.get("scope", {}).get("failure_case_count") != 7
        or manifest.get("scope", {}).get("control_case_count") != 10
    ):
        raise ValueError(f"{prototype_id} source manifest differs")


def _load_all_arrays(path: Path) -> dict[str, NDArray[Any]]:
    with np.load(path, allow_pickle=False) as source:
        return {name: np.array(source[name], copy=True) for name in source.files}


def _vector_sha256(value: NDArray[Any]) -> str:
    canonical = np.asarray(value, dtype="<i8")
    return hashlib.sha256(canonical.tobytes(order="C")).hexdigest()
