"""Deterministic data, access and artifact boundary for V25 M0a."""

from __future__ import annotations

import hashlib
import json
import math
import os
import shutil
import struct
import subprocess
import tempfile
from collections.abc import Iterable, Mapping
from dataclasses import dataclass
from enum import IntEnum
from pathlib import Path
from typing import Any

import numpy as np
from scipy import signal

PROTOCOL_PATH = "docs/development/physical-sound-v25-m0a-causal-material-neural-student-protocol-2026-09-02.md"
PROTOCOL_SHA256 = "2311f2410b44c89617dbaf4a6f4b9f7d30fd0a183f5fd262853c85f5be956e68"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.manifest.v1"
COMBINED_SCHEMA = "nextengine.experimental-physical-sound-neural-data-plane.manifest.v3"
TEACHER_SCHEMA = "nextengine.experimental-physical-sound-v24-t0-teacher.evidence.v1"
X0_SCHEMA = "nextengine.experimental-physical-sound-v24-x0.lineage.v1"
T0_LINEAGE_ID = "v24-t0-analytic-teacher"
X0_LINEAGE_ID = "v24-x0-blue-bowl-real"
T0_CLAIM = "SYNTHETIC_ANALYTIC_MODAL_TEACHER_ONLY / NO_REAL_MATERIAL_QUALITY_OR_RUNTIME_AUTHORITY"
X0_CLAIM = "DISCLOSED_BLUE_BOWL_REAL_EVIDENCE_ONLY / NO_MISSING_AXIS_MODEL_ADMISSION_OR_RUNTIME_AUTHORITY"
PREPROCESS_SCHEMA = "nextengine.experimental-physical-sound-v25-m0a.preprocess.v1"
TENSOR_MAGIC = b"NEM0ATN1"
MESH_MAGIC = b"NEMESH01"
MODAL_MAGIC = b"NEMODT01"
GAIN_MAGIC = b"NEGAIN01"
MODEL_ID = "m0a-contact-modal-field-v1"
OFFICIAL_COMBINED_SHA256 = "c43ba8eac68e32a8ef8fbe37d3ffa3d21db1d59e0443766cee1d98f51cad70bb"
OFFICIAL_T0_EVIDENCE_SHA256 = "884da56ff9005e9dd63e11ec74bafa8796b187b15ed6099fbca99dda7617e7ed"
OFFICIAL_X0_LINEAGE_SHA256 = "e4f6bb117a09f77b2bf8602ec895c95ae63262bc9d42db1233d972d73242534f"
OFFICIAL_T0_ARTIFACT_ROOT_SHA256 = "b8c4d82f038b7b592584caf292087b76ed08219ebf2eada12a0285d6796a908c"
ROLES = ("train", "development", "calibration", "method_holdout", "admission_shadow")
MATERIAL_IDS = ("elastic-a", "elastic-b", "elastic-c", "glass")
SUPPORT_IDS = ("cantilever-clamped-u0", "simply-supported-all-edges")
MATERIAL_PHYSICS = {
    "elastic-a": (69_000_000_000.0, 2_700.0, 0.33, 8.0, 0.0015),
    "elastic-b": (200_000_000_000.0, 7_850.0, 0.29, 12.0, 0.0010),
    "elastic-c": (110_000_000_000.0, 8_900.0, 0.34, 18.0, 0.0008),
}
T0_IMPLEMENTATION_SHA256 = "ef5458df64272714f8aaff38b004bd1320bf2a6fa2c58b587505146b18b73617"
MAX_JSON_BYTES = 16 * 1024 * 1024
MAX_ARTIFACT_BYTES = 256 * 1024 * 1024


class M0Error(RuntimeError):
    """Stable M0 rejection boundary."""


@dataclass(frozen=True)
class ExecutionProfile:
    profile_id: str
    synthetic_steps: int
    real_steps: int
    batch_size: int
    transfer_samples: int
    transfer_window: int
    transfer_anchor: int
    mode_count: int
    require_exact_mode_count: bool


@dataclass(frozen=True)
class Mesh:
    vertices: np.ndarray
    triangles: np.ndarray
    sha256: str


@dataclass(frozen=True)
class Modes:
    frequencies_hz: np.ndarray
    decay_per_second: np.ndarray
    families: np.ndarray


@dataclass(frozen=True)
class GainField:
    mesh_sha256: str
    values: np.ndarray


@dataclass(frozen=True)
class SyntheticExample:
    row_id: str
    split_role: str
    sample_role: str
    object_id: str
    object_features: np.ndarray
    contact_features: np.ndarray
    frequencies_hz: np.ndarray
    decay_per_second: np.ndarray
    gains: np.ndarray
    mode_mask: np.ndarray


@dataclass(frozen=True)
class RealTransferExample:
    row_id: str
    sample_role: str
    object_features: np.ndarray
    contact_features: np.ndarray
    samples: np.ndarray


@dataclass(frozen=True)
class IdentifiedRecording:
    row_id: str
    sample_role: str
    samples: np.ndarray


