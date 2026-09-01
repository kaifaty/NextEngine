#!/usr/bin/env python3
"""Frozen lineage and access boundary for Physical Sound V20 M0b."""

from __future__ import annotations

import json
import os
import platform
import subprocess
from dataclasses import dataclass
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
import physical_sound_v20_m0_common as m0_common
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v20-m0b-confound-resistant-metric-calibration"
REVISION = "development-confound-resistant-metric-calibration-v0"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v20-m0b.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v20-m0b.corpus.v0"
METRIC_SCHEMA = "nextengine.experimental-physical-sound-v20-m0b.metric.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v20-m0b.report.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v20-m0b.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v20-m0b-confound-resistant-metric-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "9480a5fba5a98e775b9ba6f14c08a20f0279949fc74f5939082ad3e0a36d9cbb"
M0_RESULT_PATH = Path(
    "docs/development/"
    "physical-sound-v20-m0-phase-consistent-metric-result-2026-09-01.md"
)
M0_RESULT_SHA256 = "78503fbde6c706f06ac785b130e4009ca38ba2b4c0dbbde8656b816353a30bfa"
RESEARCH_PATH = Path(
    "docs/development/"
    "physical-sound-v20-m0b-confound-resistant-metric-research-2026-09-01.md"
)
RESEARCH_SHA256 = "600554aacc76f3ddda777fa48e23aedba2e88a1a146d9fa3fafdd00d20950b7d"
M0_IMPLEMENTATION_COMMIT = "b8bbeb39e574ab2fcbbe91a628e36183383baceb"
M0_IMPLEMENTATION_HASHES = {
    "physical_sound_v20_m0_common.py": (
        "a742b4e941d08cd0dd9ed55d07e4d710359d61de1762a17e2b4a415e78c8535d"
    ),
    "physical_sound_v20_m0_metric_lab.py": (
        "d1bbeb8f838c47c207a1e213cdb82d4ae6eaad5089a90b67a54d38da771aee69"
    ),
}
M0_TEST_HASH = "cbe9235a106487a88735f441f1c159920a4d3669621044aeae66a50441932bf0"
M0_TREE_DIGEST = "12f3ce7298dadff53688262f9d3f34b202b9881d7bfbf4eaf2c60a753549fbfd"
M0_FILE_HASHES = {
    "access-ledger.json": "5c29bd2ee5d0decdad9435a823727a47281754df227ba549cb2a19d14f4a2899",
    "corpus.json": "2ef1fd1f69161824dc7856a27f250a29d45b4e828fe8150399db07e422cb0e0e",
    "manifest.json": "c52f0c8b1cd49e84657b556066339cc2c05cb8792534959ec442af5de51def51",
    "metrics.jsonl": "550306a20306ea251f321a802df3e3f4b3c43917be2ccf7364ccf95994be99ae",
    "report.json": "2bc3c3cf39195e0c9b6fef604182cb4dabb947324cba563cf716c0fe0cf6da51",
}

B0_TREE_DIGEST = m0_common.B0_TREE_DIGEST
F0_TREE_DIGEST = m0_common.F0_TREE_DIGEST
DEVELOPMENT_ROW_ROOT = m0_common.DEVELOPMENT_ROW_ROOT
DEVELOPMENT_RECORD_ROOT = m0_common.DEVELOPMENT_RECORD_ROOT
EXPERIMENT_ROOT = m0_common.EXPERIMENT_ROOT
REQUIRED_ENVIRONMENT = m0_common.REQUIRED_ENVIRONMENT
IMPLEMENTATION_FILES = (
    "physical_sound_v20_m0b_common.py",
    "physical_sound_v20_m0b_metric_lab.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v20_m0b_metric_lab.py")

ZERO_ACCESS = {
    **m0_common.ZERO_ACCESS,
    "m0_predecessor_metric_values_used_for_case_selection": 0,
}


