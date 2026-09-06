#!/usr/bin/env python3
"""Freeze the R3A internet source and contact roles without reading audio bytes."""

from __future__ import annotations

import argparse
import io
import struct
import subprocess
import tempfile
import zlib
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_contact_field_r3a_common as common


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def parse_headers(data: bytes) -> tuple[int, dict[str, str]]:
    blocks = [block for block in data.split(b"\r\n\r\n") if block.startswith(b"HTTP/")]
    if not blocks:
        raise common.R3AError("HTTPS response has no HTTP header block")
    lines = blocks[-1].decode("latin1").split("\r\n")
    parts = lines[0].split()
    if len(parts) < 2 or not parts[1].isdigit():
        raise common.R3AError("HTTPS response status is malformed")
    headers = {
        name.strip().lower(): value.strip()
        for line in lines[1:]
        if ":" in line
        for name, value in [line.split(":", 1)]
    }
    return int(parts[1]), headers


def curl_request(url: str, arguments: list[str]) -> tuple[bytes, int, dict[str, str]]:
    with tempfile.TemporaryDirectory(prefix="nextengine-r3a-fetch-") as directory:
        body = Path(directory) / "body"
        headers = Path(directory) / "headers"
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
            "--dump-header",
            str(headers),
            "--output",
            str(body),
            *arguments,
            url,
        ]
        result = subprocess.run(command, capture_output=True, check=False)
        if result.returncode != 0:
            raise common.R3AError(
                f"HTTPS source request failed: {result.stderr.decode(errors='replace').strip()}"
            )
        status, parsed = parse_headers(headers.read_bytes())
        return body.read_bytes(), status, parsed


def require_archive_headers(status: int, headers: dict[str, str]) -> None:
    if (
        status not in {200, 206}
        or headers.get("etag", "").strip('"') != common.ARCHIVE_ETAG
        or headers.get("last-modified") != common.ARCHIVE_LAST_MODIFIED
    ):
        raise common.R3AError("REALIMPACT archive HTTP identity changed")


def fetch_entry(spec: dict[str, Any]) -> bytes:
    header_bytes = spec["data_offset"] - spec["local_offset"]
    length = header_bytes + spec["compressed_bytes"]
    end = spec["local_offset"] + length - 1
    body, status, headers = curl_request(
        common.ARCHIVE_URL,
        ["--range", f"{spec['local_offset']}-{end}"],
    )
    require_archive_headers(status, headers)
    if len(body) != length:
        raise common.R3AError(f"short ZIP entry range for {spec['name']}")
    local = body[:header_bytes]
    if len(local) < 30 or local[:4] != b"PK\x03\x04":
        raise common.R3AError(f"bad ZIP local header for {spec['name']}")
    flags, method = struct.unpack_from("<HH", local, 6)
    crc32, compressed, uncompressed = struct.unpack_from("<III", local, 14)
    name_bytes, extra_bytes = struct.unpack_from("<HH", local, 26)
    name = local[30 : 30 + name_bytes].decode("utf-8")
    if (
        flags != 0
        or method != 8
        or name != spec["name"]
        or 30 + name_bytes + extra_bytes != len(local)
        or crc32 != int(spec["crc32"], 16)
        or compressed != spec["compressed_bytes"]
        or uncompressed != spec["uncompressed_bytes"]
    ):
        raise common.R3AError(f"ZIP metadata changed for {spec['name']}")
    try:
        raw = zlib.decompress(body[header_bytes:], wbits=-15)
    except zlib.error as error:
        raise common.R3AError(f"decompress {spec['name']}: {error}") from error
    if (
        len(raw) != spec["uncompressed_bytes"]
        or f"{zlib.crc32(raw) & 0xFFFFFFFF:08x}" != spec["crc32"]
        or common.sha256_bytes(raw) != spec["sha256"]
    ):
        raise common.R3AError(f"decoded integrity changed for {spec['name']}")
    return raw


def npy(raw: bytes, label: str) -> np.ndarray:
    try:
        value = np.load(io.BytesIO(raw), allow_pickle=False)
    except (ValueError, OSError) as error:
        raise common.R3AError(f"parse {label}: {error}") from error
    return np.asarray(value)


def mesh_summary(raw: bytes) -> dict[str, Any]:
    vertices = []
    for line in raw.decode("utf-8").splitlines():
        if line.startswith("v "):
            coordinates = [float(value) for value in line[2:].split()]
            if len(coordinates) != 3:
                raise common.R3AError("REALIMPACT mesh has a malformed vertex")
            vertices.append(coordinates)
    values = np.asarray(vertices, dtype=np.float64)
    if values.shape != (47_738, 3) or not np.all(np.isfinite(values)):
        raise common.R3AError("REALIMPACT Blue Bowl mesh identity changed")
    return {
        "vertex_count": int(values.shape[0]),
        "bbox_min_metres": values.min(axis=0).tolist(),
        "bbox_max_metres": values.max(axis=0).tolist(),
        "sha256": common.sha256_bytes(raw),
    }


