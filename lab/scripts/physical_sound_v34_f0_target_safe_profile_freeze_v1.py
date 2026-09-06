#!/usr/bin/env python3
"""Freeze the V34 target-safe overlay without opening target/model values."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import shutil
import tempfile
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v34-f0-target-safe-spectral-recovery-profile.v1"
PROFILE_ID = "physical-sound-v34-f0-target-safe-spectral-recovery-v1"
PROFILE_SHA256 = "7be7e96a2899d0e4ef7aad668368669886d7f45adac34ec6676afbbe6b45e429"
BASELINE_COMMIT = "1740807640c580f3373a7b1ec4e1cf98fb42c83b"
CLAIM = (
    "TARGET_SAFE_SYNTHETIC_SPECTRAL_RECOVERY_PROFILE_ONLY / "
    "NO_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
OWNER_PATH = "lab/scripts/physical_sound_v34_f0_target_safe_profile_freeze_v1.py"
PROFILE_PATH = (
    "lab/profiles/physical-sound-v34-f0-target-safe-spectral-recovery.v1.json"
)
MAX_PROFILE_BYTES = 1_048_576
ZERO_ACCESS = {
    "development_target_rows": 0,
    "feature_rows_materialized": 0,
    "method_holdout_target_rows": 0,
    "model_parameters_initialized": 0,
    "network_requests": 0,
    "oracle_values_evaluated": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "train_target_rows": 0,
}


class F0FreezeError(RuntimeError):
    """The target-safe F0 freeze contract failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise F0FreezeError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise F0FreezeError("profile must not be a symlink")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise F0FreezeError("profile size outside bound")
    try:
        profile = json.loads(data)
    except json.JSONDecodeError as error:
        raise F0FreezeError(f"invalid profile JSON: {error}") from error
    if data != canonical_json(profile):
        raise F0FreezeError("profile is not canonical JSON")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise F0FreezeError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
    ):
        raise F0FreezeError("profile identity mismatch")
    authority = profile.get("authority", {})
    required = {
        "admission_authority": False,
        "authored_fallback_required": True,
        "c0_allowed_after_f0": True,
        "external_research_only": True,
        "model_allowed_after_f0": False,
        "network_allowed": False,
        "quality_authority": False,
        "real_material_authority": False,
        "real_signal_allowed": False,
        "runtime_authority": False,
        "synthetic_only": True,
        "t0_allowed_after_c0": True,
        "validator_release_authority": False,
    }
    if authority != required:
        raise F0FreezeError("profile authority drift")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    declarations = {
        **profile["parent"],
        "protocol": profile["protocol"],
        "research": profile["research"],
    }
    root = repository_root()
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, declared in sorted(declarations.items()):
        path = root / declared["path"]
        if path.is_symlink():
            raise F0FreezeError(f"dependency must not be a symlink: {dependency_id}")
        data = path.read_bytes()
        if sha256_bytes(data) != declared["sha256"]:
            raise F0FreezeError(f"dependency drift: {dependency_id}")
        result[dependency_id] = {
            "bytes": len(data),
            "path": declared["path"],
            "sha256": declared["sha256"],
        }
    return result


def load_declared_json(profile: dict[str, Any], dependency_id: str) -> dict[str, Any]:
    declared = profile["parent"][dependency_id]
    data = (repository_root() / declared["path"]).read_bytes()
    if sha256_bytes(data) != declared["sha256"]:
        raise F0FreezeError(f"dependency drift: {dependency_id}")
    return json.loads(data)


def section_sha256(value: Any) -> str:
    return sha256_bytes(canonical_json(value))


def validate_preserved_sections(
    profile: dict[str, Any], v33_profile: dict[str, Any]
) -> dict[str, str]:
    sections = {
        "controls": v33_profile["controls"],
        "corpus_counts": v33_profile["corpus"]["counts"],
        "corpus_families": v33_profile["corpus"]["families"],
        "corpus_role_plan": v33_profile["corpus"]["role_plan"],
        "features": v33_profile["features"],
        "gates": v33_profile["gates"],
        "model": v33_profile["model"],
        "resources": v33_profile["resources"],
        "training": v33_profile["training"],
    }
    actual = {name: section_sha256(value) for name, value in sorted(sections.items())}
    if actual != profile["preserved_sha256"]:
        raise F0FreezeError("preserved V33 section hash drift")
    return actual


