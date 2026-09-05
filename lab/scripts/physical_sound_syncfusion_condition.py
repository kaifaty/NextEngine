"""TRAIN-only conditioning discriminator; audio prototypes are diagnostic oracles."""

import argparse
import csv
import hashlib
import importlib
import io
import itertools
import json
import sys
import tarfile
import time
import types
from pathlib import Path

import numpy as np
import physical_sound_syncfusion_pilot as pilot
import soundfile as sf
import torch

URL = "https://zenodo.org/api/records/12634671/files/train_shard_1.tar/content"
MATERIALS = ("glass", "wood")
TIMES = [0.6, 1.5, 2.7, 4.0]


def first_hit(annotation, material):
    rows = list(csv.reader(io.StringIO(annotation)))
    rows = [(float(r[0]), r[1]) for r in rows if r]
    for (start, label), (end, _) in itertools.pairwise(rows):
        if label.split()[:2] == [material, "hit"] and 0.2 <= end - start <= 2:
            return start, end, label
    return None


def acquire(output, archive_path):
    import requests

    if archive_path.stat().st_size != 2099322880:
        raise ValueError("Incomplete TRAIN archive")
    with archive_path.open("rb") as stream:
        if (
            hashlib.file_digest(stream, "md5").hexdigest()
            != "d95042871d2e1c5892acc5a31bd35cb9"
        ):
            raise ValueError("TRAIN archive MD5 mismatch")
    output.mkdir(parents=True, exist_ok=False)
    response = requests.get("https://zenodo.org/api/records/12634671", timeout=60)
    response.raise_for_status()
    metadata = response.json()
    pilot.save(output / "publisher.json", metadata)
    entry = next(f for f in metadata["files"] if f["key"] == "train_shard_1.tar")
    if (
        entry["size"] != 2099322880
        or entry["checksum"] != "md5:d95042871d2e1c5892acc5a31bd35cb9"
    ):
        raise ValueError("Publisher TRAIN shard changed")
    report = {
        "status": "acquiring",
        "url": URL,
        "role": "author_train",
        "license": metadata["metadata"]["license"],
        "whole_archive_md5_verified": True,
        "archive_sha256": pilot.sha(archive_path),
        "selected": [],
        "visited": [],
        "selection": "first two distinct archive recordings per glass/wood with first annotated hit interval 0.2..2s",
    }
    pilot.save(output / "source.json", report)
    counts = dict.fromkeys(MATERIALS, 0)
    members = {}
    with (
        archive_path.open("rb") as remote,
        tarfile.open(fileobj=remote, mode="r:") as archive,
    ):
        for member in archive:
            if not member.isfile() or not member.name.startswith("train/"):
                raise ValueError("Unexpected non-TRAIN member")
            if member.name.endswith(".resampled.wav"):
                members[member.name] = member
            if not member.name.endswith(".times.csv"):
                continue
            if member.size > 100000:
                raise ValueError("Oversized annotation")
            data = archive.extractfile(member).read()
            annotation = data.decode("utf-8")
            key = member.name.removesuffix(".times.csv")
            report["visited"].append(key)
            for material in MATERIALS:
                hit = first_hit(annotation, material)
                if counts[material] >= 2 or hit is None:
                    continue
                wav_member = members[key + ".resampled.wav"]
                if wav_member.size > 100 * 1024**2:
                    raise ValueError("Oversized source WAV")
                payload = archive.extractfile(wav_member).read()
                wave, rate = sf.read(io.BytesIO(payload), dtype="float32")
                if rate != pilot.RATE or wave.ndim != 1 or not np.isfinite(wave).all():
                    raise ValueError("Invalid source waveform")
                start, end, label = hit
                a, b = int(start * rate), int(end * rate)
                if not 0 <= a < b <= wave.size:
                    raise ValueError("Annotation outside waveform")
                name = f"{material}-{counts[material]}"
                (output / f"{name}-source.wav").write_bytes(payload)
                (output / f"{name}-times.csv").write_bytes(data)
                sf.write(output / f"{name}-event.wav", wave[a:b], rate, subtype="FLOAT")
                row = {
                    "id": name,
                    "material": material,
                    "key": key,
                    "label": label,
                    "start": start,
                    "end": end,
                    "samples": [a, b],
                    "source_sha256": pilot.sha(output / f"{name}-source.wav"),
                    "event_sha256": pilot.sha(output / f"{name}-event.wav"),
                    "annotation_sha256": pilot.sha(output / f"{name}-times.csv"),
                    "archive_member": wav_member.name,
                    "archive_offset": wav_member.offset_data,
                }
                report["selected"].append(row)
                counts[material] += 1
                print(json.dumps(row), flush=True)
            pilot.save(output / "source.json", report)
            if all(n == 2 for n in counts.values()):
                break
    report["status"] = (
        "complete" if all(n == 2 for n in counts.values()) else "insufficient"
    )
    pilot.save(output / "source.json", report)


