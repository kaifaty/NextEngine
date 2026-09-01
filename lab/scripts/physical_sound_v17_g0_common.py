#!/usr/bin/env python3
"""Frozen V17 G0 corpus, truth, and serialization boundary."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import shutil
import struct
import sys
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
from typing import Any, Iterable

import numpy as np
import scipy
import torch


STUDY_ID = "physical-sound-v17-g0-scale-separated-global-oracle"
REVISION = "factorized-global-modes-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v17-g0.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v17-g0.report.v1"
PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v17-p0a-disjoint-factorized-truth-protocol-2026-09-01.md"
)
IMPLEMENTATION_FILES = (
    "physical_sound_v17_g0_common.py",
    "physical_sound_v17_g0_model.py",
    "physical_sound_v17_g0_oracle.py",
)
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
    "torch": "2.13.0+cu130",
}

MATERIAL_ORDER = ("Steel", "Wood", "Glass")
TOPOLOGY_ORDER = ("Plate", "Cylinder", "Bowl", "RolledSheet")
SUPPORT_ORDER = ("Free", "BaseClamped")
MATERIALS = {
    "Steel": {"density": 7_850.0, "wave_speed": 5_000.0, "decay": 6.0},
    "Wood": {"density": 650.0, "wave_speed": 3_300.0, "decay": 16.0},
    "Glass": {"density": 2_500.0, "wave_speed": 5_200.0, "decay": 4.0},
}
TOPOLOGY_FREQUENCY = (1.00, 1.16, 1.30, 1.08)
TOPOLOGY_DAMPING = (1.00, 1.08, 1.16, 1.04)
SUPPORT_FREQUENCY = (1.00, 1.19)
SUPPORT_DAMPING = (1.00, 1.10)
Q = np.asarray([1.40, 2.25, 3.30, 4.55, 6.00, 7.65, 9.50, 11.55])

MODE_COUNT = 8
SEEDS = (170_001, 170_002, 170_003)
UPDATES = 1_500
LEARNING_RATE_START = 2.0e-3
LEARNING_RATE_END = 1.0e-5
WEIGHT_DECAY = 1.0e-6

ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
    "v16_artifact_bytes_read": 0,
}


class G0Error(RuntimeError):
    """The frozen V17 G0 boundary or numerical contract was violated."""


@dataclass(frozen=True)
class GRow:
    object_id: str
    role: str
    stratum: str
    cell: int
    replicate: int
    halton_index: int
    material: str
    topology: str
    support: str
    length_m: float
    aspect: float
    slenderness: float
    wall_m: float
    frequencies: np.ndarray
    damping: np.ndarray

    @property
    def material_index(self) -> int:
        return MATERIAL_ORDER.index(self.material)

    @property
    def topology_index(self) -> int:
        return TOPOLOGY_ORDER.index(self.topology)

    @property
    def support_index(self) -> int:
        return SUPPORT_ORDER.index(self.support)

    @property
    def frequency_scale(self) -> float:
        return (
            MATERIALS[self.material]["wave_speed"]
            * self.wall_m
            / (self.length_m * self.length_m)
        )

    @property
    def damping_scale(self) -> float:
        return MATERIALS[self.material]["decay"]

    def record(self, include_truth: bool = True) -> dict[str, Any]:
        value: dict[str, Any] = {
            "aspect": self.aspect,
            "cell": self.cell,
            "halton_index": self.halton_index,
            "length_m": self.length_m,
            "material": self.material,
            "object_id": self.object_id,
            "replicate": self.replicate,
            "role": self.role,
            "slenderness": self.slenderness,
            "stratum": self.stratum,
            "support": self.support,
            "topology": self.topology,
            "wall_m": self.wall_m,
        }
        if include_truth:
            value["damping"] = self.damping
            value["frequencies_hz"] = self.frequencies
        return value


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def json_ready(value: Any) -> Any:
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, np.ndarray):
        return value.tolist()
    if isinstance(value, dict):
        return {str(key): json_ready(item) for key, item in value.items()}
    if isinstance(value, (list, tuple)):
        return [json_ready(item) for item in value]
    return value


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(json_ready(value), indent=2, sort_keys=True, allow_nan=False)
            + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise G0Error(f"cannot encode canonical JSON: {error}") from error


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def radical_inverse(index: int, base: int) -> float:
    if index <= 0 or base <= 1:
        raise G0Error("radical inverse requires positive index and base > 1")
    result = 0.0
    factor = 1.0
    value = index
    while value:
        factor /= base
        result += factor * (value % base)
        value //= base
    return result


def _truth(
    material_index: int,
    topology_index: int,
    support_index: int,
    length_m: float,
    aspect: float,
    wall_m: float,
) -> tuple[np.ndarray, np.ndarray]:
    material = MATERIALS[MATERIAL_ORDER[material_index]]
    mode = np.arange(MODE_COUNT, dtype=np.float64)
    aspect_log = math.log(aspect)
    aspect_factor = (
        1.0
        + 0.08 * abs(aspect_log)
        + 0.012 * np.sin((mode + 1.0) * aspect_log)
    )
    material_factor = 1.0 + 0.01 * material_index * np.cos(
        0.7 * (mode + 1.0)
    )
    frequency_scale = material["wave_speed"] * wall_m / (length_m * length_m)
    frequency_ratio = (
        Q
        * TOPOLOGY_FREQUENCY[topology_index]
        * SUPPORT_FREQUENCY[support_index]
        * aspect_factor
        * material_factor
    )
    damping_multiplier = (
        (1.0 + 0.08 * mode)
        * TOPOLOGY_DAMPING[topology_index]
        * SUPPORT_DAMPING[support_index]
        * (1.0 + 0.06 * abs(aspect_log))
    )
    frequencies = frequency_scale * frequency_ratio
    damping = material["decay"] * damping_multiplier
    if (
        not np.isfinite(frequencies).all()
        or not np.isfinite(damping).all()
        or frequencies[0] < 40.0
        or frequencies[-1] > 7_500.0
        or np.any(np.diff(frequencies) <= 0.0)
        or np.any(damping <= 0.0)
        or np.any(damping > 128.0)
    ):
        raise G0Error("generated G truth violates frozen bounds")
    return frequencies, damping


def _make_row(role: str, stratum: str, cell: int, replicate: int, n: int) -> GRow:
    material_index = cell // 8
    topology_index = (cell // 2) % 4
    support_index = cell % 2
    aspect = 0.70 + 0.80 * radical_inverse(n, 3)
    slenderness = 0.004 + 0.004 * radical_inverse(n, 5)
    if stratum == "scale_transfer":
        if cell % 2 == 0:
            length_m = 0.15 + 0.05 * radical_inverse(n, 2)
        else:
            length_m = 0.50 + 0.12 * radical_inverse(n, 2)
    else:
        length_m = 0.22 + 0.24 * radical_inverse(n, 2)
    wall_m = length_m * slenderness
    frequencies, damping = _truth(
        material_index,
        topology_index,
        support_index,
        length_m,
        aspect,
        wall_m,
    )
    material = MATERIAL_ORDER[material_index]
    topology = TOPOLOGY_ORDER[topology_index]
    support = SUPPORT_ORDER[support_index]
    object_id = (
        f"g-{role}-{material}-{topology}-{support}-{replicate}".lower()
    )
    return GRow(
        object_id=object_id,
        role=role,
        stratum=stratum,
        cell=cell,
        replicate=replicate,
        halton_index=n,
        material=material,
        topology=topology,
        support=support,
        length_m=length_m,
        aspect=aspect,
        slenderness=slenderness,
        wall_m=wall_m,
        frequencies=frequencies,
        damping=damping,
    )


def generate_corpus() -> tuple[GRow, ...]:
    rows: list[GRow] = []
    for cell in range(24):
        for replicate in range(3):
            rows.append(
                _make_row(
                    "train",
                    "interpolation",
                    cell,
                    replicate,
                    1 + 3 * cell + replicate,
                )
            )
    for cell in range(24):
        rows.append(
            _make_row("development", "interpolation", cell, 0, 101 + cell)
        )
    for cell in range(24):
        rows.append(
            _make_row("test-interpolation", "interpolation", cell, 0, 201 + cell)
        )
    for cell in range(24):
        rows.append(
            _make_row("test-scale-transfer", "scale_transfer", cell, 0, 301 + cell)
        )
    result = tuple(rows)
    counts = {
        role: sum(row.role == role for row in result)
        for role in (
            "train",
            "development",
            "test-interpolation",
            "test-scale-transfer",
        )
    }
    expected = {
        "train": 72,
        "development": 24,
        "test-interpolation": 24,
        "test-scale-transfer": 24,
    }
    if counts != expected:
        raise G0Error(f"G corpus counts changed: {counts}")
    identities = [row.object_id for row in result]
    if len(set(identities)) != len(identities):
        raise G0Error("G corpus object identity collision")
    return result


@dataclass(frozen=True)
class Normalizer:
    mean: np.ndarray
    scale: np.ndarray

    def apply(self, value: np.ndarray) -> np.ndarray:
        return (np.asarray(value, dtype=np.float64) - self.mean) / self.scale

    def record(self) -> dict[str, Any]:
        return {"mean": self.mean, "scale": self.scale}


def fit_normalizer(values: Iterable[np.ndarray]) -> Normalizer:
    matrix = np.stack([np.asarray(value, dtype=np.float64) for value in values])
    mean = np.mean(matrix, axis=0)
    scale = np.std(matrix, axis=0)
    return Normalizer(mean, np.where(scale > 1.0e-12, scale, 1.0))


def inverse_softplus(value: np.ndarray) -> np.ndarray:
    value = np.asarray(value, dtype=np.float64)
    return value + np.log(-np.expm1(-value))


def hard_validate(frequencies: np.ndarray, damping: np.ndarray) -> bool:
    return bool(
        np.isfinite(frequencies).all()
        and np.isfinite(damping).all()
        and frequencies[0] >= 40.0
        and frequencies[-1] <= 7_500.0
        and np.all(np.diff(frequencies) > 0.0)
        and np.all(damping > 0.0)
        and np.all(damping <= 128.0)
    )


def frequency_cents(prediction: np.ndarray, truth: np.ndarray) -> np.ndarray:
    return np.abs(1_200.0 * np.log2(prediction / truth))


def damping_relative(prediction: np.ndarray, truth: np.ndarray) -> np.ndarray:
    return np.abs(prediction - truth) / truth


def array_bytes(value: np.ndarray) -> bytes:
    buffer = BytesIO()
    np.save(buffer, np.asarray(value), allow_pickle=False)
    return buffer.getvalue()


def model_parameter_bytes(state: dict[str, torch.Tensor]) -> bytes:
    output = bytearray(b"NEXTENGINE-V17-G0-MODEL\0")
    for name in sorted(state):
        value = state[name].detach().cpu().numpy().astype("<f8", copy=False)
        encoded = name.encode("utf-8")
        output.extend(struct.pack("<I", len(encoded)))
        output.extend(encoded)
        output.extend(struct.pack("<I", value.ndim))
        for dimension in value.shape:
            output.extend(struct.pack("<Q", dimension))
        output.extend(value.tobytes(order="C"))
    return bytes(output)


def environment_identity(require_exact: bool = True) -> dict[str, Any]:
    observed = {
        "byteorder": sys.byteorder,
        "compute_device": "cpu",
        "machine": platform.machine(),
        "numpy": np.__version__,
        "platform": platform.platform(),
        "python": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
        "torch_deterministic_algorithms": True,
        "torch_interop_threads": 1,
        "torch_threads": 1,
    }
    if require_exact:
        for key, expected in REQUIRED_ENVIRONMENT.items():
            if observed[key] != expected:
                raise G0Error(
                    f"G0 environment changed for {key}: {observed[key]} != {expected}"
                )
    return observed


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    result = {}
    for filename in IMPLEMENTATION_FILES:
        path = directory / filename
        if not path.is_file():
            raise G0Error(f"missing G0 implementation: {filename}")
        result[filename] = sha256_file(path)
    return result


def prepare_output(output: Path) -> tuple[Path, Path]:
    repository = repository_root().resolve()
    allowed = Path(
        "/home/kaifaty/.codex/experiments/nextengine/physical-sound"
    ).resolve()
    resolved = output.resolve()
    if resolved.is_relative_to(repository):
        raise G0Error("G0 output must stay outside the repository")
    if not resolved.is_relative_to(allowed):
        raise G0Error("G0 output must stay under the physical-sound experiment root")
    if resolved.exists():
        raise G0Error("G0 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def publish_output(output: Path, staging: Path) -> None:
    staging.rename(output)
