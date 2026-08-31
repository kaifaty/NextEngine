#!/usr/bin/env python3
"""Build the V15-S0b metadata-only YCB capability and cost inventory."""

from __future__ import annotations

import argparse
import html
import hashlib
import io
import json
import os
import re
import shutil
import tempfile
import urllib.parse
import urllib.request
import zipfile
from collections import defaultdict, deque
from pathlib import Path, PurePosixPath
from typing import Any, Callable
from xml.etree import ElementTree

SNAPSHOT_SCHEMA = "nextengine.experimental-physical-sound-ycb-osf-snapshot.v1"
INVENTORY_SCHEMA = "nextengine.experimental-physical-sound-ycb-capability.v1"
COST_SCHEMA = "nextengine.experimental-physical-sound-ycb-cost-census.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-ycb-capability.report.v1"

WORKBOOK_IDENTITY = (
    7_994,
    "27672ecfdfaf2a1ecfc8926127ab9ab962e9cb59adb593b141caadc0459ae0fd",
)
MODEL_INDEX_IDENTITY = (
    105_935,
    "d90b97268e9fd4eaf501efc22d17e27d419e9d94f498c6f6b6f63defd2a49891",
)
PAPER_IDENTITY = (
    7_130_560,
    "13fcf1fe0adce22a9e108c95baaeaf0c0f9a06e2fbd04cb60ee810d10254554b",
)

PROJECT_ID = "4tcp6"
PROJECT_MODIFIED = "2022-09-27T11:37:09.232408"
COMPONENTS = {
    "bj5w8": ("Robot_impact_Data", "2022-09-27T11:36:47.167853"),
    "hjdby": ("Manually_generated_Data", "2022-03-04T12:41:07.255807"),
}
OSF_API_HOST = "api.osf.io"
MODEL_INDEX_BASE = "https://ycb-benchmarks.s3.amazonaws.com/"
MAX_REQUESTS = 1_024
MAX_METADATA_BYTES = 64 * 1024 * 1024
MAX_RESPONSE_BYTES = 8 * 1024 * 1024
MAX_INPUT_BYTES = 16 * 1024 * 1024
MAX_ENTRIES = 50_000

TARGET_MATERIALS = {"Glass": "Glass", "Steel": "Metal", "Wood": "Wood"}
ADAPTER_REFERENCED_YCB_IDS = {
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "11",
    "13",
    "19",
    "20",
    "23",
    "24",
    "25",
    "26",
    "27",
    "28",
    "29",
    "36",
    "38",
    "41",
    "48",
    "51",
    "60",
    "61",
    "65",
    "68",
    "70",
    "71",
}

# Narrow publisher-name aliases. Numeric equality without one of these slugs is
# intentionally insufficient for a recording-parent binding.
OBJECT_SLUG_ALIASES = {
    "7": {"tunafishcan"},
    "39": {"key"},
    "43": {"phillipsscrewdriver"},
    "70": {"coloredwoodblocks"},
    "72": {"toyairplane"},
}

SNAPSHOT_KEYS = {"schema", "project", "components", "entries", "access"}
PROJECT_KEYS = {"id", "title", "date_modified"}
COMPONENT_KEYS = {"id", "title", "date_modified"}
ENTRY_KEYS = {
    "component_id",
    "id",
    "kind",
    "name",
    "materialized_path",
    "parent_id",
    "size",
    "date_created",
    "date_modified",
    "current_version",
    "hashes",
}
HASH_KEYS = {"md5", "sha256"}
ACCESS_KEYS = {
    "network_requests",
    "metadata_response_bytes",
    "audio_body_bytes",
    "video_body_bytes",
    "force_body_bytes",
    "mesh_body_bytes",
    "archive_body_bytes",
    "payload_members_opened",
    "signal_values_decoded",
}
ZERO_BODY_ACCESS = {
    "audio_body_bytes": 0,
    "video_body_bytes": 0,
    "force_body_bytes": 0,
    "mesh_body_bytes": 0,
    "archive_body_bytes": 0,
    "payload_members_opened": 0,
    "signal_values_decoded": 0,
}

Fetch = Callable[[str], tuple[str, str, bytes]]


