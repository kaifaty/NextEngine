"""Small publisher-TRAIN EPIC-SOUNDS collision slice; research only, CC BY-NC4.

Huh, Chalk, Kazakos, Damen and Zisserman; EPIC-KITCHENS original recordings.
Material labels are unordered categories, not measured striker/target physics.
Remote MP4s are range-read by ffmpeg; whole-video MD5 is NOT verified.
"""

import argparse
import hashlib
import io
import json
import re
import subprocess
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

import numpy as np
import pandas as pd
import requests
from scipy.io import wavfile

ANNOTATIONS = "57a922f0d352e9429f1ef8a37eee21758dd3a33c"
DOWNLOADER = "4f11fb2b579833f360c3c7bb917bf1e24a9787b5"
RATE = 24000
CLASSES = (
    "metal / glass collision",
    "metal / wood collision",
    "wood / glass collision",
    "metal / plastic collision",
    "metal / ceramic collision",
    "plastic / wood collision",
)
BASE55 = "https://data.bris.ac.uk/datasets/3h91syskeag572hl6tvuovwv4d"
BASE100 = "https://data.bris.ac.uk/datasets/2g1n6qdydwa9u22shpxqzp0t8m"


def save(path, value):
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def select(frame):
    if frame.annotation_id.duplicated().any():
        raise ValueError("duplicate annotation")
    if not frame.annotation_id.str.fullmatch(r"P\d{2}_\d{2,3}_\d+").all():
        raise ValueError("invalid annotation identity")
    if not ((frame.start_sample >= 0) & (frame.stop_sample > frame.start_sample)).all():
        raise ValueError("invalid sample bounds")
    for key in ("start_sample", "stop_sample"):
        if not np.equal(frame[key], np.floor(frame[key])).all():
            raise ValueError("integer sample bounds required")
    result = []
    for label in CLASSES:
        seen = set()
        for row in frame[frame["class"] == label].to_dict("records"):
            if row["participant_id"] in seen:
                continue
            length = (row["stop_sample"] - row["start_sample"]) / RATE
            if not 0.25 <= length <= 3:
                continue
            overlapping = frame[
                (frame.video_id == row["video_id"])
                & (frame.start_sample < row["stop_sample"])
                & (frame.stop_sample > row["start_sample"])
                & (frame.annotation_id != row["annotation_id"])
            ]
            if len(overlapping):
                continue
            result.append(row)
            seen.add(row["participant_id"])
            if len(seen) == 4:
                break
        if len(seen) != 4:
            raise ValueError("insufficient participant coverage for " + label)
    return result


def video_source(row, splits55, checksums):
    video, participant = row["video_id"], row["participant_id"]
    if not re.fullmatch(r"P\d{2}_\d{2,3}", video) or video.split("_")[0] != participant:
        raise ValueError("invalid video identity")
    records = splits55[splits55.video_id == video]
    if len(records):
        if len(records) != 1 or records.iloc[0]["split"] not in ("train", "test"):
            raise ValueError("invalid original video split")
        path = f"videos/{records.iloc[0]['split']}/{participant}/{video}.MP4"
        base, version = BASE55, 55
    else:
        path, base, version = f"{participant}/videos/{video}.MP4", BASE100, 100
    matches = checksums[
        (checksums.file_remote_path == path)
        & (checksums.version.astype(str) == str(version))
    ]
    if len(matches) != 1 or not re.fullmatch(r"[a-f0-9]{32}", matches.iloc[0].md5):
        raise ValueError("missing publisher video checksum")
    return {
        "url": f"{base}/{path}",
        "publisher_md5": matches.iloc[0].md5,
        "whole_video_md5_verified": False,
        "epic_version": version,
    }


def extract(row, source, output):
    name = row["annotation_id"]
    raw = output / f"{name}-decoded.wav"
    samples = int(row["stop_sample"] - row["start_sample"])
    command = [
        "ffmpeg",
        "-nostdin",
        "-v",
        "error",
        "-n",
        "-tls_verify",
        "1",
        "-rw_timeout",
        "15000000",
        "-protocol_whitelist",
        "https,tls,tcp",
        "-ss",
        f"{row['start_sample'] / RATE:.9f}",
        "-i",
        source["url"],
        "-t",
        f"{samples / RATE:.9f}",
        "-map",
        "0:a:0",
        "-vn",
        "-ac",
        "1",
        "-ar",
        str(RATE),
        "-c:a",
        "pcm_s16le",
        "-map_metadata",
        "-1",
        str(raw),
    ]
    subprocess.run(command, check=True, capture_output=True, timeout=60)
    rate, pcm = wavfile.read(raw)
    if rate != RATE or pcm.dtype != np.int16 or pcm.shape != (samples,):
        raise ValueError("decoded audio does not match annotation sample bounds")
    peak = int(np.abs(pcm.astype(np.int32)).max())
    if peak == 0:
        raise ValueError("selected annotation decoded to silence")
    gain = min(1.0, 0.98 * 32767 / peak)
    published = np.rint(pcm.astype(np.float64) * gain).astype(np.int16)
    path = output / f"{name}.wav"
    wavfile.write(path, RATE, published)
    return {
        **row,
        **source,
        "role": "publisher_train_source_audition",
        "materials_unordered": row["class"].removesuffix(" collision").split(" / "),
        "physical_object_id": None,
        "striker_material": None,
        "dimensions": None,
        "force": None,
        "velocity": None,
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "decoded_wav": str(raw),
        "decoded_sha256": hashlib.sha256(raw.read_bytes()).hexdigest(),
        "samples": samples,
        "sample_rate": RATE,
        "pcm_gain": gain,
        "source_full_scale_samples": int((np.abs(pcm.astype(np.int32)) >= 32767).sum()),
    }


