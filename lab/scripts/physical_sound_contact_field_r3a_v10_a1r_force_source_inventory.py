#!/usr/bin/env python3
"""Freeze an ObjectFolder object-51 raw-force source without PCM decode."""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import os
import platform
import shutil
import struct
import tarfile
from pathlib import Path
from typing import Any

import numpy as np

import physical_sound_contact_field_r3a_v10_real_source_inventory as compact

STUDY_ID = "physical-sound-contact-field-r3a-v10-a1r-force-source"
REVISION = "objectfolder-real-object51-raw-force-zero-decode-v1"
MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v10-a1r-force-source.manifest.v1"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-r3a-v10-a1r-force-source.report.v1"
)

OFFICIAL_SITE_REPOSITORY = "https://github.com/objectfolder/objectfolder.github.io"
OFFICIAL_SITE_COMMIT = "d058ba09e7e7a1a8358d64d7b48e5f588b377eb8"
OFFICIAL_PAGE_URL = (
    "https://raw.githubusercontent.com/objectfolder/objectfolder.github.io/"
    f"{OFFICIAL_SITE_COMMIT}/source/_pages/objectfolder-real-download.md"
)
OFFICIAL_PAGE_BYTES = 8_668
OFFICIAL_PAGE_SHA256 = (
    "2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32"
)

RAW_ARCHIVE_URL = (
    "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
    "audio_data_51_60.tar.gz"
)
RAW_ARCHIVE_BYTES = 34_360_300_077
RAW_ARCHIVE_ETAG = '"63e36c0f-80008922d"'
RAW_ARCHIVE_LAST_MODIFIED = "Wed, 08 Feb 2023 09:31:59 GMT"
RAW_PREFIX_RANGE = "bytes=0-536870911"
RAW_PREFIX_BYTES = 536_870_912
RAW_PREFIX_SHA256 = (
    "f58e242af7b1b9f500b86e438f2c90ad6cb3f18d29914e1431c69c6a20c78584"
)

OBJECT_ID = "51"
OBJECT_NAME = "Fruit_Bowl"
PUBLISHED_MATERIAL = "Glass"
OBJECT_SCALE = 0.29946099617930483
CONTACTS = (
    {"contact_id": 27, "role": "fit", "archive_order": 1},
    {"contact_id": 15, "role": "fit", "archive_order": 2},
    {"contact_id": 4, "role": "fit", "archive_order": 3},
    {"contact_id": 3, "role": "fit", "archive_order": 4},
    {"contact_id": 9, "role": "development", "archive_order": 5},
    {"contact_id": 18, "role": "sealed", "archive_order": 6},
)
EXPECTED_ARCHIVE_CONTACT_ORDER = tuple(item["contact_id"] for item in CONTACTS)
RAW_MEMBER_NAMES = ("mic.wav", "Force.wav", "metadata.yaml", "striking_force.yaml")