def build_effective_profile(
    profile: dict[str, Any], v33_profile: dict[str, Any]
) -> dict[str, Any]:
    effective = copy.deepcopy(v33_profile)
    overlay = profile["corpus_overlay"]
    effective["corpus"]["contacts"] = copy.deepcopy(overlay["contacts"])
    effective["corpus"]["geometry_multiplier_cells"] = copy.deepcopy(
        overlay["geometry_multiplier_cells"]
    )
    effective["corpus"]["materials"] = copy.deepcopy(overlay["materials"])
    effective["oracle"] = copy.deepcopy(overlay["oracle"])
    effective["profile_id"] = PROFILE_ID
    effective["schema"] = PROFILE_SCHEMA
    return effective


def contact_pairs(corpus: dict[str, Any]) -> dict[str, tuple[tuple[str, str], ...]]:
    return {
        role: tuple(tuple(pair) for pair in pairs)
        for role, pairs in corpus["contacts"].items()
    }


def enumerate_role_cases(
    corpus: dict[str, Any], role: str
) -> tuple[set[tuple[int, int, int, tuple[str, str]]], dict[str, dict[str, int]]]:
    family_count = len(corpus["families"])
    material_count = len(corpus["materials"])
    contacts = contact_pairs(corpus)
    cases: set[tuple[int, int, int, tuple[str, str]]] = set()
    strata: dict[str, dict[str, int]] = {}
    for entry in corpus["role_plan"][role]:
        stratum_cases = {
            (family, material, cell, contact)
            for family in range(family_count)
            for material in range(material_count)
            for cell in entry["geometry_cells"]
            for contact in contacts[entry["contact_set"]]
        }
        if entry["stratum"] in strata or cases & stratum_cases:
            raise F0FreezeError(f"role stratum overlap: {role}/{entry['stratum']}")
        cases |= stratum_cases
        strata[entry["stratum"]] = {
            "cases": len(stratum_cases),
            "modal_rows": len(stratum_cases) * corpus["mode_count"],
            "role_local_geometry_groups": (
                family_count * material_count * len(set(entry["geometry_cells"]))
            ),
        }
    return cases, strata


