#!/usr/bin/env python3
"""Freeze ObjectFolder object-92 sources and six roles without PCM decode."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import math
import os
import shutil
import struct
import tarfile
from pathlib import Path
from typing import Any

STUDY_ID = "physical-sound-v13-m2a-object92-source-role-freeze"
REVISION = "objectfolder-real-92-canonical-impact-source-role-v0"
MANIFEST_SCHEMA = (
    "nextengine.experimental-physical-sound-object-source-role-freeze.v0"
)
REPORT_SCHEMA = (
    "nextengine.experimental-physical-sound-object-source-role-freeze.report.v0"
)

OBJECT_ID = "92"
OBJECT_NAME = "Glass_Red"
PUBLISHED_MATERIAL = "Glass"
OBJECT_SCALE = 0.3435394121395228
EXPECTED_CONTACT_IDS = tuple(range(36))
EXPECTED_WAV_BYTES = 576_044
EXPECTED_WAV_HEADER = {
    "audio_format": 1,
    "channels": 1,
    "sample_rate_hz": 48_000,
    "sample_width_bytes": 2,
    "frame_count": 288_000,
}
EXPECTED_POINT_SHAPE = (1024, 3)

SHORTLIST_FRESH = (
    ("59", "Soap_Dish", "Glass"),
    ("82", "Can", "Glass"),
    ("92", "Glass_Red", "Glass"),
    ("93", "Vase", "Glass"),
)

PUBLISHER_SPLIT = {
    "train": (
        3,
        20,
        18,
        15,
        27,
        4,
        34,
        30,
        0,
        23,
        29,
        16,
        24,
        7,
        5,
        14,
        26,
        21,
        19,
        32,
        8,
        6,
        17,
        25,
        10,
        1,
    ),
    "val": (9, 11, 13),
    "test": (33, 12, 35, 2, 28, 22, 31),
}

ROLE_SEPARATOR = "nextengine-physical-sound-v13-m2a-object92-role-v0"
EXPECTED_ROLES = {
    "formula_fit": (8, 32, 19, 26, 16, 21, 29, 1),
    "generator_development": (10, 23, 6, 3, 34, 15),
    "representation_holdout": (0, 4, 5, 7, 14, 17, 18, 20, 24, 25, 27, 30),
    "validator_calibration": (9, 11, 13),
    "validator_method_holdout": (2, 12, 31, 33),
    "admission_shadow": (22, 28, 35),
}

SOURCES = {
    "shortlist": {
        "relative_name": "M1c/shortlist.json",
        "bytes": 3_375,
        "sha256": "0bd87fc3e62bfa6e574c4c206e795282223a4ecbd6221790e1ff4c60a113f6d7",
    },
    "audio": {
        "relative_name": "DATA_real/audio.tar.gz",
        "bytes": 463_486_373,
        "sha256": "14a15b96dda7c3933a4a5829dc5ee21e1f9c2fe29f7c558925df24e87097a2c9",
    },
    "contacts": {
        "relative_name": "DATA_real/contacts.tar.gz",
        "bytes": 121_961,
        "sha256": "310c45e11e40ceafc9c9fc1379059848ff664a7c2999fea5aabea479fc61eeee",
    },
    "point_cloud": {
        "relative_name": "DATA_real/global_gt_points.tar.gz",
        "bytes": 2_359_020,
        "sha256": "29ebf37bb88bc5c654b096f31552e4677fc156135930da72c82e306c5c517e51",
    },
    "split": {
        "relative_name": "DATA_real/split.json",
        "bytes": 19_258,
        "sha256": "77ea2d99724197ad9e88c3ad90f49885c582dea6835fdc0ad533e5b9ce78674e",
    },
    "scale": {
        "relative_name": "DATA_real/scale.json",
        "bytes": 2_652,
        "sha256": "39c84eafabe999b1503710a9d601b0bc94fc6e358461aa2e5988b34b6b6d0b8a",
    },
}

RAW_ARCHIVE = {
    "url": (
        "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
        "audio_data_91_100.tar.gz"
    ),
    "bytes": 38_866_981_268,
    "etag": '"63e3844c-90ca71194"',
    "last_modified": "Wed, 08 Feb 2023 11:15:24 GMT",
    "range_support": "bytes",
    "m2b_prefix_increment_bytes": 1_073_741_824,
    "m2b_prefix_ceiling_bytes": 12_884_901_888,
}


class FreezeError(RuntimeError):
    """The preregistered M2a source, role or access boundary was violated."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        encoded = json.dumps(value, indent=2, sort_keys=True, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise FreezeError(f"cannot encode canonical JSON: {error}") from error
    return (encoded + "\n").encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def require_source(root: Path, path: Path, source: dict[str, Any]) -> Path:
    try:
        resolved = path.resolve(strict=True)
    except OSError as error:
        raise FreezeError(f"missing source: {source['relative_name']}") from error
    if resolved.is_relative_to(root) or not resolved.is_file():
        raise FreezeError("source inputs must be external regular files")
    if resolved.stat().st_size != source["bytes"]:
        raise FreezeError(f"source byte count changed: {source['relative_name']}")
    if sha256_file(resolved) != source["sha256"]:
        raise FreezeError(f"source hash changed: {source['relative_name']}")
    return resolved


