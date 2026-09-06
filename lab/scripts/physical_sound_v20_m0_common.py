#!/usr/bin/env python3
"""Frozen dependencies and access boundary for Physical Sound V20 M0."""

from __future__ import annotations

import json
import os
import platform
import subprocess
from pathlib import Path
from typing import Any

for _thread_variable in (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "NUMEXPR_NUM_THREADS",
):
    os.environ[_thread_variable] = "1"

import numpy as np
import physical_sound_v19_i0_common as i0_common
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v20-m0-phase-consistent-metric-calibration"
REVISION = "development-metric-calibration-v0"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v20-m0.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v20-m0.corpus.v0"
METRIC_SCHEMA = "nextengine.experimental-physical-sound-v20-m0.metric.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v20-m0.report.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v20-m0.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v20-m0-phase-consistent-metric-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "aa27ecfe9c7c91db47568cd41a9270592cc0bb796a6ae16ebc3e72b04b25e26a"
I0_RESULT_PATH = Path(
    "docs/development/physical-sound-v19-i0-development-control-result-2026-09-01.md"
)
I0_RESULT_SHA256 = "7608cff54a09ff322aba0c5f4c0f696b56231262f7cda7793fbf3b2440bd69da"
I0_IMPLEMENTATION_HASHES = {
    "physical_sound_v19_i0_common.py": (
        "9606f4c7a3a2d3a2ed893e5cabdc0b49c976da836540790a9836a4888eaa5170"
    ),
    "physical_sound_v19_i0_oracle.py": (
        "704265828cd7213eec6ac8d6df301cc484d70eaf0c170a6b2f62d5077a981f7d"
    ),
}
I0_IMPLEMENTATION_COMMIT = "22988542b834ce1eac75a8aa57ba3dabdb716143"
B0_TREE_DIGEST = i0_common.B0_TREE_DIGEST
F0_TREE_DIGEST = i0_common.F0_TREE_DIGEST
DEVELOPMENT_ROW_ROOT = (
    "825d0aa5de9bda2ffefc0737cfd8221f8e55585d3d6431b19bf5ef4db33806a2"
)
DEVELOPMENT_RECORD_ROOT = (
    "f21db879bdf497cb37d52a5dfd3fcc92d488c30d9ad9e013a93d348898f0785e"
)

IMPLEMENTATION_FILES = (
    "physical_sound_v20_m0_common.py",
    "physical_sound_v20_m0_metric_lab.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v20_m0_metric_lab.py")
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
    "torch": "2.13.0+cu130",
}
EXPERIMENT_ROOT = i0_common.EXPERIMENT_ROOT

ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "i1_rows_generated": 0,
    "integration_rows_generated": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "opened_v16_artifact_bytes_read": 0,
    "opened_v17_artifact_bytes_read": 0,
    "opened_v18_non_b0_artifact_bytes_read": 0,
    "opened_v19_non_f0_artifact_bytes_read": 0,
    "optional_successor_rows_generated": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
}


class M0Error(RuntimeError):
    """Stable failure for the frozen development-only metric laboratory."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


sha256_bytes = i0_common.sha256_bytes
sha256_file = i0_common.sha256_file
canonical_json = i0_common.canonical_json
canonical_json_lines = i0_common.canonical_json_lines
identity_hash = i0_common.identity_hash
directory_file_map = i0_common.directory_file_map
tree_digest = i0_common.tree_digest
compare_directories = i0_common.compare_directories


def verify_protocol_environment() -> dict[str, Any]:
    """Verify every frozen dependency before development values are rendered."""
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        I0_RESULT_PATH: I0_RESULT_SHA256,
        **{
            Path("lab/scripts") / name: digest
            for name, digest in I0_IMPLEMENTATION_HASHES.items()
        },
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise M0Error(f"M0 dependency hash changed for {path}: {observed}")
    i0_environment = i0_common.verify_protocol_environment()
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT or i0_environment["libraries"] != libraries:
        raise M0Error(f"M0 environment changed: {libraries}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise M0Error("M0 torch determinism changed")
    return {"libraries": libraries}


def load_dependencies(b0_root: Path, f0_root: Path) -> i0_common.LoadedDependencies:
    try:
        return i0_common.load_dependencies(b0_root, f0_root)
    except i0_common.I0Error as error:
        raise M0Error(str(error).replace("I0", "M0")) from error


def development_objects() -> tuple[i0_common.f0_common.FieldObject, ...]:
    """Generate only the already-opened V19 development views."""
    try:
        objects = i0_common.f0_common.generate_objects("development")
    except i0_common.f0_common.F0Error as error:
        raise M0Error(str(error).replace("F0", "M0")) from error
    rows = i0_common.f0_common.generate_rows("development")
    observed_rows = sha256_bytes(canonical_json([row.record() for row in rows]))
    observed_records = sha256_bytes(
        canonical_json([item.identity_record() for item in objects])
    )
    if observed_rows != DEVELOPMENT_ROW_ROOT:
        raise M0Error(f"M0 development row root changed: {observed_rows}")
    if observed_records != DEVELOPMENT_RECORD_ROOT:
        raise M0Error(f"M0 development record root changed: {observed_records}")
    if len(objects) != 24 or len({item.row.object_id for item in objects}) != 24:
        raise M0Error("M0 development view identity changed")
    if any(item.row.role != "development" for item in objects):
        raise M0Error("M0 attempted to generate a non-development role")
    return objects


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    return {name: sha256_file(directory / name) for name in IMPLEMENTATION_FILES}


def verify_committed_implementation() -> str:
    root = repository_root()
    paths = [str(Path("lab/scripts") / name) for name in IMPLEMENTATION_FILES]
    paths.append(str(TEST_FILE))
    status = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=all", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    if status.strip():
        raise M0Error("M0 implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise M0Error("M0 implementation commit is unavailable")
    return commit


def prepare_output(output: Path) -> tuple[Path, Path]:
    try:
        return i0_common.prepare_output(output)
    except i0_common.I0Error as error:
        raise M0Error(str(error).replace("I0", "M0")) from error


def publish_output(staging: Path, output: Path) -> None:
    try:
        i0_common.publish_output(staging, output)
    except i0_common.I0Error as error:
        raise M0Error(str(error).replace("I0", "M0")) from error


def abandon_output(staging: Path) -> None:
    i0_common.abandon_output(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    try:
        i0_common.write_files(directory, files)
    except i0_common.I0Error as error:
        raise M0Error(str(error).replace("I0", "M0")) from error


def json_object(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise M0Error(f"M0 invalid JSON artifact: {path.name}") from error
    if not isinstance(value, dict):
        raise M0Error(f"M0 JSON artifact is not an object: {path.name}")
    return value