class Phase(IntEnum):
    PREPROCESS = 0
    TRAINING = 1
    TRAINED = 2
    CANDIDATE_FROZEN = 3
    METHOD_HOLDOUT_OPENED = 4
    REAL_QUERY_OPENED = 5


def execution_profile(profile_id: str) -> ExecutionProfile:
    if profile_id == "official-v1":
        return ExecutionProfile(profile_id, 1_500, 500, 16, 230_215, 144_000, 512, 10, True)
    if profile_id == "contract-fixture-v1":
        return ExecutionProfile(profile_id, 4, 2, 2, 8_192, 4_096, 128, 10, False)
    raise M0Error(f"unsupported M0 execution profile: {profile_id}")


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode("utf-8")


def _no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise M0Error(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_json_bytes(data: bytes, role: str, maximum: int = MAX_JSON_BYTES) -> dict[str, Any]:
    if not 0 < len(data) <= maximum:
        raise M0Error(f"{role} must be 1..={maximum} bytes")
    try:
        value = json.loads(data, object_pairs_hook=_no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise M0Error(f"invalid {role} JSON: {error}") from error
    if not isinstance(value, dict):
        raise M0Error(f"{role} must be a JSON object")
    return value


def _inside_repository(path: Path) -> bool:
    return path.is_relative_to(repository_root().resolve(strict=True))


def external_file(path: Path, role: str, maximum: int = MAX_ARTIFACT_BYTES) -> Path:
    resolved = path.resolve(strict=True)
    if _inside_repository(resolved) or not resolved.is_file() or resolved.is_symlink():
        raise M0Error(f"{role} must be an external regular file: {resolved}")
    size = resolved.stat().st_size
    if not 0 < size <= maximum:
        raise M0Error(f"{role} size must be 1..={maximum}, got {size}")
    return resolved


def external_directory(path: Path, role: str, *, must_exist: bool = True) -> Path:
    if must_exist:
        resolved = path.resolve(strict=True)
        if _inside_repository(resolved) or not resolved.is_dir() or resolved.is_symlink():
            raise M0Error(f"{role} must be an external directory: {resolved}")
        return resolved
    parent = path.parent.resolve(strict=True)
    resolved = parent / path.name
    if _inside_repository(resolved) or resolved.exists():
        raise M0Error(f"{role} must be a new external path: {resolved}")
    return resolved


def validate_ref(reference: Any, role: str, maximum: int = MAX_ARTIFACT_BYTES) -> tuple[Path, bytes]:
    if not isinstance(reference, dict) or set(reference) != {"path", "sha256"}:
        raise M0Error(f"{role} reference fields changed")
    path_value = reference["path"]
    expected = reference["sha256"]
    if (
        not isinstance(path_value, str)
        or not path_value
        or not isinstance(expected, str)
        or len(expected) != 64
        or any(character not in "0123456789abcdef" for character in expected)
    ):
        raise M0Error(f"{role} reference is invalid")
    path = external_file(Path(path_value), role, maximum)
    data = path.read_bytes()
    observed = sha256_bytes(data)
    if observed != expected:
        raise M0Error(f"{role} hash mismatch: expected {expected}, got {observed}")
    return path, data


def implementation_hashes() -> dict[str, str]:
    scripts = Path(__file__).resolve().parent
    names = (
        "physical_sound_v25_m0a_common.py",
        "physical_sound_v25_m0a_evaluate.py",
        "physical_sound_v25_m0a_model.py",
        "physical_sound_v25_m0a_train.py",
    )
    result = {}
    for name in names:
        path = scripts / name
        if not path.is_file():
            raise M0Error(f"M0 implementation file is missing: {name}")
        result[name] = sha256_file(path)
    return result


def implementation_root_sha256() -> str:
    return sha256_bytes(canonical_json(implementation_hashes()))


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes]:
    path = external_file(path, "M0 manifest", 128 * 1024)
    data = path.read_bytes()
    manifest = parse_json_bytes(data, "M0 manifest", 128 * 1024)
    expected = {
        "schema",
        "profile",
        "protocol_sha256",
        "implementation_root_sha256",
        "artifacts",
        "t0_root",
        "x0_root",
        "mlflow_root",
        "data_policy",
    }
    if set(manifest) != expected or manifest.get("schema") != MANIFEST_SCHEMA:
        raise M0Error("M0 manifest fields or schema changed")
    profile = execution_profile(manifest.get("profile"))
    if manifest.get("protocol_sha256") != PROTOCOL_SHA256:
        raise M0Error("M0 protocol hash changed")
    if manifest.get("implementation_root_sha256") != implementation_root_sha256():
        raise M0Error("M0 implementation hash changed")
    policy = manifest.get("data_policy")
    if policy != {
        "external_output_only": True,
        "cpu_only": True,
        "network_allowed": False,
        "admission_shadow_access_allowed": False,
        "runtime_authorized": False,
        "mlflow_remote_allowed": False,
    }:
        raise M0Error("M0 data policy changed")
    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, dict) or set(artifacts) != {
        "combined_manifest",
        "teacher_evidence",
        "x0_lineage",
        "ffmpeg",
        "ffprobe",
    }:
        raise M0Error("M0 artifact set changed")
    external_directory(Path(manifest["t0_root"]), "M0 T0 root")
    external_directory(Path(manifest["x0_root"]), "M0 X0 root")
    mlflow_root = Path(manifest["mlflow_root"])
    if mlflow_root.exists():
        external_directory(mlflow_root, "M0 MLflow root")
    else:
        external_directory(mlflow_root, "M0 MLflow root", must_exist=False)
    for name, maximum in (
        ("combined_manifest", MAX_JSON_BYTES),
        ("teacher_evidence", MAX_JSON_BYTES),
        ("x0_lineage", MAX_JSON_BYTES),
        ("ffmpeg", MAX_ARTIFACT_BYTES),
        ("ffprobe", MAX_ARTIFACT_BYTES),
    ):
        validate_ref(artifacts[name], f"M0 {name}", maximum)
    if profile.profile_id == "official-v1":
        fixed = {
            "combined_manifest": OFFICIAL_COMBINED_SHA256,
            "teacher_evidence": OFFICIAL_T0_EVIDENCE_SHA256,
            "x0_lineage": OFFICIAL_X0_LINEAGE_SHA256,
        }
        for name, expected_hash in fixed.items():
            if artifacts[name]["sha256"] != expected_hash:
                raise M0Error(f"official M0 {name} hash changed")
    return manifest, data


