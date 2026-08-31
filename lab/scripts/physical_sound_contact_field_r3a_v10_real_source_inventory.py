#!/usr/bin/env python3
"""Freeze ObjectFolder Real V10 audio/contact/geometry roles without PCM decode."""

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
from typing import Any, Callable

import numpy as np

STUDY_ID = "physical-sound-contact-field-r3a-v10-real-source"
REVISION = "objectfolder-real-beer-glass-rinsing-cup-zero-decode-v1"
MANIFEST_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-real-source.manifest.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-r3a-v10-real-source.report.v1"

BENCHMARK_REPOSITORY = "https://github.com/objectfolder/contact-localization"
BENCHMARK_COMMIT = "4bb002f519cab9d250bbbe045a6df0248bf1639f"
BENCHMARK_DATA_PAGE = (
    "https://www.dropbox.com/scl/fo/rb5yj04v3gg42qtk88ddn/"
    "ACLiNBmuujZXKruWNNJ4Ihg?rlkey=vb1anldka1wve3vcwwz5cfe9h&dl=0"
)
OBJECTFOLDER_DOWNLOAD_PAGE = (
    "https://objectfolder.stanford.edu/objectfolder-real-download"
)
OBJECTFOLDER_PAPER = (
    "https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf"
)

SOURCES = {
    "audio": {
        "relative_name": "DATA_real/audio.tar.gz",
        "url": (
            "https://www.dropbox.com/scl/fo/rb5yj04v3gg42qtk88ddn/"
            "ANlP6MqnLWpGemH4Mv3S1IU/DATA_real/audio.tar.gz?"
            "dl=1&rlkey=vb1anldka1wve3vcwwz5cfe9h"
        ),
        "etag": "1721265166249979d",
        "bytes": 463_486_373,
        "sha256": "14a15b96dda7c3933a4a5829dc5ee21e1f9c2fe29f7c558925df24e87097a2c9",
    },
    "contacts": {
        "relative_name": "DATA_real/contacts.tar.gz",
        "url": (
            "https://www.dropbox.com/scl/fo/rb5yj04v3gg42qtk88ddn/"
            "AB3obslY4qW7IEqYDELaNrg/DATA_real/contacts.tar.gz?"
            "dl=1&rlkey=vb1anldka1wve3vcwwz5cfe9h"
        ),
        "etag": "1721265168037676d",
        "bytes": 121_961,
        "sha256": "310c45e11e40ceafc9c9fc1379059848ff664a7c2999fea5aabea479fc61eeee",
    },
    "point_cloud": {
        "relative_name": "DATA_real/global_gt_points.tar.gz",
        "url": (
            "https://www.dropbox.com/scl/fo/rb5yj04v3gg42qtk88ddn/"
            "AHWKp2-7cw7V_Szn65BDo6M/DATA_real/global_gt_points.tar.gz?"
            "dl=1&rlkey=vb1anldka1wve3vcwwz5cfe9h"
        ),
        "etag": "1721265170138227d",
        "bytes": 2_359_020,
        "sha256": "29ebf37bb88bc5c654b096f31552e4677fc156135930da72c82e306c5c517e51",
    },
    "split": {
        "relative_name": "DATA_real/split.json",
        "url": (
            "https://www.dropbox.com/scl/fo/rb5yj04v3gg42qtk88ddn/"
            "ANqBuganqUvVZX6Ki2BhLh0/DATA_real/split.json?"
            "dl=1&rlkey=vb1anldka1wve3vcwwz5cfe9h"
        ),
        "etag": "1721265176279297d",
        "bytes": 19_258,
        "sha256": "77ea2d99724197ad9e88c3ad90f49885c582dea6835fdc0ad533e5b9ce78674e",
    },
    "scale": {
        "relative_name": "DATA_real/scale.json",
        "url": (
            "https://www.dropbox.com/scl/fo/rb5yj04v3gg42qtk88ddn/"
            "AL-_QeJR39l7sg1X7N_uBIA/DATA_real/scale.json?"
            "dl=1&rlkey=vb1anldka1wve3vcwwz5cfe9h"
        ),
        "etag": "1721265174204746d",
        "bytes": 2_652,
        "sha256": "39c84eafabe999b1503710a9d601b0bc94fc6e358461aa2e5988b34b6b6d0b8a",
    },
}