EXPECTED_RAW_COMMITMENTS = {
    "51/audio/27/mic.wav": (576_044, "c4b5468b8c58df99a9bfbfb82f1ed2ed378843db3105c32b129828573f0f03f3"),
    "51/audio/27/Force.wav": (576_044, "79d417c5d2a60e50540ba2b6204692178438d45eec1dc71988e231cf44cf62b6"),
    "51/audio/27/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "51/audio/27/striking_force.yaml": (31, "f6007b6d57f0ab6b7d6d6daa383560897c3eb27d693c8f82b00d0bc8b4e9a487"),
    "51/audio/15/mic.wav": (576_044, "59b81b6fb78ddbb9b33b75afd38df5a08f8611cb44e724eb583f061563480ff2"),
    "51/audio/15/Force.wav": (576_044, "4a17a4c20461906b0feedd55123fc3e3df5be4419fd43f417c1f4b5220473f9c"),
    "51/audio/15/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "51/audio/15/striking_force.yaml": (31, "3968729b898e14f707ad787697285574f8504539468206d6d4631d88a5b132ba"),
    "51/audio/4/mic.wav": (576_044, "319498af2461317973ea576bc1a0e36ddd9db25ac068228699370b705a7ca635"),
    "51/audio/4/Force.wav": (576_044, "79268387e41157a4fe5b37e1bd2377b23b5e59c2f1a0a512efe512b07fdbd54f"),
    "51/audio/4/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "51/audio/4/striking_force.yaml": (31, "d1cce1a9e4f82c03e21e603a6e5dcaa56bc2eb4af81440da577c80ecc7c4e616"),
    "51/audio/3/mic.wav": (576_044, "ce2415a9ee31c65f1726c02e1b82408f46b4e412ae22e2c9aef748a8a423d66b"),
    "51/audio/3/Force.wav": (576_044, "2ba80a5dc03b82ab11ca961ab94727d67c20c43b58da2c1471c419f8f28746ac"),
    "51/audio/3/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "51/audio/3/striking_force.yaml": (31, "47b2b3bc1282414141644a61843a882f3b247183e1abb7f6665106c68a1211fe"),
    "51/audio/9/mic.wav": (576_044, "445b4d0e10da493c318f3e95accff3c508d64bfd3331c287e698b9ec1c226015"),
    "51/audio/9/Force.wav": (576_044, "9403c4d38dfc510709547e4015aec83173da9731f10651b6cb3a6c2ce2ff92a9"),
    "51/audio/9/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "51/audio/9/striking_force.yaml": (31, "b659d82f9b4eaf3c2222c327356ebe60f890ac7daadb85cb8f7dad7cd052b421"),
    "51/audio/18/mic.wav": (576_044, "68689e34ea9a13a144e9d41da2f4f17af5e64f5733f05ad3f7135d60f68b3455"),
    "51/audio/18/Force.wav": (576_044, "6afc43aacbf7851b928c34a9f2bf2dcbc6428bc0c3c2a4140456496093973dd8"),
    "51/audio/18/metadata.yaml": (35, "8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd"),
    "51/audio/18/striking_force.yaml": (31, "ce6d940629f83a90ba6d038fa31a04c33a42da04fe9057b75e632ed87790be2e"),
}

EXPECTED_COORDINATES = {
    27: ("0b199aa558a731bd29a7c7ec9169254ef0884b7e93e2f27dd81fef5943aca396", (-0.10525473525264488, -0.07879972416921165, -0.020209989516206506)),
    15: ("3e767da95dfd0f2aa642f34a00144b1fcc8c37a79a6978e185933217e25c54ec", (0.12244503716057938, 0.03874234592587659, 0.02893196559564226)),
    4: ("03ade32b92c240170a10ac64c92489d8f67857c0ab0546228cd1338f18e9dd4b", (0.10401916982306234, -0.0021197406984874156, -0.025596647844167597)),
    3: ("132e696e9c628043d5afdd8b6fbd7ef1c30c036694d24a5c7ab0995515e7a044", (0.08556405186630008, -0.08039852199562347, -0.020197105338899735)),
    9: ("b83b89140ce1a835d87fcdf4b04f39799e28c601c8f0c3c9e0c22dda83835e31", (0.047301021881052466, 0.13547959091340878, 0.00010780326461068412)),
    18: ("df05951c11430b314678fbcace2d5fab63314c2c44923e19d9b9b82f50bdf987", (0.02129903220885626, 0.1263078714173106, 0.022248554137510035)),
}
EXPECTED_POINT_CLOUD = {
    "bytes": 24_704,
    "sha256": "5c3b25eed3d92bbd85653e4e7540f539fa6bb80bbe71e834b5b9050f74b1d41d",
    "shape": (1024, 3),
    "dtype": "float64",
}
EXPECTED_WAV_HEADER = compact.EXPECTED_WAV_HEADER