def load_combined(manifest: dict[str, Any]) -> tuple[dict[str, Any], bytes]:
    _, data = validate_ref(
        manifest["artifacts"]["combined_manifest"],
        "combined V3 manifest",
        MAX_JSON_BYTES,
    )
    value = parse_json_bytes(data, "combined V3 manifest")
    if (
        value.get("schema") != COMBINED_SCHEMA
        or value.get("projection_id") != "v24-blue-bowl-real-pilot"
        or value.get("revision") != "v1"
        or not isinstance(value.get("rows"), list)
        or not isinstance(value.get("lineage_reports"), list)
    ):
        raise M0Error("combined V3 manifest identity changed")
    row_ids = [row.get("row_id") for row in value["rows"]]
    if row_ids != sorted(row_ids) or len(row_ids) != len(set(row_ids)):
        raise M0Error("combined V3 rows are not unique and sorted")
    for row in value["rows"]:
        exposed_identity = " ".join(
            str(row.get(field, "")) for field in ("row_id", "recording_parent_id", "condition_group_id")
        )
        exposed_identity += " " + str(row.get("audio", {}).get("path", ""))
        if "2407" in exposed_identity:
            raise M0Error("sealed REALIMPACT row 2407 materialized")
    return value, data


def validate_combined_lineage_bindings(combined: dict[str, Any], manifest: dict[str, Any]) -> None:
    expected = {
        T0_LINEAGE_ID: {
            "id": T0_LINEAGE_ID,
            "expected_schema": TEACHER_SCHEMA,
            "expected_claim": T0_CLAIM,
            "artifact": manifest["artifacts"]["teacher_evidence"],
        },
        X0_LINEAGE_ID: {
            "id": X0_LINEAGE_ID,
            "expected_schema": X0_SCHEMA,
            "expected_claim": X0_CLAIM,
            "artifact": manifest["artifacts"]["x0_lineage"],
        },
    }
    actual = {item.get("id"): item for item in combined["lineage_reports"] if isinstance(item, dict)}
    if actual != expected:
        raise M0Error("combined V3 lineage bindings do not match the M0a manifest")


def parse_mesh(data: bytes, expected_sha256: str | None = None) -> Mesh:
    if len(data) < 20 or data[:8] != MESH_MAGIC:
        raise M0Error("M0 mesh header is invalid")
    version, vertex_count, triangle_count = struct.unpack_from("<III", data, 8)
    expected_length = 20 + vertex_count * 3 * 8 + triangle_count * 3 * 4
    if version != 1 or vertex_count < 8 or triangle_count == 0 or len(data) != expected_length:
        raise M0Error("M0 mesh shape or version is invalid")
    vertices_end = 20 + vertex_count * 24
    vertices = np.frombuffer(data, dtype="<f8", count=vertex_count * 3, offset=20).reshape(vertex_count, 3).copy()
    triangles = (
        np.frombuffer(data, dtype="<u4", count=triangle_count * 3, offset=vertices_end)
        .reshape(triangle_count, 3)
        .copy()
    )
    observed = sha256_bytes(data)
    if expected_sha256 is not None and observed != expected_sha256:
        raise M0Error("M0 mesh hash mismatch")
    if not np.all(np.isfinite(vertices)) or np.any(triangles >= vertex_count):
        raise M0Error("M0 mesh contains invalid values or indices")
    if float(np.max(np.ptp(vertices, axis=0))) <= 0.0:
        raise M0Error("M0 mesh is degenerate")
    return Mesh(vertices, triangles, observed)


