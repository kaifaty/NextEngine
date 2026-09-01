#!/usr/bin/env python3
"""Frozen dependency and integration boundary for Physical Sound V19 I0."""

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
import physical_sound_v18_b0_common as b0_common
import physical_sound_v18_b0_model as b0_model
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

STUDY_ID = "physical-sound-v19-i0-frozen-hybrid-integration"
REVISION = "b0-c0-f0-frozen-integration-v0"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v19-i0.manifest.v0"
CORPUS_SCHEMA = "nextengine.experimental-physical-sound-v19-i0.corpus.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v19-i0.report.v0"
DECISION_SCHEMA = "nextengine.experimental-physical-sound-v19-i0.decision.v0"
FALLBACK_SCHEMA = "nextengine.experimental-physical-sound-v19-i0.fallback.v0"
ACCESS_SCHEMA = "nextengine.experimental-physical-sound-v19-i0.access-ledger.v0"
I0_EXECUTION_AUTHORIZED = False
I0_CLOSED_REASON = (
    "V19 I0 closed by development control: frozen phase-sensitive integration "
    "gates conflict with the accepted B0/F0 component tolerances"
)

PROTOCOL_PATH = Path(
    "docs/development/physical-sound-v19-p0b-field-integration-protocol-2026-09-01.md"
)
PROTOCOL_SHA256 = "1c2681a3ac4d505118d08e485cff9e38b23e72a897e8d149b21c909cd11ac24d"
F0_RESULT_PATH = Path(
    "docs/development/"
    "physical-sound-v19-f0-residual-harmonic-field-result-2026-09-01.md"
)
F0_RESULT_SHA256 = "fb4fe71522045dec69de905c04a893f4370ba6c1b701e33d8a159bbdd6ab3dc6"
B0_RESULT_PATH = Path(
    "docs/development/"
    "physical-sound-v18-b0-deterministic-global-baseline-result-2026-09-01.md"
)
B0_RESULT_SHA256 = "c275dc49664e91064f01a9123727a8f1cc5c61b8a2b47a95f1b942f56680ce75"
C0_RESULT_PATH = Path(
    "docs/development/physical-sound-v19-c0-composite-coverage-result-2026-09-01.md"
)
C0_RESULT_SHA256 = "fc3141a354d900d5cdda57ece99ce4b45ed9c9334db4fdbadc5ffb9aefb30d42"

B0_TREE_DIGEST = "bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111"
F0_TREE_DIGEST = "7918d8b4fd29f5aabb201b1dcc51d85fce9df44463ecb72ebd4eceb42d608718"
B0_MODEL_HASHES = {
    "coefficients.npy": "3dddb1b00b55055fb6c78ae34638df20c03428c8b2b620295b827a5f4805b7ea",
    "model.json": "cc5f2a41c5702e0a58b4df91c5a026b811807d0b5cd117ec807319d91d4e7a43",
    "normalization.npy": "44c1312b2805207422c967d15d01b7ed54c85462547d390647da8308d7022b01",
}
F0_MODEL_HASH = "855e81c38563aad47b4393b954b4bac2caa85c573683fe792d32fe9e62c2b00c"
F0_IMPLEMENTATION_HASHES = {
    "physical_sound_v19_f0_common.py": "84b742502f7be48742a5ee7d42d7073fd2f0061cb9fc43d8283bbbe7125942e7",
    "physical_sound_v19_f0_model.py": "ce48434d14a445949ad8e8d0516ac34306f3fd4d1bcbc623ba14daad27e884be",
    "physical_sound_v19_f0_oracle.py": "7a9107f292bfe33a3bf14de900adec667a95ac8c9d16fb9784441aeb81cf57f4",
}
B0_IMPLEMENTATION_HASHES = {
    "physical_sound_v18_b0_common.py": "8ad95bdc11b9053e6fe473cccff03c7ea2cee94ce5612371e2a6f72127bcd65f",
    "physical_sound_v18_b0_model.py": "abf148ec8725d7ce242b1a3a6457555b6a9e2e1896c5cf772054244a810e2aa2",
}

