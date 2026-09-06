"""TRAIN-only magnitude/sign/scalar discriminator and modal-basis control.

All waveform variants use target poles/count/ranks, not standalone generation.
No fit, no phase/sign search and no target correction admitted to the model.
The OF2 sign flip tests representation sensitivity, not an identified FEM gauge
of its unavailable source operator. Mechanical port invariance is a separate
known FEM control, not a way to interpret signed OF2 gains as self-admittance.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_shared as shared
import torch
from physical_sound_objectfolder2_expanded_train import ALL_TRAIN
from physical_sound_sonicgauss_waveform_fit import audio_metrics
from safetensors.torch import load_file
from scipy.io import wavfile

WEIGHTS = "81cc9a90a9da5c1a4cf5b2f82823fa267aaba20a7c75896fa9feb56fecad1fdb"


def port_residues(amplitudes):
    return np.einsum("pm,qm->mpq", amplitudes, amplitudes)


def variants(target, predicted):
    if (
        target.shape != predicted.shape
        or target.ndim != 3
        or not np.isfinite([target, predicted]).all()
    ):
        raise ValueError("matching finite point/axis/mode fields required")
    magnitude = np.sqrt(np.mean(predicted**2))
    scalar = np.sqrt(np.mean(target**2)) / magnitude if magnitude else None
    signs = np.where(np.arange(target.shape[-1]) % 2, -1.0, 1.0)
    result = {
        "reference": target,
        "learned": predicted,
        "oracle_sign": np.abs(predicted) * np.sign(target),
        "oracle_magnitude": np.abs(target) * np.sign(predicted),
        "reference_mode_flip": target * signs,
    }
    # A zero prediction remains a recorded failure, never repaired with target.
    if scalar is not None:
        result["oracle_scalar_rms"] = predicted * scalar
    return result, scalar


def matrix_control(args):
    rows = json.loads((args.ports / "train.json").read_text())["rows"]
    row = rows[0]
    if row["role"] != "train" or row["index"] != 0:
        raise ValueError("fixed first TRAIN FEM port control required")
    path = args.ports / row["file"]
    if teacher.sha(path) != row["sha256"]:
        raise ValueError("FEM port control changed")
    with np.load(path, allow_pickle=False) as d:
        a, omega = d["port_modes"].copy(), d["omega"].copy()
    signs = np.where(np.arange(len(omega)) % 2, -1.0, 1.0)
    residues = port_residues(a)
    changed = port_residues(a * signs)
    if not np.array_equal(residues, changed):
        raise ValueError("consistent modal-basis flip changed physical residues")
    # Audible scaling only: known cuboid reference E/rho/L, declared synthetic
    # 1% damping. This is mechanical displacement, NOT a microphone recording.
    omega = omega * np.sqrt(64e9 / 2230) / 0.18
    damping = 0.01 * omega
    wd = np.sqrt(omega**2 - damping**2)

    def wave(g):
        return teacher.waveform(
            np.vstack([g, np.zeros_like(g), np.zeros_like(g)]),
            wd / (2 * np.pi),
            damping,
            [1, 0, 0],
        )

    proxy, proxy_flipped = wave(a[0] / wd), wave(a[0] * signs / wd)
    physical = wave(a[0] * a[-1] / wd)
    physical_flipped = wave((a[0] * signs) * (a[-1] * signs) / wd)
    if not np.array_equal(physical, physical_flipped):
        raise ValueError("physical waveform changed under consistent basis flip")
    waves = [physical, physical_flipped, proxy, proxy_flipped]
    gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves))
    wavfile.write(
        args.output / "fem-physical-pair-modal-coordinate-pair.wav",
        teacher.RATE,
        (np.concatenate(waves) * gain).astype(np.float32),
    )
    np.savez(
        args.output / "fem-control.npz",
        port_modes=a,
        omega=omega,
        damping=damping,
        signs=signs,
    )
    return {
        "source_sha256": row["sha256"],
        "residues_exact": True,
        "physical_waveform_exact": True,
        "coordinate_sum_change": audio_metrics(proxy, proxy_flipped),
        "common_gain": gain,
        "scope": "known mass-normalized FEM control: a mode and its negative are identical physical coordinates only if both input/output factors transform; NOT proof of a gauge mismatch in original OF2 assets",
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
        raise ValueError("frozen eleven-body weights required")
    model = shared.SharedStudent()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    model.eval().requires_grad_(False)
    args.output.mkdir(parents=True)
    results = []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("TRAIN target changed")
        with np.load(path, allow_pickle=False) as d:
            data = dict(d)
        target = data["gains"].astype(float)
        count = target.shape[-1]
        rank = ((torch.arange(count) + 0.5) / count)[:, None]
        with torch.no_grad():
            encoded = model.encode(
                torch.tensor(data["cloud"][None]), torch.tensor(data["features"][None])
            ).expand(count, -1)
            predicted = np.array(
                [
                    (
                        model.signed_field(
                            encoded, rank, torch.tensor(p[None]).expand(count, -1)
                        ).sinh()
                        * model.gain_scale
                    )
                    .numpy()
                    .T
                    for p in data["contacts"]
                ]
            ).astype(float)
        fields, scalar = variants(target, predicted)
        waves = {
            kind: teacher.waveform(g[0], data["frequency"], data["damping"], [1, 1, 1])
            for kind, g in fields.items()
        }
        metrics = {
            kind: audio_metrics(waves["reference"], w)
            for kind, w in waves.items()
            if kind != "reference"
        }
        polarity = audio_metrics(waves["reference"], -waves["reference"])
        if any(v != 0 for v in polarity.values()):
            raise ValueError("metric global-polarity control failed")
        gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
        name = f"object-{row['object_id']}"
        np.savez(
            args.output / f"{name}.npz",
            frequency=data["frequency"],
            damping=data["damping"],
            **fields,
        )
        for kind, wave in waves.items():
            wavfile.write(
                args.output / f"{name}-{kind}.wav",
                teacher.RATE,
                (wave * gain).astype(np.float32),
            )
        wavfile.write(
            args.output / f"{name}-comparison.wav",
            teacher.RATE,
            (np.concatenate(list(waves.values())) * gain).astype(np.float32),
        )
        coefficient = {
            kind: float(
                np.mean(
                    (
                        np.arcsinh(g / float(model.gain_scale))
                        - np.arcsinh(target / float(model.gain_scale))
                    )
                    ** 2
                )
            )
            for kind, g in fields.items()
            if kind != "reference"
        }
        result = {
            "object_id": row["object_id"],
            "source_sha256": row["sha256"],
            "modes": count,
            "variants": list(waves),
            "common_gain": gain,
            "all32contacts_scalar_rms": scalar,
            "all32contacts_asinh_mse": coefficient,
            "first_contact_audio": metrics,
            "global_polarity_metrics": polarity,
        }
        results.append(result)
        print(json.dumps(result), flush=True)
    summary = {
        kind: {
            metric: float(
                np.mean([r["first_contact_audio"][kind][metric] for r in results])
            )
            for metric in ("spectrum", "envelope", "level")
        }
        for kind in results[0]["first_contact_audio"]
    }
    control = matrix_control(args)
    shared.write_json(
        args.output / "field-cause.json",
        {
            "rows": results,
            "summary": summary,
            "fem_control": control,
            "weights_sha256": WEIGHTS,
            "script_sha256": teacher.sha(Path(__file__)),
            "scope": "eleven TRAIN compact-target first-contact audio/all32contact coefficients; every variant oracle-pole/count/rank assisted; sign/magnitude/scalar oracles NOT deployable predictions; no per-object tuning or new fit",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "fit", "ports", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    torch.set_num_threads(4)
    run(parser.parse_args())