def parse_modes(data: bytes, profile: ExecutionProfile) -> Modes:
    if len(data) < 16 or data[:8] != MODAL_MAGIC:
        raise M0Error("M0 modal header is invalid")
    version, count = struct.unpack_from("<II", data, 8)
    if version != 1 or not 1 <= count <= profile.mode_count or len(data) != 16 + count * 28:
        raise M0Error("M0 modal count, length or version is invalid")
    if profile.require_exact_mode_count and count != profile.mode_count:
        raise M0Error("official M0 target must contain exactly ten modes")
    frequencies, decay, families = [], [], []
    for index in range(count):
        ordinal, family_a, family_b, frequency, rate = struct.unpack_from("<IIIdd", data, 16 + index * 28)
        if ordinal != index:
            raise M0Error("M0 modal ordinals are not canonical")
        frequencies.append(frequency)
        decay.append(rate)
        families.append((family_a, family_b))
    frequency_array = np.asarray(frequencies, dtype=np.float64)
    decay_array = np.asarray(decay, dtype=np.float64)
    if (
        not np.all(np.isfinite(frequency_array))
        or not np.all(np.isfinite(decay_array))
        or np.any(frequency_array <= 0.0)
        or np.any(decay_array <= 0.0)
        or np.any(np.diff(frequency_array) <= 0.0)
    ):
        raise M0Error("M0 modal values are non-finite, non-positive or unsorted")
    return Modes(frequency_array, decay_array, np.asarray(families, dtype=np.uint32))


def parse_gain_field(data: bytes, mesh: Mesh, modes: Modes) -> GainField:
    if len(data) < 52 or data[:8] != GAIN_MAGIC:
        raise M0Error("M0 gain header is invalid")
    version, vertex_count, mode_count = struct.unpack_from("<III", data, 8)
    expected_length = 52 + vertex_count * mode_count * 8
    if (
        version != 1
        or vertex_count != len(mesh.vertices)
        or mode_count != len(modes.frequencies_hz)
        or len(data) != expected_length
        or data[20:52] != bytes.fromhex(mesh.sha256)
    ):
        raise M0Error("M0 gain shape or mesh binding changed")
    values = np.frombuffer(data, dtype="<f8", offset=52).reshape(vertex_count, mode_count).copy()
    if not np.all(np.isfinite(values)):
        raise M0Error("M0 gain field is non-finite")
    return GainField(mesh.sha256, values)


def _mesh_closed(triangles: np.ndarray) -> bool:
    edges: dict[tuple[int, int], int] = {}
    for triangle in triangles:
        a, b, c = (int(value) for value in triangle)
        for left, right in ((a, b), (b, c), (c, a)):
            edge = (min(left, right), max(left, right))
            edges[edge] = edges.get(edge, 0) + 1
    return bool(edges) and all(count == 2 for count in edges.values())


def object_descriptor(mesh: Mesh) -> np.ndarray:
    vertices = mesh.vertices
    centred = vertices - np.mean(vertices, axis=0)
    lengths = np.ptp(vertices, axis=0)
    diagonal = float(np.linalg.norm(lengths))
    if not math.isfinite(diagonal) or diagonal <= 0.0:
        raise M0Error("M0 mesh has zero bounding diagonal")
    covariance = (centred.T @ centred) / float(len(vertices))
    eigenvalues = np.maximum(np.linalg.eigvalsh(covariance), 0.0)
    p0 = vertices[mesh.triangles[:, 0]]
    p1 = vertices[mesh.triangles[:, 1]]
    p2 = vertices[mesh.triangles[:, 2]]
    surface_area = 0.5 * float(np.sum(np.linalg.norm(np.cross(p1 - p0, p2 - p0), axis=1)))
    closed = _mesh_closed(mesh.triangles)
    signed_volume = float(np.sum(np.einsum("ij,ij->i", p0, np.cross(p1, p2)))) / 6.0 if closed else 0.0
    radii = np.linalg.norm(centred, axis=1) / diagonal
    quantiles = np.quantile(radii, np.linspace(0.0, 1.0, 8), method="linear")
    values = np.asarray(
        [
            math.log1p(len(vertices)),
            math.log1p(len(mesh.triangles)),
            *(lengths / diagonal),
            diagonal,
            *(np.mean(vertices, axis=0) / diagonal),
            *(eigenvalues / (diagonal * diagonal)),
            math.log1p(surface_area),
            math.log1p(abs(signed_volume)),
            1.0 if closed else 0.0,
            float(np.mean(radii)),
            *quantiles,
        ],
        dtype=np.float32,
    )
    if values.shape != (24,) or not np.all(np.isfinite(values)):
        raise M0Error(f"M0 object descriptor shape or values changed: {values.shape}")
    return values


def farthest_landmarks(mesh: Mesh, count: int = 8) -> np.ndarray:
    vertices = mesh.vertices
    if len(vertices) < count:
        raise M0Error("M0 mesh has fewer than eight landmark candidates")
    centroid = np.mean(vertices, axis=0)
    distance = np.sum((vertices - centroid) ** 2, axis=1)
    selected = [int(np.argmax(distance))]
    minimum = np.sum((vertices - vertices[selected[0]]) ** 2, axis=1)
    while len(selected) < count:
        maximum = float(np.max(minimum))
        candidates = np.flatnonzero(minimum == maximum)
        next_index = int(candidates[0])
        selected.append(next_index)
        minimum = np.minimum(minimum, np.sum((vertices - vertices[next_index]) ** 2, axis=1))
    return vertices[np.asarray(selected)]


