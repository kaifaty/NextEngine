"""Bounded structured-condition learning; no target audio at render time."""

import argparse
import collections
import csv
import hashlib
import io
import itertools
import json
import tarfile
import time
from pathlib import Path

import numpy as np
import physical_sound_syncfusion_condition as oracle
import physical_sound_syncfusion_pilot as pilot
import soundfile as sf
import torch
from torch.nn import functional as F

MATERIALS = ("glass", "wood", "metal")
MOTIONS = ("static", "rigid-motion")
AUDITIONS = [("glass", "static"), ("wood", "static"), ("glass", "rigid-motion")]


def features(material, motion):
    if material not in MATERIALS or motion not in MOTIONS:
        raise ValueError("Unsupported physical descriptor; no silent OOD fallback")
    value = torch.zeros(5)
    value[MATERIALS.index(material)] = 1
    value[3 + MOTIONS.index(motion)] = 1
    return value


class Adapter(torch.nn.Module):
    def __init__(self):
        super().__init__()
        self.net = torch.nn.Sequential(
            torch.nn.Linear(5, 32), torch.nn.SiLU(), torch.nn.Linear(32, 512)
        )

    def forward(self, x):
        return F.normalize(self.net(x), dim=-1)


def selection(archive):
    rows, members = [], {}
    for member in archive:
        if not member.isfile() or not member.name.startswith("train/"):
            raise ValueError("Only author TRAIN files allowed")
        if member.name.endswith(".resampled.wav"):
            members[member.name] = member
        if not member.name.endswith(".times.csv"):
            continue
        if member.size > 100000:
            raise ValueError("Oversized annotation")
        labels = list(
            csv.reader(io.StringIO(archive.extractfile(member).read().decode()))
        )
        counts = collections.Counter()
        for first, second in itertools.pairwise(labels):
            if not first or not second:
                continue
            start, end = float(first[0]), float(second[0])
            label = first[1].split()
            if (
                len(label) != 3
                or label[0] not in MATERIALS
                or label[1] != "hit"
                or label[2] not in MOTIONS
                or not 0.2 <= end - start <= 2
            ):
                continue
            pair = (label[0], label[2])
            if counts[pair] >= 2:
                continue
            counts[pair] += 1
            rows.append(
                {
                    "recording": member.name,
                    "material": pair[0],
                    "motion": pair[1],
                    "start": start,
                    "end": end,
                }
            )
    excluded = {
        r["recording"]
        for r in rows
        if (r["material"], r["motion"]) == ("glass", "rigid-motion")
    }
    for row in rows:
        recording = row["recording"]
        row["role"] = (
            "combination-dev"
            if recording in excluded
            else "recording-dev"
            if int(hashlib.sha256(recording.encode()).hexdigest()[:8], 16) % 5 == 0
            else "train"
        )
    return rows, members