def prepare_output(root: Path, output: Path) -> tuple[Path, Path]:
    resolved = output.resolve()
    if resolved.is_relative_to(root):
        raise FreezeError("output must stay outside the repository")
    if resolved.exists():
        raise FreezeError("output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = resolved.parent / f".{resolved.name}.staging-{os.getpid()}"
    if staging.exists():
        raise FreezeError("staging output already exists")
    staging.mkdir()
    return resolved, staging


def load_json(path: Path, context: str) -> Any:
    try:
        return json.loads(path.read_bytes())
    except (UnicodeDecodeError, json.JSONDecodeError, OSError) as error:
        raise FreezeError(f"cannot decode {context} JSON") from error


def validate_shortlist(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise FreezeError("shortlist must be an object")
    if value.get("decision") != "FreshGlassCandidatesAvailable":
        raise FreezeError("shortlist decision changed")
    observed = []
    for candidate in value.get("fresh_candidates", []):
        if not isinstance(candidate, dict):
            raise FreezeError("shortlist candidate is not an object")
        if candidate.get("decision") != "FreshMetadataOnly":
            raise FreezeError("shortlist candidate is not metadata-only fresh")
        if candidate.get("direct_exposure_count") != 0:
            raise FreezeError("shortlist candidate has direct prior exposure")
        if candidate.get("path_token_exposure_count") != 0:
            raise FreezeError("shortlist candidate has path-token prior exposure")
        observed.append(
            (
                str(candidate.get("object_id")),
                candidate.get("name"),
                candidate.get("material"),
            )
        )
    if tuple(observed) != SHORTLIST_FRESH:
        raise FreezeError("fresh candidate set or order changed")
    return value


def parse_npy_float64(payload: bytes, expected_shape: tuple[int, ...]) -> tuple[float, ...]:
    if len(payload) < 12 or payload[:6] != b"\x93NUMPY":
        raise FreezeError("selected NPY has an invalid magic")
    major = payload[6]
    if major == 1:
        header_size = struct.unpack_from("<H", payload, 8)[0]
        data_offset = 10 + header_size
    elif major in (2, 3):
        header_size = struct.unpack_from("<I", payload, 8)[0]
        data_offset = 12 + header_size
    else:
        raise FreezeError("selected NPY version is unsupported")
    try:
        header = ast.literal_eval(payload[data_offset - header_size : data_offset].decode("latin1").strip())
    except (SyntaxError, ValueError, UnicodeDecodeError) as error:
        raise FreezeError("selected NPY header cannot be decoded") from error
    if not isinstance(header, dict):
        raise FreezeError("selected NPY header is not an object")
    if header.get("descr") != "<f8" or header.get("fortran_order") is not False:
        raise FreezeError("selected NPY dtype/order changed")
    shape = tuple(header.get("shape", ()))
    if shape != expected_shape:
        raise FreezeError("selected NPY shape changed")
    count = math.prod(shape)
    if len(payload) != data_offset + count * 8:
        raise FreezeError("selected NPY payload size changed")
    values = struct.unpack_from(f"<{count}d", payload, data_offset)
    if not all(math.isfinite(value) for value in values):
        raise FreezeError("selected NPY contains non-finite values")
    return values


def parse_pcm_header(payload: bytes) -> dict[str, int]:
    if len(payload) != EXPECTED_WAV_BYTES:
        raise FreezeError("selected WAV byte count changed")
    header = payload[:64]
    if len(header) < 44 or header[:4] != b"RIFF" or header[8:12] != b"WAVE":
        raise FreezeError("selected WAV has an invalid RIFF/WAVE header")
    if header[12:16] != b"fmt " or struct.unpack_from("<I", header, 16)[0] != 16:
        raise FreezeError("selected WAV fmt chunk changed")
    if header[36:40] != b"data":
        raise FreezeError("selected WAV data chunk changed")
    audio_format, channels, sample_rate = struct.unpack_from("<HHI", header, 20)
    block_align, bits_per_sample = struct.unpack_from("<HH", header, 32)
    data_bytes = struct.unpack_from("<I", header, 40)[0]
    if block_align == 0 or data_bytes % block_align != 0:
        raise FreezeError("selected WAV block alignment changed")
    descriptor = {
        "audio_format": audio_format,
        "channels": channels,
        "sample_rate_hz": sample_rate,
        "sample_width_bytes": bits_per_sample // 8,
        "frame_count": data_bytes // block_align,
    }
    if descriptor != EXPECTED_WAV_HEADER or data_bytes + 44 != len(payload):
        raise FreezeError("selected WAV PCM descriptor changed")
    return descriptor


def read_member(archive: tarfile.TarFile, member: tarfile.TarInfo) -> bytes:
    if not member.isfile():
        raise FreezeError(f"selected archive member is not a file: {member.name}")
    stream = archive.extractfile(member)
    if stream is None:
        raise FreezeError(f"cannot read selected archive member: {member.name}")
    return stream.read()


def scan_audio(path: Path) -> dict[int, dict[str, Any]]:
    expected = {f"audio/{OBJECT_ID}/{contact}.wav" for contact in EXPECTED_CONTACT_IDS}
    found: dict[str, dict[str, Any]] = {}
    observed_ids: set[int] = set()
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                parts = member.name.split("/")
                if len(parts) == 3 and parts[:2] == ["audio", OBJECT_ID]:
                    stem = Path(parts[2]).stem
                    if stem.isdecimal() and Path(parts[2]).suffix == ".wav":
                        observed_ids.add(int(stem))
                if member.name not in expected:
                    continue
                if member.name in found:
                    raise FreezeError(f"duplicate selected audio member: {member.name}")
                payload = read_member(archive, member)
                found[member.name] = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": sha256_bytes(payload),
                    "header": parse_pcm_header(payload),
                    "sample_values_decoded": 0,
                }
    except (tarfile.TarError, EOFError, OSError) as error:
        raise FreezeError("cannot scan audio archive") from error
    if set(found) != expected or observed_ids != set(EXPECTED_CONTACT_IDS):
        raise FreezeError("object-92 audio contact set changed")
    return {contact: found[f"audio/{OBJECT_ID}/{contact}.wav"] for contact in EXPECTED_CONTACT_IDS}


def scan_contacts(path: Path) -> tuple[dict[int, dict[str, Any]], dict[int, tuple[float, float, float]]]:
    expected = {f"contacts/{OBJECT_ID}/{contact}.npy" for contact in EXPECTED_CONTACT_IDS}
    found: dict[str, dict[str, Any]] = {}
    coordinates: dict[int, tuple[float, float, float]] = {}
    observed_ids: set[int] = set()
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                parts = member.name.split("/")
                if len(parts) == 3 and parts[:2] == ["contacts", OBJECT_ID]:
                    stem = Path(parts[2]).stem
                    if stem.isdecimal() and Path(parts[2]).suffix == ".npy":
                        observed_ids.add(int(stem))
                if member.name not in expected:
                    continue
                if member.name in found:
                    raise FreezeError(f"duplicate selected contact member: {member.name}")
                payload = read_member(archive, member)
                values = parse_npy_float64(payload, (3,))
                contact = int(Path(member.name).stem)
                coordinate = (values[0], values[1], values[2])
                coordinates[contact] = coordinate
                found[member.name] = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": sha256_bytes(payload),
                    "coordinate_m": list(coordinate),
                }
    except (tarfile.TarError, EOFError, OSError) as error:
        raise FreezeError("cannot scan contact archive") from error
    if set(found) != expected or observed_ids != set(EXPECTED_CONTACT_IDS):
        raise FreezeError("object-92 coordinate contact set changed")
    records = {
        contact: found[f"contacts/{OBJECT_ID}/{contact}.npy"]
        for contact in EXPECTED_CONTACT_IDS
    }
    return records, coordinates