def contact_descriptor(mesh: Mesh, point_metres: Iterable[float]) -> np.ndarray:
    point = np.asarray(tuple(point_metres), dtype=np.float64)
    if point.shape != (3,) or not np.all(np.isfinite(point)):
        raise M0Error("M0 contact point is invalid")
    minimum = np.min(mesh.vertices, axis=0)
    lengths = np.ptp(mesh.vertices, axis=0)
    diagonal = float(np.linalg.norm(lengths))
    centre = np.mean(mesh.vertices, axis=0)
    xyz = (point - centre) / diagonal
    radial = float(np.linalg.norm(xyz))
    landmark_distances = np.linalg.norm(farthest_landmarks(mesh) - point, axis=1) / diagonal
    unit = np.divide(point - minimum, lengths, out=np.zeros(3), where=lengths > 0.0)
    sinusoidal = []
    for coordinate in unit:
        sinusoidal.extend(
            (
                math.sin(math.pi * coordinate),
                math.cos(math.pi * coordinate),
                math.sin(2.0 * math.pi * coordinate),
                math.cos(2.0 * math.pi * coordinate),
            )
        )
    result = np.asarray([*xyz, radial, *landmark_distances, *sinusoidal], dtype=np.float32)
    if result.shape != (24,) or not np.all(np.isfinite(result)):
        raise M0Error("M0 contact descriptor shape or values changed")
    return result


def material_support_features(
    material: str | None,
    support: str | None,
    *,
    physical_override: tuple[float, float, float, float, float] | None = None,
) -> np.ndarray:
    material_known = material in MATERIAL_IDS
    support_known = support in SUPPORT_IDS
    values = [1.0 if material_known and material == item else 0.0 for item in MATERIAL_IDS]
    values.append(1.0 if material_known else 0.0)
    physical = physical_override if physical_override is not None else MATERIAL_PHYSICS.get(material or "")
    if physical is None:
        values.extend((0.0, 0.0, 0.0, 0.0, 0.0, 0.0))
    else:
        youngs, density, poisson, damping_base, damping_slope = physical
        if (
            not all(math.isfinite(item) for item in physical)
            or youngs <= 0.0
            or density <= 0.0
            or not 0.0 < poisson < 0.5
            or damping_base <= 0.0
            or damping_slope < 0.0
        ):
            raise M0Error("M0a physical material vector is invalid")
        values.extend(
            (
                math.log(youngs / 1.0e9),
                math.log(density / 1_000.0),
                poisson,
                math.log(damping_base),
                1_000.0 * damping_slope,
                1.0,
            )
        )
    values.extend(1.0 if support_known and support == item else 0.0 for item in SUPPORT_IDS)
    values.append(1.0 if support_known else 0.0)
    result = np.asarray(values, dtype=np.float32)
    if result.shape != (14,) or not np.all(np.isfinite(result)):
        raise M0Error("M0a material/support feature shape changed")
    return result


def full_object_features(
    mesh: Mesh,
    material: str | None,
    support: str | None,
    *,
    physical_override: tuple[float, float, float, float, float] | None = None,
) -> np.ndarray:
    result = np.concatenate(
        (
            object_descriptor(mesh),
            material_support_features(material, support, physical_override=physical_override),
        )
    )
    if result.shape != (38,) or not np.all(np.isfinite(result)):
        raise M0Error("M0a full object feature shape changed")
    return result


def nearest_vertex(mesh: Mesh, point: Iterable[float]) -> int:
    target = np.asarray(tuple(point), dtype=np.float64)
    distances = np.sum((mesh.vertices - target) ** 2, axis=1)
    index = int(np.argmin(distances))
    tolerance = max(1.0e-12, float(np.linalg.norm(np.ptp(mesh.vertices, axis=0))) * 1.0e-9)
    if math.sqrt(float(distances[index])) > tolerance:
        raise M0Error("M0 contact does not bind an exact mesh vertex")
    return index


def decode_float_wav(data: bytes) -> tuple[int, np.ndarray]:
    if len(data) < 44 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise M0Error("M0 WAV container is invalid")
    offset = 12
    fmt: tuple[int, int, int, int] | None = None
    payload: bytes | None = None
    while offset + 8 <= len(data):
        chunk_id = data[offset : offset + 4]
        size = struct.unpack_from("<I", data, offset + 4)[0]
        start = offset + 8
        end = start + size
        if end > len(data):
            raise M0Error("M0 WAV chunk exceeds the container")
        if chunk_id == b"fmt " and size >= 16:
            audio_format, channels, rate, _, _, bits = struct.unpack_from("<HHIIHH", data, start)
            fmt = (audio_format, channels, rate, bits)
        elif chunk_id == b"data":
            payload = data[start:end]
        offset = end + (size & 1)
    if fmt != (3, 1, 48_000, 32) or payload is None or len(payload) % 4:
        raise M0Error("M0 teacher WAV format changed")
    samples = np.frombuffer(payload, dtype="<f4").astype(np.float64)
    if len(samples) == 0 or not np.all(np.isfinite(samples)):
        raise M0Error("M0 teacher WAV is empty or non-finite")
    return 48_000, samples