def prepare(assets, archive_path, package, output):
    if (
        archive_path.stat().st_size != 2099322880
        or pilot.sha(archive_path)
        != "7284c9dd5eb5eb4caf81ee3b7acca71be6c11f4f6d91c19a85936a4ed634ece3"
    ):
        raise ValueError("Expected verified author TRAIN shard1")
    output.mkdir(parents=True, exist_ok=False)
    torch.set_num_threads(4)
    identity = json.loads((assets / "source.json").read_text())
    if pilot.sha(assets / "model.ckpt") != identity["checkpoint_sha256"]:
        raise ValueError("Changed generator checkpoint")
    state = torch.load(
        assets / "model.ckpt", weights_only=True, mmap=True, map_location="cpu"
    )["state_dict"]
    encoder, projection = oracle.audio_encoder(package, state)
    del state
    encoder.cuda()
    projection.cuda()
    encoded = []
    with tarfile.open(archive_path, "r:") as archive:
        rows, members = selection(archive)
        report = {
            "status": "encoding",
            "rows": rows,
            "archive_sha256": pilot.sha(archive_path),
            "checkpoint_sha256": identity["checkpoint_sha256"],
            "split": "all glass+rigid-motion recording keys excluded from fit; other SHA256(recording) first8hex mod5=0 development",
            "author_role": "TRAIN only; generator pretraining overlap known; adapter development not pristine object test",
        }
        pilot.save(output / "data.json", report)
        cached_key = None
        for i, row in enumerate(rows):
            key = row["recording"].removesuffix(".times.csv") + ".resampled.wav"
            if key != cached_key:
                member = members[key]
                if member.size > 100 * 1024**2:
                    raise ValueError("Oversized source")
                payload = archive.extractfile(member).read()
                wave, rate = sf.read(io.BytesIO(payload), dtype="float32")
                if rate != pilot.RATE or wave.ndim != 1 or not np.isfinite(wave).all():
                    raise ValueError("Invalid author source")
                cached_key = key
                source_sha = hashlib.sha256(payload).hexdigest()
            a, b = int(row["start"] * rate), int(row["end"] * rate)
            if not 0 <= a < b <= wave.size:
                raise ValueError("Invalid annotated interval")
            event = wave[a:b]
            row.update(samples=[a, b], source_sha256=source_sha)
            path = output / f"event-{i:03}.wav"
            sf.write(path, event, rate, subtype="FLOAT")
            row.update(wav=str(path), sha256=pilot.sha(path))
            with torch.inference_mode():
                v = encoder(
                    {"waveform": oracle.repeatpad(event)[None].cuda()}, device="cuda"
                )["embedding"]
                encoded.append(F.normalize(projection(v), dim=-1).cpu())
            if i % 25 == 0:
                print("encoded", i + 1, "/", len(rows), flush=True)
        target = torch.cat(encoded)
        if not torch.isfinite(target).all():
            raise ValueError("Nonfinite encoded target")
        torch.save(target, output / "embeddings.pt")
        report.update(
            status="complete",
            embeddings_sha256=pilot.sha(output / "embeddings.pt"),
            role_counts=dict(collections.Counter(r["role"] for r in rows)),
        )
        pilot.save(output / "data.json", report)


def prototype(rows, targets, material, motion):
    exact = [
        i
        for i, r in enumerate(rows)
        if r["role"] == "train" and (r["material"], r["motion"]) == (material, motion)
    ]
    indices = exact or [
        i
        for i, r in enumerate(rows)
        if r["role"] == "train" and r["material"] == material
    ]
    if not indices:
        raise ValueError("No TRAIN support for prototype")
    return F.normalize(
        targets[indices].mean(0), dim=-1
    ), "exact-pair" if exact else "material-only-fallback"


