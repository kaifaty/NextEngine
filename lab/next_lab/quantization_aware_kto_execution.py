from __future__ import annotations

import hashlib
import io
import json
import os
import resource
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.quantization_aware_kto_formulation import (
    CHECK_ID as R114_CHECK_ID,
)
from next_lab.quantization_aware_kto_formulation import (
    FORMULATION_ID as R114_FORMULATION_ID,
)
from next_lab.quantization_aware_kto_formulation import (
    _load_complete_arrays,
    _load_prior_arrays,
    audit_descriptor_lineage,
    audit_source_identity,
    audit_trajectory_lineage,
)
from next_lab.quantization_aware_kto_formulation import (
    _validate_profile as _validate_r114_profile,
)
from next_lab.quantization_aware_kto_solver import solve_quantization_aware_kto

EXECUTION_ID = "nextengine.humanoid-quantization-aware-kto-execution.v1"
CHECK_ID = "TRAIN-4-QUANTIZATION-AWARE-KTO-EXECUTION"
CACHE_FILE_NAME = "solver-private-kto-warm-start.npz"


def execute_quantization_aware_kto(
    *,
    execution_profile_path: Path,
    r114_report_path: Path,
    r114_profile_path: Path,
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
) -> tuple[dict[str, Any], bytes | None]:
    """Execute exactly one R115 solve after all report-only preflights pass."""

    direct_paths = tuple(
        path.resolve()
        for path in (
            execution_profile_path,
            r114_report_path,
            r114_profile_path,
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
        execution_profile_path,
        r114_report_path,
        r114_profile_path,
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
        raise FileNotFoundError("R115 execution input is absent")

    execution_profile = json.loads(execution_profile_path.read_bytes())
    _validate_profile(execution_profile)
    _validate_environment(execution_profile)
    _validate_repository(repository)
    r114_profile = json.loads(r114_profile_path.read_bytes())
    r114 = validate_r114(
        execution_profile=execution_profile,
        report_path=r114_report_path,
        profile_path=r114_profile_path,
    )
    source_identity = audit_source_identity(r114_profile, sources)
    descriptor_lineage = audit_descriptor_lineage(
        profile=r114_profile,
        descriptor_bytes=descriptor_bytes,
        legacy_descriptor_path=legacy_descriptor_path,
        v9_profile_path=v9_profile_path,
    )
    trajectory_lineage = audit_trajectory_lineage(
        profile=r114_profile,
        v9_profile_path=v9_profile_path,
        v9_manifest_path=v9_manifest_path,
        v9_complete_clip_path=v9_complete_clip_path,
        v7_profile_path=v7_profile_path,
        v7_manifest_path=v7_manifest_path,
        v7_case_path=v7_case_path,
    )
    validations = _validate_results(execution_profile, validation_results)
    v9_arrays, _ = _load_complete_arrays(v9_complete_clip_path)
    v7_arrays, _ = _load_prior_arrays(v7_case_path)
    v9_arrays["metadata_json_utf8"] = _raw_npz_array(
        v9_complete_clip_path, "metadata_json_utf8"
    )
    v7_arrays["metadata_json_utf8"] = _raw_npz_array(v7_case_path, "metadata_json_utf8")
    descriptor = descriptor_lineage.pop("current_descriptor")
    v9_profile = trajectory_lineage.pop("v9_profile")
    solve_result, cache_arrays = solve_quantization_aware_kto(
        descriptor=descriptor,
        v9_arrays=v9_arrays,
        v7_arrays=v7_arrays,
        v9_profile=v9_profile,
        r114_profile=r114_profile,
    )
    peak_memory_bytes = _peak_memory_bytes()
    memory_ok = peak_memory_bytes <= int(
        r114_profile["execution_budget"]["maximum_resident_memory_bytes"]
    )
    if not memory_ok:
        solve_result = {
            **solve_result,
            "status": "FAIL",
            "termination": "resident_memory_budget_exceeded",
            "resource_budget_satisfied": False,
        }
        cache_arrays = None
    passed = solve_result["status"] == "PASS"
    cache_bytes = _serialize_cache(cache_arrays) if passed else None
    cache_record = (
        {
            "status": "EMITTED_TRANSIENT_SOLVER_PRIVATE",
            "file_name": CACHE_FILE_NAME,
            "sha256": hashlib.sha256(cache_bytes).hexdigest(),
            "bytes": len(cache_bytes),
            "candidate_or_corpus_authority": False,
            "retention": r114_profile["quantization_and_emission"][
                "warm_start_retention"
            ],
        }
        if cache_bytes is not None
        else {
            "status": "NOT_EMITTED",
            "file_name": None,
            "sha256": None,
            "bytes": 0,
            "candidate_or_corpus_authority": False,
        }
    )
    gate = execution_profile["result_transitions"][
        "exact_pass" if passed else "exact_fail"
    ]
    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": solve_result["status"],
        "claim": execution_profile["claim"],
        "gate_decision": gate,
        "scope": execution_profile["scope"],
        "source_gate": {
            "r114_status": r114["status"],
            "r114_gate_decision": r114["gate_decision"],
            "r114_report_sha256": r114["report_sha256"],
        },
        "execution_contract": execution_profile["execution_contract"],
        "descriptor_lineage": descriptor_lineage,
        "trajectory_lineage": trajectory_lineage,
        "solver_result": solve_result,
        "solver_private_warm_start_cache": cache_record,
        "resource_usage": {
            "peak_resident_memory_bytes": peak_memory_bytes,
            "maximum_resident_memory_bytes": r114_profile["execution_budget"][
                "maximum_resident_memory_bytes"
            ],
            "memory_budget_satisfied": memory_ok,
            "wall_clock_budget_satisfied": solve_result["resource_budget_satisfied"],
        },
        "validation_results": validations,
        "identities": {
            "execution_profile_sha256": sha256(execution_profile_path),
            "r114_report_file_sha256": sha256(r114_report_path),
            "r114_profile_sha256": sha256(r114_profile_path),
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
            "execution_module_sha256": sha256(Path(__file__).resolve()),
            "solver_module_sha256": sha256(
                Path(solve_quantization_aware_kto.__code__.co_filename).resolve()
            ),
        },
        "bounded_acceptance": execution_profile["bounded_acceptance"],
        "kto_execution_formulations": 0,
        "model_identity_preflights": 0,
        "solver_runs": 1,
        "kto_solves": 1,
        "qp_solves": solve_result["qp_solves"],
        "in_memory_emitted_iterates": solve_result["exact_emission_audits"],
        "solver_private_warm_start_caches": int(cache_bytes is not None),
        "candidate_target_constructions": 0,
        "candidate_artifacts_built": 0,
        "offline_candidate_evaluations": solve_result["exact_emission_audits"],
        "inverse_dynamics_solves": 0,
        "kinodynamic_solves": 0,
        "physx_scene_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["report_sha256"] = hashlib.sha256(canonical_json(report)).hexdigest()
    return report, cache_bytes


def validate_r114(
    *,
    execution_profile: Mapping[str, Any],
    report_path: Path,
    profile_path: Path,
) -> dict[str, Any]:
    expected = execution_profile["source"]["r114"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R115 R114 source identity differs")
    source_profile = json.loads(profile_path.read_bytes())
    _validate_r114_profile(source_profile)
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    identities = report.get("identities", {})
    bounded = report.get("bounded_acceptance", {})
    if (
        expected.get("formulation_revision") != 2
        or embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("check") != R114_CHECK_ID
        or report.get("formulation_id") != R114_FORMULATION_ID
        or report.get("formulation_revision") != 2
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R115_SINGLE_BOUNDED_QUANTIZATION_AWARE_KTO_EXECUTION_ONLY"
        or identities.get("formulation_module_sha256")
        != expected["formulation_module_sha256"]
        or identities.get("tool_sha256") != expected["tool_sha256"]
        or bounded.get("r115_quantization_aware_kto_execution")
        != "AUTHORIZED_ONE_IN_MEMORY_SOLVE_ONLY"
        or bounded.get("additional_kto_solve") != "NOT_AUTHORIZED"
        or report.get("repository", {}).get("dirty") is not False
        or report.get("kto_execution_formulations") != 1
        or any(
            int(report.get(key, -1)) != 0
            for key in (
                "solver_runs",
                "kto_solves",
                "candidate_artifacts_built",
                "solver_private_warm_start_caches",
                "physx_scene_runs",
                "optimizer_steps",
                "training_runs",
            )
        )
    ):
        raise ValueError("R115 R114 contract differs")
    return report


def _serialize_cache(arrays: Mapping[str, NDArray[Any]] | None) -> bytes:
    if arrays is None:
        raise ValueError("R115 PASS cache is absent")
    output = io.BytesIO()
    np.savez_compressed(output, **arrays)
    return output.getvalue()


def _raw_npz_array(path: Path, name: str) -> NDArray[Any]:
    with np.load(path, allow_pickle=False) as source:
        return np.asarray(source[name]).copy()


def _peak_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected_ids = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected_ids or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R115 validation result differs")
    return normalized


def _validate_environment(profile: Mapping[str, Any]) -> None:
    expected = profile["execution_contract"]["environment"]
    if any(os.environ.get(name) != value for name, value in expected.items()):
        raise ValueError("R115 deterministic process environment differs")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    commit = repository.get("commit")
    if (
        not isinstance(commit, str)
        or len(commit) != 40
        or any(character not in "0123456789abcdef" for character in commit)
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R115 requires a clean repository")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    contract = profile.get("execution_contract", {})
    transitions = profile.get("result_transitions", {})
    bounded = profile.get("bounded_acceptance", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleExecution"
        or profile.get("claim") != "SingleBoundedQuantizationAwareKtoExecutionOnly"
        or scope.get("run_id") != "R115"
        or scope.get("clip_id") != "cmu16-walk-nominal-b"
        or scope.get("frame_count") != 801
        or scope.get("decision_scalar_count") != 69687
        or scope.get("kto_process_count") != 1
        or scope.get("kto_solve_count") != 1
        or scope.get("physx_scene_runs") != 0
        or scope.get("all_17") is not False
        or scope.get("training") is not False
        or contract.get("candidate_artifact") != "FORBIDDEN"
        or contract.get("root_yaw_branch_resolution")
        != "use the R114 revision-2 nearest continuous V9 branch; no independent unwrap, repair or branch choice"
        or contract.get("native_scene") != "FORBIDDEN"
        or contract.get("learned_optimization") != "FORBIDDEN"
        or transitions.get("exact_pass")
        != "PERMIT_SEPARATE_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_FORMULATION_ONLY"
        or transitions.get("exact_fail") != "STOP_AND_RESEARCH"
        or transitions.get("invalid") != "STOP_INVALID_EVIDENCE"
        or bounded.get("r116_inverse_dynamics_execution_formulation")
        != "AUTHORIZED_ONLY_ON_R115_EXACT_PASS"
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
        or tuple(row.get("id") for row in profile.get("validation_commands", ()))
        != ("ruff_check", "ruff_format", "lab_full", "motor", "host_check")
    ):
        raise ValueError("R115 execution profile differs")