def derive_contacts(arrays: dict[str, np.ndarray]) -> list[dict[str, Any]]:
    expected_shapes = {
        "vertex_xyz": (common.ROW_COUNT, 3),
        "listener_xyz": (common.ROW_COUNT, 3),
        "vertex_id": (common.ROW_COUNT,),
        "microphone_id": (common.ROW_COUNT,),
        "distance": (common.ROW_COUNT,),
        "angle": (common.ROW_COUNT,),
    }
    for name, shape in expected_shapes.items():
        if arrays[name].shape != shape:
            raise common.R3AError(f"{name} shape changed: {arrays[name].shape}")
    if not np.all(np.isfinite(arrays["vertex_xyz"])) or not np.all(
        np.isfinite(arrays["listener_xyz"])
    ):
        raise common.R3AError("REALIMPACT coordinates contain non-finite values")

    vertex_ids = arrays["vertex_id"].astype(np.int64)
    source_order = []
    for vertex_id in vertex_ids.tolist():
        if vertex_id not in source_order:
            source_order.append(vertex_id)
    if len(source_order) != common.IMPACT_COUNT:
        raise common.R3AError("Blue Bowl no longer exposes exactly five impacts")

    roles = ["fit", "fit", "fit", "representation_development", "field_holdout"]
    contacts = []
    canonical_listener = None
    for impact_index, (vertex_id, role) in enumerate(
        zip(source_order, roles, strict=True)
    ):
        impact_rows = np.flatnonzero(vertex_ids == vertex_id)
        mask = (
            (arrays["angle"][impact_rows] == common.CANONICAL_ANGLE_DEGREES)
            & (arrays["distance"][impact_rows] == common.CANONICAL_DISTANCE_MILLIMETRES)
            & (arrays["microphone_id"][impact_rows] == common.CANONICAL_MICROPHONE_ID)
        )
        selected = impact_rows[mask]
        if impact_rows.size != common.LISTENERS_PER_IMPACT or selected.size != 1:
            raise common.R3AError("canonical listener is not unique for every impact")
        row = int(selected[0])
        position = arrays["vertex_xyz"][row].astype(np.float64)
        listener = arrays["listener_xyz"][row].astype(np.float64)
        if not np.all(arrays["vertex_xyz"][impact_rows] == position):
            raise common.R3AError("impact position changes across listeners")
        if canonical_listener is None:
            canonical_listener = listener
        elif not np.array_equal(listener, canonical_listener):
            raise common.R3AError(
                "canonical listener coordinates change across impacts"
            )
        contacts.append(
            {
                "impact_index": impact_index,
                "vertex_id": int(vertex_id),
                "position_metres": position.tolist(),
                "row_index": row,
                "role": role,
                "waveform_access": "authorized"
                if role != "field_holdout"
                else "sealed",
            }
        )
    if [item["row_index"] for item in contacts] != sorted(
        item["row_index"] for item in contacts
    ):
        raise common.R3AError("impact source order no longer matches audio row order")
    return contacts


def objectfolder_head() -> dict[str, Any]:
    _, status, headers = curl_request(
        common.OBJECTFOLDER_AUDIO_BATCH["url"], ["--head"]
    )
    expected = common.OBJECTFOLDER_AUDIO_BATCH
    if (
        status != 200
        or int(headers.get("content-length", "-1")) != expected["content_length"]
        or headers.get("etag", "").strip('"') != expected["etag"]
        or headers.get("last-modified") != expected["last_modified"]
        or headers.get("accept-ranges") != "bytes"
    ):
        raise common.R3AError("ObjectFolder Real audio batch identity changed")
    return {
        **expected,
        "availability": "available_but_not_selected_for_first_projection",
        "payload_bytes_accessed": 0,
        "reason": (
            "richer 30-50-contact source retained as a later source-disjoint control; "
            "the 36.37 GB ten-object batch is not needed for the first bounded gate"
        ),
    }


