#!/usr/bin/env python3
"""Freeze RealImpact five-impact control metadata without opening audio."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import math
import os
import shutil
import struct
import urllib.error
import urllib.parse
import urllib.request
import zlib
from pathlib import Path
from typing import Any

STUDY_ID = "physical-sound-v13-m2c-realimpact-five-impact-control"
REVISION = "realimpact-93-green-goblet-derived-response-control-v0"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-derived-response-control.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-derived-response-control.report.v0"
PROFILE_SHA256 = "c2301789813b836b84ffb9a41380af02421490073d6585c488e276f1a5677e28"

URL = "https://downloads.cs.stanford.edu/viscam/RealImpact/93_GreenGoblet.zip"
HOST = "downloads.cs.stanford.edu"
ARCHIVE_BYTES = 2_311_697_935
ETAG = '"6433e478-89c9b60f"'
LAST_MODIFIED = "Mon, 10 Apr 2023 10:27:04 GMT"
CENTRAL = {
    "name": "zip-central-directory",
    "offset": 2_311_696_618,
    "compressed_bytes": 1_295,
    "sha256": "083a0677196ee18688c42379d869137fc48e2efd756cd418bf4ef0cfa40eb3a4",
}
ENTRIES = {
    "vertex_xyz": {
        "name": "93_GreenGoblet/preprocessed/vertexXYZ.npy",
        "offset": 258,
        "compressed_bytes": 441,
        "raw_bytes": 72_128,
        "crc32": "695feafe",
        "sha256": "cbc94c54a7ed8f35fd9743c8725678811566f17fc97d9739d110d390d909151e",
        "dtype": "<f8",
        "shape": (3_000, 3),
    },
    "microphone_id": {
        "name": "93_GreenGoblet/preprocessed/micID.npy",
        "offset": 794,
        "compressed_bytes": 225,
        "raw_bytes": 24_128,
        "crc32": "082ec0c6",
        "sha256": "d603b6155b4bad60d9ed6633734b8f52fcad208d5320f925911fc7c463224d6b",
        "dtype": "<i8",
        "shape": (3_000,),
    },
    "mesh": {
        "name": "93_GreenGoblet/preprocessed/transformed.obj",
        "offset": 2_783_916,
        "compressed_bytes": 557_368,
        "raw_bytes": 3_528_966,
        "crc32": "101b1df8",
        "sha256": "96252fe0200699f06ae148d3eeeaf8eb67a1dc87dc4da94796bf375b5dd47192",
    },
    "vertex_id": {
        "name": "93_GreenGoblet/preprocessed/vertexID.npy",
        "offset": 3_341_382,
        "compressed_bytes": 164,
        "raw_bytes": 24_128,
        "crc32": "4f58b204",
        "sha256": "ad1a3143fda803aeaace7b225cfe86ecea5e9393b8eb9f5c99ff5b54513e282d",
        "dtype": "<i8",
        "shape": (3_000,),
    },
    "listener_xyz": {
        "name": "93_GreenGoblet/preprocessed/listenerXYZ.npy",
        "offset": 3_341_647,
        "compressed_bytes": 2_923,
        "raw_bytes": 72_128,
        "crc32": "ee426e91",
        "sha256": "83fa3f27780ab2f56e1b33afa4fbcb25fb712ac0b3731d5343a42ecff7d94dd4",
        "dtype": "<f8",
        "shape": (3_000, 3),
    },
    "distance": {
        "name": "93_GreenGoblet/preprocessed/distance.npy",
        "offset": 3_344_863,
        "compressed_bytes": 294,
        "raw_bytes": 24_128,
        "crc32": "570b48dd",
        "sha256": "95dbc48e33263762a9d0c6233902e53ba754dc5e53943b17043491827dc7188a",
        "dtype": "<i8",
        "shape": (3_000,),
    },
    "angle": {
        "name": "93_GreenGoblet/preprocessed/angle.npy",
        "offset": 2_311_696_326,
        "compressed_bytes": 292,
        "raw_bytes": 24_128,
        "crc32": "fabb9a2e",
        "sha256": "ed65ac28e45cc119b5d42c49293546f2749aa5f9629f6ffa9e0f35f2420874a3",
        "dtype": "<i8",
        "shape": (3_000,),
    },
}
AUDIO_ENTRY = {
    "name": "93_GreenGoblet/preprocessed/deconvolved_0db.npy",
    "offset": 3_345_262,
    "compressed_bytes": 2_308_350_969,
    "raw_bytes": 2_499_876_128,
    "crc32": "d41ca14a",
    "dtype": "<f4",
    "shape": (3_000, 208_323),
}
IMPACT_COUNT = 5
ROWS_PER_IMPACT = 600
EXPECTED_CANONICAL_ROWS = (7, 607, 1207, 1807, 2407)
FOLDS = {
    f"loio-{held}": {
        "fit_impact_parents": tuple(value for value in range(IMPACT_COUNT) if value != held),
        "held_control_parent": held,
    }
    for held in range(IMPACT_COUNT)
}


class ControlError(RuntimeError):
    """The preregistered M2c control boundary was violated."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def require_profile(path: Path) -> dict[str, Any]:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(repository_root()) or not resolved.is_file():
        raise ControlError("frozen profile must be an external regular file")
    if sha256_file(resolved) != PROFILE_SHA256:
        raise ControlError("frozen profile identity changed")
    try:
        value = json.loads(resolved.read_bytes())
    except (UnicodeDecodeError, json.JSONDecodeError, OSError) as error:
        raise ControlError("frozen profile JSON cannot be decoded") from error
    if value.get("dataset_object_id") != "93_GreenGoblet":
        raise ControlError("frozen profile object changed")
    if value.get("impact_vertex_id") != 31_676:
        raise ControlError("frozen profile impact identity changed")
    return value


class SameHostRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, new_url):  # type: ignore[no-untyped-def]
        parsed = urllib.parse.urlparse(new_url)
        if parsed.scheme != "https" or parsed.hostname != HOST:
            raise ControlError("source redirected outside the frozen HTTPS host")
        return super().redirect_request(request, fp, code, msg, headers, new_url)


def audio_interval() -> tuple[int, int]:
    start = AUDIO_ENTRY["offset"]
    return start, start + AUDIO_ENTRY["compressed_bytes"] - 1


def require_no_audio_overlap(start: int, end: int) -> None:
    audio_start, audio_end = audio_interval()
    if not (end < audio_start or start > audio_end):
        raise ControlError("metadata range overlaps the forbidden audio entry")


def fetch_range(start: int, byte_count: int) -> bytes:
    end = start + byte_count - 1
    require_no_audio_overlap(start, end)
    request = urllib.request.Request(
        URL,
        headers={
            "Range": f"bytes={start}-{end}",
            "Accept-Encoding": "identity",
            "User-Agent": "NextEngine-PhysicalSound-Research/0",
        },
    )
    opener = urllib.request.build_opener(SameHostRedirectHandler())
    try:
        with opener.open(request, timeout=120) as response:
            if response.status != 206:
                raise ControlError("range response status changed")
            headers = response.headers
            if headers.get("Content-Range") != f"bytes {start}-{end}/{ARCHIVE_BYTES}":
                raise ControlError("range response Content-Range changed")
            if int(headers.get("Content-Length", "-1")) != byte_count:
                raise ControlError("range response Content-Length changed")
            if headers.get("ETag") != ETAG:
                raise ControlError("range response ETag changed")
            if headers.get("Last-Modified") != LAST_MODIFIED:
                raise ControlError("range response Last-Modified changed")
            final = urllib.parse.urlparse(response.geturl())
            if final.scheme != "https" or final.hostname != HOST:
                raise ControlError("range response final URL changed")
            payload = response.read(byte_count + 1)
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        raise ControlError(f"metadata range fetch failed: {start}-{end}") from error
    if len(payload) != byte_count:
        raise ControlError("range response body length changed")
    return payload


def decode_entry(spec: dict[str, Any], compressed: bytes) -> bytes:
    if len(compressed) != spec["compressed_bytes"]:
        raise ControlError(f"compressed byte count changed: {spec['name']}")
    try:
        raw = zlib.decompress(compressed, -zlib.MAX_WBITS)
    except zlib.error as error:
        raise ControlError(f"cannot inflate metadata entry: {spec['name']}") from error
    if len(raw) != spec["raw_bytes"]:
        raise ControlError(f"raw byte count changed: {spec['name']}")
    if f"{zlib.crc32(raw) & 0xFFFFFFFF:08x}" != spec["crc32"]:
        raise ControlError(f"CRC32 changed: {spec['name']}")
    if hashlib.sha256(raw).hexdigest() != spec["sha256"]:
        raise ControlError(f"raw SHA-256 changed: {spec['name']}")
    return raw


