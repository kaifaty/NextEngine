#!/usr/bin/env python3
"""Build the zero-signal Physical Sound V14-N1b source inventory."""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import math
import os
import re
import shutil
import tarfile
import tempfile
import urllib.error
import urllib.parse
import urllib.request
from collections import Counter, defaultdict
from concurrent.futures import ThreadPoolExecutor
from dataclasses import dataclass
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath
from typing import Any, Iterable

import physical_sound_dataset_contract_v1 as contract_v1
import physical_sound_exposure_ledger_v0 as ledger_v0

INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-source-inventory.v1"
COST_SCHEMA = "nextengine.experimental-physical-sound-source-cost-report.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v14-n1b.report.v1"
HTTP_SCHEMA = "nextengine.experimental-physical-sound-source-http-snapshot.v1"

N1A_DESCRIPTOR_SHA256 = (
    "493f8bcde32190c046400ed65fbc49f0bc9fbdb017ce632c780bd55ecd194073"
)
LEDGER_SHA256 = "5d69b8211d451dff5a6ef9fadfe2609dafbfde378007ff77475fc2db7cafbbea"
OBJECTFOLDER_CURRENT_SHA256 = (
    "0111f57a336fdeb805eb03543fb10b2cb174f8d0bcb7ef5f0e30c17324e06db0"
)
OBJECTFOLDER_HISTORICAL_SHA256 = (
    "2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32"
)
OBJECTFOLDER2_CSV_SHA256 = (
    "5565b7e8f739194616d5f276e49b6e3bbf34f447f2a07b79ad02b1c23d8b1557"
)
OBJECTFOLDER2_README_SHA256 = (
    "bf37922f51280a981a6bd2a27cba3e8a9698f05d5d52be8d09c8a925eed8b1c0"
)
REALIMPACT_NAMES_SHA256 = (
    "3ee26ac9b34130353dc93dc16dfa448b54b309d183f4e7b8c7d2464cb807ad5a"
)
REALIMPACT_README_SHA256 = (
    "3dd228b826651745f0cba8c8bfc0ff142f8fb4d9574f5adda91c272ffd8a2049"
)
REALIMPACT_DOWNLOAD_SHA256 = (
    "4d6c2d7967d7dc2b8c56c7bfd7fe1550202099b6a7b1207aadc40e19d553a45a"
)

COMPACT_SOURCES = {
    "audio": (
        463_486_373,
        "14a15b96dda7c3933a4a5829dc5ee21e1f9c2fe29f7c558925df24e87097a2c9",
    ),
    "contacts": (
        121_961,
        "310c45e11e40ceafc9c9fc1379059848ff664a7c2999fea5aabea479fc61eeee",
    ),
    "geometry": (
        2_359_020,
        "29ebf37bb88bc5c654b096f31552e4677fc156135930da72c82e306c5c517e51",
    ),
    "split": (
        19_258,
        "77ea2d99724197ad9e88c3ad90f49885c582dea6835fdc0ad533e5b9ce78674e",
    ),
    "scale": (
        2_652,
        "39c84eafabe999b1503710a9d601b0bc94fc6e358461aa2e5988b34b6b6d0b8a",
    ),
}

REAL_INPUTS = {
    "descriptor": (2_836, N1A_DESCRIPTOR_SHA256),
    "ledger": (562_493, LEDGER_SHA256),
    "objectfolder_current": (34_749, OBJECTFOLDER_CURRENT_SHA256),
    "objectfolder_historical": (8_668, OBJECTFOLDER_HISTORICAL_SHA256),
    "objectfolder2_csv": (67_378, OBJECTFOLDER2_CSV_SHA256),
    "objectfolder2_readme": (9_608, OBJECTFOLDER2_README_SHA256),
    "realimpact_names": (738, REALIMPACT_NAMES_SHA256),
    "realimpact_readme": (515, REALIMPACT_README_SHA256),
    "realimpact_download": (168, REALIMPACT_DOWNLOAD_SHA256),
    **{key: value for key, value in COMPACT_SOURCES.items()},
}

SOURCE_REVISIONS = {
    "objectfolder_real_current": "rendered-table-sha256-0111f57a",
    "objectfolder_real_historical_d058": (
        "d058ba09e7e7a1a8358d64d7b48e5f588b377eb8"
    ),
    "realimpact_fca2": "fca2bd6cbb7e9f96ac61328d2a0d51594bf01987",
    "objectfolder2_3c6c": "3c6cd8930b2dcbadb6d94dadf2745c956bdcd236",
}

MATERIAL_MAP = {"Glass": "Glass", "Wood": "Wood", "Steel": "Metal", "Iron": "Metal"}
MATERIALS = ("Glass", "Metal", "Wood")
HTTP_ENTRY_KEYS = {
    "accept_ranges",
    "content_length",
    "error",
    "etag",
    "final_url",
    "last_modified",
    "requested_url",
    "status",
}
ALLOWED_HTTP_HOSTS = {"download.cs.stanford.edu", "downloads.cs.stanford.edu"}
MAX_JSON_BYTES = 16 * 1024 * 1024


class SourceInventoryError(RuntimeError):
    """The N1b input or result violates the frozen metadata-only protocol."""


