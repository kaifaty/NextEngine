#!/usr/bin/env python3
"""Frozen lineage, metadata, and access boundary for Physical Sound V21 F1a."""

from __future__ import annotations

import json
import math
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
import physical_sound_v19_f0_common as f0_common
import physical_sound_v19_f0_model as f0_model
import scipy
import torch

torch.set_num_threads(1)
try:
    torch.set_num_interop_threads(1)
except RuntimeError:
    pass
torch.use_deterministic_algorithms(True)

STUDY_ID = "physical-sound-v21-f1a-continuous-residual-field"
REVISION = "continuous-residual-operator-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v21-f1a.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v21-f1a.corpus.v0"
METRIC_SCHEMA = "nextengine.experimental-physical-sound-v21-f1a.metric.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v21-f1a.report.v0"
SELECTION_SCHEMA = "nextengine.experimental-physical-sound-v21-f1a.selection.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v21-f1a.access-ledger.v0"

PROTOCOL_PATH = Path(
    "docs/development/"
    "physical-sound-v21-p1a-continuous-residual-field-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "18465f95b423bc619d2987106332b753de77a65091210b9625b52235277e4bae"
AUDIT_PATH = Path(
    "docs/development/physical-sound-v21-f0-protocol-conformance-audit-2026-09-01.md"
)
AUDIT_SHA256 = "78fa857cdc4ca643e451bc756c37cbf72f584a86bde615b321b25d722d35abb4"
P0D_PATH = Path(
    "docs/development/"
    "physical-sound-v21-p0d-counterfactual-owner-correction-protocol-2026-09-01.md"
)
P0D_SHA256 = "3513d3f0f9230af989be25f00541c9ac5fda997b46209e74349805d3f60d937a"
I1_RESULT_PATH = Path(
    "docs/development/physical-sound-v20-i1-frozen-integration-result-2026-09-01.md"
)
I1_RESULT_SHA256 = "f072dd206fc497240121800c05913d156f987860f6fe954ad8f0594ead1da404"

B0_TREE_DIGEST = "bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111"
C0_TREE_DIGEST = "ac0f3a2893ed40415cfb5cd84b736d335a74b37603c65307401fcbe81be49ace"
F0_TREE_DIGEST = "7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718"
M0B_TREE_DIGEST = "7eb5c75edcff8bd67fef592af30aa9fa90ebc73a25aa2629cba43935e23bab22"
F0_FILE_HASHES = {
    "access-ledger.json": "43e8104cc281f061680214d9a855af3ad4616905ea518ae98130775bf5288534",
    "corpus.json": "7075675bd066ada673ae4327aef483b80affc7847f08eb9ac80bb930e150ab11",
    "decisions.jsonl": "325f78669945073590d6fa2d38308153d0dc7c0777f16ddb941cf454d04d5938",
    "geometry.npz": "912f5261284b371c331121fcb848eab43aeeab499cd91cd209c2dced15451c2e",
    "manifest.json": "e2e94a4b815bfe926b23879ce49bae0744442bb1fd7df142435a322fdb965ee7",
    "model.npz": "855e81c38563aad47b4393b954b4bac2caa85c573683fe792d32fe9e62c2b00c",
    "predictions.npz": "7c236a9e8ddb066ae336244c742e983eb6a29a442ea77ea8bf2b362c7d662d35",
    "report.json": "1fa32ba99e0c212fc6efe2de23e10b18b6ee35b967ae27fb75939d2ab1486bad",
}

DEPENDENCY_HASHES = {
    Path("lab/scripts/physical_sound_v19_f0_common.py"): (
        "84b742502f7be48742a5ee7d42d7073fd2f0061cb9fc43d8283bbbe7125942e7"
    ),
    Path("lab/scripts/physical_sound_v19_f0_model.py"): (
        "ce48434d14a445949ad8e8d0516ac34306f3fd4d1bcbc623ba14daad27e884be"
    ),
    Path("lab/scripts/physical_sound_v19_f0_oracle.py"): (
        "7a9107f292bfe33a3bf14de900adec667a95ac8c9d16fb9784441aeb81cf57f4"
    ),
    Path("lab/scripts/physical_sound_v19_c0_common.py"): (
        "1224038462ba54e94986419f5db0510125474fbb9487d964066a6b3200548024"
    ),
    Path("lab/scripts/physical_sound_v19_c0_oracle.py"): (
        "be586995efd17efda30bf6631526a6315a2c04b65228d046d8218cb46b6c5e88"
    ),
    Path("lab/scripts/physical_sound_v18_o0_common.py"): (
        "baa201197a9790c7c827e36d043ccdbf3362b7458f207b4cd7a3d5f41cecccad"
    ),
    Path("lab/scripts/physical_sound_v20_m0b_common.py"): (
        "8f5ad986847f971e32e1c7e814cfb7c162143ad2a869824b6dc5cc5fb624fcce"
    ),
    Path("lab/scripts/physical_sound_v20_m0b_metric_lab.py"): (
        "b74208de60fa1889cd035b177919244b5a50e7d7416eaa1eaf7b347d232c4bcb"
    ),
}

ROLE_BASE = {"train": 1801, "development": 1901, "test": 2001, "integration": 2101}
ROW_ROOTS = {
    "train": "3d7be93c39b1f29be178a814e863f5894b93644a81d0f479c822abc3f7c3c6b2",
    "development": "9eecedcc541c32d394dc036f26be6e29f976b9d28e86de36ab9a51b0f458b5af",
    "test": "4537153ade7a4e9804142e2c7f431ebda22c786ba307aed65e4fdb84b62d990e",
    "integration": "a1bc5aa4bb185d069892fc6a0e24d2b82a890856bfd9a4e27d27a077dc7f793f",
}
ROW_LEDGER_ROOT = "9324c04d9abb538bbe2e102323ce576f9c109ea31f38af4ad69296a5981ba433"
REQUIRED_ENVIRONMENT = f0_common.REQUIRED_ENVIRONMENT
EXPERIMENT_ROOT = f0_common.EXPERIMENT_ROOT
MODE_COUNT = f0_common.MODE_COUNT

IMPLEMENTATION_FILES = (
    "physical_sound_v21_f1_common.py",
    "physical_sound_v21_f1_model.py",
    "physical_sound_v21_f1_tournament.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v21_f1_tournament.py")

ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "i1_artifact_bytes_read": 0,
    "integration_mesh_views_generated": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "opened_v16_artifact_bytes_read": 0,
    "opened_v17_artifact_bytes_read": 0,
    "opened_v18_non_b0_artifact_bytes_read": 0,
    "opened_v19_non_f0_artifact_bytes_read": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
    "test_mesh_views_generated": 0,
}

