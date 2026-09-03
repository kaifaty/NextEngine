#!/usr/bin/env python3
"""Freeze and exercise the exact V37 truth protocol without official targets."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import shutil
import struct
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any, TypeAlias, cast

import numpy as np
import physical_sound_v37_c0_query_surface_structural_cost_v1 as c0
import physical_sound_v37_query_surface_contract_v1 as contract
import torch
from numpy.typing import NDArray

PROFILE_PATH = "lab/profiles/physical-sound-v37-t0-exact-truth-protocol.v1.json"
F0_PROFILE_PATH = (
    "lab/profiles/physical-sound-v37-f0-fresh-query-surface-operator.v1.json"
)
E0_SEAL_PATH = "lab/profiles/physical-sound-v37-e0-execution-seal.v1.json"
OFFICIAL_INPUT_SEAL_PATH = (
    "lab/profiles/physical-sound-v37-t0-official-input-seal.v1.json"
)
OWNER_PATH = "lab/scripts/physical_sound_v37_t0_exact_truth_protocol_v1.py"
PROFILE_SCHEMA = (
    "nextengine.experimental-physical-sound-v37-t0-exact-truth-protocol-profile.v1"
)
PROFILE_ID = "physical-sound-v37-t0-exact-truth-protocol-v1"
PROFILE_SHA256 = "8eeda45716f95ec0b74abcc2ce90c1896657eb460fd3fd7a28fd8cf53257e5b6"
CLAIM = (
    "EXACT_VALUE_FREE_V37_TRUTH_GENERATION_PROTOCOL_AND_COMPOSITE_PRE_ACCESS_"
    "SEAL_ONLY / NO_OFFICIAL_TARGET_MODEL_QUALITY_REAL_MATERIAL_VALIDATOR_"
    "RELEASE_ADMISSION_COOKER_DEMO_OR_RUNTIME_AUTHORITY"
)
MAX_PROFILE_BYTES = 1_048_576
ROLE_NAMES = ("train", "development", "method_holdout")
TARGET_AXIS_COUNT = 3
EXPECTED_AUTHORITY = {
    "artificial_fixture_evaluation_allowed": True,
    "composite_pre_access_seal_allowed": True,
    "external_research_only": True,
    "model_execution_allowed": False,
    "network_allowed": False,
    "official_capability_allowed": False,
    "official_target_evaluation_allowed": False,
    "prior_numeric_value_access_allowed": False,
    "protected_or_real_signal_access_allowed": False,
    "runtime_authority": False,
}
ZERO_ACCESS = {
    "development_target_rows": 0,
    "method_holdout_target_rows": 0,
    "model_parameters_initialized": 0,
    "network_requests": 0,
    "official_capabilities_issued": 0,
    "official_truth_rows_evaluated": 0,
    "prior_generation_numeric_values_read": 0,
    "protected_signal_values_decoded": 0,
    "real_signal_values_decoded": 0,
    "train_target_rows": 0,
}
FloatArray: TypeAlias = NDArray[np.float64]


class T0ProtocolError(RuntimeError):
    """The exact truth protocol or its pre-access seal is invalid."""


@dataclass(frozen=True, slots=True)
class TruthRowMetadataV1:
    row_id: str
    mixture_id: str
    component_weights: tuple[float, float, float]


@dataclass(frozen=True, slots=True)
class TruthEvaluationV1:
    targets: FloatArray
    components: dict[str, FloatArray]


@dataclass(frozen=True, slots=True)
class LoadedContext:
    profile: dict[str, Any]
    profile_data: bytes
    dependencies: dict[str, dict[str, object]]
    f0_profile: dict[str, Any]
    e0_seal: dict[str, Any]
    environment: dict[str, object]


@dataclass(frozen=True, slots=True)
class FieldTruthState:
    positions: FloatArray
    modes: FloatArray
    area_normalized: FloatArray
    node_xy: FloatArray
    centroid: FloatArray
    span: float
    mode_rms: float
    normalized_adjacency: FloatArray
    field_moments: FloatArray


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: object) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode()
    except (TypeError, ValueError) as error:
        raise T0ProtocolError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = cast(object, json.loads(data))
    except json.JSONDecodeError as error:
        raise T0ProtocolError(f"invalid JSON: {label}") from error
    if not isinstance(value, dict):
        raise T0ProtocolError(f"JSON root must be an object: {label}")
    result = cast(dict[str, Any], value)
    if canonical_json(result) != data:
        raise T0ProtocolError(f"JSON is not canonical: {label}")
    return result


def bound_file(path_text: str, expected_sha256: str) -> dict[str, object]:
    path = repository_root() / path_text
    if not path.is_file() or path.is_symlink():
        raise T0ProtocolError(f"bound file is absent or linked: {path_text}")
    data = path.read_bytes()
    if sha256_bytes(data) != expected_sha256:
        raise T0ProtocolError(f"bound file drift: {path_text}")
    return {"bytes": len(data), "path": path_text, "sha256": expected_sha256}


def validate_protocol(profile: dict[str, Any]) -> None:
    protocol = cast(dict[str, Any], profile.get("protocol"))
    prf = cast(dict[str, Any], protocol.get("coefficient_prf"))
    expected_prf = {
        "algorithm": "sha256",
        "byte_layout": (
            "domain_utf8 || 0x00 || seed_u64_le || purpose_utf8 || 0x00 || index_u32_le"
        ),
        "domain": "nextengine.physical-sound.v37.t0.coefficient.v1",
        "float_mapping": (
            "u53 = uint64_be(digest[0:8]) >> 11; unit = u53 / 2^53; "
            "value = low + (high - low) * unit"
        ),
        "purpose": "coefficient-group-key-verbatim",
        "rounding": (
            "IEEE-754-binary64-round-to-nearest-ties-to-even-after-each-"
            "language-operation"
        ),
    }
    if prf != expected_prf:
        raise T0ProtocolError("coefficient PRF contract drift")
    input_contract = cast(dict[str, Any], protocol.get("input_contract"))
    science = cast(dict[str, Any], profile.get("science", {}))
    del science
    f0_names = cast(
        list[str],
        cast(
            dict[str, Any],
            load_json_bytes(
                (repository_root() / F0_PROFILE_PATH).read_bytes(), "F0 profile"
            )["science"],
        )["context_feature_names"],
    )
    if (
        input_contract.get("canonical_probe_count_per_field") != 9
        or input_contract.get("context_feature_names") != f0_names
        or input_contract.get("dtype") != "float64-cpu"
    ):
        raise T0ProtocolError("truth input contract drift")
    groups = cast(dict[str, dict[str, Any]], protocol.get("coefficient_groups"))
    expected_counts = {
        "decay.bias": 1,
        "decay.linear": 11,
        "diffusion.modulation": 2,
        "diffusion.neighbor_mix": 2,
        "diffusion.output_scale": 1,
        "diffusion.self_mix": 2,
        "global.bias": 4,
        "global.query_linear": 24,
        "global.rank_scale": 4,
        "global_gain.bias": 1,
        "global_gain.linear": 13,
        "local.modulation": 3,
        "local.output_scale": 1,
        "local.sigma_parallel": 1,
        "local.sigma_perpendicular": 1,
        "local.theta_radians": 1,
    }
    if set(groups) != set(expected_counts):
        raise T0ProtocolError("coefficient group closure drift")
    for name, expected_count in expected_counts.items():
        group = groups[name]
        if group.get("count") != expected_count:
            raise T0ProtocolError(f"coefficient count drift: {name}")
        low = float(cast(str, group.get("low")))
        high = float(cast(str, group.get("high")))
        seed = group.get("seed")
        if (
            not math.isfinite(low)
            or not math.isfinite(high)
            or not low < high
            or isinstance(seed, bool)
            or not isinstance(seed, int)
        ):
            raise T0ProtocolError(f"coefficient range or seed invalid: {name}")
    numeric = cast(dict[str, Any], protocol.get("numeric_reduction"))
    if numeric.get("target_axis_order") != [
        "decay",
        "global_gain",
        "contact",
    ] or numeric.get("implementation") != (
        "python-3.12.13-numpy-2.3.5-torch-2.8.0+cu128-float64-cpu-single-process"
    ):
        raise T0ProtocolError("target axis order drift")
    contact = cast(dict[str, Any], protocol.get("contact"))
    if contact.get("mixture_weight_order") != ["local", "diffusion", "global"]:
        raise T0ProtocolError("mixture component order drift")
    global_component = cast(dict[str, Any], contact.get("global"))
    if global_component.get("rank") != 4 or global_component.get(
        "query_vector_order"
    ) != [
        "query_x",
        "query_y",
        "p1_contact_value",
        "geometry.log_multiplier_0",
        "material.loss",
        "mode.ordinal_signed",
    ]:
        raise T0ProtocolError("global interaction contract drift")


def load_context(profile_path: Path) -> LoadedContext:
    if not profile_path.is_file() or profile_path.is_symlink():
        raise T0ProtocolError("T0 profile must be a regular file")
    data = profile_path.read_bytes()
    if not data or len(data) > MAX_PROFILE_BYTES:
        raise T0ProtocolError("T0 profile size outside bound")
    if sha256_bytes(data) != PROFILE_SHA256:
        raise T0ProtocolError("T0 profile hash mismatch")
    profile = load_json_bytes(data, "T0 profile")
    if (
        profile.get("schema") != PROFILE_SCHEMA
        or profile.get("profile_id") != PROFILE_ID
        or profile.get("revision") != 1
        or profile.get("claim") != CLAIM
        or profile.get("authority") != EXPECTED_AUTHORITY
    ):
        raise T0ProtocolError("T0 profile identity or authority drift")
    validate_protocol(profile)
    dependencies: dict[str, dict[str, object]] = {}
    paths: list[str] = []
    for declaration in cast(list[dict[str, str]], profile["dependencies"]):
        path_text = declaration["path"]
        dependencies[path_text] = bound_file(path_text, declaration["sha256"])
        paths.append(path_text)
    if paths != sorted(set(paths)):
        raise T0ProtocolError("T0 dependencies must be canonical and unique")
    if F0_PROFILE_PATH not in dependencies or E0_SEAL_PATH not in dependencies:
        raise T0ProtocolError("T0 dependency closure is incomplete")
    f0_profile = load_json_bytes(
        (repository_root() / F0_PROFILE_PATH).read_bytes(), "F0 profile"
    )
    e0_seal = load_json_bytes(
        (repository_root() / E0_SEAL_PATH).read_bytes(), "E0 execution seal"
    )
    validate_e0_seal(e0_seal)
    configure_execution()
    environment = environment_record(profile)
    return LoadedContext(profile, data, dependencies, f0_profile, e0_seal, environment)


def configure_execution() -> None:
    torch.use_deterministic_algorithms(True)
    if torch.get_num_threads() != 1:
        torch.set_num_threads(1)
    if torch.get_num_interop_threads() != 1:
        torch.set_num_interop_threads(1)


def environment_record(profile: dict[str, Any]) -> dict[str, object]:
    actual: dict[str, object] = {
        "device": "cpu",
        "dtype": "float64",
        "numpy": np.__version__,
        "python": platform.python_version(),
        "torch": torch.__version__,
        "torch_deterministic_algorithms": torch.are_deterministic_algorithms_enabled(),
        "torch_interop_threads": torch.get_num_interop_threads(),
        "torch_intraop_threads": torch.get_num_threads(),
    }
    if canonical_json(actual) != canonical_json(profile.get("environment")):
        raise T0ProtocolError("T0 execution environment identity drift")
    return actual


def validate_e0_seal(seal: dict[str, Any]) -> None:
    expected_hash = seal.get("seal_sha256")
    unsigned = dict(seal)
    unsigned.pop("seal_sha256", None)
    if (
        seal.get("schema")
        != "nextengine.experimental-physical-sound-v37-execution-seal.v1"
        or seal.get("status") != "SealedBeforeOfficialAccess"
        or expected_hash != sha256_bytes(canonical_json(unsigned))
        or expected_hash
        != "608f901938c4031e63353a9231e005bb1040ac21c50ed3c7746e01525e211d7f"
    ):
        raise T0ProtocolError("prior E0 execution seal is invalid")
    execution = cast(dict[str, Any], seal.get("execution_seal"))
    if (
        execution.get("forbidden_access_count") != 0
        or execution.get("rehearsal_run_count") != 2
        or execution.get("repeat_exact") is not True
    ):
        raise T0ProtocolError("prior E0 access or repeat contract drift")


def coefficient_unit(seed: int, purpose: str, index: int, domain: str) -> float:
    if seed < 0 or index < 0:
        raise T0ProtocolError("coefficient seed and index must be nonnegative")
    payload = (
        domain.encode("utf-8")
        + b"\x00"
        + struct.pack("<Q", seed)
        + purpose.encode("utf-8")
        + b"\x00"
        + struct.pack("<I", index)
    )
    integer = int.from_bytes(hashlib.sha256(payload).digest()[:8], "big") >> 11
    return integer / float(1 << 53)


def coefficient_table(profile: dict[str, Any]) -> dict[str, tuple[float, ...]]:
    protocol = cast(dict[str, Any], profile["protocol"])
    domain = cast(str, protocol["coefficient_prf"]["domain"])
    groups = cast(dict[str, dict[str, Any]], protocol["coefficient_groups"])
    result: dict[str, tuple[float, ...]] = {}
    for purpose in sorted(groups):
        group = groups[purpose]
        low = float(cast(str, group["low"]))
        high = float(cast(str, group["high"]))
        seed = cast(int, group["seed"])
        count = cast(int, group["count"])
        result[purpose] = tuple(
            low + (high - low) * coefficient_unit(seed, purpose, index, domain)
            for index in range(count)
        )
    return result


def coefficient_commitment(profile: dict[str, Any]) -> dict[str, object]:
    values = coefficient_table(profile)
    groups = {name: list(rows) for name, rows in values.items()}
    return {
        "coefficient_count": sum(len(rows) for rows in values.values()),
        "coefficient_root_sha256": sha256_bytes(canonical_json(groups)),
        "groups": groups,
        "prf": profile["protocol"]["coefficient_prf"],
        "schema": "nextengine.experimental-physical-sound-v37-t0-coefficients.v1",
    }


def mixture_map(f0_profile: dict[str, Any]) -> dict[str, tuple[float, float, float]]:
    grouped = cast(
        dict[str, list[dict[str, Any]]],
        cast(dict[str, Any], f0_profile["corpus"])["operator_mixtures"],
    )
    result: dict[str, tuple[float, float, float]] = {}
    for role in ROLE_NAMES:
        for row in grouped[role]:
            mixture_id = cast(str, row["mixture_id"])
            raw = cast(list[str], row["component_weights"])
            weights = cast(
                tuple[float, float, float], tuple(float(value) for value in raw)
            )
            if mixture_id in result or len(weights) != 3:
                raise T0ProtocolError("mixture identity closure invalid")
            if any(value < 0.0 or not math.isfinite(value) for value in weights):
                raise T0ProtocolError("mixture weight invalid")
            if not math.isclose(math.fsum(weights), 1.0, rel_tol=0.0, abs_tol=1e-15):
                raise T0ProtocolError("mixture weights are not unit-sum")
            result[mixture_id] = weights
    if len(result) != 8:
        raise T0ProtocolError("mixture map is incomplete")
    return result


def official_role_metadata(
    f0_profile: dict[str, Any], role: str
) -> tuple[TruthRowMetadataV1, ...]:
    if role not in ROLE_NAMES:
        raise T0ProtocolError(f"unknown official role: {role}")
    weights_by_id = mixture_map(f0_profile)
    rows: list[TruthRowMetadataV1] = []
    for case in c0.enumerate_cases(f0_profile, role):
        for mode_ordinal in range(10):
            rows.append(
                TruthRowMetadataV1(
                    f"{case.case_id}:mode-{mode_ordinal:02d}",
                    case.mixture_id,
                    weights_by_id[case.mixture_id],
                )
            )
    result = tuple(sorted(rows, key=lambda row: row.row_id))
    if len(result) != len({row.row_id for row in result}):
        raise T0ProtocolError("official truth metadata contains duplicate row IDs")
    expected = c0.f0.EXPECTED_CASE_COUNTS[role] * 10
    if len(result) != expected:
        raise T0ProtocolError(f"official truth metadata row-count drift: {role}")
    return result


def metadata_document(rows: tuple[TruthRowMetadataV1, ...]) -> list[dict[str, object]]:
    return [
        {
            "component_weights": list(row.component_weights),
            "mixture_id": row.mixture_id,
            "row_id": row.row_id,
        }
        for row in rows
    ]


def metadata_commitments(f0_profile: dict[str, Any]) -> dict[str, object]:
    roles: dict[str, object] = {}
    roots: list[str] = []
    for role in ROLE_NAMES:
        rows = official_role_metadata(f0_profile, role)
        root = sha256_bytes(canonical_json(metadata_document(rows)))
        roots.append(f"{role}:{root}")
        roles[role] = {"root_sha256": root, "row_count": len(rows)}
    return {
        "all_roles_root_sha256": sha256_bytes(("\n".join(roots) + "\n").encode()),
        "roles": roles,
        "schema": "nextengine.experimental-physical-sound-v37-t0-row-metadata.v1",
    }


def field_states(fields: contract.SurfaceFieldSetV1) -> tuple[FieldTruthState, ...]:
    result: list[FieldTruthState] = []
    for field_index in range(fields.field_count):
        start = int(fields.field_offsets[field_index])
        end = int(fields.field_offsets[field_index + 1])
        if end - start != 9:
            raise T0ProtocolError(
                "truth protocol requires exactly nine probes per field"
            )
        positions = np.asarray(fields.probe_positions[start:end], dtype=np.float64)
        modes = np.asarray(fields.probe_mode_values[start:end], dtype=np.float64)
        area = np.asarray(fields.probe_area_weights[start:end], dtype=np.float64)
        if np.any(area <= 0.0) or np.any(~np.isfinite(area)):
            raise T0ProtocolError(
                "truth field area weights must be finite and positive"
            )
        area_normalized = area / np.sum(area)
        centroid = np.sum(area_normalized[:, None] * positions, axis=0)
        span = max(float(np.max(np.ptp(positions, axis=0))), 1.0e-12)
        node_xy = (positions[:, :2] - centroid[None, :2]) / span
        mode_rms = math.sqrt(float(np.sum(area_normalized * modes * modes)) + 1.0e-6)
        adjacency: FloatArray = np.zeros((9, 9), dtype=np.float64)
        for source in range(start, end):
            edge_start = int(fields.edge_offsets[source])
            edge_end = int(fields.edge_offsets[source + 1])
            for target_raw in fields.edge_indices[edge_start:edge_end]:
                target = int(target_raw)
                if not start <= target < end or target == source:
                    raise T0ProtocolError(
                        "truth field adjacency crosses a field or loops"
                    )
                adjacency[source - start, target - start] = 1.0
        if not np.array_equal(adjacency, adjacency.T):
            raise T0ProtocolError("truth field adjacency must be symmetric")
        degrees = np.sum(adjacency, axis=1)
        if np.any(degrees <= 0.0):
            raise T0ProtocolError("truth field contains an isolated probe")
        normalized = adjacency / np.sqrt(degrees[:, None] * degrees[None, :])
        basis = np.column_stack(
            (
                np.ones(9, dtype=np.float64),
                node_xy[:, 0],
                node_xy[:, 1],
                node_xy[:, 0] * node_xy[:, 1],
            )
        )
        moments = (
            np.sum(area_normalized[:, None] * modes[:, None] * basis, axis=0) / mode_rms
        )
        result.append(
            FieldTruthState(
                positions,
                modes,
                area_normalized,
                node_xy,
                centroid,
                span,
                mode_rms,
                normalized,
                moments,
            )
        )
    return tuple(result)


def validate_metadata(
    batch: contract.SurfaceQueryBatchV1,
    metadata: tuple[TruthRowMetadataV1, ...],
) -> None:
    if batch.has_targets:
        raise T0ProtocolError("truth protocol accepts only an unlabelled batch")
    if len(metadata) != batch.row_count:
        raise T0ProtocolError("truth metadata row count differs from batch")
    if tuple(row.row_id for row in metadata) != batch.row_ids:
        raise T0ProtocolError("truth metadata is not in canonical batch row order")
    for row in metadata:
        if not row.mixture_id or len(row.component_weights) != 3:
            raise T0ProtocolError("truth metadata identity or width invalid")
        if any(
            not math.isfinite(value) or value < 0.0 for value in row.component_weights
        ) or not math.isclose(
            math.fsum(row.component_weights), 1.0, rel_tol=0.0, abs_tol=1.0e-15
        ):
            raise T0ProtocolError("truth metadata mixture is invalid")


def evaluate_targets(
    profile: dict[str, Any],
    batch: contract.SurfaceQueryBatchV1,
    metadata: tuple[TruthRowMetadataV1, ...],
) -> TruthEvaluationV1:
    validate_protocol(profile)
    validate_metadata(batch, metadata)
    names = cast(
        list[str], profile["protocol"]["input_contract"]["context_feature_names"]
    )
    if list(batch.context_feature_names) != names:
        raise T0ProtocolError("truth batch context feature order drift")
    index = {name: offset for offset, name in enumerate(names)}
    coefficients = coefficient_table(profile)
    states = field_states(batch.fields)
    local_rows = np.empty(batch.row_count, dtype=np.float64)
    diffusion_rows = np.empty(batch.row_count, dtype=np.float64)
    global_rows = np.empty(batch.row_count, dtype=np.float64)
    targets = np.empty((batch.row_count, TARGET_AXIS_COUNT), dtype=np.float64)
    theta = coefficients["local.theta_radians"][0]
    cosine = math.cos(theta)
    sine = math.sin(theta)
    sigma_parallel = coefficients["local.sigma_parallel"][0]
    sigma_perpendicular = coefficients["local.sigma_perpendicular"][0]
    local_modulation = coefficients["local.modulation"]
    diffusion_modulation = coefficients["diffusion.modulation"]
    self_mix = coefficients["diffusion.self_mix"]
    neighbor_mix = coefficients["diffusion.neighbor_mix"]
    global_linear: FloatArray = np.asarray(
        coefficients["global.query_linear"], dtype=np.float64
    ).reshape(4, 6)
    global_bias: FloatArray = np.asarray(coefficients["global.bias"], dtype=np.float64)
    rank_scale: FloatArray = np.asarray(
        coefficients["global.rank_scale"], dtype=np.float64
    )
    decay_linear: FloatArray = np.asarray(
        coefficients["decay.linear"], dtype=np.float64
    )
    gain_linear: FloatArray = np.asarray(
        coefficients["global_gain.linear"], dtype=np.float64
    )
    for row_index, row_metadata in enumerate(metadata):
        field_index = int(batch.row_field_indices[row_index])
        if not 0 <= field_index < len(states):
            raise T0ProtocolError("truth row references an invalid field")
        state = states[field_index]
        query_position = np.asarray(batch.query_positions[row_index], dtype=np.float64)
        delta = (state.positions - query_position[None, :]) / state.span
        parallel = cosine * delta[:, 0] + sine * delta[:, 1]
        perpendicular = -sine * delta[:, 0] + cosine * delta[:, 1]
        raw_kernel = state.area_normalized * np.exp(
            -0.5
            * (
                (parallel / sigma_parallel) ** 2
                + (perpendicular / sigma_perpendicular) ** 2
            )
        )
        kernel_sum = float(np.sum(raw_kernel))
        if not kernel_sum > 0.0 or not math.isfinite(kernel_sum):
            raise T0ProtocolError("truth query kernel is invalid")
        query_kernel = raw_kernel / kernel_sum
        node_x = state.node_xy[:, 0]
        node_y = state.node_xy[:, 1]
        local_drive = state.modes * (
            1.0
            + local_modulation[0] * node_x
            + local_modulation[1] * node_y
            + local_modulation[2] * node_x * node_y
        )
        local = (
            coefficients["local.output_scale"][0]
            * float(np.sum(query_kernel * local_drive))
            / state.mode_rms
        )
        diffusion_drive = state.modes * (
            1.0 + diffusion_modulation[0] * node_x + diffusion_modulation[1] * node_y
        )
        first = np.tanh(
            self_mix[0] * diffusion_drive
            + neighbor_mix[0] * (state.normalized_adjacency @ diffusion_drive)
        )
        second = np.tanh(
            self_mix[1] * first + neighbor_mix[1] * (state.normalized_adjacency @ first)
        )
        diffusion = (
            coefficients["diffusion.output_scale"][0]
            * float(np.sum(query_kernel * second))
            / state.mode_rms
        )
        query_xy = (query_position[:2] - state.centroid[:2]) / state.span
        context = np.asarray(batch.context_features[row_index], dtype=np.float64)
        query_vector = np.asarray(
            [
                query_xy[0],
                query_xy[1],
                batch.p1_contact_values[row_index],
                context[index["geometry.log_multiplier_0"]],
                context[index["material.loss"]],
                context[index["mode.ordinal_signed"]],
            ],
            dtype=np.float64,
        )
        query_scores = np.tanh(global_bias + global_linear @ query_vector)
        global_component = float(
            np.sum(rank_scale * state.field_moments * query_scores)
        )
        weights = row_metadata.component_weights
        contact_value = 0.25 * math.tanh(
            weights[0] * local + weights[1] * diffusion + weights[2] * global_component
        )
        decay_value = 0.25 * math.tanh(
            coefficients["decay.bias"][0] + float(np.dot(decay_linear, context[:11]))
        )
        gain_value = 0.20 * math.tanh(
            coefficients["global_gain.bias"][0]
            + float(np.dot(gain_linear, context[:13]))
        )
        local_rows[row_index] = local
        diffusion_rows[row_index] = diffusion
        global_rows[row_index] = global_component
        targets[row_index] = (decay_value, gain_value, contact_value)
    arrays = (targets, local_rows, diffusion_rows, global_rows)
    if any(np.any(~np.isfinite(value)) for value in arrays):
        raise T0ProtocolError("truth evaluation produced a non-finite value")
    if (
        np.any(np.abs(targets[:, 0]) > 0.25)
        or np.any(np.abs(targets[:, 1]) > 0.20)
        or np.any(np.abs(targets[:, 2]) > 0.25)
    ):
        raise T0ProtocolError("truth evaluation exceeded a frozen bound")
    targets.setflags(write=False)
    for value in (local_rows, diffusion_rows, global_rows):
        value.setflags(write=False)
    return TruthEvaluationV1(
        targets,
        {
            "diffusion": diffusion_rows,
            "global": global_rows,
            "local": local_rows,
        },
    )


def array_sha256(value: FloatArray) -> str:
    data = np.asarray(value, dtype="<f8").tobytes(order="C")
    return sha256_bytes(data)


def artificial_context(row: int) -> tuple[float, ...]:
    return (
        0.0,
        1.0,
        -0.4 + 0.2 * row,
        0.3 - 0.1 * row,
        -0.2 + 0.05 * row,
        0.1,
        -0.15,
        0.2 + 0.03 * row,
        -0.1,
        -0.3 + 0.15 * row,
        -1.0 + 0.4 * row,
        0.0,
        0.0,
        1.0,
    )


def artificial_fixture(
    *, reverse: bool = False, mode_delta: float = 0.0, drop_edge: bool = False
) -> tuple[contract.SurfaceQueryBatchV1, tuple[TruthRowMetadataV1, ...]]:
    field_id = "artificial-v37-t0-field"
    mode_values = (0.10, 0.55, -0.20, -0.45, 0.80, 0.35, 0.15, -0.60, 0.25)
    one_dimensional = (0.25, 0.50, 0.25)
    probes: list[contract.ProbeInputV1] = []
    for v_index in range(3):
        for u_index in range(3):
            probe_id = f"{field_id}:probe-v{v_index:02d}-u{u_index:02d}"
            neighbors: list[str] = []
            for delta_v, delta_u in ((-1, 0), (0, -1), (0, 1), (1, 0)):
                neighbor_v = v_index + delta_v
                neighbor_u = u_index + delta_u
                if 0 <= neighbor_v < 3 and 0 <= neighbor_u < 3:
                    if drop_edge and {
                        (v_index, u_index),
                        (neighbor_v, neighbor_u),
                    } == {(1, 1), (1, 2)}:
                        continue
                    neighbors.append(
                        f"{field_id}:probe-v{neighbor_v:02d}-u{neighbor_u:02d}"
                    )
            ordinal = 3 * v_index + u_index
            probes.append(
                contract.ProbeInputV1(
                    probe_id,
                    (0.55 * u_index, 0.40 * v_index, 0.02),
                    (0.0, 0.0, 1.0),
                    0.88 * one_dimensional[u_index] * one_dimensional[v_index],
                    mode_values[ordinal] + (mode_delta if ordinal == 4 else 0.0),
                    tuple(sorted(neighbors)),
                )
            )
    if reverse:
        probes.reverse()
        probes = [
            contract.ProbeInputV1(
                probe.probe_id,
                probe.position,
                probe.normal,
                probe.area_weight,
                probe.mode_value,
                tuple(reversed(probe.neighbor_ids)),
            )
            for probe in probes
        ]
    fields = contract.canonical_surface_fields(
        (contract.FieldInputV1(field_id, tuple(probes)),)
    )
    names = cast(
        tuple[str, ...],
        tuple(
            cast(
                list[str],
                load_json_bytes(
                    (repository_root() / PROFILE_PATH).read_bytes(), "T0 profile"
                )["protocol"]["input_contract"]["context_feature_names"],
            )
        ),
    )
    query_specs = (
        ("artificial-row-0", 0.12, 0.21, 0.18),
        ("artificial-row-1", 0.37, 0.68, -0.22),
        ("artificial-row-2", 0.73, 0.44, 0.51),
        ("artificial-row-3", 0.91, 0.86, -0.36),
    )
    queries: list[contract.QueryInputV1] = []
    metadata_by_id: dict[str, TruthRowMetadataV1] = {}
    weights = (
        (0.50, 0.30, 0.20),
        (0.35, 0.45, 0.20),
        (0.30, 0.25, 0.45),
        (0.42, 0.28, 0.30),
    )
    for row, (row_id, u, v, p1_value) in enumerate(query_specs):
        context_values = artificial_context(row)
        query_names = names
        if reverse:
            context_values = tuple(reversed(context_values))
            query_names = tuple(reversed(names))
        queries.append(
            contract.QueryInputV1(
                row_id,
                field_id,
                f"{field_id}:artificial-triangle-{row}",
                (1.10 * u, 0.80 * v, 0.02),
                (0.0, 0.0, 1.0),
                (0.2, 0.3, 0.5),
                context_values,
                p1_value,
            )
        )
        metadata_by_id[row_id] = TruthRowMetadataV1(
            row_id, f"artificial-mixture-{row}", weights[row]
        )
    if reverse:
        queries.reverse()
    batch = contract.canonical_query_batch(
        contract.RoleKind.TRAIN,
        fields,
        query_names,
        tuple(queries),
        targets=None,
    )
    metadata = tuple(metadata_by_id[row_id] for row_id in batch.row_ids)
    return batch, metadata


def artificial_conformance(profile: dict[str, Any]) -> dict[str, object]:
    batch, metadata = artificial_fixture()
    repeated = evaluate_targets(profile, batch, metadata)
    reverse_batch, reverse_metadata = artificial_fixture(reverse=True)
    reversed_result = evaluate_targets(profile, reverse_batch, reverse_metadata)
    if (
        batch.structural_root_sha256 != reverse_batch.structural_root_sha256
        or not np.array_equal(repeated.targets, reversed_result.targets)
    ):
        raise T0ProtocolError("artificial canonical permutation changed truth")
    mode_batch, mode_metadata = artificial_fixture(mode_delta=0.125)
    mode_result = evaluate_targets(profile, mode_batch, mode_metadata)
    topology_batch, topology_metadata = artificial_fixture(drop_edge=True)
    topology_result = evaluate_targets(profile, topology_batch, topology_metadata)
    mutated_metadata = list(metadata)
    first = mutated_metadata[0]
    mutated_metadata[0] = TruthRowMetadataV1(
        first.row_id, first.mixture_id + "-mutation", (0.20, 0.55, 0.25)
    )
    mixture_result = evaluate_targets(profile, batch, tuple(mutated_metadata))
    if (
        np.array_equal(repeated.targets, mode_result.targets)
        or np.array_equal(repeated.targets, topology_result.targets)
        or np.array_equal(repeated.targets, mixture_result.targets)
    ):
        raise T0ProtocolError("artificial truth mutation was not observable")
    component_roots = {
        name: array_sha256(values) for name, values in repeated.components.items()
    }
    return {
        "artificial_only": True,
        "component_roots": component_roots,
        "mixture_mutation_observed": True,
        "mode_mutation_observed": True,
        "official_access": ZERO_ACCESS,
        "permutation_exact": True,
        "row_count": batch.row_count,
        "schema": "nextengine.experimental-physical-sound-v37-t0-artificial-conformance.v1",
        "target_root_sha256": array_sha256(repeated.targets),
        "topology_mutation_observed": True,
    }


def owner_identity() -> dict[str, object]:
    data = (repository_root() / OWNER_PATH).read_bytes()
    return {"bytes": len(data), "path": OWNER_PATH, "sha256": sha256_bytes(data)}


def composite_seal(
    context: LoadedContext,
    coefficients: dict[str, object],
    metadata: dict[str, object],
    conformance: dict[str, object],
) -> dict[str, object]:
    dependency_rows = [
        context.dependencies[path] for path in sorted(context.dependencies)
    ]
    unsigned: dict[str, object] = {
        "claim": CLAIM,
        "composite_seal": {
            "artificial_conformance_root_sha256": sha256_bytes(
                canonical_json(conformance)
            ),
            "coefficient_root_sha256": coefficients["coefficient_root_sha256"],
            "dependency_root_sha256": sha256_bytes(canonical_json(dependency_rows)),
            "environment_sha256": sha256_bytes(canonical_json(context.environment)),
            "forbidden_access_count": 0,
            "official_metadata_root_sha256": metadata["all_roles_root_sha256"],
            "owner_sha256": owner_identity()["sha256"],
            "prior_e0_execution_seal_sha256": context.e0_seal["seal_sha256"],
            "profile_sha256": PROFILE_SHA256,
            "target_axis_order": ["decay", "global_gain", "contact"],
        },
        "schema": "nextengine.experimental-physical-sound-v37-t0-official-input-seal.v1",
        "status": "SealedBeforeOfficialTargetAccess",
    }
    unsigned["seal_sha256"] = sha256_bytes(canonical_json(unsigned))
    return unsigned


def validate_checked_official_input_seal(
    expected: dict[str, object],
    path: Path | None = None,
) -> dict[str, Any]:
    seal_path = repository_root() / OFFICIAL_INPUT_SEAL_PATH if path is None else path
    if not seal_path.is_file() or seal_path.is_symlink():
        raise T0ProtocolError("checked official-input seal is absent or linked")
    data = seal_path.read_bytes()
    actual = load_json_bytes(data, "T0 official-input seal")
    if data != canonical_json(expected):
        raise T0ProtocolError(
            "checked official-input seal differs from current closure"
        )
    unsigned = dict(actual)
    declared = unsigned.pop("seal_sha256", None)
    if declared != sha256_bytes(canonical_json(unsigned)):
        raise T0ProtocolError("checked official-input seal self-hash mismatch")
    return actual


def write_tree_atomic(output: Path, files: dict[str, bytes]) -> None:
    if output.exists() or output.is_symlink():
        raise T0ProtocolError("output must be a fresh external path")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v37-t0-", dir=output.parent))
    try:
        for name, data in files.items():
            path = staging / name
            path.write_bytes(data)
        os.replace(staging, output)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def run(profile_path: Path, output: Path) -> dict[str, object]:
    context = load_context(profile_path)
    coefficients = coefficient_commitment(context.profile)
    metadata = metadata_commitments(context.f0_profile)
    conformance = artificial_conformance(context.profile)
    seal = composite_seal(context, coefficients, metadata, conformance)
    validate_checked_official_input_seal(seal)
    evidence: dict[str, object] = {
        "artificial_conformance": conformance,
        "dependencies": context.dependencies,
        "environment": context.environment,
        "official_access": ZERO_ACCESS,
        "owner": owner_identity(),
        "prior_e0_seal": {
            "file_sha256": context.dependencies[E0_SEAL_PATH]["sha256"],
            "seal_sha256": context.e0_seal["seal_sha256"],
        },
        "profile_sha256": PROFILE_SHA256,
        "schema": "nextengine.experimental-physical-sound-v37-t0-evidence.v1",
    }
    report: dict[str, object] = {
        "claim": CLAIM,
        "decision": "T0_EXACT_TRUTH_PROTOCOL_PASS",
        "next_authorized_action": "implement-seal-verifying-one-shot-v37-d0-provider",
        "official_access": ZERO_ACCESS,
        "official_values_opened": False,
        "schema": "nextengine.experimental-physical-sound-v37-t0-report.v1",
        "seal_sha256": seal["seal_sha256"],
        "status": "EXACT_TRUTH_PROTOCOL_AND_COMPOSITE_PRE_ACCESS_SEAL_COMPLETE",
    }
    files = {
        "artificial-conformance.json": canonical_json(conformance),
        "coefficient-commitment.json": canonical_json(coefficients),
        "evidence.json": canonical_json(evidence),
        "official-input-seal.json": canonical_json(seal),
        "report.json": canonical_json(report),
        "row-metadata-commitments.json": canonical_json(metadata),
    }
    write_tree_atomic(output, files)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", default=PROFILE_PATH, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_args()
    report = run(arguments.profile, arguments.output.absolute())
    print(canonical_json(report).decode(), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