def parse_npy(payload: bytes, dtype: str, shape: tuple[int, ...]) -> tuple[int | float, ...]:
    if len(payload) < 12 or payload[:6] != b"\x93NUMPY":
        raise ControlError("metadata NPY magic changed")
    major = payload[6]
    if major == 1:
        header_bytes = struct.unpack_from("<H", payload, 8)[0]
        offset = 10 + header_bytes
    elif major in (2, 3):
        header_bytes = struct.unpack_from("<I", payload, 8)[0]
        offset = 12 + header_bytes
    else:
        raise ControlError("metadata NPY version changed")
    try:
        header = ast.literal_eval(payload[offset - header_bytes : offset].decode("latin1").strip())
    except (SyntaxError, ValueError, UnicodeDecodeError) as error:
        raise ControlError("metadata NPY header cannot be decoded") from error
    if header.get("descr") != dtype or header.get("fortran_order") is not False:
        raise ControlError("metadata NPY dtype/order changed")
    if tuple(header.get("shape", ())) != shape:
        raise ControlError("metadata NPY shape changed")
    count = math.prod(shape)
    code = "d" if dtype == "<f8" else "q" if dtype == "<i8" else None
    if code is None or len(payload) != offset + count * 8:
        raise ControlError("metadata NPY payload size changed")
    values = struct.unpack_from(f"<{count}{code}", payload, offset)
    if dtype == "<f8" and not all(math.isfinite(value) for value in values):
        raise ControlError("metadata NPY contains non-finite values")
    return values


def parse_mesh(payload: bytes) -> dict[str, Any]:
    try:
        text = payload.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ControlError("mesh is not UTF-8") from error
    vertices: list[tuple[float, float, float]] = []
    face_count = 0
    for line in text.splitlines():
        if line.startswith("v "):
            fields = line.split()
            if len(fields) < 4:
                raise ControlError("mesh vertex line changed")
            vertex = tuple(float(value) for value in fields[1:4])
            if not all(math.isfinite(value) for value in vertex):
                raise ControlError("mesh contains non-finite vertex")
            vertices.append(vertex)
        elif line.startswith("f "):
            face_count += 1
    if not vertices or not face_count:
        raise ControlError("mesh has no vertices or faces")
    columns = tuple(tuple(vertex[index] for vertex in vertices) for index in range(3))
    return {
        "revision": "mesh-96252fe02006-v1",
        "vertex_count": len(vertices),
        "face_count": face_count,
        "minimum_m": [min(column) for column in columns],
        "maximum_m": [max(column) for column in columns],
    }


def triples(values: tuple[int | float, ...]) -> tuple[tuple[float, float, float], ...]:
    return tuple(
        (float(values[index]), float(values[index + 1]), float(values[index + 2]))
        for index in range(0, len(values), 3)
    )


