#!/usr/bin/env python3
"""Freeze R3A V2 source, split and learned codec before any target audio read."""

from __future__ import annotations

import argparse
import importlib.metadata
import io
import platform
import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path
from typing import Any

import numpy as np
import physical_sound_contact_field_r3a_v2_common as common


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--dac-repository", required=True, type=Path)
    parser.add_argument("--dac-weights", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def parse_headers(data: bytes) -> tuple[int, dict[str, str]]:
    blocks = [block for block in data.split(b"\r\n\r\n") if block.startswith(b"HTTP/")]
    if not blocks:
        raise common.R3AV2Error("HTTPS response has no HTTP header block")
    lines = blocks[-1].decode("latin1").split("\r\n")
    parts = lines[0].split()
    if len(parts) < 2 or not parts[1].isdigit():
        raise common.R3AV2Error("HTTPS response status is malformed")
    headers = {
        name.strip().lower(): value.strip()
        for line in lines[1:]
        if ":" in line
        for name, value in [line.split(":", 1)]
    }
    return int(parts[1]), headers


def curl_request(url: str, arguments: list[str]) -> tuple[bytes, int, dict[str, str]]:
    with tempfile.TemporaryDirectory(prefix="nextengine-r3a-v2-fetch-") as directory:
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
            raise common.R3AV2Error(
                "HTTPS source request failed: "
                + result.stderr.decode(errors="replace").strip()
            )
        status, parsed = parse_headers(headers.read_bytes())
        return body.read_bytes(), status, parsed


def require_archive_headers(status: int, headers: dict[str, str]) -> None:
    if (
        status not in {200, 206}
        or headers.get("etag", "").strip('"') != common.ARCHIVE_ETAG
        or headers.get("last-modified") != common.ARCHIVE_LAST_MODIFIED
    ):
        raise common.R3AV2Error("REALIMPACT archive HTTP identity changed")


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
        raise common.R3AV2Error(f"short ZIP entry range for {spec['name']}")
    local = body[:header_bytes]
    if len(local) < 30 or local[:4] != b"PK\x03\x04":
        raise common.R3AV2Error(f"bad ZIP local header for {spec['name']}")
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
        raise common.R3AV2Error(f"ZIP metadata changed for {spec['name']}")
    try:
        raw = zlib.decompress(body[header_bytes:], wbits=-15)
    except zlib.error as error:
        raise common.R3AV2Error(f"decompress {spec['name']}: {error}") from error
    if (
        len(raw) != spec["uncompressed_bytes"]
        or f"{zlib.crc32(raw) & 0xFFFFFFFF:08x}" != spec["crc32"]
        or common.sha256_bytes(raw) != spec["sha256"]
    ):
        raise common.R3AV2Error(f"decoded integrity changed for {spec['name']}")
    return raw


def npy(raw: bytes, label: str) -> np.ndarray:
    try:
        value = np.load(io.BytesIO(raw), allow_pickle=False)
    except (ValueError, OSError) as error:
        raise common.R3AV2Error(f"parse {label}: {error}") from error
    return np.asarray(value)


def mesh_summary(raw: bytes) -> dict[str, Any]:
    vertices = []
    for line in raw.decode("utf-8").splitlines():
        if line.startswith("v "):
            coordinates = [float(value) for value in line[2:].split()]
            if len(coordinates) != 3:
                raise common.R3AV2Error("REALIMPACT mesh has a malformed vertex")
            vertices.append(coordinates)
    values = np.asarray(vertices, dtype=np.float64)
    if values.shape != (48_084, 3) or not np.all(np.isfinite(values)):
        raise common.R3AV2Error("REALIMPACT Large Swan mesh identity changed")
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
            raise common.R3AV2Error(f"{name} shape changed: {arrays[name].shape}")
    if not np.all(np.isfinite(arrays["vertex_xyz"])) or not np.all(
        np.isfinite(arrays["listener_xyz"])
    ):
        raise common.R3AV2Error("REALIMPACT coordinates contain non-finite values")

    vertex_ids = arrays["vertex_id"].astype(np.int64)
    source_order: list[int] = []
    for vertex_id in vertex_ids.tolist():
        if vertex_id not in source_order:
            source_order.append(vertex_id)
    if len(source_order) != common.IMPACT_COUNT:
        raise common.R3AV2Error("Large Swan no longer exposes exactly five impacts")

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
            raise common.R3AV2Error("canonical listener is not unique for every impact")
        row = int(selected[0])
        position = arrays["vertex_xyz"][row].astype(np.float64)
        listener = arrays["listener_xyz"][row].astype(np.float64)
        if not np.all(arrays["vertex_xyz"][impact_rows] == position):
            raise common.R3AV2Error("impact position changes across listeners")
        if canonical_listener is None:
            canonical_listener = listener
        elif not np.array_equal(listener, canonical_listener):
            raise common.R3AV2Error("canonical listener changes across impacts")
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
    if [item["row_index"] for item in contacts] != [7, 607, 1207, 1807, 2407]:
        raise common.R3AV2Error("Large Swan canonical source order changed")
    return contacts


def repository_identity(repository: Path) -> dict[str, Any]:
    def git(*arguments: str) -> str:
        result = subprocess.run(
            ["git", "-C", str(repository), *arguments],
            capture_output=True,
            check=False,
            text=True,
        )
        if result.returncode != 0:
            raise common.R3AV2Error(f"inspect DAC repository: {result.stderr.strip()}")
        return result.stdout.strip()

    revision = git("rev-parse", "HEAD")
    status = git("status", "--porcelain", "--untracked-files=all")
    if revision != common.DAC_REPOSITORY_REVISION or status:
        raise common.R3AV2Error("DAC repository revision or cleanliness changed")
    return {
        "url": common.DAC_REPOSITORY_URL,
        "revision": revision,
        "tree": git("rev-parse", "HEAD^{tree}"),
        "worktree_clean": True,
    }


def environment_identity() -> dict[str, Any]:
    if sys.version_info[:3] != (3, 11, 15) or platform.system() != "Linux":
        raise common.R3AV2Error("DAC oracle Python/platform identity changed")
    distributions = {
        distribution.metadata["Name"].lower(): distribution
        for distribution in importlib.metadata.distributions()
        if distribution.metadata["Name"]
    }
    installed = {
        name: distribution.version for name, distribution in distributions.items()
    }
    selected = {name: installed.get(name) for name in common.DAC_REQUIRED_PACKAGES}
    if selected != common.DAC_REQUIRED_PACKAGES:
        raise common.R3AV2Error(f"DAC oracle package identity changed: {selected}")
    record_sha256 = {}
    for name in common.DAC_REQUIRED_PACKAGES:
        record = distributions[name].read_text("RECORD")
        if record is None:
            raise common.R3AV2Error(f"DAC dependency has no RECORD: {name}")
        record_sha256[name] = common.sha256_bytes(record.encode())
    complete = sorted(installed.items())
    return {
        "python": platform.python_version(),
        "platform": platform.platform(),
        "required_packages": selected,
        "required_distribution_record_sha256": record_sha256,
        "complete_distribution_count": len(complete),
        "complete_distribution_sha256": common.sha256_bytes(
            common.canonical_json(complete)
        ),
    }


def build_manifest(
    contacts: list[dict[str, Any]],
    mesh: dict[str, Any],
    implementation_sha256: dict[str, str],
    dependency: dict[str, Any],
) -> dict[str, Any]:
    return {
        "schema": common.MANIFEST_SCHEMA,
        "status": "frozen_zero_target_audio_preflight",
        "profile": common.PROFILE_ID,
        "implementation_sha256": implementation_sha256,
        "source_repository_revision": common.SOURCE_REPOSITORY_REVISION,
        "source_roster_sha256": common.SOURCE_ROSTER_SHA256,
        "prior_object_payload_bytes_opened": 0,
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
            "retain published windowed-hammer deconvolution; fit-only peak scaling is "
            "applied unchanged to development"
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
            "sealed field holdout; fixed from metadata before selected audio read"
        ),
        "dac_dependency": dependency,
        "oracle_policy": {
            "query_audio_visibility": "representation_development_only",
            "target_baseline": "nearest_fit_contact_by_euclidean_mesh_position",
            "candidate": "descript_audio_codec_44khz_8kbps_query_reconstruction",
            "resampling": "scipy_resample_poly_147_160_then_160_147_kaiser_beta_5",
            "evaluation_band_hz": [120, 18_000],
            "neural_decoder_role": "report_only_representation_ceiling",
            "ready_for_exact_object_field_requires": (
                "separate bounded deterministic modal_or_multiresolution_cooker"
            ),
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
            "deterministic_runtime_decoder",
            "production_or_runtime_admission",
        ],
    }


