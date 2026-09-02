#!/usr/bin/env python3
"""Prove the V35 local expert and coverage gate on discarded analytic fields."""

from __future__ import annotations

import argparse
import ast
import hashlib
import inspect
import json
import math
import random
import resource
import shutil
import struct
import sys
import tempfile
import textwrap
import time
from collections import defaultdict
from collections.abc import Iterable, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1 as f0

F0_OWNER_SHA256 = "5173638f200ded42f50a72258617b69df0225f9a0b8fd3fe092475533e1548b2"
C0_OWNER_PATH = "lab/scripts/physical_sound_v35_c0_witness_and_coverage_census_v1.py"
C0_OWNER_SHA256 = "c0f5aa7ea02e51b1c43e46d95ae0fd47518f411759ca1b32dd9d7f5b6def9cd7"
C0_RESULT_PATH = (
    "docs/development/"
    "physical-sound-v35-c0-witness-and-coverage-census-result-2026-09-02.md"
)
C0_RESULT_SHA256 = "65ff4534b04264914cc2ed676df80c998356b11fe640619761ec71a6285d9bdf"
OWNER_PATH = "lab/scripts/physical_sound_v35_b0_local_gate_conformance_v1.py"
CLAIM = (
    "DISCARDED_ANALYTIC_LOCAL_EXPERT_AND_COVERAGE_GATE_CONFORMANCE_ONLY / "
    "NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
CONTACT_TARGET_BOUND = 0.22
NEURAL_OUTPUT_BOUND = 0.25
LOCAL_WEIGHT_CAP = 0.80
OOD_NORMALIZED_SQUARED_DISTANCE_STRICT_MAX = 16.0
LOCAL_KEY_WIDTH = 15
MAX_COMPATIBLE_ROWS = 216
DISCARDED_FAMILIES = (
    ("discarded-plate", "discarded-simple-support"),
    ("discarded-beam", "discarded-cantilever-support"),
    ("discarded-beam", "discarded-simple-support"),
)
DISCARDED_MATERIALS = (
    (-0.70, -0.48, -0.35, -0.82),
    (-0.04, 0.08, 0.16, 0.02),
    (0.66, 0.57, 0.38, 0.78),
)
DISCARDED_GEOMETRY = (
    (-0.82, -0.42, 0.18),
    (-0.48, 0.64, -0.56),
    (-0.12, -0.74, 0.81),
    (0.27, 0.53, -0.68),
    (0.61, -0.21, 0.67),
    (0.91, 0.29, -0.14),
)
DISCARDED_CONTACTS = (
    (0.04, 0.28),
    (0.14, 0.76),
    (0.24, 0.44),
    (0.36, 0.88),
    (0.48, 0.12),
    (0.56, 0.64),
    (0.66, 0.34),
    (0.80, 0.92),
    (0.90, 0.50),
    (0.20, 0.98),
    (0.72, 0.04),
    (0.96, 0.68),
)
FORBIDDEN_IMPORT_STEMS = {
    "physical_sound_v31_p1_modal_owner_v1",
    "physical_sound_v33_d0_development_tournament_v1",
    "physical_sound_v33_i0_mode_local_spectral_owner_v1",
    "physical_sound_v34_d0_development_tournament_v1",
    "physical_sound_v34_h0_method_holdout_v1",
    "physical_sound_v35_c0_witness_and_coverage_census_v1",
    "torch",
}
ZERO_FORBIDDEN_ACCESS = dict(f0.ZERO_ACCESS)


class B0ConformanceError(RuntimeError):
    """The discarded V35 B0 implementation-conformance contract failed."""


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise B0ConformanceError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def validate_bound_file(path_text: str, expected_sha256: str) -> dict[str, Any]:
    path = repository_root() / path_text
    if path.is_symlink():
        raise B0ConformanceError(f"bound file must not be a symlink: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise B0ConformanceError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_context(profile_path: Path) -> tuple[dict[str, Any], dict[str, Any]]:
    overlay, profile_data = f0.load_profile(profile_path)
    if sha256_bytes(profile_data) != f0.PROFILE_SHA256:
        raise B0ConformanceError("F0 profile identity drift")
    f0.validate_dependencies(overlay)
    v33 = f0.load_declared_json(overlay, "v33_profile")
    effective = f0.build_effective_profile(overlay, v33)
    f0.validate_preserved_sections(overlay, v33)
    f0.validate_method_freeze(overlay, effective, v33)
    validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256)
    validate_bound_file(C0_OWNER_PATH, C0_OWNER_SHA256)
    validate_bound_file(C0_RESULT_PATH, C0_RESULT_SHA256)
    expected_prefix = [
        "f0-identity-freshness-and-static-freeze",
        "c0-signal-blind-witness-and-coverage-census",
        "b0-discarded-local-and-gate-conformance",
    ]
    if overlay["access_order"][:3] != expected_prefix:
        raise B0ConformanceError("F0/C0/B0 access order drift")
    return overlay, effective


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
        raise B0ConformanceError(f"forbidden owner import: {','.join(forbidden)}")
    if "physical_sound_v35_f0_geometry_hybrid_profile_freeze_v1" not in imports:
        raise B0ConformanceError("required F0 owner import missing")
    return {"forbidden_imports": forbidden, "imports": sorted(imports)}


def validate_no_exact_contact_equality_branch() -> dict[str, Any]:
    functions = (coverage_gate, LocalInterpolator.predict)
    equality_count = 0
    for function in functions:
        tree = ast.parse(textwrap.dedent(inspect.getsource(function)))
        equality_count += sum(
            isinstance(operator, ast.Eq)
            for node in ast.walk(tree)
            if isinstance(node, ast.Compare)
            for operator in node.ops
        )
    if equality_count != 0:
        raise B0ConformanceError(
            "exact equality branch entered local/gate implementation"
        )
    return {
        "exact_equality_comparisons": equality_count,
        "functions": ["LocalInterpolator.predict", "coverage_gate"],
    }


def finite_tuple(values: Iterable[float], width: int, name: str) -> tuple[float, ...]:
    result = tuple(float(value) for value in values)
    if len(result) != width or not all(math.isfinite(value) for value in result):
        raise B0ConformanceError(f"{name} must be finite width-{width}")
    return result


def encode_causal(value: Any) -> Any:
    if isinstance(value, float):
        if not math.isfinite(value):
            raise B0ConformanceError("causal value is non-finite")
        return {"binary64_hex": value.hex()}
    if isinstance(value, tuple):
        return [encode_causal(item) for item in value]
    if isinstance(value, (str, int)):
        return value
    raise B0ConformanceError(f"unsupported causal value type: {type(value).__name__}")


@dataclass(frozen=True)
class LocalRow:
    trace_id: str
    partition: tuple[str, str, int]
    causal_key: tuple[float, ...]
    group_key: tuple[Any, ...]
    target: float

    def validated(self) -> LocalRow:
        if not self.trace_id:
            raise B0ConformanceError("trace identity must be nonempty")
        if (
            len(self.partition) != 3
            or not isinstance(self.partition[0], str)
            or not isinstance(self.partition[1], str)
            or not isinstance(self.partition[2], int)
        ):
            raise B0ConformanceError("partition must be family/support/ordinal")
        finite_tuple(self.causal_key, LOCAL_KEY_WIDTH, "local causal key")
        if not self.group_key:
            raise B0ConformanceError("causal case group must be nonempty")
        encode_causal(self.group_key)
        if not math.isfinite(self.target) or abs(self.target) > CONTACT_TARGET_BOUND:
            raise B0ConformanceError("local target outside finite contact bound")
        return self


@dataclass(frozen=True)
class LocalPrediction:
    value: float
    nearest_squared_distance: float
    denominator: float
    contributor_count: int
    contributor_trace_ids: tuple[str, ...]
    contributor_group_keys: tuple[tuple[Any, ...], ...]


def squared_distance(left: Sequence[float], right: Sequence[float]) -> float:
    if len(left) != len(right):
        raise B0ConformanceError("distance key width mismatch")
    value = math.fsum(
        (float(a) - float(b)) ** 2 for a, b in zip(left, right, strict=True)
    )
    if not math.isfinite(value) or value < 0.0:
        raise B0ConformanceError("distance must be finite nonnegative")
    return value


class LocalInterpolator:
    """Permutation-invariant train-only Gaussian interpolation."""

    def __init__(self, rows: Sequence[LocalRow]) -> None:
        if not rows:
            raise B0ConformanceError("local train set must be nonempty")
        by_partition: dict[tuple[str, str, int], list[LocalRow]] = defaultdict(list)
        seen_trace_ids: set[str] = set()
        duplicate_values: dict[tuple[Any, ...], bytes] = {}
        for source in rows:
            row = source.validated()
            if row.trace_id in seen_trace_ids:
                raise B0ConformanceError("duplicate trace identity")
            seen_trace_ids.add(row.trace_id)
            causal_identity = (row.partition, row.causal_key, row.group_key)
            target_bytes = struct.pack("<d", row.target)
            previous = duplicate_values.setdefault(causal_identity, target_bytes)
            if previous != target_bytes:
                raise B0ConformanceError("same causal row has conflicting targets")
            by_partition[row.partition].append(row)
        for partition_rows in by_partition.values():
            partition_rows.sort(
                key=lambda row: (row.partition, row.causal_key, row.group_key)
            )
            if len(partition_rows) > MAX_COMPATIBLE_ROWS:
                raise B0ConformanceError("local partition exceeds 216-row bound")
        self._by_partition = dict(by_partition)
        canonical = [
            {
                "causal_key": encode_causal(row.causal_key),
                "group_key": encode_causal(row.group_key),
                "partition": encode_causal(row.partition),
                "target_hex": row.target.hex(),
            }
            for partition in sorted(self._by_partition)
            for row in self._by_partition[partition]
        ]
        self.canonical_train_sha256 = sha256_bytes(canonical_json(canonical))
        self.partition_counts = {
            "/".join((partition[0], partition[1], str(partition[2]))): len(rows)
            for partition, rows in sorted(self._by_partition.items())
        }

    def predict(self, query: LocalRow, *, exclude_query_group: bool) -> LocalPrediction:
        query.validated()
        compatible = self._by_partition.get(query.partition, [])
        if exclude_query_group:
            compatible = [row for row in compatible if row.group_key != query.group_key]
        if not compatible:
            raise B0ConformanceError("local query has no compatible train support")
        distances = [
            squared_distance(query.causal_key, row.causal_key) for row in compatible
        ]
        nearest = min(distances)
        weights = [math.exp(-0.5 * (distance - nearest)) for distance in distances]
        denominator = math.fsum(weights)
        numerator = math.fsum(
            weight * row.target for weight, row in zip(weights, compatible, strict=True)
        )
        if not math.isfinite(denominator) or denominator <= 0.0:
            raise B0ConformanceError("local kernel denominator is not finite-positive")
        value = numerator / denominator
        if not math.isfinite(value) or abs(value) > CONTACT_TARGET_BOUND:
            raise B0ConformanceError("local prediction outside finite contact bound")
        return LocalPrediction(
            value=value,
            nearest_squared_distance=nearest,
            denominator=denominator,
            contributor_count=len(compatible),
            contributor_trace_ids=tuple(sorted(row.trace_id for row in compatible)),
            contributor_group_keys=tuple(row.group_key for row in compatible),
        )


@dataclass(frozen=True)
class CoverageGateResult:
    normalized_squared_distance: float
    local_weight: float
    neural_weight: float
    decision: str


def positive_support_bandwidth(points: Sequence[Sequence[float]]) -> float:
    normalized = [tuple(float(value) for value in point) for point in points]
    try:
        return f0.median_positive_nearest_distance(normalized)
    except f0.F0FreezeError as error:
        raise B0ConformanceError(str(error)) from error


def coverage_gate(
    geometry_key: Sequence[float],
    contact_key: Sequence[float],
    train_geometry_keys: Sequence[Sequence[float]],
    train_contact_keys: Sequence[Sequence[float]],
    geometry_bandwidth: float,
    contact_bandwidth: float,
) -> CoverageGateResult:
    geometry = finite_tuple(geometry_key, 3, "coverage geometry key")
    contact = finite_tuple(contact_key, 2, "coverage contact key")
    train_geometry = [
        finite_tuple(point, 3, "train geometry key") for point in train_geometry_keys
    ]
    train_contacts = [
        finite_tuple(point, 2, "train contact key") for point in train_contact_keys
    ]
    if not train_geometry or not train_contacts:
        raise B0ConformanceError("coverage support must be nonempty")
    if (
        not math.isfinite(geometry_bandwidth)
        or geometry_bandwidth <= 0.0
        or not math.isfinite(contact_bandwidth)
        or contact_bandwidth <= 0.0
    ):
        raise B0ConformanceError("coverage bandwidth must be finite-positive")
    geometry_distance = math.sqrt(
        min(squared_distance(geometry, candidate) for candidate in train_geometry)
    )
    contact_distance = math.sqrt(
        min(squared_distance(contact, candidate) for candidate in train_contacts)
    )
    normalized_squared = (geometry_distance / geometry_bandwidth) ** 2 + (
        contact_distance / contact_bandwidth
    ) ** 2
    local_weight = LOCAL_WEIGHT_CAP * math.exp(-0.5 * normalized_squared)
    neural_weight = 1.0 - local_weight
    if not all(
        math.isfinite(value)
        for value in (normalized_squared, local_weight, neural_weight)
    ):
        raise B0ConformanceError("coverage gate produced non-finite output")
    decision = (
        "InDomain"
        if normalized_squared < OOD_NORMALIZED_SQUARED_DISTANCE_STRICT_MAX
        else "FallbackOutOfDomain"
    )
    return CoverageGateResult(
        normalized_squared_distance=normalized_squared,
        local_weight=local_weight,
        neural_weight=neural_weight,
        decision=decision,
    )


def blend_contact(
    local_value: float, neural_value: float, local_weight: float
) -> float:
    if (
        not math.isfinite(local_value)
        or abs(local_value) > CONTACT_TARGET_BOUND
        or not math.isfinite(neural_value)
        or abs(neural_value) > NEURAL_OUTPUT_BOUND
        or not math.isfinite(local_weight)
        or not 0.0 <= local_weight <= LOCAL_WEIGHT_CAP
    ):
        raise B0ConformanceError("hybrid blend input outside frozen bounds")
    value = local_weight * local_value + (1.0 - local_weight) * neural_value
    value = max(-NEURAL_OUTPUT_BOUND, min(NEURAL_OUTPUT_BOUND, value))
    if not math.isfinite(value) or abs(value) > NEURAL_OUTPUT_BOUND:
        raise B0ConformanceError("hybrid blend output outside frozen bounds")
    return value


def modal_field(family: int, ordinal: int, u: float, v: float) -> float:
    if family == 0:
        index_u = ordinal % 3 + 1
        index_v = ordinal // 3 + 1
        return math.sin(index_u * math.pi * u) * math.sin(index_v * math.pi * v)
    if family == 1:
        return math.sin((ordinal + 0.5) * math.pi * u) * (
            0.80 + 0.20 * math.cos(math.pi * v)
        )
    return math.sin((ordinal + 1.0) * math.pi * u) * (
        0.90 + 0.10 * math.cos(2.0 * math.pi * v)
    )


def discarded_target(key: tuple[float, ...], family: int, ordinal: int) -> float:
    value = 0.18 * math.tanh(
        0.19 * key[0]
        - 0.13 * key[1]
        + 0.11 * key[4] * key[10]
        + 0.09 * key[5] * key[7]
        - 0.08 * key[6] * key[8]
        + 0.24 * key[10]
        + 0.12 * (key[12] - key[11])
        + 0.06 * family
        - 0.03 * ordinal
    )
    if not math.isfinite(value) or abs(value) >= CONTACT_TARGET_BOUND:
        raise B0ConformanceError("discarded analytic target outside bound")
    return value


def discarded_row(
    *,
    family: int,
    ordinal: int,
    material: tuple[float, float, float, float],
    geometry: tuple[float, float, float],
    contact: tuple[float, float],
    trace_id: str,
    group_suffix: tuple[Any, ...] = (),
) -> LocalRow:
    if not 0 <= family < len(DISCARDED_FAMILIES) or not 0 <= ordinal < 10:
        raise B0ConformanceError("discarded family or ordinal outside bound")
    u, v = contact
    if not 0.0 <= u <= 1.0 or not 0.0 <= v <= 1.0:
        raise B0ConformanceError("discarded contact outside unit surface")
    h = 0.0625
    p1_contact = modal_field(family, ordinal, u, v)
    stencil = (
        modal_field(family, ordinal, max(0.0, u - h), v),
        modal_field(family, ordinal, min(1.0, u + h), v),
        modal_field(family, ordinal, u, max(0.0, v - h)),
        modal_field(family, ordinal, u, min(1.0, v + h)),
    )
    log_frequency = max(
        -1.5,
        min(
            1.5,
            -0.82
            + 0.13 * ordinal
            + 0.04 * family
            + 0.03 * geometry[0]
            - 0.02 * geometry[1]
            + 0.015 * material[0],
        ),
    )
    key = finite_tuple(
        (
            *material,
            *geometry,
            2.0 * u - 1.0,
            2.0 * v - 1.0,
            log_frequency,
            p1_contact,
            *stencil,
        ),
        LOCAL_KEY_WIDTH,
        "discarded local key",
    )
    formula, support = DISCARDED_FAMILIES[family]
    group_key = (formula, support, material, geometry, contact, *group_suffix)
    return LocalRow(
        trace_id=trace_id,
        partition=(formula, support, ordinal),
        causal_key=key,
        group_key=group_key,
        target=discarded_target(key, family, ordinal),
    ).validated()


def build_discarded_official_shape() -> list[LocalRow]:
    rows: list[LocalRow] = []
    for family in range(len(DISCARDED_FAMILIES)):
        for ordinal in range(10):
            for material_index, material in enumerate(DISCARDED_MATERIALS):
                for geometry_index, geometry in enumerate(DISCARDED_GEOMETRY):
                    for contact_index, contact in enumerate(DISCARDED_CONTACTS):
                        rows.append(
                            discarded_row(
                                family=family,
                                ordinal=ordinal,
                                material=material,
                                geometry=geometry,
                                contact=contact,
                                trace_id=(
                                    f"discarded/b0/f{family}/o{ordinal:02d}/"
                                    f"m{material_index}/g{geometry_index}/c{contact_index:02d}"
                                ),
                            )
                        )
    if len(rows) != 6480:
        raise B0ConformanceError("discarded official-shape row count drift")
    return rows


def id_commitment(values: Sequence[str]) -> dict[str, Any]:
    ordered = sorted(values)
    if len(ordered) != len(set(ordered)):
        raise B0ConformanceError("identity commitment contains duplicates")
    return {
        "count": len(ordered),
        "first": ordered[0] if ordered else None,
        "last": ordered[-1] if ordered else None,
        "sha256": sha256_bytes(canonical_json(ordered)),
    }


def prediction_commitment(values: Sequence[tuple[str, float, int]]) -> dict[str, Any]:
    encoded = [
        {
            "contributor_count": count,
            "prediction_hex": value.hex(),
            "trace_id": trace_id,
        }
        for trace_id, value, count in sorted(values)
    ]
    return {
        "count": len(encoded),
        "sha256": sha256_bytes(canonical_json(encoded)),
    }


def evaluate_discarded_query(
    interpolator: LocalInterpolator,
    *,
    family: int,
    ordinal: int,
    material: tuple[float, float, float, float],
    geometry: tuple[float, float, float],
    contact: tuple[float, float],
    train_geometry: Sequence[Sequence[float]],
    train_contacts: Sequence[Sequence[float]],
    geometry_bandwidth: float,
    contact_bandwidth: float,
) -> dict[str, Any]:
    query = discarded_row(
        family=family,
        ordinal=ordinal,
        material=material,
        geometry=geometry,
        contact=contact,
        trace_id="discarded/b0/continuity-query",
        group_suffix=("query",),
    )
    local = interpolator.predict(query, exclude_query_group=False)
    signed_contact = (2.0 * contact[0] - 1.0, 2.0 * contact[1] - 1.0)
    gate = coverage_gate(
        geometry,
        signed_contact,
        train_geometry,
        train_contacts,
        geometry_bandwidth,
        contact_bandwidth,
    )
    neural = 0.20 * math.tanh(
        0.17 * query.causal_key[0]
        + 0.11 * query.causal_key[4]
        - 0.09 * query.causal_key[5]
        + 0.23 * query.causal_key[10]
        + 0.07 * query.causal_key[7] * query.causal_key[8]
    )
    hybrid = blend_contact(local.value, neural, gate.local_weight)
    return {
        "gate": gate,
        "hybrid": hybrid,
        "local": local.value,
        "neural": neural,
    }


def continuity_conformance(
    overlay: dict[str, Any], interpolator: LocalInterpolator
) -> dict[str, Any]:
    epsilon = float(overlay["witness_contract"]["b0_perturbation_epsilon"])
    train_geometry = list(DISCARDED_GEOMETRY)
    train_contacts = [
        (2.0 * contact[0] - 1.0, 2.0 * contact[1] - 1.0)
        for contact in DISCARDED_CONTACTS
    ]
    geometry_bandwidth = positive_support_bandwidth(train_geometry)
    contact_bandwidth = positive_support_bandwidth(train_contacts)
    base_geometry = (0.08, 0.19, -0.09)
    base_contact = (0.42, 0.58)
    material = (0.21, -0.17, 0.12, 0.33)
    axes = ("geometry_0", "geometry_1", "geometry_2", "contact_u", "contact_v")
    probes: dict[str, Any] = {}
    for axis in axes:
        scales: list[dict[str, Any]] = []
        deltas_by_output: dict[str, list[float]] = {
            "gate_local_weight": [],
            "hybrid": [],
            "local": [],
        }
        for divisor in (1.0, 2.0, 4.0):
            delta = epsilon / divisor
            plus_geometry = list(base_geometry)
            minus_geometry = list(base_geometry)
            plus_contact = list(base_contact)
            minus_contact = list(base_contact)
            if axis.startswith("geometry_"):
                index = int(axis[-1])
                plus_geometry[index] += delta
                minus_geometry[index] -= delta
            elif axis == "contact_u":
                plus_contact[0] += delta
                minus_contact[0] -= delta
            else:
                plus_contact[1] += delta
                minus_contact[1] -= delta
            plus = evaluate_discarded_query(
                interpolator,
                family=0,
                ordinal=3,
                material=material,
                geometry=tuple(plus_geometry),
                contact=tuple(plus_contact),
                train_geometry=train_geometry,
                train_contacts=train_contacts,
                geometry_bandwidth=geometry_bandwidth,
                contact_bandwidth=contact_bandwidth,
            )
            minus = evaluate_discarded_query(
                interpolator,
                family=0,
                ordinal=3,
                material=material,
                geometry=tuple(minus_geometry),
                contact=tuple(minus_contact),
                train_geometry=train_geometry,
                train_contacts=train_contacts,
                geometry_bandwidth=geometry_bandwidth,
                contact_bandwidth=contact_bandwidth,
            )
            if (
                plus["gate"].decision != "InDomain"
                or minus["gate"].decision != "InDomain"
            ):
                raise B0ConformanceError("continuity probe unexpectedly entered OOD")
            deltas = {
                "gate_local_weight": abs(
                    plus["gate"].local_weight - minus["gate"].local_weight
                ),
                "hybrid": abs(plus["hybrid"] - minus["hybrid"]),
                "local": abs(plus["local"] - minus["local"]),
            }
            for name, value in deltas.items():
                if not math.isfinite(value):
                    raise B0ConformanceError("continuity probe is non-finite")
                deltas_by_output[name].append(value)
            scales.append(
                {
                    "delta_hex": delta.hex(),
                    "gate_local_weight_delta_hex": deltas["gate_local_weight"].hex(),
                    "hybrid_delta_hex": deltas["hybrid"].hex(),
                    "local_delta_hex": deltas["local"].hex(),
                }
            )
        for name, values in deltas_by_output.items():
            if values[1] > values[0] + 2.0**-48 or values[2] > values[1] + 2.0**-48:
                raise B0ConformanceError(
                    f"continuity refinement does not converge: {axis}/{name}"
                )
        probes[axis] = scales
    return {
        "contact_bandwidth_hex": contact_bandwidth.hex(),
        "epsilon_hex": epsilon.hex(),
        "geometry_bandwidth_hex": geometry_bandwidth.hex(),
        "probe_axes": probes,
        "proof": "finite-positive-denominator-continuous-exp-weighted-sum-and-blend",
    }


def ood_and_zero_distance_conformance(
    interpolator: LocalInterpolator, rows: Sequence[LocalRow]
) -> dict[str, Any]:
    exact_source = rows[0]
    exact_query = LocalRow(
        trace_id="discarded/b0/exact-query",
        partition=exact_source.partition,
        causal_key=exact_source.causal_key,
        group_key=(*exact_source.group_key, "evaluation-query"),
        target=exact_source.target,
    )
    exact_prediction = interpolator.predict(exact_query, exclude_query_group=False)
    if (
        exact_prediction.nearest_squared_distance != 0.0
        or exact_prediction.contributor_count != MAX_COMPATIBLE_ROWS
        or struct.pack("<d", exact_prediction.value)
        == struct.pack("<d", exact_query.target)
    ):
        raise B0ConformanceError("zero-distance path used a nearest-row shortcut")
    exact_gate = coverage_gate(
        (0.0, 0.0, 0.0),
        (0.0, 0.0),
        [(0.0, 0.0, 0.0)],
        [(0.0, 0.0)],
        1.0,
        1.0,
    )
    ood_probes = {
        "inside": coverage_gate(
            (3.999, 0.0, 0.0),
            (0.0, 0.0),
            [(0.0, 0.0, 0.0)],
            [(0.0, 0.0)],
            1.0,
            1.0,
        ),
        "threshold": coverage_gate(
            (4.0, 0.0, 0.0),
            (0.0, 0.0),
            [(0.0, 0.0, 0.0)],
            [(0.0, 0.0)],
            1.0,
            1.0,
        ),
        "outside": coverage_gate(
            (4.001, 0.0, 0.0),
            (0.0, 0.0),
            [(0.0, 0.0, 0.0)],
            [(0.0, 0.0)],
            1.0,
            1.0,
        ),
    }
    if (
        exact_gate.normalized_squared_distance != 0.0
        or exact_gate.local_weight != LOCAL_WEIGHT_CAP
        or exact_gate.decision != "InDomain"
        or ood_probes["inside"].decision != "InDomain"
        or ood_probes["threshold"].decision != "FallbackOutOfDomain"
        or ood_probes["outside"].decision != "FallbackOutOfDomain"
    ):
        raise B0ConformanceError("zero-distance or strict OOD semantics drift")
    return {
        "exact_gate": {
            "decision": exact_gate.decision,
            "local_weight_hex": exact_gate.local_weight.hex(),
            "normalized_squared_distance_hex": exact_gate.normalized_squared_distance.hex(),
        },
        "exact_local": {
            "contributor_count": exact_prediction.contributor_count,
            "nearest_squared_distance_hex": exact_prediction.nearest_squared_distance.hex(),
            "prediction_hex": exact_prediction.value.hex(),
            "target_hex": exact_query.target.hex(),
        },
        "ood": {
            name: {
                "decision": result.decision,
                "normalized_squared_distance_hex": result.normalized_squared_distance.hex(),
            }
            for name, result in sorted(ood_probes.items())
        },
    }


def build_conformance(profile_path: Path) -> dict[str, Any]:
    overlay, effective = validate_context(profile_path)
    import_boundary = validate_owner_import_boundary()
    equality_boundary = validate_no_exact_contact_equality_branch()
    rows = build_discarded_official_shape()
    interpolator = LocalInterpolator(rows)
    if len(interpolator.partition_counts) != 30 or set(
        interpolator.partition_counts.values()
    ) != {MAX_COMPATIBLE_ROWS}:
        raise B0ConformanceError("discarded partition algebra drift")

    cross_fit: list[tuple[str, float, int]] = []
    for row in rows:
        prediction = interpolator.predict(row, exclude_query_group=True)
        if prediction.contributor_count != 215 or any(
            group == row.group_key for group in prediction.contributor_group_keys
        ):
            raise B0ConformanceError("case/remesh group leaked into train prediction")
        cross_fit.append((row.trace_id, prediction.value, prediction.contributor_count))
    cross_fit_commitment = prediction_commitment(cross_fit)

    seeds = overlay["witness_contract"]["b0_train_permutation_seeds"]
    probe_rows = [
        row
        for index, row in enumerate(rows)
        if index % MAX_COMPATIBLE_ROWS in {0, 107, 215}
    ]
    baseline_probe = {
        row.trace_id: struct.pack(
            "<d", interpolator.predict(row, exclude_query_group=True).value
        )
        for row in probe_rows
    }
    permutation_results: list[dict[str, Any]] = []
    for seed in seeds:
        permuted_rows = list(rows)
        random.Random(seed).shuffle(permuted_rows)
        permuted = LocalInterpolator(permuted_rows)
        predictions = {
            row.trace_id: struct.pack(
                "<d", permuted.predict(row, exclude_query_group=True).value
            )
            for row in probe_rows
        }
        if (
            permuted.canonical_train_sha256 != interpolator.canonical_train_sha256
            or predictions != baseline_probe
        ):
            raise B0ConformanceError(f"train permutation changed result: {seed}")
        permutation_results.append(
            {
                "canonical_train_sha256": permuted.canonical_train_sha256,
                "prediction_count": len(predictions),
                "prediction_sha256": sha256_bytes(
                    b"".join(
                        trace_id.encode() + predictions[trace_id]
                        for trace_id in sorted(predictions)
                    )
                ),
                "seed": seed,
            }
        )

    continuity = continuity_conformance(overlay, interpolator)
    zero_ood = ood_and_zero_distance_conformance(interpolator, rows)
    blend_probes = [
        blend_contact(local, neural, weight)
        for local, neural, weight in (
            (CONTACT_TARGET_BOUND, NEURAL_OUTPUT_BOUND, LOCAL_WEIGHT_CAP),
            (-CONTACT_TARGET_BOUND, -NEURAL_OUTPUT_BOUND, LOCAL_WEIGHT_CAP),
            (0.0, NEURAL_OUTPUT_BOUND, 0.0),
            (CONTACT_TARGET_BOUND, -NEURAL_OUTPUT_BOUND, 0.37),
        )
    ]
    if any(abs(value) > NEURAL_OUTPUT_BOUND for value in blend_probes):
        raise B0ConformanceError("bounded blend probe failed")
    forbidden = set(overlay["method_overlay"]["features"]["forbidden_fields"])
    declared_inputs = set(
        overlay["method_overlay"]["local_expert"]["distance_key"]
        + overlay["method_overlay"]["coverage_gate"]["contact_key"]
        + overlay["method_overlay"]["coverage_gate"]["geometry_key"]
    )
    forbidden_intersection = sorted(forbidden & declared_inputs)
    if forbidden_intersection:
        raise B0ConformanceError("forbidden field entered local/gate input")

    return {
        "access": {
            "discarded_analytic_target_rows": len(rows),
            "discarded_cross_fit_predictions": len(cross_fit),
            "discarded_permutation_probe_predictions": len(probe_rows) * len(seeds),
            "official_forbidden_access": ZERO_FORBIDDEN_ACCESS,
            "official_role_rows_materialized": 0,
            "p1_cases_solved": 0,
        },
        "claim": CLAIM,
        "continuity": continuity,
        "cross_fit": {
            "compatible_after_group_leave_out": {"maximum": 215, "minimum": 215},
            "prediction_commitment": cross_fit_commitment,
        },
        "discarded_shape": {
            "partition_count": len(interpolator.partition_counts),
            "partition_row_count": {"maximum": 216, "minimum": 216},
            "row_ids": id_commitment([row.trace_id for row in rows]),
        },
        "gates": {
            "bounded_blend": "Pass",
            "case_remesh_group_leave_out": "Pass",
            "continuous_local_and_gate": "Pass",
            "deterministic_strict_ood": "Pass",
            "forbidden_fields_absent": "Pass",
            "no_exact_contact_split": "Pass",
            "same_equation_zero_distance": "Pass",
            "train_permutation_exact": "Pass",
        },
        "implementation": {
            "canonical_train_sha256": interpolator.canonical_train_sha256,
            "equality_boundary": equality_boundary,
            "forbidden_feature_intersection": forbidden_intersection,
            "import_boundary": import_boundary,
            "local_complexity": "O(compatible_rows*15)-time/O(compatible_rows)-scratch",
            "maximum_compatible_rows": MAX_COMPATIBLE_ROWS,
        },
        "permutations": permutation_results,
        "preserved_resources": effective["resources"],
        "schema": "nextengine.experimental-physical-sound-v35-b0-conformance.v1",
        "status": "Pass",
        "zero_distance_and_ood": zero_ood,
    }


def external_output(path: Path) -> Path:
    try:
        return f0.external_output(path)
    except f0.F0FreezeError as error:
        raise B0ConformanceError(str(error)) from error


def peak_rss_bytes() -> int:
    return int(resource.getrusage(resource.RUSAGE_SELF).ru_maxrss) * 1024


def write_bytes(root: Path, name: str, data: bytes) -> dict[str, Any]:
    (root / name).write_bytes(data)
    return {"bytes": len(data), "path": name, "sha256": sha256_bytes(data)}


def run(profile_path: Path, output_path: Path) -> dict[str, Any]:
    started = time.monotonic()
    output = external_output(output_path)
    conformance = build_conformance(profile_path)
    elapsed = time.monotonic() - started
    rss = peak_rss_bytes()
    resources = conformance["preserved_resources"]
    resource_gates = {
        "peak_rss_within_1_gib": rss <= resources["max_peak_rss_bytes"],
        "wall_within_300_seconds": elapsed <= resources["max_wall_seconds"],
    }
    if not all(resource_gates.values()):
        raise B0ConformanceError("B0 resource envelope exceeded")
    owner_data = (repository_root() / OWNER_PATH).read_bytes()
    dependencies = {
        "c0_owner": validate_bound_file(C0_OWNER_PATH, C0_OWNER_SHA256),
        "c0_result": validate_bound_file(C0_RESULT_PATH, C0_RESULT_SHA256),
        "f0_owner": validate_bound_file(f0.OWNER_PATH, F0_OWNER_SHA256),
    }
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v35-b0-", dir=output.parent))
    try:
        conformance_ref = write_bytes(
            staging, "conformance.json", canonical_json(conformance)
        )
        evidence = {
            "claim": CLAIM,
            "dependencies": dependencies,
            "forbidden_access": ZERO_FORBIDDEN_ACCESS,
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
            "schema": "nextengine.experimental-physical-sound-v35-b0-evidence.v1",
        }
        evidence_ref = write_bytes(staging, "evidence.json", canonical_json(evidence))
        report = {
            "claim": CLAIM,
            "conformance": conformance_ref,
            "decision": "B0_LOCAL_EXPERT_AND_COVERAGE_GATE_CONFORMANCE_PASS",
            "evidence": evidence_ref,
            "next_authorized_stage": "V35-I0-discarded-whole-owner-terminal-path-proof",
            "official_target_or_model_values_opened": False,
            "schema": "nextengine.experimental-physical-sound-v35-b0-report.v1",
            "status": "Pass",
        }
        write_bytes(staging, "report.json", canonical_json(report))
        total_bytes = sum(path.stat().st_size for path in staging.iterdir())
        if total_bytes > resources["max_output_bytes"]:
            raise B0ConformanceError("B0 output resource envelope exceeded")
        staging.replace(output)
        print(
            f"b0-resource wall_seconds={elapsed:.6f} peak_rss_bytes={rss} "
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
        B0ConformanceError,
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