class YcbCapabilityError(RuntimeError):
    """The S0b input or output violates the frozen metadata-only protocol."""


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--stage", required=True, choices=["acquire", "build"])
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--workbook", type=Path)
    parser.add_argument("--model-index", type=Path)
    parser.add_argument("--paper", type=Path)
    parser.add_argument("--osf-snapshot", type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        text = json.dumps(value, indent=2, sort_keys=True, allow_nan=False)
    except (TypeError, ValueError) as error:
        raise YcbCapabilityError(f"cannot encode canonical JSON: {error}") from error
    return (text + "\n").encode("utf-8")


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def reject_constant(value: str) -> None:
    raise YcbCapabilityError(f"non-finite JSON number is forbidden: {value}")


def reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise YcbCapabilityError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def parse_json_bytes(data: bytes, context: str, *, canonical: bool = False) -> Any:
    if len(data) > MAX_INPUT_BYTES:
        raise YcbCapabilityError(f"{context} exceeds frozen byte limit")
    try:
        value = json.loads(
            data,
            object_pairs_hook=reject_duplicate_keys,
            parse_constant=reject_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, RecursionError) as error:
        raise YcbCapabilityError(f"cannot parse {context}: {error}") from error
    if canonical and canonical_json(value) != data:
        raise YcbCapabilityError(f"{context} is not canonical JSON")
    return value


def require_exact_keys(value: Any, expected: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise YcbCapabilityError(f"{context} must be an object")
    actual = set(value)
    if actual != expected:
        raise YcbCapabilityError(
            f"{context} keys changed; missing={sorted(expected - actual)}, "
            f"unknown={sorted(actual - expected)}"
        )
    return value


def require_string(value: Any, context: str, *, allow_empty: bool = False) -> str:
    if not isinstance(value, str) or (not allow_empty and not value) or len(value) > 512:
        raise YcbCapabilityError(f"{context} must be a bounded string")
    return value


def require_nonnegative_integer(value: Any, context: str) -> int:
    if type(value) is not int or value < 0:
        raise YcbCapabilityError(f"{context} must be a non-negative integer")
    return value


def hash_regular_file(path: Path, context: str) -> tuple[int, str]:
    if path.is_symlink() or not path.is_file():
        raise YcbCapabilityError(f"{context} must be a regular file")
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
        raise YcbCapabilityError(f"{context} changed while hashing")
    return size, digest.hexdigest()


def read_bound_file(
    path: Path, context: str, expected: tuple[int, str] | None
) -> bytes:
    identity = hash_regular_file(path, context)
    if expected is not None and identity != expected:
        raise YcbCapabilityError(
            f"{context} identity changed: got bytes={identity[0]}, sha256={identity[1]}"
        )
    return path.read_bytes()


def normalized_materialized_path(
    value: Any, context: str, *, kind: str | None = None
) -> str:
    path = require_string(value, context)
    if not path.startswith("/") or "\\" in path:
        raise YcbCapabilityError(f"{context} is not a normalized materialized path")
    parts = PurePosixPath(path).parts
    if any(part in {".", ".."} for part in parts):
        raise YcbCapabilityError(f"{context} contains traversal")
    normalized = "/" + "/".join(part for part in parts if part != "/")
    if kind == "folder" or (kind is None and path.endswith("/")):
        normalized += "/"
    if kind == "file" and path.endswith("/"):
        raise YcbCapabilityError(f"{context} gives a file a folder path")
    if kind == "folder" and not path.endswith("/"):
        raise YcbCapabilityError(f"{context} gives a folder a file path")
    if normalized != path or "\x00" in path:
        raise YcbCapabilityError(f"{context} is not normalized")
    return path


def validate_osf_url(url: str, component_id: str | None = None) -> str:
    parsed = urllib.parse.urlsplit(url)
    if parsed.scheme != "https" or parsed.hostname != OSF_API_HOST or parsed.port:
        raise YcbCapabilityError(f"OSF metadata URL escaped allowed host: {url}")
    if component_id is None:
        if not re.fullmatch(r"/v2/nodes/[a-z0-9]+/", parsed.path):
            raise YcbCapabilityError(f"unexpected OSF node URL: {url}")
    elif not parsed.path.startswith(f"/v2/nodes/{component_id}/files/osfstorage/"):
        raise YcbCapabilityError(f"OSF relationship escaped component {component_id}: {url}")
    query = urllib.parse.parse_qs(parsed.query, keep_blank_values=True)
    if not set(query).issubset({"page", "page[size]"}):
        raise YcbCapabilityError(f"unexpected OSF metadata query: {url}")
    return url


def default_fetch(url: str) -> tuple[str, str, bytes]:
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/vnd.api+json", "User-Agent": "NextEngine-S0b/1"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            final_url = response.geturl()
            content_type = response.headers.get_content_type()
            declared = response.headers.get("Content-Length")
            if declared is not None and int(declared) > MAX_RESPONSE_BYTES:
                raise YcbCapabilityError("OSF metadata response exceeds per-request limit")
            body = response.read(MAX_RESPONSE_BYTES + 1)
    except (OSError, ValueError) as error:
        raise YcbCapabilityError(f"OSF metadata request failed: {url}: {error}") from error
    if len(body) > MAX_RESPONSE_BYTES:
        raise YcbCapabilityError("OSF metadata response exceeds per-request limit")
    return final_url, content_type, body


def add_page_size(url: str) -> str:
    parsed = urllib.parse.urlsplit(url)
    query = urllib.parse.parse_qs(parsed.query, keep_blank_values=True)
    query.setdefault("page[size]", ["100"])
    encoded = urllib.parse.urlencode(query, doseq=True)
    return urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, parsed.path, encoded, ""))


