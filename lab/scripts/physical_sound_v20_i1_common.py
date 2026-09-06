#!/usr/bin/env python3
"""Frozen metadata, lineage, and value boundary for Physical Sound V20 I1."""

from __future__ import annotations

import json
import os
import platform
import subprocess
from dataclasses import dataclass, replace
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
import physical_sound_v20_m0b_common as m0b_common
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v20-i1-frozen-hybrid-integration"
REVISION = "b0-c0-f0-m0b-frozen-integration-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.corpus.v0"
METRIC_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.metric.v0"
DECISION_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.decision.v0"
FALLBACK_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.fallback.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.report.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v20-i1.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/physical-sound-v20-p0c-i1-integration-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "b591164a09c4a9aeb078cef7ab5f4dbebd1c2117b09a8fd21277be79427218c5"
M0B_RESULT_PATH = Path(
    "docs/development/"
    "physical-sound-v20-m0b-confound-resistant-metric-result-2026-09-01.md"
)
M0B_RESULT_SHA256 = "465e806a9052f4f86ceeb07407a42d09703272f4d36054fb5a164b3526d663c9"
M0B_IMPLEMENTATION_COMMIT = "cfee8cb844b945d4ba85ef3fe768947ee563e8ed"
M0B_IMPLEMENTATION_HASHES = {
    "physical_sound_v20_m0b_common.py": (
        "8f5ad986847f971e32e1c7e814cfb7c162143ad2a869824b6dc5cc5fb624fcce"
    ),
    "physical_sound_v20_m0b_metric_lab.py": (
        "b74208de60fa1889cd035b177919244b5a50e7d7416eaa1eaf7b347d232c4bcb"
    ),
}
M0B_TEST_HASH = "16021bbbdfee2d0b099fd1015fb472a487a39be92b0d54ce269b9a0e45982a62"
M0B_TREE_DIGEST = "7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22"
M0B_FILE_HASHES = {
    "access-ledger.json": "b68fcf2ae50edcb7aedc27d2b44cd410ddfdf3de46af9871badbd54464eab306",
    "corpus.json": "c4c23b655e904173af6c39c8d0044bd42536e3b71194998552a454071b35dda0",
    "manifest.json": "063cd43bc3abbbf8027a843a725954032f3f8dd870a04d55e6db5c535b784d87",
    "metrics.jsonl": "259851b487f9baaad21df8d4caf15e6a21fa6b69da5d0d10234b2a8fdefc003f",
    "report.json": "4ddedcd285ac2a9d12cd266f2901067c1db97cf3fff27c2bb112b5117b8350ec",
}

B0_TREE_DIGEST = m0b_common.B0_TREE_DIGEST
F0_TREE_DIGEST = m0b_common.F0_TREE_DIGEST
I1_ROW_ROOT = "f1709914a56a82482f1c79133453c13da1de6d36aac25c1f5c9db21fb654b7b8"
EXPERIMENT_ROOT = m0b_common.EXPERIMENT_ROOT
REQUIRED_ENVIRONMENT = m0b_common.REQUIRED_ENVIRONMENT
IMPLEMENTATION_FILES = (
    "physical_sound_v20_i1_common.py",
    "physical_sound_v20_i1_integration.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v20_i1_integration.py")

ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "i1_rows_generated_before_implementation_commit": 0,
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
    "v19_integration_rows_generated": 0,
}


class I1Error(RuntimeError):
    """Stable failure for the frozen I1 integration experiment."""


@dataclass(frozen=True)
class M0bCertificate:
    root: Path
    bytes_read: int
    file_map: dict[str, str]
    report: dict[str, Any]


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


sha256_bytes = m0b_common.sha256_bytes
sha256_file = m0b_common.sha256_file
canonical_json = m0b_common.canonical_json
canonical_json_lines = m0b_common.canonical_json_lines
identity_hash = m0b_common.identity_hash
directory_file_map = m0b_common.directory_file_map
tree_digest = m0b_common.tree_digest
compare_directories = m0b_common.compare_directories
deterministic_npz = m0b_common.m0_common.i0_common.deterministic_npz
geometry_npz = m0b_common.m0_common.i0_common.geometry_npz