class InventoryError(RuntimeError):
    """The frozen V10 A1R source boundary was violated."""


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

    return (
        json.dumps(ready(value), indent=2, sort_keys=True, allow_nan=False) + "\n"
    ).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def require_external_file(root: Path, path: Path, label: str) -> Path:
    resolved = path.resolve(strict=True)
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
        raise InventoryError("A1R inventory output must stay outside the repository")
    if resolved.exists():
        raise InventoryError("A1R inventory output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        shutil.rmtree(staging)
    staging.mkdir()
    return resolved, staging


def parse_pcm_header(header: bytes, member_size: int) -> dict[str, int]:
    try:
        return compact.parse_pcm_header(header, member_size)
    except compact.InventoryError as error:
        raise InventoryError(str(error)) from error


def validate_official_page(path: Path) -> dict[str, Any]:
    try:
        text = path.read_text()
    except (OSError, UnicodeDecodeError) as error:
        raise InventoryError("official page is not UTF-8 text") from error
    required = (
        "ground-truth contact force profile",
        "coordinate of the striking location on the object mesh",
        "|  51   |     Fruit_Bowl     |     Glass",
        "audio_data_[X+1]_[X+10].tar.gz",
    )
    if any(fragment not in text for fragment in required):
        raise InventoryError("official page semantics changed")
    return {
        "repository": OFFICIAL_SITE_REPOSITORY,
        "commit": OFFICIAL_SITE_COMMIT,
        "url": OFFICIAL_PAGE_URL,
        "bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def scan_raw_prefix(
    prefix: Path,
    expected: dict[str, tuple[int, str]] = EXPECTED_RAW_COMMITMENTS,
    expected_order: tuple[int, ...] = EXPECTED_ARCHIVE_CONTACT_ORDER,
) -> tuple[dict[str, dict[str, Any]], list[int]]:
    found: dict[str, dict[str, Any]] = {}
    contact_order: list[int] = []
    try:
        with prefix.open("rb") as raw, tarfile.open(fileobj=raw, mode="r|gz") as archive:
            for member in archive:
                parts = member.name.split("/")
                if len(parts) >= 3 and parts[:2] == [OBJECT_ID, "audio"]:
                    if parts[2].isdecimal():
                        contact_id = int(parts[2])
                        if contact_id not in contact_order:
                            contact_order.append(contact_id)
                if member.name not in expected:
                    continue
                if member.name in found or not member.isfile():
                    raise InventoryError(f"duplicate or invalid raw member: {member.name}")
                stream = archive.extractfile(member)
                if stream is None:
                    raise InventoryError(f"cannot read raw member: {member.name}")
                payload = stream.read()
                expected_bytes, expected_sha256 = expected[member.name]
                if (
                    len(payload) != expected_bytes
                    or member.size != expected_bytes
                    or sha256_bytes(payload) != expected_sha256
                ):
                    raise InventoryError(f"raw member commitment changed: {member.name}")
                descriptor = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": expected_sha256,
                    "wav_header": None,
                }
                if member.name.endswith(".wav"):
                    descriptor["wav_header"] = parse_pcm_header(
                        payload[:64], len(payload)
                    )
                found[member.name] = descriptor
                if len(found) == len(expected):
                    break
    except (tarfile.TarError, EOFError, OSError) as error:
        raise InventoryError("cannot scan bounded raw-force prefix") from error
    missing = sorted(set(expected) - set(found))
    if missing:
        raise InventoryError(f"raw-force members are missing: {missing}")
    observed_order = contact_order[: len(expected_order)]
    if tuple(observed_order) != expected_order:
        raise InventoryError("raw archive contact order changed")
    return found, observed_order


def scan_processed_audio(
    path: Path,
) -> tuple[dict[int, dict[str, Any]], tuple[int, ...]]:
    selected_ids = {item["contact_id"] for item in CONTACTS}
    found: dict[int, dict[str, Any]] = {}
    observed_ids: set[int] = set()
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                parts = member.name.split("/")
                if len(parts) != 3 or parts[:2] != ["audio", OBJECT_ID]:
                    continue
                stem = Path(parts[2]).stem
                if not stem.isdecimal():
                    continue
                contact_id = int(stem)
                observed_ids.add(contact_id)
                if contact_id not in selected_ids:
                    continue
                if contact_id in found or not member.isfile():
                    raise InventoryError("duplicate or invalid processed audio member")
                stream = archive.extractfile(member)
                if stream is None:
                    raise InventoryError("cannot read processed audio member")
                payload = stream.read()
                raw_path = f"{OBJECT_ID}/audio/{contact_id}/mic.wav"
                expected_bytes, expected_sha256 = EXPECTED_RAW_COMMITMENTS[raw_path]
                if (
                    len(payload) != expected_bytes
                    or sha256_bytes(payload) != expected_sha256
                ):
                    raise InventoryError("processed/raw microphone identity changed")
                found[contact_id] = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": expected_sha256,
                    "wav_header": parse_pcm_header(payload[:64], len(payload)),
                    "raw_microphone_byte_identity": True,
                }
    except (tarfile.TarError, EOFError, OSError) as error:
        raise InventoryError("cannot scan processed audio source") from error
    if set(found) != selected_ids or tuple(sorted(observed_ids)) != tuple(range(32)):
        raise InventoryError("processed object-51 audio contact set changed")
    return found, tuple(sorted(observed_ids))