def run(
    root: Path,
    repository_argument: Path,
    weights_argument: Path,
    output_argument: Path,
) -> Path:
    repository = common.external_directory(root, repository_argument, "DAC repository")
    weights = common.external_file(root, weights_argument, "DAC weights")
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
        dependency = {
            "repository": repository_identity(repository),
            "repository_revision": common.DAC_REPOSITORY_REVISION,
            "model_type": common.DAC_MODEL_TYPE,
            "model_bitrate": common.DAC_MODEL_BITRATE,
            "model_tag": common.DAC_MODEL_TAG,
            "weights_url": common.DAC_WEIGHTS_URL,
            "weights_sha256": common.DAC_WEIGHTS_SHA256,
            "weights_audit": common.validate_dac_weights(weights),
            "environment": environment_identity(),
        }
        import physical_sound_contact_field_r3a_v2_codec_oracle as oracle

        dependency["synthetic_determinism_control"] = oracle.synthetic_control(weights)
        script_directory = Path(__file__).resolve().parent
        implementation_sha256 = common.implementation_hashes(script_directory)
        manifest = build_manifest(contacts, mesh, implementation_sha256, dependency)
        common.validate_manifest(manifest)
        manifest_bytes = common.canonical_json(manifest)
        report = {
            "schema": common.PREFLIGHT_REPORT_SCHEMA,
            "status": "Validated",
            "decision": "R3AV2LearnedCodecOracleReady",
            "profile": common.PROFILE_ID,
            "manifest_sha256": common.sha256_bytes(manifest_bytes),
            "implementation_sha256": implementation_sha256,
            "realimpact_metadata_payload_bytes": sum(
                spec["compressed_bytes"] for spec in common.METADATA_ENTRIES.values()
            ),
            "target_audio_payload_bytes_accessed": 0,
            "selected_exact_object_impact_count": len(contacts),
            "authorized_audio_row_count": 4,
            "sealed_audio_row_count": 1,
            "dac_dependency": dependency,
            "neural_training_authorized": False,
            "next_action": (
                "extract only fit plus representation-development contacts and run the "
                "frozen learned-codec ceiling oracle twice"
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
    output = run(
        root,
        arguments.dac_repository,
        arguments.dac_weights,
        arguments.output,
    )
    print(f"R3A V2 source/dependency preflight: {output}")
    print(f"manifest sha256: {common.sha256_file(output / 'manifest.json')}")


if __name__ == "__main__":
    main()
