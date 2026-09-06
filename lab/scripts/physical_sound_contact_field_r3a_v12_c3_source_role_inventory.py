#!/usr/bin/env python3
"""Freeze and audit V12-C3 ObjectFolder source roles without PCM decode."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import math
import os
import platform
import shutil
import tarfile
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_contact_field_r3a_v10_real_source_inventory as compact

STUDY_ID = "physical-sound-contact-field-r3a-v12-c3-source-role-inventory"
REVISION = "object41-steel-six-role-zero-decode-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c3-source-role.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v12-c3-source-role.report.v1"

OBJECT_ID = "41"
OBJECT_NAME = "Wrench_Large"
PUBLISHED_MATERIAL = "Steel"
OBJECT_SCALE = 0.25329818850723695
CONTACT_IDS = tuple(range(35))
ROLE_SALT = "nextengine-v12-c3-object41-role-freeze-v1"
ROLE_COUNTS = (
    ("estimator_fit", 16),
    ("generator_development", 5),
    ("representation_holdout", 4),
    ("validator_calibration", 4),
    ("validator_method_holdout", 3),
    ("admission_shadow", 3),
)
EXPECTED_ROLE_IDS = {
    "estimator_fit": (30, 13, 24, 14, 19, 9, 6, 15, 5, 33, 2, 29, 4, 16, 31, 34),
    "generator_development": (23, 32, 18, 0, 10),
    "representation_holdout": (1, 25, 17, 21),
    "validator_calibration": (3, 12, 26, 11),
    "validator_method_holdout": (28, 22, 8),
    "admission_shadow": (27, 7, 20),
}
ROLE_ORDER_ROOT = "235e418fb62febf3673af2606f7a9efb3b856ac67526bda0ae6464834abe0620"

OFFICIAL_SITE_COMMIT = "d058ba09e7e7a1a8358d64d7b48e5f588b377eb8"
OFFICIAL_PAGE_URL = (
    "https://raw.githubusercontent.com/objectfolder/objectfolder.github.io/"
    f"{OFFICIAL_SITE_COMMIT}/source/_pages/objectfolder-real-download.md"
)
OFFICIAL_PAGE_BYTES = 8_668
OFFICIAL_PAGE_SHA256 = "2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32"
BENCHMARK_COMMIT = "4bb002f519cab9d250bbbe045a6df0248bf1639f"

RAW_ARCHIVE_URL = (
    "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
    "audio_data_41_50.tar.gz"
)
RAW_ARCHIVE_BYTES = 40_775_630_586
RAW_ARCHIVE_ETAG = '"63e36ee1-97e6abefa"'
RAW_ARCHIVE_LAST_MODIFIED = "Wed, 08 Feb 2023 09:44:01 GMT"
RAW_PREFIX_RANGE = "bytes=0-4294967295"
RAW_PREFIX_BYTES = 4_294_967_296
PREFREEZE_PREFIX_BYTES = 536_870_912
PREFREEZE_PREFIX_SHA256 = "eca66c012e1a07e07f7fc6642460f02fb89e16d8dedb8568beb0e99cb97786d5"
PREFREEZE_CONTACT_IDS = (5, 15, 27, 33, 34)

SOURCE_SPECS = {
    "audio_archive": {
        "bytes": 463_486_373,
        "sha256": "14a15b96dda7c3933a4a5829dc5ee21e1f9c2fe29f7c558925df24e87097a2c9",
    },
    "contacts_archive": {
        "bytes": 121_961,
        "sha256": "310c45e11e40ceafc9c9fc1379059848ff664a7c2999fea5aabea479fc61eeee",
    },
    "point_cloud_archive": {
        "bytes": 2_359_020,
        "sha256": "29ebf37bb88bc5c654b096f31552e4677fc156135930da72c82e306c5c517e51",
    },
    "split_json": {
        "bytes": 19_258,
        "sha256": "77ea2d99724197ad9e88c3ad90f49885c582dea6835fdc0ad533e5b9ce78674e",
    },
    "scale_json": {
        "bytes": 2_652,
        "sha256": "39c84eafabe999b1503710a9d601b0bc94fc6e358461aa2e5988b34b6b6d0b8a",
    },
}
RAW_MEMBER_NAMES = ("mic.wav", "Force.wav", "metadata.yaml", "striking_force.yaml")
EXPECTED_WAV_HEADER = {
    "audio_format": 1,
    "channels": 1,
    "sample_rate_hz": 48_000,
    "sample_width_bytes": 2,
    "frame_count": 288_000,
}

PROTOCOL_PATH = (
    Path("docs/development")
    / "physical-sound-r3a-v12-c3-object41-source-role-freeze-protocol-2026-08-31.md"
)


class InventoryError(RuntimeError):
    """The preregistered C3 source or role boundary was violated."""


def canonical_json(value: Any) -> bytes:
    def ready(item: Any) -> Any:
        if isinstance(item, np.generic):
            return item.item()
        if isinstance(item, np.ndarray):
            return item.tolist()
        if isinstance(item, dict):
            return {key: ready(child) for key, child in item.items()}
        if isinstance(item, (list, tuple)):
            return [ready(child) for child in item]
        return item

    return (json.dumps(ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def implementation_hashes() -> dict[str, str]:
    root = repository_root()
    paths = {
        "protocol": root / PROTOCOL_PATH,
        "runner": Path(__file__).resolve(),
        "compact_inventory": root / "lab/scripts/physical_sound_contact_field_r3a_v10_real_source_inventory.py",
    }
    return {name: sha256_file(path) for name, path in paths.items()}


def role_partition() -> tuple[dict[str, tuple[int, ...]], str, tuple[int, ...]]:
    ranked = sorted(
        (sha256_bytes(f"{ROLE_SALT}:{contact_id}".encode()), contact_id)
        for contact_id in CONTACT_IDS
    )
    partition: dict[str, tuple[int, ...]] = {}
    offset = 0
    for role, count in ROLE_COUNTS:
        partition[role] = tuple(contact_id for _, contact_id in ranked[offset : offset + count])
        offset += count
    root = sha256_bytes(
        ("\n".join(f"{digest} {contact_id}" for digest, contact_id in ranked) + "\n").encode()
    )
    order = tuple(contact_id for _, contact_id in ranked)
    if partition != EXPECTED_ROLE_IDS or root != ROLE_ORDER_ROOT or offset != len(CONTACT_IDS):
        raise InventoryError("frozen role partition changed")
    return partition, root, order


def role_for_contact(partition: dict[str, tuple[int, ...]]) -> dict[int, str]:
    result: dict[int, str] = {}
    for role, contacts in partition.items():
        for contact_id in contacts:
            if contact_id in result:
                raise InventoryError("role partition overlaps")
            result[contact_id] = role
    if tuple(sorted(result)) != CONTACT_IDS:
        raise InventoryError("role partition is incomplete")
    return result


def sample_budgets(partition: dict[str, tuple[int, ...]]) -> dict[str, dict[str, int]]:
    return {
        role: {
            "contact_count": len(contacts),
            "microphone_sample_values": len(contacts) * EXPECTED_WAV_HEADER["frame_count"],
            "force_sample_values": len(contacts) * EXPECTED_WAV_HEADER["frame_count"],
        }
        for role, contacts in partition.items()
    }


def require_external_file(root: Path, path: Path, label: str) -> Path:
    try:
        resolved = path.resolve(strict=True)
    except OSError as error:
        raise InventoryError(f"{label} is missing") from error
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise InventoryError(f"{label} must be an external regular file")
    return resolved


def require_exact_file(
    root: Path, path: Path, label: str, expected_bytes: int, expected_sha256: str
) -> Path:
    resolved = require_external_file(root, path, label)
    if resolved.stat().st_size != expected_bytes:
        raise InventoryError(f"{label} byte count changed")
    if sha256_file(resolved) != expected_sha256:
        raise InventoryError(f"{label} hash changed")
    return resolved


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise InventoryError("C3 output must stay outside the repository")
    if resolved.exists():
        raise InventoryError("C3 output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def publish_output(staging: Path, output: Path) -> None:
    staging.replace(output)


def write_json(path: Path, value: Any) -> str:
    payload = canonical_json(value)
    path.write_bytes(payload)
    return sha256_bytes(payload)


def validate_manifest(manifest_path: Path) -> tuple[dict[str, Any], str]:
    try:
        payload = manifest_path.read_bytes()
        manifest = json.loads(payload)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InventoryError("cannot read C3 manifest") from error
    if canonical_json(manifest) != payload:
        raise InventoryError("C3 manifest is not canonical JSON")
    if manifest.get("schema") != MANIFEST_SCHEMA or manifest.get("study_id") != STUDY_ID:
        raise InventoryError("C3 manifest identity changed")
    if manifest.get("implementation_sha256") != implementation_hashes():
        raise InventoryError("C3 implementation hashes changed")
    partition, root, order = role_partition()
    if manifest.get("roles") != {key: list(value) for key, value in partition.items()}:
        raise InventoryError("C3 manifest roles changed")
    if manifest.get("role_order_root") != root or manifest.get("contact_hash_order") != list(order):
        raise InventoryError("C3 manifest role order changed")
    prefix = manifest.get("raw_prefix", {})
    if prefix.get("range") != RAW_PREFIX_RANGE or prefix.get("bytes") != RAW_PREFIX_BYTES:
        raise InventoryError("C3 raw-prefix budget changed")
    digest = prefix.get("sha256")
    if not isinstance(digest, str) or len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
        raise InventoryError("C3 raw-prefix hash is invalid")
    return manifest, sha256_bytes(payload)


def freeze(prefix_sha256: str, output_argument: Path) -> Path:
    if len(prefix_sha256) != 64 or any(c not in "0123456789abcdef" for c in prefix_sha256):
        raise InventoryError("--prefix-sha256 must be lowercase SHA-256")
    root = repository_root()
    output, staging = prepare_output(root, output_argument)
    partition, role_root, order = role_partition()
    manifest = {
        "schema": MANIFEST_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "object": {
            "id": OBJECT_ID,
            "name": OBJECT_NAME,
            "published_material": PUBLISHED_MATERIAL,
            "scale": OBJECT_SCALE,
            "contact_ids": list(CONTACT_IDS),
            "claim": "canonical_recorded_setup_normalized_instrument_counts",
        },
        "official_sources": {
            "site_commit": OFFICIAL_SITE_COMMIT,
            "site_page_url": OFFICIAL_PAGE_URL,
            "benchmark_commit": BENCHMARK_COMMIT,
            "raw_archive_url": RAW_ARCHIVE_URL,
            "raw_archive_bytes": RAW_ARCHIVE_BYTES,
            "raw_archive_etag": RAW_ARCHIVE_ETAG,
            "raw_archive_last_modified": RAW_ARCHIVE_LAST_MODIFIED,
        },
        "raw_prefix": {
            "range": RAW_PREFIX_RANGE,
            "bytes": RAW_PREFIX_BYTES,
            "sha256": prefix_sha256,
            "extension_allowed": False,
        },
        "compact_sources": SOURCE_SPECS,
        "roles": {key: list(value) for key, value in partition.items()},
        "role_salt": ROLE_SALT,
        "role_order_root": role_root,
        "contact_hash_order": list(order),
        "sample_budgets": sample_budgets(partition),
        "data_policy": {
            "pcm_sample_decode_allowed": False,
            "striking_force_scalar_parse_allowed": False,
            "network_allowed": False,
            "outputs_must_be_external": True,
            "only_fit_may_be_authorized_by_next_protocol": True,
            "all_other_roles_sealed": True,
        },
        "missing_axes": [
            "numeric_listener_pose",
            "per_object_support_fixture",
            "force_si_calibration",
            "microphone_si_calibration",
        ],
        "expected_wav_header": EXPECTED_WAV_HEADER,
        "implementation_sha256": implementation_hashes(),
        "environment": {
            "python": platform.python_version(),
            "python_implementation": platform.python_implementation(),
            "numpy": np.__version__,
        },
        "counters": {
            "network_requests": 0,
            "source_bytes_read": 0,
            "microphone_sample_values_decoded": 0,
            "force_sample_values_decoded": 0,
        },
        "next_authorized_step": "C3_ZERO_READ_PREFLIGHT",
    }
    write_json(staging / "manifest.json", manifest)
    publish_output(staging, output)
    return output


def zero_counters(partition: dict[str, tuple[int, ...]]) -> dict[str, Any]:
    return {
        "network_requests": 0,
        "source_bytes_read": 0,
        "raw_prefix_bytes_hashed": 0,
        "selected_raw_member_bytes_hashed": 0,
        "compact_microphone_bytes_hashed": 0,
        "coordinate_values_decoded": 0,
        "point_cloud_values_decoded": 0,
        "microphone_sample_values_decoded": 0,
        "force_sample_values_decoded": 0,
        "selected_payloads_emitted": 0,
        "per_role": {
            role: {
                "microphone_sample_values_decoded": 0,
                "force_sample_values_decoded": 0,
                "payloads_emitted": 0,
            }
            for role in partition
        },
    }


def preflight(manifest_path: Path, output_argument: Path) -> Path:
    root = repository_root()
    output, staging = prepare_output(root, output_argument)
    manifest, manifest_sha256 = validate_manifest(manifest_path)
    partition, _, _ = role_partition()
    report = {
        "schema": REPORT_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "stage": "preflight",
        "manifest_sha256": manifest_sha256,
        "implementation_sha256": manifest["implementation_sha256"],
        "decision": "C3_SOURCE_ROLE_FREEZE_FROZEN",
        "roles": manifest["roles"],
        "role_decode_authorized": {role: False for role in partition},
        "counters": zero_counters(partition),
        "next_authorized_step": "C3_REPEAT_EXACT_SOURCE_INVENTORY",
    }
    write_json(staging / "report.json", report)
    publish_output(staging, output)
    return output


def parse_wav_header(payload: bytes, member_size: int) -> dict[str, int]:
    try:
        observed = compact.parse_pcm_header(payload[:64], member_size)
    except compact.InventoryError as error:
        raise InventoryError(str(error)) from error
    if observed != EXPECTED_WAV_HEADER:
        raise InventoryError("WAV header changed")
    return observed


def validate_official_page(path: Path) -> dict[str, Any]:
    text = path.read_text()
    required = (
        "ground-truth contact force profile",
        "coordinate of the striking location on the object mesh",
        "|  41   |    Wrench_Large    |     Steel",
        "audio_data_[X+1]_[X+10].tar.gz",
    )
    if any(fragment not in text for fragment in required):
        raise InventoryError("official ObjectFolder page semantics changed")
    return {
        "bytes": path.stat().st_size,
        "sha256": sha256_file(path),
        "commit": OFFICIAL_SITE_COMMIT,
        "url": OFFICIAL_PAGE_URL,
    }


def scan_compact_audio(path: Path) -> dict[int, dict[str, Any]]:
    expected = {f"audio/{OBJECT_ID}/{contact_id}.wav": contact_id for contact_id in CONTACT_IDS}
    found: dict[int, dict[str, Any]] = {}
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                contact_id = expected.get(member.name)
                if contact_id is None:
                    continue
                if contact_id in found or not member.isfile():
                    raise InventoryError("duplicate or invalid compact microphone member")
                stream = archive.extractfile(member)
                if stream is None:
                    raise InventoryError("cannot read compact microphone member")
                payload = stream.read()
                found[contact_id] = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": sha256_bytes(payload),
                    "wav_header": parse_wav_header(payload, member.size),
                }
    except (tarfile.TarError, OSError) as error:
        raise InventoryError("cannot scan compact audio archive") from error
    if tuple(sorted(found)) != CONTACT_IDS:
        raise InventoryError("compact object-41 microphone set changed")
    return found


def npy_from_payload(payload: bytes, label: str) -> np.ndarray:
    try:
        value = np.load(io.BytesIO(payload), allow_pickle=False)
    except (OSError, ValueError) as error:
        raise InventoryError(f"cannot decode {label} NPY metadata") from error
    if not np.all(np.isfinite(value)):
        raise InventoryError(f"{label} contains non-finite metadata")
    return np.asarray(value)


def scan_coordinates(path: Path) -> dict[int, dict[str, Any]]:
    expected = {f"contacts/{OBJECT_ID}/{contact_id}.npy": contact_id for contact_id in CONTACT_IDS}
    found: dict[int, dict[str, Any]] = {}
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                contact_id = expected.get(member.name)
                if contact_id is None:
                    continue
                stream = archive.extractfile(member)
                if stream is None or not member.isfile() or contact_id in found:
                    raise InventoryError("duplicate or invalid coordinate member")
                payload = stream.read()
                value = npy_from_payload(payload, "coordinate")
                if value.shape != (3,):
                    raise InventoryError("coordinate shape changed")
                found[contact_id] = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": sha256_bytes(payload),
                    "coordinate_m": value.astype(np.float64).tolist(),
                }
    except (tarfile.TarError, OSError) as error:
        raise InventoryError("cannot scan coordinate archive") from error
    if tuple(sorted(found)) != CONTACT_IDS:
        raise InventoryError("coordinate object-41 key set changed")
    return found


def scan_point_cloud(path: Path) -> dict[str, Any]:
    expected = f"global_gt_points/{OBJECT_ID}.npy"
    try:
        with tarfile.open(path, "r:gz") as archive:
            member = archive.getmember(expected)
            stream = archive.extractfile(member)
            if stream is None or not member.isfile():
                raise InventoryError("invalid object-41 point cloud member")
            payload = stream.read()
    except (tarfile.TarError, KeyError, OSError) as error:
        raise InventoryError("cannot scan point-cloud archive") from error
    value = npy_from_payload(payload, "point cloud")
    if value.shape != (1024, 3):
        raise InventoryError("point-cloud shape changed")
    return {
        "path": expected,
        "bytes": len(payload),
        "sha256": sha256_bytes(payload),
        "shape": list(value.shape),
        "dtype": str(value.dtype),
        "bounds_m": [np.min(value, axis=0).tolist(), np.max(value, axis=0).tolist()],
    }


def validate_split(path: Path) -> dict[str, list[Any]]:
    try:
        value = json.loads(path.read_text())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InventoryError("cannot read split JSON") from error
    selected = {
        role: [entry for entry in value[role] if str(entry[0]) == OBJECT_ID]
        for role in ("train", "val", "test")
    }
    if any(selected.values()):
        raise InventoryError("object 41 unexpectedly entered the official split")
    return selected


def validate_scale(path: Path) -> float:
    try:
        value = float(json.loads(path.read_text())[OBJECT_ID])
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, KeyError, TypeError, ValueError) as error:
        raise InventoryError("cannot read object-41 scale") from error
    if not math.isfinite(value) or value != OBJECT_SCALE:
        raise InventoryError("object-41 scale changed")
    return value


def scan_raw_prefix(path: Path, contact_ids: tuple[int, ...] = CONTACT_IDS) -> dict[str, Any]:
    expected = {
        f"{OBJECT_ID}/audio/{contact_id}/{member_name}": (contact_id, member_name)
        for contact_id in contact_ids
        for member_name in RAW_MEMBER_NAMES
    }
    found: dict[str, dict[str, Any]] = {}
    archive_order: list[int] = []
    bytes_consumed = 0
    truncated = False
    try:
        with path.open("rb") as raw:
            try:
                with tarfile.open(fileobj=raw, mode="r|gz") as archive:
                    for member in archive:
                        parts = member.name.split("/")
                        if len(parts) >= 3 and parts[:2] == [OBJECT_ID, "audio"] and parts[2].isdecimal():
                            contact_id = int(parts[2])
                            if contact_id not in archive_order:
                                archive_order.append(contact_id)
                        identity = expected.get(member.name)
                        if identity is None:
                            continue
                        if member.name in found or not member.isfile():
                            raise InventoryError(f"duplicate or invalid raw member: {member.name}")
                        stream = archive.extractfile(member)
                        if stream is None:
                            raise InventoryError(f"cannot read raw member: {member.name}")
                        payload = stream.read()
                        descriptor: dict[str, Any] = {
                            "path": member.name,
                            "contact_id": identity[0],
                            "member_name": identity[1],
                            "bytes": len(payload),
                            "sha256": sha256_bytes(payload),
                            "wav_header": None,
                        }
                        if identity[1].endswith(".wav"):
                            descriptor["wav_header"] = parse_wav_header(payload, member.size)
                        found[member.name] = descriptor
                        if len(found) == len(expected):
                            break
            except (tarfile.ReadError, EOFError):
                truncated = True
            bytes_consumed = raw.tell()
    except OSError as error:
        raise InventoryError("cannot scan bounded raw prefix") from error

    missing = sorted(set(expected) - set(found))
    complete_contacts = sorted(
        contact_id
        for contact_id in contact_ids
        if all(f"{OBJECT_ID}/audio/{contact_id}/{name}" in found for name in RAW_MEMBER_NAMES)
    )
    return {
        "records": found,
        "missing_members": missing,
        "complete_contact_ids": complete_contacts,
        "archive_contact_order": archive_order,
        "compressed_bytes_consumed": bytes_consumed,
        "truncated_prefix_observed": truncated,
        "complete": not missing,
    }


def inventory(
    manifest_path: Path,
    official_page_argument: Path,
    raw_prefix_argument: Path,
    audio_argument: Path,
    contacts_argument: Path,
    point_cloud_argument: Path,
    split_argument: Path,
    scale_argument: Path,
    output_argument: Path,
) -> Path:
    root = repository_root()
    output, staging = prepare_output(root, output_argument)
    manifest, manifest_sha256 = validate_manifest(manifest_path)
    partition, _, _ = role_partition()
    contact_roles = role_for_contact(partition)

    official_page = require_exact_file(
        root, official_page_argument, "official page", OFFICIAL_PAGE_BYTES, OFFICIAL_PAGE_SHA256
    )
    raw_prefix = require_exact_file(
        root,
        raw_prefix_argument,
        "raw prefix",
        RAW_PREFIX_BYTES,
        manifest["raw_prefix"]["sha256"],
    )
    sources = {
        "audio_archive": require_exact_file(
            root, audio_argument, "compact audio", SOURCE_SPECS["audio_archive"]["bytes"], SOURCE_SPECS["audio_archive"]["sha256"]
        ),
        "contacts_archive": require_exact_file(
            root, contacts_argument, "contacts", SOURCE_SPECS["contacts_archive"]["bytes"], SOURCE_SPECS["contacts_archive"]["sha256"]
        ),
        "point_cloud_archive": require_exact_file(
            root, point_cloud_argument, "point clouds", SOURCE_SPECS["point_cloud_archive"]["bytes"], SOURCE_SPECS["point_cloud_archive"]["sha256"]
        ),
        "split_json": require_exact_file(
            root, split_argument, "split", SOURCE_SPECS["split_json"]["bytes"], SOURCE_SPECS["split_json"]["sha256"]
        ),
        "scale_json": require_exact_file(
            root, scale_argument, "scale", SOURCE_SPECS["scale_json"]["bytes"], SOURCE_SPECS["scale_json"]["sha256"]
        ),
    }

    page = validate_official_page(official_page)
    compact_audio = scan_compact_audio(sources["audio_archive"])
    coordinates = scan_coordinates(sources["contacts_archive"])
    point_cloud = scan_point_cloud(sources["point_cloud_archive"])
    split = validate_split(sources["split_json"])
    scale = validate_scale(sources["scale_json"])
    raw = scan_raw_prefix(raw_prefix)

    identity_matches = 0
    paired_headers = 0
    records: list[dict[str, Any]] = []
    if raw["complete"]:
        for contact_id in CONTACT_IDS:
            microphone = raw["records"][f"{OBJECT_ID}/audio/{contact_id}/mic.wav"]
            force = raw["records"][f"{OBJECT_ID}/audio/{contact_id}/Force.wav"]
            if microphone["sha256"] == compact_audio[contact_id]["sha256"]:
                identity_matches += 1
            if microphone["wav_header"] == force["wav_header"] == EXPECTED_WAV_HEADER:
                paired_headers += 1
            records.append(
                {
                    "object_id": OBJECT_ID,
                    "contact_id": contact_id,
                    "role": contact_roles[contact_id],
                    "raw_microphone": microphone,
                    "raw_force": force,
                    "metadata": raw["records"][f"{OBJECT_ID}/audio/{contact_id}/metadata.yaml"],
                    "striking_force_metadata": raw["records"][f"{OBJECT_ID}/audio/{contact_id}/striking_force.yaml"],
                    "compact_microphone": compact_audio[contact_id],
                    "coordinate": coordinates[contact_id],
                }
            )

    passed = (
        raw["complete"]
        and identity_matches == len(CONTACT_IDS)
        and paired_headers == len(CONTACT_IDS)
        and tuple(sorted(compact_audio)) == CONTACT_IDS
        and tuple(sorted(coordinates)) == CONTACT_IDS
    )
    decision = "READY_FOR_C4_ESTIMATOR_FIT" if passed else "DATA_INSUFFICIENT_RAW_PREFIX"
    selected_raw_bytes = sum(record["bytes"] for record in raw["records"].values())
    compact_bytes = sum(record["bytes"] for record in compact_audio.values())
    counters = zero_counters(partition)
    counters.update(
        {
            "source_bytes_read": (
                OFFICIAL_PAGE_BYTES
                + RAW_PREFIX_BYTES
                + sum(spec["bytes"] for spec in SOURCE_SPECS.values())
            ),
            "raw_prefix_bytes_hashed": RAW_PREFIX_BYTES,
            "raw_prefix_compressed_bytes_consumed": raw["compressed_bytes_consumed"],
            "selected_raw_member_bytes_hashed": selected_raw_bytes,
            "compact_microphone_bytes_hashed": compact_bytes,
            "coordinate_values_decoded": len(coordinates) * 3,
            "point_cloud_values_decoded": 1024 * 3,
        }
    )
    report = {
        "schema": REPORT_SCHEMA,
        "study_id": STUDY_ID,
        "revision": REVISION,
        "stage": "inventory",
        "manifest_sha256": manifest_sha256,
        "implementation_sha256": manifest["implementation_sha256"],
        "decision": decision,
        "gate": {
            "passed": passed,
            "checks": {
                "raw_complete": raw["complete"],
                "raw_compact_microphone_identity_count": identity_matches,
                "paired_header_count": paired_headers,
                "expected_contact_count": len(CONTACT_IDS),
                "role_partition_complete": tuple(sorted(contact_roles)) == CONTACT_IDS,
                "official_split_absent": not any(split.values()),
                "point_cloud_shape": point_cloud["shape"] == [1024, 3],
                "scale_exact": scale == OBJECT_SCALE,
            },
        },
        "object": manifest["object"],
        "roles": manifest["roles"],
        "role_decode_authorized": {role: False for role in partition},
        "fit_role_available_for_separate_protocol": passed,
        "official_page": page,
        "source_identity": {
            "raw_prefix": manifest["raw_prefix"],
            "compact_sources": manifest["compact_sources"],
        },
        "raw_inventory": {
            "complete_contact_ids": raw["complete_contact_ids"],
            "archive_contact_order": raw["archive_contact_order"],
            "missing_members": raw["missing_members"],
            "truncated_prefix_observed": raw["truncated_prefix_observed"],
        },
        "point_cloud": point_cloud,
        "official_split_for_object": split,
        "records": records,
        "missing_axes": manifest["missing_axes"],
        "counters": counters,
        "next_authorized_step": "C4_ESTIMATOR_FIT_PROTOCOL" if passed else "STOP_DATA_INSUFFICIENT",
    }
    write_json(staging / "report.json", report)
    publish_output(staging, output)
    return output


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=("freeze", "preflight", "inventory"))
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--manifest", type=Path)
    parser.add_argument("--prefix-sha256")
    parser.add_argument("--official-page", type=Path)
    parser.add_argument("--raw-prefix", type=Path)
    parser.add_argument("--audio-archive", type=Path)
    parser.add_argument("--contacts-archive", type=Path)
    parser.add_argument("--point-cloud-archive", type=Path)
    parser.add_argument("--split-json", type=Path)
    parser.add_argument("--scale-json", type=Path)
    return parser.parse_args()


def required(value: Any, label: str) -> Any:
    if value is None:
        raise InventoryError(f"{label} is required for this stage")
    return value


def main() -> int:
    args = arguments()
    if args.stage == "freeze":
        output = freeze(required(args.prefix_sha256, "--prefix-sha256"), args.output)
    elif args.stage == "preflight":
        output = preflight(required(args.manifest, "--manifest"), args.output)
    else:
        output = inventory(
            required(args.manifest, "--manifest"),
            required(args.official_page, "--official-page"),
            required(args.raw_prefix, "--raw-prefix"),
            required(args.audio_archive, "--audio-archive"),
            required(args.contacts_archive, "--contacts-archive"),
            required(args.point_cloud_archive, "--point-cloud-archive"),
            required(args.split_json, "--split-json"),
            required(args.scale_json, "--scale-json"),
            args.output,
        )
    print(f"C3 {args.stage}: {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