def scan_point_cloud(path: Path) -> dict[str, Any]:
    selected = f"global_gt_points/{OBJECT_ID}.npy"
    found: dict[str, Any] | None = None
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                if member.name != selected:
                    continue
                if found is not None:
                    raise FreezeError("duplicate selected point-cloud member")
                payload = read_member(archive, member)
                values = parse_npy_float64(payload, EXPECTED_POINT_SHAPE)
                columns = tuple(values[index::3] for index in range(3))
                found = {
                    "path": member.name,
                    "bytes": len(payload),
                    "sha256": sha256_bytes(payload),
                    "shape": list(EXPECTED_POINT_SHAPE),
                    "dtype": "float64",
                    "minimum_m": [min(column) for column in columns],
                    "maximum_m": [max(column) for column in columns],
                }
    except (tarfile.TarError, EOFError, OSError) as error:
        raise FreezeError("cannot scan point-cloud archive") from error
    if found is None:
        raise FreezeError("selected point cloud is missing")
    return found


def validate_split(value: Any) -> dict[str, tuple[int, ...]]:
    if not isinstance(value, dict):
        raise FreezeError("official split must be an object")
    selected: dict[str, tuple[int, ...]] = {}
    for publisher_role in ("train", "val", "test"):
        try:
            contacts = tuple(
                int(contact)
                for object_id, contact in value[publisher_role]
                if str(object_id) == OBJECT_ID
            )
        except (KeyError, TypeError, ValueError) as error:
            raise FreezeError("official split structure changed") from error
        if contacts != PUBLISHER_SPLIT[publisher_role]:
            raise FreezeError(f"official {publisher_role} split changed")
        selected[publisher_role] = contacts
    flat = [contact for contacts in selected.values() for contact in contacts]
    if len(flat) != len(set(flat)) or set(flat) != set(EXPECTED_CONTACT_IDS):
        raise FreezeError("official object-92 split is not a complete partition")
    return selected


