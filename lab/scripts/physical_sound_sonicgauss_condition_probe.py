"""Localize 3D conditioning loss with frozen weights and audible counterfactuals.

These interventions test dependency, not physical accuracy. Contracting Gaussian
centers leaves ellipsoid scales/orientations unchanged; removing SH appearance
is not a material conversion. Neither variant is a calibrated new real object.
"""

from __future__ import annotations

import argparse
import copy
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_pilot as pilot
import torch
from scipy.io import wavfile
from torch import nn

CONTACTS = [
    [0.9306724345821787, -0.918678529191214, 0.15392957406368474],
    [-0.6213642203178644, -0.35045376636680214, 0.10162437371200583],
]


def compare(first, second):
    a = np.asarray(first, dtype=np.float64)
    b = np.asarray(second, dtype=np.float64)
    if a.shape != b.shape or not np.isfinite(a).all() or not np.isfinite(b).all():
        raise ValueError("expected same-shaped finite tensors")
    norm = np.linalg.norm(a)
    if norm == 0:
        raise ValueError("zero reference norm")
    bn = np.linalg.norm(b)
    return {
        "exact": bool(np.array_equal(a, b)),
        "relative_rms": float(np.linalg.norm(a - b) / norm),
        "cosine": float(np.vdot(a, b) / (norm * bn)) if bn else None,
        "norm_ratio": float(bn / norm),
        "max_abs": float(np.abs(a - b).max()),
    }


def counterfactual(normalized, contact, intervention):
    result = {key: value.clone() for key, value in normalized.items()}
    if intervention == "centers-contract-x":
        # Anchor at the original contact, holding the position-encoder input fixed.
        result["means"][:, 0] = contact[0] + 0.6 * (result["means"][:, 0] - contact[0])
    elif intervention == "neutral-appearance":
        result["features_dc"].zero_()
        result["features_rest"].zero_()
    else:
        raise ValueError("unknown intervention")
    return result


class CaptureGaussian(nn.Module):
    def __init__(self, encoder):
        super().__init__()
        self.encoder = encoder
        self.features = None
        self.rng = None

    def forward(self, normalized, device):
        value = self.encoder(normalized, device)
        self.features = value.detach().clone()
        # Replay the RNG state immediately AFTER encoding; do not inadvertently
        # change the diffusion noise when skipping encoder work.
        self.rng = (torch.get_rng_state(), torch.cuda.get_rng_state())
        return value


class CachedGaussian(nn.Module):
    def __init__(self, features, rng):
        super().__init__()
        self.register_buffer("features", features.detach().clone())
        self.rng = rng

    def forward(self, normalized, device):
        torch.set_rng_state(self.rng[0])
        torch.cuda.set_rng_state(self.rng[1])
        return self.features


class FusionProbe(nn.Module):
    def __init__(self, fusion, intervention=None):
        super().__init__()
        self.fusion = fusion
        self.intervention = intervention
        self.captured = {}

    def forward(self, gaussian, position):
        if self.intervention == "zero-position":
            position = torch.zeros_like(position)
        fused = self.fusion(gaussian, position)
        if self.intervention == "zero-conditioning":
            fused = torch.zeros_like(fused)
        self.captured = {
            "gaussian": gaussian.detach().cpu().numpy(),
            "position": position.detach().cpu().numpy(),
            "fused": fused.detach().cpu().numpy(),
        }
        return fused


def render(ns, models, gaussian, fusion, normalized, contact):
    model, _, position, _, vae, _ = models
    first_input = {}

    def hook(module, args, kwargs):
        if not first_input:
            for name in [
                "hidden_states",
                "encoder_hidden_states",
                "pooled_projections",
            ]:
                first_input[name] = kwargs[name].detach().cpu().numpy().copy()

    handle = model.transformer.register_forward_pre_hook(hook, with_kwargs=True)
    try:
        with torch.no_grad():
            latent = ns["inference_with_stage3"](
                model,
                gaussian,
                position,
                fusion,
                normalized,
                contact,
                "cuda",
                duration=3.0,
                num_inference_steps=50,
                guidance_scale=-1,
                seed=0,
            )
            wave = vae.decode(latent.transpose(2, 1)).sample[0].cpu().numpy()
    finally:
        handle.remove()
    return {
        **fusion.captured,
        "initial_noise": first_input["hidden_states"],
        "decoder_condition": first_input["encoder_hidden_states"],
        "pooled_projection": first_input["pooled_projections"],
        "latent": latent.cpu().numpy(),
        "wave": wave,
    }


