"""Frozen-network size self-similarity, not new fit or measured-sound validation.

Six TRAIN geometries at 0.8/1/1.25 size. Compare fresh network predictions with
analytic transport of its OWN baseline prediction. The reference is not FEM or
target audio. Only known modes below 10kHz undamped at baseline are evaluated,
avoiding claims about unseen above-Nyquist modes entering after enlargement.
Modal gains are held for transport: pitch/decay control, NOT amplitude physics.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_channels as channels
import physical_sound_objectfolder2_rayleigh as rayleigh
import physical_sound_objectfolder2_shared as shared
import torch
from safetensors.torch import load_file
from scipy.io import wavfile
from scipy.stats import wasserstein_distance

SCALES = (0.8, 1.0, 1.25)
CUTOFF = 10000.0


def eigenvalues(parameters):
    return (2 * np.pi * parameters["frequency"]) ** 2 + parameters["damping"] ** 2


def subset(parameters, keep):
    return {
        "frequency": parameters["frequency"][keep],
        "damping": parameters["damping"][keep],
        "gains": parameters["gains"][:, :, keep],
    }


def transport(parameters, scale, material):
    if not np.isfinite(scale) or scale <= 0:
        raise ValueError("positive finite size scale required")
    if scale == 1:
        return {k: parameters[k].copy() for k in ("frequency", "damping", "gains")}
    lam = eigenvalues(parameters) / scale**2
    alpha, beta = rayleigh.COEFFICIENTS[material]
    decay = (alpha + beta * lam) / 2
    radicand = lam - decay**2
    if not np.isfinite(lam).all() or np.any(radicand <= 0):
        raise ValueError("transport leaves oscillatory domain")
    frequency = np.sqrt(radicand) / (2 * np.pi)
    if np.any(frequency >= teacher.RATE / 2):
        raise ValueError("transport would alias; select declared common band first")
    return {
        "frequency": frequency,
        "damping": decay,
        "gains": parameters["gains"].copy(),
    }


def comparison(expected, actual):
    if not len(expected["frequency"]) or not len(actual["frequency"]):
        return {
            "log_frequency_w1": None,
            "mode_count_error": len(actual["frequency"]) - len(expected["frequency"]),
            "lowest_natural_frequency_relative_error": None,
        }
    left, right = [np.sqrt(eigenvalues(p)) / (2 * np.pi) for p in (expected, actual)]
    return {
        "log_frequency_w1": float(wasserstein_distance(np.log(left), np.log(right))),
        "mode_count_error": len(right) - len(left),
        "lowest_natural_frequency_relative_error": float(right.min() / left.min() - 1),
    }


def run(args):
    info = json.loads((args.fit / "fit.json").read_text())
    if teacher.sha(args.fit / "model.safetensors") != info["weights_sha256"]:
        raise ValueError("weights changed")
    model = channels.ChannelStudent()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    model.eval().requires_grad_(False)
    inputs = json.loads((args.data / "inputs.json").read_text())
    if teacher.sha(args.data / "inputs.npz") != inputs["sha256"]:
        raise ValueError("geometry inputs changed")
    if (
        len(inputs["rows"]) != 9
        or {r["object_id"] for r in inputs["rows"]} != shared.TRAIN | shared.DEV
    ):
        raise ValueError("fixed input cohort required")
    with np.load(args.data / "inputs.npz", allow_pickle=False) as d:
        if set(d.files) != {"cloud", "features", "contacts"}:
            raise ValueError("acoustic targets in inference inputs")
        clouds, features, contacts = [
            d[k].copy() for k in ("cloud", "features", "contacts")
        ]
    if (
        clouds.shape != (9, 512, 3)
        or features.shape != (9, 8)
        or contacts.shape != (9, 32, 3)
        or not all(np.isfinite(v).all() for v in (clouds, features, contacts))
    ):
        raise ValueError("invalid geometry shapes")
    args.output.mkdir(parents=True)
    results = []
    previews = {"neural": [], "transport": []}
    for i, row in enumerate(inputs["rows"]):
        identity = row["object_id"]
        if identity not in shared.TRAIN:
            continue
        material = shared.MATERIALS[int(features[i, 3:].argmax())]
        base = rayleigh.predict(model, clouds[i], features[i], contacts[i])
        base_band = subset(base, eigenvalues(base) <= (2 * np.pi * CUTOFF) ** 2)
        waves, conditions = {}, []
        for scale in SCALES:
            changed = features[i].copy()
            changed[:3] += np.float32(np.log(scale))
            predicted_full = rayleigh.predict(model, clouds[i], changed, contacts[i])
            limit = (2 * np.pi * CUTOFF / scale) ** 2
            predicted = subset(predicted_full, eigenvalues(predicted_full) <= limit)
            expected = transport(base_band, scale, material)
            unchanged = subset(base, eigenvalues(base) <= limit)
            key = f"object-{identity}-scale-{scale:g}"
            for kind, parameters in (("neural", predicted), ("transport", expected)):
                np.savez(args.output / f"{key}-{kind}.npz", **parameters)
                waves[f"{key}-{kind}"] = teacher.waveform(
                    parameters["gains"][0],
                    parameters["frequency"],
                    parameters["damping"],
                    [1, 1, 1],
                )
            match = comparison(expected, predicted)
            negative = comparison(expected, unchanged)
            beat = (
                scale != 1
                and match["log_frequency_w1"] is not None
                and negative["log_frequency_w1"] is not None
                and match["log_frequency_w1"] < negative["log_frequency_w1"]
                and abs(match["mode_count_error"]) <= abs(negative["mode_count_error"])
            )
            baseline_identity = (
                all(np.array_equal(predicted[k], expected[k]) for k in expected)
                if scale == 1
                else None
            )
            if scale == 1 and not baseline_identity:
                raise ValueError("identity-size control changed")
            condition = {
                "object_id": identity,
                "scale": scale,
                "material": material,
                "expected_common_modes": len(expected["frequency"]),
                "actual_common_modes": len(predicted["frequency"]),
                "network": match,
                "ignore_size_control": negative,
                "beats_ignore_size_control": bool(beat),
                "identity_control_exact": baseline_identity,
                "prefix": key,
            }
            conditions.append(condition)
            print(json.dumps(condition), flush=True)
        gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
        gallery = []
        for condition in conditions:
            key = condition["prefix"]
            for kind in ("transport", "neural"):
                name = f"{key}-{kind}.wav"
                pcm = (waves[f"{key}-{kind}"] * gain).astype(np.float32)
                wavfile.write(args.output / name, teacher.RATE, pcm)
                gallery.append(pcm)
            condition["common_gain"] = gain
        wavfile.write(
            args.output / f"object-{identity}-size-comparison.wav",
            teacher.RATE,
            np.concatenate(gallery),
        )
        if identity in (23, 29, 66):
            for kind, preview in previews.items():
                preview.extend(
                    [
                        (waves[f"object-{identity}-scale-{s:g}-{kind}"] * gain).astype(
                            np.float32
                        )
                        for s in SCALES
                    ]
                )
        results.extend(conditions)
    changed = [r for r in results if r["scale"] != 1]
    summary = {
        "conditions": len(changed),
        "beats_ignore_size_control": sum(
            r["beats_ignore_size_control"] for r in changed
        ),
        "exact_mode_count_matches": sum(
            r["network"]["mode_count_error"] == 0 for r in changed
        ),
        "mean_log_frequency_w1": float(
            np.mean(
                [
                    r["network"]["log_frequency_w1"]
                    for r in changed
                    if r["network"]["log_frequency_w1"] is not None
                ]
            )
        ),
        "mean_absolute_lowest_frequency_relative_error": float(
            np.mean(
                [
                    abs(r["network"]["lowest_natural_frequency_relative_error"])
                    for r in changed
                    if r["network"]["lowest_natural_frequency_relative_error"]
                    is not None
                ]
            )
        ),
    }
    for kind, preview in previews.items():
        label = "profile" if kind == "transport" else kind
        wavfile.write(
            args.output / f"steel-wood-ceramic-{label}-size-variants.wav",
            teacher.RATE,
            np.concatenate(preview),
        )
    shared.write_json(
        args.output / "size-probe.json",
        {
            "rows": results,
            "summary": summary,
            "weights_sha256": info["weights_sha256"],
            "input_sha256": inputs["sha256"],
            "script_sha256": teacher.sha(Path(__file__)),
            "scales": SCALES,
            "baseline_undamped_cutoff_hz": CUTOFF,
            "scope": "frozen six TRAIN geometry self-similarity; expected poles transport OWN baseline prediction, no target audio/FEM; held gain is pitch/decay control NOT amplitude scaling; no complete spectral-coverage or realism claim",
            "gallery_order": "per-object: transported own baseline then fresh NN at .8/1/1.25; combined neural/profile: fresh NN/transported NN baseline respectively, objects23/29/66, three scales each",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--fit", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    torch.set_num_threads(4)
    run(parser.parse_args())
