#!/usr/bin/env python3
"""Hash ObjectFolder Real V8 members without decoding or emitting PCM samples."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import struct
import tarfile
from pathlib import Path
from typing import Any

STUDY_ID = "physical-sound-contact-field-r3a-v8-objectfolder-real-source"
REVISION = "objectfolder-real-object91-prefix-inventory-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8-real-source.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v8-real-source.report.v1"

ARCHIVE_URL = (
    "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
    "audio_data_91_100.tar.gz"
)
ARCHIVE_BYTES = 38_866_981_268
ARCHIVE_ETAG = '"63e3844c-90ca71194"'
ARCHIVE_LAST_MODIFIED = "Wed, 08 Feb 2023 11:15:24 GMT"
PREFIX_RANGE = "bytes=0-536870911"
PREFIX_BYTES = 536_870_912
PREFIX_SHA256 = "5ef9789ad023233377aa60d58f66100f0bb62dd5df52556e051e757d13797313"

OBJECT_ID = "91"
OBJECT_NAME = "Glass_Green"
PUBLISHED_MATERIAL = "Glass"
CONTACTS = (
    {"contact_id": "18", "role": "fit", "archive_order": 1},
    {"contact_id": "12", "role": "fit", "archive_order": 2},
    {"contact_id": "4", "role": "fit", "archive_order": 3},
    {"contact_id": "20", "role": "development", "archive_order": 4},
    {"contact_id": "27", "role": "sealed", "archive_order": 5},
)
MEMBER_ORDER = ("Force.wav", "mic.wav", "striking_force.yaml", "metadata.yaml")

EXPECTED_COMMITMENTS = {
    "91/audio/18/Force.wav": (576_044, "69429fa37362c6a46aff4fc0f7312375fc56e68a16d3e96ce518620e666d77a9"),
    "91/audio/18/mic.wav": (576_044, "4cfbfee343d0a336f48f22168645b847d388a05c13142fd4688a36c52b02b5db"),
    "91/audio/18/striking_force.yaml": (30, "934ea4934f5f7ffd7391569fea137af480f0c115a7a7f7552032bae69d1707d7"),
    "91/audio/18/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "91/audio/12/Force.wav": (576_044, "e0cbb5421c62ee782bae47b8eb62db5cc1e7841e907d8bd1dc82bff17d30f6df"),
    "91/audio/12/mic.wav": (576_044, "53d8bf10c9dba1b1ad13b441c45442b3a1ec4c8ecfb83c884b601677487df73d"),
    "91/audio/12/striking_force.yaml": (31, "082fb8338864144915f97e43e97fbba2a81b24de91618c4956bb01b3f56f9ca4"),
    "91/audio/12/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "91/audio/4/Force.wav": (576_044, "cd2260da461f4caaa4e44135e1b3372a19fc4ebcc7a4bb39dd21045d181a808e"),
    "91/audio/4/mic.wav": (576_044, "55ec5ea105c449bb9e5851e221ec114ac17d8f889f1b44d7e9df7446d997ee39"),
    "91/audio/4/striking_force.yaml": (31, "45114ff2f7a6ab759339465f02ec1a69717e19af39fb0576fa32d5ff0f1bdcaf"),
    "91/audio/4/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "91/audio/20/Force.wav": (576_044, "9a59e098345d8d422e0848030f5fb70243e983a69fbb76a2fc75027eb66c2abd"),
    "91/audio/20/mic.wav": (576_044, "5197764481d296f7162b05a95ba5074b4ef4eebe858f7fbd489dfdad5ae03aee"),
    "91/audio/20/striking_force.yaml": (31, "7461c09a89991e57b853568a96ec7ddbe8e612907f843930e9ceabc091c712b9"),
    "91/audio/20/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "91/audio/27/Force.wav": (576_044, "f7d21df0a54d46a0be23615181e8310669123d641ddb4d6f086a8f8906a3ccaf"),
    "91/audio/27/mic.wav": (576_044, "321453de9634d2e2a15a346a69d3a5c1f3233b5bbf5b1da53b8d8dcc4a5332e5"),
    "91/audio/27/striking_force.yaml": (31, "b6cc4d23de4b920fd67b72a13ac0c680d15782ec1904b7b8d3ffd0460df1089a"),
    "91/audio/27/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
}

EXPECTED_WAV_HEADER = {
    "audio_format": 1,
    "channels": 1,
    "sample_rate_hz": 48_000,
    "sample_width_bytes": 2,
    "frame_count": 288_000,
}


class InventoryError(RuntimeError):
    """The frozen V8 real-source boundary was violated."""


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_file(root: Path, path: Path) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise InventoryError("ObjectFolder archive prefix must be an external file")
    return resolved


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise InventoryError("inventory output must stay outside the repository")
    if resolved.exists():
        raise InventoryError("inventory output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def parse_pcm_header(header: bytes, member_size: int) -> dict[str, int]:
    if len(header) < 44 or header[:4] != b"RIFF" or header[8:12] != b"WAVE":
        raise InventoryError("selected WAV has an invalid RIFF/WAVE header")
    if header[12:16] != b"fmt " or struct.unpack_from("<I", header, 16)[0] != 16:
        raise InventoryError("selected WAV does not use the frozen PCM fmt chunk")
    if header[36:40] != b"data":
        raise InventoryError("selected WAV does not use the frozen data chunk layout")
    audio_format, channels, sample_rate = struct.unpack_from("<HHI", header, 20)
    block_align, bits_per_sample = struct.unpack_from("<HH", header, 32)
    data_bytes = struct.unpack_from("<I", header, 40)[0]
    if block_align == 0 or data_bytes % block_align != 0:
        raise InventoryError("selected WAV has an invalid block alignment")
    descriptor = {
        "audio_format": audio_format,
        "channels": channels,
        "sample_rate_hz": sample_rate,
        "sample_width_bytes": bits_per_sample // 8,
        "frame_count": data_bytes // block_align,
    }
    if descriptor != EXPECTED_WAV_HEADER or member_size != 44 + data_bytes:
        raise InventoryError("selected WAV header changed")
    return descriptor


def scan_selected_members(
    prefix: Path,
    expected: dict[str, tuple[int, str]] = EXPECTED_COMMITMENTS,
) -> dict[str, dict[str, Any]]:
    found: dict[str, dict[str, Any]] = {}
    try:
        with prefix.open("rb") as raw, tarfile.open(fileobj=raw, mode="r|gz") as archive:
            for member in archive:
                if member.name not in expected:
                    continue
                if member.name in found:
                    raise InventoryError(f"duplicate selected member: {member.name}")
                if not member.isfile():
                    raise InventoryError(f"selected member is not a file: {member.name}")
                payload = archive.extractfile(member)
                if payload is None:
                    raise InventoryError(f"cannot read selected member: {member.name}")
                digest = hashlib.sha256()
                first_bytes = bytearray()
                byte_count = 0
                while block := payload.read(1024 * 1024):
                    digest.update(block)
                    byte_count += len(block)
                    if len(first_bytes) < 64:
                        first_bytes.extend(block[: 64 - len(first_bytes)])
                expected_bytes, expected_sha256 = expected[member.name]
                observed_sha256 = digest.hexdigest()
                if byte_count != expected_bytes or member.size != expected_bytes:
                    raise InventoryError(f"selected member size changed: {member.name}")
                if observed_sha256 != expected_sha256:
                    raise InventoryError(f"selected member hash changed: {member.name}")
                descriptor: dict[str, Any] = {
                    "path": member.name,
                    "bytes": byte_count,
                    "sha256": observed_sha256,
                    "wav_header": None,
                }
                if member.name.endswith(".wav"):
                    descriptor["wav_header"] = parse_pcm_header(
                        bytes(first_bytes), byte_count
                    )
                found[member.name] = descriptor
                if len(found) == len(expected):
                    break
    except (tarfile.TarError, EOFError, OSError) as error:
        raise InventoryError("cannot scan the bounded ObjectFolder prefix") from error
    missing = sorted(set(expected) - set(found))
    if missing:
        raise InventoryError(f"selected ObjectFolder members are missing: {missing}")
    return found


def contact_manifest(found: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    result = []
    for contact in CONTACTS:
        members = []
        for filename in MEMBER_ORDER:
            path = f"{OBJECT_ID}/audio/{contact['contact_id']}/{filename}"
            members.append(found[path])
        result.append({**contact, "members": members})
    return result


def build_manifest(
    prefix_sha256: str,
    contacts: list[dict[str, Any]],
    implementation_sha256: str,
) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenZeroDecodeInventory",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "source": {
            "archive_url": ARCHIVE_URL,
            "archive_bytes": ARCHIVE_BYTES,
            "archive_etag": ARCHIVE_ETAG,
            "archive_last_modified": ARCHIVE_LAST_MODIFIED,
            "prefix_range": PREFIX_RANGE,
            "prefix_bytes": PREFIX_BYTES,
            "prefix_sha256": prefix_sha256,
        },
        "object": {
            "id": OBJECT_ID,
            "name": OBJECT_NAME,
            "published_material": PUBLISHED_MATERIAL,
            "signal_semantics": "recorded_impact_waveform_plus_measured_force_profile",
            "listener_condition": "dataset_fixed_microphone_geometry_unpublished",
            "contact_coordinates_available_in_bounded_source": False,
        },
        "contacts": contacts,
        "implementation_sha256": implementation_sha256,
        "waveform_sample_values_decoded": 0,
        "fit_waveform_sample_values_decoded": 0,
        "development_waveform_sample_values_decoded": 0,
        "sealed_waveform_sample_values_decoded": 0,
        "selected_member_payloads_emitted": 0,
        "real_fit_extraction_authorized": False,
        "real_development_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def run(root: Path, prefix_argument: Path, output_argument: Path) -> Path:
    prefix = require_external_file(root, prefix_argument)
    if prefix.stat().st_size != PREFIX_BYTES:
        raise InventoryError("ObjectFolder prefix byte count changed")
    prefix_sha256 = sha256_file(prefix)
    if prefix_sha256 != PREFIX_SHA256:
        raise InventoryError("ObjectFolder prefix hash changed")
    found = scan_selected_members(prefix)
    contacts = contact_manifest(found)
    implementation_sha256 = sha256_file(Path(__file__).resolve())
    manifest = build_manifest(prefix_sha256, contacts, implementation_sha256)

    output, staging = prepare_output(root, output_argument)
    try:
        manifest_bytes = canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        selected_bytes_hashed = sum(
            member["bytes"]
            for contact in contacts
            for member in contact["members"]
        )
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": "READY_FOR_V8_REAL_FIT_EXTRACTION",
            "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
            "selected_contact_count": len(contacts),
            "selected_member_count": len(found),
            "selected_member_bytes_hashed": selected_bytes_hashed,
            "unselected_member_payloads_hashed": 0,
            "selected_member_payloads_emitted": 0,
            "wav_headers_validated": sum(
                member["wav_header"] is not None
                for contact in contacts
                for member in contact["members"]
            ),
            "waveform_sample_values_decoded": 0,
            "fit_waveform_sample_values_decoded": 0,
            "development_waveform_sample_values_decoded": 0,
            "sealed_waveform_sample_values_decoded": 0,
            "development_member_bytes_hashed_for_commitment": sum(
                member["bytes"]
                for contact in contacts
                if contact["role"] == "development"
                for member in contact["members"]
            ),
            "sealed_member_bytes_hashed_for_commitment": sum(
                member["bytes"]
                for contact in contacts
                if contact["role"] == "sealed"
                for member in contact["members"]
            ),
            "contact_coordinates_available": False,
            "real_quality_credit": False,
            "next_authorized_step": "FREEZE_AND_IMPLEMENT_V8_REAL_FIT_ONLY_RUNNER",
            "real_development_authorized": False,
            "r3b_authorized": False,
            "runtime_or_public_contract_changed": False,
            "authored_clip_fallback_required": True,
            "environment": {
                "python": platform.python_version(),
                "python_implementation": platform.python_implementation(),
                "platform": platform.platform(),
            },
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        staging.rename(output)
    except BaseException:
        shutil.rmtree(staging)
        raise
    return output


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive-prefix", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    output = run(repository_root(), arguments.archive_prefix, arguments.output)
    report = json.loads((output / "report.json").read_bytes())
    print(f"R3A V8 ObjectFolder Real inventory: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