def align_transfer(data: bytes, profile: ExecutionProfile) -> np.ndarray:
    if len(data) != profile.transfer_samples * 4:
        raise M0Error("M0 transfer sample count changed")
    samples = np.frombuffer(data, dtype="<f4").astype(np.float64)
    if not np.all(np.isfinite(samples)) or float(np.max(np.abs(samples))) <= 0.0:
        raise M0Error("M0 transfer is empty, silent or non-finite")
    peak = int(np.argmax(np.abs(samples)))
    start = peak - profile.transfer_anchor
    end = start + profile.transfer_window
    if start < 0 or end > len(samples):
        raise M0Error("M0 transfer peak cannot satisfy the frozen alignment window")
    result = samples[start:end].copy()
    if int(np.argmax(np.abs(result))) != profile.transfer_anchor:
        raise M0Error("M0 transfer alignment drifted")
    return result


def decode_objectfolder(path: Path, ffmpeg: Path, ffprobe: Path) -> np.ndarray:
    probe = subprocess.run(
        [
            str(ffprobe),
            "-v",
            "error",
            "-select_streams",
            "a:0",
            "-show_entries",
            "stream=sample_rate,channels:format=format_name",
            "-of",
            "json",
            str(path),
        ],
        check=False,
        capture_output=True,
        timeout=30,
    )
    if probe.returncode != 0:
        raise M0Error("M0 ffprobe rejected the ObjectFolder container")
    metadata = parse_json_bytes(probe.stdout, "M0 ffprobe output", 64 * 1024)
    streams = metadata.get("streams")
    format_name = metadata.get("format", {}).get("format_name") if isinstance(metadata.get("format"), dict) else None
    if (
        not isinstance(streams, list)
        or len(streams) != 1
        or streams[0].get("sample_rate") != "44100"
        or streams[0].get("channels") != 2
        or not isinstance(format_name, str)
        or not any(name in format_name.split(",") for name in ("mov", "mp4", "m4a", "3gp", "3g2", "mj2"))
    ):
        raise M0Error("M0 ObjectFolder channel, rate or container changed")
    decode = subprocess.run(
        [
            str(ffmpeg),
            "-v",
            "error",
            "-i",
            str(path),
            "-map",
            "0:a:0",
            "-f",
            "f32le",
            "-acodec",
            "pcm_f32le",
            "pipe:1",
        ],
        check=False,
        capture_output=True,
        timeout=60,
    )
    if decode.returncode != 0 or len(decode.stdout) % 8:
        raise M0Error("M0 ffmpeg failed to decode exact float32 stereo")
    stereo = np.frombuffer(decode.stdout, dtype="<f4").reshape(-1, 2).astype(np.float64)
    if len(stereo) == 0 or not np.all(np.isfinite(stereo)):
        raise M0Error("M0 ObjectFolder decode is empty or non-finite")
    mono = (stereo[:, 0] + stereo[:, 1]) * 0.5
    resampled = signal.resample_poly(mono, 160, 147, window=("kaiser", 5.0), padtype="constant")
    if len(resampled) == 0 or not np.all(np.isfinite(resampled)):
        raise M0Error("M0 ObjectFolder resample failed")
    return np.asarray(resampled, dtype=np.float64)


def canonical_tensor_bytes(tensors: Mapping[str, np.ndarray]) -> bytes:
    names = list(tensors)
    if names != sorted(names) or len(names) != len(set(names)) or not names:
        raise M0Error("M0 tensor names must be non-empty, unique and sorted")
    result = bytearray(TENSOR_MAGIC + struct.pack("<II", 1, len(names)))
    dtype_codes = {"<f4": 1, "<f8": 2, "<i8": 3}
    for name in names:
        encoded = name.encode("utf-8")
        if not 0 < len(encoded) <= 255:
            raise M0Error("M0 tensor name length is invalid")
        array = np.asarray(tensors[name])
        if array.ndim > 8 or any(size < 0 for size in array.shape):
            raise M0Error("M0 tensor rank or shape is invalid")
        little = array.astype(array.dtype.newbyteorder("<"), copy=False)
        dtype = little.dtype.str
        if dtype not in dtype_codes:
            raise M0Error(f"M0 tensor dtype is unsupported: {dtype}")
        if np.issubdtype(little.dtype, np.floating) and not np.all(np.isfinite(little)):
            raise M0Error(f"M0 tensor is non-finite: {name}")
        payload = np.ascontiguousarray(little).tobytes()
        result.extend(struct.pack("<HBB", len(encoded), dtype_codes[dtype], little.ndim))
        result.extend(encoded)
        for size in little.shape:
            result.extend(struct.pack("<Q", size))
        result.extend(struct.pack("<Q", len(payload)))
        result.extend(payload)
    return bytes(result)