def fit(data, output):
    output.mkdir(parents=True, exist_ok=False)
    report = json.loads((data / "data.json").read_text())
    if (
        report["status"] != "complete"
        or pilot.sha(data / "embeddings.pt") != report["embeddings_sha256"]
    ):
        raise ValueError("Incomplete or changed encoded data")
    rows = report["rows"]
    targets = torch.load(data / "embeddings.pt", weights_only=True, map_location="cpu")
    if targets.shape != (len(rows), 512) or not torch.isfinite(targets).all():
        raise ValueError("Invalid target embeddings")
    x = torch.stack([features(r["material"], r["motion"]) for r in rows])
    training = torch.tensor([r["role"] == "train" for r in rows])
    train_keys = {r["recording"] for r in rows if r["role"] == "train"}
    dev_keys = {r["recording"] for r in rows if r["role"] != "train"}
    if train_keys & dev_keys or any(
        r["role"] == "train"
        and (r["material"], r["motion"]) == ("glass", "rigid-motion")
        for r in rows
    ):
        raise ValueError("Recording or combination leakage")
    groups = collections.Counter(
        (r["material"], r["motion"]) for r in rows if r["role"] == "train"
    )
    weights = torch.tensor(
        [1 / groups[(r["material"], r["motion"])] for r in rows if r["role"] == "train"]
    )
    weights /= weights.sum()
    torch.set_num_threads(4)
    torch.manual_seed(42)
    model = Adapter()
    optimizer = torch.optim.AdamW(model.parameters(), lr=0.003, weight_decay=0.0001)
    losses = []
    for _ in range(200):
        loss = ((1 - (model(x[training]) * targets[training]).sum(-1)) * weights).sum()
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        losses.append(float(loss.detach()))
    model.eval().requires_grad_(False)
    table, baseline = {}, {}
    for m, motion in itertools.product(MATERIALS, MOTIONS):
        value, kind = prototype(rows, targets, m, motion)
        table[m + "/" + motion] = value
        baseline[m + "/" + motion] = kind
    torch.save(
        {"state_dict": model.state_dict(), "prototypes": table}, output / "adapter.pt"
    )
    with torch.inference_mode():
        predicted = model(x)
    validation = []
    for role in ("train", "recording-dev", "combination-dev"):
        for m, motion in itertools.product(MATERIALS, MOTIONS):
            indices = [
                i
                for i, r in enumerate(rows)
                if r["role"] == role and (r["material"], r["motion"]) == (m, motion)
            ]
            if indices:
                truth = targets[indices]
                validation.append(
                    {
                        "role": role,
                        "material": m,
                        "motion": motion,
                        "n": len(indices),
                        "adapter_cosine_loss": float(
                            (1 - (predicted[indices] * truth).sum(-1)).mean()
                        ),
                        "prototype_cosine_loss": float(
                            (1 - (table[m + "/" + motion] * truth).sum(-1)).mean()
                        ),
                    }
                )
    pilot.save(
        output / "fit.json",
        {
            "status": "complete",
            "data_sha256": pilot.sha(data / "data.json"),
            "checkpoint_sha256": report["checkpoint_sha256"],
            "adapter_sha256": pilot.sha(output / "adapter.pt"),
            "steps": 200,
            "seed": 42,
            "loss_first_last": [losses[0], losses[-1]],
            "parameters": sum(p.numel() for p in model.parameters()),
            "prototypes": baseline,
            "validation": validation,
            "claim": "embedding fit diagnostic only; no acoustic quality or physical calibration admission",
        },
    )


def render(assets, fitted, output, auditions=None, kinds=None, times=None):
    """Standalone inference has no data cache, reference audio or audio encoder."""
    auditions = AUDITIONS if auditions is None else auditions
    kinds = ("text", "prototype", "adapter") if kinds is None else kinds
    times = oracle.TIMES if times is None else times
    pilot.event_track(times)
    for material, motion in auditions:
        features(material, motion)
    if not kinds or not set(kinds) <= {"text", "prototype", "adapter"}:
        raise ValueError("Unsupported inference kind")
    output.mkdir(parents=True, exist_ok=False)
    report = json.loads((fitted / "fit.json").read_text())
    if (
        report["status"] != "complete"
        or pilot.sha(fitted / "adapter.pt") != report["adapter_sha256"]
        or pilot.sha(assets / "model.ckpt") != report["checkpoint_sha256"]
    ):
        raise ValueError("Changed inference weights")
    torch.set_num_threads(4)
    torch.manual_seed(42)
    trained = torch.load(fitted / "adapter.pt", weights_only=True, map_location="cpu")
    adapter = Adapter().eval().requires_grad_(False)
    adapter.load_state_dict(trained["state_dict"], strict=True)
    state = torch.load(
        assets / "model.ckpt", weights_only=True, mmap=True, map_location="cpu"
    )["state_dict"]
    prompts = [
        f"A drumstick taps a {m} object that stays in place."
        if motion == "static"
        else f"A drumstick taps a {m} object, causing it to move."
        for m, motion in auditions
    ]
    text = pilot.text_conditions(state, prompts) if "text" in kinds else None
    model, encoder = pilot.build()
    model.load_state_dict(pilot.subset(state, "model."), strict=True)
    encoder.load_state_dict(pilot.subset(state, "onsets_encoder."), strict=True)
    del state
    model.cuda()
    encoder.cuda()
    result = {
        "status": "running",
        "reference_audio_input": False,
        "audio_encoder_instantiated": False,
        "text_encoder_instantiated": "text" in kinds,
        "adapter_sha256": report["adapter_sha256"],
        "checkpoint_sha256": report["checkpoint_sha256"],
        "events_seconds": times,
        "seed": 42,
        "steps": 150,
        "embedding_scale": 2,
        "playback_gain": 0.5,
        "rows": [],
    }
    pilot.save(output / "result.json", result)
    for i, (material, motion) in enumerate(auditions):
        with torch.inference_mode():
            learned = adapter(features(material, motion)[None])[:, None]
        conditions = {
            "text": text[i : i + 1] if text is not None else None,
            "prototype": trained["prototypes"][material + "/" + motion][None, None],
            "adapter": learned,
        }
        for kind, condition in conditions.items():
            if kind not in kinds:
                continue
            started = time.monotonic()
            with torch.inference_mode():
                _, context = encoder(pilot.event_track(times).cuda(), with_info=True)
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
            name = f"{material}-{motion}-{kind}"
            raw = output / f"{name}-raw.wav"
            sf.write(raw, wave, pilot.RATE, subtype="FLOAT")
            finite = bool(np.isfinite(wave).all())
            peak = float(np.abs(wave).max()) if finite else None
            row = {
                "id": name,
                "material": material,
                "motion": motion,
                "kind": kind,
                "prompt": prompts[i] if kind == "text" else None,
                "raw_sha256": pilot.sha(raw),
                "raw_peak": peak,
                "seconds": time.monotonic() - started,
            }
            if finite and peak * 0.5 < 0.98:
                path = output / f"{name}.wav"
                sf.write(path, wave * 0.5, pilot.RATE, subtype="PCM_16")
                row.update(
                    status="published",
                    wav=str(path),
                    sha256=pilot.sha(path),
                    timing=pilot.timing.match(
                        times, pilot.timing.onsets(wave * 0.5, pilot.RATE)
                    ),
                )
            else:
                row.update(status="rejected", reason="finite/headroom")
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