def validate_scale(value: Any) -> float:
    if not isinstance(value, dict):
        raise FreezeError("scale source must be an object")
    selected = value.get(OBJECT_ID)
    if type(selected) not in (int, float) or float(selected) != OBJECT_SCALE:
        raise FreezeError("published object-92 scale changed")
    return float(selected)


def role_rank(domain: str, contact: int) -> str:
    payload = f"{ROLE_SEPARATOR}\0{domain}\0{contact}".encode("utf-8")
    return sha256_bytes(payload)


def squared_distance(
    left: tuple[float, float, float], right: tuple[float, float, float]
) -> float:
    return sum((a - b) * (a - b) for a, b in zip(left, right, strict=True))


def farthest_point_order(
    candidates: tuple[int, ...] | list[int],
    count: int,
    domain: str,
    coordinates: dict[int, tuple[float, float, float]],
    base: tuple[int, ...] | list[int] = (),
) -> tuple[int, ...]:
    pool = set(candidates)
    selected = list(base)
    output: list[int] = []
    if count < 0 or count > len(pool):
        raise FreezeError("invalid farthest-point count")
    if not selected and count:
        seed = min(pool, key=lambda contact: (role_rank(f"{domain}-seed", contact), contact))
        pool.remove(seed)
        selected.append(seed)
        output.append(seed)
    while len(output) < count:
        scores = {
            contact: min(
                squared_distance(coordinates[contact], coordinates[other])
                for other in selected
            )
            for contact in pool
        }
        maximum = max(scores.values())
        tied = [contact for contact, score in scores.items() if score == maximum]
        chosen = min(tied, key=lambda contact: (role_rank(domain, contact), contact))
        pool.remove(chosen)
        selected.append(chosen)
        output.append(chosen)
    return tuple(output)


