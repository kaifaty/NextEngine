#!/usr/bin/env python3
"""Validate the frozen V32 M0 physics-locked residual owner without M1 values."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
import shutil
import tempfile
from decimal import Decimal
from pathlib import Path
from typing import Any

import numpy as np
import torch
from torch import nn

import physical_sound_v31_p1_modal_owner_v1 as p1


PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-m0-physics-locked-residual-profile.v1"
)
PROFILE_ID = "physical-sound-v32-m0-physics-locked-residual-v1"
PROFILE_SHA256 = "dd77fa3ee416061777775ddc00cf3eb4b5fc32d03574a569ca06935d4e23973d"
BASELINE_COMMIT = "30dfa03c6418ad850ab093046c5524ca30528427"
CLAIM = (
    "SYNTHETIC_PHYSICS_LOCKED_RESIDUAL_REPRESENTATION_ONLY / NO_REAL_MATERIAL_"
    "QUALITY_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-m0-owner-conformance-report.v1"
)
EVIDENCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-m0-owner-conformance-evidence.v1"
)
CONFORMANCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v32-m0-owner-conformance.v1"
)
OWNER_PATH = "lab/scripts/physical_sound_v32_m0_physics_locked_residual_v1.py"
MAX_PROFILE_BYTES = 256 * 1024

AUTHORITY = {
    "admission_authority": False,
    "authored_fallback_required": True,
    "external_research_only": True,
    "model_allowed": True,
    "network_allowed": False,
    "quality_authority": False,
    "real_material_authority": False,
    "real_signal_allowed": False,
    "runtime_authority": False,
    "synthetic_only": True,
    "validator_release_authority": False,
}
ZERO_ACCESS = {
    "development_rows_materialized": 0,
    "method_holdout_rows_materialized": 0,
    "network_requests": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
}


class M0ConformanceError(RuntimeError):
    """The frozen M0 owner cannot establish value-independent conformance."""


def canonical_json(value: Any) -> bytes:
    try:
        return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()
    except (TypeError, ValueError) as error:
        raise M0ConformanceError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    if path.is_symlink():
        raise M0ConformanceError("profile must not be a symlink")
    data = path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise M0ConformanceError("profile size outside bound")
    try:
        profile = json.loads(data)
    except json.JSONDecodeError as error:
        raise M0ConformanceError(f"invalid profile JSON: {error}") from error
    if data != canonical_json(profile):
        raise M0ConformanceError("profile is not canonical JSON")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise M0ConformanceError("profile hash mismatch")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("baseline_commit") != BASELINE_COMMIT
        or profile.get("claim") != CLAIM
        or profile.get("authority") != AUTHORITY
    ):
        raise M0ConformanceError("profile identity or authority mismatch")
    if profile["model"]["parameter_count"] != 1491:
        raise M0ConformanceError("model parameter contract mismatch")
    return profile, data


def validate_dependencies(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    root = repository_root()
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, declared in sorted(profile["parent"].items()):
        data = (root / declared["path"]).read_bytes()
        actual = sha256_bytes(data)
        if actual != declared["sha256"]:
            raise M0ConformanceError(f"dependency drift: {dependency_id}")
        result[dependency_id] = {"bytes": len(data), **declared}
    for dependency_id in ("protocol", "research"):
        declared = profile[dependency_id]
        data = (root / declared["path"]).read_bytes()
        if sha256_bytes(data) != declared["sha256"]:
            raise M0ConformanceError(f"dependency drift: {dependency_id}")
        result[dependency_id] = {"bytes": len(data), **declared}
    return result


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise M0ConformanceError("output must not be a symlink")
    parent = unresolved.parent.resolve(strict=True)
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise M0ConformanceError("output must be a fresh external path")
    return output


def decimal_product(value: str, multiplier: str) -> str:
    product = Decimal(value) * Decimal(multiplier)
    return format(product, "f")


def smoke_fixture(profile: dict[str, Any], p0_profile: dict[str, Any]) -> dict[str, Any]:
    fixture = copy.deepcopy(p0_profile["fixtures"][0])
    fixture["fixture_id"] = "m0-nonofficial-conformance-plate"
    material = profile["corpus"]["materials"][1]
    fixture["material"] = {
        "density_kg_m3": material["density_kg_m3"],
        "loss_rate_per_second": material["loss_rate_per_second"],
        "material_id": "synthetic-elastic-reference",
        "poisson_ratio": material["poisson_ratio"],
        "youngs_modulus_pa": material["youngs_modulus_pa"],
    }
    multipliers = ("1.05", "0.95", "1.05")
    for field, multiplier in zip(
        ("length_x_m", "length_y_m", "thickness_m"), multipliers, strict=True
    ):
        fixture["geometry"][field] = decimal_product(
            fixture["geometry"][field], multiplier
        )
    fixture["contact"] = {"normal_impulse_ns": "1", "u": "0.3", "v": "0.4"}
    return fixture


def one_hot_context(fixture: dict[str, Any]) -> tuple[list[float], list[float], float, float]:
    family = fixture["family"]
    support = fixture["support"]
    family_values = [1.0, 0.0] if family == "rectangular_plate" else [0.0, 1.0]
    support_values = [
        float(support == "simply-supported-all-edges"),
        float(support == "cantilever-clamped-u0"),
        float(support == "simply-supported-both-ends"),
    ]
    family_code = -1.0 if family == "rectangular_plate" else 1.0
    support_code = {-1.0: support_values[0], 1.0: support_values[2]}
    signed_support = support_code[1.0] - support_code[-1.0]
    return family_values, support_values, family_code, signed_support


def normalized_row(
    fixture: dict[str, Any],
    mode: dict[str, Any],
    geometry_multipliers: tuple[float, float, float],
) -> dict[str, Any]:
    material = fixture["material"]
    ordinal = 2.0 * float(mode["ordinal"]) / 9.0 - 1.0
    log_frequency = max(
        -1.5, min(1.5, math.log2(float(mode["frequency_hz"]) / 1000.0) / 4.0)
    )
    index_a = 2.0 * (float(mode["family_index_a"]) - 1.0) / 9.0 - 1.0
    index_b = (
        -1.0
        if mode["family_index_b"] == 0
        else 2.0 * (float(mode["family_index_b"]) - 1.0) / 9.0 - 1.0
    )
    mode_features = [ordinal, log_frequency, index_a, index_b]
    material_features = [
        math.log2(float(material["youngs_modulus_pa"]) / 69_000_000_000.0) / 2.0,
        math.log2(float(material["density_kg_m3"]) / 2700.0) / 2.0,
        (float(material["poisson_ratio"]) - 0.315) / 0.035,
        (float(material["loss_rate_per_second"]) - 12.0) / 6.0,
    ]
    family, support, family_code, support_code = one_hot_context(fixture)
    geometry = [math.log(value) / math.log(1.25) for value in geometry_multipliers]
    u = float(fixture["contact"]["u"])
    v = float(fixture["contact"]["v"])
    contact = [2.0 * u - 1.0, 2.0 * v - 1.0]
    return {
        "contact": np.asarray(
            mode_features
            + family
            + support
            + contact
            + [float(mode["contact_participation"])],
            dtype=np.float64,
        ),
        "decay": np.asarray(mode_features + material_features + support, dtype=np.float64),
        "global_gain": np.asarray(
            mode_features + material_features + family + geometry, dtype=np.float64
        ),
        "oracle_context": {
            "density": material_features[1],
            "family": family_code,
            "geometry": geometry,
            "log_frequency": log_frequency,
            "loss": material_features[3],
            "ordinal": ordinal,
            "p1_contact": float(mode["contact_participation"]),
            "support": support_code,
            "u": u,
            "v": v,
            "youngs": material_features[0],
        },
    }


def oracle_targets(row: dict[str, Any]) -> np.ndarray:
    c = row["oracle_context"]
    decay = 0.20 * math.tanh(
        0.55 * c["loss"]
        + 0.25 * c["log_frequency"]
        + 0.20 * c["support"] * c["ordinal"]
        + 0.15 * c["youngs"] * c["density"]
    )
    gain = 0.16 * math.tanh(
        0.45 * c["ordinal"] * c["geometry"][0]
        + 0.35 * c["geometry"][2]
        + 0.25 * c["youngs"] * c["ordinal"]
        - 0.20 * c["density"]
        + 0.15 * c["family"] * c["geometry"][1]
    )
    contact = 0.22 * math.tanh(
        0.50 * c["p1_contact"]
        + 0.35 * math.sin(2.0 * math.pi * c["u"]) * math.cos(math.pi * c["v"])
        + 0.25 * c["ordinal"] * c["support"]
        + 0.15 * c["family"] * (2.0 * c["u"] - 1.0)
    )
    return np.asarray([decay, gain, contact], dtype=np.float64)


class ResidualModel(nn.Module):
    def __init__(self, profile: dict[str, Any]) -> None:
        super().__init__()
        self.decay = self._head(11, 0.25)
        self.global_gain = self._head(13, 0.20)
        self.contact = self._head(12, 0.25)
        self._initialize(profile["model"]["initializer"]["seed"])

    @staticmethod
    def _head(inputs: int, bound: float) -> nn.Sequential:
        return nn.Sequential(
            nn.Linear(inputs, 16, dtype=torch.float64),
            nn.SiLU(),
            nn.Linear(16, 16, dtype=torch.float64),
            nn.SiLU(),
            nn.Linear(16, 1, dtype=torch.float64),
            nn.Tanh(),
            BoundScale(bound),
        )

    def _initialize(self, seed: int) -> None:
        generator = np.random.Generator(np.random.PCG64(seed))
        with torch.no_grad():
            for name, parameter in sorted(self.named_parameters()):
                if name.endswith("bias"):
                    parameter.zero_()
                else:
                    fan_out, fan_in = parameter.shape
                    limit = math.sqrt(6.0 / float(fan_in + fan_out))
                    values = generator.uniform(-limit, limit, size=tuple(parameter.shape))
                    parameter.copy_(torch.from_numpy(values))


class BoundScale(nn.Module):
    def __init__(self, bound: float) -> None:
        super().__init__()
        self.bound = bound

    def forward(self, values: torch.Tensor) -> torch.Tensor:
        return values * self.bound


def parameter_count(model: nn.Module) -> int:
    return sum(parameter.numel() for parameter in model.parameters())


def compose_modes(
    modes: list[dict[str, Any]], corrections: np.ndarray
) -> list[dict[str, float | int]]:
    result: list[dict[str, float | int]] = []
    for mode, correction in zip(modes, corrections, strict=True):
        decay_log = float(np.clip(correction[0], -0.25, 0.25))
        gain_log = float(np.clip(correction[1], -0.20, 0.20))
        contact_log = float(np.clip(correction[2], -0.25, 0.25))
        contact = float(mode["contact_participation"]) * math.exp(contact_log)
        signed_gain = contact * float(mode["pickup_participation"]) * math.exp(gain_log)
        result.append(
            {
                "decay_per_second": float(mode["decay_per_second"])
                * math.exp(decay_log),
                "frequency_hz": mode["frequency_hz"],
                "ordinal": mode["ordinal"],
                "signed_gain": signed_gain,
            }
        )
    return result


def build_conformance(profile: dict[str, Any], profile_data: bytes) -> dict[str, Any]:
    dependencies = validate_dependencies(profile)
    p0_profile, _ = p1.p0.load_profile(repository_root() / profile["parent"]["p0_profile"]["path"])
    fixture = smoke_fixture(profile, p0_profile)
    solution = p1.solve_case("m0-nonofficial-smoke", fixture, p0_profile)
    modes = solution["modal_document"]["modes"]
    rows = [normalized_row(fixture, mode, (1.05, 0.95, 1.05)) for mode in modes]
    targets = np.stack([oracle_targets(row) for row in rows])
    model = ResidualModel(profile)
    if parameter_count(model) != profile["model"]["parameter_count"]:
        raise M0ConformanceError("parameter count mismatch")
    torch.use_deterministic_algorithms(True)
    torch.set_num_threads(1)
    torch.set_num_interop_threads(1)
    optimizer = torch.optim.AdamW(
        model.parameters(), lr=0.003, weight_decay=0.000001
    )
    tensors = {
        name: torch.from_numpy(np.stack([row[name] for row in rows]))
        for name in ("decay", "global_gain", "contact")
    }
    target_tensor = torch.from_numpy(targets)
    losses: list[float] = []
    for _ in range(3):
        optimizer.zero_grad(set_to_none=True)
        predictions = torch.cat(
            [
                model.decay(tensors["decay"]),
                model.global_gain(tensors["global_gain"]),
                model.contact(tensors["contact"]),
            ],
            dim=1,
        )
        scales = torch.tensor([0.20, 0.16, 0.22], dtype=torch.float64)
        loss = torch.mean(((predictions - target_tensor) / scales) ** 2)
        loss.backward()
        torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
        optimizer.step()
        losses.append(float(loss.detach()))
    with torch.no_grad():
        predicted = torch.cat(
            [
                model.decay(tensors["decay"]),
                model.global_gain(tensors["global_gain"]),
                model.contact(tensors["contact"]),
            ],
            dim=1,
        ).numpy()
    composed = compose_modes(modes, predicted)
    frequency_exact = all(
        item["frequency_hz"] == mode["frequency_hz"]
        for item, mode in zip(composed, modes, strict=True)
    )
    zero_exact = all(
        mode["contact_participation"] != 0.0 or item["signed_gain"] == 0.0
        for item, mode in zip(composed, modes, strict=True)
    )
    sign_preserved = all(
        item["signed_gain"] == 0.0
        or math.copysign(1.0, float(item["signed_gain"]))
        == math.copysign(1.0, float(mode["signed_gain"]))
        for item, mode in zip(composed, modes, strict=True)
    )
    bounded = bool(
        np.all(np.abs(predicted[:, 0]) <= 0.25)
        and np.all(np.abs(predicted[:, 1]) <= 0.20)
        and np.all(np.abs(predicted[:, 2]) <= 0.25)
    )
    if not all((frequency_exact, zero_exact, sign_preserved, bounded)):
        raise M0ConformanceError("physics-locked composition invariant failed")
    owner_data = Path(__file__).read_bytes()
    return {
        "access": ZERO_ACCESS,
        "claim": CLAIM,
        "corpus_count_algebra": profile["corpus"]["counts"],
        "dependencies": dependencies,
        "gates": {
            "branch_input_counts": "11/13/12 Pass",
            "correction_bounds": "Pass",
            "frequency_passthrough": "10/10 Exact",
            "nodal_zero_preservation": "Pass",
            "parameter_count": "1491 Exact",
            "signed_gain_sign_preservation": "Pass",
            "smoke_steps": "3/3 Complete",
        },
        "owner_identity": {
            "bytes": len(owner_data),
            "path": OWNER_PATH,
            "sha256": sha256_bytes(owner_data),
        },
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "schema": CONFORMANCE_SCHEMA,
        "smoke": {
            "final_loss": losses[-1],
            "initial_loss": losses[0],
            "mode_count": len(modes),
            "official_metric": False,
            "values_discarded": True,
        },
        "status": "Pass",
    }


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    path = root / name
    path.write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    profile, profile_data = load_profile(profile_path)
    output = external_output(output_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v32-m0-", dir=output.parent))
    try:
        conformance = build_conformance(profile, profile_data)
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        evidence = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "conformance_sha256": conformance_ref["sha256"],
            "owner_identity": conformance["owner_identity"],
            "profile_identity": conformance["profile_identity"],
            "schema": EVIDENCE_SCHEMA,
            "status": "Pass",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "access": ZERO_ACCESS,
            "claim": CLAIM,
            "decision": "M0_OWNER_CONFORMANCE_PASS",
            "evidence_sha256": evidence_ref["sha256"],
            "next_authorized_stage": "V32-M1-one-shot-known-truth-tournament",
            "official_candidate_values_opened": False,
            "schema": REPORT_SCHEMA,
            "status": "Pass",
        }
        write_bytes(staging, "report.json", canonical_json(report))
        files = [path for path in staging.iterdir() if path.is_file()]
        if len(files) != 3:
            raise M0ConformanceError("conformance publication closure mismatch")
        os.replace(staging, output)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.profile, arguments.output)
    except Exception as error:  # noqa: BLE001
        print(f"physical-sound-v32-m0: ContractReject: {error}", file=os.sys.stderr)
        return 2
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
