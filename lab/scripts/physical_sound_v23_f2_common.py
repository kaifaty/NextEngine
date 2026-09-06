#!/usr/bin/env python3
"""Fresh identities and fail-closed guards for Physical Sound V23 F2a."""

from __future__ import annotations

import os
import platform
import subprocess
from dataclasses import replace
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
import physical_sound_v21_f1_common as lineage
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v23-f2-fixed-feature-ridge-field"
REVISION = "fixed-feature-ridge-prior-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v23-f2.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v23-f2.corpus.v0"
METRIC_SCHEMA = "nextengine.experimental-physical-sound-v23-f2.metric.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v23-f2.report.v0"
SELECTION_SCHEMA = "nextengine.experimental-physical-sound-v23-f2.selection.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v23-f2.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/physical-sound-v23-p2a-fixed-feature-ridge-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "7de82551dac2f0bf2d0bcc57ef51018f22739543c1f39285ae223e38d23d7db0"
RESULT_PATH = Path(
    "docs/development/physical-sound-v22-f1r-resource-bounded-result-2026-09-01.md"
)
RESULT_SHA256 = "a99f4d32053ce45c53bde5d8f821943bb3dfa6531f416ca2b362e32839164885"
RESEARCH_PATH = Path(
    "docs/development/physical-sound-v23-closed-form-field-research-2026-09-01.md"
)
RESEARCH_SHA256 = "771081265840eaf17e8741df2d246130db1cf22670ac9674e319454be6197057"
ROADMAP_PATH = Path("docs/plans/physical-sound-synthesis-roadmap-v23.md")
ROADMAP_SHA256 = "c18fe9daf6bd4d13e3c52ee87d17417f425400ecc434728c1ed34d4f044c5515"

PREDECESSOR_HASHES = {
    Path("lab/scripts/physical_sound_v22_f1r_common.py"): (
        "c97288eac3ba988da712b8a6cf8c21c1870939026c70b9b6531d1fc9fecff1a8"
    ),
    Path("lab/scripts/physical_sound_v22_f1r_model.py"): (
        "36aea212d92861ff2643ed4bad14369c5e26e0131d42c25c0e0d12f53fc1fe36"
    ),
    Path("lab/scripts/physical_sound_v22_f1r_tournament.py"): (
        "503fae494af10aa819617b269dd55b93aecaae863b281d452ffb2c462a46c18f"
    ),
    Path("lab/tests/test_physical_sound_v22_f1r_tournament.py"): (
        "ce1f5b817b8273c19e7b41e765c8e9ca8c44cc6b8d21b964b3cb8c72345e42ae"
    ),
}

ROLE_BASE = {"train": 2601, "development": 2701, "test": 2801, "integration": 2901}
ROW_ROOTS = {
    "train": "09a79ea5dd20ac3003b56f80482d214cb728f8911c09a940eb45ff5e0a855be8",
    "development": "a29a16a135c44d93f80496bacabf76ac08879234188a904a5ac16491a0decda1",
    "test": "93effeeab47652146a5b96a35da97a5ebd291edb31def2246517dfa81ad38214",
    "integration": "1eb174ef3562cf6bba33d8f6537fab07c1ec6d7afc1a0ae4b72288ebe1e49476",
}
ROW_LEDGER_ROOT = "bea16c3c957c0a7feb23779e2f18b67360edef7cd3699ad1927056c37b4253bc"

IMPLEMENTATION_FILES = (
    "physical_sound_v23_f2_common.py",
    "physical_sound_v23_f2_model.py",
    "physical_sound_v23_f2_tournament.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v23_f2_tournament.py")

ZERO_ACCESS = {
    **lineage.ZERO_ACCESS,
    "v21_artifact_or_staging_bytes_read": 0,
    "v22_artifact_or_staging_bytes_read": 0,
}

f0_common = lineage.f0_common
f0_model = lineage.f0_model
FieldRow = lineage.FieldRow
FieldObject = lineage.FieldObject
F1Error = lineage.F1Error
REQUIRED_ENVIRONMENT = lineage.REQUIRED_ENVIRONMENT
EXPERIMENT_ROOT = lineage.EXPERIMENT_ROOT
MODE_COUNT = lineage.MODE_COUNT
B0_TREE_DIGEST = lineage.B0_TREE_DIGEST
C0_TREE_DIGEST = lineage.C0_TREE_DIGEST
F0_TREE_DIGEST = lineage.F0_TREE_DIGEST
M0B_TREE_DIGEST = lineage.M0B_TREE_DIGEST

repository_root = lineage.repository_root
sha256_bytes = lineage.sha256_bytes
sha256_file = lineage.sha256_file
canonical_json = lineage.canonical_json
canonical_json_lines = lineage.canonical_json_lines
identity_hash = lineage.identity_hash
deterministic_npz = lineage.deterministic_npz
geometry_npz = lineage.geometry_npz
directory_file_map = lineage.directory_file_map
tree_digest = lineage.tree_digest
compare_directories = lineage.compare_directories
load_f0_control = lineage.load_f0_control
prepare_output = lineage.prepare_output
publish_output = lineage.publish_output
abandon_output = lineage.abandon_output
write_files = lineage.write_files


