"""Signed-level and local acoustic-gradient discriminator; no fitting/PCGrad.

Existing disclosed data/weights only. Eguchi et al./CC BY4; Powered by Stability
AI/TangoFlux. Local gradients cannot establish historical optimization causality.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_texture_acoustic as acoustic
import physical_sound_texture_dc_audit as audit
import torch

surface, flow, event = acoustic.surface, acoustic.flow, acoustic.event


def cosine(a, b):
    a, b = np.asarray(a, np.float64), np.asarray(b, np.float64)
    if (
        a.ndim != 1
        or a.shape != b.shape
        or not np.isfinite(a).all()
        or not np.isfinite(b).all()
    ):
        raise ValueError("matching finite vectors required")
    norm = np.linalg.norm(a) * np.linalg.norm(b)
    return None if norm == 0 else float(np.clip(np.dot(a, b) / norm, -1, 1))


def gain_projection(candidate, reference):
    if (
        candidate.shape != reference.shape
        or candidate.ndim != 1
        or not np.isfinite(candidate).all()
        or not np.isfinite(reference).all()
    ):
        raise ValueError("matching finite signals required")
    power = float(np.dot(reference, reference))
    if power <= 1e-18:
        raise ValueError("non-silent reference required")
    gain = float(np.dot(candidate, reference) / power)
    residual = candidate - gain * reference
    return {
        "least_squares_gain_to_fm": gain,
        "gain_fit_residual_power_fraction": float(
            np.dot(residual, residual) / max(float(np.dot(candidate, candidate)), 1e-18)
        ),
    }


def central(wave, row):
    start = round(row["crop_start_seconds"] * 44100)
    crop = wave[start : start + 33075]
    if len(crop) != 33075:
        raise ValueError("complete source-defined750ms interval required")
    return event.resample_poly(crop.mean(1), 1, 2)


def cached(root, output, rows, report):
    endpoint, endpoint_root, endpoint_sha = audit.read_report(
        root, "texture-acoustic-endpoint-2026-09-05"
    )
    sampled, sampled_root, sampled_sha = audit.read_report(
        root, "texture-acoustic-full-sampler-2026-09-05"
    )
    if (
        endpoint["source_sha256"] != report["source_sha256"]
        or sampled["source_sha256"] != report["source_sha256"]
    ):
        raise ValueError("source lineage mismatch")
    report["parent_reports"] = {"endpoint": endpoint_sha, "sampled": sampled_sha}
    metadata = {r["id"]: r for r in rows}
    references = {r["id"]: r for r in sampled["references"]}
    entries = {
        (r["id"], r["seed"], r["variant"]): (r, sampled_root) for r in sampled["rows"]
    }
    for r in endpoint["rows"]:
        key = (r["id"], r["seed"], r["variant"])
        if r["variant"] == "acoustic":
            entries[key] = (r, endpoint_root)
        elif entries[key][0]["sha256"] != r["sha256"]:
            raise ValueError("paired controls changed")
    montage = []
    for ident, seed in sorted({(r[0], r[1]) for r in entries}):
        row = metadata[ident]
        real = audit.hybrid.checked_wave(references[ident], sampled_root)
        target = central(real, row)
        waves = {
            arm: audit.hybrid.checked_wave(*entries[(ident, seed, arm)])
            for arm in ("base", "fm_only", "acoustic", "sampled")
        }
        fm = central(waves["fm_only"], row)
        fm_level = flow.codec.level(fm)
        for arm, wave in waves.items():
            x = central(wave, row)
            signed = flow.codec.level(x) - flow.codec.level(target)
            stored = entries[(ident, seed, arm)][0]
            if not np.isclose(
                abs(signed), stored["moving_level_absolute_error_db"], atol=1e-8
            ):
                raise ValueError("signed/absolute metric mismatch")
            report["signed_rows"].append(
                {
                    "id": ident,
                    "texture_id": row["texture_id"],
                    "seed": seed,
                    "variant": arm,
                    "signed_level_error_db": signed,
                    "level_change_from_fm_db": flow.codec.level(x) - fm_level,
                    "shape_change_from_fm_db": float(
                        np.std(
                            flow.spectrum.spectrum(x)[1:]
                            - flow.spectrum.spectrum(fm)[1:]
                        )
                    ),
                    **gain_projection(x, fm),
                }
            )
        if (
            row["texture_id"] in surface.HELD
            and row["commanded_speed_mm_s"] == 40
            and row["commanded_normal_force_N"] == 0.5
            and seed == 314
        ):
            group = []
            for wave in (real, waves["fm_only"], waves["acoustic"], waves["sampled"]):
                group.extend((wave, np.zeros((11025, 2))))
            entry, _ = flow.codec.publish(
                output / f"surface-{row['texture_id']}-comparison.wav",
                np.concatenate(group),
            )
            report["previews"].append(entry)
            montage.extend(group)
    if len(report["signed_rows"]) != 288:
        raise ValueError("exact72 paired cases/four arms required")
    report["comparison"], _ = flow.codec.publish(
        output / "comparison.wav", np.concatenate(montage)
    )
    report["comparison_order"] = (
        "oak/steel/frosted glass40mm/s,.5N,seed314: real/FM-only/endpoint/full-sampler; no gain matching"
    )
    flow.save(output / "result.json", report)


def gradients(root, source, rows, manifest, output, report):
    directory = root / "texture-acoustic-endpoint-2026-09-05"
    parent = json.loads((directory / "model.json").read_text())["parent"]
    models, _ = acoustic.load_candidates(directory, parent)
    model = models["fm_only"]
    vae, _ = flow.load_codec("cuda")
    table = surface.metadata(source)
    originals = {r["id"]: r for r in manifest["rows"]}
    selected = [
        r for r in rows if r["role"] == "train" and r["commanded_speed_mm_s"] == 30
    ]
    if len(selected) != 12 or {r["texture_id"] for r in selected} != set(surface.TRAIN):
        raise ValueError("exact six-surface/two-force TRAIN30mm/s probe required")
    vectors = []
    parameters = list(model.parameters())
    for row in selected:
        original = originals[row["id"]]
        position, force = [
            np.genfromtxt(
                flow.spectrum.checked(original[k], manifest, source.parent),
                delimiter=",",
                names=True,
            )
            for k in ("position", "force")
        ]
        real, rate = event.sf.read(
            flow.spectrum.checked(original["audio"], manifest, source.parent),
            always_2d=True,
            dtype="float32",
        )
        if rate != 44100 or real.shape[1] != 1:
            raise ValueError("original mono44100 expected")
        real = np.repeat(real * flow.GAIN, 2, axis=1)
        count = int(np.ceil(len(real) / flow.HOP))
        times = (np.arange(count) + 0.5) * flow.HOP / 44100
        physical = surface.builder(table)(
            row["texture_id"],
            event.position_speed(position, times),
            np.interp(times, force["time"], force["force"]),
        )
        latent = acoustic.sampled_latent(model, physical, 607)
        full = vae.decode(latent).sample
        first = (len(real) // flow.HOP - 32) // 2
        start = first * flow.HOP
        wave = full[:, :, start : start + 32 * flow.HOP]
        truth = torch.tensor(real[start : start + 32 * flow.HOP].T[None], device="cuda")
        parts = acoustic.acoustic_parts(wave, truth)
        core = acoustic.shared_band(wave)[:, 8192:-8192]
        gain_axis = core.square().mean().clamp_min(1e-12).log()
        gradients = []
        for j, value in enumerate((parts["spectrum"], parts["envelope"], gain_axis)):
            gg = torch.autograd.grad(value, parameters, retain_graph=j < 2)
            vector = torch.cat([g.flatten() for g in gg]).detach().cpu().numpy()
            if not np.isfinite(vector).all() or np.linalg.norm(vector) == 0:
                raise ValueError("invalid local gradient")
            gradients.append(vector)
        gs, ge, gl = gradients
        vectors.append(np.stack(gradients))
        report["gradient_rows"].append(
            {
                "id": row["id"],
                "texture_id": row["texture_id"],
                "category": table[row["texture_id"]]["category"],
                "force_N": row["commanded_normal_force_N"],
                "seed": 607,
                "first_frame": first,
                "frames": count,
                "spectrum_loss": float(parts["spectrum"].detach()),
                "envelope_loss": float(parts["envelope"].detach()),
                "spectrum_envelope_cosine": cosine(gs, ge),
                "spectrum_gain_axis_cosine": cosine(gs, gl),
                "envelope_gain_axis_cosine": cosine(ge, gl),
                "norms": [float(np.linalg.norm(g)) for g in gradients],
            }
        )
        if (
            row["texture_id"] in surface.source.TEXTURES
            and row["commanded_normal_force_N"] == 0.5
        ):
            entry, _ = event.publish(
                output,
                f"train-{row['id']}-fm-only",
                full.detach()[0].T.cpu().numpy(),
                len(real),
            )
            report["training_previews"].append(
                {"id": row["id"], "reference_audio_input": False, **entry}
            )
        print(json.dumps(report["gradient_rows"][-1]), flush=True)
        flow.save(output / "result.json", report)
        del latent, full, wave, truth, parts, core, gain_axis, gg
    vectors = np.stack(vectors)
    np.savez(output / "local-gradients.npz", gradients=vectors)
    report["gradient_archive_sha256"] = flow.codec.sha(output / "local-gradients.npz")
    report["gradient_order"] = ["spectrum", "envelope", "log_moving_power"]
    report["category_cosines"] = []
    groups = {
        c: np.mean(
            [
                vectors[i, 0] + vectors[i, 1]
                for i, r in enumerate(report["gradient_rows"])
                if r["category"] == c
            ],
            axis=0,
        )
        for c in surface.CATEGORIES
    }
    for a in surface.CATEGORIES:
        for b in surface.CATEGORIES:
            report["category_cosines"].append(
                {"a": a, "b": b, "cosine": cosine(groups[a], groups[b])}
            )


def run(root, output):
    torch.set_num_threads(4)
    root = root.resolve()
    source = root / "cluster-surface-transfer-grid-2026-09-05" / "result.json"
    rows, _, _, _, manifest = surface.load_data(source)
    output = flow.fresh(output)
    report = {
        "status": "running",
        "scope": "posthoc signed-level and local TRAIN-gradient discriminator; no fitting,correction or quality admission",
        "source_sha256": flow.codec.sha(source),
        "new_training_steps": 0,
        "signed_rows": [],
        "gradient_rows": [],
        "previews": [],
        "training_previews": [],
    }
    flow.save(output / "result.json", report)
    try:
        cached(root, output, rows, report)
        gradients(root, source, rows, manifest, output, report)
        report["status"] = "complete"
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lab-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.lab_root, args.output)