def fetch_json(
    url: str,
    *,
    fetch: Fetch,
    component_id: str | None,
    counters: dict[str, int],
) -> dict[str, Any]:
    if counters["network_requests"] >= MAX_REQUESTS:
        raise YcbCapabilityError("OSF metadata request limit exceeded")
    validate_osf_url(url, component_id)
    final_url, content_type, body = fetch(url)
    validate_osf_url(final_url, component_id)
    if content_type not in {"application/json", "application/vnd.api+json"}:
        raise YcbCapabilityError(f"OSF metadata response is not JSON: {content_type}")
    counters["network_requests"] += 1
    counters["metadata_response_bytes"] += len(body)
    if counters["metadata_response_bytes"] > MAX_METADATA_BYTES:
        raise YcbCapabilityError("OSF metadata byte limit exceeded")
    value = parse_json_bytes(body, f"OSF response {url}")
    if not isinstance(value, dict):
        raise YcbCapabilityError("OSF response must be an object")
    return value


def normalized_node(value: Any, expected_id: str) -> dict[str, str]:
    data = value.get("data") if isinstance(value, dict) else None
    if not isinstance(data, dict) or data.get("id") != expected_id or data.get("type") != "nodes":
        raise YcbCapabilityError(f"OSF node identity changed for {expected_id}")
    attributes = data.get("attributes")
    if not isinstance(attributes, dict):
        raise YcbCapabilityError(f"OSF node attributes missing for {expected_id}")
    return {
        "date_modified": require_string(
            attributes.get("date_modified"), f"node {expected_id}.date_modified"
        ),
        "id": expected_id,
        "title": require_string(attributes.get("title"), f"node {expected_id}.title"),
    }


def normalize_osf_entry(value: Any, component_id: str) -> tuple[dict[str, Any], str | None]:
    if not isinstance(value, dict) or value.get("type") != "files":
        raise YcbCapabilityError("OSF file entry has unexpected type")
    entry_id = require_string(value.get("id"), "OSF entry id")
    attributes = value.get("attributes")
    relationships = value.get("relationships")
    if not isinstance(attributes, dict) or not isinstance(relationships, dict):
        raise YcbCapabilityError(f"OSF entry {entry_id} lacks attributes/relationships")
    kind = attributes.get("kind")
    if kind not in {"file", "folder"}:
        raise YcbCapabilityError(f"OSF entry {entry_id} has unknown kind")
    parent_data = relationships.get("parent_folder", {}).get("data")
    parent_id = None
    if parent_data is not None:
        if not isinstance(parent_data, dict):
            raise YcbCapabilityError(f"OSF entry {entry_id} has malformed parent")
        parent_id = require_string(parent_data.get("id"), f"OSF entry {entry_id}.parent")
    hashes = attributes.get("extra", {}).get("hashes")
    if not isinstance(hashes, dict):
        raise YcbCapabilityError(f"OSF entry {entry_id} lacks hashes")
    normalized_hashes: dict[str, str | None] = {}
    for key in ("md5", "sha256"):
        item = hashes.get(key)
        if item is not None and (not isinstance(item, str) or not re.fullmatch(r"[0-9a-f]+", item)):
            raise YcbCapabilityError(f"OSF entry {entry_id} has malformed {key}")
        normalized_hashes[key] = item
    size = attributes.get("size")
    if size is not None:
        size = require_nonnegative_integer(size, f"OSF entry {entry_id}.size")
    version = require_nonnegative_integer(
        attributes.get("current_version"), f"OSF entry {entry_id}.current_version"
    )
    entry = {
        "component_id": component_id,
        "current_version": version,
        "date_created": attributes.get("date_created"),
        "date_modified": attributes.get("date_modified"),
        "hashes": normalized_hashes,
        "id": entry_id,
        "kind": kind,
        "materialized_path": normalized_materialized_path(
            attributes.get("materialized_path"),
            f"OSF entry {entry_id}.materialized_path",
            kind=kind,
        ),
        "name": require_string(attributes.get("name"), f"OSF entry {entry_id}.name"),
        "parent_id": parent_id,
        "size": size,
    }
    for key in ("date_created", "date_modified"):
        if entry[key] is not None:
            require_string(entry[key], f"OSF entry {entry_id}.{key}")
    child_url = None
    if kind == "folder":
        child_url = relationships.get("files", {}).get("links", {}).get("related", {}).get("href")
        if not isinstance(child_url, str):
            raise YcbCapabilityError(f"OSF folder {entry_id} lacks child relationship")
        child_url = add_page_size(child_url)
        validate_osf_url(child_url, component_id)
    return entry, child_url


