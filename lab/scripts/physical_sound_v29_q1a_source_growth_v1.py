#!/usr/bin/env python3
"""Capture and audit V29 Q1a-M publisher metadata without opening audio."""

from __future__ import annotations

import argparse
import hashlib
import html
import itertools
import json
import os
import re
import shutil
import subprocess
import tempfile
from collections import Counter, defaultdict
from html.parser import HTMLParser
from pathlib import Path
from typing import AbstractSet, Any


PROFILE_SCHEMA = "nextengine.experimental-physical-sound-v29-q1a-source-growth-profile.v1"
CAPTURE_SCHEMA = "nextengine.experimental-physical-sound-v29-q1a-source-capture.v1"
AUDIT_SCHEMA = "nextengine.experimental-physical-sound-v29-q1a-source-growth-audit.v1"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-v29-q1a.report.v1"
Q0_SCHEMA = "nextengine.experimental-physical-sound-v29-q0m-inventory.v1"
Q1_SCHEMA = "nextengine.experimental-physical-sound-v29-q1m-role-power-audit.v1"

Q0_IDENTITY = (
    172_172,
    "9a04c8a2fb9f0c6a3d351dac2796936d933d223e9400afc79a95f06e01665f43",
)
Q1_IDENTITY = (
    72_478,
    "f98e80c7ad21fd698718d2e31c858b3b4d53f386c69356b5252397a1d3e20944",
)
MAX_PROFILE_BYTES = 256 * 1024
MAX_INPUT_BYTES = 4 * 1024 * 1024
MAX_HTML_BYTES = 1024 * 1024
USER_AGENT = "NextEngine-physical-sound-q1a-metadata/1.0"
CURL_TIMEOUT_SECONDS = 45

ZERO_SIGNAL_ACCESS = {
    "audio_headers_parsed": 0,
    "audio_preview_bytes_read": 0,
    "force_sample_values_decoded": 0,
    "mesh_values_decoded": 0,
    "pcm_sample_values_decoded": 0,
    "protected_signal_values_decoded": 0,
    "role_signal_values_opened": 0,
    "source_payload_bytes_read": 0,
    "waveform_or_feature_values_decoded": 0,
}

PROFILE_KEYS = {"baseline_commit", "minimums", "projects", "schema"}
MINIMUM_KEYS = {
    "added_exact_steel_groups",
    "added_non_metal_groups",
    "added_role_capable_projects",
    "protected_exact_steel_groups_per_role",
    "protected_non_metal_groups_per_role",
    "protected_projects_per_role",
    "reserved_unprotected_projects",
}
PROJECT_KEYS = {"author", "groups", "pack_id", "pack_title"}
PROJECT_KEYS_WITH_CONTROLS = PROJECT_KEYS | {"conflict_controls"}
GROUP_KEYS = {
    "action_tokens",
    "evidence_phrase",
    "group_id",
    "object_name",
    "primary_material",
    "relation",
    "sound_ids",
}
CONTROL_KEYS = {"description_conflict_token", "sound_id", "title_token"}
CAPTURE_KEYS = {
    "access",
    "baseline_commit",
    "baseline_exposure",
    "pages",
    "profile_identity",
    "schema",
}
PAGE_KEYS = {"bytes", "content_type", "file", "final_url", "sha256", "url"}
CAPTURE_ACCESS_KEYS = set(ZERO_SIGNAL_ACCESS) | {
    "network_requests",
    "publisher_metadata_bytes_read",
}

ALLOWED_LICENSES = {
    "http://creativecommons.org/publicdomain/zero/1.0/": "CC0-1.0",
    "https://creativecommons.org/publicdomain/zero/1.0/": "CC0-1.0",
    "http://creativecommons.org/licenses/by/3.0/": "CC-BY-3.0",
    "https://creativecommons.org/licenses/by/3.0/": "CC-BY-3.0",
    "http://creativecommons.org/licenses/by/4.0/": "CC-BY-4.0",
    "https://creativecommons.org/licenses/by/4.0/": "CC-BY-4.0",
}
DISALLOWED_STEEL_QUALIFIERS = {
    "carbon",
    "galvanized",
    "galvanised",
    "stainless",
    "zinc-plated",
    "zincplated",
}


class Q1ASourceGrowthError(RuntimeError):
    """A Q1a source or output violates the frozen protocol."""