def decode_canonical_tensors(data: bytes) -> dict[str, np.ndarray]:
    if len(data) < 16 or data[:8] != TENSOR_MAGIC:
        raise M0Error("M0 tensor container header is invalid")
    version, count = struct.unpack_from("<II", data, 8)
    if version != 1 or count == 0:
        raise M0Error("M0 tensor container version or count is invalid")
    offset = 16
    names: list[str] = []
    result: dict[str, np.ndarray] = {}
    dtypes = {1: np.dtype("<f4"), 2: np.dtype("<f8"), 3: np.dtype("<i8")}
    for _ in range(count):
        if offset + 4 > len(data):
            raise M0Error("M0 tensor entry header is truncated")
        name_length, dtype_code, rank = struct.unpack_from("<HBB", data, offset)
        offset += 4
        if name_length == 0 or rank > 8 or dtype_code not in dtypes or offset + name_length + rank * 8 + 8 > len(data):
            raise M0Error("M0 tensor entry metadata is invalid")
        try:
            name = data[offset : offset + name_length].decode("utf-8")
        except UnicodeDecodeError as error:
            raise M0Error("M0 tensor name is not UTF-8") from error
        offset += name_length
        shape = tuple(struct.unpack_from("<Q", data, offset + index * 8)[0] for index in range(rank))
        offset += rank * 8
        payload_length = struct.unpack_from("<Q", data, offset)[0]
        offset += 8
        expected = math.prod(shape) * dtypes[dtype_code].itemsize
        if payload_length != expected or offset + payload_length > len(data):
            raise M0Error("M0 tensor payload length is invalid")
        array = (
            np.frombuffer(data, dtype=dtypes[dtype_code], count=math.prod(shape), offset=offset).reshape(shape).copy()
        )
        offset += payload_length
        if np.issubdtype(array.dtype, np.floating) and not np.all(np.isfinite(array)):
            raise M0Error("M0 tensor payload is non-finite")
        names.append(name)
        result[name] = array
    if offset != len(data) or names != sorted(names) or len(names) != len(set(names)):
        raise M0Error("M0 tensor order, uniqueness or trailing bytes are invalid")
    return result


def prepare_output(output: Path) -> tuple[Path, Path]:
    output = external_directory(output, "M0 output", must_exist=False)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v25-m0a-", dir=output.parent))
    return staging, output


def publish_output(staging: Path, output: Path) -> None:
    if output.exists():
        raise M0Error("M0 output appeared before atomic publication")
    os.replace(staging, output)


def abandon_output(staging: Path) -> None:
    shutil.rmtree(staging, ignore_errors=True)


def write_artifact(directory: Path, relative: str, data: bytes) -> dict[str, Any]:
    path = directory / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {"path": relative, "sha256": sha256_bytes(data), "byte_count": len(data)}


def reconstruct_teacher_artifact_root(
    t0_root: Path,
    evidence: dict[str, Any],
    combined: dict[str, Any],
) -> dict[str, Any]:
    objects = evidence.get("objects")
    if not isinstance(objects, list) or not objects:
        raise M0Error("M0 teacher object inventory is invalid")
    t0_rows = [row for row in combined["rows"] if row.get("evidence_lane") == "synthetic_teacher"]
    artifacts: list[dict[str, Any]] = []
    for item in objects:
        object_id = item.get("object_id")
        if not isinstance(object_id, str) or not object_id:
            raise M0Error("M0 teacher object ID is invalid")
        relatives = [
            f"objects/{object_id}/mesh-coarse.bin",
            f"objects/{object_id}/mesh.bin",
            f"objects/{object_id}/modal-parameters.bin",
            f"objects/{object_id}/contact-gain-field-coarse.bin",
            f"objects/{object_id}/contact-gain-field.bin",
        ]
        audio_paths = []
        for row in t0_rows:
            path = Path(row.get("audio", {}).get("path", ""))
            try:
                relative = path.resolve(strict=True).relative_to(t0_root)
            except (FileNotFoundError, ValueError) as error:
                raise M0Error("M0 teacher audio escapes the T0 root") from error
            if relative.parts[:2] == ("objects", object_id):
                audio_paths.append(relative.as_posix())
        relatives.extend(sorted(audio_paths))
        if len(audio_paths) != item.get("contact_count"):
            raise M0Error(f"M0 teacher contact inventory changed for {object_id}")
        for relative in relatives:
            path = external_file(t0_root / relative, f"M0 teacher artifact {relative}")
            data = path.read_bytes()
            artifacts.append(
                {
                    "path": relative,
                    "sha256": sha256_bytes(data),
                    "byte_count": len(data),
                }
            )
    observed = sha256_bytes(canonical_json(artifacts))
    if len(artifacts) != evidence.get("artifact_count") or observed != evidence.get("artifact_root_sha256"):
        raise M0Error("M0 teacher artifact root mismatch")
    return {
        "verified": True,
        "artifact_count": len(artifacts),
        "artifact_root_sha256": observed,
    }


