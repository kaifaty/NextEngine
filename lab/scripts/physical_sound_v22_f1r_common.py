#!/usr/bin/env python3
"""Fresh identities and guards for the V22 resource-bounded F1r study."""

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
import physical_sound_v21_f1_common as predecessor
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v22-f1r-resource-bounded-continuous-field"
REVISION = "continuous-residual-operator-v1-batched-execution-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v22-f1r.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v22-f1r.corpus.v0"
METRIC_SCHEMA = "nextengine.experimental-physical-sound-v22-f1r.metric.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v22-f1r.report.v0"
SELECTION_SCHEMA = "nextengine.experimental-physical-sound-v22-f1r.selection.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v22-f1r.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v22-p1r-resource-bounded-continuous-field-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "9352e678508550518c384f5b38decd766cb95b86707c6809d16c8e5055ded9e3"
RESULT_PATH = Path(
    "docs/development/physical-sound-v21-f1a-continuous-field-result-2026-09-01.md"
)
RESULT_SHA256 = "7a9695ede548968074c8355cb037dbbd8e0193ab42103320e83e87ac2d1a4318"
ROADMAP_PATH = Path("docs/plans/physical-sound-synthesis-roadmap-v22.md")
ROADMAP_SHA256 = "59d0cb21d5d559bf7fd94b72781089a182e739c25635d50ba19a5a979640a3a5"

PREDECESSOR_HASHES = {
    Path("lab/scripts/physical_sound_v21_f1_common.py"): (
        "0f1a0b428ae2420e98ae422eae68a1a5654a196793e15d0f0c9d8ddc7b1eb99a"
    ),
    Path("lab/scripts/physical_sound_v21_f1_model.py"): (
        "760d4856446aa372681a7a06988bf4cc82316a1c5b975f2c6ff00bbbb3c80190"
    ),
    Path("lab/scripts/physical_sound_v21_f1_tournament.py"): (
        "6ab9d8e31a3caa1b668a4db9a795d9bf3ed3d5aded6068f2c36c1678ba8a22b1"
    ),
    Path("lab/tests/test_physical_sound_v21_f1_tournament.py"): (
        "9fc1eb3f084a6f29d308e097065492d9997bb66d459d4ded50ac0ce1a4eeb242"
    ),
}

ROLE_BASE = {"train": 2201, "development": 2301, "test": 2401, "integration": 2501}
ROW_ROOTS = {
    "train": "4fd6d5b42777ffe5e7eed22c34045bae0319dcb4a895ccec4de5b4c475467e77",
    "development": "f5365e82c86e644e68ae60f5a2ad307c1b05d3e68ae95a8768ead55fe8425d9d",
    "test": "1f45f83b881387429a1fb640df28b6c1c90084d0239d2fa39fa50168f31dce72",
    "integration": "61dc577e44fd9557bd4de8b39210cba8dee92cfc824196ae123598987899ea62",
}
ROW_LEDGER_ROOT = "08098820b99388243f3d78940943747a1361c9a24e1e1e9ade68b56967137e09"

IMPLEMENTATION_FILES = (
    "physical_sound_v22_f1r_common.py",
    "physical_sound_v22_f1r_model.py",
    "physical_sound_v22_f1r_tournament.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v22_f1r_tournament.py")

ZERO_ACCESS = {
    **predecessor.ZERO_ACCESS,
    "v21_abandoned_staging_bytes_read": 0,
    "v21_f1_value_artifact_bytes_read": 0,
}

f0_common = predecessor.f0_common
f0_model = predecessor.f0_model
FieldRow = predecessor.FieldRow
FieldObject = predecessor.FieldObject
F1Error = predecessor.F1Error
REQUIRED_ENVIRONMENT = predecessor.REQUIRED_ENVIRONMENT
EXPERIMENT_ROOT = predecessor.EXPERIMENT_ROOT
MODE_COUNT = predecessor.MODE_COUNT
B0_TREE_DIGEST = predecessor.B0_TREE_DIGEST
C0_TREE_DIGEST = predecessor.C0_TREE_DIGEST
F0_TREE_DIGEST = predecessor.F0_TREE_DIGEST
M0B_TREE_DIGEST = predecessor.M0B_TREE_DIGEST