def validate_freshness_counts_and_witness_plan(
    profile: dict[str, Any],
    effective: dict[str, Any],
    v32_profile: dict[str, Any],
    v33_profile: dict[str, Any],
) -> dict[str, Any]:
    corpus = effective["corpus"]
    contacts = contact_pairs(corpus)
    all_contacts = {pair for pairs in contacts.values() for pair in pairs}
    if sum(map(len, contacts.values())) != len(all_contacts):
        raise F0FreezeError("V34 contact sets overlap")
    for parent_name, parent in (("V32", v32_profile), ("V33", v33_profile)):
        parent_contacts = {
            tuple(pair)
            for pairs in (
                parent["corpus"]["contacts"].values()
                if isinstance(parent["corpus"]["contacts"], dict)
                else [parent["corpus"]["contacts"]]
            )
            for pair in pairs
        }
        if all_contacts & parent_contacts:
            raise F0FreezeError(f"{parent_name} contact reused")
        parent_cells = {
            tuple(cell) for cell in parent["corpus"]["geometry_multiplier_cells"]
        }
        if parent_cells & {tuple(cell) for cell in corpus["geometry_multiplier_cells"]}:
            raise F0FreezeError(f"{parent_name} geometry cell reused")
        parent_materials = {
            canonical_json(material) for material in parent["corpus"]["materials"]
        }
        if parent_materials & {
            canonical_json(material) for material in corpus["materials"]
        }:
            raise F0FreezeError(f"{parent_name} material reused")
    expected_contact_counts = {"development": 6, "method_holdout": 6, "train": 12}
    if {
        name: len(pairs) for name, pairs in contacts.items()
    } != expected_contact_counts:
        raise F0FreezeError("contact-set count drift")
    nodal_contacts = {
        name: [pair for pair in pairs if pair[0] == "0"]
        for name, pairs in contacts.items()
    }
    if any(len(pairs) != 1 for pairs in nodal_contacts.values()):
        raise F0FreezeError("each contact set requires exactly one declared node")

    expected_strata = {
        "development": {"contact-only", "geometry-only", "joint"},
        "method_holdout": {"contact-only", "geometry-only", "joint"},
        "train": {"train"},
    }
    role_cases: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    summaries: dict[str, Any] = {}
    for role in ("train", "development", "method_holdout"):
        cases, strata = enumerate_role_cases(corpus, role)
        if set(strata) != expected_strata[role]:
            raise F0FreezeError(f"stratum closure mismatch: {role}")
        expected = corpus["counts"][role]
        geometry_cells = {
            cell
            for entry in corpus["role_plan"][role]
            for cell in entry["geometry_cells"]
        }
        groups = (
            len(corpus["families"]) * len(corpus["materials"]) * len(geometry_cells)
        )
        if (
            len(cases) != expected["cases"]
            or len(cases) * corpus["mode_count"] != expected["modal_rows"]
            or groups != expected["role_local_geometry_groups"]
        ):
            raise F0FreezeError(f"role count algebra mismatch: {role}")
        if role != "train" and strata != expected["strata"]:
            raise F0FreezeError(f"stratum count algebra mismatch: {role}")
        role_cases[role] = cases
        summaries[role] = {"cases": len(cases), "modal_rows": len(cases) * 10}
    for left, right in (
        ("train", "development"),
        ("train", "method_holdout"),
        ("development", "method_holdout"),
    ):
        if role_cases[left] & role_cases[right]:
            raise F0FreezeError(f"case identity crosses roles: {left}/{right}")

    witness = profile["witness_contract"]
    if (
        witness["target_rows_allowed"] != 0
        or witness["evaluation_roles"] != ["development", "method_holdout"]
        or witness["strata"] != ["contact-only", "geometry-only", "joint"]
        or witness["exact_nodal_min_modal_rows_per_stratum"] != 180
        or set(witness["forbidden_dependencies"])
        != {
            "development_metrics",
            "model_owner",
            "optimizer",
            "oracle_targets",
            "protected_signal",
            "real_signal",
        }
    ):
        raise F0FreezeError("witness contract drift")
    predicted_nodal_rows: dict[str, dict[str, int]] = {}
    for role in witness["evaluation_roles"]:
        predicted_nodal_rows[role] = {}
        for entry in corpus["role_plan"][role]:
            nodal_count = len(nodal_contacts[entry["contact_set"]])
            predicted = (
                len(corpus["families"])
                * len(corpus["materials"])
                * len(entry["geometry_cells"])
                * nodal_count
                * corpus["mode_count"]
            )
            if predicted < witness["exact_nodal_min_modal_rows_per_stratum"]:
                raise F0FreezeError(f"nodal witness algebra shortfall: {role}")
            predicted_nodal_rows[role][entry["stratum"]] = predicted
    return {
        "contact_count_by_set": expected_contact_counts,
        "declared_nodal_contact_by_set": nodal_contacts,
        "predicted_nodal_rows": predicted_nodal_rows,
        "role_identity_prefix": profile["corpus_overlay"]["role_identity_prefix"],
        "roles": summaries,
        "total_cases": sum(len(cases) for cases in role_cases.values()),
        "total_modal_rows": sum(len(cases) for cases in role_cases.values())
        * corpus["mode_count"],
    }


