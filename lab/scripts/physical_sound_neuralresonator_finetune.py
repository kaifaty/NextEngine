"""One shared last-layer fit, with shape/material-disjoint local development WAVs.

Synthetic 2D study only. Uses the published frozen shape encoder and numeric
inputs, not target audio at inference. No tuning on development results.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import numpy as np
import physical_sound_neuralresonator_pilot as pilot
import physical_sound_neuralresonator_reference as reference
import torch
from scipy.io import wavfile


def polygons() -> list[np.ndarray]:
    rng = np.random.default_rng(20260906)
    result = []
    for index in range(12):
        count = 5 + index % 4
        theta = np.arange(count) * 2 * np.pi / count + rng.uniform(0, 0.5)
        radius = rng.uniform(0.28, 0.40, 2)
        result.append(
            np.stack([radius[0] * np.cos(theta), radius[1] * np.sin(theta)], 1) + 0.5
        )
    return result


def differentiable_wave(coefficients: torch.Tensor) -> torch.Tensor:
    # Double-length periodic impulse response suppresses wraparound in the
    # declared full one-second output. Checked against the causal SOS renderer.
    b = torch.fft.rfft(coefficients[..., :3], n=2 * pilot.RATE)
    a = torch.fft.rfft(coefficients[..., 3:], n=2 * pilot.RATE)
    transfer = (b / a).prod(dim=-2).sum(dim=-2)
    return torch.fft.irfft(transfer, n=2 * pilot.RATE)[..., : pilot.RATE]


def losses(
    prediction: torch.Tensor, target: torch.Tensor
) -> tuple[torch.Tensor, torch.Tensor]:
    pred_spec, target_spec = (
        torch.fft.rfft(prediction).abs(),
        torch.fft.rfft(target).abs(),
    )
    spectral = (pred_spec - target_spec).abs().mean(-1) / target_spec.mean(
        -1
    ).clamp_min(1e-8)

    def envelope(wave):
        return (wave.reshape(*wave.shape[:-1], -1, 64).square().mean(-1) + 1e-12).sqrt()

    p, t = envelope(prediction), envelope(target)
    temporal = (p - t).abs().mean(-1) / t.mean(-1).clamp_min(1e-8)
    return spectral, temporal


def prepare(assets: Path, output: Path) -> dict:
    if output.exists():
        raise ValueError("preserve existing data")
    output.mkdir(parents=True)
    torch.set_num_threads(4)
    _, encoder, _, _ = pilot.load_models(assets)
    train_materials = [
        pilot.BASE,
        reference.MATERIALS["combined"],
        [10500.0, 4.2e10, 0.19, 3.0, 0.7e-6],
    ]
    dev_materials = [
        [4300.0, 3.4e10, 0.28, 4.0, 1.4e-6],
        [8900.0, 2.2e10, 0.22, 7.0, 0.9e-6],
    ]
    rows, inputs, targets = [], [], []
    masks = set()
    for shape_id, polygon in enumerate(polygons()):
        role = "train" if shape_id < 8 else "development"
        mask = reference.mask_for(polygon)
        mask_hash = hashlib.sha256(mask.numpy().tobytes()).hexdigest()
        if mask_hash in masks:
            raise ValueError("duplicate shape mask")
        masks.add(mask_hash)
        with torch.inference_mode():
            features = encoder(mask[None, None].repeat(1, 3, 1, 1))[0].numpy()
        for material in train_materials if role == "train" else dev_materials:
            modes = reference.solve_modes(polygon, material, 4)
            center = polygon.mean(0)
            for position, contact in enumerate((center, (center + polygon[0]) / 2)):
                row = {
                    "shape_id": shape_id,
                    "role": role,
                    "polygon": polygon.tolist(),
                    "material": material,
                    "contact": contact.tolist(),
                    "position": position,
                    "mask_sha256": mask_hash,
                }
                x = np.concatenate(
                    [features, contact, pilot.material_input(material).numpy()]
                ).astype(np.float32)
                y = reference.render_reference(modes, contact).astype(np.float32)
                inputs.append(x)
                targets.append(y)
                rows.append(row)
        print(f"prepared shape {shape_id} {role}", flush=True)
    np.savez(output / "data.npz", inputs=np.array(inputs), targets=np.array(targets))
    result = {
        "rows": rows,
        "train_count": sum(r["role"] == "train" for r in rows),
        "claim": "OWN_SYNTHETIC_TRAIN_AND_SHAPE_MATERIAL_DISJOINT_DEVELOPMENT_NOT_PRETRAINING_HOLDOUT",
    }
    (output / "rows.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def fit(assets: Path, data: Path, output: Path) -> dict:
    if output.exists():
        raise ValueError("preserve previous fit")
    output.mkdir(parents=True)
    torch.set_num_threads(4)
    torch.manual_seed(42)
    model, _, _, _ = pilot.load_models(assets)
    model.to("cuda")
    model.fc.network[-1].requires_grad_(True)
    rows = json.loads((data / "rows.json").read_text())["rows"]
    archive = np.load(data / "data.npz", allow_pickle=False)
    train = [i for i, r in enumerate(rows) if r["role"] == "train"]
    if {r["shape_id"] for r in rows if r["role"] == "train"} & {
        r["shape_id"] for r in rows if r["role"] == "development"
    }:
        raise ValueError("shape split overlap")
    x = torch.tensor(archive["inputs"][train], device="cuda")
    y = torch.tensor(archive["targets"][train], device="cuda")
    optimizer = torch.optim.Adam(
        [p for p in model.parameters() if p.requires_grad], lr=1e-5
    )
    history = []
    for step in range(100):
        indices = torch.randperm(len(x), device="cuda")[:4]
        optimizer.zero_grad(set_to_none=True)
        prediction = differentiable_wave(model(x[indices]))
        spectral, temporal = losses(prediction, y[indices])
        loss = (spectral + temporal).mean()
        if not torch.isfinite(loss):
            raise ValueError("nonfinite loss; do not resume with altered settings")
        loss.backward()
        norm = torch.nn.utils.clip_grad_norm_(
            [p for p in model.parameters() if p.requires_grad],
            1.0,
            error_if_nonfinite=True,
        )
        optimizer.step()
        row = {
            "step": step + 1,
            "spectral": float(spectral.mean().detach()),
            "temporal": float(temporal.mean().detach()),
            "gradient_norm": float(norm),
        }
        history.append(row)
        if step % 10 == 0:
            print(json.dumps(row), flush=True)
    checkpoint = output / "model.pt"
    torch.save({k: v.detach().cpu() for k, v in model.state_dict().items()}, checkpoint)
    result = {
        "steps": 100,
        "seed": 42,
        "lr": 1e-5,
        "batch_size": 4,
        "train_count": len(train),
        "trainable_parameters": sum(
            p.numel() for p in model.parameters() if p.requires_grad
        ),
        "history": history,
        "checkpoint_sha256": hashlib.sha256(checkpoint.read_bytes()).hexdigest(),
        "data_sha256": hashlib.sha256((data / "data.npz").read_bytes()).hexdigest(),
        "rows_sha256": hashlib.sha256((data / "rows.json").read_bytes()).hexdigest(),
        "claim": "ONE_FIXED_LAST_LAYER_FIT_NO_DEVELOPMENT_SELECTION",
    }
    (output / "fit.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def evaluate(assets: Path, data: Path, fitted: Path, output: Path) -> dict:
    if output.exists():
        raise ValueError("preserve previous evaluation")
    output.mkdir(parents=True)
    torch.set_num_threads(4)
    base, _, _, _ = pilot.load_models(assets)
    candidate, _, _, _ = pilot.load_models(assets)
    fit_record = json.loads((fitted / "fit.json").read_text())
    pilot.checked_bytes(fitted / "model.pt", fit_record["checkpoint_sha256"])
    pilot.checked_bytes(data / "data.npz", fit_record["data_sha256"])
    pilot.checked_bytes(data / "rows.json", fit_record["rows_sha256"])
    candidate.load_state_dict(
        torch.load(fitted / "model.pt", weights_only=True, map_location="cpu"),
        strict=True,
    )
    rows = json.loads((data / "rows.json").read_text())["rows"]
    archive = np.load(data / "data.npz", allow_pickle=False)
    records, all_waves = [], []
    for index, row in enumerate(rows):
        if row["role"] != "development":
            continue
        inputs = torch.tensor(archive["inputs"][index : index + 1])
        target = archive["targets"][index].astype(np.float64)
        waves = [target]
        metrics = {}
        name = f"case-{index:03d}"
        for label, model in [("base", base), ("candidate", candidate)]:
            with torch.inference_mode():
                ba = model(inputs)[0]
            wave, radius = pilot.render_coefficients(ba.numpy())
            with torch.no_grad():
                spectral, temporal = losses(
                    torch.tensor(wave[None]), torch.tensor(target[None])
                )
                approximation = differentiable_wave(ba[None]).numpy()[0]
            metrics[label] = {
                "spectral": float(spectral),
                "temporal": float(temporal),
                "max_pole_radius": radius,
                "fft_vs_sos_relative_rms": float(
                    np.linalg.norm(approximation - wave) / np.linalg.norm(wave)
                ),
                **pilot.waveform_observations(wave),
            }
            np.save(output / f"{name}-{label}-coefficients.npy", ba.numpy())
            waves.append(wave)
        records.append({"name": name, "input": row, "metrics": metrics})
        all_waves.append(waves)
        for label, wave in zip(["reference", "base", "candidate"], waves):
            np.save(output / f"{name}-{label}-raw.npy", wave)
    gain = 0.5 / max(float(abs(w).max()) for waves in all_waves for w in waves)
    for record, waves in zip(records, all_waves):
        pcm = [(w * gain).astype(np.float32) for w in waves]
        for label, w in zip(["reference", "base", "candidate"], pcm):
            wavfile.write(output / f"{record['name']}-{label}.wav", pilot.RATE, w)
        comparison = np.concatenate(
            [part for w in pcm for part in (w, np.zeros(pilot.RATE // 2, np.float32))]
        )
        wavfile.write(
            output / f"{record['name']}-comparison.wav", pilot.RATE, comparison
        )
    result = {
        "records": records,
        "shared_gain": gain,
        "checkpoint_sha256": fit_record["checkpoint_sha256"],
        "claim": "OPENED_SHAPE_MATERIAL_DISJOINT_DEVELOPMENT / NOT_REALISM_OR_FULL_GOAL_ADMISSION",
    }
    (output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(
        json.dumps(
            {
                label: {
                    metric: float(
                        np.mean([r["metrics"][label][metric] for r in records])
                    )
                    for metric in ["spectral", "temporal"]
                }
                for label in ["base", "candidate"]
            }
        ),
        flush=True,
    )
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("stage", choices=["prepare", "fit", "evaluate"])
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--data", type=Path)
    parser.add_argument("--fit", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.stage == "prepare":
        prepare(args.assets, args.output)
    elif args.stage == "fit":
        fit(args.assets, args.data, args.output)
    else:
        evaluate(args.assets, args.data, args.fit, args.output)
