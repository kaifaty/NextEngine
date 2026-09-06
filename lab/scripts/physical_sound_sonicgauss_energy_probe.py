"""Source-free two-sample generation and a bounded energy-score discriminator.

Uses the energy-score formulation of Gritsenko et al., NeurIPS 2020.
This is an explicit FFT/audible adaptation, NOT their TensorFlow/Mel pipeline
or a physical-realism validator. No optimizer, reference-normalized distance,
per-object gain, random crop, or reference audio in the render path.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_cohort as cohort
import physical_sound_sonicgauss_pilot as pilot
import physical_sound_sonicgauss_shared_fit as shared
import physical_sound_sonicgauss_waveform_fit as decoded
import torch
from safetensors.torch import load_file
from scipy.io import wavfile
from torch.nn import functional as F

LENGTH = int(44100 * 2.98)


def features(wave):
    if wave.ndim != 3 or wave.shape[1] != 2 or wave.shape[-1] > LENGTH:
        raise ValueError("expected bounded batch/stereo waveform")
    mono = decoded.audible(F.pad(wave.mean(1), (0, LENGTH - wave.shape[-1])))
    result = []
    for size in (64, 128, 256, 512, 1024, 2048):
        spectrum = torch.stft(
            mono,
            size,
            size // 2,
            window=torch.hann_window(size, periodic=False, device=wave.device),
            return_complex=True,
        )
        # Smooth magnitude and fixed floor, not normalization by a target.
        result.append((spectrum.real.square() + spectrum.imag.square() + 1e-8).sqrt())
    return result


def distance(left, right):
    if len(left) != 6 or len(right) != 6:
        raise ValueError("six matching spectral resolutions required")
    parts = []
    for a, b in zip(left, right, strict=True):
        if a.shape != b.shape:
            raise ValueError("spectral shape mismatch")
        linear = (a - b).abs().mean((1, 2))
        # Framewise Euclidean norm / sqrt(bin count), exact zero at identity.
        log = (
            torch.linalg.vector_norm((a + 1e-8).log() - (b + 1e-8).log(), dim=1).mean(1)
            / a.shape[1] ** 0.5
        )
        parts.append(linear + log)
    return torch.stack(parts).sum(0).mean()


def energy(reference, first, second):
    cross = distance(reference, first) + distance(reference, second)
    repulsion = distance(first, second)
    return cross - repulsion, {"cross": cross, "repulsion": repulsion}


def render(args):
    rows, geometry, hashes = shared.caches(args.data, args.baseline)
    torch.manual_seed(0)
    torch.backends.cudnn.deterministic = True
    models = pilot.build_models(
        args.source, args.assets, pilot.load_definitions(args.source)
    )
    model, _, position, fusion, vae, _ = models
    fit_receipt = None
    if args.fit is not None:
        fit_receipt = json.loads((args.fit / "fit.json").read_text())
        if fit_receipt["input_sha256"] != pilot.sha256(
            args.data / "inputs.json"
        ) or fit_receipt["adapter_sha256"] != pilot.sha256(
            args.fit / "adapter.safetensors"
        ):
            raise ValueError("adapter/input binding mismatch")
        adapter = shared.ContactResidual().cuda().eval().requires_grad_(False)
        adapter.load_state_dict(
            load_file(args.fit / "adapter.safetensors", device="cuda"), strict=True
        )
        fusion = shared.AdaptedFusion(fusion, adapter)
    rng = torch.Generator(device="cuda").manual_seed(20260906)
    common_noise = getattr(args, "common_noise", False)
    noise_pair = (
        [torch.randn(1, 64, 64, device="cuda", generator=rng) for _ in range(2)]
        if common_noise
        else None
    )
    records, waves = [], []
    args.output.mkdir(parents=True)
    with torch.no_grad():
        for row in rows:
            oid = row["object_id"]
            cache = geometry[oid]
            for contact in row["contacts"]:
                index = contact["index"]
                fused = fusion(
                    cache["features"].cuda(),
                    position(cache["contacts"][index].cuda())[None],
                )
                pair = []
                for sample in range(2):
                    noise = (
                        noise_pair[sample]
                        if common_noise
                        else torch.randn(1, 64, 64, device="cuda", generator=rng)
                    )
                    latent = decoded.generate(model, fused, noise)
                    wave = vae.decode(latent.transpose(1, 2)).sample[0]
                    if wave.shape != (2, 131072) or not torch.isfinite(wave).all():
                        raise ValueError("invalid full generation")
                    if not records:
                        repeated = vae.decode(
                            decoded.generate(model, fused, noise).transpose(1, 2)
                        ).sample[0]
                        if not torch.equal(wave, repeated):
                            raise ValueError("explicit-noise replay differs")
                    name = f"object-{oid:02d}-contact-{index}-sample-{sample}"
                    path = args.output / (name + ".npz")
                    wave = wave.cpu().numpy()
                    pair.append(noise.cpu().numpy())
                    np.savez(
                        path,
                        wave=wave,
                        initial_noise=pair[-1],
                        latent=latent.cpu().numpy(),
                    )
                    records.append(
                        {
                            "object_id": oid,
                            "contact_index": index,
                            "sample": sample,
                            "local_fit_role": row["local_fit_role"],
                            "name": name,
                            "npz_sha256": pilot.sha256(path),
                        }
                    )
                    waves.append(wave)
                if np.array_equal(*pair):
                    raise ValueError("samples reused the same noise")
                print("independent pair", oid, index, flush=True)
    gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in waves))
    for row, wave in zip(records, waves, strict=True):
        path = args.output / (row["name"] + ".wav")
        wavfile.write(path, 44100, (wave.T * gain).astype(np.float32))
        row.update(wav=str(path), wav_sha256=pilot.sha256(path))
    cohort.save(
        args.output / "result.json",
        {
            "rows": records,
            "shared_gain": gain,
            "input_sha256": pilot.sha256(args.data / "inputs.json"),
            "geometry_hashes": hashes,
            "script_sha256": pilot.sha256(Path(__file__)),
            "seed": 20260906,
            "common_noise_across_conditions": common_noise,
            "target_audio_read": False,
            "explicit_noise_replay_exact": True,
            "weights": pilot.WEIGHTS,
            "fit_sha256": pilot.sha256(args.fit / "fit.json")
            if fit_receipt is not None
            else None,
            "scope": "source-free independent noise on frozen published backbone; optional shared adapter; no physical acceptance",
        },
    )


def probe(args):
    corpus = json.loads((args.data / "corpus.json").read_text())
    shared.validate_inputs(corpus)
    receipt = json.loads((args.generated / "result.json").read_text())
    if receipt["input_sha256"] != pilot.sha256(args.data / "inputs.json"):
        raise ValueError("generation binding mismatch")
    by_key = {
        (r["object_id"], r["contact_index"], r["sample"]): r for r in receipt["rows"]
    }
    expected = {(o, c, s) for o in cohort.OBJECTS for c in range(6) for s in range(2)}
    if set(by_key) != expected or len(receipt["rows"]) != len(expected):
        raise ValueError("incomplete independent cohort")
    args.output.mkdir(parents=True)
    rows, auditions = [], []
    for obj in corpus["objects"]:
        oid = obj["object_id"]
        for contact in obj["contacts"]:
            index = contact["index"]
            ref = contact["reference"]
            reference = shared.teacher_audio(Path(ref["path"]), ref["sha256"])
            waves, noises = [], []
            for sample in range(2):
                record = by_key[oid, index, sample]
                path = args.generated / (record["name"] + ".npz")
                if pilot.sha256(path) != record["npz_sha256"]:
                    raise ValueError("sample hash mismatch")
                with np.load(path, allow_pickle=False) as saved:
                    waves.append(saved["wave"].copy())
                    noises.append(saved["initial_noise"].copy())
            if np.array_equal(*noises):
                raise ValueError("identical noise is not two independent samples")
            target = features(torch.from_numpy(reference)[None])
            gain = torch.tensor(1.0, requires_grad=True)
            predicted = [features(torch.from_numpy(w)[None] * gain) for w in waves]
            score, parts = energy(target, *predicted)
            paired_grad = torch.autograd.grad(parts["cross"], gain, retain_graph=True)[
                0
            ].item()
            energy_grad = torch.autograd.grad(score, gain)[0].item()
            metrics = [
                decoded.audio_metrics(reference.mean(0), w.mean(0)) for w in waves
            ]
            filtered = [
                decoded.audible(torch.from_numpy(w.mean(0))[None]).numpy()
                for w in [reference, *waves]
            ]
            rms = [float(np.sqrt(np.mean(w.astype(float) ** 2))) for w in filtered]
            rows.append(
                {
                    "object_id": oid,
                    "contact_index": index,
                    "local_fit_role": obj["local_fit_role"],
                    "energy": score.item(),
                    **{k: v.item() for k, v in parts.items()},
                    "paired_gain_gradient": paired_grad,
                    "energy_gain_gradient": energy_grad,
                    "audible_rms": rms,
                    "both_already_quieter": max(rms[1:]) < rms[0],
                    "sample_metrics": metrics,
                }
            )
            if index == 0:
                auditions.append(
                    (oid, [reference.mean(0), *[w.mean(0) for w in waves]])
                )
    summary = {}
    for role in ("train", "development"):
        selected = [r for r in rows if r["local_fit_role"] == role]
        summary[role] = {
            "count": len(selected),
            "both_already_quieter": sum(r["both_already_quieter"] for r in selected),
            **{
                kind: {
                    "attenuation_count": sum(
                        r[kind + "_gain_gradient"] > 0 for r in selected
                    ),
                    "already_quieter_attenuation": sum(
                        r[kind + "_gain_gradient"] > 0 and r["both_already_quieter"]
                        for r in selected
                    ),
                    "mean_gain_gradient": float(
                        np.mean([r[kind + "_gain_gradient"] for r in selected])
                    ),
                }
                for kind in ("paired", "energy")
            },
            "mean_repulsion": float(np.mean([r["repulsion"] for r in selected])),
        }
    gain = min(
        1.0, 0.98 / max(float(abs(w).max()) for _, parts in auditions for w in parts)
    )
    for oid, parts in auditions:
        wavfile.write(
            args.output / f"object-{oid:02d}-comparison.wav",
            44100,
            np.concatenate(
                [v for w in parts for v in (w * gain, np.zeros(22050))]
            ).astype(np.float32),
        )
    result = {
        "rows": rows,
        "summary": summary,
        "comparison_gain": gain,
        "generated_receipt_sha256": pilot.sha256(args.generated / "result.json"),
        "script_sha256": pilot.sha256(Path(__file__)),
        "distance": "six FFT sizes 64..2048; hop half; symmetric Hann; audible20Hz; mean L1 + framewise log L2/sqrt(bins); no target normalization",
        "scope": "scalar common-gain derivative at raw output, not an output gain fit or proof of neural optimization/physical realism",
    }
    cohort.save(args.output / "result.json", result)
    print(json.dumps(summary, indent=2), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("render", "probe"))
    parser.add_argument(
        "--common-noise",
        action="store_true",
        help="reuse one independent noise pair across all conditions for controlled interventions",
    )
    for name in ("source", "assets", "data", "baseline", "generated", "fit", "output"):
        parser.add_argument("--" + name, type=Path, required=name in ("data", "output"))
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    required = (
        ("source", "assets", "baseline") if args.mode == "render" else ("generated",)
    )
    if any(getattr(args, name) is None for name in required):
        parser.error("missing mode-specific input")
    torch.set_num_threads(4)
    (render if args.mode == "render" else probe)(args)


if __name__ == "__main__":
    main()