RAW_ARCHIVE_LINEAGE = {
    "60": {
        "archive_url": (
            "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
            "audio_data_51_60.tar.gz"
        ),
        "archive_bytes": 34_360_300_077,
        "archive_etag": '"63e36c0f-80008922d"',
        "archive_last_modified": "Wed, 08 Feb 2023 09:31:59 GMT",
    },
    "22": {
        "archive_url": (
            "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
            "audio_data_21_30.tar.gz"
        ),
        "archive_bytes": 37_503_322_690,
        "archive_etag": '"63e36d00-8bb5f4a42"',
        "archive_last_modified": "Wed, 08 Feb 2023 09:36:00 GMT",
    },
}

EXPECTED_OFFICIAL_SPLITS = {
    "60": {
        "train": (0, 1, 2, 3, 5, 6, 7, 8, 10, 12, 14, 15, 17, 18, 19, 21, 22, 24, 26, 27, 28, 29),
        "val": (4, 11, 16, 23, 25),
        "test": (9, 13, 20),
    },
    "22": {
        "train": (0, 1, 2, 3, 4, 5, 7, 8, 9, 10, 11, 14, 15, 18, 20, 21, 22, 24, 25, 27, 28),
        "val": (6, 19, 23, 29),
        "test": (12, 13, 16, 17, 26),
    },
}

OBJECTS = {
    "60": {
        "name": "Beer_Glass",
        "material": "Glass",
        "scale": 0.1671128693615049,
        "roles": {
            "fit": EXPECTED_OFFICIAL_SPLITS["60"]["train"],
            "development": EXPECTED_OFFICIAL_SPLITS["60"]["val"],
            "exact_object_query": EXPECTED_OFFICIAL_SPLITS["60"]["test"],
        },
    },
    "22": {
        "name": "Rinsing_Cup",
        "material": "Glass",
        "scale": 0.11354385784440345,
        "roles": {
            "representation_holdout": EXPECTED_OFFICIAL_SPLITS["22"]["test"],
            "protected_unused": tuple(
                sorted(
                    set(range(30))
                    - set(EXPECTED_OFFICIAL_SPLITS["22"]["test"])
                )
            ),
        },
    },
}

EXPECTED_CONTACT_IDS = tuple(range(30))
EXPECTED_POINT_CLOUD_SHAPE = (1024, 3)
EXPECTED_NPY_DTYPE = "float64"
EXPECTED_WAV_HEADER = {
    "audio_format": 1,
    "channels": 1,
    "sample_rate_hz": 48_000,
    "sample_width_bytes": 2,
    "frame_count": 288_000,
}

RAW_IDENTITY_CONTROL = {
    "processed_path": "audio/91/18.wav",
    "raw_path": "91/audio/18/mic.wav",
    "bytes": 576_044,
    "sha256": "4cfbfee343d0a336f48f22168645b847d388a05c13142fd4688a36c52b02b5db",
}


class InventoryError(RuntimeError):
    """The frozen V10 source or role boundary was violated."""


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


def require_source(root: Path, path: Path, source: dict[str, Any]) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise InventoryError("source inputs must be external regular files")
    if resolved.stat().st_size != source["bytes"]:
        raise InventoryError(f"source byte count changed: {source['relative_name']}")
    if sha256_file(resolved) != source["sha256"]:
        raise InventoryError(f"source hash changed: {source['relative_name']}")
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
        raise InventoryError("selected WAV has invalid block alignment")
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


def parse_npy(payload: bytes, expected_shape: tuple[int, ...]) -> np.ndarray:
    try:
        values = np.load(io.BytesIO(payload), allow_pickle=False)
    except (ValueError, OSError) as error:
        raise InventoryError("selected NPY cannot be decoded") from error
    if values.shape != expected_shape or str(values.dtype) != EXPECTED_NPY_DTYPE:
        raise InventoryError("selected NPY shape or dtype changed")
    if not np.isfinite(values).all():
        raise InventoryError("selected NPY contains non-finite values")
    return values


