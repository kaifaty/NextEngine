from __future__ import annotations

import hashlib
import json
from copy import deepcopy
from pathlib import Path
from typing import Any, Mapping, Sequence

from next_lab.contact_manifold_physx import load_contact_prototype_cases


BUNDLE_ID = "nextengine.humanoid-contact-counterfactual-bundle.v1"
BUNDLE_CHECK = "TRAIN-4-CONTACT-MANIFOLD-COUNTERFACTUAL-BUNDLE"
_ROLES = ("contact-reserve", "emitted-acceleration")
_R95_ORDINALS = (2, 10)
_SOURCE_ORDINALS = (25, 7967)


def build_counterfactual_bundle(
    *,
    profile_path: Path,
    source_audit_path: Path,
    source_manifest_paths: Sequence[Path],
    staging_directory: Path,
    tool_path: Path,
    repository: Mapping[str, Any],
) -> dict[str, Any]:
    """Copy two independent R96 cases into one hash-closed R97 input."""

    profile_path = profile_path.resolve()
    source_audit_path = source_audit_path.resolve()
    manifest_paths = tuple(path.resolve() for path in source_manifest_paths)
    tool_path = tool_path.resolve()
    if (
        not profile_path.is_file()
        or not source_audit_path.is_file()
        or not tool_path.is_file()
        or len(manifest_paths) != 2
        or any(not path.is_file() for path in manifest_paths)
        or not staging_directory.is_dir()
    ):
        raise FileNotFoundError("counterfactual bundle input is absent")
    profile = json.loads(profile_path.read_bytes())
    specifications = profile.get("ordered_counterfactuals")
    if (
        profile.get("schema_version") != 1
        or profile.get("bundle_id") != BUNDLE_ID
        or profile.get("status") != "FrozenResearchOnly"
        or profile.get("source_audit_sha256") != sha256(source_audit_path)
        or not isinstance(specifications, list)
        or len(specifications) != len(manifest_paths)
        or tuple(item.get("role") for item in specifications) != _ROLES
        or tuple(item.get("r95_case_ordinal") for item in specifications)
        != _R95_ORDINALS
        or tuple(item.get("source_case_ordinal") for item in specifications)
        != _SOURCE_ORDINALS
        or profile.get("execution")
        != {
            "fresh_scene_case_count": 2,
            "indexed_partial_reset_enabled": False,
            "all_17_enabled": False,
            "optimizer_steps": 0,
            "training_runs": 0,
        }
    ):
        raise ValueError("counterfactual bundle profile is invalid")

    artifact_directory = staging_directory / "cases"
    artifact_directory.mkdir()
    case_records = []
    source_records = []
    shared_identities: dict[str, str] | None = None
    for bundle_ordinal, (specification, manifest_path) in enumerate(
        zip(specifications, manifest_paths, strict=True)
    ):
        manifest, _, cases = load_contact_prototype_cases(
            manifest_path=manifest_path,
            source_audit_path=source_audit_path,
        )
        if (
            sha256(manifest_path) != specification.get("manifest_file_sha256")
            or manifest.get("manifest_sha256")
            != specification.get("manifest_sha256")
            or manifest.get("prototype_id") != specification.get("prototype_id")
            or manifest.get("identities", {}).get("prototype_profile_sha256")
            != specification.get("prototype_profile_sha256")
            or len(cases) != 1
            or cases[0].source_case_ordinal
            != int(specification["source_case_ordinal"])
            or cases[0].artifact_sha256
            != specification.get("case_artifact_sha256")
            or manifest.get("scope", {}).get("case_scope") != "discriminator"
            or manifest.get("scope", {}).get("case_count") != 1
            or len(manifest.get("complete_clips", ())) != 1
            or manifest["complete_clips"][0].get("solve_count") != 1
            or manifest["complete_clips"][0]
            .get("projection_diagnostics", {})
            .get("status")
            != "PASS"
            or manifest["complete_clips"][0]
            .get("projection_diagnostics", {})
            .get("contact_point_deletion_count")
            != 0
            or manifest["cases"][0].get("exact_complete_clip_slice_status")
            != "PASS"
        ):
            raise ValueError(
                f"counterfactual source {specification.get('role')} differs"
            )
        case = cases[0]
        manifest_identities = manifest["identities"]
        current_shared = {
            key: str(manifest_identities[key])
            for key in (
                "corpus_manifest_sha256",
                "corpus_manifest_file_sha256",
                "descriptor_sha256",
            )
        }
        if shared_identities is None:
            shared_identities = current_shared
        elif shared_identities != current_shared:
            raise ValueError("counterfactual source identities disagree")
        payload = case.artifact_path.read_bytes()
        artifact_name = (
            f"{bundle_ordinal:02d}-{specification['role']}--"
            f"{case.artifact_path.name}"
        )
        artifact_path = artifact_directory / artifact_name
        artifact_path.write_bytes(payload)
        record = deepcopy(manifest["cases"][0])
        record["artifact"] = {
            "relative_path": str(artifact_path.relative_to(staging_directory)),
            "sha256": hashlib.sha256(payload).hexdigest(),
            "bytes": len(payload),
        }
        record["counterfactual_role"] = specification["role"]
        record["r95_case_ordinal"] = specification["r95_case_ordinal"]
        record["source_counterfactual_manifest_sha256"] = manifest[
            "manifest_sha256"
        ]
        case_records.append(record)
        source_records.append(
            {
                "role": specification["role"],
                "r95_case_ordinal": specification["r95_case_ordinal"],
                "source_case_ordinal": case.source_case_ordinal,
                "prototype_id": manifest["prototype_id"],
                "manifest_sha256": manifest["manifest_sha256"],
                "manifest_file_sha256": sha256(manifest_path),
                "prototype_profile_sha256": manifest["identities"][
                    "prototype_profile_sha256"
                ],
                "case_artifact_sha256": case.artifact_sha256,
                "bundle_artifact_sha256": record["artifact"]["sha256"],
            }
        )

    report = {
        "schema_version": 1,
        "check": BUNDLE_CHECK,
        "status": "PASS",
        "claim": "TwoIndependentCounterfactualsResearchOnly",
        "gate_decision": "PERMIT_EXACTLY_TWO_FRESH_R97_CASES_ONLY",
        "prototype_id": BUNDLE_ID,
        "scope": {
            "case_count": 2,
            "failure_case_count": 0,
            "control_case_count": 2,
            "case_scope": "two-independent-counterfactuals",
            "projection_domain": "clip-global",
            "clip_ids": [record["clip_id"] for record in case_records],
            "ordered_r95_case_ordinals": list(_R95_ORDINALS),
            "ordered_source_case_ordinals": list(_SOURCE_ORDINALS),
        },
        "identities": {
            "source_audit_sha256": sha256(source_audit_path),
            "prototype_profile_sha256": sha256(profile_path),
            **(shared_identities or {}),
            "source_counterfactuals": source_records,
            "tool_sha256": sha256(tool_path),
            "bundle_module_sha256": sha256(Path(__file__).resolve()),
        },
        "cases": case_records,
        "fresh_scene_runs": 0,
        "physx_runs": 0,
        "optimizer_steps": 0,
        "training_runs": 0,
        "learned_policy_claim": False,
        "repository": dict(repository),
    }
    report["manifest_sha256"] = hashlib.sha256(
        canonical_json(report)
    ).hexdigest()
    return report


def canonical_json(value: Any) -> bytes:
    return json.dumps(
        value, sort_keys=True, separators=(",", ":"), ensure_ascii=True
    ).encode("utf-8")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