def _grid(role: str, cell: int, replicate: int) -> tuple[int, int]:
    if role == "train":
        return 15 + 2 * replicate + cell % 2, 13 + replicate + cell % 3
    base = {"development": (19, 16), "test": (21, 17), "integration": (23, 18)}
    if role not in base:
        raise F1Error(f"unknown F2 role: {role}")
    return base[role][0] + cell % 2, base[role][1] + cell % 3


def _support_index(role: str, cell: int, replicate: int) -> int:
    if role == "train":
        return (cell + replicate) % 2
    if role in ("development", "integration"):
        return (cell + 1) % 2
    if role == "test":
        return cell % 2
    raise F1Error(f"unknown F2 role: {role}")


def _primary_row(role: str, cell: int, replicate: int) -> FieldRow:
    if role not in ROLE_BASE or not 0 <= cell < 12:
        raise F1Error(f"invalid F2 row request: {role}/{cell}")
    if role == "train":
        if replicate not in (0, 1):
            raise F1Error("F2 train replicate is out of range")
        halton_index = ROLE_BASE[role] + 2 * cell + replicate
    else:
        if replicate != 0:
            raise F1Error("F2 non-train replicate must be zero")
        halton_index = ROLE_BASE[role] + cell
    material = f0_common.MATERIAL_ORDER[cell // 4]
    topology = f0_common.TOPOLOGY_ORDER[cell % 4]
    support = f0_common.SUPPORT_ORDER[_support_index(role, cell, replicate)]
    length_m = 0.21 + 0.27 * f0_common.geometry_common.radical_inverse(halton_index, 2)
    aspect = 0.72 + 0.76 * f0_common.geometry_common.radical_inverse(halton_index, 3)
    slenderness = 0.004 + 0.004 * f0_common.geometry_common.radical_inverse(
        halton_index, 5
    )
    grid_u, grid_v = _grid(role, cell, replicate)
    group = f"v23-f2-{role}-{material}-{topology}-{support}-{halton_index}".lower()
    return FieldRow(
        physical_group_id=group,
        object_id=f"{group}-primary",
        role=role,
        cell=cell,
        halton_index=halton_index,
        material=material,
        topology=topology,
        support=support,
        length_m=length_m,
        aspect=aspect,
        slenderness=slenderness,
        wall_m=length_m * slenderness,
        grid_u=grid_u,
        grid_v=grid_v,
    )


def generate_rows(role: str) -> tuple[FieldRow, ...]:
    if role not in ROLE_BASE:
        raise F1Error(f"unknown F2 role: {role}")
    rows: list[FieldRow] = []
    for cell in range(12):
        replicates = (0, 1) if role == "train" else (0,)
        for replicate in replicates:
            primary = _primary_row(role, cell, replicate)
            rows.extend(
                (
                    primary,
                    replace(
                        primary,
                        object_id=f"{primary.physical_group_id}-twin",
                        grid_u=primary.grid_u + 1,
                        grid_v=primary.grid_v - 1,
                    ),
                )
            )
    observed = sha256_bytes(canonical_json([row.record() for row in rows]))
    if observed != ROW_ROOTS[role]:
        raise F1Error(f"F2 {role} row root changed: {observed}")
    return tuple(rows)


def row_ledger() -> dict[str, dict[str, Any]]:
    result = {
        role: {"row_count": len(generate_rows(role)), "row_root": ROW_ROOTS[role]}
        for role in ("train", "development", "test", "integration")
    }
    if sha256_bytes(canonical_json(result)) != ROW_LEDGER_ROOT:
        raise F1Error("F2 row ledger root changed")
    return result


def generate_objects(role: str, implementation_commit: str) -> tuple[FieldObject, ...]:
    if role not in ("train", "development"):
        raise F1Error(f"F2a cannot generate sealed role: {role}")
    if implementation_commit != verify_committed_implementation():
        raise F1Error("F2 implementation commit identity changed")
    objects = tuple(lineage._generate_object(row) for row in generate_rows(role))
    expected = 48 if role == "train" else 24
    if (
        len(objects) != expected
        or len({item.mesh.mesh_hash for item in objects}) != expected
    ):
        raise F1Error(f"F2 {role} object identity changed")
    return objects


def verify_protocol_environment() -> dict[str, Any]:
    lineage.verify_protocol_environment()
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        RESULT_PATH: RESULT_SHA256,
        RESEARCH_PATH: RESEARCH_SHA256,
        ROADMAP_PATH: ROADMAP_SHA256,
        **PREDECESSOR_HASHES,
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise F1Error(f"F2 dependency hash changed for {path}: {observed}")
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT:
        raise F1Error(f"F2 environment changed: {libraries}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise F1Error("F2 torch determinism changed")
    row_ledger()
    return {"libraries": libraries}


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    result = {}
    for name in IMPLEMENTATION_FILES:
        path = directory / name
        if not path.is_file():
            raise F1Error(f"missing F2 implementation: {name}")
        result[name] = sha256_file(path)
    return result


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
        raise F1Error("F2 implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise F1Error("F2 implementation commit is unavailable")
    return commit
