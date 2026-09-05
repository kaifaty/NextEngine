"""Acquire a small, disclosed physical-control audio probe from Figshare.

Cluster Haptic Texture Dataset, Eguchi et al., CC BY 4.0, article v5.
Internet recordings, not generated sounds, training, or protected evaluation.
Only fixed members are read; no downloaded code or archive paths are executed.
"""

from __future__ import annotations

import argparse
import hashlib
import io
import json
import zipfile
from pathlib import Path

import fsspec
import numpy as np
import requests
import soundfile as sf

ARTICLE = "https://api.figshare.com/v2/articles/29438288/versions/5"
DOWNLOAD = "https://ndownloader.figshare.com/files/61118716"
ARCHIVE_MD5 = "4bc91dacaaf5afc33ec0a1fbf0122284"
# Multipart S3 ETag is an object identity, NOT the archive MD5.
ARCHIVE_ETAG = "f10dbaae9d8d3c7c915973047637a0af-15"
ARCHIVE_BYTES = 15220748922
PREFIX = "cluster_haptic_texture_dataset/"
# Names inspected in the archive's texture_list.xlsx; retain that original.
TEXTURES = {0: "Nyatoh", 65: "Stainless steel", 74: "Float glass"}


def conditions(training_grid: bool = False) -> list[dict]:
    return [
        {
            "id": f"{texture}_0_{speed}_{force}_{repeat}",
            "texture_id": texture,
            "texture_name": name,
            "probe_material": "urethane rubber",
            "direction_degrees": 0,
            "commanded_speed_mm_s": speed,
            "commanded_normal_force_N": force / 1000,
            "repeat": repeat,
        }
        for texture, name in TEXTURES.items()
        for speed in ((20, 30, 40, 50, 60) if training_grid else (20, 60))
        for force in (500, 1000)
        for repeat in ((0, 1) if training_grid else (0,))
    ]


def validate_audio(data: bytes, channels: int) -> dict:
    info = sf.info(io.BytesIO(data))
    if (
        info.format != "WAV"
        or info.samplerate != 44100
        or info.channels != channels
        or not 0 < info.frames <= 44100 * 15
    ):
        raise ValueError("unexpected audio format or length")
    audio, _ = sf.read(io.BytesIO(data), always_2d=True)
    if not np.isfinite(audio).all():
        raise ValueError("nonfinite source audio")
    return {
        "seconds": info.duration,
        "subtype": info.subtype,
        "channels": info.channels,
        "peak": np.abs(audio).max(axis=0).tolist(),
        "rms": np.sqrt(np.mean(audio**2, axis=0)).tolist(),
    }


def run(output: Path, training_grid: bool = False) -> dict:
    output = output.resolve()
    if output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("dataset must stay outside the repository")
    output.mkdir(parents=True, exist_ok=False)
    response = requests.get(ARTICLE, timeout=30)
    response.raise_for_status()
    if len(response.content) > 2_000_000:
        raise ValueError("oversized article metadata")
    article = response.json()
    entry = next(item for item in article["files"] if item["id"] == 61118716)
    if (
        article["version"] != 5
        or article["license"]["name"] != "CC BY 4.0"
        or entry["computed_md5"] != ARCHIVE_MD5
        or entry["size"] != ARCHIVE_BYTES
        or entry["download_url"] != DOWNLOAD
    ):
        raise ValueError("source identity changed")
    (output / "article.json").write_bytes(response.content)
    report = {
        "status": "acquiring",
        "scope": "disclosed development acquisition; no model or quality claim",
        "training_grid": training_grid,
        "article": ARTICLE,
        "article_sha256": hashlib.sha256(response.content).hexdigest(),
        "archive": entry,
        "license": article["license"],
        "authors": article["authors"],
        "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "integrity": "range access, source ETag, ZIP member CRC and local SHA256; whole archive MD5 NOT verified",
        "rows": [],
        "files": [],
    }

    def save() -> None:
        (output / "result.json").write_text(
            json.dumps(report, indent=2, allow_nan=False) + "\n"
        )

    def read_member(archive: zipfile.ZipFile, member: str, local: str) -> bytes:
        info = archive.getinfo(PREFIX + member)
        if not 0 < info.file_size <= 4_000_000 or info.compress_size > 4_000_000:
            raise ValueError("oversized ZIP member")
        data = archive.read(info)  # zipfile verifies the member CRC.
        path = output / local  # local names are constructed here, not archive paths.
        path.write_bytes(data)
        report["files"].append(
            {
                "member": info.filename,
                "path": str(path),
                "bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
                "zip_crc32": info.CRC,
            }
        )
        return data

    save()
    try:
        with requests.get(
            DOWNLOAD, headers={"Range": "bytes=0-0"}, stream=True, timeout=30
        ) as response:
            if (
                response.status_code != 206
                or response.headers.get("Content-Range") != f"bytes 0-0/{ARCHIVE_BYTES}"
                or response.headers.get("ETag", "").strip('"') != ARCHIVE_ETAG
            ):
                raise ValueError("range support or archive identity mismatch")
        # Figshare redirects to a signed URL valid for only ten seconds.
        # Resolve the canonical URL per range request, never retain that token.
        with fsspec.open(DOWNLOAD, "rb", block_size=65536) as remote:
            if remote.size != ARCHIVE_BYTES:
                raise ValueError("archive size changed")
            with zipfile.ZipFile(remote) as archive:
                read_member(archive, "README.md", "README.md")
                read_member(archive, "texture_list.xlsx", "texture_list.xlsx")
                for condition in conditions(training_grid):
                    row = dict(condition)
                    for kind in ("audio", "raw_audio", "force", "position"):
                        ext = "wav" if "audio" in kind else "csv"
                        filename = f"{row['id']}.{ext}"
                        local = f"{kind}-{filename}"
                        data = read_member(
                            archive,
                            f"sensor_data/{kind}/{row['texture_id']}/{filename}",
                            local,
                        )
                        row[kind] = str(output / local)
                        if "audio" in kind:
                            row[kind + "_signal"] = validate_audio(
                                data, 2 if kind == "raw_audio" else 1
                            )
                    report["rows"].append(row)
                    save()
                    print(json.dumps({"acquired": row["id"]}), flush=True)
        report["status"] = "complete"
    except Exception as error:  # noqa: BLE001 -- redact signed URLs at this boundary.
        report.update(status="failed", error_type=type(error).__name__)
        save()
        raise RuntimeError(f"acquisition failed: {type(error).__name__}") from None
    save()
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--training-grid", action="store_true")
    args = parser.parse_args()
    run(args.output, args.training_grid)