class M0bError(RuntimeError):
    """Stable failure for the frozen M0b experiment."""


@dataclass(frozen=True)
class M0Predecessor:
    root: Path
    bytes_read: int
    file_map: dict[str, str]
    report: dict[str, Any]


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


sha256_bytes = m0_common.sha256_bytes
sha256_file = m0_common.sha256_file
canonical_json = m0_common.canonical_json
canonical_json_lines = m0_common.canonical_json_lines
identity_hash = m0_common.identity_hash
directory_file_map = m0_common.directory_file_map
tree_digest = m0_common.tree_digest
compare_directories = m0_common.compare_directories


def verify_protocol_environment() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        M0_RESULT_PATH: M0_RESULT_SHA256,
        RESEARCH_PATH: RESEARCH_SHA256,
        Path("lab/tests/test_physical_sound_v20_m0_metric_lab.py"): M0_TEST_HASH,
        **{
            Path("lab/scripts") / name: digest
            for name, digest in M0_IMPLEMENTATION_HASHES.items()
        },
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise M0bError(f"M0b dependency hash changed for {path}: {observed}")
    m0_environment = m0_common.verify_protocol_environment()
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT or m0_environment["libraries"] != libraries:
        raise M0bError(f"M0b environment changed: {libraries}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise M0bError("M0b torch determinism changed")
    return {"libraries": libraries}


def load_predecessor(root: Path) -> M0Predecessor:
    if not root.is_absolute():
        raise M0bError("M0b predecessor root must be absolute")
    experiment_root = EXPERIMENT_ROOT.resolve()
    resolved = root.resolve(strict=True)
    if not resolved.is_relative_to(experiment_root) or not resolved.is_dir():
        raise M0bError("M0b predecessor root escaped the experiment boundary")
    if root.is_symlink():
        raise M0bError("M0b predecessor root cannot be a symlink")
    file_map = directory_file_map(resolved)
    if file_map != M0_FILE_HASHES:
        raise M0bError("M0b predecessor member identity changed")
    observed_tree = tree_digest(file_map)
    if observed_tree != M0_TREE_DIGEST:
        raise M0bError(f"M0b predecessor tree changed: {observed_tree}")
    try:
        report = json.loads((resolved / "report.json").read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise M0bError("M0b predecessor report is invalid") from error
    if not isinstance(report, dict) or report.get("single_run_pass") is not False:
        raise M0bError("M0b predecessor decision changed")
    return M0Predecessor(
        root=resolved,
        bytes_read=sum((resolved / name).stat().st_size for name in file_map),
        file_map=file_map,
        report=report,
    )


def load_dependencies(
    b0_root: Path, f0_root: Path
) -> m0_common.i0_common.LoadedDependencies:
    try:
        return m0_common.load_dependencies(b0_root, f0_root)
    except m0_common.M0Error as error:
        raise M0bError(str(error).replace("M0", "M0b")) from error


def development_objects() -> tuple[m0_common.i0_common.f0_common.FieldObject, ...]:
    try:
        return m0_common.development_objects()
    except m0_common.M0Error as error:
        raise M0bError(str(error).replace("M0", "M0b")) from error


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
        raise M0bError("M0b implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise M0bError("M0b implementation commit is unavailable")
    return commit


def prepare_output(output: Path) -> tuple[Path, Path]:
    try:
        return m0_common.prepare_output(output)
    except m0_common.M0Error as error:
        raise M0bError(str(error).replace("M0", "M0b")) from error


def publish_output(staging: Path, output: Path) -> None:
    try:
        m0_common.publish_output(staging, output)
    except m0_common.M0Error as error:
        raise M0bError(str(error).replace("M0", "M0b")) from error


def abandon_output(staging: Path) -> None:
    m0_common.abandon_output(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    try:
        m0_common.write_files(directory, files)
    except m0_common.M0Error as error:
        raise M0bError(str(error).replace("M0", "M0b")) from error