IMPLEMENTATION_FILES = (
    "physical_sound_v19_i0_common.py",
    "physical_sound_v19_i0_oracle.py",
)
TEST_FILE = Path("lab/tests/test_physical_sound_v19_i0_oracle.py")
REQUIRED_ENVIRONMENT = {
    "python": "3.12.13",
    "numpy": "2.5.2",
    "scipy": "1.18.0",
    "torch": "2.13.0+cu130",
}
EXPERIMENT_ROOT = f0_common.EXPERIMENT_ROOT
ZERO_ACCESS = {
    "admission_shadow_values_decoded": 0,
    "external_checkpoint_bytes_read": 0,
    "external_dataset_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "generator_real_values_decoded": 0,
    "integration_rows_generated_before_implementation_commit": 0,
    "method_holdout_values_decoded": 0,
    "network_requests": 0,
    "opened_v16_artifact_bytes_read": 0,
    "opened_v17_artifact_bytes_read": 0,
    "opened_v18_non_b0_artifact_bytes_read": 0,
    "protected_calibration_values_decoded": 0,
    "real_waveform_samples_decoded": 0,
    "source_body_bytes_read": 0,
}


class I0Error(RuntimeError):
    """Stable failure for the frozen I0 experiment."""


@dataclass(frozen=True)
class LoadedDependencies:
    b0: b0_model.RidgeArtifact
    f0_model: f0_model.PriorNetwork
    gain_scale: np.ndarray
    b0_root: Path
    f0_root: Path
    b0_bytes_read: int
    f0_bytes_read: int
    b0_file_map: dict[str, str]
    f0_file_map: dict[str, str]


@dataclass(frozen=True)
class GlobalModes:
    truth_frequencies: np.ndarray
    truth_damping: np.ndarray
    predicted_frequencies: np.ndarray
    predicted_damping: np.ndarray


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


sha256_bytes = f0_common.sha256_bytes
sha256_file = f0_common.sha256_file
canonical_json = f0_common.canonical_json
canonical_json_lines = f0_common.canonical_json_lines
identity_hash = f0_common.identity_hash
deterministic_npz = f0_common.deterministic_npz
directory_file_map = f0_common.directory_file_map
tree_digest = f0_common.tree_digest
compare_directories = f0_common.compare_directories
geometry_npz = f0_common.geometry_npz


def integration_rows() -> tuple[f0_common.FieldRow, ...]:
    return f0_common.generate_rows("integration")


