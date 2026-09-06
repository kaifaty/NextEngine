#!/usr/bin/env python3
"""Frozen corpus and artifact boundary for Physical Sound V18 B0."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import shutil
import sys
from dataclasses import dataclass
from io import BytesIO
from pathlib import Path
from typing import Any, Iterable

for _thread_variable in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
):
    os.environ[_thread_variable] = "1"

import numpy as np
import scipy


STUDY_ID = "physical-sound-v18-b0-deterministic-global-baseline"
REVISION = "hybrid-truth-global-baseline-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v18-b0.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v18-b0.report.v1"
MODEL_SCHEMA = "nextengine.experimental-physical-sound-v18-b0.model.v1"
PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v18-p0b-hybrid-truth-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "216acd012108725cbcbcf5454f1db707a6ac688e182bfd9fa92d45082280475a"
PARENT_PROTOCOL_SHA256 = (
    "39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e"
)
IMPLEMENTATION_FILES = (
    "physical_sound_v18_b0_common.py",
    "physical_sound_v18_b0_model.py",
    "physical_sound_v18_b0_oracle.py",
)
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
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
RIDGE_LAMBDA = 1.0e-6
EXPERIMENT_ROOT = Path(
    "/home/kaifaty/.codex/experiments/nextengine/physical-sound"
)
ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "opened_v16_artifact_bytes_read": 0,
    "opened_v17_g0_artifact_bytes_read": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
}


class B0Error(RuntimeError):
    """Stable failure for the frozen B0 experiment."""


@dataclass(frozen=True)
class BRow:
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
    frequency_scale: float
    damping_scale: float
    frequencies: np.ndarray
    damping: np.ndarray

    def input_record(self) -> dict[str, Any]:
        return {
            "aspect": self.aspect,
            "cell": self.cell,
            "damping_scale": self.damping_scale,
            "frequency_scale": self.frequency_scale,
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

    def record(self) -> dict[str, Any]:
        result = self.input_record()
        result.update({"damping": self.damping, "frequencies": self.frequencies})
        return result


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def _json_value(value: Any) -> Any:
    if isinstance(value, dict):
        return {str(key): _json_value(item) for key, item in value.items()}
    if isinstance(value, (tuple, list)):
        return [_json_value(item) for item in value]
    if isinstance(value, np.ndarray):
        return _json_value(value.tolist())
    if isinstance(value, np.generic):
        return value.item()
    if isinstance(value, Path):
        return value.as_posix()
    return value


def canonical_json(value: Any) -> bytes:
    return (
        json.dumps(
            _json_value(value),
            allow_nan=False,
            ensure_ascii=False,
            separators=(",", ":"),
            sort_keys=True,
        )
        + "\n"
    ).encode("utf-8")


def array_bytes(value: np.ndarray) -> bytes:
    array = np.asarray(value)
    if array.dtype.kind not in "fiu":
        raise B0Error(f"unsupported canonical array dtype: {array.dtype}")
    if array.dtype.kind == "f" and not np.isfinite(array).all():
        raise B0Error("canonical array is non-finite")
    little = np.ascontiguousarray(array.astype(array.dtype.newbyteorder("<"), copy=False))
    buffer = BytesIO()
    np.save(buffer, little, allow_pickle=False)
    return buffer.getvalue()


def array_from_bytes(value: bytes, shape: tuple[int, ...]) -> np.ndarray:
    try:
        array = np.load(BytesIO(value), allow_pickle=False)
    except (OSError, ValueError) as error:
        raise B0Error("invalid canonical array") from error
    if array.shape != shape or array.dtype.str != "<f8":
        raise B0Error(
            f"canonical float64 array shape/dtype changed: {array.shape}/{array.dtype.str}"
        )
    if not np.isfinite(array).all():
        raise B0Error("canonical array is non-finite")
    return np.asarray(array, dtype=np.float64)


def radical_inverse(index: int, base: int) -> float:
    if index <= 0 or base <= 1:
        raise B0Error("radical inverse requires positive index and base > 1")
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
) -> tuple[float, float, np.ndarray, np.ndarray]:
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
    damping_scale = material["decay"]
    damping_multiplier = (
        (1.0 + 0.08 * mode)
        * TOPOLOGY_DAMPING[topology_index]
        * SUPPORT_DAMPING[support_index]
        * (1.0 + 0.06 * abs(aspect_log))
    )
    frequencies = frequency_scale * frequency_ratio
    damping = damping_scale * damping_multiplier
    if not hard_validate(frequencies, damping):
        raise B0Error("generated B truth violates frozen bounds")
    return frequency_scale, damping_scale, frequencies, damping


def _make_row(
    role: str,
    stratum: str,
    cell: int,
    replicate: int,
    halton_index: int,
    identity_prefix: str,
) -> BRow:
    material_index = cell // 8
    topology_index = (cell % 8) // 2
    support_index = cell % 2
    aspect = 0.70 + 0.80 * radical_inverse(halton_index, 3)
    slenderness = 0.004 + 0.004 * radical_inverse(halton_index, 5)
    if stratum == "scale-transfer":
        if cell % 2 == 0:
            length_m = 0.15 + 0.05 * radical_inverse(halton_index, 2)
        else:
            length_m = 0.50 + 0.12 * radical_inverse(halton_index, 2)
    elif stratum == "interpolation":
        length_m = 0.22 + 0.24 * radical_inverse(halton_index, 2)
    else:
        raise B0Error(f"unknown B stratum: {stratum}")
    wall_m = length_m * slenderness
    frequency_scale, damping_scale, frequencies, damping = _truth(
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
        f"{identity_prefix}-{role}-{material}-{topology}-{support}-{replicate}".lower()
    )
    return BRow(
        object_id=object_id,
        role=role,
        stratum=stratum,
        cell=cell,
        replicate=replicate,
        halton_index=halton_index,
        material=material,
        topology=topology,
        support=support,
        length_m=length_m,
        aspect=aspect,
        slenderness=slenderness,
        wall_m=wall_m,
        frequency_scale=frequency_scale,
        damping_scale=damping_scale,
        frequencies=frequencies,
        damping=damping,
    )


def generate_training_rows() -> tuple[BRow, ...]:
    rows = tuple(
        _make_row(
            "train",
            "interpolation",
            cell,
            replicate,
            1 + 3 * cell + replicate,
            "g",
        )
        for cell in range(24)
        for replicate in range(3)
    )
    _validate_rows(rows, {"train": 72})
    return rows


def generate_fresh_rows() -> tuple[BRow, ...]:
    rows: list[BRow] = []
    roles = (
        ("development", "interpolation", 801),
        ("test-interpolation", "interpolation", 901),
        ("test-scale-transfer", "scale-transfer", 1001),
    )
    for role, stratum, base in roles:
        for cell in range(24):
            rows.append(_make_row(role, stratum, cell, 0, base + cell, "b"))
    result = tuple(rows)
    _validate_rows(
        result,
        {
            "development": 24,
            "test-interpolation": 24,
            "test-scale-transfer": 24,
        },
    )
    return result


def _validate_rows(rows: tuple[BRow, ...], expected: dict[str, int]) -> None:
    counts = {role: sum(row.role == role for row in rows) for role in expected}
    if counts != expected or len(rows) != sum(expected.values()):
        raise B0Error(f"B corpus counts changed: {counts}")
    identities = [row.object_id for row in rows]
    if len(set(identities)) != len(identities):
        raise B0Error("B corpus object identity collision")
    if any(not hard_validate(row.frequencies, row.damping) for row in rows):
        raise B0Error("B corpus hard validation failed")


def row_signature(row: BRow) -> tuple[Any, ...]:
    return (
        row.material,
        row.topology,
        row.support,
        row.length_m,
        row.aspect,
        row.wall_m,
    )


def v17_opened_g_signatures() -> set[tuple[Any, ...]]:
    rows: list[BRow] = []
    for cell in range(24):
        for replicate in range(3):
            rows.append(
                _make_row(
                    "train",
                    "interpolation",
                    cell,
                    replicate,
                    1 + 3 * cell + replicate,
                    "g",
                )
            )
        rows.append(_make_row("development", "interpolation", cell, 0, 101 + cell, "g"))
        rows.append(
            _make_row("test-interpolation", "interpolation", cell, 0, 201 + cell, "g")
        )
        rows.append(
            _make_row("test-scale-transfer", "scale-transfer", cell, 0, 301 + cell, "g")
        )
    return {row_signature(row) for row in rows}


def hard_validate(frequencies: np.ndarray, damping: np.ndarray) -> bool:
    frequencies = np.asarray(frequencies, dtype=np.float64)
    damping = np.asarray(damping, dtype=np.float64)
    return bool(
        frequencies.shape == (MODE_COUNT,)
        and damping.shape == (MODE_COUNT,)
        and np.isfinite(frequencies).all()
        and np.isfinite(damping).all()
        and frequencies[0] >= 40.0
        and frequencies[-1] <= 7_500.0
        and np.all(np.diff(frequencies) > 0.0)
        and np.all(damping > 0.0)
        and np.all(damping <= 128.0)
    )


def frequency_cents(prediction: np.ndarray, truth: np.ndarray) -> np.ndarray:
    return np.abs(1_200.0 * np.log2(np.asarray(prediction) / np.asarray(truth)))


def damping_relative(prediction: np.ndarray, truth: np.ndarray) -> np.ndarray:
    prediction = np.asarray(prediction)
    truth = np.asarray(truth)
    return np.abs(prediction - truth) / truth


def identity_root(rows: Iterable[BRow]) -> str:
    return sha256_bytes(canonical_json([row.record() for row in rows]))


def verify_protocol_and_environment() -> dict[str, Any]:
    protocol = repository_root() / PROTOCOL_PATH
    observed_protocol = sha256_file(protocol)
    if observed_protocol != PROTOCOL_SHA256:
        raise B0Error(
            f"B0 protocol hash changed: {observed_protocol} != {PROTOCOL_SHA256}"
        )
    observed = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
    }
    if observed != REQUIRED_ENVIRONMENT:
        raise B0Error(f"B0 environment changed: {observed}")
    threads = {
        name: os.environ.get(name)
        for name in (
            "OMP_NUM_THREADS",
            "OPENBLAS_NUM_THREADS",
            "MKL_NUM_THREADS",
            "NUMEXPR_NUM_THREADS",
        )
    }
    if any(value != "1" for value in threads.values()):
        raise B0Error(f"B0 thread environment changed: {threads}")
    return {"libraries": observed, "threads": threads}


def implementation_hashes() -> dict[str, str]:
    root = Path(__file__).resolve().parent
    result = {name: sha256_file(root / name) for name in IMPLEMENTATION_FILES}
    if len(result) != len(IMPLEMENTATION_FILES):
        raise B0Error("B0 implementation file list changed")
    return result


def _check_no_symlink(path: Path, root: Path) -> None:
    current = path
    while current != root:
        if current.is_symlink():
            raise B0Error(f"symlinked B0 output path is forbidden: {current}")
        if current.parent == current:
            raise B0Error("B0 output path escaped experiment root")
        current = current.parent


def prepare_output(output: Path) -> tuple[Path, Path]:
    if not output.is_absolute():
        raise B0Error("B0 output path must be absolute")
    root = EXPERIMENT_ROOT.resolve()
    resolved = output.resolve(strict=False)
    if resolved == root or not resolved.is_relative_to(root):
        raise B0Error("B0 output path must be below the external experiment root")
    _check_no_symlink(output, EXPERIMENT_ROOT)
    if output.exists():
        raise B0Error("B0 output path must not already exist")
    output.parent.mkdir(parents=True, exist_ok=True)
    staging = output.with_name(f".{output.name}.staging-{os.getpid()}")
    if staging.exists() or staging.is_symlink():
        raise B0Error("B0 staging path already exists")
    staging.mkdir()
    return staging, output


def publish_output(staging: Path, output: Path) -> None:
    if output.exists():
        raise B0Error("B0 output appeared before atomic publication")
    os.replace(staging, output)


def abandon_output(staging: Path) -> None:
    if staging.exists() and staging.is_dir() and not staging.is_symlink():
        shutil.rmtree(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    for name in sorted(files):
        if Path(name).name != name:
            raise B0Error(f"invalid B0 artifact name: {name}")
        (directory / name).write_bytes(files[name])


def directory_file_map(directory: Path) -> dict[str, str]:
    return {
        path.name: sha256_file(path)
        for path in sorted(directory.iterdir(), key=lambda value: value.name)
        if path.is_file()
    }


def tree_digest(file_map: dict[str, str]) -> str:
    return sha256_bytes(canonical_json(file_map))


def compare_directories(left: Path, right: Path) -> dict[str, Any]:
    left_map = directory_file_map(left)
    right_map = directory_file_map(right)
    return {
        "byte_identical": left_map == right_map,
        "file_count": len(left_map),
        "left_tree_digest": tree_digest(left_map),
        "right_tree_digest": tree_digest(right_map),
    }