def derive_roles(
    split: dict[str, tuple[int, ...]],
    coordinates: dict[int, tuple[float, float, float]],
) -> dict[str, tuple[int, ...]]:
    formula_fit = farthest_point_order(
        split["train"], 8, "formula-fit", coordinates
    )
    train_remaining = tuple(
        contact for contact in split["train"] if contact not in formula_fit
    )
    development = farthest_point_order(
        train_remaining,
        6,
        "generator-development",
        coordinates,
        formula_fit,
    )
    representation = tuple(sorted(set(train_remaining) - set(development)))
    shadow = tuple(
        sorted(
            sorted(
                split["test"],
                key=lambda contact: (role_rank("admission-shadow", contact), contact),
            )[:3]
        )
    )
    method = tuple(sorted(set(split["test"]) - set(shadow)))
    roles = {
        "formula_fit": formula_fit,
        "generator_development": development,
        "representation_holdout": representation,
        "validator_calibration": tuple(sorted(split["val"])),
        "validator_method_holdout": method,
        "admission_shadow": shadow,
    }
    if roles != EXPECTED_ROLES:
        raise FreezeError("derived role partition changed")
    flat = [contact for contacts in roles.values() for contact in contacts]
    if len(flat) != len(set(flat)) or set(flat) != set(EXPECTED_CONTACT_IDS):
        raise FreezeError("derived roles are not a disjoint complete partition")
    return roles


def role_lookup(roles: dict[str, tuple[int, ...]]) -> dict[int, dict[str, Any]]:
    result: dict[int, dict[str, Any]] = {}
    for role, contacts in roles.items():
        for index, contact in enumerate(contacts):
            result[contact] = {"role": role, "role_order": index}
    return result


def build_manifest(
    shortlist: dict[str, Any],
    audio: dict[int, dict[str, Any]],
    contacts: dict[int, dict[str, Any]],
    point_cloud: dict[str, Any],
    split: dict[str, tuple[int, ...]],
    scale: float,
    roles: dict[str, tuple[int, ...]],
    implementation_sha256: str,
) -> dict[str, Any]:
    role_by_contact = role_lookup(roles)
    role_root_payload = {
        "object_id": OBJECT_ID,
        "role_separator": ROLE_SEPARATOR,
        "roles": roles,
    }
    role_root = sha256_bytes(canonical_json(role_root_payload))
    source_bytes = sum(source["bytes"] for source in SOURCES.values())
    selected_audio_bytes = sum(record["bytes"] for record in audio.values())
    return {
        "schema": MANIFEST_SCHEMA,
        "status": "FrozenZeroSampleSourceAndRoles",
        "study_id": STUDY_ID,
        "revision": REVISION,
        "claim_ceiling": "canonical_impact_field",
        "target": {
            "project_id": "objectfolder-real",
            "object_id": OBJECT_ID,
            "name": OBJECT_NAME,
            "published_material": PUBLISHED_MATERIAL,
            "scale": scale,
        },
        "candidate_decision": {
            "selected_object_id": OBJECT_ID,
            "fresh_candidates": [list(item) for item in SHORTLIST_FRESH],
            "shortlist_decision": shortlist["decision"],
            "selection_basis": "only_fresh_candidate_with_complete_selected_contact_localization_axes",
            "waveform_derived_selection_values": 0,
        },
        "primary_sources": {
            "objectfolder_download_page": "https://objectfolder.stanford.edu/objectfolder-real-download",
            "contact_localization_repository": "https://github.com/objectfolder/contact-localization",
            "contact_localization_commit": "4bb002f519cab9d250bbbe045a6df0248bf1639f",
            "files": SOURCES,
            "raw_archive": RAW_ARCHIVE,
        },
        "publisher_split": split,
        "role_algorithm": {
            "separator": ROLE_SEPARATOR,
            "fit_count": 8,
            "development_count": 6,
            "shadow_count": 3,
            "coordinate_metric": "squared_euclidean_float64_m",
            "selection": "deterministic_maximin_with_sha256_exact_tie_break",
        },
        "roles": roles,
        "role_root_sha256": role_root,
        "point_cloud": point_cloud,
        "contacts": [
            {
                "object_id": OBJECT_ID,
                "contact_id": contact,
                **role_by_contact[contact],
                "coordinate": contacts[contact],
                "microphone": audio[contact],
            }
            for contact in EXPECTED_CONTACT_IDS
        ],
        "axes": {
            "object_identity": "known",
            "recorded_response": "known_hash_and_header_only",
            "contact_position": "known_official_xyz_m",
            "geometry": "known_official_1024_point_cloud",
            "listener_condition": "fixed_dataset_condition_numeric_pose_absent",
            "canonical_excitation": "pending_m2b_raw_force",
            "raw_force": "not_acquired_m2a",
            "force_frequency_coverage": "not_applicable_to_canonical_claim",
            "support_condition": "absent",
            "microphone_calibration": "absent",
        },
        "read_accounting": {
            "network_requests": 0,
            "new_raw_archive_bytes_acquired": 0,
            "source_file_bytes_hashed": source_bytes,
            "selected_wav_members_hash_committed": len(audio),
            "selected_wav_payload_bytes_hashed": selected_audio_bytes,
            "selected_wav_header_bytes_parsed": len(audio) * 64,
            "pcm_sample_values_decoded": 0,
            "force_sample_values_decoded": 0,
            "contact_coordinate_scalar_values_decoded": len(contacts) * 3,
            "point_cloud_scalar_values_decoded": math.prod(EXPECTED_POINT_SHAPE),
            "selected_split_pairs_decoded": len(EXPECTED_CONTACT_IDS),
            "selected_scale_values_decoded": 1,
            "waveform_payloads_emitted": 0,
        },
        "implementation_sha256": implementation_sha256,
        "next_authorized_step": "M2B_RAW_FORCE_ACQUISITION_PROTOCOL_ONLY",
        "all_real_roles_sample_decode_closed": True,
        "formula_or_model_fit_authorized": False,
        "runtime_consumer_allowed": False,
        "public_contract": False,
        "authored_clip_fallback_required": True,
    }