def acquire_osf_snapshot(fetch: Fetch = default_fetch) -> dict[str, Any]:
    counters = {"network_requests": 0, "metadata_response_bytes": 0}
    project_value = fetch_json(
        f"https://api.osf.io/v2/nodes/{PROJECT_ID}/",
        fetch=fetch,
        component_id=None,
        counters=counters,
    )
    project = normalized_node(project_value, PROJECT_ID)
    if (
        project["title"] != "YCB-impact sounds dataset"
        or project["date_modified"] != PROJECT_MODIFIED
    ):
        raise YcbCapabilityError("OSF project identity/revision changed")

    components = []
    entries: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    seen_paths: set[tuple[str, str]] = set()
    for component_id, (expected_title, expected_modified) in sorted(COMPONENTS.items()):
        node_value = fetch_json(
            f"https://api.osf.io/v2/nodes/{component_id}/",
            fetch=fetch,
            component_id=None,
            counters=counters,
        )
        component = normalized_node(node_value, component_id)
        if component["title"] != expected_title or component["date_modified"] != expected_modified:
            raise YcbCapabilityError(f"OSF component revision changed for {component_id}")
        components.append(component)
        queue = deque(
            [
                add_page_size(
                    f"https://api.osf.io/v2/nodes/{component_id}/files/osfstorage/"
                )
            ]
        )
        seen_urls: set[str] = set()
        while queue:
            url = queue.popleft()
            if url in seen_urls:
                raise YcbCapabilityError(f"duplicate OSF collection URL: {url}")
            seen_urls.add(url)
            page = fetch_json(
                url, fetch=fetch, component_id=component_id, counters=counters
            )
            data = page.get("data")
            links = page.get("links")
            if not isinstance(data, list) or not isinstance(links, dict):
                raise YcbCapabilityError("OSF collection lacks data/links")
            for raw_entry in data:
                entry, child_url = normalize_osf_entry(raw_entry, component_id)
                if entry["id"] in seen_ids:
                    raise YcbCapabilityError(f"duplicate OSF entry id: {entry['id']}")
                path_key = (component_id, entry["materialized_path"])
                if path_key in seen_paths:
                    raise YcbCapabilityError(f"duplicate OSF materialized path: {path_key}")
                seen_ids.add(entry["id"])
                seen_paths.add(path_key)
                entries.append(entry)
                if child_url is not None:
                    queue.append(child_url)
                if len(entries) > MAX_ENTRIES:
                    raise YcbCapabilityError("OSF entry limit exceeded")
            next_url = links.get("next")
            if next_url is not None:
                if not isinstance(next_url, str):
                    raise YcbCapabilityError("OSF next-page link is malformed")
                validate_osf_url(next_url, component_id)
                queue.append(next_url)

    entries.sort(key=lambda item: (item["component_id"], item["materialized_path"], item["id"]))
    access = {
        "metadata_response_bytes": counters["metadata_response_bytes"],
        "network_requests": counters["network_requests"],
        **ZERO_BODY_ACCESS,
    }
    return {
        "access": access,
        "components": components,
        "entries": entries,
        "project": project,
        "schema": SNAPSHOT_SCHEMA,
    }


def validate_osf_snapshot(value: Any) -> dict[str, Any]:
    snapshot = require_exact_keys(value, SNAPSHOT_KEYS, "OSF snapshot")
    if snapshot["schema"] != SNAPSHOT_SCHEMA:
        raise YcbCapabilityError("unknown OSF snapshot schema")
    project = require_exact_keys(snapshot["project"], PROJECT_KEYS, "OSF project")
    if project != {
        "date_modified": PROJECT_MODIFIED,
        "id": PROJECT_ID,
        "title": "YCB-impact sounds dataset",
    }:
        raise YcbCapabilityError("OSF project identity changed")
    components = snapshot["components"]
    if not isinstance(components, list) or len(components) != len(COMPONENTS):
        raise YcbCapabilityError("OSF component roster changed")
    for component in components:
        require_exact_keys(component, COMPONENT_KEYS, "OSF component")
    expected_components = [
        {"date_modified": modified, "id": item, "title": title}
        for item, (title, modified) in sorted(COMPONENTS.items())
    ]
    if components != expected_components:
        raise YcbCapabilityError("OSF component identity/revision changed")
    access = require_exact_keys(snapshot["access"], ACCESS_KEYS, "OSF access")
    request_count = require_nonnegative_integer(
        access["network_requests"], "OSF network_requests"
    )
    metadata_bytes = require_nonnegative_integer(
        access["metadata_response_bytes"], "OSF metadata_response_bytes"
    )
    if request_count > MAX_REQUESTS or metadata_bytes > MAX_METADATA_BYTES:
        raise YcbCapabilityError("OSF snapshot exceeds frozen acquisition limits")
    if any(access[key] != value for key, value in ZERO_BODY_ACCESS.items()):
        raise YcbCapabilityError("OSF snapshot records forbidden payload or signal access")
    entries = snapshot["entries"]
    if not isinstance(entries, list) or len(entries) > MAX_ENTRIES:
        raise YcbCapabilityError("OSF entries must be a bounded array")
    seen_ids: set[str] = set()
    seen_paths: set[tuple[str, str]] = set()
    for index, entry in enumerate(entries):
        require_exact_keys(entry, ENTRY_KEYS, f"OSF entries[{index}]")
        component_id = entry["component_id"]
        if component_id not in COMPONENTS:
            raise YcbCapabilityError("OSF entry has unknown component")
        entry_id = require_string(entry["id"], f"OSF entries[{index}].id")
        kind = entry["kind"]
        if kind not in {"file", "folder"}:
            raise YcbCapabilityError("OSF entry has unknown kind")
        path = normalized_materialized_path(
            entry["materialized_path"],
            f"OSF entries[{index}].materialized_path",
            kind=kind,
        )
        if entry_id in seen_ids or (component_id, path) in seen_paths:
            raise YcbCapabilityError("duplicate OSF entry id/path")
        seen_ids.add(entry_id)
        seen_paths.add((component_id, path))
        require_string(entry["name"], f"OSF entries[{index}].name")
        if entry["parent_id"] is not None:
            require_string(entry["parent_id"], f"OSF entries[{index}].parent_id")
        if entry["size"] is not None:
            require_nonnegative_integer(entry["size"], f"OSF entries[{index}].size")
        require_nonnegative_integer(
            entry["current_version"], f"OSF entries[{index}].current_version"
        )
        hashes = require_exact_keys(entry["hashes"], HASH_KEYS, "OSF hashes")
        for key, digest in hashes.items():
            if digest is not None and (
                not isinstance(digest, str)
                or not re.fullmatch(
                    r"[0-9a-f]{32}" if key == "md5" else r"[0-9a-f]{64}", digest
                )
            ):
                raise YcbCapabilityError(f"invalid OSF {key}")
    expected_order = sorted(
        entries, key=lambda item: (item["component_id"], item["materialized_path"], item["id"])
    )
    if entries != expected_order:
        raise YcbCapabilityError("OSF entries are not canonically ordered")
    return snapshot