def selected_audio_paths() -> set[str]:
    selected = {RAW_IDENTITY_CONTROL["processed_path"]}
    for object_id, description in OBJECTS.items():
        for role, contacts in description["roles"].items():
            if role == "protected_unused":
                continue
            selected.update(f"audio/{object_id}/{contact}.wav" for contact in contacts)
    return selected


def scan_tar(
    path: Path,
    selected: set[str],
    parser: Callable[[bytes, int], Any] | None = None,
) -> tuple[dict[str, dict[str, Any]], dict[str, set[int]]]:
    found: dict[str, dict[str, Any]] = {}
    observed_ids = {object_id: set() for object_id in OBJECTS}
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                parts = member.name.split("/")
                if len(parts) == 3 and parts[1] in observed_ids:
                    stem = Path(parts[2]).stem
                    if stem.isdecimal():
                        observed_ids[parts[1]].add(int(stem))
                if member.name not in selected:
                    continue
                if member.name in found or not member.isfile():
                    raise InventoryError(f"duplicate or invalid selected member: {member.name}")
                stream = archive.extractfile(member)
                if stream is None:
                    raise InventoryError(f"cannot read selected member: {member.name}")
                payload = stream.read()
                descriptor = parser(payload, member.size) if parser else None
                found[member.name] = {
                    "path": member.name,
                    "bytes": member.size,
                    "sha256": hashlib.sha256(payload).hexdigest(),
                    "descriptor": descriptor,
                }
    except (tarfile.TarError, EOFError, OSError) as error:
        raise InventoryError("cannot scan source tar archive") from error
    missing = sorted(selected - set(found))
    if missing:
        raise InventoryError(f"selected source members are missing: {missing}")
    return found, observed_ids


def parse_audio_member(payload: bytes, member_size: int) -> dict[str, int]:
    return parse_pcm_header(payload[:64], member_size)


def parse_contact_member(payload: bytes, member_size: int) -> dict[str, Any]:
    del member_size
    values = parse_npy(payload, (3,))
    return {"coordinate_m": [float(value) for value in values]}


def parse_point_cloud_member(payload: bytes, member_size: int) -> dict[str, Any]:
    del member_size
    values = parse_npy(payload, EXPECTED_POINT_CLOUD_SHAPE)
    return {
        "shape": list(values.shape),
        "dtype": str(values.dtype),
        "minimum_m": [float(value) for value in values.min(axis=0)],
        "maximum_m": [float(value) for value in values.max(axis=0)],
    }


def official_splits(split_path: Path) -> dict[str, dict[str, tuple[int, ...]]]:
    try:
        source = json.loads(split_path.read_text())
    except (UnicodeDecodeError, json.JSONDecodeError, OSError) as error:
        raise InventoryError("official split JSON cannot be decoded") from error
    result: dict[str, dict[str, tuple[int, ...]]] = {}
    for object_id in OBJECTS:
        result[object_id] = {}
        for role in ("train", "val", "test"):
            contacts = tuple(
                sorted(int(contact) for obj, contact in source[role] if str(obj) == object_id)
            )
            if contacts != EXPECTED_OFFICIAL_SPLITS[object_id][role]:
                raise InventoryError(f"official split changed for object {object_id}/{role}")
            result[object_id][role] = contacts
    return result


def validate_roles() -> None:
    object60_roles = OBJECTS["60"]["roles"]
    all_object60 = [contact for contacts in object60_roles.values() for contact in contacts]
    if tuple(sorted(all_object60)) != EXPECTED_CONTACT_IDS or len(set(all_object60)) != 30:
        raise InventoryError("object 60 roles are not a disjoint complete partition")
    object22_roles = OBJECTS["22"]["roles"]
    all_object22 = [contact for contacts in object22_roles.values() for contact in contacts]
    if tuple(sorted(all_object22)) != EXPECTED_CONTACT_IDS or len(set(all_object22)) != 30:
        raise InventoryError("object 22 roles are not a disjoint complete partition")


def validate_scale(scale_path: Path) -> dict[str, float]:
    try:
        values = json.loads(scale_path.read_text())
    except (UnicodeDecodeError, json.JSONDecodeError, OSError) as error:
        raise InventoryError("official scale JSON cannot be decoded") from error
    selected = {object_id: float(values[object_id]) for object_id in OBJECTS}
    expected = {object_id: description["scale"] for object_id, description in OBJECTS.items()}
    if selected != expected:
        raise InventoryError("official object scale changed")
    return selected