def _integration_object(row: f0_common.FieldRow) -> f0_common.FieldObject:
    mesh = f0_common.geometry_common.build_mesh(row)
    analysis = f0_common.coverage_oracle.analyze_mesh(mesh)
    allowed = np.arange(mesh.vertex_count, dtype=np.int64)
    context = f0_common.coverage_oracle._restricted_fps(
        analysis.all_pairs,
        analysis.lexicographic_rank,
        allowed,
        f0_common._context_count(row, mesh.vertex_count),
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
    return f0_common.FieldObject(
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


def generate_integration_objects() -> tuple[f0_common.FieldObject, ...]:
    if not I0_EXECUTION_AUTHORIZED:
        raise I0Error(I0_CLOSED_REASON)
    objects = tuple(_integration_object(row) for row in integration_rows())
    if len(objects) != 24 or len({item.mesh.mesh_hash for item in objects}) != 24:
        raise I0Error("I0 integration object identity changed")
    return objects


def global_modes(
    artifact: b0_model.RidgeArtifact, row: f0_common.FieldRow
) -> GlobalModes:
    material_index = b0_common.MATERIAL_ORDER.index(row.material)
    topology_index = b0_common.TOPOLOGY_ORDER.index(row.topology)
    support_index = b0_common.SUPPORT_ORDER.index(row.support)
    frequency_scale, damping_scale, truth_frequency, truth_damping = b0_common._truth(
        material_index,
        topology_index,
        support_index,
        row.length_m,
        row.aspect,
        row.wall_m,
    )
    inputs = b0_model.BaselineInput(
        object_id=row.object_id,
        role="integration",
        stratum="interpolation",
        cell=row.cell,
        material=row.material,
        topology=row.topology,
        support=row.support,
        length_m=row.length_m,
        aspect=row.aspect,
        slenderness=row.slenderness,
        wall_m=row.wall_m,
        frequency_scale=frequency_scale,
        damping_scale=damping_scale,
    )
    predicted_frequency, predicted_damping = b0_model.predict(artifact, inputs)
    return GlobalModes(
        truth_frequency,
        truth_damping,
        predicted_frequency,
        predicted_damping,
    )


def _validate_artifact_root(
    root: Path, expected_tree: str, expected_files: set[str]
) -> dict[str, str]:
    if not root.is_absolute():
        raise I0Error("I0 dependency root must be absolute")
    experiment_root = EXPERIMENT_ROOT.resolve()
    resolved = root.resolve(strict=True)
    if not resolved.is_relative_to(experiment_root) or not resolved.is_dir():
        raise I0Error("I0 dependency root escaped the experiment boundary")
    if root.is_symlink():
        raise I0Error("I0 dependency root cannot be a symlink")
    file_map = directory_file_map(resolved)
    if set(file_map) != expected_files:
        raise I0Error(f"I0 dependency file set changed: {sorted(file_map)}")
    observed_tree = tree_digest(file_map)
    if observed_tree != expected_tree:
        raise I0Error(f"I0 dependency tree changed: {observed_tree}")
    return file_map


def load_dependencies(b0_root: Path, f0_root: Path) -> LoadedDependencies:
    b0_files = {
        "coefficients.npy",
        "corpus.json",
        "manifest.json",
        "model.json",
        "normalization.npy",
        "predictions.npy",
        "report.json",
    }
    f0_files = {
        "access-ledger.json",
        "corpus.json",
        "decisions.jsonl",
        "geometry.npz",
        "manifest.json",
        "model.npz",
        "predictions.npz",
        "report.json",
    }
    b0_map = _validate_artifact_root(b0_root, B0_TREE_DIGEST, b0_files)
    f0_map = _validate_artifact_root(f0_root, F0_TREE_DIGEST, f0_files)
    if any(b0_map[name] != digest for name, digest in B0_MODEL_HASHES.items()):
        raise I0Error("I0 B0 model member hash changed")
    if f0_map["model.npz"] != F0_MODEL_HASH:
        raise I0Error("I0 F0 model member hash changed")
    b0_payload = {name: (b0_root / name).read_bytes() for name in B0_MODEL_HASHES}
    f0_payload = (f0_root / "model.npz").read_bytes()
    try:
        b0_artifact = b0_model.decode_artifact(b0_payload)
        field_model, gain_scale = f0_model.decode_model(f0_payload)
    except (b0_common.B0Error, f0_common.F0Error) as error:
        raise I0Error(f"I0 dependency decode failed: {error}") from error
    b0_bytes = sum((b0_root / name).stat().st_size for name in b0_files)
    f0_bytes = sum((f0_root / name).stat().st_size for name in f0_files)
    return LoadedDependencies(
        b0_artifact,
        field_model,
        gain_scale,
        b0_root.resolve(),
        f0_root.resolve(),
        b0_bytes,
        f0_bytes,
        b0_map,
        f0_map,
    )


def verify_protocol_environment() -> dict[str, Any]:
    root = repository_root()
    guards = {
        PROTOCOL_PATH: PROTOCOL_SHA256,
        F0_RESULT_PATH: F0_RESULT_SHA256,
        B0_RESULT_PATH: B0_RESULT_SHA256,
        C0_RESULT_PATH: C0_RESULT_SHA256,
    }
    for name, digest in F0_IMPLEMENTATION_HASHES.items():
        guards[Path("lab/scripts") / name] = digest
    for name, digest in B0_IMPLEMENTATION_HASHES.items():
        guards[Path("lab/scripts") / name] = digest
    for path, expected in guards.items():
        observed = sha256_file(root / path)
        if observed != expected:
            raise I0Error(f"I0 dependency hash changed for {path}: {observed}")
    libraries = {
        "numpy": np.__version__,
        "python": platform.python_version(),
        "scipy": scipy.__version__,
        "torch": torch.__version__,
    }
    if libraries != REQUIRED_ENVIRONMENT:
        raise I0Error(f"I0 environment changed: {libraries}")
    if torch.get_num_threads() != 1 or not torch.are_deterministic_algorithms_enabled():
        raise I0Error("I0 torch determinism changed")
    return {"libraries": libraries}


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
        raise I0Error("I0 implementation/test files are not committed and clean")
    commit = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", *paths],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise I0Error("I0 implementation commit is unavailable")
    return commit


def prepare_output(output: Path) -> tuple[Path, Path]:
    try:
        return f0_common.prepare_output(output)
    except f0_common.F0Error as error:
        raise I0Error(str(error).replace("F0", "I0")) from error


def publish_output(staging: Path, output: Path) -> None:
    try:
        f0_common.publish_output(staging, output)
    except f0_common.F0Error as error:
        raise I0Error(str(error).replace("F0", "I0")) from error


def abandon_output(staging: Path) -> None:
    f0_common.abandon_output(staging)


def write_files(directory: Path, files: dict[str, bytes]) -> None:
    try:
        f0_common.write_files(directory, files)
    except f0_common.F0Error as error:
        raise I0Error(str(error).replace("F0", "I0")) from error


def json_object(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise I0Error(f"I0 invalid JSON dependency: {path.name}") from error
    if not isinstance(value, dict):
        raise I0Error(f"I0 JSON dependency is not an object: {path.name}")
    return value