def scan_coordinates(
    path: Path,
) -> tuple[dict[int, dict[str, Any]], tuple[int, ...]]:
    selected_ids = {item["contact_id"] for item in CONTACTS}
    found: dict[int, dict[str, Any]] = {}
    observed_ids: set[int] = set()
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                parts = member.name.split("/")
                if len(parts) != 3 or parts[:2] != ["contacts", OBJECT_ID]:
                    continue
                stem = Path(parts[2]).stem
                if not stem.isdecimal():
                    continue
                contact_id = int(stem)
                observed_ids.add(contact_id)
                if contact_id not in selected_ids:
                    continue
                if contact_id in found or not member.isfile():
                    raise InventoryError("duplicate or invalid coordinate member")
                stream = archive.extractfile(member)
                if stream is None:
                    raise InventoryError("cannot read coordinate member")
                payload = stream.read()
                expected_sha256, expected_coordinate = EXPECTED_COORDINATES[contact_id]
                if len(payload) != 152 or sha256_bytes(payload) != expected_sha256:
                    raise InventoryError("coordinate commitment changed")
                try:
                    value = np.load(io.BytesIO(payload), allow_pickle=False)
                except (ValueError, OSError) as error:
                    raise InventoryError("coordinate NPY cannot be decoded") from error
                if (
                    value.shape != (3,)
                    or str(value.dtype) != "float64"
                    or not np.array_equal(value, np.asarray(expected_coordinate))
                ):
                    raise InventoryError("coordinate value changed")
                found[contact_id] = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": expected_sha256,
                    "coordinate_m": value.tolist(),
                }
    except (tarfile.TarError, EOFError, OSError) as error:
        raise InventoryError("cannot scan coordinate source") from error
    if set(found) != selected_ids or tuple(sorted(observed_ids)) != tuple(range(32)):
        raise InventoryError("processed object-51 coordinate contact set changed")
    return found, tuple(sorted(observed_ids))


def scan_point_cloud(path: Path) -> tuple[dict[str, Any], np.ndarray]:
    member_name = f"global_gt_points/{OBJECT_ID}.npy"
    try:
        with tarfile.open(path, "r:gz") as archive:
            member = archive.getmember(member_name)
            stream = archive.extractfile(member)
            if stream is None:
                raise InventoryError("cannot read point-cloud member")
            payload = stream.read()
    except (tarfile.TarError, KeyError, EOFError, OSError) as error:
        raise InventoryError("cannot scan point-cloud source") from error
    if (
        len(payload) != EXPECTED_POINT_CLOUD["bytes"]
        or sha256_bytes(payload) != EXPECTED_POINT_CLOUD["sha256"]
    ):
        raise InventoryError("point-cloud commitment changed")
    try:
        values = np.load(io.BytesIO(payload), allow_pickle=False)
    except (ValueError, OSError) as error:
        raise InventoryError("point-cloud NPY cannot be decoded") from error
    if (
        values.shape != EXPECTED_POINT_CLOUD["shape"]
        or str(values.dtype) != EXPECTED_POINT_CLOUD["dtype"]
        or not np.isfinite(values).all()
    ):
        raise InventoryError("point-cloud shape, dtype or finiteness changed")
    return {
        "path": member_name,
        "bytes": len(payload),
        "sha256": EXPECTED_POINT_CLOUD["sha256"],
        "shape": list(values.shape),
        "dtype": str(values.dtype),
        "minimum_m": values.min(axis=0).tolist(),
        "maximum_m": values.max(axis=0).tolist(),
    }, values