class EvidenceVault:
    """Trusted role owner; candidate code receives only explicitly opened rows."""

    def __init__(self, combined: dict[str, Any]) -> None:
        self._rows = tuple(combined["rows"])
        self.phase = Phase.PREPROCESS
        self.candidate_sha256: str | None = None
        self.access_log: list[dict[str, Any]] = []

    def _select(
        self,
        *,
        lane: str,
        split_role: str,
        sample_role: str | None,
        purpose: str,
    ) -> tuple[dict[str, Any], ...]:
        if split_role == "admission_shadow":
            raise M0Error("M0 admission shadow access is forbidden")
        rows = tuple(
            row
            for row in self._rows
            if row.get("evidence_lane") == lane
            and row.get("split_role") == split_role
            and (sample_role is None or row.get("sample_role") == sample_role)
        )
        self.access_log.append(
            {
                "phase": self.phase.name,
                "purpose": purpose,
                "lane": lane,
                "split_role": split_role,
                "sample_role": sample_role,
                "row_count": len(rows),
            }
        )
        return rows

    def begin_training(self) -> None:
        if self.phase != Phase.PREPROCESS:
            raise M0Error("M0 training phase transition is invalid")
        self.phase = Phase.TRAINING

    def synthetic_train_context(self) -> tuple[dict[str, Any], ...]:
        if self.phase != Phase.TRAINING:
            raise M0Error("M0 synthetic training rows opened outside training")
        return self._select(
            lane="synthetic_teacher",
            split_role="train",
            sample_role="context",
            purpose="gradient",
        )

    def synthetic_train_query(self) -> tuple[dict[str, Any], ...]:
        if self.phase not in {Phase.TRAINING, Phase.TRAINED}:
            raise M0Error("M0 train query opened outside diagnostic phase")
        return self._select(
            lane="synthetic_teacher",
            split_role="train",
            sample_role="query",
            purpose="diagnostic_only",
        )

    def real_context(self, lane: str) -> tuple[dict[str, Any], ...]:
        if self.phase != Phase.TRAINING or lane not in {
            "exact_real_transfer",
            "identified_real_recording",
        }:
            raise M0Error("M0 real context opened outside training")
        return self._select(
            lane=lane,
            split_role="development",
            sample_role="context",
            purpose="gradient",
        )

    def finish_training(self) -> None:
        if self.phase != Phase.TRAINING:
            raise M0Error("M0 finish-training transition is invalid")
        self.phase = Phase.TRAINED

    def synthetic_development(self) -> tuple[dict[str, Any], ...]:
        if self.phase != Phase.TRAINED:
            raise M0Error("M0 development opened before training comparison")
        return self._select(
            lane="synthetic_teacher",
            split_role="development",
            sample_role="query",
            purpose="candidate_control_comparison",
        )

    def synthetic_calibration(self) -> tuple[dict[str, Any], ...]:
        if self.phase != Phase.TRAINED:
            raise M0Error("M0 calibration opened outside calibration phase")
        return self._select(
            lane="synthetic_teacher",
            split_role="calibration",
            sample_role=None,
            purpose="uncertainty_only",
        )

    def freeze_candidate(self, candidate_sha256: str) -> None:
        if self.phase != Phase.TRAINED or len(candidate_sha256) != 64:
            raise M0Error("M0 candidate freeze transition is invalid")
        self.candidate_sha256 = candidate_sha256
        self.phase = Phase.CANDIDATE_FROZEN

    def synthetic_method_holdout(self) -> tuple[dict[str, Any], ...]:
        if self.phase != Phase.CANDIDATE_FROZEN or self.candidate_sha256 is None:
            raise M0Error("M0 method holdout opened before candidate freeze")
        rows = self._select(
            lane="synthetic_teacher",
            split_role="method_holdout",
            sample_role="query",
            purpose="one_shot",
        )
        self.phase = Phase.METHOD_HOLDOUT_OPENED
        return rows

    def real_queries(self, lane: str) -> tuple[dict[str, Any], ...]:
        if self.phase != Phase.METHOD_HOLDOUT_OPENED or lane not in {
            "exact_real_transfer",
            "identified_real_recording",
        }:
            raise M0Error("M0 disclosed real query opened before method holdout")
        rows = self._select(
            lane=lane,
            split_role="development",
            sample_role="query",
            purpose="disclosed_query",
        )
        return rows

    def finish_real_queries(self) -> None:
        if self.phase != Phase.METHOD_HOLDOUT_OPENED:
            raise M0Error("M0 real-query completion transition is invalid")
        self.phase = Phase.REAL_QUERY_OPENED

    def admission_shadow(self) -> tuple[dict[str, Any], ...]:
        raise M0Error("M0 admission shadow access is forbidden")


def file_map(directory: Path) -> dict[str, str]:
    return {
        str(path.relative_to(directory)): sha256_file(path) for path in sorted(directory.rglob("*")) if path.is_file()
    }
