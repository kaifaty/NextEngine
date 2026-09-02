#!/usr/bin/env python3
"""Freeze and audit the V31 P0 synthetic causal-baseline contract."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from decimal import Decimal, InvalidOperation, localcontext
from pathlib import Path
from typing import Any


PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v31-p0-causal-baseline-profile.v1"
)
CONTRACT_SCHEMA = (
    "nextengine.experimental-physical-sound-v31-p0-causal-baseline-contract.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v31-p0-causal-baseline-report.v1"
)
PROFILE_ID = "physical-sound-v31-p0-causal-baseline-v1"
PROFILE_SHA256 = "c6b7f816d65bdcaa18618a72d40cfedb6ed970359d3a1cf8f28825cd4c686a3d"
CLAIM = (
    "SYNTHETIC_CAUSAL_BASELINE_PROTOCOL_ONLY / NO_SOLVER_QUALITY_REAL_MATERIAL_"
    "VALIDATOR_ADMISSION_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 128 * 1024
PI = Decimal("3.1415926535897932384626433832795028841971693993751")
CANTILEVER_BETA_1 = Decimal("1.875104068711961")

TOP_KEYS = {
    "authority",
    "baseline_commit",
    "claim",
    "fallback",
    "fixtures",
    "formulae",
    "interventions",
    "modal_output",
    "numeric_profile",
    "profile_id",
    "protocol",
    "resources",
    "revision",
    "schema",
}
AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "external_research_only": True,
    "model_allowed": False,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "validator_authority": False,
}
NUMERIC_PROFILE = {
    "canonical_json": "utf8-sort-keys-indent2-lf",
    "coordinates": "right-handed",
    "execution": "single-thread-cpu-reference",
    "modal_sort": ["frequency_hz", "family_index_a", "family_index_b"],
    "output_encoding": "mono-f32le-wav",
    "randomness": "forbidden",
    "render_frames": 144_000,
    "render_scale_per_ns": "0.01",
    "rounding": "nearest-ties-to-even",
    "sample_rate_hz": 48_000,
    "scalar_format": "ieee754-binary64-little-endian",
    "units": "SI",
}
RESOURCES = {
    "max_mesh_elements": 200_000,
    "max_mesh_vertices": 100_000,
    "max_modes": 64,
    "max_output_bytes": 256 * 1024 * 1024,
    "max_peak_rss_bytes": 1024 * 1024 * 1024,
    "max_render_frames": 144_000,
    "max_wall_seconds": 300,
}
FALLBACK = {
    "authored_clip_class": "engine.authored-impact-clip",
    "contract_failure": "ContractReject",
    "out_of_domain": "FallbackOutOfDomain",
    "publish_partial_output": False,
    "reason_codes": [
        "UnsupportedGeometry",
        "UnsupportedAcousticMaterial",
        "UnsupportedSupport",
        "UnsupportedContact",
        "InvalidNumericInput",
        "ResourceLimitExceeded",
    ],
}
FORMULAE = [
    {
        "family": "rectangular_plate",
        "formula_id": "kirchhoff-love-simply-supported-v1",
        "support": "simply-supported-all-edges",
    },
    {
        "beta_roots": [
            "1.875104068711961",
            "4.694091132974175",
            "7.854757438237612",
            "10.99554073487547",
            "14.13716839104647",
            "17.27875953208824",
            "20.42035225104125",
            "23.56194490180644",
            "26.70353755551830",
            "29.84513020910282",
        ],
        "family": "rectangular_beam",
        "formula_id": "euler-bernoulli-cantilever-v1",
        "support": "cantilever-clamped-u0",
    },
    {
        "beta_rule": "n*pi",
        "family": "rectangular_beam",
        "formula_id": "euler-bernoulli-simply-supported-v1",
        "support": "simply-supported-both-ends",
    },
]
MODAL_OUTPUT = {
    "beam_cantilever_shape": "v24-t0-normalized-cantilever-v1",
    "beam_simply_supported_shape": "sin(n*pi*u)",
    "decay_law": "declared-loss-rate-per-second",
    "mode_count": 10,
    "normalization": (
        "single-global-render-scale/no-per-mode-contact-or-render-normalization"
    ),
    "plate_candidate_range": "m,n=1..10/take-lowest-10",
    "plate_shape": "sin(m*pi*u)*sin(n*pi*v)",
    "record_fields": [
        "ordinal",
        "family_index_a",
        "family_index_b",
        "frequency_hz",
        "decay_per_second",
        "contact_participation",
        "pickup_participation",
        "signed_gain",
    ],
    "render_equation": "v31-p0-linear-damped-modal-sum-v1",
}
ZERO_ACCESS = {
    "audio_headers_parsed": 0,
    "audio_sample_values_decoded": 0,
    "model_parameters_opened": 0,
    "network_requests": 0,
    "protected_signal_values_decoded": 0,
    "solver_output_values_opened": 0,
    "waveform_or_feature_values_decoded": 0,
}

FIXTURE_KEYS = {
    "contact",
    "family",
    "fixture_id",
    "formula_id",
    "geometry",
    "material",
    "mesh_pair",
    "mode_count",
    "pickup",
    "support",
}
MATERIAL_KEYS = {
    "density_kg_m3",
    "loss_rate_per_second",
    "material_id",
    "poisson_ratio",
    "youngs_modulus_pa",
}
MESH_KEYS = {"coarse_u", "coarse_v", "fine_u", "fine_v"}
CONTACT_KEYS = {"normal_impulse_ns", "u", "v"}
PICKUP_KEYS = {"u", "v"}


class CausalBaselineError(RuntimeError):
    """The P0 profile or publication violates the frozen contract."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise CausalBaselineError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CausalBaselineError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _reject_constant(value: str) -> None:
    raise CausalBaselineError(f"non-finite JSON number: {value}")


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise CausalBaselineError("profile must not be a symlink")
    try:
        resolved = path.resolve(strict=True)
        data = resolved.read_bytes()
    except OSError as error:
        raise CausalBaselineError(f"cannot read profile: {error}") from error
    if not resolved.is_file() or not 0 < len(data) <= MAX_PROFILE_BYTES:
        raise CausalBaselineError("profile must be a bounded regular file")
    try:
        value = json.loads(
            data,
            object_pairs_hook=_no_duplicates,
            parse_constant=_reject_constant,
        )
    except (json.JSONDecodeError, UnicodeDecodeError) as error:
        raise CausalBaselineError(f"invalid profile JSON: {error}") from error
    if not isinstance(value, dict):
        raise CausalBaselineError("profile root must be an object")
    if data != canonical_json(value):
        raise CausalBaselineError("profile must use canonical JSON encoding")
    validate_profile(value)
    return value, data


