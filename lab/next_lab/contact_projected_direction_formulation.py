from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any, Mapping

from next_lab.contact_feasible_direction_audit import (
    AUDIT_ID as R105_AUDIT_ID,
    CHECK_ID as R105_CHECK_ID,
)
from next_lab.contact_target_knot_formulation import canonical_json, sha256


FORMULATION_ID = "nextengine.humanoid-contact-projected-direction-formulation.v1"
CHECK_ID = "TRAIN-4-CONTACT-PROJECTED-DIRECTION-FORMULATION"
SELECTED_BASIS_ID = "anchor-offset-11"
SELECTED_COEFFICIENTS = (0, 0, 10_000)


def build_projected_direction_formulation(
    *,
    profile_path: Path,
    r105_report_path: Path,
    r105_profile_path: Path,
    v9_profile_path: Path,
    descriptor_path: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Freeze R106 reconstruction and future exact-audit semantics only."""

    paths = tuple(
        path.resolve()
        for path in (
            profile_path,
            r105_report_path,
            r105_profile_path,
            v9_profile_path,
            descriptor_path,
            tool_path,
        )
    )
    (
        profile_path,
        r105_report_path,
        r105_profile_path,
        v9_profile_path,
        descriptor_path,
        tool_path,
    ) = paths
    if any(not path.is_file() for path in paths):
        raise FileNotFoundError("projected-direction formulation input is absent")

    profile = json.loads(profile_path.read_bytes())
    _validate_profile(profile)
    r105 = validate_r105(
        report_path=r105_report_path,
        expected=profile["source"]["r105"],
    )
    if (
        sha256(r105_profile_path)
        != profile["source"]["r105"]["profile_sha256"]
        or sha256(v9_profile_path)
        != profile["source"]["v9_profile_sha256"]
        or sha256(descriptor_path)
        != profile["source"]["descriptor_sha256"]
    ):
        raise ValueError("projected-direction formulation identity differs")
    r105_profile = json.loads(r105_profile_path.read_bytes())
    if (
        r105_profile.get("audit_id") != R105_AUDIT_ID
        or r105_profile.get("analysis") != r105["analysis_contract"]
    ):
        raise ValueError("R105 analysis profile differs")

    selected = next(
        row
        for row in r105["basis_results"]
        if row["basis_id"] == SELECTED_BASIS_ID
    )
    report = {
        "schema_version": 1,
        "check": CHECK_ID,
        "status": "COMPLETE",
        "claim": profile["claim"],
        "gate_decision": profile["decision"]["complete"],
        "formulation_id": FORMULATION_ID,
        "scope": profile["scope"],
        "identities": {
            "profile_sha256": sha256(profile_path),
            "r105_report_sha256": r105["report_sha256"],
            "r105_report_file_sha256": sha256(r105_report_path),
            "r105_profile_sha256": sha256(r105_profile_path),
            "r105_audit_module_sha256": r105["identities"][
                "audit_module_sha256"
            ],
            "r105_tool_sha256": r105["identities"]["tool_sha256"],
            "v9_profile_sha256": sha256(v9_profile_path),
            "descriptor_sha256": sha256(descriptor_path),
            "tool_sha256": sha256(tool_path),
            "formulation_module_sha256": sha256(Path(__file__).resolve()),
        },
        "source_lineage": {
            "r103_report_sha256": r105["identities"]["r103_report_sha256"],
            "r104_report_sha256": r105["identities"]["r104_report_sha256"],
            "v7_manifest_sha256": r105["identities"]["v7_manifest_sha256"],
            "v9_manifest_sha256": r105["identities"]["v9_manifest_sha256"],
            "v9_complete_clip_artifact_sha256": r105["identities"][
                "v9_complete_clip_artifact_sha256"
            ],
        },
        "selected_direction": {
            "basis_id": SELECTED_BASIS_ID,
            "knot_coefficients_basis_points": list(SELECTED_COEFFICIENTS),
            "selection_rule": profile["selection"]["rule"],
            "r105_projection_facts": selected["projection"],
            "r105_raw_direction_facts": selected["raw_direction"],
            "r105_projection_maximum_normalized_constraint_violation": (
                selected[
                    "maximum_projected_normalized_constraint_violation"
                ]
            ),
        },
        "reconstruction_contract": profile["reconstruction"],
        "quantization_contract": profile["quantization"],
        "future_exact_offline_audit": profile["future_exact_offline_audit"],
        "failure_disposition": profile["failure_disposition"],
        "bounded_acceptance": {
            "r107_exact_offline_projected_direction_audit": "AUTHORIZED",
            "candidate_target_construction": "NOT_AUTHORIZED",
            "candidate_artifact": "NOT_AUTHORIZED",
            "candidate_search": "NOT_AUTHORIZED",
            "physx": "NOT_AUTHORIZED",
            "all_17": "NOT_AUTHORIZED",
            "full_v19": "NOT_AUTHORIZED",
            "training": "NOT_AUTHORIZED",
        },
        "projection_qp_solves": 0,
        "candidate_target_constructions": 0,
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


def validate_r105(
    *, report_path: Path, expected: Mapping[str, Any]
) -> dict[str, Any]:
    if sha256(report_path) != expected.get("report_file_sha256"):
        raise ValueError("R105 report file identity differs")
    report = json.loads(report_path.read_bytes())
    embedded = report.get("report_sha256")
    without_hash = dict(report)
    without_hash.pop("report_sha256", None)
    rows = {row.get("basis_id"): row for row in report.get("basis_results", ())}
    selected = rows.get(SELECTED_BASIS_ID, {})
    projection = selected.get("projection", {})
    linearization = report.get("linearization", {})
    binding = linearization.get("binding_rows", {})
    summary = report.get("summary", {})
    bounded = report.get("bounded_acceptance", {})
    if (
        report.get("check") != R105_CHECK_ID
        or report.get("audit_id") != R105_AUDIT_ID
        or report.get("status") != "COMPLETE"
        or report.get("gate_decision")
        != "PERMIT_R106_EXACT_OFFLINE_PROJECTED_DIRECTION_FORMULATION_ONLY"
        or embedded != expected.get("report_sha256")
        or hashlib.sha256(canonical_json(without_hash)).hexdigest() != embedded
        or report.get("identities", {}).get("profile_sha256")
        != expected.get("profile_sha256")
        or report.get("repository", {}).get("dirty") is not False
        or set(rows) != {
            "anchor-offset-02",
            "anchor-offset-06",
            SELECTED_BASIS_ID,
        }
        or rows["anchor-offset-02"].get("useful_projection") is not False
        or rows["anchor-offset-06"].get("useful_projection") is not False
        or selected.get("useful_projection") is not True
        or tuple(selected.get("knot_coefficients_basis_points", ()))
        != SELECTED_COEFFICIENTS
        or projection.get("anchor_component_basis_points") != 9722
        or projection.get("cosine_similarity_basis_points") != 9872
        or projection.get("quantized_nonzero_joint_cell_count") != 45
        or projection.get(
            "quantized_maximum_absolute_joint_correction_microradians"
        )
        != 5688
        or projection.get("quantized_maximum_root_correction_micrometres")
        != 41
        or summary.get("basis_count") != 3
        or summary.get("useful_projection_count") != 1
        or summary.get("useful_projection_ids") != [SELECTED_BASIS_ID]
        or linearization.get("local_support_variable_count") != 130
        or linearization.get("local_relevant_constraint_count") != 600
        or linearization.get("zero_direction_maximum_normalized_violation")
        != 0.0
        or linearization.get("complete_clip_proxy_acceptance_authority")
        is not False
        or binding.get("near_binding_row_count") != 69
        or binding.get("near_binding_rank") != 69
        or binding.get("local_nullity") != 61
        or bounded.get("r106_exact_offline_formulation") != "AUTHORIZED"
        or any(
            bounded.get(key) != "NOT_AUTHORIZED"
            for key in (
                "candidate_artifact",
                "exact_nonlinear_candidate_audit",
                "candidate_search",
                "physx",
                "all_17",
                "full_v19",
                "training",
            )
        )
        or report.get("basis_constructions") != 3
        or report.get("local_projection_qp_solves") != 3
        or any(
            int(report.get(key, -1)) != 0
            for key in (
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
        raise ValueError("R105 feasible-direction contract differs")
    return report


def _validate_profile(profile: Mapping[str, Any]) -> None:
    scope = profile.get("scope", {})
    selection = profile.get("selection", {})
    reconstruction = profile.get("reconstruction", {})
    quantization = profile.get("quantization", {})
    future = profile.get("future_exact_offline_audit", {})
    if (
        profile.get("schema_version") != 1
        or profile.get("formulation_id") != FORMULATION_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("claim")
        != "OptimizerFreeProjectedDirectionFormulationOnly"
        or scope.get("source_case_ordinal") != 7967
        or scope.get("frame_first") != 238
        or scope.get("frame_last") != 249
        or scope.get("selected_basis_count") != 1
        or selection.get("basis_id") != SELECTED_BASIS_ID
        or tuple(selection.get("knot_coefficients_basis_points", ()))
        != SELECTED_COEFFICIENTS
        or reconstruction.get("projection_qp_execution_in_r106") is not False
        or reconstruction.get("support_frame_first") != 240
        or reconstruction.get("support_frame_last") != 249
        or quantization.get("rounding") != "integer ties-to-even"
        or quantization.get("candidate_construction_in_r106") is not False
        or future.get("next_run_id") != "R107"
        or future.get("candidate_artifact_policy")
        != "construct in memory only; emit exact metrics report, not artifact"
        or profile.get("decision", {})
        != {
            "complete": "PERMIT_R107_EXACT_OFFLINE_PROJECTED_DIRECTION_AUDIT_ONLY"
        }
    ):
        raise ValueError("projected-direction formulation profile is invalid")
