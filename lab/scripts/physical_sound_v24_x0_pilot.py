#!/usr/bin/env python3
"""Build the V24 X0 disclosed Blue Bowl real-evidence lane outside Git."""

from __future__ import annotations

import argparse
import binascii
import hashlib
import io
import json
import math
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import zlib
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import numpy as np


MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-v24-x0.manifest.v1"
LANE_SCHEMA = "nextengine.experimental-physical-sound-v24-x0.lane-records.v1"
LINEAGE_SCHEMA = "nextengine.experimental-physical-sound-v24-x0.lineage.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v24-x0.report.v1"
PROTOCOL_SHA256 = "74b80bb02d51d3ed527c179c40ef1928c61dd8947a7abb16cbc266c2333fd14b"
CLAIM = (
    "DISCLOSED_BLUE_BOWL_REAL_EVIDENCE_ONLY / "
    "NO_MISSING_AXIS_MODEL_ADMISSION_OR_RUNTIME_AUTHORITY"
)
LINEAGE_ID = "v24-x0-blue-bowl-real"
MESH_MAGIC = b"NEMESH01"
MAX_INPUT_BYTES = 128 * 1024 * 1024
MAX_OUTPUT_BYTES = 16 * 1024 * 1024

PREDECESSOR_HASHES = {
    "contacts_array": "1971f01a202cbcbe36d7ac7f16c92b6d559f06a432eb948ae782c1ae43a11ba3",
    "contacts_metadata": "c935a3fc8ef6cc43a0dafcea2b4aacf67d520ba08d17553193ccb3412c6e8457",
    "extraction_report": "caf41093d51134332579af95d8e571c90225fcad4316dd2b50f1b79c9bd0bb24",
    "objectfolder_identified_report": "ade80e0fd6eab2c2ae1cc2a47720f573a358082dcfef9237d730e394ba89c882",
    "objectfolder_source_report": "993007be837df3baf7fea5a49d7603451a02f96a15cb24ca8b5927be8d1adc03",
    "preflight_manifest": "d9554d29c85fa601d72c6b35a0b126c038f2cf2338bc36d5d805da6505e7a6f9",
    "preflight_report": "a8c50ec3f6a41a2805eeb4752c56b878e2e4aff4c788f965afdde5bb552e6461",
}
RECORDINGS = (
    ("000", "context", 675_243, "8efe186609fd47205fc935e1f0f7e1a14701f2c0744144afca1de904454a494c"),
    ("020", "context", 695_082, "db03b0905df0708c5e19e7ccc703058949d2629faf93aee468ffa09e45666ac0"),
    ("039", "query", 629_585, "50b126b12fc06e43dc6f6e600dc13c0b320fa7f80b3527e7b17580ca94f2a2aa"),
)
CONTACTS = (
    (0, 7, "context", 35_950, (-0.03610739, -0.0520688, 0.03753229)),
    (1, 607, "context", 21_823, (0.00936602, -0.0155827, 0.00078345)),
    (2, 1207, "context", 10_221, (-0.05887816, -0.0508747, 0.0790166)),
    (3, 1807, "query", 25_307, (-0.04113073, -0.0616422, 0.06258844)),
)
SEALED = (4, 2407, 31_104, (0.04350088, -0.064883, 0.06707589))
LISTENER = (0.23, -0.04345, 0.0)
ARCHIVE = {
    "url": "https://downloads.cs.stanford.edu/viscam/RealImpact/6_Bowl.zip",
    "bytes": 2_397_750_726,
    "etag": "6433dc5b-8eeac5c6",
    "last_modified": "Mon, 10 Apr 2023 09:52:27 GMT",
}


@dataclass(frozen=True)
class ZipEntry:
    name: str
    local_offset: int
    data_offset: int
    compressed_bytes: int
    uncompressed_bytes: int
    crc32: int
    sha256: str