def _exact_keys(value: Any, expected: set[str], label: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != expected:
        raise CausalBaselineError(f"{label} fields changed")
    return value


def _decimal(value: Any, label: str, *, positive: bool = True) -> Decimal:
    if not isinstance(value, str):
        raise CausalBaselineError(f"{label} must be a decimal string")
    try:
        result = Decimal(value)
    except InvalidOperation as error:
        raise CausalBaselineError(f"{label} is not decimal") from error
    if not result.is_finite() or (positive and result <= 0):
        raise CausalBaselineError(f"{label} is outside the finite positive domain")
    return result


def _unit_interval(value: Any, label: str) -> Decimal:
    result = _decimal(value, label, positive=False)
    if result < 0 or result > 1:
        raise CausalBaselineError(f"{label} must be in [0, 1]")
    return result


def _validate_material(value: Any, label: str) -> None:
    material = _exact_keys(value, MATERIAL_KEYS, label)
    if material["material_id"] != "synthetic-elastic-reference":
        raise CausalBaselineError(f"{label} must retain the non-real material ID")
    density = _decimal(material["density_kg_m3"], f"{label}.density")
    youngs = _decimal(material["youngs_modulus_pa"], f"{label}.youngs")
    poisson = _decimal(material["poisson_ratio"], f"{label}.poisson")
    loss = _decimal(material["loss_rate_per_second"], f"{label}.loss")
    if not Decimal("100") <= density <= Decimal("50000"):
        raise CausalBaselineError(f"{label}.density exceeds the synthetic domain")
    if not Decimal("1000000") <= youngs <= Decimal("1000000000000"):
        raise CausalBaselineError(f"{label}.youngs exceeds the synthetic domain")
    if not Decimal("0") < poisson < Decimal("0.5") or loss > Decimal("10000"):
        raise CausalBaselineError(f"{label} has invalid elastic or loss values")


def _validate_mesh(value: Any, label: str) -> None:
    mesh = _exact_keys(value, MESH_KEYS, label)
    if any(type(mesh[key]) is not int or mesh[key] < 3 for key in MESH_KEYS):
        raise CausalBaselineError(f"{label} dimensions must be integers >= 3")
    if (
        mesh["fine_u"] != 2 * mesh["coarse_u"] - 1
        or mesh["fine_v"] != 2 * mesh["coarse_v"] - 1
    ):
        raise CausalBaselineError(f"{label} must be an exact nested remesh pair")
    if mesh["fine_u"] * mesh["fine_v"] > RESOURCES["max_mesh_vertices"]:
        raise CausalBaselineError(f"{label} exceeds the vertex envelope")


def _validate_fixture(value: Any) -> str:
    fixture = _exact_keys(value, FIXTURE_KEYS, "fixture")
    fixture_id = fixture["fixture_id"]
    if fixture_id not in {"synthetic-plate-reference", "synthetic-beam-reference"}:
        raise CausalBaselineError("fixture ID changed")
    _validate_material(fixture["material"], f"{fixture_id}.material")
    _validate_mesh(fixture["mesh_pair"], f"{fixture_id}.mesh_pair")
    contact = _exact_keys(fixture["contact"], CONTACT_KEYS, f"{fixture_id}.contact")
    pickup = _exact_keys(fixture["pickup"], PICKUP_KEYS, f"{fixture_id}.pickup")
    for key in ("u", "v"):
        _unit_interval(contact[key], f"{fixture_id}.contact.{key}")
        _unit_interval(pickup[key], f"{fixture_id}.pickup.{key}")
    _decimal(contact["normal_impulse_ns"], f"{fixture_id}.impulse")
    if type(fixture["mode_count"]) is not int or not 1 <= fixture["mode_count"] <= 64:
        raise CausalBaselineError(f"{fixture_id}.mode_count exceeds the envelope")

    geometry = fixture["geometry"]
    if fixture_id == "synthetic-plate-reference":
        _exact_keys(
            geometry,
            {"length_x_m", "length_y_m", "thickness_m"},
            f"{fixture_id}.geometry",
        )
        expected = (
            "rectangular_plate",
            "kirchhoff-love-simply-supported-v1",
            "simply-supported-all-edges",
        )
    else:
        _exact_keys(
            geometry,
            {"height_m", "length_m", "width_m"},
            f"{fixture_id}.geometry",
        )
        expected = (
            "rectangular_beam",
            "euler-bernoulli-cantilever-v1",
            "cantilever-clamped-u0",
        )
    for key, scalar in geometry.items():
        if _decimal(scalar, f"{fixture_id}.geometry.{key}") > Decimal("10"):
            raise CausalBaselineError(f"{fixture_id}.geometry exceeds 10 m")
    if (fixture["family"], fixture["formula_id"], fixture["support"]) != expected:
        raise CausalBaselineError(f"{fixture_id} formula/support binding changed")
    return fixture_id


def _expected_interventions() -> dict[str, dict[str, Any]]:
    with localcontext() as context:
        context.prec = 50
        factor_four = Decimal("4")
        factor_two = Decimal("2")
        youngs_ratio = factor_four.sqrt()
        density_ratio = Decimal("1") / factor_four.sqrt()
        thickness_ratio = factor_two
        uniform_scale_ratio = Decimal("1") / factor_two
        support_ratio = (PI / CANTILEVER_BETA_1) ** 2
    contact_target = {"m": 2, "n": 1, "u": "0.5", "v": "0.5"}
    nodal_phase = Decimal(contact_target["m"]) * Decimal(contact_target["u"])
    if nodal_phase != nodal_phase.to_integral_value():
        raise CausalBaselineError("frozen plate contact is not on the target-mode node")
    common = {"fixture_id": "synthetic-plate-reference"}
    return {
        "youngs_modulus": {
            **common,
            "changed_coordinate": "material.youngs_modulus_pa",
            "operation": "multiply",
            "value": str(factor_four),
            "expected_frequency_ratio": str(youngs_ratio),
            "expected_gain": "not-gated",
        },
        "density": {
            **common,
            "changed_coordinate": "material.density_kg_m3",
            "operation": "multiply",
            "value": str(factor_four),
            "expected_frequency_ratio": str(density_ratio),
            "expected_gain": "not-gated",
        },
        "thickness": {
            **common,
            "changed_coordinate": "geometry.thickness_m",
            "operation": "multiply",
            "value": str(factor_two),
            "expected_frequency_ratio": str(thickness_ratio),
            "expected_gain": "not-gated",
        },
        "uniform_scale": {
            **common,
            "changed_coordinate": "geometry.uniform_scale",
            "operation": "multiply",
            "value": str(factor_two),
            "expected_frequency_ratio": str(uniform_scale_ratio),
            "expected_gain": "not-gated",
        },
        "impulse": {
            **common,
            "changed_coordinate": "contact.normal_impulse_ns",
            "operation": "multiply",
            "value": str(factor_two),
            "expected_frequency_ratio": "1",
            "expected_gain": f"linear-ratio:{factor_two}",
        },
        "contact": {
            **common,
            "changed_coordinate": "contact.normalized_position",
            "operation": "set_plate_mode_node",
            "target": contact_target,
            "expected_frequency_ratio": "1",
            "expected_gain": "target-mode-zero",
        },
        "support": {
            "fixture_id": "synthetic-beam-reference",
            "changed_coordinate": "support",
            "operation": "replace",
            "value": "simply-supported-both-ends",
            "expected_frequency_ratio": str(support_ratio),
            "expected_gain": "not-gated",
        },
    }


def _validate_interventions(value: Any) -> list[dict[str, Any]]:
    if not isinstance(value, list) or len(value) != 7:
        raise CausalBaselineError("exactly seven isolated interventions are required")
    expected = _expected_interventions()
    seen: set[str] = set()
    normalized: list[dict[str, Any]] = []
    for item in value:
        if not isinstance(item, dict):
            raise CausalBaselineError("intervention must be an object")
        axis = item.get("axis")
        if axis not in expected or axis in seen:
            raise CausalBaselineError(f"missing, unknown or duplicate intervention axis: {axis}")
        seen.add(axis)
        wanted = {"axis", *expected[axis].keys()}
        _exact_keys(item, wanted, f"{axis} intervention")
        actual_without_axis = {key: item[key] for key in item if key != "axis"}
        if actual_without_axis != expected[axis]:
            raise CausalBaselineError(f"{axis} intervention is not the frozen isolated mutation")
        _decimal(item["expected_frequency_ratio"], f"{axis}.frequency_ratio")
        normalized.append(item)
    if seen != set(expected):
        raise CausalBaselineError("required intervention coverage changed")
    return sorted(normalized, key=lambda item: item["axis"])


def _validate_protocol(value: Any) -> dict[str, str]:
    protocol = _exact_keys(value, {"path", "sha256"}, "protocol")
    if protocol["path"] != (
        "docs/development/physical-sound-v31-p0-causal-baseline-protocol-2026-09-02.md"
    ):
        raise CausalBaselineError("protocol path changed")
    if not isinstance(protocol["sha256"], str) or not re.fullmatch(
        r"[0-9a-f]{64}", protocol["sha256"]
    ):
        raise CausalBaselineError("protocol hash is invalid")
    path = repository_root() / protocol["path"]
    try:
        data = path.read_bytes()
    except OSError as error:
        raise CausalBaselineError(f"cannot read bound protocol: {error}") from error
    if sha256_bytes(data) != protocol["sha256"]:
        raise CausalBaselineError("bound protocol hash drift")
    return protocol


def validate_profile(value: dict[str, Any]) -> dict[str, Any]:
    _exact_keys(value, TOP_KEYS, "profile")
    if (
        value["schema"] != PROFILE_SCHEMA
        or value["profile_id"] != PROFILE_ID
        or value["revision"] != 1
        or value["claim"] != CLAIM
    ):
        raise CausalBaselineError("profile identity or claim changed")
    if not isinstance(value["baseline_commit"], str) or not re.fullmatch(
        r"[0-9a-f]{40}", value["baseline_commit"]
    ):
        raise CausalBaselineError("baseline commit must be a full Git object ID")
    if value["authority"] != AUTHORITY:
        raise CausalBaselineError("external-only authority changed")
    if value["numeric_profile"] != NUMERIC_PROFILE:
        raise CausalBaselineError("deterministic numeric profile changed")
    if value["resources"] != RESOURCES:
        raise CausalBaselineError("resource envelope changed")
    if value["fallback"] != FALLBACK:
        raise CausalBaselineError("typed authored fallback changed")
    if value["formulae"] != FORMULAE:
        raise CausalBaselineError("analytic formula table changed")
    if value["modal_output"] != MODAL_OUTPUT:
        raise CausalBaselineError("modal output contract changed")
    _validate_protocol(value["protocol"])
    fixtures = value["fixtures"]
    if not isinstance(fixtures, list) or len(fixtures) != 2:
        raise CausalBaselineError("exactly two analytic fixtures are required")
    fixture_ids = {_validate_fixture(fixture) for fixture in fixtures}
    if fixture_ids != {"synthetic-plate-reference", "synthetic-beam-reference"}:
        raise CausalBaselineError("analytic fixture coverage changed")
    interventions = _validate_interventions(value["interventions"])
    if sha256_bytes(canonical_json(value)) != PROFILE_SHA256:
        raise CausalBaselineError("frozen P0 profile drift")
    return {
        "fixture_ids": sorted(fixture_ids),
        "interventions": interventions,
        "protocol": value["protocol"],
    }


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def external_output(path: Path) -> Path:
    root = repository_root()
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise CausalBaselineError("output must not be a symlink")
    try:
        parent = unresolved.parent.resolve(strict=True)
    except OSError as error:
        raise CausalBaselineError(f"output parent must exist: {error}") from error
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise CausalBaselineError("output must be a fresh external path")
    return output


def build_contract(
    profile: dict[str, Any], profile_data: bytes, validated: dict[str, Any]
) -> dict[str, Any]:
    owner_path = Path(__file__).resolve()
    owner_data = owner_path.read_bytes()
    fixtures = [
        {
            "family": fixture["family"],
            "fixture_id": fixture["fixture_id"],
            "formula_id": fixture["formula_id"],
            "mesh_pair": fixture["mesh_pair"],
            "mode_count": fixture["mode_count"],
            "support": fixture["support"],
        }
        for fixture in sorted(profile["fixtures"], key=lambda item: item["fixture_id"])
    ]
    return {
        "access": ZERO_ACCESS,
        "authority": profile["authority"],
        "baseline_commit": profile["baseline_commit"],
        "claim": CLAIM,
        "fallback": profile["fallback"],
        "fixtures": fixtures,
        "formulae": profile["formulae"],
        "interventions": validated["interventions"],
        "modal_output": profile["modal_output"],
        "numeric_profile": profile["numeric_profile"],
        "owner_identity": {
            "bytes": len(owner_data),
            "path": "lab/scripts/physical_sound_v31_p0_causal_baseline_v1.py",
            "sha256": sha256_bytes(owner_data),
        },
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "protocol": validated["protocol"],
        "resources": profile["resources"],
        "schema": CONTRACT_SCHEMA,
        "status": "ProtocolFrozen",
    }


def build_report(contract_data: bytes) -> dict[str, Any]:
    return {
        "access": ZERO_ACCESS,
        "authority": AUTHORITY,
        "claim": CLAIM,
        "contract_sha256": sha256_bytes(contract_data),
        "gates": {
            "analytic_formula_bindings": 3,
            "deterministic_numeric_profile": "Pass",
            "external_publication": "Pass",
            "fixture_count": 2,
            "isolated_intervention_count": 7,
            "modal_output_contract": "Pass",
            "resource_envelope": "Pass",
            "typed_fallback": "Pass",
            "zero_signal": "Pass",
        },
        "next_authorized_stage": "V31-P1-deterministic-modal-owner",
        "schema": REPORT_SCHEMA,
        "status": "ProtocolFrozen",
    }


def publish(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    validated = validate_profile(profile)
    output = external_output(output_path)
    contract_data = canonical_json(build_contract(profile, profile_data, validated))
    report = build_report(contract_data)
    report_data = canonical_json(report)
    if len(contract_data) + len(report_data) > RESOURCES["max_output_bytes"]:
        raise CausalBaselineError("contract publication exceeds the output envelope")

    temporary = Path(
        tempfile.mkdtemp(prefix=f".{output.name}.tmp-", dir=str(output.parent))
    )
    try:
        (temporary / "contract.json").write_bytes(contract_data)
        (temporary / "report.json").write_bytes(report_data)
        os.replace(temporary, output)
    except BaseException:
        shutil.rmtree(temporary, ignore_errors=True)
        raise
    return report


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = publish(arguments.profile, arguments.output)
    except CausalBaselineError as error:
        print(f"P0_REJECT: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
