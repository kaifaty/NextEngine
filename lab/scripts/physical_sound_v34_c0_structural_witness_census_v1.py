#!/usr/bin/env python3
"""Count V34 P1 structural witnesses without importing targets or models."""

from __future__ import annotations

import argparse
import ast
import copy
import hashlib
import json
import math
import resource
import shutil
import struct
import sys
import tempfile
import time
from collections import defaultdict
from decimal import Decimal
from pathlib import Path
from typing import Any

import physical_sound_v31_p1_modal_owner_v1 as p1
import physical_sound_v34_f0_target_safe_profile_freeze_v1 as f0

F0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v34-f0-target-safe-profile-freeze-result-2026-09-02.md"
)
F0_RESULT_SHA256 = "0993148fb9d5642c05505e6aabdcc5c3ce86ef4b50b3f3b6f160b7d21b378628"
F0_OWNER_SHA256 = "e71a2b9b8b86b190e6c129644b29306fe4a8cbffff68bd24c231f961f086ca28"
P1_OWNER_SHA256 = "04b44d77ce073d07f30e94c3c361ca4c199cb554ecbe017947aa423842781650"
OWNER_PATH = "lab/scripts/physical_sound_v34_c0_structural_witness_census_v1.py"
CLAIM = (
    "SIGNAL_BLIND_P1_STRUCTURAL_WITNESS_CENSUS_ONLY / "
    "NO_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
ALLOWED_ROLES = ("development", "method_holdout")
FORBIDDEN_IMPORT_STEMS = {
    "physical_sound_v33_d0_development_tournament_v1",
    "physical_sound_v33_i0_mode_local_spectral_owner_v1",
    "torch",
}
ZERO_FORBIDDEN_ACCESS = dict(f0.ZERO_ACCESS)


class C0CensusError(RuntimeError):
    """The signal-blind structural census contract failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise C0CensusError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise C0CensusError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise C0CensusError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_owner_import_boundary() -> dict[str, Any]:
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    tree = ast.parse(owner_data)
    imports: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            imports.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom) and node.module:
            imports.add(node.module.split(".")[0])
    forbidden = sorted(imports & FORBIDDEN_IMPORT_STEMS)
    if forbidden:
        raise C0CensusError(f"forbidden owner import: {','.join(forbidden)}")
    if not {
        "physical_sound_v31_p1_modal_owner_v1",
        "physical_sound_v34_f0_target_safe_profile_freeze_v1",
    }.issubset(imports):
        raise C0CensusError("required P1/F0 owner imports missing")
    return {"forbidden_imports": forbidden, "imports": sorted(imports)}


def load_context(profile_path: Path) -> tuple[dict[str, Any], dict[str, Any], bytes]:
    overlay, profile_data = f0.load_profile(profile_path)
    if sha256_bytes(profile_data) != f0.PROFILE_SHA256:
        raise C0CensusError("F0 profile identity drift")
    f0.validate_dependencies(overlay)
    v33 = f0.load_declared_json(overlay, "v33_profile")
    effective = f0.build_effective_profile(overlay, v33)
    p0_declaration = v33["parent"]["p0_profile"]
    p0_path = repository_root() / p0_declaration["path"]
    p0_data = p0_path.read_bytes()
    if sha256_bytes(p0_data) != p0_declaration["sha256"]:
        raise C0CensusError("P0 profile dependency drift")
    p1_declaration = v33["parent"]["p1_owner"]
    if p1_declaration["sha256"] != P1_OWNER_SHA256:
        raise C0CensusError("P1 owner declaration drift")
    validate_bound_file(p1_declaration["path"], P1_OWNER_SHA256)
    validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256)
    validate_bound_file(F0_RESULT_PATH, F0_RESULT_SHA256)
    return overlay, effective, p0_data


def decimal_product(value: str, multiplier: str) -> str:
    return format(Decimal(value) * Decimal(multiplier), "f")


def base_fixture_for_family(
    family: dict[str, Any], p0_profile: dict[str, Any]
) -> dict[str, Any]:
    for fixture in p0_profile["fixtures"]:
        if fixture["family"] == family["family"]:
            result = copy.deepcopy(fixture)
            result["formula_id"] = family["formula_id"]
            result["support"] = family["support"]
            return result
    raise C0CensusError(f"missing P0 family fixture: {family['family']}")


def build_fixture(
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
    role: str,
    stratum: str,
    family_index: int,
    material_index: int,
    geometry_cell_index: int,
    contact_set: str,
    contact_index: int,
) -> tuple[str, dict[str, Any]]:
    corpus = effective["corpus"]
    family = corpus["families"][family_index]
    fixture = base_fixture_for_family(family, p0_profile)
    case_id = (
        f"v34/{role}/{stratum}/f{family_index}/m{material_index}/"
        f"g{geometry_cell_index:02d}/{contact_set}/c{contact_index:02d}"
    )
    fixture["fixture_id"] = case_id
    material = corpus["materials"][material_index]
    fixture["material"] = {
        "density_kg_m3": material["density_kg_m3"],
        "loss_rate_per_second": material["loss_rate_per_second"],
        "material_id": "synthetic-elastic-reference",
        "poisson_ratio": material["poisson_ratio"],
        "youngs_modulus_pa": material["youngs_modulus_pa"],
    }
    multipliers = corpus["geometry_multiplier_cells"][geometry_cell_index]
    for field, multiplier in zip(
        family["geometry_component_order"], multipliers, strict=True
    ):
        fixture["geometry"][field] = decimal_product(
            fixture["geometry"][field], multiplier
        )
    u, v = corpus["contacts"][contact_set][contact_index]
    fixture["contact"] = {
        "normal_impulse_ns": corpus["impulse_ns"],
        "u": u,
        "v": v,
    }
    fixture["mode_count"] = corpus["mode_count"]
    return case_id, fixture


def id_commitment(values: list[str]) -> dict[str, Any]:
    ordered = sorted(values)
    if len(ordered) != len(set(ordered)):
        raise C0CensusError("witness identity is duplicated")
    return {
        "count": len(ordered),
        "first": ordered[0] if ordered else None,
        "last": ordered[-1] if ordered else None,
        "sha256": sha256_bytes(canonical_json(ordered)),
    }


def binary64_pair(value_a: float, value_b: float) -> bytes:
    return struct.pack("<dd", value_a, value_b)


def binary64(value: float) -> bytes:
    return struct.pack("<d", value)


def census_role(
    role: str,
    overlay: dict[str, Any],
    effective: dict[str, Any],
    p0_profile: dict[str, Any],
) -> dict[str, Any]:
    if role not in ALLOWED_ROLES:
        raise C0CensusError(f"C0 role access forbidden: {role}")
    corpus = effective["corpus"]
    all_rows: list[str] = []
    nodal: dict[str, list[str]] = defaultdict(list)
    pickup_positive: list[str] = []
    pickup_negative: list[str] = []
    remesh: dict[str, list[str]] = defaultdict(list)
    non_silent: dict[str, dict[str, list[str]]] = defaultdict(lambda: defaultdict(list))
    material_groups: dict[str, dict[tuple[Any, ...], list[tuple[str, bytes]]]] = (
        defaultdict(lambda: defaultdict(list))
    )
    contact_groups: dict[str, dict[tuple[Any, ...], list[tuple[str, bytes]]]] = (
        defaultdict(lambda: defaultdict(list))
    )
    case_ids: list[str] = []
    for entry in corpus["role_plan"][role]:
        stratum = entry["stratum"]
        contact_set = entry["contact_set"]
        for family_index, family in enumerate(corpus["families"]):
            family_key = f"f{family_index}:{family['formula_id']}"
            for material_index in range(len(corpus["materials"])):
                for geometry_cell_index in entry["geometry_cells"]:
                    for contact_index in range(len(corpus["contacts"][contact_set])):
                        case_id, fixture = build_fixture(
                            effective,
                            p0_profile,
                            role,
                            stratum,
                            family_index,
                            material_index,
                            geometry_cell_index,
                            contact_set,
                            contact_index,
                        )
                        solution = p1.solve_case(case_id, fixture, p0_profile)
                        case_ids.append(case_id)
                        if (
                            solution["metrics"]["remesh_common_vertices_exact"]
                            is not True
                        ):
                            raise C0CensusError(f"remesh witness failed: {case_id}")
                        remesh[family_key].append(case_id)
                        peak = float(solution["metrics"]["sample_peak"])
                        energy = float(solution["metrics"]["sample_energy"])
                        if peak > 0.0 and energy > 0.0:
                            non_silent[stratum][family_key].append(case_id)
                        modes = solution["modal_document"]["modes"]
                        if len(modes) != corpus["mode_count"]:
                            raise C0CensusError(f"mode count drift: {case_id}")
                        for ordinal, mode in enumerate(modes):
                            row_id = f"{case_id}/o{ordinal:02d}"
                            all_rows.append(row_id)
                            contact = float(mode["contact_participation"])
                            pickup = float(mode["pickup_participation"])
                            signed_gain = float(mode["signed_gain"])
                            frequency = float(mode["frequency_hz"])
                            decay = float(mode["decay_per_second"])
                            if (
                                int(mode["ordinal"]) != ordinal
                                or not all(
                                    math.isfinite(value)
                                    for value in (
                                        contact,
                                        pickup,
                                        signed_gain,
                                        frequency,
                                        decay,
                                    )
                                )
                                or frequency <= 0.0
                                or decay <= 0.0
                                or signed_gain != contact * pickup
                            ):
                                raise C0CensusError(f"invalid structural row: {row_id}")
                            if contact == 0.0:
                                if signed_gain != 0.0:
                                    raise C0CensusError(f"nodal gain drift: {row_id}")
                                nodal[stratum].append(row_id)
                            if pickup > 0.0:
                                pickup_positive.append(row_id)
                            elif pickup < 0.0:
                                pickup_negative.append(row_id)
                            material_key = (
                                family_index,
                                geometry_cell_index,
                                contact_set,
                                contact_index,
                                ordinal,
                            )
                            material_groups[stratum][material_key].append(
                                (row_id, binary64_pair(frequency, decay))
                            )
                            contact_key = (
                                family_index,
                                material_index,
                                geometry_cell_index,
                                ordinal,
                            )
                            contact_groups[stratum][contact_key].append(
                                (row_id, binary64(contact))
                            )

    expected = corpus["counts"][role]
    if len(case_ids) != expected["cases"] or len(all_rows) != expected["modal_rows"]:
        raise C0CensusError(f"role count closure mismatch: {role}")
    witness = overlay["witness_contract"]
    strata: dict[str, Any] = {}
    for stratum in witness["strata"]:
        nodal_commitment = id_commitment(nodal[stratum])
        if (
            nodal_commitment["count"]
            < witness["exact_nodal_min_modal_rows_per_stratum"]
        ):
            raise C0CensusError(f"nodal witness shortfall: {role}/{stratum}")
        material_sensitive = [
            row_id
            for rows in material_groups[stratum].values()
            if len({value for _, value in rows}) > 1
            for row_id, _ in rows
        ]
        contact_sensitive = [
            row_id
            for rows in contact_groups[stratum].values()
            if len({value for _, value in rows}) > 1
            for row_id, _ in rows
        ]
        material_commitment = id_commitment(material_sensitive)
        contact_commitment = id_commitment(contact_sensitive)
        if (
            material_commitment["count"]
            < witness["minimum_material_sensitive_rows_per_stratum"]
            or contact_commitment["count"]
            < witness["minimum_contact_sensitive_rows_per_stratum"]
        ):
            raise C0CensusError(f"sensitivity witness shortfall: {role}/{stratum}")
        family_non_silent: dict[str, Any] = {}
        for family_index, family in enumerate(corpus["families"]):
            family_key = f"f{family_index}:{family['formula_id']}"
            commitment = id_commitment(non_silent[stratum][family_key])
            if (
                commitment["count"]
                < witness["minimum_non_silent_cases_per_family_and_stratum"]
            ):
                raise C0CensusError(
                    f"non-silent witness shortfall: {role}/{stratum}/{family_key}"
                )
            family_non_silent[family_key] = commitment
        strata[stratum] = {
            "contact_sensitive_rows": contact_commitment,
            "material_sensitive_rows": material_commitment,
            "nodal_zero_rows": nodal_commitment,
            "non_silent_cases_by_family": family_non_silent,
        }

    positive = id_commitment(pickup_positive)
    negative = id_commitment(pickup_negative)
    if (
        positive["count"] < witness["minimum_nonzero_pickup_positive_rows_per_role"]
        or negative["count"] < witness["minimum_nonzero_pickup_negative_rows_per_role"]
    ):
        raise C0CensusError(f"pickup-sign witness shortfall: {role}")
    remesh_result: dict[str, Any] = {}
    for family_index, family in enumerate(corpus["families"]):
        family_key = f"f{family_index}:{family['formula_id']}"
        commitment = id_commitment(remesh[family_key])
        if commitment["count"] < witness["minimum_remesh_pairs_per_family_and_role"]:
            raise C0CensusError(f"remesh witness shortfall: {role}/{family_key}")
        remesh_result[family_key] = commitment
    return {
        "all_case_ids": id_commitment(case_ids),
        "all_modal_row_ids": id_commitment(all_rows),
        "pickup_negative_rows": negative,
        "pickup_positive_rows": positive,
        "remesh_cases_by_family": remesh_result,
        "role": role,
        "strata": strata,
    }


def external_output(path: Path) -> Path:
    try:
        return f0.external_output(path)
    except f0.F0FreezeError as error:
        raise C0CensusError(str(error)) from error


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    overlay, effective, p0_data = load_context(profile_path)
    output = external_output(output_path)
    import_boundary = validate_owner_import_boundary()
    p0_profile = json.loads(p0_data)
    roles = {
        role: census_role(role, overlay, effective, p0_profile)
        for role in ALLOWED_ROLES
    }
    structural_access = {
        "p1_cases_solved": sum(
            role["all_case_ids"]["count"] for role in roles.values()
        ),
        "p1_modal_rows_observed": sum(
            role["all_modal_row_ids"]["count"] for role in roles.values()
        ),
        "roles_observed": list(ALLOWED_ROLES),
    }
    if structural_access != {
        "p1_cases_solved": 864,
        "p1_modal_rows_observed": 8640,
        "roles_observed": ["development", "method_holdout"],
    }:
        raise C0CensusError("structural access closure mismatch")
    census = {
        "claim": CLAIM,
        "forbidden_access": ZERO_FORBIDDEN_ACCESS,
        "roles": roles,
        "schema": "nextengine.experimental-physical-sound-v34-c0-census.v1",
        "status": "Pass",
        "structural_access": structural_access,
    }
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    dependencies = {
        "f0_owner": validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256),
        "f0_result": validate_bound_file(F0_RESULT_PATH, F0_RESULT_SHA256),
        "p1_owner": validate_bound_file(
            "lab/scripts/physical_sound_v31_p1_modal_owner_v1.py", P1_OWNER_SHA256
        ),
    }
    elapsed = time.monotonic() - started
    rss = peak_rss_bytes()
    resources = effective["resources"]
    resource_gates = {
        "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
        "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
    }
    if not all(resource_gates.values()):
        raise C0CensusError("C0 resource envelope exceeded")
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v34-c0-", dir=output.parent))
    try:
        census_ref = write_bytes(staging, "census.json", canonical_json(census))
        evidence = {
            "claim": CLAIM,
            "dependencies": dependencies,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "import_boundary": import_boundary,
            "owner_identity": {
                "bytes": len(owner_data),
                "path": OWNER_PATH,
                "sha256": sha256_bytes(owner_data),
            },
            "profile_identity": {
                "bytes": len((repository_root() / f0.PROFILE_PATH).read_bytes()),
                "path": f0.PROFILE_PATH,
                "sha256": f0.PROFILE_SHA256,
            },
            "resource_gates": resource_gates,
            "schema": "nextengine.experimental-physical-sound-v34-c0-evidence.v1",
            "structural_access": structural_access,
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "census": census_ref,
            "claim": CLAIM,
            "decision": "C0_STRUCTURAL_WITNESS_CENSUS_PASS",
            "evidence": evidence_ref,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "next_authorized_stage": "V34-T0-discarded-whole-owner-terminal-path-proof",
            "official_target_or_model_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v34-c0-report.v1",
            "status": "Pass",
            "structural_access": structural_access,
        }
        write_bytes(staging, "report.json", canonical_json(report))
        total_bytes = sum(path.stat().st_size for path in staging.iterdir())
        if total_bytes > resources["max_output_bytes"]:
            raise C0CensusError("C0 output resource envelope exceeded")
        staging.replace(output)
        print(
            f"c0-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
            f"output_bytes={total_bytes}",
            file=sys.stderr,
        )
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
    except (
        C0CensusError,
        f0.F0FreezeError,
        OSError,
        KeyError,
        TypeError,
        ValueError,
    ) as error:
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
                "official_target_or_model_values_opened": report[
                    "official_target_or_model_values_opened"
                ],
                "profile_sha256": f0.PROFILE_SHA256,
            }
        ).decode(),
        end="",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