@dataclass(frozen=True)
class Inputs:
    descriptor: Path
    ledger: Path
    objectfolder_current: Path
    objectfolder_historical: Path
    objectfolder2_csv: Path
    objectfolder2_readme: Path
    realimpact_names: Path
    realimpact_readme: Path
    realimpact_download: Path
    audio: Path
    contacts: Path
    geometry: Path
    split: Path
    scale: Path
    http_snapshot: Path


def canonical_json(value: Any) -> bytes:
    return ledger_v0.canonical_json(value)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def reject_constant(value: str) -> None:
    raise SourceInventoryError(f"non-finite JSON number is forbidden: {value}")


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise SourceInventoryError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_json(data: bytes, context: str, *, canonical: bool = False) -> Any:
    if len(data) > MAX_JSON_BYTES:
        raise SourceInventoryError(f"{context} exceeds the byte limit")
    try:
        value = json.loads(
            data,
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as error:
        raise SourceInventoryError(f"cannot parse {context}: {error}") from error
    if canonical and canonical_json(value) != data:
        raise SourceInventoryError(f"{context} is not canonical JSON")
    return value


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise SourceInventoryError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise SourceInventoryError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def hash_regular_file(path: Path, context: str) -> tuple[int, str]:
    if path.is_symlink() or not path.is_file():
        raise SourceInventoryError(f"{context} must be a regular file")
    before = path.stat()
    digest = hashlib.sha256()
    size = 0
    with path.open("rb") as source:
        while block := source.read(1024 * 1024):
            digest.update(block)
            size += len(block)
    after = path.stat()
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
        after.st_dev,
        after.st_ino,
        after.st_size,
        after.st_mtime_ns,
    ):
        raise SourceInventoryError(f"{context} changed while hashing")
    return size, digest.hexdigest()


def read_bound_file(
    path: Path,
    context: str,
    expected: tuple[int, str] | None,
) -> bytes:
    size, digest = hash_regular_file(path, context)
    if expected is not None and (size, digest) != expected:
        raise SourceInventoryError(
            f"{context} identity changed: got bytes={size}, sha256={digest}"
        )
    return path.read_bytes()


