"""Frozen magnitude candidate: TRAIN-only count/pole/field counterfactuals.

Five variants: compact target, unchanged standalone prediction, oracle count,
oracle count+poles, oracle count+field. Oracles are diagnostics, not new models.
All11TRAIN first contacts, one gain per object; no DEV, fit or selection by sound.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_magnitude as magnitude
import physical_sound_objectfolder2_shared as shared
import torch
from physical_sound_objectfolder2_expanded_train import ALL_TRAIN
from physical_sound_sonicgauss_waveform_fit import audio_metrics
from safetensors.torch import load_file
from scipy.io import wavfile

WEIGHTS = "ef22695e230b3846e9747f0da98d9a99a088f6f0d5a903a5a5ed2d16b5f1a400"


def oracle_count_query(model, cloud, features, contacts, count):
    """Explicit target-count path; NEVER used by standalone prediction."""
    if not isinstance(count, int) or not 1 <= count <= 4096:
        raise ValueError("bounded diagnostic mode count required")
    rank = ((torch.arange(count) + 0.5) / count)[:, None]
    with torch.no_grad():
        encoded = model.core.encode(
            torch.tensor(cloud[None]), torch.tensor(features[None])
        ).expand(count, -1)
        poles = model.core.mode_parameters(encoded, rank).exp().numpy().astype(float)
        fields = []
        for p in contacts:
            point = torch.tensor(p[None]).expand(count, -1)
            old = (
                model.core.signed_field(encoded, rank, point).sinh()
                * model.core.gain_scale
            )
            positive = (
                model.positive_field(encoded, rank, point).sinh()
                * model.core.gain_scale
            )
            fields.append((positive * old.sign()).numpy().T.astype(float))
    result = {
        "frequency": poles[:, 0],
        "damping": poles[:, 1],
        "gains": np.array(fields),
    }
    if not all(np.isfinite(v).all() for v in result.values()):
        raise ValueError("nonfinite diagnostic parameters")
    return result


def compose(target, predicted):
    if target["gains"].shape != predicted["gains"].shape:
        raise ValueError("aligned target count required for counterfactuals")
    return {
        "oracle_poles": {
            "frequency": target["frequency"],
            "damping": target["damping"],
            "gains": predicted["gains"],
        },
        "oracle_field": {
            "frequency": predicted["frequency"],
            "damping": predicted["damping"],
            "gains": target["gains"],
        },
    }


def run(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 11
        or {r["object_id"] for r in rows} != ALL_TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact eleven TRAIN objects required")
    if teacher.sha(args.fit / "model.safetensors") != WEIGHTS:
        raise ValueError("frozen magnitude weights required")
    model = magnitude.MagnitudeStudent().eval().requires_grad_(False)
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    args.output.mkdir(parents=True)
    results = []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("TRAIN target changed")
        with np.load(path, allow_pickle=False) as saved:
            data = dict(saved)
        geom = [data[k] for k in ("cloud", "features", "contacts")]
        raw = magnitude.predict(model, *geom)
        count = len(data["frequency"])
        queried = oracle_count_query(model, *geom, count)
        identity = len(raw["frequency"]) == count
        if identity and not all(np.array_equal(raw[k], queried[k]) for k in queried):
            raise ValueError("same-count identity control changed")
        parameters = {
            "reference": {k: data[k] for k in ("frequency", "damping", "gains")},
            "standalone": {k: raw[k] for k in queried},
            "oracle_count": queried,
            **compose(data, queried),
        }
        waves = {
            kind: teacher.waveform(
                p["gains"][0], p["frequency"], p["damping"], [1, 1, 1]
            )
            for kind, p in parameters.items()
        }
        metrics = {
            kind: audio_metrics(waves["reference"], w)
            for kind, w in waves.items()
            if kind != "reference"
        }
        gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
        stem = f"object-{row['object_id']}"
        for kind, p in parameters.items():
            np.savez(args.output / f"{stem}-{kind}.npz", **p)
            wavfile.write(
                args.output / f"{stem}-{kind}.wav",
                teacher.RATE,
                (waves[kind] * gain).astype(np.float32),
            )
        wavfile.write(
            args.output / f"{stem}-comparison.wav",
            teacher.RATE,
            (np.concatenate(list(waves.values())) * gain).astype(np.float32),
        )
        result = {
            "object_id": row["object_id"],
            "source_sha256": row["sha256"],
            "target_modes": count,
            "predicted_modes": len(raw["frequency"]),
            "same_count_exact_control": identity,
            "common_gain": gain,
            "order": list(parameters),
            "metrics": metrics,
        }
        results.append(result)
        print(json.dumps(result), flush=True)
    summary = {
        kind: {
            metric: float(np.mean([r["metrics"][kind][metric] for r in results]))
            for metric in ("spectrum", "envelope", "level")
        }
        for kind in results[0]["metrics"]
    }
    shared.write_json(
        args.output / "diagnostic.json",
        {
            "rows": results,
            "summary": summary,
            "weights_sha256": WEIGHTS,
            "script_sha256": teacher.sha(Path(__file__)),
            "scope": "all11TRAIN first contacts/compact target; count fixes rank queries too; poles means frequency AND damping; oracle-assisted controls NOT standalone generation/realism or new trained weights",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "fit", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    torch.set_num_threads(4)
    run(parser.parse_args())