def validate_scale_and_split(scale_path: Path, split_path: Path) -> dict[str, Any]:
    try:
        scales = json.loads(scale_path.read_text())
        split = json.loads(split_path.read_text())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise InventoryError("scale/split JSON cannot be decoded") from error
    if float(scales.get(OBJECT_ID, -1.0)) != OBJECT_SCALE:
        raise InventoryError("object-51 scale changed")
    listed = [row for role in ("train", "val", "test") for row in split[role] if row[0] == OBJECT_ID]
    if listed:
        raise InventoryError("object 51 unexpectedly entered the benchmark split")
    return {
        "scale": OBJECT_SCALE,
        "official_contact_localization_split_role": "absent",
        "role_selection": "first_six_complete_raw_contacts_in_archive_order",
    }


def contact_records(
    raw: dict[str, dict[str, Any]],
    processed: dict[int, dict[str, Any]],
    coordinates: dict[int, dict[str, Any]],
    points: np.ndarray,
) -> list[dict[str, Any]]:
    records = []
    for contact in CONTACTS:
        contact_id = contact["contact_id"]
        coordinate = np.asarray(coordinates[contact_id]["coordinate_m"])
        distance = np.sqrt(np.sum(np.square(points - coordinate), axis=1))
        raw_members = [
            raw[f"{OBJECT_ID}/audio/{contact_id}/{name}"] for name in RAW_MEMBER_NAMES
        ]
        if raw_members[0]["sha256"] != processed[contact_id]["sha256"]:
            raise InventoryError("raw/processed microphone identity is inconsistent")
        records.append(
            {
                **contact,
                "raw_members": raw_members,
                "processed_audio": processed[contact_id],
                "coordinate": coordinates[contact_id],
                "nearest_sampled_point_distance_m": float(distance.min()),
                "nearest_sampled_point_index": int(distance.argmin()),
                "microphone_sample_values_decoded": 0,
                "force_sample_values_decoded": 0,
            }
        )
    return records