class FreesoundPageParser(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.title_parts: list[str] = []
        self.in_title = False
        self.meta: dict[str, str] = {}
        self.players: list[dict[str, str]] = []
        self.links: list[str] = []

    def handle_starttag(
        self, tag: str, attrs: list[tuple[str, str | None]]
    ) -> None:
        values = {key: value or "" for key, value in attrs}
        if tag == "title":
            self.in_title = True
        if tag == "meta":
            key = values.get("name") or values.get("property")
            content = values.get("content")
            if key and content:
                self.meta[key] = content
        if tag == "a" and values.get("href"):
            self.links.append(values["href"])
        if "bw-player" in values.get("class", "").split():
            self.players.append(values)

    def handle_endtag(self, tag: str) -> None:
        if tag == "title":
            self.in_title = False

    def handle_data(self, data: str) -> None:
        if self.in_title:
            self.title_parts.append(data)

    @property
    def page_title(self) -> str:
        return normalize_text("".join(self.title_parts))


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    commands = parser.add_subparsers(dest="command", required=True)

    capture = commands.add_parser("capture")
    capture.add_argument("--profile", required=True, type=Path)
    capture.add_argument("--output", required=True, type=Path)

    audit = commands.add_parser("audit")
    audit.add_argument("--profile", required=True, type=Path)
    audit.add_argument("--capture", required=True, type=Path)
    audit.add_argument("--q0-inventory", required=True, type=Path)
    audit.add_argument("--q1-audit", required=True, type=Path)
    audit.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    try:
        return (
            json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n"
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise Q1ASourceGrowthError(f"cannot serialize canonical JSON: {error}") from error


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise Q1ASourceGrowthError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def parse_json(data: bytes, context: str) -> dict[str, Any]:
    try:
        value = json.loads(data, object_pairs_hook=unique_object)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Q1ASourceGrowthError(f"{context} is not strict JSON: {error}") from error
    if not isinstance(value, dict):
        raise Q1ASourceGrowthError(f"{context} root must be an object")
    return value


def require_keys(value: Any, keys: set[str], context: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != keys:
        raise Q1ASourceGrowthError(f"{context} has unknown or missing fields")
    return value


def require_string(value: Any, context: str, maximum: int = 512) -> str:
    if not isinstance(value, str) or not 1 <= len(value) <= maximum:
        raise Q1ASourceGrowthError(f"{context} must be a bounded string")
    return value


def require_positive_int(value: Any, context: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise Q1ASourceGrowthError(f"{context} must be a positive integer")
    return value


def normalize_text(value: str) -> str:
    return " ".join(html.unescape(value).split())


def bounded_regular_file(path: Path, maximum: int, context: str) -> bytes:
    if path.is_symlink() or not path.is_file():
        raise Q1ASourceGrowthError(f"{context} must be a regular non-symlink file")
    size = path.stat().st_size
    if size <= 0 or size > maximum:
        raise Q1ASourceGrowthError(f"{context} size is out of bounds")
    return path.read_bytes()


def validate_external_path(path: Path, context: str) -> Path:
    resolved = path.resolve(strict=False)
    if resolved == repository_root() or resolved.is_relative_to(repository_root()):
        raise Q1ASourceGrowthError(f"{context} must remain outside the repository")
    return resolved


def canonical_pack_url(project: dict[str, Any]) -> str:
    return (
        f"https://freesound.org/people/{project['author']}/packs/"
        f"{project['pack_id']}/"
    )


def canonical_sound_url(author: str, sound_id: int) -> str:
    return f"https://freesound.org/people/{author}/sounds/{sound_id}/"


def validate_profile(value: dict[str, Any]) -> dict[str, Any]:
    require_keys(value, PROFILE_KEYS, "profile")
    if value["schema"] != PROFILE_SCHEMA:
        raise Q1ASourceGrowthError("profile schema mismatch")
    baseline = require_string(value["baseline_commit"], "baseline commit", 64)
    if not re.fullmatch(r"[0-9a-f]{8,40}", baseline):
        raise Q1ASourceGrowthError("baseline commit is not a Git object identity")
    minimums = require_keys(value["minimums"], MINIMUM_KEYS, "minimums")
    for key, item in minimums.items():
        require_positive_int(item, f"minimum {key}")
    projects = value["projects"]
    if not isinstance(projects, list) or not projects or len(projects) > 64:
        raise Q1ASourceGrowthError("projects must be a bounded non-empty list")

    project_ids: set[tuple[str, int]] = set()
    group_ids: set[str] = set()
    sound_ids: set[int] = set()
    for project_index, project_value in enumerate(projects):
        if not isinstance(project_value, dict) or set(project_value) not in (
            PROJECT_KEYS,
            PROJECT_KEYS_WITH_CONTROLS,
        ):
            raise Q1ASourceGrowthError(f"project {project_index} fields mismatch")
        author = require_string(project_value["author"], "author", 64)
        if not re.fullmatch(r"[A-Za-z0-9_-]+", author):
            raise Q1ASourceGrowthError("author is not a canonical Freesound identity")
        pack_id = require_positive_int(project_value["pack_id"], "pack id")
        require_string(project_value["pack_title"], "pack title", 256)
        project_identity = (author, pack_id)
        if project_identity in project_ids:
            raise Q1ASourceGrowthError("duplicate publisher/pack project")
        project_ids.add(project_identity)
        groups = project_value["groups"]
        if not isinstance(groups, list) or not groups or len(groups) > 128:
            raise Q1ASourceGrowthError("project groups must be bounded and non-empty")
        for group in groups:
            require_keys(group, GROUP_KEYS, "group")
            group_id = require_string(group["group_id"], "group id", 160)
            if not re.fullmatch(r"[a-z0-9-]+", group_id) or group_id in group_ids:
                raise Q1ASourceGrowthError("group id is invalid or duplicated")
            group_ids.add(group_id)
            for key in ("evidence_phrase", "object_name", "primary_material"):
                require_string(group[key], key, 256)
            relation = group["relation"]
            if relation not in {"exact_steel_candidate", "non_metal_candidate"}:
                raise Q1ASourceGrowthError("unsupported candidate relation")
            if relation == "exact_steel_candidate" and group["primary_material"] != "Steel":
                raise Q1ASourceGrowthError("exact Steel group must preserve label Steel")
            if relation == "non_metal_candidate" and group["primary_material"] in {
                "Steel",
                "Iron",
                "Aluminium",
                "Metal",
                "Stainless Steel",
            }:
                raise Q1ASourceGrowthError("non-Metal group has a Metal label")
            actions = group["action_tokens"]
            if not isinstance(actions, list) or not actions or len(actions) > 8:
                raise Q1ASourceGrowthError("action tokens must be bounded and non-empty")
            for token in actions:
                require_string(token, "action token", 64)
            ids = group["sound_ids"]
            if not isinstance(ids, list) or not ids or len(ids) > 64:
                raise Q1ASourceGrowthError("sound ids must be bounded and non-empty")
            for sound_id in ids:
                require_positive_int(sound_id, "sound id")
                if sound_id in sound_ids:
                    raise Q1ASourceGrowthError("sound id is assigned more than once")
                sound_ids.add(sound_id)
        for control in project_value.get("conflict_controls", []):
            require_keys(control, CONTROL_KEYS, "conflict control")
            sound_id = require_positive_int(control["sound_id"], "control sound id")
            if sound_id in sound_ids:
                raise Q1ASourceGrowthError("control sound id overlaps a group")
            sound_ids.add(sound_id)
            require_string(control["title_token"], "control title token", 64)
            require_string(
                control["description_conflict_token"], "conflict token", 128
            )
    return value


def read_profile(path: Path) -> tuple[dict[str, Any], bytes]:
    data = bounded_regular_file(path, MAX_PROFILE_BYTES, "profile")
    return validate_profile(parse_json(data, "profile")), data


def all_source_urls(profile: dict[str, Any]) -> list[str]:
    urls: set[str] = set()
    for project in profile["projects"]:
        urls.add(canonical_pack_url(project))
        for group in project["groups"]:
            for sound_id in group["sound_ids"]:
                urls.add(canonical_sound_url(project["author"], sound_id))
        for control in project.get("conflict_controls", []):
            urls.add(canonical_sound_url(project["author"], control["sound_id"]))
    return sorted(urls)


def git_output(
    args: list[str], context: str, allowed: AbstractSet[int] = frozenset({0})
) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repository_root(),
        check=False,
        capture_output=True,
        text=True,
        timeout=30,
    )
    if result.returncode not in allowed:
        raise Q1ASourceGrowthError(
            f"{context} failed ({result.returncode}): {result.stderr.strip()}"
        )
    return result.stdout


def baseline_exposure(profile: dict[str, Any]) -> tuple[str, list[dict[str, Any]]]:
    commit = git_output(
        ["rev-parse", f"{profile['baseline_commit']}^{{commit}}"],
        "resolve baseline commit",
    ).strip()
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise Q1ASourceGrowthError("baseline commit did not resolve exactly")
    rows = []
    for project in profile["projects"]:
        needle = canonical_pack_url(project)
        matches = git_output(
            ["grep", "-n", "-F", "-e", needle, commit, "--"],
            "baseline exposure search",
            {0, 1},
        ).splitlines()
        rows.append(
            {
                "matches": sorted(matches),
                "pack_id": project["pack_id"],
                "publisher": project["author"],
                "url": needle,
            }
        )
    return commit, rows


def fetch_html_batch(urls: list[str]) -> dict[str, tuple[bytes, str, str]]:
    if not urls or len(urls) != len(set(urls)):
        raise Q1ASourceGrowthError("fetch URL list must be non-empty and unique")
    with tempfile.TemporaryDirectory(prefix="nextengine-q1a-fetch-") as temporary:
        body_paths = [Path(temporary) / f"{index:03d}.html" for index in range(len(urls))]
        command = [
            "curl",
            "--disable",
            "--silent",
            "--show-error",
            "--fail-early",
            "--proto",
            "=https",
            "--connect-timeout",
            "15",
            "--max-time",
            str(CURL_TIMEOUT_SECONDS),
            "--max-filesize",
            str(MAX_HTML_BYTES),
            "--user-agent",
            USER_AGENT,
            "--write-out",
            "%{urlnum}\t%{http_code}\t%{content_type}\t%{url_effective}\n",
        ]
        for url, body_path in zip(urls, body_paths, strict=True):
            command.extend(("--output", str(body_path), url))
        try:
            result = subprocess.run(
                command,
                check=False,
                capture_output=True,
                text=True,
                timeout=CURL_TIMEOUT_SECONDS * len(urls) + 5,
            )
        except (OSError, subprocess.TimeoutExpired) as error:
            raise Q1ASourceGrowthError(f"fetch transport failed: {error}") from error
        metadata = [line.split("\t", 3) for line in result.stdout.splitlines()]
        if result.returncode != 0 or len(metadata) != len(urls):
            detail = result.stderr.strip() or "invalid curl metadata"
            raise Q1ASourceGrowthError(
                f"fetch batch failed ({result.returncode}): {detail}"
            )
        fetched: dict[str, tuple[bytes, str, str]] = {}
        for expected_index, (url, body_path, fields) in enumerate(
            zip(urls, body_paths, metadata, strict=True)
        ):
            if len(fields) != 4:
                raise Q1ASourceGrowthError("fetch metadata fields are malformed")
            url_index, status, raw_content_type, final_url = fields
            if url_index != str(expected_index):
                raise Q1ASourceGrowthError("fetch transfer order drifted")
            if status != "200":
                raise Q1ASourceGrowthError(f"source HTTP status is {status}: {url}")
            content_type = raw_content_type.partition(";")[0].strip().lower()
            data = bounded_regular_file(body_path, MAX_HTML_BYTES, "source HTML")
            if final_url != url:
                raise Q1ASourceGrowthError(
                    f"source redirected away from canonical URL: {url}"
                )
            if content_type != "text/html":
                raise Q1ASourceGrowthError(f"source is not HTML: {url}")
            fetched[url] = (data, final_url, content_type)
        return fetched


def fetch_html(url: str) -> tuple[bytes, str, str]:
    return fetch_html_batch([url])[url]


def page_file_name(index: int, url: str) -> str:
    identity = sha256_bytes(url.encode("utf-8"))[:16]
    return f"{index:03d}-{identity}.html"


def write_fresh_directory(output: Path, files: dict[str, bytes]) -> None:
    resolved = validate_external_path(output, "output")
    if output.exists() or output.is_symlink():
        raise Q1ASourceGrowthError("output already exists")
    resolved.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{resolved.name}.", dir=resolved.parent)
    )
    try:
        for relative, data in files.items():
            target = staging / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
        os.rename(staging, resolved)
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def run_capture(profile_path: Path, output: Path) -> None:
    profile, profile_data = read_profile(profile_path)
    baseline_commit, exposure = baseline_exposure(profile)
    pages = []
    files: dict[str, bytes] = {}
    metadata_bytes = 0
    urls = all_source_urls(profile)
    fetched = fetch_html_batch(urls)
    for index, url in enumerate(urls):
        data, final_url, content_type = fetched[url]
        file_name = f"raw/{page_file_name(index, url)}"
        files[file_name] = data
        metadata_bytes += len(data)
        pages.append(
            {
                "bytes": len(data),
                "content_type": content_type,
                "file": file_name,
                "final_url": final_url,
                "sha256": sha256_bytes(data),
                "url": url,
            }
        )
    access = {
        **ZERO_SIGNAL_ACCESS,
        "network_requests": len(pages),
        "publisher_metadata_bytes_read": metadata_bytes,
    }
    capture = {
        "access": access,
        "baseline_commit": baseline_commit,
        "baseline_exposure": exposure,
        "pages": pages,
        "profile_identity": {
            "bytes": len(profile_data),
            "sha256": sha256_bytes(profile_data),
        },
        "schema": CAPTURE_SCHEMA,
    }
    files["capture.json"] = canonical_json(capture)
    write_fresh_directory(output, files)


def parse_page(data: bytes, context: str) -> FreesoundPageParser:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise Q1ASourceGrowthError(f"{context} is not UTF-8 HTML") from error
    parser = FreesoundPageParser()
    try:
        parser.feed(text)
        parser.close()
    except Exception as error:
        raise Q1ASourceGrowthError(f"parse {context}: {error}") from error
    return parser


def selected_player(
    parser: FreesoundPageParser, author: str, sound_id: int, context: str
) -> dict[str, str]:
    matches = [
        player
        for player in parser.players
        if player.get("data-sound-id") == str(sound_id)
    ]
    if len(matches) != 1:
        raise Q1ASourceGrowthError(f"{context} lacks exact sound/uploader identity")
    player = matches[0]
    player_username = player.get("data-username", "")
    if player_username:
        if player_username != author:
            raise Q1ASourceGrowthError(
                f"{context} lacks exact sound/uploader identity"
            )
    else:
        expected_sound_url = (
            f"https://freesound.org/people/{author}/sounds/{sound_id}/"
        )
        expected_author_link = f"/people/{author}/"
        if (
            parser.meta.get("og:url") != expected_sound_url
            or parser.meta.get("og:audio:artist") != author
            or expected_author_link not in parser.links
            or not parser.page_title.endswith(f" by {author}")
        ):
            raise Q1ASourceGrowthError(
                f"{context} lacks exact sound/uploader identity"
            )
    if not player.get("data-user-id", "").isdigit():
        raise Q1ASourceGrowthError(f"{context} lacks canonical uploader id")
    return player


def license_identity(parser: FreesoundPageParser, context: str) -> tuple[str, str]:
    links = sorted({link for link in parser.links if link in ALLOWED_LICENSES})
    other_cc = sorted(
        {
            link
            for link in parser.links
            if "creativecommons.org/" in link and link not in ALLOWED_LICENSES
        }
    )
    if len(links) != 1 or other_cc:
        raise Q1ASourceGrowthError(f"{context} has absent, ambiguous or unsupported license")
    return links[0], ALLOWED_LICENSES[links[0]]


def unqualified_steel_in_phrase(text: str, phrase: str) -> bool:
    lowered = normalize_text(text).lower()
    evidence = normalize_text(phrase).lower()
    if evidence not in lowered or not re.search(r"\bsteel\b", evidence):
        return False
    for evidence_match in re.finditer(re.escape(evidence), lowered):
        evidence_start, evidence_end = evidence_match.span()
        for steel_match in re.finditer(
            r"\bsteel\b", lowered[evidence_start:evidence_end]
        ):
            steel_start = evidence_start + steel_match.start()
            prefix = lowered[:steel_start].rstrip(" -_/.")
            previous = re.search(r"([a-z]+(?:-[a-z]+)?)$", prefix)
            if (
                previous is None
                or previous.group(1) not in DISALLOWED_STEEL_QUALIFIERS
            ):
                return True
    return False


def normalized_sound(
    raw: bytes, author: str, pack_id: int, sound_id: int
) -> dict[str, Any]:
    context = f"Freesound sound {sound_id}"
    parser = parse_page(raw, context)
    player = selected_player(parser, author, sound_id, context)
    expected_pack_link = f"/people/{author}/packs/{pack_id}/"
    if expected_pack_link not in parser.links:
        raise Q1ASourceGrowthError(f"{context} lacks exact pack membership")
    title = normalize_text(parser.meta.get("twitter:title", player.get("data-title", "")))
    description = normalize_text(parser.meta.get("twitter:description", ""))
    if not title or len(title) > 512 or len(description) > 4096:
        raise Q1ASourceGrowthError(f"{context} has invalid publisher text")
    license_url, license_expression = license_identity(parser, context)
    return {
        "author_id": player["data-user-id"],
        "description": description,
        "license_expression": license_expression,
        "license_url": license_url,
        "pack_id": pack_id,
        "sound_id": sound_id,
        "title": title,
        "uploader": author,
    }


def validate_group_evidence(group: dict[str, Any], sounds: list[dict[str, Any]]) -> None:
    combined = " ".join(
        f"{sound['title']} {sound['description']}" for sound in sounds
    )
    lowered = normalize_text(combined).lower()
    phrase = group["evidence_phrase"]
    if normalize_text(phrase).lower() not in lowered:
        raise Q1ASourceGrowthError(
            f"group {group['group_id']} lacks its publisher evidence phrase"
        )
    if not any(
        re.search(rf"\b{re.escape(token.lower())}\b", lowered)
        for token in group["action_tokens"]
    ):
        raise Q1ASourceGrowthError(
            f"group {group['group_id']} lacks its impact action"
        )
    if group["relation"] == "exact_steel_candidate" and not unqualified_steel_in_phrase(
        combined, phrase
    ):
        raise Q1ASourceGrowthError(
            f"group {group['group_id']} does not preserve exact unqualified Steel"
        )


def read_capture(
    capture_root: Path, profile_data: bytes
) -> tuple[dict[str, Any], dict[str, bytes]]:
    if capture_root.is_symlink():
        raise Q1ASourceGrowthError("capture root must not be a symlink")
    root = validate_external_path(capture_root, "capture root")
    if not root.is_dir():
        raise Q1ASourceGrowthError("capture root must be a directory")
    capture_data = bounded_regular_file(root / "capture.json", MAX_INPUT_BYTES, "capture")
    capture = parse_json(capture_data, "capture")
    if capture_data != canonical_json(capture):
        raise Q1ASourceGrowthError("capture manifest is not canonical JSON")
    require_keys(capture, CAPTURE_KEYS, "capture")
    if capture["schema"] != CAPTURE_SCHEMA:
        raise Q1ASourceGrowthError("capture schema mismatch")
    identity = require_keys(
        capture["profile_identity"], {"bytes", "sha256"}, "profile identity"
    )
    if identity != {"bytes": len(profile_data), "sha256": sha256_bytes(profile_data)}:
        raise Q1ASourceGrowthError("capture/profile identity mismatch")
    pages = capture["pages"]
    if not isinstance(pages, list) or not pages:
        raise Q1ASourceGrowthError("capture pages are missing")
    access = require_keys(capture["access"], CAPTURE_ACCESS_KEYS, "capture access")
    for key, value in access.items():
        if not isinstance(value, int) or isinstance(value, bool) or value < 0:
            raise Q1ASourceGrowthError(f"capture access {key} must be non-negative")
    for key in ZERO_SIGNAL_ACCESS:
        if access[key] != 0:
            raise Q1ASourceGrowthError(f"capture opened forbidden signal: {key}")
    raw_by_url: dict[str, bytes] = {}
    files_seen: set[Path] = set()
    declared_metadata_bytes = 0
    for row in pages:
        require_keys(row, PAGE_KEYS, "capture page")
        url = require_string(row["url"], "capture URL", 1024)
        final_url = require_string(row["final_url"], "capture final URL", 1024)
        content_type = require_string(row["content_type"], "capture content type", 128)
        if url in raw_by_url or final_url != url or content_type != "text/html":
            raise Q1ASourceGrowthError("capture URL identity is duplicated or non-canonical")
        relative = Path(require_string(row["file"], "capture file", 256))
        if relative.is_absolute() or ".." in relative.parts or relative in files_seen:
            raise Q1ASourceGrowthError("capture file escapes its root")
        files_seen.add(relative)
        candidate = root / relative
        try:
            resolved_candidate = candidate.resolve(strict=True)
        except OSError as error:
            raise Q1ASourceGrowthError("captured HTML cannot be resolved") from error
        if not resolved_candidate.is_relative_to(root):
            raise Q1ASourceGrowthError("capture file resolves outside its root")
        data = bounded_regular_file(candidate, MAX_HTML_BYTES, "captured HTML")
        declared_bytes = require_positive_int(row["bytes"], "capture page bytes")
        declared_sha = require_string(row["sha256"], "capture page sha256", 64)
        if not re.fullmatch(r"[0-9a-f]{64}", declared_sha):
            raise Q1ASourceGrowthError("capture page sha256 is invalid")
        if declared_bytes != len(data) or declared_sha != sha256_bytes(data):
            raise Q1ASourceGrowthError("captured HTML identity drift")
        declared_metadata_bytes += len(data)
        raw_by_url[url] = data
    if access["network_requests"] != len(pages):
        raise Q1ASourceGrowthError("capture network request count mismatch")
    if access["publisher_metadata_bytes_read"] != declared_metadata_bytes:
        raise Q1ASourceGrowthError("capture metadata byte count mismatch")
    return capture, raw_by_url


def read_known_input(path: Path, identity: tuple[int, str], context: str) -> bytes:
    data = bounded_regular_file(path, MAX_INPUT_BYTES, context)
    if (len(data), sha256_bytes(data)) != identity:
        raise Q1ASourceGrowthError(f"{context} identity drift")
    return data


def project_power_from_q0(q0: dict[str, Any]) -> list[dict[str, Any]]:
    if q0.get("schema") != Q0_SCHEMA or not isinstance(q0.get("groups"), list):
        raise Q1ASourceGrowthError("Q0 inventory schema mismatch")
    counts: dict[str, Counter[str]] = defaultdict(Counter)
    for group in q0["groups"]:
        if not isinstance(group, dict) or not group.get(
            "candidate_available_after_historical_exclusion"
        ):
            continue
        key = "--".join(
            (
                require_string(group.get("publisher_id"), "Q0 publisher"),
                require_string(group.get("project_id"), "Q0 project"),
                require_string(group.get("revision"), "Q0 revision"),
            )
        )
        relation = group.get("material_relation")
        if relation in {"exact_steel_candidate", "non_metal_candidate"}:
            counts[key][relation] += 1
    return [
        {
            "exact_steel_groups": value["exact_steel_candidate"],
            "non_metal_groups": value["non_metal_candidate"],
            "origin": "q0-m",
            "project_revision_id": key,
        }
        for key, value in sorted(counts.items())
    ]


def solve_protected_partition(
    projects: list[dict[str, Any]], minimums: dict[str, int]
) -> dict[str, Any]:
    project_min = minimums["protected_projects_per_role"]
    steel_min = minimums["protected_exact_steel_groups_per_role"]
    reject_min = minimums["protected_non_metal_groups_per_role"]
    reserve_min = minimums["reserved_unprotected_projects"]
    if len(projects) > 18:
        raise Q1ASourceGrowthError("protected partition search exceeds bounded project count")

    best: tuple[Any, ...] | None = None
    best_document: dict[str, Any] | None = None
    feasible: list[tuple[Any, ...]] = []
    for assignment in itertools.product((0, 1, 2), repeat=len(projects)):
        role_a = [projects[i] for i, role in enumerate(assignment) if role == 1]
        role_b = [projects[i] for i, role in enumerate(assignment) if role == 2]
        reserved = len(projects) - len(role_a) - len(role_b)
        if len(role_a) < project_min or len(role_b) < project_min or reserved < reserve_min:
            continue
        ids_a = tuple(row["project_revision_id"] for row in role_a)
        ids_b = tuple(row["project_revision_id"] for row in role_b)
        if ids_a > ids_b:
            continue

        def summary(rows: list[dict[str, Any]]) -> dict[str, Any]:
            steel = sum(row["exact_steel_groups"] for row in rows)
            rejects = sum(row["non_metal_groups"] for row in rows)
            return {
                "exact_steel_deficit": max(0, steel_min - steel),
                "exact_steel_groups": steel,
                "non_metal_deficit": max(0, reject_min - rejects),
                "non_metal_groups": rejects,
                "project_count": len(rows),
                "project_deficit": max(0, project_min - len(rows)),
                "project_revision_ids": [row["project_revision_id"] for row in rows],
            }

        a = summary(role_a)
        b = summary(role_b)
        deficits = (
            a["exact_steel_deficit"]
            + a["non_metal_deficit"]
            + b["exact_steel_deficit"]
            + b["non_metal_deficit"]
        )
        maximum_deficit = max(
            a["exact_steel_deficit"],
            a["non_metal_deficit"],
            b["exact_steel_deficit"],
            b["non_metal_deficit"],
        )
        document = {
            "reserved_unprotected_project_count": reserved,
            "role_a": a,
            "role_b": b,
        }
        score = (deficits, maximum_deficit, -reserved, ids_a, ids_b)
        if best is None or score < best:
            best = score
            best_document = document
        if deficits == 0:
            feasible.append((len(role_a) + len(role_b), ids_a, ids_b, document))
    if feasible:
        feasible.sort()
        return {"feasible": True, "selected_partition": feasible[0][3]}
    return {"best_frontier": best_document, "feasible": False}


def build_documents(
    profile: dict[str, Any],
    profile_data: bytes,
    capture: dict[str, Any],
    raw_by_url: dict[str, bytes],
    q0_data: bytes,
    q1_data: bytes,
) -> dict[str, bytes]:
    q0 = parse_json(q0_data, "Q0 inventory")
    q1 = parse_json(q1_data, "Q1 audit")
    if q1.get("schema") != Q1_SCHEMA:
        raise Q1ASourceGrowthError("Q1 audit schema mismatch")
    expected_urls = all_source_urls(profile)
    if sorted(raw_by_url) != expected_urls:
        raise Q1ASourceGrowthError("capture URL set does not match the frozen profile")
    expected_baseline_commit, expected_exposure_rows = baseline_exposure(profile)
    if capture.get("baseline_commit") != expected_baseline_commit:
        raise Q1ASourceGrowthError("capture baseline commit identity drift")
    if capture.get("baseline_exposure") != expected_exposure_rows:
        raise Q1ASourceGrowthError("capture baseline exposure evidence drift")
    exposure_rows = capture.get("baseline_exposure")
    if not isinstance(exposure_rows, list) or len(exposure_rows) != len(profile["projects"]):
        raise Q1ASourceGrowthError("baseline exposure ledger is incomplete")
    exposure_by_url = {row.get("url"): row for row in exposure_rows if isinstance(row, dict)}

    q0_project_pairs = {
        (group.get("publisher_id"), group.get("project_id"))
        for group in q0.get("groups", [])
        if isinstance(group, dict)
    }
    q1_project_ids = {
        row.get("project_revision_id")
        for row in q1.get("groups", [])
        if isinstance(row, dict)
    }
    normalized_projects = []
    candidate_power = []
    candidate_groups = []
    for project in profile["projects"]:
        author = project["author"]
        pack_id = project["pack_id"]
        pack_url = canonical_pack_url(project)
        pack_parser = parse_page(raw_by_url[pack_url], f"Freesound pack {pack_id}")
        expected_page_title = f"Freesound - {project['pack_title']} by {author}"
        if pack_parser.page_title != expected_page_title:
            raise Q1ASourceGrowthError(f"pack {pack_id} title/uploader mismatch")
        exposure = exposure_by_url.get(pack_url)
        if exposure is None or not isinstance(exposure.get("matches"), list):
            raise Q1ASourceGrowthError(f"pack {pack_id} lacks baseline exposure evidence")
        baseline_matches = exposure["matches"]
        project_id = f"freesound-user-{author}--pack-{pack_id}"
        overlaps_q0 = (f"freesound-user-{author}", f"pack-{pack_id}") in q0_project_pairs
        overlaps_q1 = any(
            isinstance(item, str) and project_id in item for item in q1_project_ids
        )
        if baseline_matches or overlaps_q0 or overlaps_q1:
            raise Q1ASourceGrowthError(f"candidate project {project_id} is not fresh")

        sound_cache: dict[int, dict[str, Any]] = {}
        for group in project["groups"]:
            sounds = []
            for sound_id in group["sound_ids"]:
                if sound_id not in sound_cache:
                    sound_cache[sound_id] = normalized_sound(
                        raw_by_url[canonical_sound_url(author, sound_id)],
                        author,
                        pack_id,
                        sound_id,
                    )
                sounds.append(sound_cache[sound_id])
            validate_group_evidence(group, sounds)
            candidate_groups.append(
                {
                    "evidence_phrase": group["evidence_phrase"],
                    "exposure_state": "current_metadata_hash_closed_signal_unopened",
                    "group_id": group["group_id"],
                    "object_name": group["object_name"],
                    "primary_material": group["primary_material"],
                    "project_revision_id": project_id,
                    "relation": group["relation"],
                    "sound_ids": group["sound_ids"],
                }
            )
        controls = []
        for control in project.get("conflict_controls", []):
            sound_id = control["sound_id"]
            sound = normalized_sound(
                raw_by_url[canonical_sound_url(author, sound_id)],
                author,
                pack_id,
                sound_id,
            )
            if (
                control["title_token"].lower() not in sound["title"].lower()
                or control["description_conflict_token"].lower()
                not in sound["description"].lower()
            ):
                raise Q1ASourceGrowthError("frozen material-conflict control changed")
            controls.append(
                {
                    "decision": "excluded_title_description_material_conflict",
                    "description_conflict_token": control["description_conflict_token"],
                    "sound": sound,
                    "title_token": control["title_token"],
                }
            )

        normalized_core = {
            "author": author,
            "conflict_controls": controls,
            "groups": [
                row for row in candidate_groups if row["project_revision_id"] == project_id
            ],
            "pack_id": pack_id,
            "pack_title": project["pack_title"],
            "pack_url": pack_url,
            "sounds": [sound_cache[key] for key in sorted(sound_cache)],
        }
        revision = sha256_bytes(canonical_json(normalized_core))
        normalized_core["revision_sha256"] = revision
        normalized_core["exposure"] = {
            "baseline_matches": baseline_matches,
            "q0_project_overlap": overlaps_q0,
            "q1_project_overlap": overlaps_q1,
            "state": "current_repository_metadata_audited_signal_unopened",
        }
        normalized_projects.append(normalized_core)
        relations = Counter(
            row["relation"] for row in normalized_core["groups"]
        )
        candidate_power.append(
            {
                "exact_steel_groups": relations["exact_steel_candidate"],
                "non_metal_groups": relations["non_metal_candidate"],
                "origin": "q1a-internet-metadata",
                "project_revision_id": f"{project_id}--sha256-{revision}",
            }
        )

    current_power = project_power_from_q0(q0)
    combined_power = sorted(
        current_power + candidate_power, key=lambda row: row["project_revision_id"]
    )
    partition = solve_protected_partition(combined_power, profile["minimums"])
    added_exact = sum(row["exact_steel_groups"] for row in candidate_power)
    added_rejects = sum(row["non_metal_groups"] for row in candidate_power)
    combined_exact = sum(row["exact_steel_groups"] for row in combined_power)
    combined_rejects = sum(row["non_metal_groups"] for row in combined_power)
    minimums = profile["minimums"]
    gates = {
        "added_exact_steel_groups": added_exact >= minimums["added_exact_steel_groups"],
        "added_non_metal_groups": added_rejects >= minimums["added_non_metal_groups"],
        "added_role_capable_projects": len(candidate_power)
        >= minimums["added_role_capable_projects"],
        "combined_protected_raw_exact_steel_floor": combined_exact
        >= 2 * minimums["protected_exact_steel_groups_per_role"],
        "combined_protected_raw_non_metal_floor": combined_rejects
        >= 2 * minimums["protected_non_metal_groups_per_role"],
        "whole_project_protected_partition": partition["feasible"],
        "zero_signal_access": all(
            capture["access"][key] == 0 for key in ZERO_SIGNAL_ACCESS
        ),
    }
    raw_gates = [value for key, value in gates.items() if key != "whole_project_protected_partition"]
    if not all(raw_gates):
        decision = "Q1A_SOURCE_GROWTH_INSUFFICIENT"
    elif partition["feasible"]:
        decision = "Q1A_BALANCED_SOURCE_POWER_FEASIBLE_FRESH_Q1_NEXT"
    else:
        decision = "Q1A_RAW_GROWTH_VERIFIED_BALANCED_ROLE_POWER_REQUIRED"

    audit_access = {
        **ZERO_SIGNAL_ACCESS,
        "network_requests": 0,
        "publisher_metadata_bytes_read": sum(map(len, raw_by_url.values())),
    }
    audit = {
        "access": {
            "audit": audit_access,
            "capture": capture["access"],
        },
        "authority": "METADATA_ONLY_SOURCE_GROWTH / NO_ROLE_OR_PAYLOAD_ACCESS_AUTHORITY",
        "candidate_groups": sorted(candidate_groups, key=lambda row: row["group_id"]),
        "candidate_projects": sorted(
            normalized_projects, key=lambda row: (row["author"], row["pack_id"])
        ),
        "decision": decision,
        "gates": gates,
        "input_identities": {
            "capture": {
                "bytes": len(canonical_json(capture)),
                "sha256": sha256_bytes(canonical_json(capture)),
            },
            "profile": {"bytes": len(profile_data), "sha256": sha256_bytes(profile_data)},
            "q0_inventory": {"bytes": len(q0_data), "sha256": sha256_bytes(q0_data)},
            "q1_audit": {"bytes": len(q1_data), "sha256": sha256_bytes(q1_data)},
        },
        "partition": partition,
        "power": {
            "added": {
                "exact_steel_groups": added_exact,
                "non_metal_groups": added_rejects,
                "project_revisions": len(candidate_power),
            },
            "combined": {
                "exact_steel_groups": combined_exact,
                "non_metal_groups": combined_rejects,
                "project_revisions": len(combined_power),
            },
            "projects": combined_power,
        },
        "schema": AUDIT_SCHEMA,
        "target_policy": "exact_unqualified_publisher_steel_v1",
    }
    audit_bytes = canonical_json(audit)
    report = {
        "access": audit["access"],
        "added_power": audit["power"]["added"],
        "audit_sha256": sha256_bytes(audit_bytes),
        "combined_power": audit["power"]["combined"],
        "decision": decision,
        "gates": gates,
        "input_identities": audit["input_identities"],
        "partition": partition,
        "schema": REPORT_SCHEMA,
    }
    return {
        "metal-source-growth-audit.json": audit_bytes,
        "report.json": canonical_json(report),
    }


def run_audit(
    profile_path: Path,
    capture_root: Path,
    q0_path: Path,
    q1_path: Path,
    output: Path,
) -> None:
    profile, profile_data = read_profile(profile_path)
    capture, raw_by_url = read_capture(capture_root, profile_data)
    q0_data = read_known_input(q0_path, Q0_IDENTITY, "Q0 inventory")
    q1_data = read_known_input(q1_path, Q1_IDENTITY, "Q1 audit")
    files = build_documents(
        profile, profile_data, capture, raw_by_url, q0_data, q1_data
    )
    write_fresh_directory(output, files)


def main() -> int:
    args = parse_arguments()
    try:
        if args.command == "capture":
            run_capture(args.profile, args.output)
        else:
            run_audit(
                args.profile,
                args.capture,
                args.q0_inventory,
                args.q1_audit,
                args.output,
            )
    except Q1ASourceGrowthError as error:
        raise SystemExit(f"Q1A_SOURCE_GROWTH_ERROR: {error}") from error
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