def attention_structure(fusion, gaussian, position):
    """Probe a private copy; do not train or mutate the generator's weights."""
    local = copy.deepcopy(fusion).cpu().eval().requires_grad_(False)
    local.cross_attn.in_proj_weight.requires_grad_(True)
    gs = torch.as_tensor(gaussian).clone()
    pos = torch.as_tensor(position).clone()
    local(gs, pos).square().mean().backward()
    q, k, v = local.cross_attn.in_proj_weight.grad.chunk(3)
    with torch.no_grad():
        first, _ = local.cross_attn(local.norm1(gs), pos[:, None], pos[:, None])
        second, _ = local.cross_attn(torch.zeros_like(gs), pos[:, None], pos[:, None])
    return {
        "query_key_value_gradient_norms": [float(x.norm()) for x in (q, k, v)],
        "attention_output_unchanged_when_queries_zeroed": bool(
            torch.equal(first, second)
        ),
        "key_count": 1,
        "scope": "Singleton position key makes attention weights constant; full residual fusion and decoder can still respond to geometry/position.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ["source", "assets", "input", "output"]:
        parser.add_argument("--" + arg, required=True, type=Path)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("use a new external output directory")
    if pilot.sha256(args.input / "bowl-6.ply") != pilot.BOWL_SHA:
        raise ValueError("only the disclosed TRAIN bowl is in scope")
    torch.set_num_threads(4)
    torch.manual_seed(0)
    ns = pilot.load_definitions(args.source)
    models = pilot.build_models(args.source, args.assets, ns)
    _, encoder, _, original_fusion, vae, _ = models
    gs = ns["load_ply"](str(args.input / "bowl-6.ply"))
    normalized, scaler = ns["preprocess_gaussian"](
        gs, "cuda", scaler_class=ns["MinMaxScaler"], return_scaler=True
    )
    positions = [ns["normalize_position"](p, scaler, "cuda") for p in CONTACTS]
    args.output.mkdir(parents=True)
    rows = []
    arrays = {}

    def run(name, ge, fp, condition, position):
        value = render(ns, models, ge, fp, condition, position)
        np.savez(args.output / f"{name}.npz", **value)
        if not all(np.isfinite(x).all() for x in value.values()):
            raise ValueError("nonfinite generation; arrays retained")
        arrays[name] = value
        row = {
            "name": name,
            "raw_peak": float(np.abs(value["wave"]).max()),
            "stage_norms": {k: float(np.linalg.norm(v)) for k, v in value.items()},
        }
        if len(arrays) > 1:
            row["against_full_a"] = {
                k: compare(arrays["full-a"][k], v) for k, v in value.items()
            }
        rows.append(row)
        print(name, json.dumps(row.get("against_full_a", {})), flush=True)

    capture = CaptureGaussian(encoder)
    run("full-a", capture, FusionProbe(original_fusion), normalized, positions[0])
    base_features, base_rng = capture.features, capture.rng
    cached = CachedGaussian(base_features, base_rng)
    run("cached-a", cached, FusionProbe(original_fusion), normalized, positions[0])
    run("cached-b", cached, FusionProbe(original_fusion), normalized, positions[1])
    run(
        "full-a-repeat", capture, FusionProbe(original_fusion), normalized, positions[0]
    )
    run(
        "cached-a-repeat",
        cached,
        FusionProbe(original_fusion),
        normalized,
        positions[0],
    )
    for intervention in ["centers-contract-x", "neutral-appearance"]:
        changed = counterfactual(normalized, positions[0], intervention)
        torch.manual_seed(0)
        with torch.no_grad():
            features = encoder(changed, "cuda")
        run(
            intervention,
            CachedGaussian(features, base_rng),
            FusionProbe(original_fusion),
            changed,
            positions[0],
        )
    for intervention in ["zero-position", "zero-conditioning"]:
        run(
            intervention,
            cached,
            FusionProbe(original_fusion, intervention),
            normalized,
            positions[0],
        )
    # Ensure all interventions really used identical initial diffusion noise.
    if not all(
        np.array_equal(value["initial_noise"], arrays["full-a"]["initial_noise"])
        for value in arrays.values()
    ):
        raise ValueError(
            "diffusion noise changed; causal comparison invalid, arrays retained"
        )
    rate = vae.config.sampling_rate
    gain = min(1.0, 0.98 / max(row["raw_peak"] for row in rows))
    for name, value in arrays.items():
        path = args.output / f"{name}.wav"
        wavfile.write(path, rate, (value["wave"].T * gain).astype(np.float32))
        next(row for row in rows if row["name"] == name)["wav_sha256"] = pilot.sha256(
            path
        )
    order = [
        "cached-a",
        "cached-b",
        "centers-contract-x",
        "neutral-appearance",
        "zero-position",
        "zero-conditioning",
    ]
    comparison = np.concatenate(
        [
            part
            for name in order
            for part in [arrays[name]["wave"].T * gain, np.zeros((rate // 2, 2))]
        ]
    ).astype(np.float32)
    wavfile.write(args.output / "comparison.wav", rate, comparison)
    for label, second in [
        ("contact", "cached-b"),
        ("layout", "centers-contract-x"),
        ("appearance", "neutral-appearance"),
    ]:
        pair = np.concatenate(
            [
                arrays["cached-a"]["wave"].T * gain,
                np.zeros((rate // 2, 2)),
                arrays[second]["wave"].T * gain,
            ]
        )
        wavfile.write(
            args.output / f"{label}-comparison.wav", rate, pair.astype(np.float32)
        )
    result = {
        "status": "REPORT_ONLY_CONDITIONING_DISCRIMINATOR",
        "rows": rows,
        "script_sha256": pilot.sha256(Path(__file__)),
        "pilot_sha256": pilot.sha256(Path(pilot.__file__)),
        "source_hashes": pilot.SOURCES,
        "weight_hashes": pilot.WEIGHTS,
        "geometry_sha256": pilot.BOWL_SHA,
        "initial_noise_equal": True,
        "shared_gain": gain,
        "rate": rate,
        "comparison_order": order,
        "attention_structure": attention_structure(
            original_fusion,
            arrays["cached-a"]["gaussian"],
            arrays["cached-a"]["position"],
        ),
        "raw_full_decode_retained": True,
        "target_audio_input": False,
        "limits": "Known TRAIN object; counterfactuals test sensitivity, not calibrated physical/material changes or generalization.",
    }
    (args.output / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print("complete", args.output, flush=True)


if __name__ == "__main__":
    main()
