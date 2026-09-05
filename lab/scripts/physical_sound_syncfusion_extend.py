"""Append verified author TRAIN shards without changing previous events or roles."""

import argparse
import collections
import hashlib
import io
import json
import tarfile
from pathlib import Path

import numpy as np
import physical_sound_syncfusion_adapter as adapter
import physical_sound_syncfusion_pilot as pilot
import soundfile as sf
import torch
from torch.nn import functional as F

SHARDS = {
    "train_shard_2.tar": (2086871040, "a630d0d456ddd32cdd1df289f9073eb1"),
    "train_shard_3.tar": (1955266560, "f003e4764debaf68097e2aefd613cd0c"),
}


def append_rows(previous, additions):
    old_keys = {r["recording"] for r in previous}
    new_keys = {r["recording"] for r in additions}
    if old_keys & new_keys:
        raise ValueError(
            "Duplicate recording across shards; no silent role reassignment"
        )
    if any(not r["recording"].startswith("train/") for r in additions):
        raise ValueError("Only author TRAIN additions")
    roles = collections.defaultdict(set)
    for row in additions:
        roles[row["recording"]].add(row["role"])
        if row["role"] == "train" and (row["material"], row["motion"]) == (
            "glass",
            "rigid-motion",
        ):
            raise ValueError("Held combination enters TRAIN")
    if any(len(values) != 1 for values in roles.values()):
        raise ValueError("Mixed recording roles")
    return previous + additions


def extend(data, archives, assets, package, output):
    original = json.loads((data / "data.json").read_text())
    if (
        original["status"] != "complete"
        or pilot.sha(data / "embeddings.pt") != original["embeddings_sha256"]
    ):
        raise ValueError("Changed base cache")
    if pilot.sha(assets / "model.ckpt") != original["checkpoint_sha256"]:
        raise ValueError("Changed encoder checkpoint")
    old_vectors = torch.load(data / "embeddings.pt", weights_only=True)
    if (
        old_vectors.shape != (len(original["rows"]), 512)
        or not torch.isfinite(old_vectors).all()
    ):
        raise ValueError("Invalid base embeddings")
    rows = original["rows"][:]
    for row in rows:
        if pilot.sha(Path(row["wav"])) != row["sha256"]:
            raise ValueError("Changed base WAV")
    verified, additions = [], []
    for path in archives:
        if path.name not in SHARDS or path.stat().st_size != SHARDS[path.name][0]:
            raise ValueError("Unexpected/incomplete author TRAIN shard")
        with path.open("rb") as stream:
            if hashlib.file_digest(stream, "md5").hexdigest() != SHARDS[path.name][1]:
                raise ValueError("Publisher MD5 mismatch")
        with tarfile.open(path, "r:") as archive:
            new, _ = adapter.selection(archive)
        for row in new:
            row["archive"] = path.name
        rows = append_rows(rows, new)
        additions.append((path, new))
        verified.append(
            {
                "name": path.name,
                "size": path.stat().st_size,
                "md5": SHARDS[path.name][1],
                "sha256": pilot.sha(path),
            }
        )
    output.mkdir(parents=True, exist_ok=False)
    report = {
        "status": "encoding",
        "rows": rows,
        "base_data_sha256": pilot.sha(data / "data.json"),
        "base_rows": len(original["rows"]),
        "base_archive_sha256": original.get(
            "archive_sha256", original.get("base_archive_sha256")
        ),
        "archives": original.get("archives", []) + verified,
        "checkpoint_sha256": original["checkpoint_sha256"],
        "split": original["split"],
        "author_role": original["author_role"],
    }
    pilot.save(output / "data.json", report)
    torch.set_num_threads(4)
    state = torch.load(
        assets / "model.ckpt", weights_only=True, mmap=True, map_location="cpu"
    )["state_dict"]
    encoder, projection = adapter.oracle.audio_encoder(package, state)
    encoder.cuda()
    projection.cuda()
    del state
    vectors = [old_vectors]
    count = len(original["rows"])
    for path, new in additions:
        with tarfile.open(path, "r:") as archive:
            members = {m.name: m for m in archive if m.isfile()}
            cached = None
            for row in new:
                key = row["recording"].removesuffix(".times.csv") + ".resampled.wav"
                if key != cached:
                    member = members[key]
                    if member.size > 100 * 1024**2:
                        raise ValueError("Oversized source WAV")
                    payload = archive.extractfile(member).read()
                    wave, rate = sf.read(io.BytesIO(payload), dtype="float32")
                    if (
                        rate != pilot.RATE
                        or wave.ndim != 1
                        or not np.isfinite(wave).all()
                    ):
                        raise ValueError("Invalid source PCM")
                    source_sha = hashlib.sha256(payload).hexdigest()
                    cached = key
                a, b = int(row["start"] * rate), int(row["end"] * rate)
                if not 0 <= a < b <= len(wave):
                    raise ValueError("Annotation bounds")
                event = wave[a:b]
                target = output / f"event-{count:04d}.wav"
                sf.write(target, event, rate, subtype="FLOAT")
                row.update(
                    samples=[a, b],
                    source_sha256=source_sha,
                    wav=str(target),
                    sha256=pilot.sha(target),
                )
                with torch.inference_mode():
                    value = encoder(
                        {"waveform": adapter.oracle.repeatpad(event)[None].cuda()},
                        device="cuda",
                    )["embedding"]
                    vectors.append(F.normalize(projection(value), dim=-1).cpu())
                count += 1
                if count % 50 == 0:
                    print("encoded", count, "/", len(rows), flush=True)
    embeddings = torch.cat(vectors)
    if embeddings.shape != (len(rows), 512) or not torch.isfinite(embeddings).all():
        raise ValueError("Invalid extended embeddings")
    if rows[: len(original["rows"])] != original["rows"] or not torch.equal(
        embeddings[: len(old_vectors)], old_vectors
    ):
        raise ValueError("Existing rows/roles/embeddings changed")
    torch.save(embeddings, output / "embeddings.pt")
    report.update(
        status="complete",
        embeddings_sha256=pilot.sha(output / "embeddings.pt"),
        role_counts=dict(collections.Counter(r["role"] for r in rows)),
    )
    pilot.save(output / "data.json", report)
    print(json.dumps(report["role_counts"]), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "assets", "package", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--archives", type=Path, nargs="+", required=True)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        parser.error("Artifacts must remain external")
    extend(args.data, args.archives, args.assets, args.package, args.output)
