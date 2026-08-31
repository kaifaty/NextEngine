#!/usr/bin/env python3
"""Acquire bounded ObjectFolder object-92 raw prefixes in exact 1 GiB chunks."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, BinaryIO

URL = (
    "https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/"
    "audio_data_91_100.tar.gz"
)
HOST = "download.cs.stanford.edu"
FULL_BYTES = 38_866_981_268
ETAG = '"63e3844c-90ca71194"'
LAST_MODIFIED = "Wed, 08 Feb 2023 11:15:24 GMT"
GIB = 1_073_741_824
CHECKPOINTS_GIB = (4, 8, 12)
SEED_BYTES = 536_870_912
SEED_SHA256 = "5ef9789ad023233377aa60d58f66100f0bb62dd5df52556e051e757d13797313"
PARENT_MANIFEST_SHA256 = "b30f88978bd6dc749661c92a13093a97d6eda98771c75b058cc6ff149b5ea6b1"
PARENT_REPORT_SHA256 = "aafcbfdd551a0f18b0ec6fb390b093abb19825162455f49822fde718bacc0276"
MARKER_SCHEMA = "nextengine.external-physical-sound-object92-raw-acquisition.v0"
REPORT_SCHEMA = "nextengine.experimental-physical-sound-object92-raw-acquisition.report.v0"


class AcquisitionError(RuntimeError):
    """The frozen M2b acquisition boundary was violated."""


def repository_root() -> Path:
    return Path(__file__).resolve().parents[2]


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        while block := handle.read(8 * 1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def require_parent(path: Path, expected_sha256: str, kind: str) -> Path:
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(repository_root()) or not resolved.is_file():
        raise AcquisitionError(f"{kind} must be an external regular file")
    if sha256_file(resolved) != expected_sha256:
        raise AcquisitionError(f"{kind} identity changed")
    return resolved


def require_seed(path: Path | None) -> Path | None:
    if path is None:
        return None
    resolved = path.resolve(strict=True)
    if resolved.is_relative_to(repository_root()) or not resolved.is_file():
        raise AcquisitionError("seed must be an external regular file")
    if resolved.stat().st_size != SEED_BYTES or sha256_file(resolved) != SEED_SHA256:
        raise AcquisitionError("optional seed identity changed")
    return resolved


def prepare_root(path: Path) -> Path:
    root = path.resolve()
    if root.is_relative_to(repository_root()):
        raise AcquisitionError("acquisition root must stay outside the repository")
    marker = root / "acquisition-marker.json"
    expected = canonical_json({"schema": MARKER_SCHEMA, "remote_url": URL})
    if root.exists():
        if not root.is_dir() or not marker.is_file() or marker.read_bytes() != expected:
            raise AcquisitionError("existing acquisition root is not an exact M2b root")
    else:
        root.mkdir(parents=True)
        marker.write_bytes(expected)
    (root / "chunks").mkdir(exist_ok=True)
    return root


class SameHostRedirectHandler(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, request, fp, code, msg, headers, new_url):  # type: ignore[no-untyped-def]
        parsed = urllib.parse.urlparse(new_url)
        if parsed.scheme != "https" or parsed.hostname != HOST:
            raise AcquisitionError("remote redirected outside the frozen HTTPS host")
        return super().redirect_request(request, fp, code, msg, headers, new_url)


def validate_response(response: BinaryIO, start: int, end: int) -> None:
    status = getattr(response, "status", None)
    if status != 206:
        raise AcquisitionError(f"range response status changed: {status}")
    headers = response.headers  # type: ignore[attr-defined]
    expected_range = f"bytes {start}-{end}/{FULL_BYTES}"
    if headers.get("Content-Range") != expected_range:
        raise AcquisitionError("range response Content-Range changed")
    if int(headers.get("Content-Length", "-1")) != end - start + 1:
        raise AcquisitionError("range response Content-Length changed")
    if headers.get("ETag") != ETAG:
        raise AcquisitionError("range response ETag changed")
    if headers.get("Last-Modified") != LAST_MODIFIED:
        raise AcquisitionError("range response Last-Modified changed")
    final_url = urllib.parse.urlparse(response.geturl())  # type: ignore[attr-defined]
    if final_url.scheme != "https" or final_url.hostname != HOST:
        raise AcquisitionError("range response final URL changed")


def fetch_range(target: Path, start: int, end: int) -> dict[str, Any]:
    request = urllib.request.Request(
        URL,
        headers={
            "Range": f"bytes={start}-{end}",
            "Accept-Encoding": "identity",
            "User-Agent": "NextEngine-PhysicalSound-Research/0",
        },
    )
    opener = urllib.request.build_opener(SameHostRedirectHandler())
    digest = hashlib.sha256()
    byte_count = 0
    try:
        with opener.open(request, timeout=120) as response:
            validate_response(response, start, end)
            with target.open("ab") as output:
                while block := response.read(8 * 1024 * 1024):
                    output.write(block)
                    digest.update(block)
                    byte_count += len(block)
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        raise AcquisitionError(f"range fetch failed: {start}-{end}") from error
    if byte_count != end - start + 1:
        raise AcquisitionError("range response body was truncated")
    return {
        "start": start,
        "end": end,
        "bytes": byte_count,
        "payload_sha256": digest.hexdigest(),
    }


def build_chunk(
    chunks: Path,
    index: int,
    seed: Path | None,
) -> tuple[Path, list[dict[str, Any]]]:
    start = index * GIB
    end = start + GIB - 1
    final = chunks / f"chunk-{index:02d}-{start}-{end}.bin"
    part = chunks / f".{final.name}.part"
    if final.exists():
        if final.stat().st_size != GIB:
            raise AcquisitionError(f"existing chunk {index} has wrong size")
        return final, []
    if part.exists():
        raise AcquisitionError(f"partial chunk {index} requires explicit removal/recovery")
    requests: list[dict[str, Any]] = []
    if index == 0 and seed is not None:
        with seed.open("rb") as source, part.open("xb") as output:
            shutil.copyfileobj(source, output, 8 * 1024 * 1024)
        requests.append(fetch_range(part, SEED_BYTES, end))
    else:
        part.touch(exist_ok=False)
        requests.append(fetch_range(part, start, end))
    if part.stat().st_size != GIB:
        raise AcquisitionError(f"assembled chunk {index} has wrong size")
    part.rename(final)
    return final, requests


def assemble_prefix(root: Path, chunks: list[Path], checkpoint_gib: int) -> tuple[Path, str]:
    final = root / f"audio_data_91_100.prefix-{checkpoint_gib}g.tar.gz"
    expected_size = checkpoint_gib * GIB
    if final.exists():
        if final.stat().st_size != expected_size:
            raise AcquisitionError("existing prefix has wrong size")
        return final, sha256_file(final)
    part = root / f".{final.name}.part"
    if part.exists():
        raise AcquisitionError("partial assembled prefix requires explicit recovery")
    digest = hashlib.sha256()
    with part.open("xb") as output:
        for chunk in chunks:
            with chunk.open("rb") as source:
                while block := source.read(8 * 1024 * 1024):
                    output.write(block)
                    digest.update(block)
    if part.stat().st_size != expected_size:
        raise AcquisitionError("assembled prefix byte count changed")
    part.rename(final)
    return final, digest.hexdigest()


def acquire(
    parent_manifest: Path,
    parent_report: Path,
    root_argument: Path,
    checkpoint_gib: int,
    seed_argument: Path | None,
) -> Path:
    if checkpoint_gib not in CHECKPOINTS_GIB:
        raise AcquisitionError("checkpoint must be one of 4, 8 or 12 GiB")
    require_parent(parent_manifest, PARENT_MANIFEST_SHA256, "parent manifest")
    require_parent(parent_report, PARENT_REPORT_SHA256, "parent report")
    seed = require_seed(seed_argument)
    root = prepare_root(root_argument)
    chunks: list[Path] = []
    requests: list[dict[str, Any]] = []
    for index in range(checkpoint_gib):
        chunk, new_requests = build_chunk(root / "chunks", index, seed)
        chunks.append(chunk)
        requests.extend(new_requests)
        print(f"chunk {index + 1}/{checkpoint_gib} ready", flush=True)
    prefix, prefix_sha256 = assemble_prefix(root, chunks, checkpoint_gib)
    report = {
        "schema": REPORT_SCHEMA,
        "status": "Acquired",
        "decision": "READY_FOR_ZERO_SAMPLE_INVENTORY",
        "checkpoint_gib": checkpoint_gib,
        "prefix": {
            "relative_path": prefix.name,
            "bytes": prefix.stat().st_size,
            "sha256": prefix_sha256,
        },
        "source": {
            "url": URL,
            "full_bytes": FULL_BYTES,
            "etag": ETAG,
            "last_modified": LAST_MODIFIED,
        },
        "parent_manifest_sha256": PARENT_MANIFEST_SHA256,
        "parent_report_sha256": PARENT_REPORT_SHA256,
        "exact_seed_used": seed is not None,
        "network_requests_this_invocation": len(requests),
        "network_payload_bytes_this_invocation": sum(item["bytes"] for item in requests),
        "requests": requests,
        "microphone_sample_values_decoded": 0,
        "force_sample_values_decoded": 0,
        "striking_force_numeric_values_decoded": 0,
        "payloads_emitted_to_repository": 0,
    }
    report_path = root / f"acquisition-report-{checkpoint_gib}g.json"
    encoded = canonical_json(report)
    if report_path.exists() and report_path.read_bytes() != encoded:
        raise AcquisitionError("existing acquisition report differs")
    report_path.write_bytes(encoded)
    return report_path


def parse_arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--parent-manifest", required=True, type=Path)
    parser.add_argument("--parent-report", required=True, type=Path)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--checkpoint-gib", required=True, type=int)
    parser.add_argument("--seed", type=Path)
    return parser.parse_args()


def main() -> int:
    arguments = parse_arguments()
    acquire(
        arguments.parent_manifest,
        arguments.parent_report,
        arguments.root,
        arguments.checkpoint_gib,
        arguments.seed,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
