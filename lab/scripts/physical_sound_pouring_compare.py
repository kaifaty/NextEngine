"""Compare frozen pouring models across disclosed phases and noise seeds.

Local research only. References enter metrics/controls, never neural sampling.
No selection by scores, training, runtime promotion or calibrated acceptance.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_pouring_pilot as p
import soundfile as sf
import torch
from safetensors.torch import load_file


def phase_crop(row, phase):
    if phase not in ("first", "middle"):
        raise ValueError("unknown phase")
    start = (
        0 if phase == "first" else max(0, (row["spectrogram"].shape[1] - p.FRAMES) // 2)
    )
    spec, controls = p.crop(row, start)
    offset = start * p.HOP
    real = row["wave"][offset : offset + p.SAMPLES]
    real = np.pad(real, (0, p.SAMPLES - len(real)))
    return spec, controls, real, offset


def load_model(directory, device):
    meta_path, weights = directory / "model.json", directory / "model.safetensors"
    if meta_path.stat().st_size > 100000 or weights.stat().st_size > 10_000_000:
        raise ValueError("oversized model")
    meta = json.loads(meta_path.read_text())
    if (
        meta["format"] != "pour-flow-v1"
        or hashlib.sha256(weights.read_bytes()).hexdigest() != meta["checkpoint_sha256"]
    ):
        raise ValueError("wrong model identity")
    state = load_file(weights)
    if not all(torch.isfinite(v).all() for v in state.values()):
        raise ValueError("nonfinite weights")
    model = p.PourFlow()
    model.load_state_dict(state, strict=True)
    return model.to(device).eval(), meta


def select_rows(rows, training_controls):
    if not training_controls:
        return [r for r in rows if r["container_id"] in p.HELDOUT]
    # One source-order recording per training object, never chosen by its score.
    first = {}
    for row in rows:
        if row["container_id"] not in p.HELDOUT:
            first.setdefault(row["container_id"], row)
    return list(first.values())


def run(source, base, candidate, output, seeds, device="cuda", training_controls=False):
    source, base, candidate, output = (
        x.resolve() for x in (source, base, candidate, output)
    )
    if (
        not seeds
        or len(seeds) != len(set(seeds))
        or any(not 0 <= x < 2**32 for x in seeds)
    ):
        raise ValueError("need unique valid seeds")
    repo = Path(__file__).resolve().parents[2]
    if output.is_relative_to(repo) or output.exists():
        raise ValueError("use a new external output directory")
    torch.set_num_threads(4)
    rows, provenance = p.load_source(source)
    models = dict(
        zip(
            ("base", "power"), (load_model(base, device), load_model(candidate, device))
        )
    )
    train_ids = [r["item_id"] for r in rows if r["container_id"] not in p.HELDOUT]
    for _, meta in models.values():
        if (
            meta["train_ids"] != train_ids
            or meta["heldout_containers"] != list(p.HELDOUT)
            or meta["source_sha256"]
            != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
        ):
            raise ValueError("model/source/split mismatch")
    development = select_rows(rows, training_controls)
    output.mkdir(parents=True)
    report = {
        "status": "running",
        "scope": (
            "first source-order recording per training object; fit diagnostic, not generalization"
            if training_controls
            else "disclosed two-object development, not an independent test"
        ),
        "training_controls": training_controls,
        "seeds": seeds,
        "models": {k: m for k, (_, m) in models.items()},
        "terms": provenance["terms"],
        "reference_audio_input_to_generator": False,
        "rows": [],
    }
    tags = {
        "status": "complete",
        "seconds": p.SAMPLES / p.RATE,
        "cases": [],
        "rows": [],
        "controls": [],
    }
    previews = {s: [] for s in seeds}
    shown = set()

    def save(name, wave):
        if (
            len(wave) != p.SAMPLES
            or not np.isfinite(wave).all()
            or abs(wave).max() > 0.98
        ):
            raise ValueError("invalid waveform; do not silently clip or normalize")
        path = output / (name + ".wav")
        sf.write(path, wave, p.RATE, subtype="PCM_16")
        return {
            "wav": str(path),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "gain": 1,
        }

    for index, row in enumerate(development):
        for phase in ("first", "middle"):
            oracle, controls, real, offset = phase_crop(row, phase)
            common = {
                "item_id": row["item_id"],
                "container_id": row["container_id"],
                "phase": phase,
                "start_sample": offset,
                "controls": controls.tolist(),
            }
            pending = [
                ("real", None, real),
                ("oracle", seeds[0], p.decode(oracle, seeds[0])),
            ]
            for seed in seeds:
                for kind, (model, _) in models.items():
                    pending.append(
                        (kind, seed, p.decode(p.sample(model, controls, seed), seed))
                    )
            for kind, seed, wave in pending:
                key = f"{index:02d}-{phase}-{kind}-{seed}"
                record = {
                    **common,
                    "kind": kind,
                    "seed": seed,
                    **save(key, wave),
                    **p.metrics(wave, real),
                }
                report["rows"].append(record)
                tags["cases"].append({"id": key, "diagnostic_id": "water-pour"})
                tags["rows"].append({**record, "case": len(tags["cases"]) - 1})
            if (
                phase == "middle"
                and row["container_id"] not in shown
                and len(shown) < 2
            ):
                for seed in seeds:
                    for kind in ("real", "base", "power"):
                        wave = next(
                            w
                            for k, s, w in pending
                            if k == kind and (kind == "real" or s == seed)
                        )
                        previews[seed].extend([wave, np.zeros(p.RATE // 2)])
                shown.add(row["container_id"])
        print(
            json.dumps({"evaluated": index + 1, "total": len(development)}), flush=True
        )
    report["previews"] = []
    for seed, clips in previews.items():
        path = output / f"middle-comparison-seed{seed}.wav"
        sf.write(path, np.concatenate(clips), p.RATE, subtype="PCM_16")
        report["previews"].append(
            {
                "wav": str(path),
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "seed": seed,
                "gain": 1,
                "order": "first two source-order objects; real/base/power; middle phase",
            }
        )
    report["status"] = "complete"
    p.save_json(output / "result.json", report)
    p.save_json(output / "tag-input.json", tags)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "base", "candidate", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--seeds", type=int, nargs="+", default=[314, 2718, 1618])
    parser.add_argument("--device", choices=("cpu", "cuda"), default="cuda")
    parser.add_argument("--training-controls", action="store_true")
    args = parser.parse_args()
    run(
        args.source.resolve(),
        args.base.resolve(),
        args.candidate.resolve(),
        args.output.resolve(),
        args.seeds,
        args.device,
        args.training_controls,
    )
