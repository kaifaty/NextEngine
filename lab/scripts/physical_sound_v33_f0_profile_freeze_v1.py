#!/usr/bin/env python3
"""Validate the V33 F0 preregistration without opening target/model values."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import tempfile
from pathlib import Path
from typing import Any

PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v33-f0-mode-local-spectral-residual-profile.v1"
PROFILE_ID = "physical-sound-v33-f0-mode-local-spectral-residual-v1"
PROFILE_SHA256 = "4c1869101af3ec2264771e17af5c2e652e5861e36c3a8ab61333e588db88fd52"
BASELINE_COMMIT = "898b663b5fb15bf5f55bbc063cac4535bf72e5a0"
CLAIM = (
    "SYNTHETIC_FRESH_ROLE_MODE_LOCAL_SPECTRAL_REPRESENTATION_ONLY / "
    "NO_REAL_MATERIAL_QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
OWNER_PATH = "lab/scripts/physical_sound_v33_f0_profile_freeze_v1.py"
MAX_PROFILE_BYTES = 1_048_576
ZERO_ACCESS = {
    "development_target_rows": 0,
    "method_holdout_target_rows": 0,
    "model_parameters_initialized": 0,
    "network_requests": 0,
    "oracle_values_evaluated": 0,
    "real_signal_values_decoded": 0,
    "train_target_rows": 0,
}


class F0FreezeError(RuntimeError):
    """The value-independent F0 freeze contract failed."""


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
    if (
        authority.get("external_research_only") is not True
        or authority.get("synthetic_only") is not True
        or authority.get("authored_fallback_required") is not True
        or authority.get("network_allowed") is not False
        or authority.get("real_signal_allowed") is not False
        or authority.get("quality_authority") is not False
        or authority.get("runtime_authority") is not False
        or authority.get("admission_authority") is not False
    ):
        raise F0FreezeError("profile authority widened")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    root = repository_root()
    declarations = {
        **profile["parent"],
        "protocol": profile["protocol"],
        "research": profile["research"],
    }
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


def load_v32_profile(profile: dict[str, Any]) -> dict[str, Any]:
    declaration = profile["parent"]["v32_m0_profile"]
    data = (repository_root() / declaration["path"]).read_bytes()
    if sha256_bytes(data) != declaration["sha256"]:
        raise F0FreezeError("V32 profile drift")
    return json.loads(data)


def contact_pairs(corpus: dict[str, Any]) -> dict[str, tuple[tuple[str, str], ...]]:
    return {
        name: tuple(tuple(pair) for pair in pairs)
        for name, pairs in corpus["contacts"].items()
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
        stratum = entry["stratum"]
        if stratum in strata:
            raise F0FreezeError(f"duplicate role stratum: {role}/{stratum}")
        stratum_cases: set[tuple[int, int, int, tuple[str, str]]] = set()
        for family_index in range(family_count):
            for material_index in range(material_count):
                for cell_index in entry["geometry_cells"]:
                    for pair in contacts[entry["contact_set"]]:
                        stratum_cases.add(
                            (family_index, material_index, cell_index, pair)
                        )
        if cases & stratum_cases:
            raise F0FreezeError(f"case overlaps role strata: {role}/{stratum}")
        cases |= stratum_cases
        strata[stratum] = {
            "cases": len(stratum_cases),
            "modal_rows": len(stratum_cases) * corpus["mode_count"],
            "role_local_geometry_groups": (
                family_count * material_count * len(set(entry["geometry_cells"]))
            ),
        }
    return cases, strata


def validate_freshness_and_counts(
    profile: dict[str, Any], v32_profile: dict[str, Any]
) -> dict[str, Any]:
    corpus = profile["corpus"]
    old = v32_profile["corpus"]
    new_materials = {canonical_json(item) for item in corpus["materials"]}
    old_materials = {canonical_json(item) for item in old["materials"]}
    new_cells = {tuple(item) for item in corpus["geometry_multiplier_cells"]}
    old_cells = {tuple(item) for item in old["geometry_multiplier_cells"]}
    new_contact_sets = contact_pairs(corpus)
    new_contacts = {pair for pairs in new_contact_sets.values() for pair in pairs}
    old_contacts = {tuple(pair) for pair in old["contacts"]}
    if new_materials & old_materials:
        raise F0FreezeError("V32 material row reused")
    if new_cells & old_cells:
        raise F0FreezeError("V32 geometry cell reused")
    if new_contacts & old_contacts:
        raise F0FreezeError("V32 contact pair reused")
    if sum(len(pairs) for pairs in new_contact_sets.values()) != len(new_contacts):
        raise F0FreezeError("fresh contact sets overlap")
    if len(new_cells) != len(corpus["geometry_multiplier_cells"]):
        raise F0FreezeError("fresh geometry cells are not unique")

    expected_strata = {
        "development": {"contact-only", "geometry-only", "joint"},
        "method_holdout": {"contact-only", "geometry-only", "joint"},
        "train": {"train"},
    }
    role_cases: dict[str, set[tuple[int, int, int, tuple[str, str]]]] = {}
    role_summaries: dict[str, dict[str, Any]] = {}
    role_group_total = 0
    for role in ("train", "development", "method_holdout"):
        cases, strata = enumerate_role_cases(corpus, role)
        if set(strata) != expected_strata[role]:
            raise F0FreezeError(f"stratum closure mismatch: {role}")
        role_cases[role] = cases
        geometry_cells = {
            cell
            for entry in corpus["role_plan"][role]
            for cell in entry["geometry_cells"]
        }
        geometry_groups = (
            len(corpus["families"]) * len(corpus["materials"]) * len(geometry_cells)
        )
        expected = corpus["counts"][role]
        if (
            len(cases) != expected["cases"]
            or len(cases) * corpus["mode_count"] != expected["modal_rows"]
            or geometry_groups != expected["role_local_geometry_groups"]
        ):
            raise F0FreezeError(f"role count algebra mismatch: {role}")
        if role != "train" and strata != expected["strata"]:
            raise F0FreezeError(f"stratum count algebra mismatch: {role}")
        role_group_total += geometry_groups
        role_summaries[role] = {
            "cases": len(cases),
            "modal_rows": len(cases) * corpus["mode_count"],
            "role_local_geometry_groups": geometry_groups,
            "strata": strata,
        }
    for left, right in (
        ("train", "development"),
        ("train", "method_holdout"),
        ("development", "method_holdout"),
    ):
        if role_cases[left] & role_cases[right]:
            raise F0FreezeError(f"case identity crosses roles: {left}/{right}")
    total_cases = sum(len(cases) for cases in role_cases.values())
    unique_geometry_groups = (
        len(corpus["families"])
        * len(corpus["materials"])
        * len(corpus["geometry_multiplier_cells"])
    )
    counts = corpus["counts"]
    if (
        total_cases != counts["cases"]
        or total_cases * corpus["mode_count"] != counts["modal_rows"]
        or role_group_total != counts["role_local_geometry_groups"]
        or unique_geometry_groups != counts["unique_geometry_groups"]
    ):
        raise F0FreezeError("total count algebra mismatch")
    return {
        "contact_count_by_set": {
            name: len(pairs) for name, pairs in sorted(new_contact_sets.items())
        },
        "fresh_against_v32": {
            "contact_pairs": len(new_contacts),
            "geometry_cells": len(new_cells),
            "material_rows": len(new_materials),
        },
        "roles": role_summaries,
        "total_cases": total_cases,
        "total_modal_rows": total_cases * corpus["mode_count"],
        "unique_geometry_groups": unique_geometry_groups,
    }


def dense_head_parameter_count(input_count: int) -> int:
    return (input_count + 1) * 16 + (16 + 1) * 16 + (16 + 1)


def validate_features_models_and_gates(
    profile: dict[str, Any], v32_profile: dict[str, Any]
) -> dict[str, Any]:
    fields = profile["features"]["branch_fields"]
    old_fields = v32_profile["features"]["branch_fields"]
    if fields["contact"][:12] != old_fields["contact"]:
        raise F0FreezeError("contact base fields differ from frozen V32 control")
    expected_fourier = [
        f"contact.fourier_{axis}_{function}_{order}"
        for order in range(1, 5)
        for axis, function in (
            ("u", "sin"),
            ("u", "cos"),
            ("v", "sin"),
            ("v", "cos"),
        )
    ]
    expected_stencil = [
        "p1.contact_participation_u_minus",
        "p1.contact_participation_u_plus",
        "p1.contact_participation_v_minus",
        "p1.contact_participation_v_plus",
    ]
    if fields["contact"][12:28] != expected_fourier:
        raise F0FreezeError("Fourier feature order mismatch")
    if fields["contact"][28:] != expected_stencil:
        raise F0FreezeError("mode-local stencil feature order mismatch")
    if len(fields["decay"]) != 11 or len(fields["global_gain"]) != 13:
        raise F0FreezeError("inherited branch field count mismatch")
    lift = profile["features"]["lift"]
    if (
        lift["contact_base_input_count"] != 12
        or lift["contact_input_count"] != 32
        or lift["fourier_orders"] != [1, 2, 3, 4]
        or lift["local_stencil_offset"] != "0.0625"
    ):
        raise F0FreezeError("spectral lift contract mismatch")

    model = profile["model"]
    expected_head_counts = {
        "contact": dense_head_parameter_count(32),
        "decay": dense_head_parameter_count(11),
        "global_gain": dense_head_parameter_count(13),
    }
    for name, expected in expected_head_counts.items():
        head = model["heads"][name]
        if head["parameter_count"] != expected:
            raise F0FreezeError(f"parameter count mismatch: {name}")
    parameter_count = sum(expected_head_counts.values())
    if parameter_count != 1811 or model["parameter_count"] != parameter_count:
        raise F0FreezeError("candidate parameter total mismatch")
    if model["parameter_count"] > model["parameter_count_max"]:
        raise F0FreezeError("candidate exceeds parameter ceiling")
    raw_control = profile["controls"]["raw_mlp"]
    if (
        raw_control["parameter_count"] != 1491
        or raw_control["contact_input_count"] != 12
    ):
        raise F0FreezeError("raw MLP control drift")

    oracle = profile["oracle"]
    for branch in ("contact", "decay", "global_gain"):
        if oracle[branch]["expression"] == v32_profile["oracle"][branch]["expression"]:
            raise F0FreezeError(f"V32 oracle expression reused: {branch}")
    if set(oracle["contact"]["components"]) != {
        "curvature_u",
        "difference_u",
        "difference_v",
        "surface_a",
        "surface_b",
    }:
        raise F0FreezeError("contact truth component closure mismatch")
    if profile["access_order"].index("d0-development-materialize-and-gate") < profile[
        "access_order"
    ].index("candidate-freeze-if-development-pass"):
        pass
    else:
        raise F0FreezeError("candidate freeze precedes development gate")
    if profile["access_order"].index("candidate-freeze-if-development-pass") < profile[
        "access_order"
    ].index("h0-method-holdout-materialize-once-if-frozen"):
        pass
    else:
        raise F0FreezeError("method holdout precedes candidate freeze")
    if not profile["gates"]["hard"]["method_holdout_zero_before_development_pass"]:
        raise F0FreezeError("holdout zero-access gate missing")
    return {
        "candidate_parameter_count": parameter_count,
        "contact_feature_count": len(fields["contact"]),
        "decay_feature_count": len(fields["decay"]),
        "fourier_feature_count": len(expected_fourier),
        "global_gain_feature_count": len(fields["global_gain"]),
        "local_stencil_feature_count": len(expected_stencil),
        "raw_mlp_parameter_count": raw_control["parameter_count"],
        "truth_expressions_distinct_from_v32": 3,
    }


def build_conformance(profile: dict[str, Any], profile_data: bytes) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    v32_profile = load_v32_profile(profile)
    freshness = validate_freshness_and_counts(profile, v32_profile)
    features = validate_features_models_and_gates(profile, v32_profile)
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    return {
        "access": ZERO_ACCESS,
        "claim": CLAIM,
        "dependencies": dependencies,
        "features_and_models": features,
        "freshness_and_counts": freshness,
        "gates": {
            "authority_narrow": "Pass",
            "canonical_profile": "Pass",
            "case_roles_disjoint": "Pass",
            "dependency_closure": "Pass",
            "fresh_against_v32": "Pass",
            "holdout_access": "0 Exact",
            "model_or_oracle_values": "0 Exact",
            "profile_counts": "1512 cases / 15120 rows Exact",
        },
        "official_values_opened": False,
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "profile_identity": {
            "bytes": len(profile_data),
            "path": "lab/profiles/physical-sound-v33-f0-mode-local-spectral-residual.v1.json",
            "sha256": sha256_bytes(profile_data),
        },
        "schema": "nextengine.experimental-physical-sound-v33-f0-conformance.v1",
        "status": "Pass",
    }


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    path = root / name
    path.write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v33-f0-", dir=output.parent))
    try:
        conformance = build_conformance(profile, profile_data)
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        owner_identity = conformance["owner_identity"]
        evidence = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "conformance": conformance_ref,
            "owner_identity": owner_identity,
            "profile_identity": conformance["profile_identity"],
            "schema": "nextengine.experimental-physical-sound-v33-f0-evidence.v1",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "decision": "F0_PROFILE_FREEZE_PASS",
            "evidence": evidence_ref,
            "official_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v33-f0-report.v1",
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