def build_manifest(
    contacts: list[dict[str, Any]],
    mesh: dict[str, Any],
    implementation_sha256: dict[str, str],
) -> dict[str, Any]:
    return {
        "schema": common.MANIFEST_SCHEMA,
        "status": "frozen_zero_audio_preflight",
        "profile": common.PROFILE_ID,
        "implementation_sha256": implementation_sha256,
        "source_repository_revision": common.SOURCE_REPOSITORY_REVISION,
        "archive": {
            "url": common.ARCHIVE_URL,
            "content_length": common.ARCHIVE_BYTES,
            "etag": common.ARCHIVE_ETAG,
            "last_modified": common.ARCHIVE_LAST_MODIFIED,
            "central_directory_sha256": common.ARCHIVE_CENTRAL_SHA256,
        },
        "dataset_object_id": common.DATASET_OBJECT_ID,
        "object_id": common.OBJECT_ID,
        "geometry_revision": common.GEOMETRY_REVISION,
        "mesh": mesh,
        "sample_rate_hz": common.SAMPLE_RATE_HZ,
        "sample_count": common.SAMPLE_COUNT,
        "audio_entry": common.AUDIO_ENTRY,
        "waveform_semantics": (
            "published_float32_force_deconvolved_transfer_response; not a raw impact recording"
        ),
        "force_policy": (
            "retain published windowed-hammer deconvolution; oracle scale derives from fit "
            "contacts only and is applied unchanged to development"
        ),
        "time_policy": {
            "alignment": f"absolute_peak_to_sample_{common.PEAK_ALIGNMENT_SAMPLE}",
            "analysis_window_samples": common.ANALYSIS_SAMPLES,
            "analysis_window_seconds": 3,
            "tail_policy": "remaining published tail excluded from primary endpoints",
        },
        "canonical_listener": {
            "azimuth_degrees": common.CANONICAL_ANGLE_DEGREES,
            "distance_offset_millimetres": common.CANONICAL_DISTANCE_MILLIMETRES,
            "microphone_id": common.CANONICAL_MICROPHONE_ID,
            "claim": "exact_published_condition_only_not_arbitrary_radiation",
        },
        "contacts": contacts,
        "future_role_commitments": {
            "calibration": {
                "group": "realimpact-object-22_Cup",
                "state": "metadata_commitment_only_no_audio_access",
            },
            "method_holdout": {
                "group": "realimpact-object-33_WoodWineGlass",
                "state": "unopened_commitment_only",
            },
            "admission_shadow": {
                "group": "objectfolder-real-object-60_Beer_Glass",
                "state": "unopened_project_disjoint_commitment_only",
            },
        },
        "split_policy": (
            "source-order impacts 0-2 fit, impact 3 representation development, impact 4 "
            "sealed field holdout; fixed from metadata before any selected audio read"
        ),
        "oracle_policy": {
            "query_audio_visibility": "representation_development_only",
            "target_baseline": "nearest_fit_contact_by_euclidean_mesh_position",
            "candidate": "32_damped_modes_plus_192_sparse_dct_residual_coefficients",
            "alternative": "256_sparse_dct_coefficients",
            "equal_scalar_budget": 512,
            "neural_training_authorized": False,
        },
        "unavailable_axes": [
            "raw_force_profile_bytes_in_selected_projection",
            "support_fixture_revision",
            "material_composition_revision",
            "repeat_recording_identity",
        ],
        "prohibited_claims": [
            "arbitrary_listener_radiation",
            "material_universality",
            "raw_recording_equivalence",
            "production_or_runtime_admission",
        ],
    }


def run(root: Path, output_argument: Path) -> Path:
    output, staging = common.prepare_output(root, output_argument)
    try:
        decoded = {
            key: fetch_entry(spec) for key, spec in common.METADATA_ENTRIES.items()
        }
        arrays = {
            key: npy(decoded[key], key)
            for key in [
                "vertex_xyz",
                "listener_xyz",
                "vertex_id",
                "microphone_id",
                "distance",
                "angle",
            ]
        }
        contacts = derive_contacts(arrays)
        mesh = mesh_summary(decoded["mesh"])
        objectfolder = objectfolder_head()
        script_directory = Path(__file__).resolve().parent
        implementation_sha256 = common.implementation_hashes(script_directory)
        manifest = build_manifest(contacts, mesh, implementation_sha256)
        common.validate_manifest(manifest)
        manifest_bytes = common.canonical_json(manifest)
        report = {
            "schema": common.PREFLIGHT_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R3ACorpusReadyForBoundedAudioExtraction",
            "profile": common.PROFILE_ID,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "implementation_sha256": implementation_sha256,
            "realimpact_metadata_payload_bytes": sum(
                spec["compressed_bytes"] for spec in common.METADATA_ENTRIES.values()
            ),
            "audio_payload_bytes_accessed": 0,
            "selected_exact_object_impact_count": len(contacts),
            "authorized_audio_row_count": 4,
            "sealed_audio_row_count": 1,
            "objectfolder_real_audit": objectfolder,
            "neural_training_authorized": False,
            "next_action": (
                "download the pinned compressed REALIMPACT audio member externally, extract "
                "only fit plus representation-development contacts, then run the frozen oracle"
            ),
        }
        (staging / "manifest.json").write_bytes(manifest_bytes)
        (staging / "report.json").write_bytes(common.canonical_json(report))
        common.publish_output(output, staging)
    except BaseException:
        for child in staging.iterdir():
            child.unlink()
        staging.rmdir()
        raise
    return output


def main() -> None:
    arguments = parse_arguments()
    root = Path(__file__).resolve().parents[2]
    output = run(root, arguments.output)
    print(f"R3A source preflight: {output}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")


if __name__ == "__main__":
    main()