FieldRow = f0_common.FieldRow
FieldObject = f0_common.FieldObject


class F1Error(RuntimeError):
    """Stable fail-closed error for the frozen V21 F1a experiment."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


sha256_bytes = f0_common.sha256_bytes
sha256_file = f0_common.sha256_file
canonical_json = f0_common.canonical_json
canonical_json_lines = f0_common.canonical_json_lines
identity_hash = f0_common.identity_hash
deterministic_npz = f0_common.deterministic_npz
geometry_npz = f0_common.geometry_npz
directory_file_map = f0_common.directory_file_map
tree_digest = f0_common.tree_digest
compare_directories = f0_common.compare_directories


def _grid(role: str, cell: int, replicate: int) -> tuple[int, int]:
    if role == "train":
        return 15 + 2 * replicate + cell % 2, 13 + replicate + cell % 3
    base = {"development": (19, 16), "test": (21, 17), "integration": (23, 18)}
    if role not in base:
        raise F1Error(f"unknown F1 role: {role}")
    return base[role][0] + cell % 2, base[role][1] + cell % 3


def _support_index(role: str, cell: int, replicate: int) -> int:
    if role == "train":
        return (cell + replicate) % 2
    if role in ("development", "integration"):
        return (cell + 1) % 2
    if role == "test":
        return cell % 2
    raise F1Error(f"unknown F1 role: {role}")


def _primary_row(role: str, cell: int, replicate: int) -> FieldRow:
    if role not in ROLE_BASE or not 0 <= cell < 12:
        raise F1Error(f"invalid F1 row request: {role}/{cell}")
    if role == "train":
        if replicate not in (0, 1):
            raise F1Error("F1 train replicate is out of range")
        halton_index = ROLE_BASE[role] + 2 * cell + replicate
    else:
        if replicate != 0:
            raise F1Error("F1 non-train replicate must be zero")
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
    group = f"v21-f1-{role}-{material}-{topology}-{support}-{halton_index}".lower()
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
        raise F1Error(f"unknown F1 role: {role}")
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
        raise F1Error(f"F1 {role} row root changed: {observed}")
    return tuple(rows)


def row_ledger() -> dict[str, dict[str, Any]]:
    result = {
        role: {"row_count": len(generate_rows(role)), "row_root": ROW_ROOTS[role]}
        for role in ("train", "development", "test", "integration")
    }
    if sha256_bytes(canonical_json(result)) != ROW_LEDGER_ROOT:
        raise F1Error("F1 row ledger root changed")
    return result


def _context_count(row: FieldRow, vertex_count: int) -> int:
    divisor = 3 if row.role == "train" else 8
    result = max(16, math.ceil(vertex_count / divisor))
    if row.object_id.endswith("-twin"):
        result += 2
    return result


def _generate_object(row: FieldRow) -> FieldObject:
    mesh = f0_common.geometry_common.build_mesh(row)
    analysis = f0_common.coverage_oracle.analyze_mesh(mesh)
    allowed = np.arange(mesh.vertex_count, dtype=np.int64)
    context = f0_common.coverage_oracle._restricted_fps(
        analysis.all_pairs,
        analysis.lexicographic_rank,
        allowed,
        _context_count(row, mesh.vertex_count),
    )
    selected = np.zeros(mesh.vertex_count, dtype=bool)
    selected[context] = True
    query = np.flatnonzero(~selected).astype(np.int64)
    coverage_input = f0_common.coverage_oracle._coverage_input(
        analysis, "valid", "valid", context, query, analysis.graph
    )
    decisions = tuple(
        f0_common.coverage_oracle.evaluate(analysis, coverage_input, "composite")
    )
    accepted = np.asarray(
        [item["query_vertex"] for item in decisions if item["reason"] == "ACCEPT"],
        dtype=np.int64,
    )
    rejected = np.asarray(
        [item["query_vertex"] for item in decisions if item["reason"] != "ACCEPT"],
        dtype=np.int64,
    )
    normals, curvatures = f0_common._normal_and_curvature(mesh)
    gains = f0_common.gain_truth(mesh, normals)
    result = FieldObject(
        row=row,
        mesh=mesh,
        analysis=analysis,
        context=context,
        query=query,
        accepted_query=accepted,
        rejected_query=rejected,
        coverage_input=coverage_input,
        coverage_decisions=decisions,
        normals=normals,
        curvatures=curvatures,
        gains=gains,
    )
    if row.role == "train" and rejected.size:
        raise F1Error(f"F1 train query rejected: {row.object_id}")
    return result


def generate_objects(role: str, implementation_commit: str) -> tuple[FieldObject, ...]:
    if role not in ("train", "development"):
        raise F1Error(f"F1a cannot generate sealed role: {role}")
    if implementation_commit != verify_committed_implementation():
        raise F1Error("F1 implementation commit identity changed")
    objects = tuple(_generate_object(row) for row in generate_rows(role))
    expected = 48 if role == "train" else 24
    if (
        len(objects) != expected
        or len({item.mesh.mesh_hash for item in objects}) != expected
    ):
        raise F1Error(f"F1 {role} object identity changed")
    return objects


def verify_protocol_environment() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        AUDIT_PATH: AUDIT_SHA256,
        P0D_PATH: P0D_SHA256,
        I1_RESULT_PATH: I1_RESULT_SHA256,
        **DEPENDENCY_HASHES,
    }
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise F1Error(f"F1 dependency hash changed for {path}: {observed}")
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT:
        raise F1Error(f"F1 environment changed: {libraries}")
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
        raise F1Error(f"F1 thread environment changed: {threads}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise F1Error("F1 torch determinism changed")
    row_ledger()
    return {"libraries": libraries, "threads": threads}


def implementation_hashes() -> dict[str, str]:
    directory = Path(__file__).resolve().parent
    result = {}
    for name in IMPLEMENTATION_FILES:
        path = directory / name
        if not path.is_file():
            raise F1Error(f"missing F1 implementation: {name}")
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
        raise F1Error("F1 implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise F1Error("F1 implementation commit is unavailable")
    return commit


def load_f0_control(root: Path) -> tuple[f0_model.PriorNetwork, np.ndarray, int]:
    if not root.is_absolute():
        raise F1Error("F1 F0 root must be absolute")
    resolved = root.resolve(strict=True)
    experiment = EXPERIMENT_ROOT.resolve()
    if (
        root.is_symlink()
        or not resolved.is_dir()
        or not resolved.is_relative_to(experiment)
    ):
        raise F1Error("F1 F0 root escaped experiment boundary")
    file_map = directory_file_map(resolved)
    if file_map != F0_FILE_HASHES or tree_digest(file_map) != F0_TREE_DIGEST:
        raise F1Error("F1 F0 control identity changed")
    try:
        manifest = json.loads((resolved / "manifest.json").read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise F1Error("F1 F0 manifest is invalid") from error
    if (
        manifest.get("implementation_commit")
        != "b815cddb1f1ac89f623f63b012c73a6f24df7866"
    ):
        raise F1Error("F1 F0 implementation identity changed")
    payload = (resolved / "model.npz").read_bytes()
    trained, scale = f0_model.decode_model(payload)
    return trained, scale, sum((resolved / name).stat().st_size for name in file_map)


def prepare_output(output: Path) -> tuple[Path, Path]:
    try:
        return f0_common.prepare_output(output)
    except f0_common.F0Error as error:
        raise F1Error(str(error).replace("F0", "F1")) from error


def publish_output(staging: Path, output: Path) -> None:
    try:
        f0_common.publish_output(staging, output)
    except f0_common.F0Error as error:
        raise F1Error(str(error).replace("F0", "F1")) from error


def abandon_output(staging: Path) -> None:
    f0_common.abandon_output(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    try:
        f0_common.write_files(directory, files)
    except f0_common.F0Error as error:
        raise F1Error(str(error).replace("F0", "F1")) from error
