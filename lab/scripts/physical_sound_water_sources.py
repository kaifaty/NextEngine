"""Acquire a bounded, attributed ESC-50 water/rain research subset externally.

Downloads data, never executes remote code. Dataset CC-BY-NC 3.0; retain
per-clip notices. Folds 1-4 train, fold 5 disclosed development, not clean test.
No physical intensity, geometry, force or material metadata is invented.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import json
import re
import subprocess
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import physical_sound_text_pilot as pilot
from scipy.io import wavfile
from scipy.signal import resample_poly

REVISION = "33c8ce9eb2cf0b1c2f8bcf322eb349b6be34dbb6"
BASE = f"https://raw.githubusercontent.com/karolpiczak/ESC-50/{REVISION}/"
PROMPTS = {
    "rain": "Rain falling and pattering.",
    "pouring_water": "Water being poured, splashing and bubbling.",
    "water_drops": "Water drops dripping and splashing.",
}
# These three have CC-Sampling+ notices, not the reviewed CC0/BY/BY-NC path.
# Leave their audio unfetched; do not silently treat a license name as consent.
DEFERRED = ("67152", "79220", "126433")


def fetch(relative, maximum):
    with urllib.request.urlopen(BASE + relative, timeout=30) as response:
        value = response.read(maximum + 1)
    if not value or len(value) > maximum:
        raise ValueError("download outside bounded size")
    return value


def select(metadata, notices):
    result = []
    for row in csv.DictReader(io.StringIO(metadata)):
        if row["category"] not in PROMPTS:
            continue
        if (
            not re.fullmatch(r"[1-5]-\d+-[A-Z]-\d+\.wav", row["filename"])
            or not row["src_file"].isdigit()
        ):
            raise ValueError("invalid ESC source identity")
        fold, source, take, target = row["filename"][:-4].split("-")
        if (fold, source, take, target) != (
            row["fold"],
            row["src_file"],
            row["take"],
            row["target"],
        ):
            raise ValueError("filename and metadata identity disagree")
        prefix = "- [" + row["filename"].rsplit("-", 1)[0] + ".ogg]"
        matches = [line for line in notices.splitlines() if line.startswith(prefix)]
        if len(matches) != 1 or f"/sounds/{row['src_file']}/" not in matches[0]:
            raise ValueError("missing or ambiguous source attribution")
        if not matches[0].endswith(("[CC0]", "[CC-BY]", "[CC-BY-NC]")):
            if row["src_file"] in DEFERRED and matches[0].endswith("[CC-Sampling+]"):
                continue
            raise ValueError("unknown source terms")
        row.update(
            attribution=matches[0],
            role="development" if row["fold"] == "5" else "train",
            prompt=PROMPTS[row["category"]],
            physical_attributes=None,
        )
        result.append(row)
    train = {r["src_file"] for r in result if r["role"] == "train"}
    development = {r["src_file"] for r in result if r["role"] == "development"}
    if not train or not development or train & development:
        raise ValueError("source-disjoint train/development required")
    return result


def prior_source_check(root, rows):
    # Conservative collision screen; no legacy audio or protected roles opened.
    # Record the scope rather than asserting unknown foundation pretraining is clean.
    found = subprocess.run(
        [
            "rg",
            "-l",
            "-g",
            "*.json",
            "--max-filesize",
            "2M",
            "freesound.org|freesound:",
            str(root),
        ],
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    if found.returncode not in (0, 1):
        raise ValueError("legacy source screen failed")
    paths = found.stdout.splitlines()
    ids = set()
    for name in paths:
        text = Path(name).read_text()
        ids.update(re.findall(r"/sounds/(\d+)/", text))
        ids.update(re.findall(r'"(?:sound_id|freesound_id)"\s*:\s*"?(\d+)', text))
        ids.update(re.findall(r"freesound:(\d+)", text))
    wanted = {r["src_file"] for r in rows}
    names = subprocess.run(
        ["rg", "--files", str(root)],
        capture_output=True,
        text=True,
        check=True,
        timeout=60,
    ).stdout
    filename_ids = set(
        re.findall(r"(?:^|[/_-])(\d{4,})(?=[_.-])", names, flags=re.MULTILINE)
    )
    overlap = sorted(wanted & (ids | filename_ids))
    if overlap:
        raise ValueError(
            f"existing source identities require role review before download: {overlap}"
        )
    return {
        "url_bearing_json_files_under_2mb": len(paths),
        "known_ids": len(ids),
        "existing_filenames_screened": True,
        "overlap": overlap,
        "scope": "local URL-bearing JSON metadata and filenames only; pretraining overlap unknown",
    }


def run(output, prior_root):
    output = output.resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("output must be a new external directory")
    metadata = fetch("meta/esc50.csv", 1024**2)
    notices = fetch("LICENSE", 2 * 1024**2)
    rows = select(metadata.decode(), notices.decode())
    if len(rows) != 117:
        raise ValueError("unexpected pinned subset size")
    screen = prior_source_check(prior_root, rows)
    output.mkdir(parents=True)
    (output / "audio").mkdir()
    (output / "esc50.csv").write_bytes(metadata)
    (output / "LICENSE").write_bytes(notices)
    report = {
        "status": "running",
        "dataset": "karolpiczak/ESC-50",
        "revision": REVISION,
        "license": "dataset CC-BY-NC 3.0; individual attribution retained; external research only",
        "metadata_sha256": hashlib.sha256(metadata).hexdigest(),
        "license_sha256": hashlib.sha256(notices).hexdigest(),
        "scope": "weak event-class labels; no physical parameter or pristine holdout claim",
        "source_screen": screen,
        "excluded_sources": {
            source: "CC-Sampling+ path not reviewed; audio not fetched"
            for source in DEFERRED
        },
        "rows": [],
    }

    def acquire(row):
        value = fetch("audio/" + row["filename"], 1024**2)
        rate, pcm = wavfile.read(io.BytesIO(value))
        if (
            rate != 44100
            or pcm.dtype != np.int16
            or pcm.shape != (220500,)
            or not np.any(pcm)
        ):
            raise ValueError("expected non-silent five-second mono PCM16/44.1kHz")
        path = output / "audio" / row["filename"]
        path.write_bytes(value)
        return {
            **row,
            "wav": str(path),
            "sha256": hashlib.sha256(value).hexdigest(),
            "source_url": BASE + "audio/" + row["filename"],
            "pcm_peak": int(np.max(np.abs(pcm.astype(np.int32)))),
        }

    try:
        with ThreadPoolExecutor(max_workers=4) as pool:
            for row in pool.map(acquire, rows):
                report["rows"].append(row)
                pilot.save_report(output / "result.json", report)
                if len(report["rows"]) % 20 == 0:
                    print(
                        json.dumps(
                            {"downloaded": len(report["rows"]), "of": len(rows)}
                        ),
                        flush=True,
                    )
        preview = []
        for category in PROMPTS:
            for role in ("train", "development"):
                row = next(
                    r
                    for r in report["rows"]
                    if r["category"] == category and r["role"] == role
                )
                _, pcm = wavfile.read(row["wav"])
                mono = resample_poly(pcm.astype(np.float32) / 32768, 160, 441)
                preview.extend((mono, np.zeros(8000, np.float32)))
        pilot.write_audio(output / "sources-preview.wav", np.concatenate(preview))
        report.update(
            status="complete",
            train=sum(r["role"] == "train" for r in rows),
            development=sum(r["role"] == "development" for r in rows),
            unique_sources=len({r["src_file"] for r in rows}),
        )
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        pilot.save_report(output / "result.json", report)
        raise
    pilot.save_report(output / "result.json", report)
    print(
        json.dumps(
            {k: report[k] for k in ("status", "train", "development", "unique_sources")}
        ),
        flush=True,
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--prior-root", type=Path, required=True)
    args = parser.parse_args()
    run(args.output, args.prior_root.resolve())