def parse_workbook(data: bytes) -> list[dict[str, str | None]]:
    try:
        with zipfile.ZipFile(io.BytesIO(data)) as workbook:
            shared_data = workbook.read("xl/sharedStrings.xml")
            sheet_data = workbook.read("xl/worksheets/sheet1.xml")
    except (zipfile.BadZipFile, KeyError) as error:
        raise YcbCapabilityError(f"cannot read frozen YCB workbook: {error}") from error
    namespace = "{http://schemas.openxmlformats.org/spreadsheetml/2006/main}"
    try:
        shared_root = ElementTree.fromstring(shared_data)
        sheet_root = ElementTree.fromstring(sheet_data)
    except ElementTree.ParseError as error:
        raise YcbCapabilityError(f"cannot parse frozen YCB workbook XML: {error}") from error
    shared = ["".join(node.itertext()) for node in shared_root.findall(f"{namespace}si")]
    rows: list[list[str | None]] = []
    for row in sheet_root.findall(f".//{namespace}row"):
        cells: dict[int, str | None] = {}
        for cell in row.findall(f"{namespace}c"):
            reference = cell.get("r", "")
            match = re.fullmatch(r"([A-Z]+)[0-9]+", reference)
            if not match or len(match.group(1)) != 1:
                raise YcbCapabilityError("YCB workbook contains unexpected cell reference")
            column = ord(match.group(1)) - ord("A")
            value_node = cell.find(f"{namespace}v")
            value = None if value_node is None else value_node.text
            if cell.get("t") == "s" and value is not None:
                try:
                    value = shared[int(value)]
                except (ValueError, IndexError) as error:
                    raise YcbCapabilityError("YCB workbook shared string is invalid") from error
            cells[column] = value
        rows.append([cells.get(index) for index in range(4)])
    if not rows or rows[0] != ["ID", "OBJECT", "Primary Material", "Secondary Material"]:
        raise YcbCapabilityError("YCB workbook headers changed")
    result = []
    seen: set[str] = set()
    for row_index, row in enumerate(rows[1:], start=2):
        raw_id, name, primary, secondary = row
        if raw_id is None or name is None or primary is None:
            raise YcbCapabilityError(f"YCB workbook row {row_index} is incomplete")
        if re.fullmatch(r"[0-9]+(?:\.0)?", raw_id):
            object_id = str(int(float(raw_id)))
        elif raw_id in {"x_01", "x_02"}:
            object_id = raw_id
        else:
            raise YcbCapabilityError(f"YCB workbook row {row_index} has invalid ID")
        if object_id in seen:
            raise YcbCapabilityError(f"duplicate YCB workbook ID: {object_id}")
        seen.add(object_id)
        result.append(
            {
                "object_id": object_id,
                "object_name": name,
                "primary_material": primary,
                "secondary_material": secondary or None,
            }
        )
    if seen != {str(index) for index in range(1, 76)} | {"x_01", "x_02"}:
        raise YcbCapabilityError("YCB workbook must contain exact IDs 1..75 and x_01/x_02")
    return result


def route_from_cell(cell: dict[str, Any], kind: str) -> dict[str, Any]:
    hrefs = cell["hrefs"]
    sups = cell["sups"]
    if len(hrefs) > 1:
        raise YcbCapabilityError(f"YCB model-index {kind} cell has multiple routes")
    if hrefs:
        url = urllib.parse.urljoin(MODEL_INDEX_BASE, hrefs[0])
        parsed = urllib.parse.urlsplit(url)
        if (
            parsed.scheme != "https"
            or parsed.hostname != "ycb-benchmarks.s3.amazonaws.com"
            or not parsed.path.startswith("/data/")
            or PurePosixPath(parsed.path).suffix != ".tgz"
        ):
            raise YcbCapabilityError(f"YCB model route escaped publisher: {url}")
        state = "route_distorted" if "3" in sups else "route_available"
        return {"kind": kind, "state": state, "url": url}
    if "3" in sups:
        raise YcbCapabilityError("YCB model index marks an absent route as distorted")
    if "2" in sups:
        state = "route_unavailable_transparency_or_size"
    elif "1" in sups:
        state = "route_pending"
    else:
        state = "route_absent"
    return {"kind": kind, "state": state, "url": None}