def fixed_reference_rows(current, reference):
    if current[: len(reference)] != reference:
        raise ValueError(
            "Fixed reference rows/roles changed or are not preserved prefix"
        )
    return reference


def assess(data, fitted, output, reference_data=None, previous_outputs=()):
    """Post-inference held-recording checks; no tuning and no claimed quality judge."""
    if (output / "assessment.json").exists():
        raise ValueError("Assessment exists")
    result = json.loads((output / "result.json").read_text())
    training = json.loads((fitted / "fit.json").read_text())
    if (
        not result["status"].startswith("complete")
        or pilot.sha(data / "data.json") != training["data_sha256"]
        or result["adapter_sha256"] != training["adapter_sha256"]
    ):
        raise ValueError("Changed or incomplete experiment")
    rows = json.loads((data / "data.json").read_text())["rows"]
    if reference_data is not None:
        reference = json.loads((reference_data / "data.json").read_text())
        if reference["status"] != "complete":
            raise ValueError("Incomplete fixed references")
        rows = fixed_reference_rows(rows, reference["rows"])
    previous_rows, previous_hashes = [], []
    for directory in previous_outputs:
        old = json.loads((directory / "result.json").read_text())
        if (
            old["status"] != "complete"
            or old["events_seconds"] != result["events_seconds"]
        ):
            raise ValueError("Incomplete or differently timed baseline")
        previous_hashes.append(pilot.sha(directory / "result.json"))
        previous_rows.extend(
            dict(r, kind="previous-adapter", id=r["id"] + "-previous")
            for r in old["rows"]
            if r["kind"] == "adapter"
        )
    report = {
        "scope": "adapter recording/combination development; generator pretraining overlap; no calibrated quality acceptance",
        "rows": [],
        "comparisons": [],
        "reference_data_sha256": pilot.sha((reference_data or data) / "data.json"),
        "previous_result_sha256": previous_hashes,
    }
    for material, motion in dict.fromkeys(
        (r["material"], r["motion"]) for r in result["rows"]
    ):
        selected = [
            r
            for r in rows
            if r["role"] != "train"
            and (r["material"], r["motion"]) == (material, motion)
        ]
        if not selected:
            raise ValueError("No held reference support")
        reference_features = []
        reference_wave = None
        for row in selected:
            path = Path(row["wav"])
            if pilot.sha(path) != row["sha256"]:
                raise ValueError("Changed development reference")
            wave, rate = sf.read(path, dtype="float32")
            if rate != pilot.RATE:
                raise ValueError("Reference sample rate")
            reference_features.append(oracle.spectral_shape(wave[:9600]))
            if reference_wave is None:
                reference_wave = wave
        reference_features = np.stack(reference_features)
        name = material + "-" + motion
        reference_path = output / f"{name}-reference.wav"
        if np.abs(reference_wave).max() * 0.5 >= 0.98:
            raise ValueError("Reference headroom")
        sf.write(reference_path, reference_wave * 0.5, pilot.RATE, subtype="PCM_16")
        waves = [sf.read(reference_path)[0], np.zeros(pilot.RATE // 2)]
        order = ["held-reference"]
        for row in previous_rows + result["rows"]:
            if (row["material"], row["motion"]) != (material, motion):
                continue
            if row["status"] != "published":
                report["rows"].append({"id": row["id"], "status": "rejected"})
                continue
            path = Path(row["wav"])
            if pilot.sha(path) != row["sha256"]:
                raise ValueError("Changed generated PCM")
            wave, rate = sf.read(path, dtype="float32")
            if rate != pilot.RATE or wave.shape != (pilot.LENGTH,):
                raise ValueError("Generated layout")
            attack_features = np.stack(
                [
                    oracle.spectral_shape(wave[int(t * rate) : int(t * rate) + 9600])
                    for t in result["events_seconds"]
                ]
            )
            distances = np.abs(
                attack_features[:, None] - reference_features[None]
            ).mean(-1)
            report["rows"].append(
                {
                    "id": row["id"],
                    "material": material,
                    "motion": motion,
                    "kind": row["kind"],
                    "status": "measured",
                    "shape_distance_db": float(distances.mean()),
                    "per_reference_distance_db": distances.mean(0).tolist(),
                    "reference_events": len(selected),
                    "reference_recordings": len({r["recording"] for r in selected}),
                    "timing": row["timing"],
                }
            )
            waves.extend([wave, np.zeros(rate // 2)])
            order.append(row["kind"])
        comparison = output / f"{name}-comparison.wav"
        sf.write(comparison, np.concatenate(waves), pilot.RATE, subtype="PCM_16")
        report["comparisons"].append(
            {
                "id": name,
                "wav": str(comparison),
                "sha256": pilot.sha(comparison),
                "order": order,
                "reference_row": selected[0],
            }
        )
    report["decision"] = (
        "Report-only; inspect acoustic comparison against prototype, not just embedding loss. No AST acceptance because previous real-reference controls failed."
    )
    pilot.save(output / "assessment.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("prepare", "fit", "render", "assess"))
    for name in (
        "assets",
        "archive",
        "clap-package",
        "data",
        "fitted",
        "reference-data",
    ):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--material", choices=MATERIALS)
    parser.add_argument("--motion", choices=MOTIONS)
    parser.add_argument("--kind", choices=("text", "prototype", "adapter"))
    parser.add_argument("--times", type=float, nargs="+")
    parser.add_argument("--previous-output", type=Path, nargs="+", default=[])
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        parser.error("Artifacts must remain external")
    required = {
        "prepare": (args.assets, args.archive, args.clap_package),
        "fit": (args.data,),
        "render": (args.assets, args.fitted),
        "assess": (args.data, args.fitted),
    }[args.mode]
    if any(p is None for p in required):
        parser.error("Missing mode-specific input")
    if args.mode == "render":
        if (args.material is None) != (args.motion is None):
            parser.error("--material and --motion must be supplied together")
        render(
            *required,
            args.output,
            auditions=[(args.material, args.motion)] if args.material else None,
            kinds=[args.kind] if args.kind else None,
            times=args.times,
        )
    elif args.mode == "assess":
        assess(
            *required,
            args.output,
            reference_data=args.reference_data,
            previous_outputs=args.previous_output,
        )
    else:
        {"prepare": prepare, "fit": fit}[args.mode](*required, args.output)