def contact_records(
    audio: dict[str, dict[str, Any]], contacts: dict[str, dict[str, Any]]
) -> list[dict[str, Any]]:
    records = []
    for object_id, description in OBJECTS.items():
        for role, contact_ids in description["roles"].items():
            for contact_id in contact_ids:
                contact_path = f"contacts/{object_id}/{contact_id}.npy"
                audio_path = f"audio/{object_id}/{contact_id}.wav"
                records.append(
                    {
                        "object_id": object_id,
                        "contact_id": contact_id,
                        "role": role,
                        "coordinate": contacts[contact_path],
                        "audio": audio.get(audio_path),
                        "audio_payload_committed": audio_path in audio,
                        "audio_sample_values_decoded": 0,
                    }
                )
    return records


def build_manifest(
    implementation_sha256: str,
    audio: dict[str, dict[str, Any]],
    contacts: dict[str, dict[str, Any]],
    points: dict[str, dict[str, Any]],
    splits: dict[str, dict[str, tuple[int, ...]]],
    scales: dict[str, float],
) -> dict[str, Any]:
    identity = audio[RAW_IDENTITY_CONTROL["processed_path"]]
    if identity["bytes"] != RAW_IDENTITY_CONTROL["bytes"] or identity["sha256"] != RAW_IDENTITY_CONTROL["sha256"]:
        raise InventoryError("processed/raw microphone identity control changed")
    objects = []
    for object_id, description in OBJECTS.items():
        objects.append(
            {
                "id": object_id,
                "name": description["name"],
                "published_material": description["material"],
                "scale": scales[object_id],
                "raw_archive_lineage": RAW_ARCHIVE_LINEAGE[object_id],
                "official_split": splits[object_id],
                "roles": description["roles"],
                "point_cloud": points[f"global_gt_points/{object_id}.npy"],
            }
        )
    return {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenZeroDecodeSourceAndRoles",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "primary_sources": {
            "benchmark_repository": BENCHMARK_REPOSITORY,
            "benchmark_commit": BENCHMARK_COMMIT,
            "benchmark_data_page": BENCHMARK_DATA_PAGE,
            "objectfolder_download_page": OBJECTFOLDER_DOWNLOAD_PAGE,
            "paper": OBJECTFOLDER_PAPER,
            "archives": SOURCES,
        },
        "signal_semantics": "recorded_impact_microphone_waveform",
        "excitation_axis": "published_in_raw_source_but_absent_from_selected_processed_bundle",
        "listener_condition": "dataset_fixed_microphone_geometry_unpublished",
        "coordinate_semantics": "official_contact_localization_ground_truth_xyz_m",
        "coordinate_audio_binding": "same_official_object_and_contact_key",
        "objects": objects,
        "contacts": contact_records(audio, contacts),
        "raw_identity_control": {**RAW_IDENTITY_CONTROL, "observed": identity},
        "implementation_sha256": implementation_sha256,
        "waveform_sample_values_decoded": 0,
        "fit_waveform_sample_values_decoded": 0,
        "development_waveform_sample_values_decoded": 0,
        "exact_object_query_waveform_sample_values_decoded": 0,
        "representation_holdout_waveform_sample_values_decoded": 0,
        "method_holdout_waveform_sample_values_decoded": 0,
        "admission_shadow_waveform_sample_values_decoded": 0,
        "selected_audio_payloads_emitted": 0,
        "next_authorized_role": "fit",
        "development_authorized": False,
        "exact_object_query_authorized": False,
        "representation_holdout_authorized": False,
        "r3b_authorized": False,
        "runtime_neural_inference_authorized": False,
        "authored_clip_fallback_required": True,
    }