def parse_model_index(data: bytes) -> dict[str, list[dict[str, Any]]]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise YcbCapabilityError("YCB model index is not UTF-8") from error
    # The publisher page contains a few missing </td> tags. Splitting a bounded
    # object row at each opening <td> is deterministic and preserves the seven
    # logical columns even for those malformed rows.
    row_blocks = re.findall(
        r"<tr\s+class=['\"]object_row['\"][^>]*>(.*?)</tr>",
        text,
        flags=re.IGNORECASE | re.DOTALL,
    )
    parsed_rows: list[list[dict[str, Any]]] = []
    for block in row_blocks:
        raw_cells = re.split(r"<td\b[^>]*>", block, flags=re.IGNORECASE)[1:]
        cells = []
        for raw_cell in raw_cells:
            without_comments = re.sub(r"<!--.*?-->", "", raw_cell, flags=re.DOTALL)
            hrefs = re.findall(
                r"href\s*=\s*['\"]([^'\"]+)['\"]",
                without_comments,
                flags=re.IGNORECASE,
            )
            sups = [
                " ".join(item.split())
                for item in re.findall(
                    r"<sup\b[^>]*>(.*?)</sup>",
                    without_comments,
                    flags=re.IGNORECASE | re.DOTALL,
                )
            ]
            plain = html.unescape(re.sub(r"<[^>]+>", " ", without_comments))
            cells.append({"hrefs": hrefs, "sups": sups, "text": " ".join(plain.split())})
        parsed_rows.append(cells)
    variants: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in parsed_rows:
        if len(row) < 7:
            raise YcbCapabilityError("YCB model-index row has fewer than seven cells")
        publisher_name = row[0]["text"]
        match = re.fullmatch(r"0*([0-9]+)(?:-([a-z]))?[_-](.+)", publisher_name)
        if not match:
            raise YcbCapabilityError(f"cannot parse YCB model object name: {publisher_name}")
        object_id = str(int(match.group(1)))
        routes = [
            route_from_cell(row[1], "processed"),
            route_from_cell(row[4], "laser_16k"),
            route_from_cell(row[5], "laser_64k"),
            route_from_cell(row[6], "laser_512k"),
        ]
        variants[object_id].append(
            {
                "publisher_name": publisher_name,
                "variant": match.group(2),
                "routes": routes,
            }
        )
    if len(variants) < 50 or not set(variants).issubset(
        {str(index) for index in range(1, 78)}
    ):
        raise YcbCapabilityError("YCB model index numeric object roster is malformed")
    for value in variants.values():
        value.sort(key=lambda item: item["publisher_name"])
    return dict(sorted(variants.items(), key=lambda item: int(item[0])))


def slug(value: str) -> str:
    return "".join(character for character in value.lower() if character.isalnum())


def accepted_slugs(row: dict[str, str | None]) -> set[str]:
    object_id = str(row["object_id"])
    return {slug(str(row["object_name"]))} | OBJECT_SLUG_ALIASES.get(object_id, set())


def object_parent_match(folder_name: str, rows: dict[str, dict[str, str | None]]) -> str | None:
    match = re.fullmatch(r"0*([0-9]+)[_-](.+?)(?:_[0-9]+)?", folder_name)
    if not match:
        return None
    object_id = str(int(match.group(1)))
    row = rows.get(object_id)
    if row is None or slug(match.group(2)) not in accepted_slugs(row):
        return None
    return object_id


def descendant_cost(entries: list[dict[str, Any]], prefix: str) -> dict[str, int]:
    files = [
        item
        for item in entries
        if item["kind"] == "file" and item["materialized_path"].startswith(prefix)
    ]
    known = [item["size"] for item in files if item["size"] is not None]
    return {
        "declared_bytes": sum(known),
        "file_count": len(files),
        "unknown_size_file_count": len(files) - len(known),
    }


def recording_parents(
    snapshot: dict[str, Any], rows: dict[str, dict[str, str | None]]
) -> dict[str, list[dict[str, Any]]]:
    entries = snapshot["entries"]
    result: dict[str, list[dict[str, Any]]] = defaultdict(list)
    seen_parent_ids: set[str] = set()
    for entry in entries:
        if entry["component_id"] != "bj5w8" or entry["kind"] != "folder":
            continue
        path = entry["materialized_path"]
        parts = [part for part in PurePosixPath(path).parts if part != "/"]
        mode = None
        object_id = None
        conditions: list[str] = []
        if len(parts) == 3 and parts[0] == "Vertical_Pokes" and parts[1] in {
            "Known_Objects",
            "Unknown_Objects",
        }:
            object_id = object_parent_match(parts[2], rows)
            mode = "vertical-known" if parts[1] == "Known_Objects" else "vertical-unknown"
            conditions = ["top-poke-speed-unknown"]
        elif len(parts) >= 2 and parts[0] == "Horizontal_Pokes":
            matches = [
                candidate_id
                for candidate_id, row in rows.items()
                if slug(parts[-1]) in accepted_slugs(row)
            ]
            if len(matches) == 1:
                object_id = matches[0]
                mode = "horizontal-object"
                conditions = ["side-poke-14mm-s", "side-poke-25mm-s"]
        if object_id is None or mode is None:
            continue
        if entry["id"] in seen_parent_ids:
            raise YcbCapabilityError("duplicate recording parent binding")
        seen_parent_ids.add(entry["id"])
        cost = descendant_cost(entries, path)
        if cost["file_count"] == 0:
            continue
        result[object_id].append(
            {
                "acquisition_mode": mode,
                "conditions": conditions,
                "cost": cost,
                "group_id": f"ycb-impact--bj5w8--object-{int(object_id):03d}--parent-{entry['id']}",
                "materialized_path": path,
                "parent_id": entry["id"],
            }
        )
    for parents in result.values():
        parents.sort(key=lambda item: (item["acquisition_mode"], item["parent_id"]))
    return result


