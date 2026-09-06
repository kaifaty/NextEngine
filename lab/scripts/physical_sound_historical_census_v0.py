#!/usr/bin/env python3
"""Build the M1c historical JSON census and fresh ObjectFolder shortlist."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import tempfile
from html.parser import HTMLParser
from pathlib import Path
from typing import Any, Iterator

import physical_sound_exposure_ledger_v0 as ledger_v0

CENSUS_SCHEMA = "nextengine.experimental-physical-sound-historical-census.v0"
UNIVERSE_SCHEMA = "nextengine.experimental-objectfolder-real-universe.v0"
EXCLUSIONS_SCHEMA = "nextengine.experimental-physical-sound-object-exclusions.v0"
SHORTLIST_SCHEMA = "nextengine.experimental-physical-sound-fresh-shortlist.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-m1c.report.v0"
REVISION = "historical-json-census-objectfolder-glass-v0"
STORE_ROOT_ID = "nextengine-external-physical-sound-store"

OFFICIAL_HTML_RELATIVE_PATH = (
    "ps2-source-feasibility-v1/sources/objectfolder-real-download.html"
)
OFFICIAL_HTML_BYTES = 34_749
OFFICIAL_HTML_SHA256 = (
    "0111f57a336fdeb805eb03543fb10b2cb174f8d0bcb7ef5f0e30c17324e06db0"
)
STALE_MARKDOWN_SHA256 = (
    "2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32"
)
EXCLUDED_COMPONENTS = {
    "dependencies",
    "venv",
    ".venv",
    "site-packages",
    "__pycache__",
}
EXCLUDED_TOP_LEVEL_PREFIXES = ("physical-sound-v13-m1",)
DIRECT_OBJECT_KEYS = {"object_id", "dataset_object_id"}
PATH_TOKEN_PATTERNS = (
    re.compile(r"(?:audio|contacts|force)/([1-9][0-9]?|100)/"),
    re.compile(r"(?:^|/)([1-9][0-9]?|100)/(?:audio|contacts|force)/"),
    re.compile(r"objectfolder-real-object-([1-9][0-9]?|100)_"),
)


class HistoricalCensusError(RuntimeError):
    """The M1c census violates the frozen zero-signal protocol."""


class TableCellParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.rows: list[list[str]] = []
        self._row: list[str] | None = None
        self._cell: list[str] | None = None

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        del attrs
        if tag == "tr":
            self._row = []
        elif tag == "td" and self._row is not None:
            self._cell = []

    def handle_data(self, data: str) -> None:
        if self._cell is not None:
            self._cell.append(data)

    def handle_endtag(self, tag: str) -> None:
        if tag == "td" and self._cell is not None and self._row is not None:
            self._row.append("".join(self._cell).replace("\xa0", " ").strip())
            self._cell = None
        elif tag == "tr" and self._row is not None:
            self.rows.append(self._row)
            self._row = None


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--store", required=True, type=Path)
    parser.add_argument("--universe-html", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return ledger_v0.canonical_json(value)


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def encode_pointer_token(value: str) -> str:
    return value.replace("~", "~0").replace("/", "~1")


def scalar_object_id(value: Any) -> str | None:
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        return None
    text = str(value)
    if not text.isdigit():
        return None
    number = int(text)
    return str(number) if 1 <= number <= 100 else None


def walk_json(value: Any, pointer: str = "") -> Iterator[tuple[str, Any]]:
    yield pointer, value
    if isinstance(value, dict):
        for key in sorted(value):
            child = f"{pointer}/{encode_pointer_token(key)}"
            yield from walk_json(value[key], child)
    elif isinstance(value, list):
        for index, child_value in enumerate(value):
            yield from walk_json(child_value, f"{pointer}/{index}")


def direct_object_pointers(value: Any) -> dict[str, str]:
    matches: dict[str, list[str]] = {}

    def visit(current: Any, pointer: str = "") -> None:
        if isinstance(current, dict):
            for key in sorted(current):
                child = current[key]
                child_pointer = f"{pointer}/{encode_pointer_token(key)}"
                if key in DIRECT_OBJECT_KEYS:
                    object_id = scalar_object_id(child)
                    if object_id is not None:
                        matches.setdefault(object_id, []).append(child_pointer)
                elif key == "object_ids" and isinstance(child, list):
                    for index, item in enumerate(child):
                        object_id = scalar_object_id(item)
                        if object_id is not None:
                            matches.setdefault(object_id, []).append(
                                f"{child_pointer}/{index}"
                            )
                elif key == "object" and isinstance(child, dict) and "id" in child:
                    object_id = scalar_object_id(child["id"])
                    if object_id is not None:
                        matches.setdefault(object_id, []).append(
                            f"{child_pointer}/id"
                        )
                visit(child, child_pointer)
        elif isinstance(current, list):
            for index, child in enumerate(current):
                visit(child, f"{pointer}/{index}")

    visit(value)
    return {object_id: sorted(pointers)[0] for object_id, pointers in matches.items()}


def path_token_hits(value: Any) -> list[dict[str, str]]:
    hits: set[tuple[str, str, str]] = set()
    for pointer, current in walk_json(value):
        if not isinstance(current, str):
            continue
        for pattern in PATH_TOKEN_PATTERNS:
            for match in pattern.finditer(current):
                hits.add((str(int(match.group(1))), pointer, match.group(0)))
    return [
        {"object_id": object_id, "json_pointer": pointer, "token": token}
        for object_id, pointer, token in sorted(hits)
    ]


def excluded_reason(relative: Path) -> str | None:
    if any(part in EXCLUDED_COMPONENTS for part in relative.parts):
        return "tool_or_dependency_component"
    if relative.parts and relative.parts[0].startswith(EXCLUDED_TOP_LEVEL_PREFIXES):
        return "generated_v13_m1_evidence"
    return None


def artifact_kind(path: Path) -> str:
    name = path.name.lower()
    if "manifest" in name:
        return "manifest"
    if "report" in name:
        return "report"
    if "record" in name:
        return "record"
    return "other_json"


def schema_identity(value: Any) -> str:
    if not isinstance(value, dict):
        return "NON_OBJECT"
    schema = value.get("schema")
    return schema if isinstance(schema, str) and schema else "NO_SCHEMA"


def artifact_identifier(relative_path: str) -> str:
    return f"artifact-{sha256_bytes(relative_path.encode())[:24]}"


def universe_from_html(
    data: bytes,
    *,
    expected_bytes: int = OFFICIAL_HTML_BYTES,
    expected_sha256: str = OFFICIAL_HTML_SHA256,
) -> dict[str, Any]:
    if len(data) != expected_bytes or sha256_bytes(data) != expected_sha256:
        if sha256_bytes(data) == STALE_MARKDOWN_SHA256:
            raise HistoricalCensusError("stale shifted Markdown is not universe authority")
        raise HistoricalCensusError("official universe HTML identity changed")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise HistoricalCensusError("official universe HTML is not UTF-8") from error
    parser = TableCellParser()
    parser.feed(text)
    objects: dict[int, dict[str, Any]] = {}
    for row in parser.rows:
        for offset in range(0, len(row) - 2, 3):
            index, name, material = row[offset : offset + 3]
            if not index.isdigit():
                continue
            object_id = int(index)
            if object_id in objects:
                raise HistoricalCensusError(f"duplicate universe object ID: {object_id}")
            if not name or not material:
                raise HistoricalCensusError("universe row has empty name or material")
            objects[object_id] = {
                "object_id": str(object_id),
                "name": name,
                "material": material,
            }
    if sorted(objects) != list(range(1, 101)):
        raise HistoricalCensusError("official universe is not exact IDs 1..100")
    return {
        "schema": UNIVERSE_SCHEMA,
        "source_relative_path": OFFICIAL_HTML_RELATIVE_PATH,
        "source_bytes": len(data),
        "source_sha256": sha256_bytes(data),
        "objects": [objects[index] for index in range(1, 101)],
    }


def scan_store(store: Path) -> dict[str, Any]:
    if store.is_symlink():
        raise HistoricalCensusError("store root cannot be a symlink")
    root = store.resolve(strict=True)
    if not root.is_dir():
        raise HistoricalCensusError("store root is not a directory")
    repo = repository_root().resolve(strict=True)
    if root == repo or root.is_relative_to(repo):
        raise HistoricalCensusError("store root must remain outside the repository")

    included: list[dict[str, Any]] = []
    excluded: list[dict[str, Any]] = []
    direct_hits: list[dict[str, Any]] = []
    token_hits: list[dict[str, Any]] = []
    parsed_by_path: dict[str, Any] = {}
    counters = {
        "json_files_parsed": 0,
        "json_bytes_read": 0,
        "json_scalar_values_parsed": 0,
        "network_requests": 0,
        "source_bytes_read": 0,
        "signal_values_decoded": 0,
        "protected_signal_values_decoded": 0,
    }
    paths = sorted(root.rglob("*.json"), key=lambda item: item.relative_to(root).as_posix())
    if len(paths) > ledger_v0.MAX_ARTIFACTS * 2:
        raise HistoricalCensusError("JSON census exceeds frozen scan limit")
    for path in paths:
        relative = path.relative_to(root)
        relative_name = relative.as_posix()
        reason = excluded_reason(relative)
        if ledger_v0.has_symlink_component(root, relative_name):
            raise HistoricalCensusError(f"JSON path contains symlink: {relative_name}")
        if not path.is_file():
            raise HistoricalCensusError(f"JSON path is not a regular file: {relative_name}")
        size = path.stat().st_size
        if size > ledger_v0.MAX_ARTIFACT_BYTES:
            raise HistoricalCensusError(f"JSON artifact exceeds limit: {relative_name}")
        if reason == "generated_v13_m1_evidence":
            continue
        if reason is not None:
            excluded.append(
                {"relative_path": relative_name, "bytes": size, "reason": reason}
            )
            continue
        data = path.read_bytes()
        if len(data) != size:
            raise HistoricalCensusError(f"JSON artifact changed while read: {relative_name}")
        try:
            value = ledger_v0.parse_json_bytes(data, f"census artifact {relative_name}")
        except ledger_v0.ExposureLedgerError as error:
            raise HistoricalCensusError(str(error)) from error
        direct = direct_object_pointers(value)
        tokens = path_token_hits(value)
        schema = schema_identity(value)
        artifact_id = artifact_identifier(relative_name)
        included.append(
            {
                "artifact_id": artifact_id,
                "relative_path": relative_name,
                "bytes": len(data),
                "sha256": sha256_bytes(data),
                "schema": schema,
                "direct_object_ids": sorted(direct, key=int),
                "path_token_object_ids": sorted(
                    {item["object_id"] for item in tokens}, key=int
                ),
            }
        )
        for object_id, pointer in direct.items():
            direct_hits.append(
                {
                    "object_id": object_id,
                    "artifact_id": artifact_id,
                    "relative_path": relative_name,
                    "json_pointer": pointer,
                }
            )
        for item in tokens:
            token_hits.append(
                {
                    **item,
                    "artifact_id": artifact_id,
                    "relative_path": relative_name,
                }
            )
        parsed_by_path[relative_name] = value
        counters["json_files_parsed"] += 1
        counters["json_bytes_read"] += len(data)
        counters["json_scalar_values_parsed"] += ledger_v0.scalar_count(value)
    included_paths = [item["relative_path"] for item in included]
    excluded_paths = [item["relative_path"] for item in excluded]
    return {
        "root": root,
        "included": included,
        "excluded": excluded,
        "direct_hits": sorted(
            direct_hits,
            key=lambda item: (
                int(item["object_id"]),
                item["relative_path"],
                item["json_pointer"],
            ),
        ),
        "token_hits": sorted(
            token_hits,
            key=lambda item: (
                int(item["object_id"]),
                item["relative_path"],
                item["json_pointer"],
                item["token"],
            ),
        ),
        "parsed_by_path": parsed_by_path,
        "counters": counters,
        "included_path_root_sha256": sha256_bytes(canonical_json(included_paths)),
        "excluded_path_root_sha256": sha256_bytes(canonical_json(excluded_paths)),
    }


def make_catalog(scan: dict[str, Any]) -> dict[str, Any]:
    direct_by_artifact: dict[str, dict[str, dict[str, Any]]] = {}
    for hit in scan["direct_hits"]:
        direct_by_artifact.setdefault(hit["artifact_id"], {})[hit["object_id"]] = hit
    artifacts = []
    exposures = []
    for item in scan["included"]:
        artifact_id = item["artifact_id"]
        parse_json = artifact_id in direct_by_artifact
        schema_hint = item["schema"]
        if (
            not parse_json
            or schema_hint in {"NO_SCHEMA", "NON_OBJECT"}
            or not ledger_v0.IDENTIFIER_PATTERN.fullmatch(schema_hint)
        ):
            schema_hint = None
        artifacts.append(
            {
                "artifact_id": artifact_id,
                "relative_path": item["relative_path"],
                "sha256": item["sha256"],
                "bytes": item["bytes"],
                "kind": artifact_kind(Path(item["relative_path"])),
                "parse_json": parse_json,
                "schema_hint": schema_hint,
            }
        )
        for object_id, hit in sorted(
            direct_by_artifact.get(artifact_id, {}).items(), key=lambda pair: int(pair[0])
        ):
            value = ledger_v0.resolve_json_pointer(
                scan["parsed_by_path"][item["relative_path"]], hit["json_pointer"]
            )
            value_hash = sha256_bytes(canonical_json(value))
            exposures.append(
                {
                    "identity": {
                        "source_namespace": "conservative-objectfolder-real-id",
                        "project_id": object_id,
                        "object_id": object_id,
                        "contact_id": None,
                        "listener_id": None,
                        "impact_id": None,
                        "mutation_parent_id": None,
                        "payload_kind": "metadata",
                    },
                    "role": "historical_unknown",
                    "access_kind": "derived_signal_decoded",
                    "values_decoded": 0,
                    "protected": True,
                    "evidence": [
                        {
                            "binds": binding,
                            "artifact_id": artifact_id,
                            "json_pointer": hit["json_pointer"],
                            "value_sha256": value_hash,
                        }
                        for binding in ("object_id", "project_id")
                    ],
                }
            )
    catalog = {
        "schema": ledger_v0.CATALOG_SCHEMA,
        "revision": REVISION,
        "store_root_id": STORE_ROOT_ID,
        "partition_policy": "historical_union",
        "artifacts": sorted(artifacts, key=lambda item: item["artifact_id"]),
        "exposures": sorted(exposures, key=ledger_v0.exposure_sort_key),
        "network_allowed": False,
        "source_signal_access_allowed": False,
        "outputs_external": True,
    }
    return ledger_v0.validate_catalog(catalog)


def make_shortlist(
    universe: dict[str, Any], direct_hits: list[dict[str, Any]], token_hits: list[dict[str, Any]]
) -> dict[str, Any]:
    direct_counts: dict[str, int] = {}
    token_counts: dict[str, int] = {}
    for item in direct_hits:
        direct_counts[item["object_id"]] = direct_counts.get(item["object_id"], 0) + 1
    for item in token_hits:
        token_counts[item["object_id"]] = token_counts.get(item["object_id"], 0) + 1
    rows = []
    for item in universe["objects"]:
        if item["material"] != "Glass":
            continue
        object_id = item["object_id"]
        direct_count = direct_counts.get(object_id, 0)
        token_count = token_counts.get(object_id, 0)
        decision = (
            "FreshMetadataOnly"
            if direct_count == 0 and token_count == 0
            else "ExcludedPriorExposure"
        )
        rows.append(
            {
                **item,
                "direct_exposure_count": direct_count,
                "path_token_exposure_count": token_count,
                "decision": decision,
            }
        )
    fresh = [item for item in rows if item["decision"] == "FreshMetadataOnly"]
    return {
        "schema": SHORTLIST_SCHEMA,
        "revision": REVISION,
        "material": "Glass",
        "candidates": rows,
        "fresh_candidates": fresh,
        "decision": (
            "FreshGlassCandidatesAvailable"
            if fresh
            else "NO_FRESH_GLASS_CANDIDATE"
        ),
        "authorizes_only": "M2_zero_signal_source_role_freeze",
    }


def prepare_output(output: Path) -> Path:
    resolved = output.resolve()
    repo = repository_root().resolve(strict=True)
    if resolved == repo or resolved.is_relative_to(repo):
        raise HistoricalCensusError("output must remain outside the repository")
    if resolved.exists():
        raise HistoricalCensusError(f"refusing to replace existing output: {resolved}")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    return resolved


def execute(
    store: Path,
    universe_html: Path,
    output: Path,
    *,
    expected_universe_bytes: int = OFFICIAL_HTML_BYTES,
    expected_universe_sha256: str = OFFICIAL_HTML_SHA256,
) -> Path:
    destination = prepare_output(output)
    scan = scan_store(store)
    universe_data = universe_html.read_bytes()
    universe = universe_from_html(
        universe_data,
        expected_bytes=expected_universe_bytes,
        expected_sha256=expected_universe_sha256,
    )
    catalog = make_catalog(scan)
    exclusions = {
        "schema": EXCLUSIONS_SCHEMA,
        "revision": REVISION,
        "direct_exposures": scan["direct_hits"],
        "path_token_exposures": scan["token_hits"],
    }
    shortlist = make_shortlist(universe, scan["direct_hits"], scan["token_hits"])
    census = {
        "schema": CENSUS_SCHEMA,
        "revision": REVISION,
        "store_root_id": STORE_ROOT_ID,
        "included": scan["included"],
        "excluded": {
            "entries": scan["excluded"],
            "component_rules": sorted(EXCLUDED_COMPONENTS),
            "top_level_prefix_rules": list(EXCLUDED_TOP_LEVEL_PREFIXES),
        },
        "roots": {
            "included_path_root_sha256": scan["included_path_root_sha256"],
            "excluded_path_root_sha256": scan["excluded_path_root_sha256"],
        },
        "counters": scan["counters"],
    }
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}.", dir=destination.parent)
    )
    try:
        top_files = {
            "census.json": canonical_json(census),
            "catalog.json": canonical_json(catalog),
            "universe.json": canonical_json(universe),
            "exclusions.json": canonical_json(exclusions),
            "shortlist.json": canonical_json(shortlist),
        }
        for name, data in top_files.items():
            (staging / name).write_bytes(data)
        ledger_output = ledger_v0.build_ledger(
            staging / "catalog.json",
            scan["root"],
            staging / "ledger",
            decision="M1C_HISTORICAL_UNION_BUILD_PASS",
            opens_only="M2_zero_signal_source_role_freeze_if_shortlist_passes",
        )
        ledger_report = json.loads((ledger_output / "report.json").read_bytes())
        build_access = ledger_report["build_access"]
        for key in (
            "network_requests",
            "source_bytes_read",
            "signal_values_decoded",
            "protected_signal_values_decoded",
        ):
            if build_access[key] != 0:
                raise HistoricalCensusError(f"M1b build access is nonzero: {key}")
        decision = (
            "M1C_HISTORICAL_CENSUS_FRESH_SHORTLIST_PASS"
            if shortlist["fresh_candidates"]
            else "NO_FRESH_GLASS_CANDIDATE"
        )
        report = {
            "schema": REPORT_SCHEMA,
            "revision": REVISION,
            "decision": decision,
            "hashes": {
                name: sha256_bytes(data) for name, data in sorted(top_files.items())
            },
            "ledger": {
                "catalog_sha256": ledger_report["catalog_sha256"],
                "ledger_sha256": ledger_report["ledger_sha256"],
                "summary_sha256": ledger_report["summary_sha256"],
                "role_root_sha256": ledger_report["role_root_sha256"],
            },
            "counts": {
                "included_json_files": len(scan["included"]),
                "excluded_json_files": len(scan["excluded"]),
                "direct_exposure_records": len(scan["direct_hits"]),
                "path_token_exposure_records": len(scan["token_hits"]),
                "official_glass_objects": len(shortlist["candidates"]),
                "fresh_glass_candidates": len(shortlist["fresh_candidates"]),
            },
            "census_access": scan["counters"],
            "universe_access": {
                "html_bytes_read": len(universe_data),
                "network_requests": 0,
                "source_bytes_read": 0,
                "signal_values_decoded": 0,
            },
            "ledger_build_access": build_access,
            "public_contract": False,
            "runtime_consumer_allowed": False,
            "authorizes_only": "M2_zero_signal_source_role_freeze",
        }
        (staging / "report.json").write_bytes(canonical_json(report))
        os.replace(staging, destination)
    finally:
        if staging.exists():
            shutil.rmtree(staging)
    return destination


def main() -> None:
    arguments = parse_arguments()
    destination = execute(arguments.store, arguments.universe_html, arguments.output)
    print(destination)


if __name__ == "__main__":
    main()