MESH_ENTRY = ZipEntry(
    "6_Bowl/preprocessed/transformed.obj",
    2_397_069_970,
    2_397_070_063,
    679_442,
    3_495_111,
    0xD9C0E316,
    "f23127b45b0b163c796b4ef44d707b59fac58c4ef6e94e7676a4d88470e7f973",
)
LISTENER_ENTRY = ZipEntry(
    "6_Bowl/preprocessed/listenerXYZ.npy",
    1_426,
    1_519,
    2_923,
    72_128,
    0xEE42_6E91,
    "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
)


@dataclass(frozen=True)
class Profile:
    profile_id: str
    contacts: tuple[tuple[int, int, str, int, tuple[float, float, float]], ...]
    sealed: tuple[int, int, int, tuple[float, float, float]]
    transfer_shape: tuple[int, int]
    mesh_vertex_count: int
    bbox_min: tuple[float, float, float]
    bbox_max: tuple[float, float, float]
    listener: tuple[float, float, float]


def profile(profile_id: str) -> Profile:
    if profile_id == "official-v1":
        return Profile(
            profile_id,
            CONTACTS,
            SEALED,
            (4, 230_215),
            47_738,
            (-0.07857965, -0.0792636, -0.00036705),
            (0.08159412, 0.0814389, 0.08488545),
            LISTENER,
        )
    if profile_id == "contract-fixture-v1":
        contacts = tuple(
            (index, index, role, index, point)
            for index, (role, point) in enumerate(
                (
                    ("context", (0.0, 0.0, 0.0)),
                    ("context", (1.0, 0.0, 0.0)),
                    ("context", (0.0, 1.0, 0.0)),
                    ("query", (0.0, 0.0, 1.0)),
                )
            )
        )
        return Profile(
            profile_id,
            contacts,
            (4, 4, 4, (1.0, 1.0, 1.0)),
            (4, 8),
            5,
            (0.0, 0.0, 0.0),
            (1.0, 1.0, 1.0),
            LISTENER,
        )
    raise ValueError(f"unsupported X0 profile: {profile_id}")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate X0 JSON key: {key}")
        result[key] = value
    return result