def geometry_for_object(
    object_id: str, model_variants: dict[str, list[dict[str, Any]]]
) -> dict[str, Any]:
    variants = model_variants.get(object_id, [])
    ambiguous = len(variants) > 1
    routes = [route for variant in variants for route in variant["routes"]]
    available = sum(route["url"] is not None for route in routes)
    return {
        "exact_payload_identity": False,
        "metric_scale_known": False,
        "route_count": available,
        "state": "variant_ambiguous"
        if ambiguous
        else ("route_available" if available else "route_absent"),
        "variants": variants,
    }


def capability_axes(has_recordings: bool, geometry: dict[str, Any]) -> dict[str, str]:
    return {
        "canonical_excitation": "partial" if has_recordings else "unknown",
        "contact_geometry_binding": "unknown",
        "geometry": "partial"
        if geometry["route_count"] and geometry["state"] != "variant_ambiguous"
        else "unknown",
        "geometry_scale": "unknown",
        "listener_condition": "partial" if has_recordings else "unknown",
        "material_identity": "known",
        "object_identity": "known",
        "recorded_response": "partial" if has_recordings else "unknown",
        "support_condition": "partial" if has_recordings else "unknown",
    }


def build_documents(
    workbook_data: bytes,
    model_index_data: bytes,
    paper_data: bytes,
    snapshot_data: bytes,
) -> dict[str, bytes]:
    workbook_rows = parse_workbook(workbook_data)
    model_variants = parse_model_index(model_index_data)
    snapshot_value = parse_json_bytes(snapshot_data, "OSF snapshot", canonical=True)
    snapshot = validate_osf_snapshot(snapshot_value)
    rows_by_id = {str(row["object_id"]): row for row in workbook_rows}
    parents_by_id = recording_parents(snapshot, rows_by_id)

    objects = []
    cost_materials: dict[str, dict[str, int]] = defaultdict(
        lambda: {
            "declared_recording_bytes": 0,
            "geometry_route_count": 0,
            "object_count": 0,
            "recording_file_count": 0,
            "recording_parent_count": 0,
            "unknown_geometry_route_size_count": 0,
            "unknown_recording_size_file_count": 0,
        }
    )
    for row in workbook_rows:
        object_id = str(row["object_id"])
        geometry = geometry_for_object(object_id, model_variants)
        parents = parents_by_id.get(object_id, [])
        axes = capability_axes(bool(parents), geometry)
        missing_training = sorted(
            key
            for key, value in axes.items()
            if key != "support_condition" and value != "known"
        )
        missing_evaluation = sorted(key for key, value in axes.items() if value != "known")
        material_family = TARGET_MATERIALS.get(str(row["primary_material"]))
        objects.append(
            {
                "capability_axes": axes,
                "evaluation_complete": False,
                "exposure_state": "repository_adapter_referenced"
                if object_id in ADAPTER_REFERENCED_YCB_IDS
                else "not_in_adapter_freshness_unassessed",
                "geometry": geometry,
                "material_family": material_family,
                "missing_evaluation_axes": missing_evaluation,
                "missing_training_axes": missing_training,
                "object_id": object_id,
                "object_name": row["object_name"],
                "primary_material": row["primary_material"],
                "recording_parents": parents,
                "secondary_material": row["secondary_material"],
                "training_usable": False,
            }
        )
        if material_family is not None:
            material_cost = cost_materials[material_family]
            material_cost["object_count"] += 1
            material_cost["geometry_route_count"] += geometry["route_count"]
            material_cost["unknown_geometry_route_size_count"] += geometry["route_count"]
            for parent in parents:
                material_cost["recording_parent_count"] += 1
                material_cost["recording_file_count"] += parent["cost"]["file_count"]
                material_cost["declared_recording_bytes"] += parent["cost"]["declared_bytes"]
                material_cost["unknown_recording_size_file_count"] += parent["cost"][
                    "unknown_size_file_count"
                ]

    inventory = {
        "acquisition_facts": {
            "horizontal_speeds_mm_s": [14, 25],
            "microphone": "Rode VideoMic Pro",
            "robot": "Kinova Gen 3 with Robotiq 2F-85 gripper",
            "sample_rate_hz": 44_100,
            "support": "table; exact material and fixture unpublished",
            "vertical_object_microphone_relation": "fixed; coordinates unpublished",
        },
        "objects": objects,
        "project": {
            "component_revision": COMPONENTS["bj5w8"][1],
            "id": "ycb-impact-sounds",
            "publisher": "iri-csic-upc-ctu",
        },
        "schema": INVENTORY_SCHEMA,
    }
    cost = {
        "materials": dict(sorted(cost_materials.items())),
        "metadata_acquisition": {
            "response_bytes": snapshot["access"]["metadata_response_bytes"],
            "requests": snapshot["access"]["network_requests"],
        },
        "schema": COST_SCHEMA,
    }
    inventory_bytes = canonical_json(inventory)
    cost_bytes = canonical_json(cost)
    target_objects = [item for item in objects if item["material_family"] is not None]
    structural = {
        material: sum(
            bool(item["recording_parents"])
            and item["geometry"]["route_count"] > 0
            and item["geometry"]["state"] != "variant_ambiguous"
            for item in target_objects
            if item["material_family"] == material
        )
        for material in ("Glass", "Metal", "Wood")
    }
    report = {
        "access": {
            "build_network_requests": 0,
            "metadata_files_parsed": 4,
            "source_audio_body_bytes": 0,
            "source_force_body_bytes": 0,
            "source_mesh_body_bytes": 0,
            "source_payload_members_opened": 0,
            "source_pcm_values_decoded": 0,
            "source_protected_values_decoded": 0,
            "source_video_body_bytes": 0,
        },
        "counts": {
            "object_count": len(objects),
            "recording_parent_count": sum(len(item["recording_parents"]) for item in objects),
            "structural_candidates_by_material": structural,
            "target_object_count": len(target_objects),
            "training_usable_count": 0,
            "evaluation_complete_count": 0,
        },
        "decision": "S0B_YCB_CAPABILITY_PASS_S0C_ROLE_FREEZE_NEXT",
        "inputs": {
            "model_index": {
                "bytes": len(model_index_data),
                "sha256": sha256_bytes(model_index_data),
            },
            "osf_snapshot": {
                "bytes": len(snapshot_data),
                "sha256": sha256_bytes(snapshot_data),
            },
            "paper": {
                "bytes": len(paper_data),
                "sha256": sha256_bytes(paper_data),
            },
            "workbook": {
                "bytes": len(workbook_data),
                "sha256": sha256_bytes(workbook_data),
            },
        },
        "outputs": {
            "capability_inventory_sha256": sha256_bytes(inventory_bytes),
            "cost_census_sha256": sha256_bytes(cost_bytes),
        },
        "schema": REPORT_SCHEMA,
    }
    return {
        "capability-inventory.json": inventory_bytes,
        "cost-census.json": cost_bytes,
        "report.json": canonical_json(report),
    }