repository_root = predecessor.repository_root
sha256_bytes = predecessor.sha256_bytes
sha256_file = predecessor.sha256_file
canonical_json = predecessor.canonical_json
canonical_json_lines = predecessor.canonical_json_lines
identity_hash = predecessor.identity_hash
deterministic_npz = predecessor.deterministic_npz
geometry_npz = predecessor.geometry_npz
directory_file_map = predecessor.directory_file_map
tree_digest = predecessor.tree_digest
compare_directories = predecessor.compare_directories
load_f0_control = predecessor.load_f0_control
prepare_output = predecessor.prepare_output
publish_output = predecessor.publish_output
abandon_output = predecessor.abandon_output
write_files = predecessor.write_files


def _grid(role: str, cell: int, replicate: int) -> tuple[int, int]:
    if role == "train":
        return 15 + 2 * replicate + cell % 2, 13 + replicate + cell % 3
    base = {"development": (19, 16), "test": (21, 17), "integration": (23, 18)}
    if role not in base:
        raise F1Error(f"unknown F1r role: {role}")
    return base[role][0] + cell % 2, base[role][1] + cell % 3


def _support_index(role: str, cell: int, replicate: int) -> int:
    if role == "train":
        return (cell + replicate) % 2
    if role in ("development", "integration"):
        return (cell + 1) % 2
    if role == "test":
        return cell % 2
    raise F1Error(f"unknown F1r role: {role}")


def _primary_row(role: str, cell: int, replicate: int) -> FieldRow:
    if role not in ROLE_BASE or not 0 <= cell < 12:
        raise F1Error(f"invalid F1r row request: {role}/{cell}")
    if role == "train":
        if replicate not in (0, 1):
            raise F1Error("F1r train replicate is out of range")
        halton_index = ROLE_BASE[role] + 2 * cell + replicate
    else:
        if replicate != 0:
            raise F1Error("F1r non-train replicate must be zero")
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
    group = f"v22-f1r-{role}-{material}-{topology}-{support}-{halton_index}".lower()
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
        raise F1Error(f"unknown F1r role: {role}")
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
        raise F1Error(f"F1r {role} row root changed: {observed}")
    return tuple(rows)


def row_ledger() -> dict[str, dict[str, Any]]:
    result = {
        role: {"row_count": len(generate_rows(role)), "row_root": ROW_ROOTS[role]}
        for role in ("train", "development", "test", "integration")
    }
    if sha256_bytes(canonical_json(result)) != ROW_LEDGER_ROOT:
        raise F1Error("F1r row ledger root changed")
    return result


def generate_objects(role: str, implementation_commit: str) -> tuple[FieldObject, ...]:
    if role not in ("train", "development"):
        raise F1Error(f"F1r cannot generate sealed role: {role}")
    if implementation_commit != verify_committed_implementation():
        raise F1Error("F1r implementation commit identity changed")
    objects = tuple(predecessor._generate_object(row) for row in generate_rows(role))
    expected = 48 if role == "train" else 24
    if (
        len(objects) != expected
        or len({item.mesh.mesh_hash for item in objects}) != expected
    ):
        raise F1Error(f"F1r {role} object identity changed")
    return objects


def verify_protocol_environment() -> dict[str, Any]:
    predecessor.verify_protocol_environment()
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        RESULT_PATH: RESULT_SHA256,
        ROADMAP_PATH: ROADMAP_SHA256,
        **PREDECESSOR_HASHES,
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise F1Error(f"F1r dependency hash changed for {path}: {observed}")
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT:
        raise F1Error(f"F1r environment changed: {libraries}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise F1Error("F1r torch determinism changed")
    row_ledger()
    return {"libraries": libraries}


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    result = {}
    for name in IMPLEMENTATION_FILES:
        path = directory / name
        if not path.is_file():
            raise F1Error(f"missing F1r implementation: {name}")
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
        raise F1Error("F1r implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise F1Error("F1r implementation commit is unavailable")
    return commit