def verify_protocol_environment() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        M0B_RESULT_PATH: M0B_RESULT_SHA256,
        Path("lab/tests/test_physical_sound_v20_m0b_metric_lab.py"): M0B_TEST_HASH,
        **{
            Path("lab/scripts") / name: digest
            for name, digest in M0B_IMPLEMENTATION_HASHES.items()
        },
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise I1Error(f"I1 dependency hash changed for {path}: {observed}")
    m0b_environment = m0b_common.verify_protocol_environment()
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT or m0b_environment["libraries"] != libraries:
        raise I1Error(f"I1 environment changed: {libraries}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise I1Error("I1 torch determinism changed")
    return {"libraries": libraries}


def load_m0b_certificate(root: Path) -> M0bCertificate:
    if not root.is_absolute():
        raise I1Error("I1 M0b root must be absolute")
    experiment_root = EXPERIMENT_ROOT.resolve()
    resolved = root.resolve(strict=True)
    if not resolved.is_relative_to(experiment_root) or not resolved.is_dir():
        raise I1Error("I1 M0b root escaped the experiment boundary")
    if root.is_symlink():
        raise I1Error("I1 M0b root cannot be a symlink")
    file_map = directory_file_map(resolved)
    if file_map != M0B_FILE_HASHES:
        raise I1Error("I1 M0b certificate member identity changed")
    if tree_digest(file_map) != M0B_TREE_DIGEST:
        raise I1Error("I1 M0b certificate tree changed")
    try:
        report = json.loads((resolved / "report.json").read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise I1Error("I1 M0b report is invalid") from error
    if not isinstance(report, dict) or report.get("single_run_pass") is not True:
        raise I1Error("I1 M0b certificate decision changed")
    return M0bCertificate(
        resolved,
        sum((resolved / name).stat().st_size for name in file_map),
        file_map,
        report,
    )


def load_dependencies(
    b0_root: Path, f0_root: Path
) -> m0b_common.m0_common.i0_common.LoadedDependencies:
    try:
        return m0b_common.load_dependencies(b0_root, f0_root)
    except m0b_common.M0bError as error:
        raise I1Error(str(error).replace("M0b", "I1")) from error


def i1_rows() -> tuple[m0b_common.m0_common.i0_common.f0_common.FieldRow, ...]:
    f0 = m0b_common.m0_common.i0_common.f0_common
    rows = []
    for cell in range(12):
        halton_index = 1_701 + cell
        material = f0.MATERIAL_ORDER[cell // 4]
        topology = f0.TOPOLOGY_ORDER[cell % 4]
        support = f0.SUPPORT_ORDER[(cell + 1) % 2]
        length_m = 0.21 + 0.27 * f0.geometry_common.radical_inverse(halton_index, 2)
        aspect = 0.72 + 0.76 * f0.geometry_common.radical_inverse(halton_index, 3)
        slenderness = 0.004 + 0.004 * f0.geometry_common.radical_inverse(
            halton_index, 5
        )
        group = (
            f"v20-i1-integration-{material}-{topology}-{support}-{halton_index}".lower()
        )
        primary = f0.FieldRow(
            physical_group_id=group,
            object_id=f"{group}-primary",
            role="integration",
            cell=cell,
            halton_index=halton_index,
            material=material,
            topology=topology,
            support=support,
            length_m=length_m,
            aspect=aspect,
            slenderness=slenderness,
            wall_m=length_m * slenderness,
            grid_u=23 + cell % 2,
            grid_v=18 + cell % 3,
        )
        rows.extend(
            (
                primary,
                replace(
                    primary,
                    object_id=f"{group}-twin",
                    grid_u=primary.grid_u + 1,
                    grid_v=primary.grid_v - 1,
                ),
            )
        )
    observed = sha256_bytes(canonical_json([row.record() for row in rows]))
    if observed != I1_ROW_ROOT:
        raise I1Error(f"I1 metadata row root changed: {observed}")
    return tuple(rows)


def generate_i1_objects(
    implementation_commit: str,
) -> tuple[m0b_common.m0_common.i0_common.f0_common.FieldObject, ...]:
    if len(implementation_commit) != 40:
        raise I1Error("I1 values require a committed implementation identity")
    if implementation_commit != verify_committed_implementation():
        raise I1Error("I1 implementation commit identity changed")
    i0 = m0b_common.m0_common.i0_common
    objects = tuple(i0._integration_object(row) for row in i1_rows())
    if len(objects) != 24 or len({item.mesh.mesh_hash for item in objects}) != 24:
        raise I1Error("I1 object identity changed")
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
        raise I1Error("I1 implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise I1Error("I1 implementation commit is unavailable")
    return commit


def prepare_output(output: Path) -> tuple[Path, Path]:
    try:
        return m0b_common.prepare_output(output)
    except m0b_common.M0bError as error:
        raise I1Error(str(error).replace("M0b", "I1")) from error


def publish_output(staging: Path, output: Path) -> None:
    try:
        m0b_common.publish_output(staging, output)
    except m0b_common.M0bError as error:
        raise I1Error(str(error).replace("M0b", "I1")) from error


def abandon_output(staging: Path) -> None:
    m0b_common.abandon_output(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    try:
        m0b_common.write_files(directory, files)
    except m0b_common.M0bError as error:
        raise I1Error(str(error).replace("M0b", "I1")) from error