def validate_oracle_and_access_order(
    profile: dict[str, Any],
    effective: dict[str, Any],
    v32: dict[str, Any],
    v33: dict[str, Any],
) -> dict[str, Any]:
    for branch in ("contact", "decay", "global_gain"):
        expression = effective["oracle"][branch]["expression"]
        if expression in {
            v32["oracle"][branch]["expression"],
            v33["oracle"][branch]["expression"],
        }:
            raise F0FreezeError(f"parent oracle expression reused: {branch}")
    order = profile["access_order"]
    expected = [
        "f0-identity-freshness-and-static-freeze",
        "c0-signal-blind-structural-witness-census",
        "t0-discarded-whole-owner-terminal-path-proof",
        "d0-train-materialize-and-fit",
        "d0-development-materialize-and-gate",
        "candidate-freeze-if-development-pass",
        "h0-method-holdout-materialize-once-if-frozen",
        "terminal-report",
    ]
    if order != expected:
        raise F0FreezeError("access order drift")
    terminal = profile["terminal_publication"]
    if (
        terminal["post_access_raise_allowed"] is not False
        or terminal["atomic_staging_required"] is not True
        or terminal["repeat_exact_required"] is not True
        or terminal["pre_access_contract_reject"]
        != {"partial_output_allowed": False, "target_rows": 0}
        or terminal["post_access_outcomes"]
        != ["Pass", "MetricReject", "HardGateReject", "ResourceReject"]
    ):
        raise F0FreezeError("terminal publication contract drift")
    return {
        "fresh_truth_expressions": 3,
        "first_target_stage": "d0-train-materialize-and-fit",
        "pretraining_stages": order[:3],
    }


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise F0FreezeError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise F0FreezeError("output must be a fresh external path")
    return output


def build_conformance(profile: dict[str, Any], profile_data: bytes) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    v32 = load_declared_json(profile, "v32_profile")
    v33 = load_declared_json(profile, "v33_profile")
    preserved = validate_preserved_sections(profile, v33)
    effective = build_effective_profile(profile, v33)
    freshness = validate_freshness_counts_and_witness_plan(profile, effective, v32, v33)
    oracle_access = validate_oracle_and_access_order(profile, effective, v32, v33)
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    return {
        "access": ZERO_ACCESS,
        "claim": CLAIM,
        "dependencies": dependencies,
        "freshness_counts_and_witness_plan": freshness,
        "gates": {
            "authority_narrow": "Pass",
            "canonical_overlay": "Pass",
            "fresh_against_v32_v33": "Pass",
            "model_or_target_values": "0 Exact",
            "preserved_v33_hypothesis": "Pass",
            "target_safe_access_order": "Pass",
            "witness_plan_nonvacuous": "Pass",
        },
        "official_values_opened": False,
        "oracle_and_access_order": oracle_access,
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "preserved_sha256": preserved,
        "profile_identity": {
            "bytes": len(profile_data),
            "path": PROFILE_PATH,
            "sha256": sha256_bytes(profile_data),
        },
        "schema": "nextengine.experimental-physical-sound-v34-f0-conformance.v1",
        "status": "Pass",
    }


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v34-f0-", dir=output.parent))
    try:
        conformance = build_conformance(profile, profile_data)
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        evidence = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "conformance": conformance_ref,
            "owner_identity": conformance["owner_identity"],
            "profile_identity": conformance["profile_identity"],
            "schema": "nextengine.experimental-physical-sound-v34-f0-evidence.v1",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "decision": "F0_TARGET_SAFE_PROFILE_FREEZE_PASS",
            "evidence": evidence_ref,
            "next_authorized_stage": "V34-C0-signal-blind-structural-witness-census",
            "official_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v34-f0-report.v1",
            "status": "Pass",
        }
        write_bytes(staging, "report.json", canonical_json(report))
        staging.replace(output)
        return report
    except Exception:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    try:
        report = run(arguments.profile, arguments.output)
    except (F0FreezeError, OSError, KeyError, TypeError, ValueError) as error:
        print(
            canonical_json(
                {"decision": "CONTRACT_REJECT", "error": str(error)}
            ).decode(),
            end="",
        )
        return 2
    print(
        canonical_json(
            {
                "decision": report["decision"],
                "official_values_opened": report["official_values_opened"],
                "profile_sha256": PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