def run(
    shortlist_argument: Path,
    audio_argument: Path,
    contacts_argument: Path,
    point_cloud_argument: Path,
    split_argument: Path,
    scale_argument: Path,
    output_argument: Path,
) -> Path:
    root = repository_root()
    paths = {
        "shortlist": require_source(root, shortlist_argument, SOURCES["shortlist"]),
        "audio": require_source(root, audio_argument, SOURCES["audio"]),
        "contacts": require_source(root, contacts_argument, SOURCES["contacts"]),
        "point_cloud": require_source(root, point_cloud_argument, SOURCES["point_cloud"]),
        "split": require_source(root, split_argument, SOURCES["split"]),
        "scale": require_source(root, scale_argument, SOURCES["scale"]),
    }
    shortlist = validate_shortlist(load_json(paths["shortlist"], "shortlist"))
    split = validate_split(load_json(paths["split"], "official split"))
    scale = validate_scale(load_json(paths["scale"], "scale"))
    contacts, coordinates = scan_contacts(paths["contacts"])
    point_cloud = scan_point_cloud(paths["point_cloud"])
    roles = derive_roles(split, coordinates)

    # Role identities are immutable before the first selected WAV member is read.
    audio = scan_audio(paths["audio"])
    manifest = build_manifest(
        shortlist,
        audio,
        contacts,
        point_cloud,
        split,
        scale,
        roles,
        sha256_file(Path(__file__).resolve()),
    )
    manifest_bytes = canonical_json(manifest)
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Validated",
        "decision": "READY_FOR_M2B_RAW_FORCE_ACQUISITION_PROTOCOL",
        "manifest_sha256": sha256_bytes(manifest_bytes),
        "role_root_sha256": manifest["role_root_sha256"],
        "target_object_id": OBJECT_ID,
        "contact_count": len(EXPECTED_CONTACT_IDS),
        "role_counts": {role: len(contacts) for role, contacts in roles.items()},
        "point_cloud_validated": True,
        "wav_headers_validated": len(audio),
        "read_accounting": manifest["read_accounting"],
        "all_real_roles_sample_decode_closed": True,
        "real_quality_credit": False,
        "formula_or_model_fit_authorized": False,
        "runtime_or_public_contract_changed": False,
        "authored_clip_fallback_required": True,
    }

    output, staging = prepare_output(root, output_argument)
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
    parser.add_argument("--shortlist", required=True, type=Path)
    parser.add_argument("--audio-archive", required=True, type=Path)
    parser.add_argument("--contacts-archive", required=True, type=Path)
    parser.add_argument("--point-cloud-archive", required=True, type=Path)
    parser.add_argument("--split-json", required=True, type=Path)
    parser.add_argument("--scale-json", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    run(
        arguments.shortlist,
        arguments.audio_archive,
        arguments.contacts_archive,
        arguments.point_cloud_archive,
        arguments.split_json,
        arguments.scale_json,
        arguments.output,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
