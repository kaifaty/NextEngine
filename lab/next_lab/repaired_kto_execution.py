from __future__ import annotations

import hashlib
import io
import json
import os
import platform
import resource
from collections.abc import Mapping, Sequence
from pathlib import Path
from typing import Any

import numpy as np
import osqp
import scipy
from numpy.typing import NDArray

from next_lab.contact_target_knot_formulation import canonical_json, sha256
from next_lab.kto_linearization_repair_conformance import (
    _validate_profile as _validate_r118_profile,
)
from next_lab.quantization_aware_kto_formulation import (
    _load_complete_arrays,
    _load_prior_arrays,
    audit_descriptor_lineage,
    audit_source_identity,
    audit_trajectory_lineage,
)
from next_lab.repaired_kto_execution_formulation import (
    _validate_profile as _validate_r119_profile,
)
from next_lab.repaired_kto_solver import solve_repaired_kto

EXECUTION_ID = "nextengine.humanoid-repaired-kto-execution.v1"
CHECK_ID = "TRAIN-4-REPAIRED-KTO-EXECUTION"
CACHE_FILE_NAME = "solver-private-r120-repaired-kto.npz"


def execute_repaired_kto(
    *,
    execution_profile_path: Path,
    r119_report_path: Path,
    r119_profile_path: Path,
    r118_profile_path: Path,
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
    """Execute the sole R120 process after every source preflight passes."""

    direct_paths = tuple(
        path.resolve()
        for path in (
            execution_profile_path,
            r119_report_path,
            r119_profile_path,
            r118_profile_path,
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
        r119_report_path,
        r119_profile_path,
        r118_profile_path,
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
        raise FileNotFoundError("R120 execution input is absent")

    profile = json.loads(execution_profile_path.read_bytes())
    _validate_profile(profile)
    _validate_environment(profile)
    _validate_runtime(profile)
    _validate_repository(repository)
    r119_profile = json.loads(r119_profile_path.read_bytes())
    _validate_r119_profile(r119_profile)
    if any(
        r119_profile["execution_budget"].get(key) != value
        for key, value in profile["execution_budget"].items()
    ):
        raise ValueError("R120 execution budget differs from R119")
    if profile["source"]["r114"] != {
        key: r119_profile["source"]["r114"][key]
        for key in ("report_sha256", "report_file_sha256", "profile_sha256")
    }:
        raise ValueError("R120 inherited R114 identity differs from R119")
    r119 = validate_r119(
        execution_profile=profile,
        report_path=r119_report_path,
        profile_path=r119_profile_path,
    )
    r118_profile = json.loads(r118_profile_path.read_bytes())
    _validate_r118_profile(r118_profile)
    if (
        sha256(r118_profile_path) != profile["source"]["r118_profile_sha256"]
        or profile["source"]["r118_profile_sha256"]
        != r119_profile["source"]["r118"]["profile_sha256"]
    ):
        raise ValueError("R120 R118 profile identity differs")
    r114_profile = json.loads(r114_profile_path.read_bytes())
    _validate_bound_implementations(profile, tool_path)
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
    if (
        sha256(r114_report_path) != profile["source"]["r114"]["report_file_sha256"]
        or sha256(r114_profile_path) != profile["source"]["r114"]["profile_sha256"]
    ):
        raise ValueError("R120 R114 source identity differs")
    validations = _validate_results(profile, validation_results)
    v9_arrays, _ = _load_complete_arrays(v9_complete_clip_path)
    v7_arrays, _ = _load_prior_arrays(v7_case_path)
    v9_arrays["metadata_json_utf8"] = _raw_npz_array(
        v9_complete_clip_path, "metadata_json_utf8"
    )
    v7_arrays["metadata_json_utf8"] = _raw_npz_array(v7_case_path, "metadata_json_utf8")
    descriptor = descriptor_lineage.pop("current_descriptor")
    v9_profile = trajectory_lineage.pop("v9_profile")
    solve_result, cache_arrays = solve_repaired_kto(
        descriptor=descriptor,
        v9_arrays=v9_arrays,
        v7_arrays=v7_arrays,
        v9_profile=v9_profile,
        r114_profile=r114_profile,
        r118_profile=r118_profile,
        r119_profile=r119_profile,
    )
    peak_memory = _peak_memory_bytes()
    memory_ok = peak_memory <= int(
        profile["execution_budget"]["maximum_resident_memory_bytes"]
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
    cache_record = _cache_record(cache_bytes, r119_profile)
    transition = {
        "PASS": "exact_pass",
        "FAIL": "exact_fail",
        "INVALID": "invalid",
    }[solve_result["status"]]
    report: dict[str, Any] = {
        "schema_version": 1,
        "check": CHECK_ID,
        "execution_id": EXECUTION_ID,
        "status": solve_result["status"],
        "claim": profile["claim"],
        "gate_decision": profile["result_transitions"][transition],
        "scope": profile["scope"],
        "source_gate": {
            "r119_status": r119["status"],
            "r119_gate_decision": r119["gate_decision"],
            "r119_report_sha256": r119["report_sha256"],
        },
        "execution_contract": profile["execution_contract"],
        "descriptor_lineage": descriptor_lineage,
        "trajectory_lineage": trajectory_lineage,
        "solver_result": solve_result,
        "solver_private_warm_start_cache": cache_record,
        "resource_usage": {
            "peak_resident_memory_bytes": peak_memory,
            "maximum_resident_memory_bytes": profile["execution_budget"][
                "maximum_resident_memory_bytes"
            ],
            "memory_budget_satisfied": memory_ok,
            "wall_clock_budget_satisfied": solve_result["resource_budget_satisfied"],
        },
        "validation_results": validations,
        "identities": {
            "execution_profile_sha256": sha256(execution_profile_path),
            "r119_report_file_sha256": sha256(r119_report_path),
            "r119_profile_sha256": sha256(r119_profile_path),
            "r118_profile_sha256": sha256(r118_profile_path),
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
                Path(solve_repaired_kto.__code__.co_filename).resolve()
            ),
        },
        "bounded_acceptance": profile["bounded_acceptance"],
        "kto_processes": 1,
        "solver_runs": int(solve_result["kto_solves"]),
        "kto_solves": int(solve_result["kto_solves"]),
        "qp_solves": int(solve_result["qp_solves"]),
        "in_memory_emitted_iterates": int(solve_result["exact_emission_audits"]),
        "solver_private_warm_start_caches": int(cache_bytes is not None),
        "candidate_artifacts_built": 0,
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


def validate_r119(
    *,
    execution_profile: Mapping[str, Any],
    report_path: Path,
    profile_path: Path,
) -> dict[str, Any]:
    expected = execution_profile["source"]["r119"]
    if (
        sha256(report_path) != expected["report_file_sha256"]
        or sha256(profile_path) != expected["profile_sha256"]
    ):
        raise ValueError("R120 R119 source file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    if (
        embedded != expected["report_sha256"]
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("check") != "TRAIN-4-REPAIRED-KTO-EXECUTION-FORMULATION"
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R120_SINGLE_BOUNDED_REPAIRED_KTO_EXECUTION_ONLY"
        or report.get("repository", {}).get("commit") != expected["repository_commit"]
        or report.get("repository", {}).get("dirty") is not False
        or report.get("solver_runs") != 0
        or report.get("kto_solves") != 0
        or report.get("qp_solves") != 0
        or report.get("candidate_artifacts_built") != 0
        or report.get("physx_scene_runs") != 0
        or report.get("training_runs") != 0
    ):
        raise ValueError("R120 R119 source contract differs")
    return report


def _validate_bound_implementations(
    profile: Mapping[str, Any], tool_path: Path
) -> None:
    module = Path(__file__).resolve().parent
    lab = module.parent
    expected = profile["source"]["implementation"]
    paths = {
        "inherited_r115_solver_sha256": module / "quantization_aware_kto_solver.py",
        "r118_conformance_module_sha256": module
        / "kto_linearization_repair_conformance.py",
        "r119_formulation_module_sha256": module
        / "repaired_kto_execution_formulation.py",
        "r120_solver_module_sha256": module / "repaired_kto_solver.py",
        "r120_execution_module_sha256": Path(__file__).resolve(),
        "r120_tool_sha256": tool_path,
    }
    for key, path in paths.items():
        if sha256(path) != expected[key]:
            raise ValueError(f"R120 bound implementation {key} differs")
    if (
        sha256(lab / "scripts/formulate_repaired_kto_execution.py")
        != expected["r119_tool_sha256"]
    ):
        raise ValueError("R120 bound R119 tool differs")


def _validate_profile(profile: Mapping[str, Any]) -> None:
    source = profile.get("source", {})
    scope = profile.get("scope", {})
    contract = profile.get("execution_contract", {})
    transitions = profile.get("result_transitions", {})
    bounded = profile.get("bounded_acceptance", {})
    budget = profile.get("execution_budget", {})
    implementation = profile.get("source", {}).get("implementation", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("execution_id") != EXECUTION_ID
        or profile.get("status") != "FrozenSingleExecution"
        or profile.get("claim") != "SingleBoundedRepairedExactKernelKtoExecutionOnly"
        or source.get("r119")
        != {
            "report_sha256": "ac38e3f0a9dfb5e900373bc5a4908168a49f3c3b799fb431bd6e5c1306a4dd1b",
            "report_file_sha256": "e233f9dd60ba8056e55b132167e5dbd6e952781fb15bece56cbb23e62b0c1882",
            "profile_sha256": "3b48c618cffc5494601a40ac15b04df0ffba2a1b7d6aadbdfd4309b6c167dcd1",
            "repository_commit": "322896b1c67376337eacfb1eab9335ef66b12776",
        }
        or source.get("r118_profile_sha256")
        != "d97340714757c1ead27b9f571332f926690104a89fed8a2d3a6ef357c9de88f3"
        or source.get("r114")
        != {
            "report_sha256": "7a735320509a303f9feacba79087f2042d451d526b57585d4fe4041603b46ae4",
            "report_file_sha256": "2666a275180b222c014eea91051ff4d3ebdb16a5740eb742ed2cca3a83b3d958",
            "profile_sha256": "8e26b84de2a25e07d19cd93400bc8c04a6bc4840fb9a4d8eb6ab88237db59a43",
        }
        or scope
        != {
            "run_id": "R120",
            "clip_id": "cmu16-walk-nominal-b",
            "frame_count": 801,
            "decision_scalar_count": 69687,
            "kto_process_count": 1,
            "kto_solve_count": 1,
            "maximum_qp_solves": 12,
            "physx_scene_runs": 0,
            "all_17": False,
            "training": False,
        }
        or contract.get("initial_anchor") != "byte-exact V9; no R115 execution state"
        or contract.get("candidate_artifact") != "FORBIDDEN"
        or contract.get("native_scene") != "FORBIDDEN"
        or contract.get("learned_optimization") != "FORBIDDEN"
        or budget.get("runtime_versions")
        != {
            "python": "3.12",
            "numpy": "2.5.2",
            "scipy": "1.18.0",
            "osqp": "1.1.3",
        }
        or budget.get("thread_count") != 1
        or budget.get("kto_process_count") != 1
        or budget.get("kto_solve_count") != 1
        or budget.get("maximum_major_iterations") != 12
        or budget.get("maximum_qp_solves") != 12
        or budget.get("maximum_exact_emission_audits") != 72
        or tuple(budget.get("line_search_fractions", ()))
        != ("1", "1/2", "1/4", "1/8", "1/16", "1/32")
        or budget.get("maximum_wall_clock_seconds") != 14400
        or budget.get("maximum_resident_memory_bytes") != 17179869184
        or budget.get("randomized_restart_count") != 0
        or budget.get("resume_or_warm_restart") != "FORBIDDEN"
        or budget.get("manual_intervention") != "FORBIDDEN"
        or set(implementation)
        != {
            "inherited_r115_solver_sha256",
            "r118_conformance_module_sha256",
            "r119_formulation_module_sha256",
            "r119_tool_sha256",
            "r120_solver_module_sha256",
            "r120_execution_module_sha256",
            "r120_tool_sha256",
        }
        or not all(_is_sha256(value) for value in implementation.values())
        or transitions.get("exact_pass")
        != "PERMIT_SEPARATE_REPORT_ONLY_R121_FIXED_PD_INVERSE_DYNAMICS_EXECUTION_FORMULATION_ONLY"
        or transitions.get("exact_fail") != "STOP_AND_RESEARCH"
        or transitions.get("invalid") != "STOP_INVALID_EVIDENCE"
        or bounded.get("r121_inverse_dynamics_formulation")
        != "AUTHORIZED_ONLY_ON_R120_EXACT_PASS"
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
        != (
            "ruff_check",
            "ruff_format",
            "focused_r120",
            "lab_full",
            "motor",
            "host_check",
        )
    ):
        raise ValueError("R120 execution profile differs")


def _is_sha256(value: Any) -> bool:
    return bool(
        isinstance(value, str)
        and len(value) == 64
        and all(character in "0123456789abcdef" for character in value)
    )


def _validate_environment(profile: Mapping[str, Any]) -> None:
    expected = profile["execution_contract"]["environment"]
    if any(os.environ.get(name) != value for name, value in expected.items()):
        raise ValueError("R120 deterministic process environment differs")


def _validate_runtime(profile: Mapping[str, Any]) -> None:
    expected = profile["execution_budget"]["runtime_versions"]
    actual = {
        "python": ".".join(platform.python_version_tuple()[:2]),
        "numpy": np.__version__,
        "scipy": scipy.__version__,
        "osqp": osqp.__version__,
    }
    if actual != expected:
        raise ValueError(f"R120 runtime version differs: {actual}")


def _validate_repository(repository: Mapping[str, Any]) -> None:
    commit = repository.get("commit")
    if (
        not isinstance(commit, str)
        or len(commit) != 40
        or any(character not in "0123456789abcdef" for character in commit)
        or repository.get("dirty") is not False
        or repository.get("dirty_paths") != []
    ):
        raise ValueError("R120 execution requires a clean repository")


def _validate_results(
    profile: Mapping[str, Any], results: Sequence[Mapping[str, str]]
) -> list[dict[str, str]]:
    expected = [row["id"] for row in profile["validation_commands"]]
    normalized = [dict(row) for row in results]
    if [row.get("id") for row in normalized] != expected or any(
        row.get("status") != "PASS" for row in normalized
    ):
        raise ValueError("R120 validation result differs")
    return normalized


def _serialize_cache(arrays: Mapping[str, NDArray[Any]] | None) -> bytes:
    if arrays is None:
        raise ValueError("R120 PASS cache is absent")
    output = io.BytesIO()
    np.savez_compressed(output, **arrays)
    return output.getvalue()


def _cache_record(
    cache_bytes: bytes | None, r119_profile: Mapping[str, Any]
) -> dict[str, Any]:
    if cache_bytes is None:
        return {
            "status": "NOT_EMITTED",
            "file_name": None,
            "sha256": None,
            "bytes": 0,
            "candidate_or_corpus_authority": False,
        }
    return {
        "status": "EMITTED_TRANSIENT_SOLVER_PRIVATE",
        "file_name": CACHE_FILE_NAME,
        "sha256": hashlib.sha256(cache_bytes).hexdigest(),
        "bytes": len(cache_bytes),
        "candidate_or_corpus_authority": False,
        "retention": r119_profile["required_diagnostics"]["outputs"]["exact_pass"],
    }


def _raw_npz_array(path: Path, name: str) -> NDArray[Any]:
    with np.load(path, allow_pickle=False) as source:
        return np.asarray(source[name]).copy()


def _peak_memory_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024