def build_manifest(
    official_page: dict[str, Any],
    contacts: list[dict[str, Any]],
    point_cloud: dict[str, Any],
    compact_sources: dict[str, dict[str, Any]],
    implementation_sha256: str,
) -> dict[str, Any]:
    return {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenZeroDecodeForceSourceAndRoles",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "official_page": official_page,
        "raw_source": {
            "archive_url": RAW_ARCHIVE_URL,
            "archive_bytes": RAW_ARCHIVE_BYTES,
            "archive_etag": RAW_ARCHIVE_ETAG,
            "archive_last_modified": RAW_ARCHIVE_LAST_MODIFIED,
            "prefix_range": RAW_PREFIX_RANGE,
            "prefix_bytes": RAW_PREFIX_BYTES,
            "prefix_sha256": RAW_PREFIX_SHA256,
        },
        "compact_sources": compact_sources,
        "object": {
            "id": OBJECT_ID,
            "name": OBJECT_NAME,
            "published_material": PUBLISHED_MATERIAL,
            "scale": OBJECT_SCALE,
            "point_cloud": point_cloud,
            "signal_semantics": "recorded_impact_microphone_plus_synchronized_measured_force",
            "coordinate_semantics": "official_contact_localization_ground_truth_xyz_m",
            "listener_condition": "dataset_fixed_microphone_geometry_unpublished",
            "object_disjoint_from_v10_a1": True,
            "project_disjoint_from_v10_a1": False,
            "archive_disjoint_from_v10_a1": False,
        },
        "role_selection": {
            "basis": "first_six_complete_raw_contacts_in_archive_order_before_pcm_decode",
            "fit": [27, 15, 4, 3],
            "development": [9],
            "sealed": [18],
            "benchmark_split_role": "absent",
        },
        "contacts": contacts,
        "implementation_sha256": implementation_sha256,
        "microphone_sample_values_decoded": 0,
        "force_sample_values_decoded": 0,
        "fit_microphone_sample_values_decoded": 0,
        "fit_force_sample_values_decoded": 0,
        "development_microphone_sample_values_decoded": 0,
        "development_force_sample_values_decoded": 0,
        "sealed_microphone_sample_values_decoded": 0,
        "sealed_force_sample_values_decoded": 0,
        "selected_payloads_emitted": 0,
        "next_authorized_role": "fit",
        "development_authorized": False,
        "sealed_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def run(
    root: Path,
    prefix_argument: Path,
    page_argument: Path,
    audio_argument: Path,
    contacts_argument: Path,
    points_argument: Path,
    split_argument: Path,
    scale_argument: Path,
    output_argument: Path,
) -> Path:
    prefix = require_exact_file(
        root, prefix_argument, "raw-force prefix", RAW_PREFIX_BYTES, RAW_PREFIX_SHA256
    )
    page = require_exact_file(
        root,
        page_argument,
        "official source page",
        OFFICIAL_PAGE_BYTES,
        OFFICIAL_PAGE_SHA256,
    )
    arguments = {
        "audio": audio_argument,
        "contacts": contacts_argument,
        "point_cloud": points_argument,
        "split": split_argument,
        "scale": scale_argument,
    }
    compact_paths = {}
    compact_sources = {}
    for key, argument in arguments.items():
        source = compact.SOURCES[key]
        compact_paths[key] = require_exact_file(
            root,
            argument,
            f"compact {key}",
            source["bytes"],
            source["sha256"],
        )
        compact_sources[key] = source

    official_page = validate_official_page(page)
    raw, archive_order = scan_raw_prefix(prefix)
    processed, audio_ids = scan_processed_audio(compact_paths["audio"])
    coordinates, coordinate_ids = scan_coordinates(compact_paths["contacts"])
    point_cloud, points = scan_point_cloud(compact_paths["point_cloud"])
    split_scale = validate_scale_and_split(
        compact_paths["scale"], compact_paths["split"]
    )
    contacts = contact_records(raw, processed, coordinates, points)
    implementation_sha256 = sha256_file(Path(__file__).resolve())
    manifest = build_manifest(
        official_page,
        contacts,
        point_cloud,
        compact_sources,
        implementation_sha256,
    )

    output, staging = prepare_output(root, output_argument)
    try:
        manifest_bytes = canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": "READY_FOR_A1R_FORCE_ONSET_FIT",
            "manifest_sha256": sha256_bytes(manifest_bytes),
            "object_id": OBJECT_ID,
            "object_name": OBJECT_NAME,
            "published_material": PUBLISHED_MATERIAL,
            "archive_contact_order": archive_order,
            "selected_contact_count": len(CONTACTS),
            "fit_contact_count": 4,
            "development_contact_count": 1,
            "sealed_contact_count": 1,
            "selected_raw_member_count": len(raw),
            "selected_raw_member_bytes_hashed": sum(
                member["bytes"] for record in contacts for member in record["raw_members"]
            ),
            "raw_processed_microphone_identity_count": sum(
                record["processed_audio"]["raw_microphone_byte_identity"]
                for record in contacts
            ),
            "wav_headers_validated": sum(
                member["wav_header"] is not None
                for record in contacts
                for member in record["raw_members"]
            )
            + len(processed),
            "object_audio_contact_ids": audio_ids,
            "object_coordinate_contact_ids": coordinate_ids,
            "split_and_scale": split_scale,
            "microphone_sample_values_decoded": 0,
            "force_sample_values_decoded": 0,
            "development_microphone_sample_values_decoded": 0,
            "development_force_sample_values_decoded": 0,
            "sealed_microphone_sample_values_decoded": 0,
            "sealed_force_sample_values_decoded": 0,
            "unselected_member_payloads_hashed": 0,
            "selected_payloads_emitted": 0,
            "real_quality_credit": False,
            "next_authorized_step": "IMPLEMENT_FROZEN_A1R_FORCE_ONSET_FIT_ONLY_RUNNER",
            "development_authorized": False,
            "sealed_authorized": False,
            "r3b_authorized": False,
            "runtime_or_public_contract_changed": False,
            "authored_clip_fallback_required": True,
            "environment": {
                "python": platform.python_version(),
                "python_implementation": platform.python_implementation(),
                "numpy": np.__version__,
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
    parser.add_argument("--official-page", required=True, type=Path)
    parser.add_argument("--audio", required=True, type=Path)
    parser.add_argument("--contacts", required=True, type=Path)
    parser.add_argument("--point-cloud", required=True, type=Path)
    parser.add_argument("--split", required=True, type=Path)
    parser.add_argument("--scale", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    output = run(
        repository_root(),
        arguments.archive_prefix,
        arguments.official_page,
        arguments.audio,
        arguments.contacts,
        arguments.point_cloud,
        arguments.split,
        arguments.scale,
        arguments.output,
    )
    report = json.loads((output / "report.json").read_bytes())
    print(f"R3A V10 A1R force-source inventory: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
