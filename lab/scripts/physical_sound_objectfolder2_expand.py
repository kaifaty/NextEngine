"""One bounded continuation of the official OF2 archive; no model execution.

Retain complete regular mesh/checkpoint pairs only. The gzip prefix is incomplete,
so individual hashes are provenance, not verification of the full archive.
New roles depend on identity before audio inspection, not a quality score.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import re
import tarfile
import urllib.request
from contextlib import ExitStack
from pathlib import Path

URL = "https://download.cs.stanford.edu/viscam/ObjectFolder/ObjectFolder1-100.tar.gz"
START = 335544320
LENGTH = 268435456
TOTAL = 3770228811
ETAG = '"624aa041-e0b9204b"'
PREFIX_SHA = "45050c62562628b9c984a8e46bc3ae350169b4890f2262f613c701d09472c474"
CSV_SHA = "5565b7e8f739194616d5f276e49b6e3bbf34f447f2a07b79ad02b1c23d8b1557"
EXISTING = {7, 11, 23, 29, 54, 66, 75, 82, 88}


def digest(path):
    with path.open("rb") as f:
        return hashlib.file_digest(f, "sha256").hexdigest()


class Joined(io.RawIOBase):
    """Read consecutive local chunks without copying another archive to disk."""

    def __init__(self, files):
        self.files = iter(files)
        self.current = next(self.files, None)

    def readable(self):
        return True

    def readinto(self, buffer):
        while self.current is not None:
            count = self.current.readinto(buffer)
            if count:
                return count
            self.current = next(self.files, None)
        return 0


def member_target(member):
    match = re.fullmatch(
        r"ObjectFolder1-100/([1-9][0-9]?|100)/(model\.obj|ObjectFile\.pth)", member.name
    )
    if not match or not member.isfile() or not 0 < member.size <= 60_000_000:
        return None
    identity = int(match[1])
    return (identity, match[2]) if identity not in EXISTING else None


def role(identity):
    # Stable identity-only development subset; old roles never change.
    return "development" if identity % 5 == 0 else "train"


def run(args):
    if args.prefix.stat().st_size != START or digest(args.prefix) != PREFIX_SHA:
        raise ValueError("wrong established prefix")
    if digest(args.metadata) != CSV_SHA:
        raise ValueError("metadata changed")
    metadata = {
        int(r[0]): r for r in csv.reader(args.metadata.read_text().splitlines())
    }
    args.output.mkdir(parents=True)
    part = args.output / "range320to576m.gz"
    request = urllib.request.Request(
        URL, headers={"Range": f"bytes={START}-{START + LENGTH - 1}", "If-Range": ETAG}
    )
    with urllib.request.urlopen(request, timeout=45) as response:
        if (
            response.status != 206
            or response.headers.get("Content-Range")
            != f"bytes {START}-{START + LENGTH - 1}/{TOTAL}"
            or response.headers.get("ETag") != ETAG
        ):
            raise ValueError("archive revision/range changed")
        with part.open("xb") as f:
            remaining = LENGTH
            while remaining:
                block = response.read(min(1024 * 1024, remaining))
                if not block:
                    raise ValueError("short HTTP range; preserve partial file")
                f.write(block)
                remaining -= len(block)
            if response.read(1):
                raise ValueError("overlong HTTP range")
    print("range complete", part.stat().st_size, flush=True)
    files, unfinished, terminal = {}, None, None
    with ExitStack() as stack:
        chunks = [stack.enter_context(p.open("rb")) for p in (args.prefix, part)]
        stream = io.BufferedReader(Joined(chunks))
        try:
            with tarfile.open(fileobj=stream, mode="r|gz") as archive:
                for member in archive:
                    target = member_target(member)
                    if target is None:
                        continue
                    identity, name = target
                    directory = args.output / f"object-{identity}"
                    directory.mkdir(exist_ok=True)
                    temporary = directory / (name + ".partial")
                    unfinished = str(temporary)
                    with (
                        archive.extractfile(member) as source,
                        temporary.open("xb") as out,
                    ):
                        remaining = member.size
                        while remaining:
                            block = source.read(min(1024 * 1024, remaining))
                            if not block:
                                raise EOFError("incomplete selected member")
                            out.write(block)
                            remaining -= len(block)
                    destination = directory / name
                    if destination.exists():
                        raise ValueError("duplicate archive member")
                    temporary.rename(destination)
                    unfinished = None
                    files.setdefault(identity, {})[name] = {
                        "path": str(destination),
                        "sha256": digest(destination),
                        "bytes": member.size,
                        "member": member.name,
                    }
                    print("complete member", identity, name, flush=True)
        except (EOFError, tarfile.ReadError) as error:
            terminal = str(error)
    pairs = []
    for identity, members in sorted(files.items()):
        if set(members) == {"model.obj", "ObjectFile.pth"}:
            row = metadata[identity]
            pairs.append(
                {
                    "object_id": identity,
                    "role": role(identity),
                    "name": row[1],
                    "published_size": row[2],
                    "material": row[3],
                    "mesh_source": row[4],
                    "files": members,
                }
            )
    report = {
        "url": URL,
        "etag": ETAG,
        "range": [START, START + LENGTH - 1],
        "archive_length": TOTAL,
        "prefix_sha256": PREFIX_SHA,
        "part_sha256": digest(part),
        "full_archive_checksum_verified": False,
        "metadata_sha256": CSV_SHA,
        "rows": pairs,
        "all_complete_members": files,
        "unfinished_member": unfinished,
        "terminal_prefix_error": terminal,
        "role_rule": "new ID divisible by5=development, otherwise train; prior nine untouched; no acoustic selection",
        "scope": "external research only; OF2 CC BY4 plus original mesh terms, not blanket redistribution",
    }
    (args.output / "expansion.json").write_text(json.dumps(report, indent=2) + "\n")
    print(
        json.dumps(
            [
                {k: r[k] for k in ("object_id", "role", "name", "material")}
                for r in pairs
            ],
            indent=2,
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", type=Path, required=True)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    run(parser.parse_args())