def run(
    root: Path,
    audio_argument: Path,
    contacts_argument: Path,
    points_argument: Path,
    split_argument: Path,
    scale_argument: Path,
    output_argument: Path,
) -> Path:
    validate_roles()
    audio_path = require_source(root, audio_argument, SOURCES["audio"])
    contacts_path = require_source(root, contacts_argument, SOURCES["contacts"])
    points_path = require_source(root, points_argument, SOURCES["point_cloud"])
    split_path = require_source(root, split_argument, SOURCES["split"])
    scale_path = require_source(root, scale_argument, SOURCES["scale"])

    audio, audio_ids = scan_tar(audio_path, selected_audio_paths(), parse_audio_member)
    contact_paths = {
        f"contacts/{object_id}/{contact_id}.npy"
        for object_id in OBJECTS
        for contact_id in EXPECTED_CONTACT_IDS
    }
    contacts, contact_ids = scan_tar(contacts_path, contact_paths, parse_contact_member)
    point_paths = {f"global_gt_points/{object_id}.npy" for object_id in OBJECTS}
    points, _ = scan_tar(points_path, point_paths, parse_point_cloud_member)
    for object_id in OBJECTS:
        if tuple(sorted(audio_ids[object_id])) != EXPECTED_CONTACT_IDS:
            raise InventoryError(f"audio contact set changed for object {object_id}")
        if tuple(sorted(contact_ids[object_id])) != EXPECTED_CONTACT_IDS:
            raise InventoryError(f"coordinate contact set changed for object {object_id}")
    splits = official_splits(split_path)
    scales = validate_scale(scale_path)
    manifest = build_manifest(
        sha256_file(Path(__file__).resolve()), audio, contacts, points, splits, scales
    )

    output, staging = prepare_output(root, output_argument)
    try:
        manifest_bytes = canonical_json(manifest)
        (staging / "manifest.json").write_bytes(manifest_bytes)
        records = manifest["contacts"]
        report = {
            "schema": REPORT_SCHEMA,
            "status": "Validated",
            "decision": "READY_FOR_V9_REAL_FIT",
            "manifest_sha256": hashlib.sha256(manifest_bytes).hexdigest(),
            "object_count": len(OBJECTS),
            "coordinate_record_count": len(records),
            "committed_audio_record_count": sum(
                record["audio_payload_committed"] for record in records
            ),
            "fit_contact_count": sum(record["role"] == "fit" for record in records),
            "development_contact_count": sum(
                record["role"] == "development" for record in records
            ),
            "exact_object_query_contact_count": sum(
                record["role"] == "exact_object_query" for record in records
            ),
            "representation_holdout_contact_count": sum(
                record["role"] == "representation_holdout" for record in records
            ),
            "processed_raw_identity_control_passed": True,
            "coordinate_audio_id_sets_equal": True,
            "point_clouds_validated": len(points),
            "waveform_headers_validated": len(audio),
            "waveform_sample_values_decoded": 0,
            "fit_waveform_sample_values_decoded": 0,
            "development_waveform_sample_values_decoded": 0,
            "exact_object_query_waveform_sample_values_decoded": 0,
            "representation_holdout_waveform_sample_values_decoded": 0,
            "method_holdout_waveform_sample_values_decoded": 0,
            "admission_shadow_waveform_sample_values_decoded": 0,
            "selected_audio_payloads_emitted": 0,
            "next_authorized_step": "IMPLEMENT_AND_RUN_V9_REAL_FIT_ONLY",
            "next_authorized_role": "fit",
            "development_authorized": False,
            "representation_holdout_authorized": False,
            "exact_object_query_authorized": False,
            "real_quality_credit": False,
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
    parser.add_argument("--audio-archive", required=True, type=Path)
    parser.add_argument("--contacts-archive", required=True, type=Path)
    parser.add_argument("--point-cloud-archive", required=True, type=Path)
    parser.add_argument("--split-json", required=True, type=Path)
    parser.add_argument("--scale-json", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> None:
    arguments = parse_arguments()
    output = run(
        repository_root(),
        arguments.audio_archive,
        arguments.contacts_archive,
        arguments.point_cloud_archive,
        arguments.split_json,
        arguments.scale_json,
        arguments.output,
    )
    report = json.loads((output / "report.json").read_bytes())
    print(f"R3A V10 real-source inventory: {output}")
    print(f"decision: {report['decision']}")
    print(f"manifest sha256: {sha256_file(output / 'manifest.json')}")
    print(f"report sha256: {sha256_file(output / 'report.json')}")


if __name__ == "__main__":
    main()
