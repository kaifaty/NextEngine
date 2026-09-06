#!/usr/bin/env python3
"""Build the V31 P1 deterministic synthetic modal owner outside the repository."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import math
import os
import platform
import shutil
import struct
import sys
import tempfile
import time
from decimal import Decimal, InvalidOperation
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_v24_t0_teacher as v24
import physical_sound_v31_p0_causal_baseline_v1 as p0


EVIDENCE_SCHEMA = (
    "nextengine.experimental-physical-sound-v31-p1-modal-owner-evidence.v1"
)
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v31-p1-modal-owner-report.v1"
MODAL_SCHEMA = "nextengine.experimental-physical-sound-v31-p1-modal-records.v1"
CLAIM = (
    "SYNTHETIC_DETERMINISTIC_MODAL_OWNER_ONLY / NO_REAL_MATERIAL_QUALITY_"
    "VALIDATOR_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
P0_OWNER_SHA256 = "47f548dac2de3e1dbdde30c44d59a9f466bc1f02bf78b469e61ee3cfaa7be374"
V24_OWNER_SHA256 = "ef5458df64272714f8aaff38b004bd1320bf2a6fa2c58b587505146b18b73617"
MESH_MAGIC = b"NEP1MSH1"
GAIN_MAGIC = b"NEP1GNA1"
RELATIVE_TOLERANCE = 1.0e-12

ZERO_ACCESS = {
    "audio_headers_parsed": 0,
    "audio_sample_values_decoded": 0,
    "model_parameters_opened": 0,
    "network_requests": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "waveform_or_feature_values_decoded": 0,
}


class ModalOwnerError(RuntimeError):
    """P1 cannot establish the complete frozen modal-owner result."""


class OutOfDomain(RuntimeError):
    """One candidate input must select the typed authored fallback."""

    def __init__(self, reason_code: str) -> None:
        super().__init__(reason_code)
        self.reason_code = reason_code


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise ModalOwnerError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_dependencies() -> dict[str, dict[str, Any]]:
    dependencies = {
        "p0_contract_owner": (
            Path(p0.__file__).resolve(),
            "lab/scripts/physical_sound_v31_p0_causal_baseline_v1.py",
            P0_OWNER_SHA256,
        ),
        "v24_analytic_teacher": (
            Path(v24.__file__).resolve(),
            "lab/scripts/physical_sound_v24_t0_teacher.py",
            V24_OWNER_SHA256,
        ),
    }
    result: dict[str, dict[str, Any]] = {}
    for dependency_id, (path, relative, expected) in dependencies.items():
        data = path.read_bytes()
        actual = sha256_bytes(data)
        if actual != expected:
            raise ModalOwnerError(f"dependency drift: {dependency_id}")
        result[dependency_id] = {
            "bytes": len(data),
            "path": relative,
            "sha256": actual,
        }
    return result


def external_output(path: Path) -> Path:
    root = repository_root().resolve(strict=True)
    unresolved = path if path.is_absolute() else root / path
    if unresolved.is_symlink():
        raise ModalOwnerError("output must not be a symlink")
    try:
        parent = unresolved.parent.resolve(strict=True)
    except OSError as error:
        raise ModalOwnerError(f"output parent must exist: {error}") from error
    output = parent / unresolved.name
    if output.is_relative_to(root) or output.exists():
        raise ModalOwnerError("output must be a fresh external path")
    return output


def decimal_value(value: Any, reason: str = "InvalidNumericInput") -> Decimal:
    if not isinstance(value, str):
        raise OutOfDomain(reason)
    try:
        result = Decimal(value)
    except InvalidOperation as error:
        raise OutOfDomain(reason) from error
    if not result.is_finite():
        raise OutOfDomain(reason)
    return result


def positive_float(value: Any) -> float:
    result = decimal_value(value)
    if result <= 0:
        raise OutOfDomain("InvalidNumericInput")
    converted = float(result)
    if not math.isfinite(converted) or converted <= 0.0:
        raise OutOfDomain("InvalidNumericInput")
    return converted


def unit_float(value: Any, reason: str = "UnsupportedContact") -> float:
    result = decimal_value(value)
    if result < 0 or result > 1:
        raise OutOfDomain(reason)
    converted = float(result)
    if not math.isfinite(converted):
        raise OutOfDomain("InvalidNumericInput")
    return converted


def formula_table(profile: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {formula["formula_id"]: formula for formula in profile["formulae"]}


def validate_fixture(
    fixture: dict[str, Any], profile: dict[str, Any]
) -> dict[str, Any]:
    if not isinstance(fixture, dict):
        raise OutOfDomain("UnsupportedGeometry")
    family = fixture.get("family")
    if family not in {"rectangular_plate", "rectangular_beam"}:
        raise OutOfDomain("UnsupportedGeometry")

    material = fixture.get("material")
    if not isinstance(material, dict) or material.get("material_id") != (
        "synthetic-elastic-reference"
    ):
        raise OutOfDomain("UnsupportedAcousticMaterial")
    density = positive_float(material.get("density_kg_m3"))
    youngs = positive_float(material.get("youngs_modulus_pa"))
    poisson = positive_float(material.get("poisson_ratio"))
    loss = positive_float(material.get("loss_rate_per_second"))
    if (
        not 100.0 <= density <= 50_000.0
        or not 1.0e6 <= youngs <= 1.0e12
        or not 0.0 < poisson < 0.5
        or loss > 10_000.0
    ):
        raise OutOfDomain("UnsupportedAcousticMaterial")

    geometry = fixture.get("geometry")
    if not isinstance(geometry, dict):
        raise OutOfDomain("UnsupportedGeometry")
    if family == "rectangular_plate":
        geometry_keys = {"length_x_m", "length_y_m", "thickness_m"}
        valid_binding = (
            fixture.get("support") == "simply-supported-all-edges"
            and fixture.get("formula_id") == "kirchhoff-love-simply-supported-v1"
        )
    else:
        geometry_keys = {"height_m", "length_m", "width_m"}
        valid_binding = (
            fixture.get("support") == "cantilever-clamped-u0"
            and fixture.get("formula_id") == "euler-bernoulli-cantilever-v1"
        ) or (
            fixture.get("support") == "simply-supported-both-ends"
            and fixture.get("formula_id") == "euler-bernoulli-simply-supported-v1"
        )
    if set(geometry) != geometry_keys:
        raise OutOfDomain("UnsupportedGeometry")
    dimensions = {key: positive_float(value) for key, value in geometry.items()}
    if any(value > 10.0 for value in dimensions.values()):
        raise OutOfDomain("UnsupportedGeometry")
    if not valid_binding or fixture.get("formula_id") not in formula_table(profile):
        raise OutOfDomain("UnsupportedSupport")

    contact = fixture.get("contact")
    pickup = fixture.get("pickup")
    if not isinstance(contact, dict) or not isinstance(pickup, dict):
        raise OutOfDomain("UnsupportedContact")
    contact_values = {
        "u": unit_float(contact.get("u")),
        "v": unit_float(contact.get("v")),
        "normal_impulse_ns": positive_float(contact.get("normal_impulse_ns")),
    }
    pickup_values = {
        "u": unit_float(pickup.get("u")),
        "v": unit_float(pickup.get("v")),
    }

    mode_count = fixture.get("mode_count")
    mesh_pair = fixture.get("mesh_pair")
    resources = profile["resources"]
    if type(mode_count) is not int or mode_count < 1:
        raise OutOfDomain("InvalidNumericInput")
    if mode_count > resources["max_modes"] or mode_count > len(v24.BEAM_ROOTS):
        raise OutOfDomain("ResourceLimitExceeded")
    if not isinstance(mesh_pair, dict) or set(mesh_pair) != {
        "coarse_u",
        "coarse_v",
        "fine_u",
        "fine_v",
    }:
        raise OutOfDomain("UnsupportedGeometry")
    if any(type(mesh_pair[key]) is not int or mesh_pair[key] < 3 for key in mesh_pair):
        raise OutOfDomain("InvalidNumericInput")
    if (
        mesh_pair["fine_u"] != 2 * mesh_pair["coarse_u"] - 1
        or mesh_pair["fine_v"] != 2 * mesh_pair["coarse_v"] - 1
    ):
        raise OutOfDomain("UnsupportedGeometry")
    vertices = mesh_pair["fine_u"] * mesh_pair["fine_v"]
    elements = 2 * (mesh_pair["fine_u"] - 1) * (mesh_pair["fine_v"] - 1)
    if (
        vertices > resources["max_mesh_vertices"]
        or elements > resources["max_mesh_elements"]
        or profile["numeric_profile"]["render_frames"]
        > resources["max_render_frames"]
    ):
        raise OutOfDomain("ResourceLimitExceeded")

    return {
        "contact": contact_values,
        "dimensions": dimensions,
        "family": family,
        "fixture_id": fixture.get("fixture_id"),
        "formula_id": fixture.get("formula_id"),
        "loss_rate_per_second": loss,
        "material": {
            "density_kg_m3": density,
            "poisson_ratio": poisson,
            "youngs_modulus_pa": youngs,
        },
        "mesh_pair": dict(mesh_pair),
        "mode_count": mode_count,
        "pickup": pickup_values,
        "support": fixture.get("support"),
    }


def plate_sine(index: int, values: np.ndarray | float) -> np.ndarray:
    phase = np.asarray(values, dtype=np.float64) * float(index)
    result = np.sin(math.pi * phase)
    result = np.where(phase == np.rint(phase), 0.0, result)
    return np.asarray(result, dtype=np.float64)


def cantilever_shape(root: float, values: np.ndarray | float) -> np.ndarray:
    return np.asarray(v24.beam_shape(root, values), dtype=np.float64)


def simply_supported_beam_shape(
    ordinal: int, values: np.ndarray | float
) -> np.ndarray:
    return plate_sine(ordinal, values)


def solve_modes(
    fixture: dict[str, Any], profile: dict[str, Any]
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    validated = validate_fixture(fixture, profile)
    material = validated["material"]
    mode_count = validated["mode_count"]
    dimensions = validated["dimensions"]
    if validated["family"] == "rectangular_plate":
        length_x = dimensions["length_x_m"]
        length_y = dimensions["length_y_m"]
        thickness = dimensions["thickness_m"]
        rigidity = material["youngs_modulus_pa"] * thickness**3 / (
            12.0 * (1.0 - material["poisson_ratio"] ** 2)
        )
        scale = math.pi**2 * math.sqrt(
            rigidity / (material["density_kg_m3"] * thickness)
        )
        candidates = []
        for m in range(1, 11):
            for n in range(1, 11):
                frequency = (
                    scale
                    * ((m / length_x) ** 2 + (n / length_y) ** 2)
                    / (2.0 * math.pi)
                )
                candidates.append((frequency, m, n))
        candidates.sort()
        selected = candidates[:mode_count]
        frequencies = np.asarray([item[0] for item in selected], dtype=np.float64)
        indices = np.asarray([[item[1], item[2]] for item in selected], dtype=np.uint32)
    else:
        length = dimensions["length_m"]
        width = dimensions["width_m"]
        height = dimensions["height_m"]
        area = width * height
        inertia = width * height**3 / 12.0
        scale = math.sqrt(
            material["youngs_modulus_pa"]
            * inertia
            / (material["density_kg_m3"] * area)
        )
        formula = formula_table(profile)[validated["formula_id"]]
        if validated["support"] == "cantilever-clamped-u0":
            roots = np.asarray(
                [float(value) for value in formula["beta_roots"][:mode_count]],
                dtype=np.float64,
            )
        else:
            roots = math.pi * np.arange(1, mode_count + 1, dtype=np.float64)
        frequencies = roots**2 / length**2 * scale / (2.0 * math.pi)
        indices = np.asarray(
            [[ordinal, 0] for ordinal in range(1, mode_count + 1)],
            dtype=np.uint32,
        )
    decay = np.full(mode_count, validated["loss_rate_per_second"], dtype=np.float64)
    angular = 2.0 * math.pi * frequencies
    if (
        np.any(~np.isfinite(frequencies))
        or np.any(frequencies <= 0.0)
        or np.any(np.diff(frequencies) <= 0.0)
        or np.any(decay <= 0.0)
        or np.any(decay >= angular)
    ):
        raise OutOfDomain("InvalidNumericInput")
    return frequencies, decay, indices


def participation(
    validated: dict[str, Any],
    profile: dict[str, Any],
    indices: np.ndarray,
    u: np.ndarray | float,
    v: np.ndarray | float,
) -> np.ndarray:
    u_values = np.asarray(u, dtype=np.float64)
    if validated["family"] == "rectangular_plate":
        columns = [
            plate_sine(int(m), u_values)
            * plate_sine(int(n), np.asarray(v, dtype=np.float64))
            for m, n in indices
        ]
    elif validated["support"] == "cantilever-clamped-u0":
        formula = formula_table(profile)[validated["formula_id"]]
        columns = [
            cantilever_shape(float(formula["beta_roots"][int(ordinal) - 1]), u_values)
            for ordinal, _ in indices
        ]
    else:
        columns = [
            simply_supported_beam_shape(int(ordinal), u_values)
            for ordinal, _ in indices
        ]
    return np.column_stack(columns)


def make_mesh(
    validated: dict[str, Any], grid_u: int, grid_v: int
) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    vertices: list[tuple[float, float, float]] = []
    normalized: list[tuple[float, float]] = []
    dimensions = validated["dimensions"]
    for v_index in range(grid_v):
        v = v_index / (grid_v - 1)
        for u_index in range(grid_u):
            u = u_index / (grid_u - 1)
            normalized.append((u, v))
            if validated["family"] == "rectangular_plate":
                vertices.append(
                    (
                        dimensions["length_x_m"] * u,
                        dimensions["length_y_m"] * v,
                        dimensions["thickness_m"] / 2.0,
                    )
                )
            else:
                vertices.append(
                    (
                        dimensions["length_m"] * u,
                        dimensions["width_m"] * (v - 0.5),
                        dimensions["height_m"] / 2.0,
                    )
                )
    triangles: list[tuple[int, int, int]] = []
    for v_index in range(grid_v - 1):
        for u_index in range(grid_u - 1):
            lower = v_index * grid_u + u_index
            triangles.append((lower, lower + 1, lower + grid_u))
            triangles.append((lower + 1, lower + grid_u + 1, lower + grid_u))
    return (
        np.asarray(vertices, dtype=np.float64),
        np.asarray(triangles, dtype=np.uint32),
        np.asarray(normalized, dtype=np.float64),
    )


def encode_mesh(vertices: np.ndarray, triangles: np.ndarray) -> bytes:
    if (
        vertices.ndim != 2
        or vertices.shape[1] != 3
        or triangles.ndim != 2
        or triangles.shape[1] != 3
        or np.any(~np.isfinite(vertices))
        or np.any(triangles >= len(vertices))
    ):
        raise ModalOwnerError("invalid P1 mesh payload")
    return (
        MESH_MAGIC
        + struct.pack("<III", 1, len(vertices), len(triangles))
        + vertices.astype("<f8", copy=False).tobytes()
        + triangles.astype("<u4", copy=False).tobytes()
    )


def encode_gains(gains: np.ndarray) -> bytes:
    if gains.ndim != 2 or gains.size == 0 or np.any(~np.isfinite(gains)):
        raise ModalOwnerError("invalid P1 gain payload")
    return (
        GAIN_MAGIC
        + struct.pack("<III", 1, gains.shape[0], gains.shape[1])
        + gains.astype("<f8", copy=False).tobytes()
    )


def encode_float32_wav(samples: np.ndarray, sample_rate_hz: int) -> bytes:
    if (
        samples.ndim != 1
        or len(samples) == 0
        or np.any(~np.isfinite(samples))
        or float(np.max(np.abs(samples))) >= 1.0
    ):
        raise ModalOwnerError("P1 render is empty, non-finite or clipped")
    payload = samples.astype("<f4", copy=False).tobytes()
    fmt = struct.pack("<HHIIHH", 3, 1, sample_rate_hz, sample_rate_hz * 4, 4, 32)
    return (
        b"RIFF"
        + struct.pack("<I", 4 + 8 + len(fmt) + 8 + len(payload))
        + b"WAVEfmt "
        + struct.pack("<I", len(fmt))
        + fmt
        + b"data"
        + struct.pack("<I", len(payload))
        + payload
    )


def solve_case(
    case_id: str, fixture: dict[str, Any], profile: dict[str, Any]
) -> dict[str, Any]:
    validated = validate_fixture(fixture, profile)
    frequencies, decay, indices = solve_modes(fixture, profile)
    contact = validated["contact"]
    pickup = validated["pickup"]
    contact_participation = participation(
        validated, profile, indices, contact["u"], contact["v"]
    )[0]
    pickup_participation = participation(
        validated, profile, indices, pickup["u"], pickup["v"]
    )[0]
    signed_gains = contact_participation * pickup_participation

    mesh_pair = validated["mesh_pair"]
    coarse_vertices, coarse_triangles, coarse_uv = make_mesh(
        validated, mesh_pair["coarse_u"], mesh_pair["coarse_v"]
    )
    fine_vertices, fine_triangles, fine_uv = make_mesh(
        validated, mesh_pair["fine_u"], mesh_pair["fine_v"]
    )
    coarse_field = participation(
        validated, profile, indices, coarse_uv[:, 0], coarse_uv[:, 1]
    ) * pickup_participation[None, :]
    fine_field = participation(
        validated, profile, indices, fine_uv[:, 0], fine_uv[:, 1]
    ) * pickup_participation[None, :]
    fine_common = np.asarray(
        [
            (2 * v_index) * mesh_pair["fine_u"] + 2 * u_index
            for v_index in range(mesh_pair["coarse_v"])
            for u_index in range(mesh_pair["coarse_u"])
        ],
        dtype=np.int64,
    )
    if not np.array_equal(coarse_field, fine_field[fine_common]):
        raise ModalOwnerError(f"P1 remesh drift: {case_id}")

    numeric = profile["numeric_profile"]
    frames = numeric["render_frames"]
    sample_rate_hz = numeric["sample_rate_hz"]
    render_scale = positive_float(numeric["render_scale_per_ns"])
    seconds = np.arange(frames, dtype=np.float64) / float(sample_rate_hz)
    modal_sum = np.zeros(frames, dtype=np.float64)
    for frequency, rate, gain in zip(
        frequencies, decay, signed_gains, strict=True
    ):
        modal_sum += float(gain) * np.exp(-float(rate) * seconds) * np.sin(
            2.0 * math.pi * float(frequency) * seconds
        )
    unit_response = render_scale * modal_sum
    samples = contact["normal_impulse_ns"] * unit_response
    peak_bound = (
        render_scale
        * contact["normal_impulse_ns"]
        * float(np.sum(np.abs(signed_gains)))
    )
    sample_peak = float(np.max(np.abs(samples)))
    initial_envelope_energy = float(
        np.sum((render_scale * contact["normal_impulse_ns"] * signed_gains) ** 2)
    )
    final_scale = np.exp(-decay * seconds[-1])
    final_envelope_energy = float(
        np.sum(
            (
                render_scale
                * contact["normal_impulse_ns"]
                * signed_gains
                * final_scale
            )
            ** 2
        )
    )
    if (
        np.any(~np.isfinite(samples))
        or not 0.0 <= sample_peak <= peak_bound + RELATIVE_TOLERANCE
        or peak_bound >= 1.0
        or not 0.0 <= final_envelope_energy <= initial_envelope_energy
    ):
        raise ModalOwnerError(f"P1 energy envelope failed: {case_id}")

    modes = [
        {
            "contact_participation": float(contact_participation[ordinal]),
            "decay_per_second": float(decay[ordinal]),
            "family_index_a": int(indices[ordinal, 0]),
            "family_index_b": int(indices[ordinal, 1]),
            "frequency_hz": float(frequencies[ordinal]),
            "ordinal": ordinal,
            "pickup_participation": float(pickup_participation[ordinal]),
            "signed_gain": float(signed_gains[ordinal]),
        }
        for ordinal in range(len(frequencies))
    ]
    modal_document = {
        "case_id": case_id,
        "claim": CLAIM,
        "fixture_id": validated["fixture_id"],
        "formula_id": validated["formula_id"],
        "modes": modes,
        "schema": MODAL_SCHEMA,
        "support": validated["support"],
    }
    return {
        "case_id": case_id,
        "coarse_field": coarse_field,
        "coarse_mesh": encode_mesh(coarse_vertices, coarse_triangles),
        "decay": decay,
        "fine_field": fine_field,
        "fine_mesh": encode_mesh(fine_vertices, fine_triangles),
        "frequencies": frequencies,
        "indices": indices,
        "metrics": {
            "analytic_peak_bound": peak_bound,
            "final_modal_envelope_energy": final_envelope_energy,
            "initial_modal_envelope_energy": initial_envelope_energy,
            "remesh_common_vertices_exact": True,
            "sample_energy": float(np.sum(samples * samples)),
            "sample_peak": sample_peak,
        },
        "modal_document": modal_document,
        "samples": samples,
        "signed_gains": signed_gains,
        "validated": validated,
    }


def decimal_product(value: str, factor: str) -> str:
    result = decimal_value(value) * decimal_value(factor)
    return format(result, "f")


def apply_intervention(
    fixture: dict[str, Any], intervention: dict[str, Any]
) -> dict[str, Any]:
    result = copy.deepcopy(fixture)
    axis = intervention["axis"]
    if axis == "youngs_modulus":
        result["material"]["youngs_modulus_pa"] = decimal_product(
            result["material"]["youngs_modulus_pa"], intervention["value"]
        )
    elif axis == "density":
        result["material"]["density_kg_m3"] = decimal_product(
            result["material"]["density_kg_m3"], intervention["value"]
        )
    elif axis == "thickness":
        result["geometry"]["thickness_m"] = decimal_product(
            result["geometry"]["thickness_m"], intervention["value"]
        )
    elif axis == "uniform_scale":
        for key, value in list(result["geometry"].items()):
            result["geometry"][key] = decimal_product(value, intervention["value"])
    elif axis == "impulse":
        result["contact"]["normal_impulse_ns"] = decimal_product(
            result["contact"]["normal_impulse_ns"], intervention["value"]
        )
    elif axis == "contact":
        result["contact"]["u"] = intervention["target"]["u"]
        result["contact"]["v"] = intervention["target"]["v"]
    elif axis == "support":
        result["support"] = intervention["value"]
        result["formula_id"] = "euler-bernoulli-simply-supported-v1"
    else:
        raise ModalOwnerError(f"unknown frozen intervention: {axis}")
    return result


def build_case_inputs(profile: dict[str, Any]) -> list[tuple[str, dict[str, Any], str | None]]:
    fixtures = {fixture["fixture_id"]: fixture for fixture in profile["fixtures"]}
    result: list[tuple[str, dict[str, Any], str | None]] = [
        ("baseline-plate", copy.deepcopy(fixtures["synthetic-plate-reference"]), None),
        ("baseline-beam", copy.deepcopy(fixtures["synthetic-beam-reference"]), None),
    ]
    for intervention in profile["interventions"]:
        fixture = fixtures[intervention["fixture_id"]]
        case_id = f"intervention-{intervention['axis'].replace('_', '-')}"
        result.append((case_id, apply_intervention(fixture, intervention), intervention["axis"]))
    return result


def intervention_controls(
    profile: dict[str, Any], solutions: dict[str, dict[str, Any]]
) -> list[dict[str, Any]]:
    baselines = {
        "synthetic-plate-reference": solutions["baseline-plate"],
        "synthetic-beam-reference": solutions["baseline-beam"],
    }
    controls: list[dict[str, Any]] = []
    for intervention in profile["interventions"]:
        axis = intervention["axis"]
        case = solutions[f"intervention-{axis.replace('_', '-')}"]
        baseline = baselines[intervention["fixture_id"]]
        expected = float(intervention["expected_frequency_ratio"])
        ratios = case["frequencies"] / baseline["frequencies"]
        if axis == "support":
            maximum_error = abs(float(ratios[0]) - expected)
            compared_modes = 1
        else:
            maximum_error = float(np.max(np.abs(ratios - expected)))
            compared_modes = len(ratios)
        if maximum_error > RELATIVE_TOLERANCE:
            raise ModalOwnerError(f"P1 frequency intervention failed: {axis}")

        gain_result = "not-gated"
        if axis == "impulse":
            expected_samples = baseline["samples"] * float(intervention["value"])
            if not np.array_equal(case["samples"], expected_samples):
                raise ModalOwnerError("P1 impulse linearity is not binary64 exact")
            if not np.array_equal(case["signed_gains"], baseline["signed_gains"]):
                raise ModalOwnerError("P1 impulse changed modal participation")
            gain_result = "linear-ratio:2-exact"
        elif axis == "contact":
            target = intervention["target"]
            matches = np.nonzero(
                (case["indices"][:, 0] == target["m"])
                & (case["indices"][:, 1] == target["n"])
            )[0]
            if len(matches) != 1:
                raise ModalOwnerError("P1 target contact mode is absent or duplicated")
            ordinal = int(matches[0])
            mode = case["modal_document"]["modes"][ordinal]
            if mode["contact_participation"] != 0.0 or mode["signed_gain"] != 0.0:
                raise ModalOwnerError("P1 plate nodal participation is not exact zero")
            if not np.array_equal(case["frequencies"], baseline["frequencies"]):
                raise ModalOwnerError("P1 contact changed modal frequencies")
            gain_result = "target-mode-zero-exact"
        controls.append(
            {
                "axis": axis,
                "case_id": case["case_id"],
                "compared_modes": compared_modes,
                "expected_frequency_ratio": expected,
                "gain_result": gain_result,
                "maximum_frequency_ratio_absolute_error": maximum_error,
                "status": "Pass",
            }
        )
    return controls


def analytic_controls(
    profile: dict[str, Any], solutions: dict[str, dict[str, Any]]
) -> dict[str, Any]:
    plate = solutions["baseline-plate"]
    plate_validated = plate["validated"]
    boundary_points = np.asarray([0.0, 1.0], dtype=np.float64)
    maximum_plate_boundary_error = 0.0
    for m, n in plate["indices"]:
        maximum_plate_boundary_error = max(
            maximum_plate_boundary_error,
            float(np.max(np.abs(plate_sine(int(m), boundary_points)))),
            float(np.max(np.abs(plate_sine(int(n), boundary_points)))),
        )

    beam = solutions["baseline-beam"]
    formula = formula_table(profile)[beam["validated"]["formula_id"]]
    maximum_beam_clamp_error = 0.0
    maximum_root_residual = 0.0
    for root_text in formula["beta_roots"]:
        root = float(root_text)
        maximum_beam_clamp_error = max(
            maximum_beam_clamp_error,
            abs(float(cantilever_shape(root, 0.0))),
        )
        maximum_root_residual = max(
            maximum_root_residual,
            abs(math.cos(root) + 1.0 / math.cosh(root)),
        )
    if (
        maximum_plate_boundary_error > RELATIVE_TOLERANCE
        or maximum_beam_clamp_error > RELATIVE_TOLERANCE
        or maximum_root_residual > RELATIVE_TOLERANCE
        or plate_validated["support"] != "simply-supported-all-edges"
    ):
        raise ModalOwnerError("P1 analytic boundary/root control failed")
    return {
        "maximum_beam_characteristic_residual": maximum_root_residual,
        "maximum_beam_clamp_displacement_error": maximum_beam_clamp_error,
        "maximum_plate_boundary_displacement_error": maximum_plate_boundary_error,
        "status": "Pass",
    }


def fallback_result(profile: dict[str, Any], reason_code: str) -> dict[str, Any]:
    if reason_code not in profile["fallback"]["reason_codes"]:
        raise ModalOwnerError(f"undeclared fallback reason: {reason_code}")
    return {
        "authored_clip_class": profile["fallback"]["authored_clip_class"],
        "partial_output_published": False,
        "reason_code": reason_code,
        "status": profile["fallback"]["out_of_domain"],
    }


def attempt_case(
    case_id: str, fixture: dict[str, Any], profile: dict[str, Any]
) -> dict[str, Any]:
    try:
        solve_case(case_id, fixture, profile)
    except OutOfDomain as error:
        return fallback_result(profile, error.reason_code)
    return {"status": "UnexpectedlySupported"}


def fallback_probes(profile: dict[str, Any]) -> list[dict[str, Any]]:
    plate = next(
        fixture
        for fixture in profile["fixtures"]
        if fixture["fixture_id"] == "synthetic-plate-reference"
    )
    probes: list[tuple[str, dict[str, Any], str]] = []

    geometry = copy.deepcopy(plate)
    geometry["family"] = "arbitrary_triangle_soup"
    probes.append(("unsupported-geometry", geometry, "UnsupportedGeometry"))

    material = copy.deepcopy(plate)
    material["material"]["material_id"] = "physics-material.humanoid-body.v1"
    probes.append(
        ("unsupported-acoustic-material", material, "UnsupportedAcousticMaterial")
    )

    support = copy.deepcopy(plate)
    support["support"] = "free-free"
    probes.append(("unsupported-support", support, "UnsupportedSupport"))

    contact = copy.deepcopy(plate)
    contact["contact"]["u"] = "1.1"
    probes.append(("unsupported-contact", contact, "UnsupportedContact"))

    numeric = copy.deepcopy(plate)
    numeric["material"]["density_kg_m3"] = "NaN"
    probes.append(("invalid-numeric", numeric, "InvalidNumericInput"))

    resource = copy.deepcopy(plate)
    resource["mode_count"] = profile["resources"]["max_modes"] + 1
    probes.append(("resource-limit", resource, "ResourceLimitExceeded"))

    results = []
    for probe_id, fixture, expected in probes:
        result = attempt_case(probe_id, fixture, profile)
        if result["status"] != "FallbackOutOfDomain" or result["reason_code"] != expected:
            raise ModalOwnerError(f"typed fallback probe failed: {probe_id}")
        results.append({"probe_id": probe_id, **result})
    return results


def write_bytes(root: Path, relative: str, data: bytes) -> dict[str, Any]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {
        "bytes": len(data),
        "path": relative,
        "sha256": sha256_bytes(data),
    }


def verify_artifacts(root: Path, artifacts: list[dict[str, Any]]) -> None:
    paths = [artifact["path"] for artifact in artifacts]
    if paths != sorted(paths) or len(paths) != len(set(paths)):
        raise ModalOwnerError("P1 artifact index is not sorted and unique")
    for artifact in artifacts:
        data = (root / artifact["path"]).read_bytes()
        if len(data) != artifact["bytes"] or sha256_bytes(data) != artifact["sha256"]:
            raise ModalOwnerError(f"P1 artifact drift: {artifact['path']}")


def deterministic_memory_bound(profile: dict[str, Any]) -> int:
    frames = profile["numeric_profile"]["render_frames"]
    maximum_vertices = max(
        fixture["mesh_pair"]["fine_u"] * fixture["mesh_pair"]["fine_v"]
        for fixture in profile["fixtures"]
    )
    modes = profile["modal_output"]["mode_count"]
    return (
        4 * frames * 8
        + 2 * maximum_vertices * modes * 8
        + 2 * maximum_vertices * 3 * 8
        + 16 * 1024 * 1024
    )


def build_into(
    staging: Path,
    profile: dict[str, Any],
    profile_data: bytes,
    started: float,
) -> dict[str, Any]:
    dependencies = validate_dependencies()
    owner_data = Path(__file__).read_bytes()
    owner_identity = {
        "bytes": len(owner_data),
        "path": "lab/scripts/physical_sound_v31_p1_modal_owner_v1.py",
        "sha256": sha256_bytes(owner_data),
    }
    cases = build_case_inputs(profile)
    solutions = {
        case_id: solve_case(case_id, fixture, profile)
        for case_id, fixture, _ in cases
    }
    analytic = analytic_controls(profile, solutions)
    interventions = intervention_controls(profile, solutions)
    fallbacks = fallback_probes(profile)

    artifacts: list[dict[str, Any]] = []
    case_reports: list[dict[str, Any]] = []
    sample_rate_hz = profile["numeric_profile"]["sample_rate_hz"]
    for case_id, _, axis in sorted(cases):
        solution = solutions[case_id]
        base = f"cases/{case_id}"
        case_artifacts = [
            write_bytes(staging, f"{base}/gain-coarse.f64le", encode_gains(solution["coarse_field"])),
            write_bytes(staging, f"{base}/gain-fine.f64le", encode_gains(solution["fine_field"])),
            write_bytes(staging, f"{base}/mesh-coarse.bin", solution["coarse_mesh"]),
            write_bytes(staging, f"{base}/mesh-fine.bin", solution["fine_mesh"]),
            write_bytes(staging, f"{base}/modal.json", canonical_json(solution["modal_document"])),
            write_bytes(
                staging,
                f"{base}/render.wav",
                encode_float32_wav(solution["samples"], sample_rate_hz),
            ),
        ]
        artifacts.extend(case_artifacts)
        case_reports.append(
            {
                "artifact_sha256": {
                    item["path"].rsplit("/", 1)[-1]: item["sha256"]
                    for item in case_artifacts
                },
                "case_id": case_id,
                "fixture_id": solution["validated"]["fixture_id"],
                "intervention_axis": axis,
                "metrics": solution["metrics"],
                "mode_count": len(solution["frequencies"]),
                "status": "Pass",
            }
        )
    artifacts.sort(key=lambda item: item["path"])
    verify_artifacts(staging, artifacts)

    memory_bound = deterministic_memory_bound(profile)
    if memory_bound > profile["resources"]["max_peak_rss_bytes"]:
        raise ModalOwnerError("P1 deterministic memory bound exceeds the profile")
    artifact_bytes = sum(artifact["bytes"] for artifact in artifacts)
    if artifact_bytes > profile["resources"]["max_output_bytes"]:
        raise ModalOwnerError("P1 artifacts exceed the output resource profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise ModalOwnerError("P1 execution exceeded the wall resource profile")

    evidence = {
        "access": ZERO_ACCESS,
        "analytic_controls": analytic,
        "artifact_bytes_before_evidence": artifact_bytes,
        "artifact_count_before_evidence": len(artifacts),
        "artifact_index_root_sha256": sha256_bytes(canonical_json(artifacts)),
        "authority": profile["authority"],
        "case_reports": case_reports,
        "claim": CLAIM,
        "dependencies": dependencies,
        "deterministic_memory_bound_bytes": memory_bound,
        "environment": {
            "numpy": np.__version__,
            "python": platform.python_version(),
        },
        "fallback_probes": fallbacks,
        "intervention_controls": interventions,
        "owner_identity": owner_identity,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "schema": EVIDENCE_SCHEMA,
        "status": "Pass",
    }
    evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
    total_before_report = sum(
        path.stat().st_size for path in staging.rglob("*") if path.is_file()
    )
    if total_before_report > profile["resources"]["max_output_bytes"]:
        raise ModalOwnerError("P1 publication exceeds the output resource profile")
    report = {
        "access": ZERO_ACCESS,
        "authority": profile["authority"],
        "case_count": len(case_reports),
        "claim": CLAIM,
        "decision": "P1_DETERMINISTIC_MODAL_OWNER_PASS",
        "evidence_sha256": evidence_ref["sha256"],
        "fallback_probe_count": len(fallbacks),
        "gates": {
            "analytic_plate_beam": "Pass",
            "byte_exact_repeat_required": True,
            "energy_bounds": "Pass",
            "isolated_interventions": "7/7 Pass",
            "remesh_pairs": f"{len(case_reports)}/{len(case_reports)} Pass",
            "resource_envelope": "Pass",
            "typed_fallbacks": f"{len(fallbacks)}/{len(fallbacks)} Pass",
            "zero_signal_model_network": "Pass",
        },
        "next_authorized_stage": "V31-T0-truth-and-mutation-library",
        "output_bytes_before_report": total_before_report,
        "owner_sha256": owner_identity["sha256"],
        "profile_sha256": sha256_bytes(profile_data),
        "schema": REPORT_SCHEMA,
        "status": "Pass",
    }
    write_bytes(staging, "report.json", canonical_json(report))
    final_bytes = sum(
        path.stat().st_size for path in staging.rglob("*") if path.is_file()
    )
    if final_bytes > profile["resources"]["max_output_bytes"]:
        raise ModalOwnerError("P1 final publication exceeds the output resource profile")
    if time.monotonic() - started > profile["resources"]["max_wall_seconds"]:
        raise ModalOwnerError("P1 final execution exceeded the wall resource profile")
    return report


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    try:
        profile, profile_data = p0.load_profile(profile_path)
    except p0.CausalBaselineError as error:
        raise ModalOwnerError(f"P0 contract rejected: {error}") from error
    output = external_output(output_path)
    staging = Path(
        tempfile.mkdtemp(prefix=".nextengine-v31-p1-", dir=str(output.parent))
    )
    try:
        report = build_into(staging, profile, profile_data, started)
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
    except Exception as error:  # noqa: BLE001 - stable external CLI boundary
        print(f"physical-sound-v31-p1: ContractReject: {error}", file=sys.stderr)
        return 2
    print(canonical_json(report).decode("utf-8"), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