def validate_metadata(
    vertex_xyz: tuple[tuple[float, float, float], ...],
    vertex_id: tuple[int, ...],
    listener_xyz: tuple[tuple[float, float, float], ...],
    microphone_id: tuple[int, ...],
    distance: tuple[int, ...],
    angle: tuple[int, ...],
    mesh_vertex_count: int,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    if not all(len(values) == IMPACT_COUNT * ROWS_PER_IMPACT for values in (
        vertex_xyz,
        vertex_id,
        listener_xyz,
        microphone_id,
        distance,
        angle,
    )):
        raise ControlError("metadata row count changed")
    impact_parents: list[dict[str, Any]] = []
    reference_listener_tuples: tuple[Any, ...] | None = None
    canonical_rows: list[int] = []
    for impact in range(IMPACT_COUNT):
        start = impact * ROWS_PER_IMPACT
        stop = start + ROWS_PER_IMPACT
        ids = vertex_id[start:stop]
        positions = vertex_xyz[start:stop]
        if len(set(ids)) != 1 or len(set(positions)) != 1:
            raise ControlError("impact parent is not constant within its listener block")
        if ids[0] < 0 or ids[0] >= mesh_vertex_count:
            raise ControlError("impact vertex ID is outside the mesh")
        listener_tuples = tuple(
            (
                angle[row],
                distance[row],
                microphone_id[row],
                *listener_xyz[row],
            )
            for row in range(start, stop)
        )
        if len(set(listener_tuples)) != ROWS_PER_IMPACT:
            raise ControlError("listener tuple is not unique inside an impact block")
        if reference_listener_tuples is None:
            reference_listener_tuples = listener_tuples
        elif listener_tuples != reference_listener_tuples:
            raise ControlError("listener grid changed across impact blocks")
        matching = [
            row
            for row in range(start, stop)
            if angle[row] == 0 and distance[row] == 0 and microphone_id[row] == 7
        ]
        if len(matching) != 1:
            raise ControlError("canonical listener is not unique in an impact block")
        canonical_rows.append(matching[0])
        impact_parents.append(
            {
                "impact_parent": impact,
                "vertex_id": ids[0],
                "position_m": list(positions[0]),
                "canonical_listener_row": matching[0],
            }
        )
    identities = {(item["vertex_id"], tuple(item["position_m"])) for item in impact_parents}
    if len(identities) != IMPACT_COUNT:
        raise ControlError("impact parent identities are not distinct")
    if tuple(canonical_rows) != EXPECTED_CANONICAL_ROWS:
        raise ControlError("canonical listener row indexes changed")
    canonical_index = EXPECTED_CANONICAL_ROWS[0]
    canonical_listener = {
        "angle": angle[canonical_index],
        "distance": distance[canonical_index],
        "microphone_id": microphone_id[canonical_index],
        "position_m": list(listener_xyz[canonical_index]),
    }
    return impact_parents, canonical_listener


def prepare_output(output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(repository_root()):
        raise ControlError("output must stay outside the repository")
    if resolved.exists():
        raise ControlError("output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        raise ControlError("staging output already exists")
    staging.mkdir()
    return resolved, staging


def run(profile_argument: Path, output_argument: Path) -> Path:
    profile = require_profile(profile_argument)
    central_payload = fetch_range(CENTRAL["offset"], CENTRAL["compressed_bytes"])
    if hashlib.sha256(central_payload).hexdigest() != CENTRAL["sha256"]:
        raise ControlError("ZIP central directory hash changed")
    raw_entries: dict[str, bytes] = {}
    for key, spec in ENTRIES.items():
        compressed = fetch_range(spec["offset"], spec["compressed_bytes"])
        raw_entries[key] = decode_entry(spec, compressed)

    vertex_xyz = triples(parse_npy(raw_entries["vertex_xyz"], "<f8", (3_000, 3)))
    listener_xyz = triples(parse_npy(raw_entries["listener_xyz"], "<f8", (3_000, 3)))
    vertex_id = tuple(int(value) for value in parse_npy(raw_entries["vertex_id"], "<i8", (3_000,)))
    microphone_id = tuple(
        int(value) for value in parse_npy(raw_entries["microphone_id"], "<i8", (3_000,))
    )
    distance = tuple(int(value) for value in parse_npy(raw_entries["distance"], "<i8", (3_000,)))
    angle = tuple(int(value) for value in parse_npy(raw_entries["angle"], "<i8", (3_000,)))
    mesh = parse_mesh(raw_entries["mesh"])
    impact_parents, canonical_listener = validate_metadata(
        vertex_xyz,
        vertex_id,
        listener_xyz,
        microphone_id,
        distance,
        angle,
        mesh["vertex_count"],
    )
    compressed_source_bytes = CENTRAL["compressed_bytes"] + sum(
        spec["compressed_bytes"] for spec in ENTRIES.values()
    )
    raw_source_bytes = sum(spec["raw_bytes"] for spec in ENTRIES.values())
    manifest = {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenMetadataOnlyControl",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "claim_kind": "publisher_derived_response_control",
        "source": {
            "dataset_object_id": "93_GreenGoblet",
            "archive_url": URL,
            "archive_bytes": ARCHIVE_BYTES,
            "etag": ETAG,
            "last_modified": LAST_MODIFIED,
            "source_repository_revision": "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987",
            "profile_sha256": PROFILE_SHA256,
            "profile_name": profile["profile"],
            "central_directory": CENTRAL,
            "entries": ENTRIES,
            "forbidden_audio_entry": AUDIO_ENTRY,
        },
        "mesh": mesh,
        "impact_parents": impact_parents,
        "canonical_listener": canonical_listener,
        "folds": FOLDS,
        "listener_policy": {
            "contact_control_rows": list(EXPECTED_CANONICAL_ROWS),
            "radiation_metadata_rows_signal_closed": 2_995,
        },
        "read_accounting": {
            "network_range_responses": 8,
            "compressed_source_bytes": compressed_source_bytes,
            "raw_metadata_geometry_bytes_decoded": raw_source_bytes,
            "metadata_scalar_values_decoded": 30_000,
            "audio_compressed_bytes_requested": 0,
            "audio_sample_values_decoded": 0,
            "prior_listener_block_payload_bytes_read": 0,
            "source_payloads_emitted": 0,
        },
        "measured_transfer_field_authorized": False,
        "validator_or_admission_role_allowed": False,
        "runtime_consumer_allowed": False,
        "public_contract": False,
        "authored_clip_fallback_required": True,
        "implementation_sha256": sha256_file(Path(__file__).resolve()),
    }
    manifest_bytes = canonical_json(manifest)
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "READY_FOR_M2D_CANONICAL_SOURCE_RECOVERY",
        "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
        "impact_parent_count": len(impact_parents),
        "leave_one_impact_out_fold_count": len(FOLDS),
        "canonical_listener_rows": list(EXPECTED_CANONICAL_ROWS),
        "mesh_vertex_count": mesh["vertex_count"],
        "mesh_face_count": mesh["face_count"],
        "read_accounting": manifest["read_accounting"],
        "real_formula_or_quality_credit": False,
        "runtime_or_public_contract_changed": False,
        "authored_clip_fallback_required": True,
    }
    output, staging = prepare_output(output_argument)
    try:
        (staging / "manifest.json").write_bytes(manifest_bytes)
        (staging / "report.json").write_bytes(canonical_json(report))
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--frozen-profile", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    run(arguments.frozen_profile, arguments.output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