def run(output):
    if output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("dataset must stay outside the repository")
    output.mkdir(parents=True, exist_ok=False)
    report = {
        "status": "running",
        "license": "CC-BY-NC-4.0; local noncommercial research only",
        "source": "https://github.com/epic-kitchens/epic-sounds-annotations",
        "annotation_revision": ANNOTATIONS,
        "downloader_revision": DOWNLOADER,
        "selection": "publisher TRAIN only; classes in fixed order; first source-order nonoverlapping 0.25–3s annotation per participant, four participants per class; no audio-score selection",
        "scope": "real source recordings, NOT neural generation or calibrated physics; unknown pretraining overlap",
        "files": [],
        "rows": [],
    }
    try:

        def fetch(repo, revision, name, local):
            url = f"https://raw.githubusercontent.com/epic-kitchens/{repo}/{revision}/{name}"
            response = requests.get(url, timeout=30)
            response.raise_for_status()
            data = response.content
            if not 0 < len(data) <= 7_000_000:
                raise ValueError("metadata byte bound")
            (output / local).write_bytes(data)
            report["files"].append(
                {"path": local, "url": url, "sha256": hashlib.sha256(data).hexdigest()}
            )
            return data

        frame = pd.read_csv(
            io.BytesIO(
                fetch(
                    "epic-sounds-annotations",
                    ANNOTATIONS,
                    "EPIC_Sounds_train.csv",
                    "train.csv",
                )
            )
        )
        fetch("epic-sounds-annotations", ANNOTATIONS, "README.md", "SOURCE_README.md")
        splits = pd.read_csv(
            io.BytesIO(
                fetch(
                    "epic-kitchens-download-scripts",
                    DOWNLOADER,
                    "data/epic_55_splits.csv",
                    "video-splits55.csv",
                )
            )
        )
        hashes = pd.read_csv(
            io.BytesIO(
                fetch(
                    "epic-kitchens-download-scripts",
                    DOWNLOADER,
                    "data/md5.csv",
                    "video-md5.csv",
                )
            )
        )
        selected = select(frame)
        report["census"] = {
            "annotations": len(frame),
            "participants": int(frame.participant_id.nunique()),
            "videos": int(frame.video_id.nunique()),
            "classes": frame["class"].value_counts().to_dict(),
        }
        report["selected_annotations"] = [r["annotation_id"] for r in selected]
        sources = {r["video_id"]: video_source(r, splits, hashes) for r in selected}
        report["videos"] = sources
        save(output / "result.json", report)
        # No upstream script is executed. TLS verification remains enabled.
        for source in sources.values():
            head = requests.head(source["url"], timeout=30)
            head.raise_for_status()
            if head.status_code != 200 or head.headers.get("Accept-Ranges") != "bytes":
                raise ValueError("partial video access unavailable")
            source.update(
                remote_bytes=int(head.headers["Content-Length"]),
                etag=head.headers.get("ETag"),
            )
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [
                pool.submit(extract, r, sources[r["video_id"]], output)
                for r in selected
            ]
            for future in futures:
                report["rows"].append(future.result())
                save(output / "result.json", report)
                print(
                    {"clips": len(report["rows"]), "total": len(selected)}, flush=True
                )
        waves = []
        for row in report["rows"]:
            path = Path(row["wav"])
            if hashlib.sha256(path.read_bytes()).hexdigest() != row["sha256"]:
                raise ValueError("published waveform hash mismatch")
            rate, pcm = wavfile.read(path)
            assert rate == RATE
            waves.extend([pcm, np.zeros(RATE // 2, np.int16)])
        path = output / "comparison.wav"
        pcm = np.concatenate(waves)
        wavfile.write(path, RATE, pcm)
        report["comparison"] = {
            "wav": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "seconds": len(pcm) / RATE,
            "order": report["selected_annotations"],
        }
        report["status"] = "complete"
        save(output / "result.json", report)
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        save(output / "result.json", report)
        raise


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    run(parser.parse_args().output)
