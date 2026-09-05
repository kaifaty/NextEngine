"""Training-length flow-field countercheck; shared full latent, no audio stitching.

No new weights. GroupNorm sees32-frame windows as during fitting. Full Oobleck
decode is unchanged. Eguchi et al./CC BY4; Powered by Stability AI/TangoFlux.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_texture_dc_audit as audit
import torch

hybrid, surface, flow = audit.hybrid, audit.surface, audit.flow
event = surface.event
GUARD = 10  # Radius of input conv + eight block convs + output conv, all kernel3.


def windows(frames):
    if type(frames) is not int or not 32 <= frames <= 256:
        raise ValueError("bounded full latent length required")
    starts = sorted(set(range(0, frames - 31, 32 - 2 * GUARD)) | {frames - 32})
    return [
        (first, 0 if first == 0 else GUARD, 32 if first + 32 == frames else 32 - GUARD)
        for first in starts
    ]


class WindowedField(torch.nn.Module):
    def __init__(self, model):
        super().__init__()
        self.model = model

    @property
    def center(self):
        return self.model.center

    @property
    def scale(self):
        return self.model.scale

    def forward(self, x, t, physical):
        if (
            physical.ndim != 3
            or physical.shape[0] != len(x)
            or physical.shape[-1] != x.shape[-1]
        ):
            raise ValueError("full time-varying condition sequence required")
        layout = windows(x.shape[-1])
        if len(layout) == 1:
            return self.model(x, t, physical)
        xx = torch.cat([x[:, :, first : first + 32] for first, _, _ in layout])
        pp = torch.cat([physical[:, :, first : first + 32] for first, _, _ in layout])
        fields = self.model(xx, t.repeat(len(layout)), pp).reshape(
            len(layout), *x.shape[:2], 32
        )
        result = torch.zeros_like(x)
        counts = torch.zeros_like(x[:, :1])
        for values, (first, low, high) in zip(fields, layout, strict=True):
            result[:, :, first + low : first + high] += values[:, :, low:high]
            counts[:, :, first + low : first + high] += 1
        if (counts == 0).any():
            raise ValueError("uncovered latent frame")
        return result / counts


def field_diagnostic(model):
    device = next(model.parameters()).device
    rng = torch.Generator(device=device).manual_seed(23)
    x = torch.randn((1, 64, 96), generator=rng, device=device)
    physical = torch.tensor(
        surface.features("Glass", 0.4, 0.38, np.full(96, 40), np.full(96, 0.5))[None],
        device=device,
    )
    altered = x.clone()
    altered[:, :, :16] *= 4
    t = torch.tensor([0.5], device=device)
    with torch.inference_mode():
        global_a, global_b = model(x, t, physical), model(altered, t, physical)
        windowed = WindowedField(model)
        local_a, local_b = windowed(x, t, physical), windowed(altered, t, physical)
    # Remote frames60..75 are outside the21-frame convolutional receptive field.
    return {
        "global_remote_change_rms": float(
            (global_a[:, :, 60:76] - global_b[:, :, 60:76]).square().mean().sqrt()
        ),
        "windowed_remote_change_rms": float(
            (local_a[:, :, 60:76] - local_b[:, :, 60:76]).square().mean().sqrt()
        ),
        "scope": "dependency discriminator, not proof of synthesis-quality causation",
    }


def run(root, output):
    torch.set_num_threads(4)
    root = root.resolve()
    source = root / "cluster-surface-transfer-grid-2026-09-05" / "result.json"
    prior_path = root / "texture-surface-transfer-2026-09-05"
    models, meta = surface.load_models(prior_path, "cuda")
    if meta["source_sha256"] != flow.codec.sha(source):
        raise ValueError("source/model lineage mismatch")
    base = models["descriptor"]
    windowed = WindowedField(base).eval()
    vae, _ = flow.load_codec("cuda")
    output = flow.fresh(output)
    report = {
        "status": "running",
        "model": meta,
        "new_training_steps": 0,
        "scope": "posthoc train-window/inference-length countercheck",
        "field_diagnostic": field_diagnostic(base),
        "rows": [],
        "references": [],
    }
    flow.save(output / "result.json", report)
    try:
        # Primary source-free artifact before source loading/evaluation.
        physical, request = event.profile()
        physical = surface.features(
            "Glass",
            0.3970170073501798,
            0.3827100881663895,
            physical[3] * 20 + 40,
            physical[4] * 0.25 + 0.75,
        )
        requested = []
        for name, model in (("global", base), ("windowed", windowed)):
            wave = surface.generate(model, vae, physical, 314)
            entry, pcm = event.publish(
                output,
                "requested-glass-" + name,
                wave,
                round(request["duration_seconds"] * 44100),
            )
            report["requested_" + name] = {
                **entry,
                "request": {k: v for k, v in request.items() if k != "texture"},
                "category": "Glass",
                "static_friction": 0.3970170073501798,
                "dynamic_friction": 0.3827100881663895,
                "seed": 314,
                "reference_audio_input": False,
                "sensor_input": False,
            }
            requested.extend((pcm, np.zeros((11025, 2))))
        report["requested_comparison"], _ = flow.codec.publish(
            output / "requested-comparison.wav", np.concatenate(requested)
        )
        flow.save(output / "result.json", report)
        rows, _, _, _, manifest = surface.load_data(source)
        table = surface.metadata(source)
        originals = {r["id"]: r for r in manifest["rows"]}
        previews = []
        for row in rows:
            # Both duration extremes; repeat0 new surfaces,repeat1 old anchors.
            if row["commanded_speed_mm_s"] not in (20, 60):
                continue
            selected = (row["texture_id"] in surface.HELD and row["repeat"] == 0) or (
                row["texture_id"] in surface.source.TEXTURES and row["repeat"] == 1
            )
            if not selected:
                continue
            entry = originals[row["id"]]
            position, force = [
                np.genfromtxt(
                    flow.spectrum.checked(entry[k], manifest, source.parent),
                    delimiter=",",
                    names=True,
                )
                for k in ("position", "force")
            ]
            path = flow.spectrum.checked(entry["audio"], manifest, source.parent)
            real_wave, rate = event.sf.read(path, always_2d=True)
            if rate != 44100:
                raise ValueError("unexpected rate")
            real_wave = np.repeat(real_wave * 50, 2, axis=1)
            real_entry, real = flow.codec.publish(
                output / f"{row['id']}-real.wav", real_wave
            )
            report["references"].append({"id": row["id"], **real_entry})
            count = int(np.ceil(len(real) / event.HOP))
            times = (np.arange(count) + 0.5) * event.HOP / 44100
            physical = surface.builder(table)(
                row["texture_id"],
                event.position_speed(position, times),
                np.interp(times, force["time"], force["force"]),
            )
            for seed in (314, 2718):
                pcms = {}
                for name, model in (("global", base), ("windowed", windowed)):
                    wave = surface.generate(model, vae, physical, seed)
                    receipt, pcm = event.publish(
                        output, f"{row['id']}-{name}-{seed}", wave, len(real)
                    )
                    pcms[name] = pcm
                    mono = event.resample_poly(pcm.mean(1), 1, 2)
                    report["rows"].append(
                        {
                            "id": row["id"],
                            "texture_id": row["texture_id"],
                            "role": row["role"],
                            "seed": seed,
                            "variant": name,
                            "latent_frames": count,
                            **receipt,
                            **surface.metrics(
                                pcm, real, {"row": row, "physical": physical}
                            ),
                            **audit.compare(
                                mono, event.resample_poly(real.mean(1), 1, 2)
                            ),
                        }
                    )
                if (
                    row["texture_id"] == 76
                    and row["commanded_normal_force_N"] == 0.5
                    and seed == 314
                ):
                    for pcm in (real, pcms["global"], pcms["windowed"]):
                        previews.extend((pcm, np.zeros((11025, 2))))
            print(json.dumps({"evaluated": row["id"], "frames": count}), flush=True)
            flow.save(output / "result.json", report)
        if len(report["rows"]) != 96:
            raise ValueError("exact24-record/two-seed/two-variant diagnostic required")
        report["comparison"], _ = flow.codec.publish(
            output / "comparison.wav", np.concatenate(previews)
        )
        report["comparison_order"] = (
            "Frosted glass20/60mm/s,.5N,repeat0,seed314: real/global/windowed"
        )
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