def ensure_external_fresh_path(path: Path, context: str) -> Path:
    resolved_parent = path.parent.resolve(strict=True)
    resolved = resolved_parent / path.name
    repository = repository_root().resolve()
    if resolved == repository or repository in resolved.parents:
        raise YcbCapabilityError(f"{context} must be outside the repository")
    if path.exists() or path.is_symlink():
        raise YcbCapabilityError(f"refusing to replace existing {context}: {path}")
    return resolved


def publish_file(path: Path, data: bytes) -> Path:
    destination = ensure_external_fresh_path(path, "snapshot output")
    temporary = Path(tempfile.mkdtemp(prefix=f".{path.name}.", dir=destination.parent))
    try:
        candidate = temporary / "snapshot.json"
        candidate.write_bytes(data)
        os.replace(candidate, destination)
    finally:
        shutil.rmtree(temporary, ignore_errors=True)
    return destination


def publish_directory(path: Path, documents: dict[str, bytes]) -> Path:
    destination = ensure_external_fresh_path(path, "build output")
    temporary = Path(tempfile.mkdtemp(prefix=f".{path.name}.", dir=destination.parent))
    try:
        for name, data in documents.items():
            (temporary / name).write_bytes(data)
        os.replace(temporary, destination)
    except BaseException:
        shutil.rmtree(temporary, ignore_errors=True)
        raise
    return destination


def run_build(arguments: argparse.Namespace) -> Path:
    required = {
        "workbook": arguments.workbook,
        "model index": arguments.model_index,
        "paper": arguments.paper,
        "OSF snapshot": arguments.osf_snapshot,
    }
    missing = [name for name, value in required.items() if value is None]
    if missing:
        raise YcbCapabilityError(f"build requires: {', '.join(missing)}")
    workbook = read_bound_file(arguments.workbook, "YCB workbook", WORKBOOK_IDENTITY)
    model_index = read_bound_file(
        arguments.model_index, "YCB model index", MODEL_INDEX_IDENTITY
    )
    paper = read_bound_file(arguments.paper, "YCB-impact paper", PAPER_IDENTITY)
    snapshot = read_bound_file(arguments.osf_snapshot, "OSF snapshot", None)
    return publish_directory(
        arguments.output,
        build_documents(workbook, model_index, paper, snapshot),
    )


def main() -> int:
    arguments = parse_arguments()
    try:
        if arguments.stage == "acquire":
            result = publish_file(arguments.output, canonical_json(acquire_osf_snapshot()))
        else:
            result = run_build(arguments)
    except YcbCapabilityError as error:
        raise SystemExit(f"physical-sound YCB capability error: {error}") from error
    print(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