def repeatpad(wave):
    """Author non-fusion path: PCM16 quantization, whole repeats then zero pad."""
    wave = np.asarray(wave, dtype=np.float32)
    if wave.ndim != 1 or not 0 < wave.size <= 480000 or not np.isfinite(wave).all():
        raise ValueError("Expected finite short mono conditioning event")
    quantized = (np.clip(wave, -1, 1) * 32767).astype(np.int16).astype(
        np.float32
    ) / 32767
    repeated = np.tile(quantized, 480000 // wave.size)
    return torch.from_numpy(np.pad(repeated, (0, 480000 - repeated.size)))


def audio_encoder(package, state):
    # Load only unmodified upstream HTSAT submodules, not global CLAP training hooks.
    directory = package / "clap_module"
    module = types.ModuleType("syncfusion_clap")
    module.__path__ = [str(directory)]
    sys.modules["syncfusion_clap"] = module
    htsat = importlib.import_module("syncfusion_clap.htsat")
    cfg = json.loads((directory / "model_configs/HTSAT-tiny.json").read_text())[
        "audio_cfg"
    ]
    model = htsat.create_htsat_model(types.SimpleNamespace(**cfg), enable_fusion=False)
    model.load_state_dict(pilot.subset(state, "clap.model.audio_branch."), strict=True)
    projection = torch.nn.Sequential(
        torch.nn.Linear(768, 512), torch.nn.ReLU(), torch.nn.Linear(512, 512)
    )
    projection.load_state_dict(
        pilot.subset(state, "clap.model.audio_projection."), strict=True
    )
    return model.eval().requires_grad_(False), projection.eval().requires_grad_(False)


def render(assets, source, package, output):
    output.mkdir(parents=True, exist_ok=False)
    source_report = json.loads((source / "source.json").read_text())
    identity = json.loads((assets / "source.json").read_text())
    if (
        source_report["status"] != "complete"
        or pilot.sha(assets / "model.ckpt") != identity["checkpoint_sha256"]
    ):
        raise ValueError("Changed or incomplete inputs")
    torch.set_num_threads(4)
    torch.manual_seed(42)
    state = torch.load(
        assets / "model.ckpt", weights_only=True, mmap=True, map_location="cpu"
    )["state_dict"]
    audio, projection = audio_encoder(package, state)
    audio.cuda()
    projection.cuda()
    events, vectors = [], []
    with torch.inference_mode():
        for row in source_report["selected"]:
            path = source / f"{row['id']}-event.wav"
            if pilot.sha(path) != row["event_sha256"]:
                raise ValueError("Changed TRAIN event")
            wave, rate = sf.read(path, dtype="float32")
            if rate != pilot.RATE:
                raise ValueError("Changed source rate")
            embedded = audio({"waveform": repeatpad(wave)[None].cuda()}, device="cuda")[
                "embedding"
            ]
            vector = torch.nn.functional.normalize(projection(embedded), dim=-1).cpu()
            if not torch.isfinite(vector).all():
                raise ValueError("Nonfinite condition")
            vectors.append(vector)
            events.append(row)
    del audio, projection
    prototypes = {}
    for material in MATERIALS:
        selected = [
            v for r, v in zip(events, vectors, strict=True) if r["material"] == material
        ]
        prototypes[material] = torch.nn.functional.normalize(
            torch.cat(selected).mean(0), dim=-1
        )[None, None]
        prototypes[material + "-individual"] = selected[0][:, None]
    torch.save(
        {"individual": torch.cat(vectors), **prototypes}, output / "conditions.pt"
    )
    model, encoder = pilot.build()
    model.load_state_dict(pilot.subset(state, "model."), strict=True)
    encoder.load_state_dict(pilot.subset(state, "onsets_encoder."), strict=True)
    del state
    model.cuda()
    encoder.cuda()
    result = {
        "status": "running",
        "checkpoint_sha256": identity["checkpoint_sha256"],
        "source_sha256": pilot.sha(source / "source.json"),
        "conditions_sha256": pilot.sha(output / "conditions.pt"),
        "reference_audio_input": True,
        "claim": "TRAIN audio-prototype diagnostic oracle, not source-free generalization",
        "training_updates": 0,
        "seed": 42,
        "steps": 150,
        "embedding_scale": 2,
        "events_seconds": TIMES,
        "playback_gain": 0.5,
        "rows": [],
    }
    pilot.save(output / "result.json", result)
    for material, condition in prototypes.items():
        started = time.monotonic()
        with torch.inference_mode():
            _, context = encoder(pilot.event_track(TIMES).cuda(), with_info=True)
            noise = torch.randn(
                1,
                1,
                pilot.LENGTH,
                device="cuda",
                generator=torch.Generator(device="cuda").manual_seed(42),
            )
            wave = (
                model.sample(
                    x_noisy=noise,
                    num_steps=150,
                    channels=context["xs"][2:-1],
                    embedding=condition.cuda(),
                    embedding_scale=2,
                )[0, 0]
                .cpu()
                .numpy()
            )
        raw = output / f"{material}-raw.wav"
        sf.write(raw, wave, pilot.RATE, subtype="FLOAT")
        peak = float(np.abs(wave).max())
        row = {
            "id": material,
            "raw_sha256": pilot.sha(raw),
            "raw_peak": peak,
            "seconds": time.monotonic() - started,
        }
        if np.isfinite(wave).all() and peak * 0.5 < 0.98:
            path = output / f"{material}.wav"
            sf.write(path, wave * 0.5, pilot.RATE, subtype="PCM_16")
            row.update(
                status="published",
                wav=str(path),
                sha256=pilot.sha(path),
                timing=pilot.timing.match(
                    TIMES, pilot.timing.onsets(wave * 0.5, pilot.RATE)
                ),
            )
        else:
            row.update(
                status="rejected", reason="fixed gain headroom or finite failure"
            )
        result["rows"].append(row)
        pilot.save(output / "result.json", result)
        print(json.dumps(row), flush=True)
    result.update(
        status="complete_with_rejections"
        if any(r["status"] == "rejected" for r in result["rows"])
        else "complete",
        cuda_peak_gib=torch.cuda.max_memory_allocated() / 2**30,
    )
    pilot.save(output / "result.json", result)


def spectral_shape(wave):
    """Gain-invariant 32-band log PSD of a fixed 200ms attack window."""
    from scipy.signal import welch

    wave = np.asarray(wave, dtype=np.float64)
    if wave.shape != (9600,) or not np.isfinite(wave).all() or np.mean(wave**2) < 1e-16:
        raise ValueError("Finite non-silent 200ms window required")
    frequencies, power = welch(wave, fs=pilot.RATE, nperseg=2048, noverlap=1024)
    power /= power.sum()
    edges = np.geomspace(200, 16000, 33)
    return np.array(
        [
            10
            * np.log10(
                max(float(power[(frequencies >= a) & (frequencies < b)].mean()), 1e-12)
            )
            for a, b in itertools.pairwise(edges)
        ]
    )


def assess(source, baseline, output):
    import physical_sound_text_tags as tags

    if (output / "assessment.json").exists() or (output / "tag-inputs.json").exists():
        raise ValueError("Assessment exists; do not overwrite evidence")
    result = json.loads((output / "result.json").read_text())
    previous = json.loads((baseline / "result.json").read_text())
    if not result["status"].startswith("complete") or not previous["status"].startswith(
        "complete"
    ):
        raise ValueError("Terminal generation results required")
    if pilot.sha(source / "source.json") != result["source_sha256"]:
        raise ValueError("Changed TRAIN sources")
    source_report = json.loads((source / "source.json").read_text())
    manifest = {
        "status": "complete",
        "seconds": pilot.LENGTH / pilot.RATE,
        "controls": [],
        "cases": [],
        "rows": [],
    }
    assessment = {
        "scope": "TRAIN reference-aided spectral diagnostic, not quality acceptance or generalization",
        "rows": [],
        "comparisons": {},
    }
    references = {}
    for material in MATERIALS:
        frames, audition, order = [], [], []
        for row in source_report["selected"]:
            if row["material"] != material:
                continue
            path = source / f"{row['id']}-event.wav"
            if pilot.sha(path) != row["event_sha256"]:
                raise ValueError("Changed reference event")
            wave, rate = sf.read(path, dtype="float32")
            if rate != pilot.RATE:
                raise ValueError("Reference rate mismatch")
            frames.append(spectral_shape(wave[:9600]))
            pcm_path = output / f"reference-{row['id']}.wav"
            if np.abs(wave).max() * 0.5 >= 0.98:
                raise ValueError("Reference playback headroom failure")
            sf.write(pcm_path, wave * 0.5, rate, subtype="PCM_16")
            audition.extend([sf.read(pcm_path)[0], np.zeros(rate // 2)])
            order.append("reference-" + row["id"])
            index = len(manifest["cases"])
            manifest["cases"].append(
                {"id": order[-1], "diagnostic_id": material + "-wood"}
            )
            manifest["rows"].append(
                {
                    "case": index,
                    "wav": str(pcm_path),
                    "sha256": pilot.sha(pcm_path),
                    "seed": 42,
                }
            )
        references[material] = np.stack(frames)
        generated = [
            ("text", next(r for r in previous["rows"] if r["id"] == material)),
            ("prototype", next(r for r in result["rows"] if r["id"] == material)),
            (
                "individual",
                next(r for r in result["rows"] if r["id"] == material + "-individual"),
            ),
        ]
        for kind, row in generated:
            if row["status"] != "published":
                assessment["rows"].append(
                    {"material": material, "kind": kind, "status": "rejected"}
                )
                continue
            path = Path(row["wav"])
            if pilot.sha(path) != row["sha256"]:
                raise ValueError("Changed generated PCM")
            wave, rate = sf.read(path, dtype="float32")
            if rate != pilot.RATE or wave.shape != (pilot.LENGTH,):
                raise ValueError("Generated layout mismatch")
            features = np.stack(
                [
                    spectral_shape(wave[int(t * rate) : int(t * rate) + 9600])
                    for t in TIMES
                ]
            )
            # Each generated attack versus each of the two TRAIN reference attacks.
            distances = np.abs(features[:, None] - references[material][None]).mean(-1)
            assessment["rows"].append(
                {
                    "material": material,
                    "kind": kind,
                    "status": "measured",
                    "shape_distance_db": float(distances.mean()),
                    "per_event_reference_distances_db": distances.tolist(),
                    "full_pcm_rms": float(np.sqrt(np.mean(wave**2))),
                }
            )
            audition.extend([wave, np.zeros(rate // 2)])
            order.append(kind)
            index = len(manifest["cases"])
            manifest["cases"].append(
                {"id": material + "-" + kind, "diagnostic_id": material + "-wood"}
            )
            manifest["rows"].append(
                {"case": index, "wav": str(path), "sha256": row["sha256"], "seed": 42}
            )
        comparison = output / f"{material}-comparison.wav"
        sf.write(comparison, np.concatenate(audition), pilot.RATE, subtype="PCM_16")
        assessment["comparisons"][material] = {
            "wav": str(comparison),
            "sha256": pilot.sha(comparison),
            "order": order,
        }
    assessment["real_glass_wood_distance_db"] = float(
        np.abs(references["glass"][:, None] - references["wood"][None]).mean()
    )
    pilot.save(output / "assessment.json", assessment)
    pilot.save(output / "tag-inputs.json", manifest)
    for name, rms in (("raw", None), ("rms", 0.005)):
        tags.run(
            output / "tag-inputs.json", output / f"tags-{name}.json", [], ast_rms=rms
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("acquire", "render", "assess"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--archive", type=Path)
    parser.add_argument("--baseline", type=Path)
    parser.add_argument("--clap-package", type=Path)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        parser.error("Artifacts must remain outside Git")
    if args.mode == "acquire":
        if args.archive is None:
            parser.error("acquire requires --archive (author train_shard_1.tar)")
        acquire(args.output, args.archive)
    elif args.mode == "assess":
        if args.source is None or args.baseline is None:
            parser.error("assess requires --source and --baseline")
        assess(args.source, args.baseline, args.output)
    else:
        if any(p is None for p in (args.assets, args.source, args.clap_package)):
            parser.error("render requires --assets, --source and --clap-package")
        render(args.assets, args.source, args.clap_package, args.output)