class TableCells(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.cells: list[str] = []
        self._parts: list[str] | None = None

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        del attrs
        if tag == "td":
            self._parts = []

    def handle_data(self, data: str) -> None:
        if self._parts is not None:
            self._parts.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "td" and self._parts is not None:
            self.cells.append("".join(self._parts).strip())
            self._parts = None


def validate_object_rows(rows: dict[int, tuple[str, str]], context: str) -> None:
    if set(rows) != set(range(1, 101)):
        raise SourceInventoryError(f"{context} must contain exact object IDs 1..100")
    for object_id, (name, material) in rows.items():
        if not name or not material or len(name) > 128 or len(material) > 64:
            raise SourceInventoryError(f"{context} row {object_id} is malformed")


def parse_current_table(data: bytes) -> dict[int, tuple[str, str]]:
    parser = TableCells()
    try:
        parser.feed(data.decode("utf-8"))
    except UnicodeDecodeError as error:
        raise SourceInventoryError("current ObjectFolder HTML is not UTF-8") from error
    rows: dict[int, tuple[str, str]] = {}
    for index in range(0, len(parser.cells), 3):
        triple = parser.cells[index : index + 3]
        if len(triple) == 3 and triple[0].isdigit():
            object_id = int(triple[0])
            if object_id in rows:
                raise SourceInventoryError("duplicate current ObjectFolder object ID")
            rows[object_id] = (triple[1], triple[2])
    validate_object_rows(rows, "current ObjectFolder table")
    return rows


def parse_historical_table(data: bytes) -> dict[int, tuple[str, str]]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourceInventoryError("historical ObjectFolder Markdown is not UTF-8") from error
    rows: dict[int, tuple[str, str]] = {}
    for line in text.splitlines():
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if len(cells) != 9:
            continue
        for offset in (0, 3, 6):
            if not cells[offset].isdigit():
                continue
            object_id = int(cells[offset])
            if object_id in rows:
                raise SourceInventoryError("duplicate historical ObjectFolder object ID")
            rows[object_id] = (cells[offset + 1], cells[offset + 2])
    validate_object_rows(rows, "historical ObjectFolder table")
    return rows


def parse_objectfolder2(data: bytes) -> dict[int, tuple[str, str, float, str]]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise SourceInventoryError("ObjectFolder 2.0 CSV is not UTF-8") from error
    rows: dict[int, tuple[str, str, float, str]] = {}
    for line_number, row in enumerate(csv.reader(io.StringIO(text)), 1):
        if len(row) != 5 or not row[0].isdigit():
            raise SourceInventoryError(f"malformed ObjectFolder 2.0 row {line_number}")
        object_id = int(row[0])
        try:
            scale = float(row[2])
        except ValueError as error:
            raise SourceInventoryError("invalid ObjectFolder 2.0 scale") from error
        if object_id in rows or not math.isfinite(scale) or scale <= 0:
            raise SourceInventoryError("invalid ObjectFolder 2.0 object identity")
        if not row[1] or not row[3] or not row[4]:
            raise SourceInventoryError("incomplete ObjectFolder 2.0 row")
        rows[object_id] = (row[1], row[3], scale, row[4])
    if set(rows) != set(range(1, 1001)):
        raise SourceInventoryError("ObjectFolder 2.0 CSV must contain exact IDs 1..1000")
    return rows


def parse_realimpact_names(data: bytes) -> dict[int, str]:
    try:
        lines = data.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise SourceInventoryError("RealImpact names are not UTF-8") from error
    rows: dict[int, str] = {}
    for line in lines:
        match = re.fullmatch(r"([1-9][0-9]{0,2})_([A-Za-z0-9_]+)", line)
        if match is None:
            raise SourceInventoryError(f"malformed RealImpact object name: {line!r}")
        object_id = int(match.group(1))
        if object_id in rows:
            raise SourceInventoryError("duplicate RealImpact numeric object ID")
        rows[object_id] = line
    if len(rows) != 50:
        raise SourceInventoryError("RealImpact must contain exactly 50 object names")
    return rows


def safe_member(member: tarfile.TarInfo, context: str) -> None:
    path = PurePosixPath(member.name)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        raise SourceInventoryError(f"unsafe {context} member path: {member.name}")
    if not (member.isfile() or member.isdir()):
        raise SourceInventoryError(f"unsupported {context} member type: {member.name}")


def tar_members(path: Path, kind: str) -> tuple[dict[int, list[dict[str, Any]]], int]:
    patterns = {
        "audio": re.compile(r"audio/([0-9]+)/([0-9]+)\.wav\Z"),
        "contacts": re.compile(r"contacts/([0-9]+)/([0-9]+)\.npy\Z"),
        "geometry": re.compile(r"global_gt_points/([0-9]+)\.npy\Z"),
    }
    result: dict[int, list[dict[str, Any]]] = defaultdict(list)
    seen: set[str] = set()
    headers = 0
    try:
        with tarfile.open(path, "r:gz") as archive:
            for member in archive:
                headers += 1
                safe_member(member, kind)
                if member.name in seen:
                    raise SourceInventoryError(f"duplicate {kind} member: {member.name}")
                seen.add(member.name)
                if member.isdir():
                    continue
                match = patterns[kind].fullmatch(member.name)
                if match is None:
                    raise SourceInventoryError(f"unexpected {kind} member: {member.name}")
                object_id = int(match.group(1))
                contact_id = int(match.group(2)) if len(match.groups()) == 2 else None
                result[object_id].append(
                    {"contact_id": contact_id, "declared_bytes": member.size, "name": member.name}
                )
    except (tarfile.TarError, OSError) as error:
        raise SourceInventoryError(f"cannot traverse {kind} archive: {error}") from error
    for members in result.values():
        members.sort(key=lambda item: item["name"])
    return dict(result), headers


def parse_split(data: bytes) -> tuple[dict[int, dict[str, list[int]]], int]:
    value = parse_json(data, "compact split JSON")
    if not isinstance(value, dict) or set(value) != {"train", "val", "test"}:
        raise SourceInventoryError("compact split roles changed")
    result: dict[int, dict[str, list[int]]] = defaultdict(
        lambda: {"test": [], "train": [], "val": []}
    )
    seen: set[tuple[int, int]] = set()
    pairs = 0
    for role in ("train", "val", "test"):
        entries = value[role]
        if not isinstance(entries, list):
            raise SourceInventoryError("compact split role must be an array")
        for pair in entries:
            if (
                not isinstance(pair, list)
                or len(pair) != 2
                or not all(isinstance(item, str) and item.isdigit() for item in pair)
            ):
                raise SourceInventoryError("compact split pair is malformed")
            key = (int(pair[0]), int(pair[1]))
            if key in seen:
                raise SourceInventoryError("duplicate compact object/contact split key")
            seen.add(key)
            result[key[0]][role].append(key[1])
            pairs += 1
    for roles in result.values():
        for contacts in roles.values():
            contacts.sort()
    return dict(result), pairs


def parse_scale(data: bytes) -> dict[int, float]:
    value = parse_json(data, "compact scale JSON")
    if not isinstance(value, dict):
        raise SourceInventoryError("compact scale must be an object")
    result: dict[int, float] = {}
    for key, scale in value.items():
        if not isinstance(key, str) or not key.isdigit() or isinstance(scale, bool):
            raise SourceInventoryError("compact scale row is malformed")
        numeric = float(scale)
        if not math.isfinite(numeric) or numeric <= 0 or int(key) in result:
            raise SourceInventoryError("compact scale value is invalid")
        result[int(key)] = numeric
    return result


def object_batch_url(object_id: int, source: str) -> str:
    if source == "objectfolder_real":
        start = ((object_id - 1) // 10) * 10 + 1
        end = start + 9
        return (
            "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
            f"audio_data_{start}_{end}.tar.gz"
        )
    if source == "objectfolder2":
        start = ((object_id - 1) // 100) * 100 + 1
        end = start + 99
        return (
            "https://download.cs.stanford.edu/viscam/ObjectFolder/"
            f"ObjectFolder{start}-{end}.tar.gz"
        )
    raise SourceInventoryError(f"unknown batch source: {source}")


def expected_urls(realimpact_names: Iterable[str]) -> list[str]:
    urls = [object_batch_url(object_id, "objectfolder_real") for object_id in range(1, 101, 10)]
    urls.extend(object_batch_url(object_id, "objectfolder2") for object_id in range(1, 1001, 100))
    urls.extend(
        f"https://downloads.cs.stanford.edu/viscam/RealImpact/{name}.zip"
        for name in sorted(realimpact_names)
    )
    if len(set(urls)) != 70:
        raise SourceInventoryError("HTTP endpoint set must contain exactly 70 URLs")
    return sorted(urls)


def validate_http_snapshot(data: bytes, urls: list[str]) -> tuple[dict[str, dict[str, Any]], str]:
    value = parse_json(data, "HTTP snapshot", canonical=True)
    snapshot = require_exact_keys(value, {"acquired_at_utc", "entries", "schema"}, "HTTP snapshot")
    if snapshot["schema"] != HTTP_SCHEMA or not isinstance(snapshot["acquired_at_utc"], str):
        raise SourceInventoryError("unknown HTTP snapshot schema or timestamp")
    if not isinstance(snapshot["entries"], list) or len(snapshot["entries"]) != 70:
        raise SourceInventoryError("HTTP snapshot must contain exactly 70 entries")
    result: dict[str, dict[str, Any]] = {}
    for index, raw in enumerate(snapshot["entries"]):
        entry = require_exact_keys(raw, HTTP_ENTRY_KEYS, f"HTTP entry {index}")
        requested = entry["requested_url"]
        if requested not in urls or requested in result:
            raise SourceInventoryError("HTTP snapshot URL set changed or duplicated")
        for field in ("final_url", "etag", "last_modified", "accept_ranges", "error"):
            if entry[field] is not None and not isinstance(entry[field], str):
                raise SourceInventoryError(f"HTTP entry {field} must be string or null")
        if entry["status"] is not None and (
            not isinstance(entry["status"], int) or isinstance(entry["status"], bool)
        ):
            raise SourceInventoryError("HTTP status must be integer or null")
        if entry["content_length"] is not None and (
            not isinstance(entry["content_length"], int)
            or isinstance(entry["content_length"], bool)
            or entry["content_length"] < 0
        ):
            raise SourceInventoryError("HTTP content length must be non-negative or null")
        final = entry["final_url"]
        if final is not None:
            parsed = urllib.parse.urlparse(final)
            if parsed.scheme != "https" or parsed.hostname not in ALLOWED_HTTP_HOSTS:
                raise SourceInventoryError("HTTP redirect escaped the frozen hosts")
        result[requested] = entry
    if set(result) != set(urls):
        raise SourceInventoryError("HTTP snapshot is missing an expected URL")
    return result, sha256_bytes(data)


def normalize_name(value: str) -> str:
    return re.sub(r"[^a-z0-9]", "", value.lower())


def inventory_identity(source_id: str, publisher_object_id: str) -> str:
    return sha256_bytes(
        canonical_json(
            {"publisher_object_id": publisher_object_id, "source_id": source_id}
        )
    )


def conservative_object_parent(object_id: int) -> str:
    identity = {
        "object_id": str(object_id),
        "project_id": str(object_id),
        "source_namespace": "conservative-objectfolder-real-id",
    }
    return sha256_bytes(canonical_json(identity))


def member_root(parts: list[dict[str, Any]]) -> str | None:
    return sha256_bytes(canonical_json(parts)) if parts else None


def exposure_for_hashes(
    query_hashes: Iterable[str], roles_by_hash: dict[str, list[str]]
) -> dict[str, Any]:
    queries = sorted(set(query_hashes))
    roles = sorted({role for digest in queries for role in roles_by_hash.get(digest, [])})
    return {
        "queried_object_parent_sha256s": queries,
        "roles": roles,
        "state": contract_v1.exposure_state(roles),
    }


def identity_available(entry: dict[str, Any]) -> bool:
    return entry["status"] in (200, 206) and entry["content_length"] is not None


def source_records(http_sha256: str) -> list[dict[str, Any]]:
    return [
        {
            "publisher_id": "stanford-objectfolder",
            "revision_id": SOURCE_REVISIONS["objectfolder_real_current"],
            "source_id": "objectfolder_real_current",
            "source_tier": "T2_sparse_real_contact",
        },
        {
            "publisher_id": "stanford-objectfolder",
            "revision_id": SOURCE_REVISIONS["objectfolder_real_historical_d058"],
            "source_id": "objectfolder_real_historical_d058",
            "source_tier": "T2_sparse_real_contact",
        },
        {
            "publisher_id": "stanford-realimpact",
            "revision_id": SOURCE_REVISIONS["realimpact_fca2"],
            "source_id": "realimpact_fca2",
            "source_tier": "T3_dense_real_listener",
        },
        {
            "publisher_id": "stanford-objectfolder",
            "revision_id": SOURCE_REVISIONS["objectfolder2_3c6c"],
            "source_id": "objectfolder2_3c6c",
            "source_tier": "T1_synthetic_teacher",
        },
    ]


def compact_structure(
    object_id: int,
    audio: dict[int, list[dict[str, Any]]],
    contacts: dict[int, list[dict[str, Any]]],
    geometry: dict[int, list[dict[str, Any]]],
    split: dict[int, dict[str, list[int]]],
    scale: dict[int, float],
) -> tuple[dict[str, Any], str | None]:
    parts = []
    for kind, members in (("audio", audio.get(object_id, [])), ("contacts", contacts.get(object_id, [])), ("geometry", geometry.get(object_id, []))):
        parts.extend({"kind": kind, **member} for member in members)
    split_roles = split.get(object_id, {"test": [], "train": [], "val": []})
    return (
        {
            "audio_member_count": len(audio.get(object_id, [])),
            "contact_member_count": len(contacts.get(object_id, [])),
            "geometry_member_count": len(geometry.get(object_id, [])),
            "scale_present": object_id in scale,
            "split_contact_counts": {
                role: len(split_roles[role]) for role in ("test", "train", "val")
            },
            "split_present": object_id in split,
        },
        member_root(parts),
    )


def build_documents(
    inputs: Inputs,
    *,
    expected_identities: dict[str, tuple[int, str]] | None = REAL_INPUTS,
) -> dict[str, bytes]:
    raw: dict[str, bytes] = {}
    paths = {
        "descriptor": inputs.descriptor,
        "ledger": inputs.ledger,
        "objectfolder_current": inputs.objectfolder_current,
        "objectfolder_historical": inputs.objectfolder_historical,
        "objectfolder2_csv": inputs.objectfolder2_csv,
        "objectfolder2_readme": inputs.objectfolder2_readme,
        "realimpact_names": inputs.realimpact_names,
        "realimpact_readme": inputs.realimpact_readme,
        "realimpact_download": inputs.realimpact_download,
        "audio": inputs.audio,
        "contacts": inputs.contacts,
        "geometry": inputs.geometry,
        "split": inputs.split,
        "scale": inputs.scale,
    }
    input_identities: dict[str, dict[str, Any]] = {}
    for key, path in paths.items():
        expected = None if expected_identities is None else expected_identities[key]
        if key in {"audio", "contacts", "geometry"}:
            size, digest = hash_regular_file(path, key)
            if expected is not None and (size, digest) != expected:
                raise SourceInventoryError(
                    f"{key} identity changed: got bytes={size}, sha256={digest}"
                )
            data = None
        else:
            data = read_bound_file(path, key, expected)
            raw[key] = data
        input_identities[key] = {
            "bytes": size if data is None else len(data),
            "sha256": digest if data is None else sha256_bytes(data),
        }

    descriptor = parse_json(raw["descriptor"], "N1a descriptor", canonical=True)
    if (
        not isinstance(descriptor, dict)
        or descriptor.get("schema") != contract_v1.DESCRIPTOR_SCHEMA
        or descriptor.get("contract_schema") != contract_v1.CONTRACT_SCHEMA
        or descriptor.get("public_contract") is not False
        or descriptor.get("runtime_consumer_allowed") is not False
        or descriptor.get("policy", {}).get("minimum_per_material")
        != {material: contract_v1.MINIMUM_PER_MATERIAL for material in MATERIALS}
    ):
        raise SourceInventoryError("N1a descriptor policy changed")

    ledger_bytes = raw["ledger"]
    ledger_raw = parse_json(ledger_bytes, "Exposure Ledger V0", canonical=True)
    try:
        ledger = contract_v1.validate_ledger(ledger_bytes, ledger_raw)
    except contract_v1.DatasetContractError as error:
        raise SourceInventoryError(f"invalid Exposure Ledger V0: {error}") from error
    roles_by_hash = contract_v1.ledger_roles_by_object(ledger)

    current = parse_current_table(raw["objectfolder_current"])
    historical = parse_historical_table(raw["objectfolder_historical"])
    objectfolder2 = parse_objectfolder2(raw["objectfolder2_csv"])
    realimpact = parse_realimpact_names(raw["realimpact_names"])
    if b"raw dataset" not in raw["realimpact_readme"] or b"wget http://downloads.cs.stanford.edu/viscam/RealImpact/$line.zip" not in raw["realimpact_download"]:
        raise SourceInventoryError("RealImpact publication boundary changed")
    if b"1,000 objects" not in raw["objectfolder2_readme"] or b"specified force profile" not in raw["objectfolder2_readme"]:
        raise SourceInventoryError("ObjectFolder 2.0 publication claims changed")

    urls = expected_urls(realimpact.values())
    http_bytes = read_bound_file(inputs.http_snapshot, "HTTP snapshot", None)
    http, http_sha256 = validate_http_snapshot(http_bytes, urls)
    input_identities["http_snapshot"] = {
        "bytes": len(http_bytes),
        "sha256": http_sha256,
    }

    audio, audio_headers = tar_members(inputs.audio, "audio")
    contacts, contact_headers = tar_members(inputs.contacts, "contacts")
    geometry, geometry_headers = tar_members(inputs.geometry, "geometry")
    split, split_pairs = parse_split(raw["split"])
    scale = parse_scale(raw["scale"])

    discrepancies = []
    drift_ids = []
    alias_edges: list[dict[str, Any]] = []
    for object_id in range(1, 101):
        same = current[object_id] == historical[object_id]
        if same:
            alias_edges.append(
                {
                    "decision": "proven_same_publisher_identity",
                    "left": inventory_identity("objectfolder_real_current", str(object_id)),
                    "right": inventory_identity("objectfolder_real_historical_d058", str(object_id)),
                }
            )
        else:
            drift_ids.append(object_id)
            discrepancies.append(
                {
                    "code": "objectfolder_revision_identity_conflict",
                    "current": {"id": str(object_id), "material": current[object_id][1], "name": current[object_id][0]},
                    "historical": {"id": str(object_id), "material": historical[object_id][1], "name": historical[object_id][0]},
                }
            )

    objects: list[dict[str, Any]] = []
    for object_id in range(1, 101):
        name, publisher_material = current[object_id]
        material = MATERIAL_MAP.get(publisher_material)
        structure, root = compact_structure(
            object_id, audio, contacts, geometry, split, scale
        )
        archive_url = object_batch_url(object_id, "objectfolder_real")
        reasons: list[str] = []
        alias_status = "proven_with_historical_revision" if object_id not in drift_ids else "ambiguous_revision_identity"
        if material is None:
            state = "source_ood"
            reasons.append("material_out_of_scope")
        elif object_id in drift_ids:
            state = "source_ood"
            reasons.append("ambiguous_revision_identity")
        elif not identity_available(http[archive_url]):
            state = "source_ood"
            reasons.append("raw_archive_identity_unavailable")
        elif (
            structure["split_present"]
            and structure["audio_member_count"] > 0
            and structure["contact_member_count"] > 0
            and structure["geometry_member_count"] == 1
            and structure["scale_present"]
        ):
            state = "metadata_candidate"
            reasons.append("compact_structure_present")
        else:
            state = "member_preflight_required"
            reasons.append("compact_structure_incomplete")
        objects.append(
            {
                "alias_status": alias_status,
                "candidate_state": state,
                "exposure": exposure_for_hashes([conservative_object_parent(object_id)], roles_by_hash),
                "inventory_object_id": inventory_identity("objectfolder_real_current", str(object_id)),
                "material_family": material,
                "member_root_sha256": root,
                "publisher_material": publisher_material,
                "publisher_name": name,
                "publisher_object_id": str(object_id),
                "reason_codes": sorted(reasons),
                "source_id": "objectfolder_real_current",
                "source_tier": "T2_sparse_real_contact",
                "structural_presence": {**structure, "remote_archive_identity_available": identity_available(http[archive_url])},
            }
        )

    for object_id, publisher_id in sorted(realimpact.items()):
        historical_name, historical_material = historical[object_id]
        suffix = publisher_id.split("_", 1)[1]
        exact_name = normalize_name(suffix) == normalize_name(historical_name)
        material = MATERIAL_MAP.get(historical_material) if exact_name else None
        archive_url = f"https://downloads.cs.stanford.edu/viscam/RealImpact/{publisher_id}.zip"
        reasons = []
        if not exact_name:
            reasons.append("ambiguous_revision_identity")
        if not identity_available(http[archive_url]):
            reasons.append("archive_identity_unavailable")
        state = "member_preflight_required" if exact_name and material and identity_available(http[archive_url]) else "source_ood"
        if state == "member_preflight_required":
            reasons.append("zip_member_preflight_required")
            alias_edges.append(
                {
                    "decision": "proven_historical_name_identity",
                    "left": inventory_identity("realimpact_fca2", publisher_id),
                    "right": inventory_identity("objectfolder_real_historical_d058", str(object_id)),
                }
            )
        elif material is None and "ambiguous_revision_identity" not in reasons:
            reasons.append("material_out_of_scope")
        objects.append(
            {
                "alias_status": "proven_with_historical_revision" if exact_name else "ambiguous_revision_identity",
                "candidate_state": state,
                "exposure": exposure_for_hashes([conservative_object_parent(object_id)] if exact_name else [], roles_by_hash),
                "inventory_object_id": inventory_identity("realimpact_fca2", publisher_id),
                "material_family": material,
                "member_root_sha256": None,
                "publisher_material": historical_material if exact_name else None,
                "publisher_name": publisher_id,
                "publisher_object_id": publisher_id,
                "reason_codes": sorted(reasons),
                "source_id": "realimpact_fca2",
                "source_tier": "T3_dense_real_listener",
                "structural_presence": {
                    "archive_bytes": http[archive_url]["content_length"],
                    "archive_identity_available": identity_available(http[archive_url]),
                    "member_preflight_complete": False,
                },
            }
        )

    for object_id, (name, publisher_material, object_scale, origin_url) in sorted(objectfolder2.items()):
        del object_scale, origin_url
        material = MATERIAL_MAP.get(publisher_material)
        archive_url = object_batch_url(object_id, "objectfolder2")
        reasons = []
        if material is None:
            state = "source_ood"
            reasons.append("material_out_of_scope")
        elif not identity_available(http[archive_url]):
            state = "source_ood"
            reasons.append("archive_identity_unavailable")
        else:
            state = "member_preflight_required"
            reasons.append("synthetic_teacher_member_preflight_required")
        objects.append(
            {
                "alias_status": "separate_synthetic_namespace",
                "candidate_state": state,
                "exposure": exposure_for_hashes([], roles_by_hash),
                "inventory_object_id": inventory_identity("objectfolder2_3c6c", str(object_id)),
                "material_family": material,
                "member_root_sha256": None,
                "publisher_material": publisher_material,
                "publisher_name": name,
                "publisher_object_id": str(object_id),
                "reason_codes": sorted(reasons),
                "source_id": "objectfolder2_3c6c",
                "source_tier": "T1_synthetic_teacher",
                "structural_presence": {
                    "archive_bytes": http[archive_url]["content_length"],
                    "archive_identity_available": identity_available(http[archive_url]),
                    "publisher_scale_present": True,
                    "member_preflight_complete": False,
                },
            }
        )

    objects.sort(key=lambda item: item["inventory_object_id"])
    alias_edges.sort(key=lambda item: (item["left"], item["right"], item["decision"]))
    discrepancies.sort(key=lambda item: (item["code"], item["current"]["id"]))

    archives = []
    for url in urls:
        entry = http[url]
        if "RealImpact" in url:
            granularity = "object"
            source_id = "realimpact_fca2"
            preflight = entry["content_length"]
            method = "zip_tail_if_ranges_else_full_object"
        elif "ObjectFolder_Real" in url:
            granularity = "batch_10"
            source_id = "objectfolder_real_current"
            preflight = entry["content_length"]
            method = "full_gzip_tar_batch"
        else:
            granularity = "batch_100"
            source_id = "objectfolder2_3c6c"
            preflight = entry["content_length"]
            method = "full_gzip_tar_batch"
        archives.append(
            {
                "accept_ranges": entry["accept_ranges"],
                "acquisition_granularity": granularity,
                "content_length": entry["content_length"],
                "etag": entry["etag"],
                "identity_available": identity_available(entry),
                "last_modified": entry["last_modified"],
                "minimum_member_preflight_bytes": preflight,
                "preflight_cost_method": method,
                "source_id": source_id,
                "url": url,
            }
        )

    candidate_counts = {
        material: {
            "all_real_metadata_candidates": sum(
                item["material_family"] == material
                and item["source_tier"] in {"T2_sparse_real_contact", "T3_dense_real_listener"}
                and item["candidate_state"] in {"metadata_candidate", "member_preflight_required"}
                for item in objects
            ),
            "unexposed_real_metadata_candidates": sum(
                item["material_family"] == material
                and item["source_tier"] in {"T2_sparse_real_contact", "T3_dense_real_listener"}
                and item["candidate_state"] in {"metadata_candidate", "member_preflight_required"}
                and item["exposure"]["state"] == "unexposed"
                for item in objects
            ),
            "synthetic_teacher_candidates": sum(
                item["material_family"] == material
                and item["source_tier"] == "T1_synthetic_teacher"
                and item["candidate_state"] == "member_preflight_required"
                for item in objects
            ),
        }
        for material in MATERIALS
    }
    full_shape_potential = all(
        counts["unexposed_real_metadata_candidates"] >= 8
        for counts in candidate_counts.values()
    )
    decision = (
        "N1B_METADATA_INVENTORY_PASS_N1C_MEMBER_PREFLIGHT_NEXT"
        if full_shape_potential
        else "N1B_COVERAGE_INSUFFICIENT_FOR_FULL_SHAPE"
    )
    cached_hashed = sum(item["bytes"] for item in input_identities.values())
    cached_traversed = sum(
        input_identities[key]["bytes"] for key in ("audio", "contacts", "geometry")
    )
    access = {
        "cached_archive_bytes_hashed": cached_traversed,
        "cached_archive_bytes_traversed": cached_traversed,
        "cached_total_bytes_hashed": cached_hashed,
        "force_sample_values_decoded": 0,
        "json_scalar_values_decoded": ledger_v0.scalar_count(ledger_raw) + ledger_v0.scalar_count(parse_json(raw["split"], "split counter")) + ledger_v0.scalar_count(parse_json(raw["scale"], "scale counter")),
        "network_archive_body_bytes": 0,
        "network_requests": 0,
        "npy_headers_parsed": 0,
        "pcm_sample_values_decoded": 0,
        "protected_signal_values_decoded": 0,
        "source_payload_members_extracted": 0,
        "tar_member_headers_seen": audio_headers + contact_headers + geometry_headers,
        "wav_headers_parsed": 0,
    }
    inventory = {
        "access": access,
        "alias_edges": alias_edges,
        "archives": archives,
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "discrepancies": discrepancies,
        "input_identities": input_identities,
        "objects": objects,
        "publisher_claims": [
            {"claim": "objectfolder_real_object_count", "publisher_value": 100, "observed_value": 100},
            {"claim": "contact_localization_real_object_count", "publisher_value": 53, "observed_value": len(split)},
            {"claim": "objectfolder2_object_count", "publisher_value": 1000, "observed_value": len(objectfolder2)},
            {"claim": "realimpact_object_count", "publisher_value": 50, "observed_value": len(realimpact)},
        ],
        "schema": INVENTORY_SCHEMA,
        "sources": source_records(http_sha256),
    }
    inventory_bytes = canonical_json(inventory)
    cost_by_source = {}
    for source_id in ("objectfolder_real_current", "objectfolder2_3c6c", "realimpact_fca2"):
        source_archives = [item for item in archives if item["source_id"] == source_id]
        cost_by_source[source_id] = {
            "archive_count": len(source_archives),
            "identity_available_count": sum(item["identity_available"] for item in source_archives),
            "remote_archive_bytes": sum(item["content_length"] or 0 for item in source_archives),
            "whole_archive_preflight_bytes": sum(item["minimum_member_preflight_bytes"] or 0 for item in source_archives),
        }
    costs = {
        "authority": {"public_contract": False, "runtime_consumer_allowed": False},
        "by_source": cost_by_source,
        "candidate_counts": candidate_counts,
        "http_request_ceiling": 70,
        "inventory_sha256": sha256_bytes(inventory_bytes),
        "signal_values_decoded": {"force": 0, "pcm": 0, "protected": 0},
        "schema": COST_SCHEMA,
    }
    costs_bytes = canonical_json(costs)
    report = {
        "access": access,
        "candidate_counts": candidate_counts,
        "counts": {
            "alias_edge_count": len(alias_edges),
            "archive_count": len(archives),
            "compact_split_object_count": len(split),
            "compact_split_pair_count": split_pairs,
            "discrepancy_count": len(discrepancies),
            "object_row_count": len(objects),
            "source_count": 4,
        },
        "costs_sha256": sha256_bytes(costs_bytes),
        "decision": decision,
        "full_n1a_shape_potential": full_shape_potential,
        "inventory_sha256": sha256_bytes(inventory_bytes),
        "public_contract": False,
        "runtime_consumer_allowed": False,
        "schema": REPORT_SCHEMA,
    }
    return {
        "costs.json": costs_bytes,
        "inventory.json": inventory_bytes,
        "report.json": canonical_json(report),
    }


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def prepare_output(output: Path) -> Path:
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise SourceInventoryError("output must remain outside the repository")
    if resolved.exists():
        raise SourceInventoryError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def publish(output: Path, documents: dict[str, bytes]) -> Path:
    destination = prepare_output(output)
    staging = Path(tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent))
    try:
        for name, data in sorted(documents.items()):
            (staging / name).write_bytes(data)
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def build_inventory(inputs: Inputs, output: Path) -> Path:
    return publish(output, build_documents(inputs))


def acquire_one(url: str, timeout_seconds: float) -> dict[str, Any]:
    request = urllib.request.Request(url, method="HEAD", headers={"User-Agent": "NextEngine-PhysicalSound-N1b/1"})
    try:
        with urllib.request.urlopen(request, timeout=timeout_seconds) as response:
            final_url = response.geturl()
            status = response.status
            headers = response.headers
            error = None
    except urllib.error.HTTPError as caught:
        final_url = caught.geturl()
        status = caught.code
        headers = caught.headers
        error = f"HTTPError:{caught.code}"
    except (urllib.error.URLError, TimeoutError, OSError) as caught:
        return {
            "accept_ranges": None,
            "content_length": None,
            "error": type(caught).__name__,
            "etag": None,
            "final_url": None,
            "last_modified": None,
            "requested_url": url,
            "status": None,
        }
    length_text = headers.get("Content-Length")
    length = int(length_text) if length_text is not None and length_text.isdigit() else None
    return {
        "accept_ranges": headers.get("Accept-Ranges"),
        "content_length": length,
        "error": error,
        "etag": headers.get("ETag"),
        "final_url": final_url,
        "last_modified": headers.get("Last-Modified"),
        "requested_url": url,
        "status": status,
    }


def acquire_snapshot(
    names_path: Path,
    output: Path,
    *,
    timeout_seconds: float = 20.0,
    workers: int = 1,
) -> Path:
    names_bytes = read_bound_file(names_path, "RealImpact names", REAL_INPUTS["realimpact_names"])
    names = parse_realimpact_names(names_bytes)
    urls = expected_urls(names.values())
    destination = output.resolve()
    if destination.exists() or destination.is_dir():
        raise SourceInventoryError(f"refusing to replace HTTP snapshot: {destination}")
    if destination == repository_root().resolve(strict=True) or destination.is_relative_to(repository_root().resolve(strict=True)):
        raise SourceInventoryError("HTTP snapshot must remain outside the repository")
    destination.parent.mkdir(parents=True, exist_ok=True)
    if workers < 1 or workers > 8:
        raise SourceInventoryError("HTTP acquisition workers must be in 1..8")
    with ThreadPoolExecutor(max_workers=workers) as executor:
        entries = list(executor.map(lambda url: acquire_one(url, timeout_seconds), urls))
    snapshot = {
        "acquired_at_utc": datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z"),
        "entries": sorted(entries, key=lambda item: item["requested_url"]),
        "schema": HTTP_SCHEMA,
    }
    data = canonical_json(snapshot)
    handle, temporary = tempfile.mkstemp(prefix=f".{destination.name}.", dir=destination.parent)
    try:
        with os.fdopen(handle, "wb") as stream:
            stream.write(data)
        os.replace(temporary, destination)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    return destination


def add_input_arguments(parser: argparse.ArgumentParser) -> None:
    for name in Inputs.__dataclass_fields__:
        parser.add_argument(f"--{name.replace('_', '-')}", required=True, type=Path)


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="stage", required=True)
    acquire = subparsers.add_parser("acquire")
    acquire.add_argument("--realimpact-names", required=True, type=Path)
    acquire.add_argument("--output", required=True, type=Path)
    acquire.add_argument("--timeout-seconds", default=20.0, type=float)
    acquire.add_argument("--workers", default=1, type=int)
    build = subparsers.add_parser("build")
    add_input_arguments(build)
    build.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    if arguments.stage == "acquire":
        result = acquire_snapshot(
            arguments.realimpact_names,
            arguments.output,
            timeout_seconds=arguments.timeout_seconds,
            workers=arguments.workers,
        )
    else:
        inputs = Inputs(**{name: getattr(arguments, name) for name in Inputs.__dataclass_fields__})
        result = build_inventory(inputs, arguments.output)
    print(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