def parse_json(data: bytes, role: str) -> dict[str, Any]:
    try:
        value = json.loads(data, object_pairs_hook=no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError(f"invalid X0 {role} JSON: {error}") from error
    if not isinstance(value, dict):
        raise ValueError(f"X0 {role} must be a JSON object")
    return value


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def external_file(root: Path, path: Path, role: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise ValueError(f"X0 {role} must be an external regular file: {resolved}")
    return resolved


def external_output(root: Path, path: Path) -> Path:
    unresolved = path if path.is_absolute() else root / path
    parent = unresolved.parent.resolve(strict=True)
    resolved = parent / unresolved.name
    if resolved.is_relative_to(root) or resolved.exists():
        raise ValueError(f"X0 output must be a new external path: {resolved}")
    return resolved


def file_ref(path: Path, data: bytes) -> dict[str, Any]:
    return {"path": str(path), "sha256": sha256_bytes(data), "byte_count": len(data)}


def validate_ref(value: Any, role: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != {"path", "sha256"}:
        raise ValueError(f"X0 {role} file reference fields changed")
    path = value["path"]
    digest = value["sha256"]
    if not isinstance(path, str) or not path or "\0" in path:
        raise ValueError(f"X0 {role} path is invalid")
    if not isinstance(digest, str) or len(digest) != 64 or any(
        char not in "0123456789abcdef" for char in digest
    ):
        raise ValueError(f"X0 {role} SHA-256 is invalid")
    return value


def load_manifest(path: Path) -> tuple[dict[str, Any], bytes]:
    data = path.read_bytes()
    if not 0 < len(data) <= 64 * 1024:
        raise ValueError("X0 manifest must be 1..=65536 bytes")
    value = parse_json(data, "manifest")
    if set(value) != {"schema", "profile", "protocol_sha256", "artifacts", "data_policy"}:
        raise ValueError("X0 manifest fields changed")
    if (
        value["schema"] != MANIFEST_SCHEMA
        or value["profile"] not in {"official-v1", "contract-fixture-v1"}
        or value["protocol_sha256"] != PROTOCOL_SHA256
        or value["data_policy"]
        != {
            "external_output_only": True,
            "large_transfer_refetch_allowed": False,
            "model_training_authorized": False,
            "sealed_contact_access_allowed": False,
            "runtime_authorized": False,
        }
    ):
        raise ValueError("X0 manifest does not match the frozen protocol")
    artifacts = value["artifacts"]
    expected = {*PREDECESSOR_HASHES, "mesh_source", "objectfolder_audio"}
    if not isinstance(artifacts, dict) or set(artifacts) != expected:
        raise ValueError("X0 artifact manifest fields changed")
    for name in PREDECESSOR_HASHES:
        validate_ref(artifacts[name], name)
    source = artifacts["mesh_source"]
    if not isinstance(source, dict) or set(source) not in (
        {"mode"},
        {"mode", "path", "sha256"},
    ):
        raise ValueError("X0 mesh_source fields changed")
    if value["profile"] == "official-v1" and source != {"mode": "bounded_refetch"}:
        raise ValueError("official X0 requires the frozen bounded mesh refetch")
    if value["profile"] == "contract-fixture-v1":
        if source.get("mode") != "local_fixture":
            raise ValueError("fixture X0 requires a local mesh")
        validate_ref({"path": source.get("path"), "sha256": source.get("sha256")}, "mesh")
    recordings = artifacts["objectfolder_audio"]
    if not isinstance(recordings, list) or len(recordings) != 3:
        raise ValueError("X0 requires exactly three ObjectFolder recordings")
    for recording in recordings:
        if not isinstance(recording, dict) or set(recording) != {
            "recording_id",
            "byte_count",
            "path",
            "sha256",
        }:
            raise ValueError("X0 ObjectFolder recording fields changed")
        validate_ref({"path": recording["path"], "sha256": recording["sha256"]}, "recording")
    return value, data


def read_reference(
    root: Path, manifest_directory: Path, reference: dict[str, Any], role: str
) -> tuple[bytes, Path]:
    path = Path(reference["path"])
    path = path if path.is_absolute() else manifest_directory / path
    path = external_file(root, path, role)
    size = path.stat().st_size
    if size > MAX_INPUT_BYTES:
        raise ValueError(f"X0 {role} exceeds {MAX_INPUT_BYTES} bytes")
    data = path.read_bytes()
    actual = sha256_bytes(data)
    if actual != reference["sha256"]:
        raise ValueError(f"X0 {role} hash mismatch: expected {reference['sha256']}, got {actual}")
    return data, path


def validate_official_hashes(manifest: dict[str, Any]) -> None:
    if manifest["profile"] != "official-v1":
        return
    artifacts = manifest["artifacts"]
    for name, expected in PREDECESSOR_HASHES.items():
        if artifacts[name]["sha256"] != expected:
            raise ValueError(f"X0 frozen predecessor hash changed: {name}")
    observed = tuple(
        (row["recording_id"], row["byte_count"], row["sha256"])
        for row in artifacts["objectfolder_audio"]
    )
    expected = tuple((recording, size, digest) for recording, _, size, digest in RECORDINGS)
    if observed != expected:
        raise ValueError("X0 frozen ObjectFolder recordings changed")


def fetch_zip_entry(entry: ZipEntry) -> bytes:
    length = entry.data_offset - entry.local_offset + entry.compressed_bytes
    end = entry.local_offset + length - 1
    command = [
        "curl",
        "--fail",
        "--silent",
        "--show-error",
        "--proto",
        "=https",
        "--noproxy",
        "*",
        "--connect-timeout",
        "30",
        "--max-time",
        "120",
        "--range",
        f"{entry.local_offset}-{end}",
        "--include",
        ARCHIVE["url"],
    ]
    result = subprocess.run(command, check=False, capture_output=True)
    if result.returncode != 0:
        raise ValueError(f"X0 bounded mesh refetch failed: {result.stderr.decode(errors='replace').strip()}")
    split = result.stdout.find(b"\r\n\r\n")
    if split < 0:
        raise ValueError("X0 range response has no HTTP header terminator")
    header = result.stdout[:split].decode("ascii")
    body = result.stdout[split + 4 :]
    lines = header.split("\r\n")
    headers = {
        name.strip().lower(): value.strip()
        for line in lines[1:]
        if ":" in line
        for name, value in [line.split(":", 1)]
    }
    if " 206 " not in f" {lines[0]} " or len(body) != length:
        raise ValueError("X0 bounded range response status or length changed")
    if headers.get("content-range") != f"bytes {entry.local_offset}-{end}/{ARCHIVE['bytes']}":
        raise ValueError("X0 archive Content-Range changed")
    if headers.get("etag", "").strip('"') != ARCHIVE["etag"]:
        raise ValueError("X0 archive ETag changed")
    if headers.get("last-modified") != ARCHIVE["last_modified"]:
        raise ValueError("X0 archive Last-Modified changed")
    header_bytes = entry.data_offset - entry.local_offset
    local = body[:header_bytes]
    if len(local) < 30 or local[:4] != b"PK\x03\x04":
        raise ValueError("X0 ZIP local header is invalid")
    name_bytes, extra_bytes = struct.unpack_from("<HH", local, 26)
    if (
        len(local) != 30 + name_bytes + extra_bytes
        or struct.unpack_from("<H", local, 6)[0] != 0
        or struct.unpack_from("<H", local, 8)[0] != 8
        or struct.unpack_from("<I", local, 14)[0] != entry.crc32
        or struct.unpack_from("<I", local, 18)[0] != entry.compressed_bytes
        or struct.unpack_from("<I", local, 22)[0] != entry.uncompressed_bytes
        or local[30 : 30 + name_bytes] != entry.name.encode()
    ):
        raise ValueError("X0 ZIP local metadata changed")
    raw = zlib.decompress(body[header_bytes:], -zlib.MAX_WBITS)
    if (
        len(raw) != entry.uncompressed_bytes
        or binascii.crc32(raw) != entry.crc32
        or sha256_bytes(raw) != entry.sha256
    ):
        raise ValueError("X0 bounded ZIP member content changed")
    return raw


def parse_obj(data: bytes) -> tuple[np.ndarray, np.ndarray]:
    try:
        lines = data.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise ValueError("X0 mesh OBJ is not UTF-8") from error
    vertices: list[tuple[float, float, float]] = []
    triangles: list[tuple[int, int, int]] = []
    for line in lines:
        fields = line.strip().split()
        if not fields or fields[0].startswith("#"):
            continue
        if fields[0] == "v":
            if len(fields) != 4:
                raise ValueError("X0 OBJ vertex is invalid")
            vertex = tuple(float(value) for value in fields[1:])
            if any(not math.isfinite(value) for value in vertex):
                raise ValueError("X0 OBJ vertex is non-finite")
            vertices.append(vertex)
        elif fields[0] == "f":
            indices = []
            for field in fields[1:]:
                raw = field.split("/", 1)[0]
                index = int(raw)
                if index <= 0 or index > len(vertices):
                    raise ValueError("X0 OBJ face index is invalid")
                indices.append(index - 1)
            if len(indices) < 3:
                raise ValueError("X0 OBJ face has fewer than three vertices")
            triangles.extend((indices[0], indices[i], indices[i + 1]) for i in range(1, len(indices) - 1))
    if not vertices or not triangles:
        raise ValueError("X0 OBJ has no usable mesh")
    return np.asarray(vertices, dtype=np.float64), np.asarray(triangles, dtype=np.uint32)


def encode_mesh(vertices: np.ndarray, triangles: np.ndarray) -> bytes:
    if (
        vertices.ndim != 2
        or vertices.shape[1] != 3
        or triangles.ndim != 2
        or triangles.shape[1] != 3
        or np.any(~np.isfinite(vertices))
        or np.any(triangles >= len(vertices))
    ):
        raise ValueError("X0 canonical mesh payload is invalid")
    return (
        MESH_MAGIC
        + struct.pack("<III", 1, len(vertices), len(triangles))
        + vertices.astype("<f8", copy=False).tobytes()
        + triangles.astype("<u4", copy=False).tobytes()
    )


def validate_predecessors(
    values: dict[str, bytes],
    selected: Profile,
    recording_specs: tuple[tuple[str, str, int, str], ...],
) -> None:
    preflight = parse_json(values["preflight_manifest"], "preflight manifest")
    report = parse_json(values["preflight_report"], "preflight report")
    contacts = parse_json(values["contacts_metadata"], "contacts metadata")
    extraction = parse_json(values["extraction_report"], "extraction report")
    identified = parse_json(values["objectfolder_identified_report"], "identified report")
    sources = parse_json(values["objectfolder_source_report"], "source report")
    if (
        preflight.get("status") != "frozen_zero_audio_preflight"
        or preflight.get("profile") != "realimpact-blue-bowl-canonical-contact-r3a-v1"
        or preflight.get("canonical_listener")
        != {
            "azimuth_degrees": 0,
            "claim": "exact_published_condition_only_not_arbitrary_radiation",
            "distance_offset_millimetres": 0,
            "microphone_id": 7,
        }
        or report.get("status") != "Validated"
        or report.get("audio_payload_bytes_accessed") != 0
        or report.get("authorized_audio_row_count") != 4
        or report.get("sealed_audio_row_count") != 1
    ):
        raise ValueError("X0 REALIMPACT preflight identity changed")
    expected_contacts = [
        {
            "impact_index": impact,
            "position_metres": list(point),
            "role": "representation_development" if role == "query" else "fit",
            "row_index": row,
            "vertex_id": vertex,
            "waveform_access": "authorized",
        }
        for impact, row, role, vertex, point in selected.contacts
    ]
    sealed_impact, sealed_row, sealed_vertex, sealed_point = selected.sealed
    expected_sealed = {
        "impact_index": sealed_impact,
        "position_metres": list(sealed_point),
        "row_index": sealed_row,
        "vertex_id": sealed_vertex,
        "waveform_access": "sealed_not_decompressed",
    }
    if (
        contacts.get("contacts") != expected_contacts
        or contacts.get("sealed_contact_commitment") != expected_sealed
        or contacts.get("sample_rate_hz") != 48_000
        or contacts.get("sample_count") != selected.transfer_shape[1]
        or extraction.get("status") != "Validated"
        or extraction.get("sealed_row_index") != sealed_row
        or extraction.get("sealed_waveform_samples_decoded") != 0
    ):
        raise ValueError("X0 contact or sealed-row lineage changed")
    if identified.get("status") != "Validated" or sources.get("status") != "Validated":
        raise ValueError("X0 ObjectFolder lineage is not Validated")
    entries = {
        entry.get("recording_id"): entry
        for entry in identified.get("entries", [])
        if entry.get("source_id") == "objectfolder-real-demo-6"
    }
    source = next(
        (item for item in sources.get("sources", []) if item.get("id") == "objectfolder-real-demo-6"),
        None,
    )
    if source is None or source.get("adapter_evidence", {}).get("material_label") != "Glass":
        raise ValueError("X0 ObjectFolder material evidence changed")
    for recording, _, size, digest in recording_specs:
        entry = entries.get(recording)
        if (
            entry is None
            or entry.get("material_label") != "Glass"
            or entry.get("object_id") != "6"
            or entry.get("audio_file_bytes") != size
            or entry.get("audio_file_sha256") != digest
        ):
            raise ValueError(f"X0 ObjectFolder identified entry changed: {recording}")


def validate_mesh(selected: Profile, vertices: np.ndarray) -> None:
    if len(vertices) != selected.mesh_vertex_count:
        raise ValueError("X0 mesh vertex count changed")
    if not np.allclose(np.min(vertices, axis=0), selected.bbox_min, rtol=0.0, atol=1.0e-12):
        raise ValueError("X0 mesh minimum bounds changed")
    if not np.allclose(np.max(vertices, axis=0), selected.bbox_max, rtol=0.0, atol=1.0e-12):
        raise ValueError("X0 mesh maximum bounds changed")
    for _, _, _, vertex, point in selected.contacts:
        if not np.array_equal(vertices[vertex], np.asarray(point)):
            raise ValueError("X0 impact position does not match its mesh vertex")
    if not np.array_equal(vertices[selected.sealed[2]], np.asarray(selected.sealed[3])):
        raise ValueError("X0 sealed contact commitment does not match its mesh vertex")


def validate_listener(raw: bytes, selected: Profile) -> None:
    listeners = np.load(io.BytesIO(raw), allow_pickle=False)
    if listeners.dtype.str != "<f8" or listeners.shape != (3000, 3) or np.any(~np.isfinite(listeners)):
        raise ValueError("X0 canonical listener array changed")
    for _, row, _, _, _ in selected.contacts:
        if not np.array_equal(listeners[row], np.asarray(selected.listener)):
            raise ValueError("X0 canonical listener coordinate changed")


def validate_lane_rows(rows: list[dict[str, Any]]) -> None:
    if len(rows) != 7 or [row["row_id"] for row in rows] != sorted(row["row_id"] for row in rows):
        raise ValueError("X0 lane row count or order changed")
    transfer = [row for row in rows if row.get("evidence_lane") == "exact_real_transfer"]
    recordings = [row for row in rows if row.get("evidence_lane") == "identified_real_recording"]
    if len(transfer) != 4 or len(recordings) != 3:
        raise ValueError("X0 lane counts changed")
    for row in transfer:
        if row.get("audio_semantics") != "force_deconvolved_transfer_response":
            raise ValueError("X0 transfer semantics changed")
        if set(row.get("axes", {})) != {"material", "geometry", "impact", "listener"}:
            raise ValueError("X0 transfer axes changed or a missing axis was fabricated")
    for row in recordings:
        if row.get("audio_semantics") != "recorded_impact_waveform":
            raise ValueError("X0 recording semantics changed")
        if set(row.get("axes", {})) != {"material"}:
            raise ValueError("X0 recording axes changed or a missing axis was fabricated")
    if any(row.get("split_role") != "development" for row in rows):
        raise ValueError("X0 rows must remain disclosed development evidence")
    if any(row.get("axes", {}).get("teacher_target") is not None for row in rows):
        raise ValueError("X0 real rows forbid teacher targets")


def write_bytes(root: Path, relative: str, data: bytes) -> dict[str, Any]:
    path = root / relative
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {"path": relative, "sha256": sha256_bytes(data), "byte_count": len(data)}


def build_into(
    staging: Path,
    root: Path,
    manifest_path: Path,
    manifest: dict[str, Any],
    manifest_bytes: bytes,
) -> dict[str, Any]:
    selected = profile(manifest["profile"])
    validate_official_hashes(manifest)
    directory = manifest_path.parent
    artifacts = manifest["artifacts"]
    predecessor_values: dict[str, bytes] = {}
    predecessor_paths: dict[str, Path] = {}
    for name in PREDECESSOR_HASHES:
        data, path = read_reference(root, directory, artifacts[name], name)
        predecessor_values[name] = data
        predecessor_paths[name] = path
    roles = {recording: role for recording, role, _, _ in RECORDINGS}
    recording_specs = tuple(
        (row["recording_id"], roles[row["recording_id"]], row["byte_count"], row["sha256"])
        for row in artifacts["objectfolder_audio"]
    )
    validate_predecessors(predecessor_values, selected, recording_specs)

    array = np.load(io.BytesIO(predecessor_values["contacts_array"]), allow_pickle=False)
    if array.dtype.str != "<f4" or array.shape != selected.transfer_shape or np.any(~np.isfinite(array)):
        raise ValueError("X0 transfer array dtype, shape or finiteness changed")

    source = artifacts["mesh_source"]
    if source["mode"] == "bounded_refetch":
        mesh_raw = fetch_zip_entry(MESH_ENTRY)
        listener_raw = fetch_zip_entry(LISTENER_ENTRY)
        validate_listener(listener_raw, selected)
    else:
        mesh_raw, _ = read_reference(root, directory, source, "fixture mesh")
    vertices, triangles = parse_obj(mesh_raw)
    validate_mesh(selected, vertices)
    mesh_ref = write_bytes(staging, "mesh.bin", encode_mesh(vertices, triangles))

    audio_paths: dict[str, tuple[Path, bytes]] = {}
    expected_recordings = RECORDINGS if selected.profile_id == "official-v1" else None
    for recording in artifacts["objectfolder_audio"]:
        data, path = read_reference(root, directory, recording, f"recording {recording['recording_id']}")
        if len(data) != recording["byte_count"]:
            raise ValueError(f"X0 recording byte count changed: {recording['recording_id']}")
        audio_paths[recording["recording_id"]] = (path, data)
    if expected_recordings is None and tuple(audio_paths) != ("000", "020", "039"):
        raise ValueError("X0 fixture recording order changed")

    transfer_refs = []
    for index, (impact, row, sample_role, vertex, point) in enumerate(selected.contacts):
        payload = array[index].astype("<f4", copy=False).tobytes()
        transfer_refs.append(
            (
                impact,
                row,
                sample_role,
                vertex,
                point,
                write_bytes(staging, f"transfer-row-{impact:03d}.f32le", payload),
            )
        )
    if len({reference[5]["sha256"] for reference in transfer_refs}) != 4:
        raise ValueError("X0 transfer rows are not hash-distinct")

    lineage = {
        "schema": LINEAGE_SCHEMA,
        "status": "Validated",
        "claim": CLAIM,
        "profile": selected.profile_id,
        "protocol_sha256": PROTOCOL_SHA256,
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "predecessors": {
            name: file_ref(predecessor_paths[name], predecessor_values[name])
            for name in sorted(PREDECESSOR_HASHES)
        },
        "source_mesh": {"sha256": sha256_bytes(mesh_raw), "byte_count": len(mesh_raw)},
        "derived_mesh": mesh_ref,
        "transfer_rows": [reference[5] for reference in transfer_refs],
        "canonical_listener": {
            "azimuth_degrees": 0,
            "distance_offset_millimetres": 0,
            "microphone_id": 7,
            "position_metres": list(selected.listener),
        },
        "sealed_contact": {
            "row_index": selected.sealed[1],
            "waveform_access": "sealed_not_decompressed",
            "decoded_sample_count": 0,
        },
        "missing_axes": ["composition", "excitation", "impact_normal", "support"],
        "model_training_authorized": False,
        "real_material_admission_authorized": False,
        "runtime_authorized": False,
    }
    lineage_ref = write_bytes(staging, "x0-lineage-report.json", canonical_json(lineage))
    evidence = {"path": lineage_ref["path"], "sha256": lineage_ref["sha256"]}
    rows = []
    for impact, row, sample_role, _, point, audio in transfer_refs:
        rows.append(
            {
                "row_id": f"x0-realimpact-blue-bowl-contact-{impact:03d}",
                "split_role": "development",
                "sample_role": sample_role,
                "corpus_role": "target",
                "evidence_lane": "exact_real_transfer",
                "audio_semantics": "force_deconvolved_transfer_response",
                "source_group_id": "v24-x0-realimpact-development",
                "family_group_id": "blue-bowl-glass",
                "object_group_id": "v24-x0-blue-bowl-development",
                "recording_parent_id": f"v24-x0-realimpact-contact-{impact:03d}",
                "condition_group_id": f"v24-x0-realimpact-row-{row}",
                "mutation_parent_id": None,
                "lineage_report_ids": [LINEAGE_ID],
                "audio": {"path": audio["path"], "sha256": audio["sha256"]},
                "audio_provenance": evidence,
                "axes": {
                    "material": {"value_id": "glass", "evidence": evidence},
                    "geometry": {
                        "geometry_id": "realimpact-6-bowl-mesh-f23127b45b0b-v1",
                        "feature_artifact": {"path": mesh_ref["path"], "sha256": mesh_ref["sha256"]},
                        "evidence": evidence,
                    },
                    "impact": {
                        "coordinate_profile": "realimpact-transformed-right-handed-metres-v1",
                        "point_metres": list(point),
                        "evidence": evidence,
                    },
                    "listener": {
                        "coordinate_profile": "realimpact-transformed-right-handed-metres-v1",
                        "point_metres": list(selected.listener),
                        "evidence": evidence,
                    },
                },
            }
        )
    recording_roles = {recording: role for recording, role, _, _ in RECORDINGS}
    for recording, (path, data) in audio_paths.items():
        rows.append(
            {
                "row_id": f"x0-objectfolder-blue-bowl-recording-{recording}",
                "split_role": "development",
                "sample_role": recording_roles[recording],
                "corpus_role": "target",
                "evidence_lane": "identified_real_recording",
                "audio_semantics": "recorded_impact_waveform",
                "source_group_id": "v24-x0-objectfolder-development",
                "family_group_id": "blue-bowl-glass",
                "object_group_id": "v24-x0-blue-bowl-development",
                "recording_parent_id": f"v24-x0-objectfolder-recording-{recording}",
                "condition_group_id": f"v24-x0-objectfolder-condition-{recording}",
                "mutation_parent_id": None,
                "lineage_report_ids": [LINEAGE_ID],
                "audio": {"path": str(path), "sha256": sha256_bytes(data)},
                "audio_provenance": evidence,
                "axes": {"material": {"value_id": "glass", "evidence": evidence}},
            }
        )
    rows.sort(key=lambda value: value["row_id"])
    validate_lane_rows(rows)
    lane = {
        "schema": LANE_SCHEMA,
        "status": "Validated",
        "claim": CLAIM,
        "lineage_report": {
            "id": LINEAGE_ID,
            "expected_schema": LINEAGE_SCHEMA,
            "expected_claim": CLAIM,
            "artifact": {"path": lineage_ref["path"], "sha256": lineage_ref["sha256"]},
        },
        "rows": rows,
    }
    lane_ref = write_bytes(staging, "x0-lane-records.json", canonical_json(lane))
    total = sum(path.stat().st_size for path in staging.rglob("*") if path.is_file())
    if total > MAX_OUTPUT_BYTES:
        raise ValueError(f"X0 output exceeds {MAX_OUTPUT_BYTES} bytes")
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "X0_DISCLOSED_BLUE_BOWL_CONTRACT_PASS",
        "claim": CLAIM,
        "profile": selected.profile_id,
        "transfer_row_count": 4,
        "identified_recording_count": 3,
        "mesh_vertex_count": len(vertices),
        "mesh_triangle_count": len(triangles),
        "derived_mesh_sha256": mesh_ref["sha256"],
        "lineage_report_sha256": lineage_ref["sha256"],
        "lane_records_sha256": lane_ref["sha256"],
        "sealed_row_index": selected.sealed[1],
        "sealed_waveform_samples_decoded": 0,
        "output_bytes_before_report": total,
        "model_training_authorized": False,
        "real_material_admission_authorized": False,
        "runtime_authorized": False,
    }
    write_bytes(staging, "report.json", canonical_json(report))
    return report


def run(manifest_path: Path, output_path: Path) -> dict[str, Any]:
    root = repository_root().resolve(strict=True)
    manifest_path = external_file(root, manifest_path, "manifest")
    output_path = external_output(root, output_path)
    manifest, manifest_bytes = load_manifest(manifest_path)
    staging = Path(tempfile.mkdtemp(prefix=".nextengine-v24-x0-", dir=output_path.parent))
    try:
        report = build_into(staging, root, manifest_path, manifest, manifest_bytes)
        os.replace(staging, output_path)
        return report
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    try:
        report = run(arguments.manifest, arguments.output)
    except Exception as error:  # noqa: BLE001 - stable CLI fallback boundary
        print(
            json.dumps(
                {
                    "status": "FallbackOutOfDomain",
                    "decision": "X0_INPUT_OR_CONTRACT_REJECT",
                    "message": str(error),
                },
                sort_keys=True,
            ),
            file=sys.stderr,
        )
        return 1
    print(json.dumps(report, indent=2, sort_keys=True, allow_nan=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
