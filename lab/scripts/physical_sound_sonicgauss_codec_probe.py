"""Separate codec reconstruction loss from conditional generation failure.

Report-only: reference audio IS an input to reconstruction, never represented as
source-free generation. No training, latent fitting, level adjustment or sweeps.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_cohort as cohort
import physical_sound_sonicgauss_pilot as pilot
import physical_sound_sonicgauss_shared_fit as shared
import torch
from scipy.io import wavfile
from scipy.signal import butter, sosfiltfilt


def controls():
    time = np.arange(131418, dtype=np.float64) / 44100
    ring = sum(
        np.sin(2 * np.pi * freq * time) / (i + 1)
        for i, freq in enumerate((700, 1700, 5100))
    )
    ring *= (1 - np.exp(-time * 2000)) * np.exp(-time * 5)
    ring = (ring / abs(ring).max() * 0.05).astype(np.float32)
    transient = (0.05 * np.exp(-(((time - 0.01) / 0.0005) ** 2))).astype(np.float32)
    return {
        name: np.stack([wave, wave])
        for name, wave in (
            ("ring", ring),
            ("quiet-ring", ring * 0.1),
            ("short-pulse", transient),
            ("silence", np.zeros(len(time), dtype=np.float32)),
        )
    }


def signal_stats(wave):
    if (
        wave.ndim != 2
        or wave.shape[0] != 2
        or not np.isfinite(wave).all()
        or not wave.shape[1]
    ):
        raise ValueError("expected finite nonempty stereo")
    mono = wave.astype(np.float64).mean(axis=0)
    energy = np.square(mono).sum()
    power = abs(np.fft.rfft(mono)) ** 2
    freq = np.fft.rfftfreq(len(mono), 1 / 44100)
    audible = power.copy()
    audible[freq < 20] = 0
    return {
        "rms": float(np.sqrt(np.mean(wave.astype(float) ** 2))),
        "peak": float(abs(wave).max()),
        "late_energy_fraction_after_100ms": float(np.square(mono[4410:]).sum() / energy)
        if energy
        else None,
        "sub20hz_power_fraction": float(power[freq < 20].sum() / power.sum())
        if power.sum()
        else None,
        "audible_peak_hz": float(freq[audible.argmax()]) if audible.sum() else None,
    }


def metric(reference, generated):
    if not np.any(reference):
        return None  # silence remains a noise-floor control, not undefined L1.
    return cohort.magnitude_distance(reference.mean(axis=0), generated.mean(axis=0))


def audible_component(wave):
    """Analysis-only common 20 Hz high-pass; never modifies audition output."""
    signal_stats(wave)
    if wave.shape[1] < 100:
        raise ValueError("signal too short for bounded filter")
    return sosfiltfilt(
        butter(4, 20, fs=44100, btype="highpass", output="sos"),
        wave.astype(np.float64),
        axis=-1,
    ).astype(np.float32)


def audible_audit(output):
    if (output / "audible-audit.json").exists():
        raise ValueError("audit already exists")
    report = json.loads((output / "result.json").read_text())
    rows = []
    for row in report["rows"]:
        with np.load(output / (row["name"] + ".npz"), allow_pickle=False) as stored:
            waves = {k: stored[k] for k in row["signals"]}
        filtered = {k: audible_component(w) for k, w in waves.items()}
        reference = filtered["reference"]
        rows.append(
            {
                "name": row["name"],
                "local_fit_role": row["local_fit_role"],
                "object_id": row.get("object_id"),
                "contact_index": row.get("contact_index"),
                "errors": {
                    k: metric(reference, w)
                    for k, w in filtered.items()
                    if k != "reference"
                },
                "rms": {k: signal_stats(w)["rms"] for k, w in filtered.items()},
            }
        )
    # A validator challenge, not an altered object or a generated sound claim:
    # removing a quiet audible ring while retaining a much louder 3 Hz signal.
    time = np.arange(131418) / 44100
    low = np.stack([0.1 * np.sin(2 * np.pi * 3 * time)] * 2).astype(np.float32)
    ring = controls()["quiet-ring"]
    reference = low + ring
    erased_ring = low
    challenge = {
        "raw_error_when_ring_erased": metric(reference, erased_ring),
        "audible_error_when_ring_erased": metric(
            audible_component(reference), audible_component(erased_ring)
        ),
        "scope": "synthetic validator mutation, no generated/recorded audio edits",
    }
    summary = {}
    for role in ("train", "development"):
        subset = [r for r in rows if r["local_fit_role"] == role]
        summary[role] = {
            k: float(np.mean([r["errors"][k] for r in subset]))
            for k in ("mode", "sample", "baseline", "candidate")
        }
        summary[role]["codec_beats_baseline"] = sum(
            r["errors"]["mode"] < r["errors"]["baseline"] for r in subset
        )
        summary[role]["candidate_beats_baseline"] = sum(
            r["errors"]["candidate"] < r["errors"]["baseline"] for r in subset
        )
    cohort.save(
        output / "audible-audit.json",
        {
            "rows": rows,
            "summary": summary,
            "challenge": challenge,
            "scope": "supplemental diagnostic; shared 4th-order 20Hz zero-phase high-pass only in measurement; raw metric and WAV unchanged; not a physical-quality gate",
            "codec_result_sha256": pilot.sha256(output / "result.json"),
        },
    )
    print(
        json.dumps({"summary": summary, "challenge": challenge}, indent=2), flush=True
    )


def load_vae(assets):
    from diffusers import AutoencoderOobleck

    path = assets / "model.safetensors"
    if pilot.sha256(path) != pilot.WEIGHTS[path.name]:
        raise ValueError("codec weight binding mismatch")
    vae = AutoencoderOobleck()
    loaded = pilot.load_weights(vae, path)
    vae.cuda()
    return vae, loaded


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audible-audit", action="store_true")
    for name in ("assets", "data", "generated", "output"):
        parser.add_argument("--" + name, required=name == "output", type=Path)
    args = parser.parse_args()
    if args.output.resolve().is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("external output required")
    if args.audible_audit:
        audible_audit(args.output)
        return
    if args.output.exists():
        raise ValueError("new codec output required")
    if any(x is None for x in (args.assets, args.data, args.generated)):
        parser.error("codec run needs --assets --data --generated")
    corpus = json.loads((args.data / "corpus.json").read_text())
    shared.validate_inputs(corpus)
    generated = json.loads((args.generated / "result.json").read_text())
    if generated["input_sha256"] != pilot.sha256(args.data / "inputs.json"):
        raise ValueError("generation input binding mismatch")
    by_key = {
        (r["object_id"], r["contact_index"], r["variant"]): r for r in generated["rows"]
    }
    if len(by_key) != 120:
        raise ValueError("complete paired cohort required")
    torch.set_num_threads(4)
    torch.manual_seed(0)
    vae, loaded = load_vae(args.assets)
    args.output.mkdir(parents=True)
    records, auditions, outputs = [], [], []

    def run(name, source, metadata, baseline=None, candidate=None):
        with torch.no_grad():
            posterior = vae.encode(torch.from_numpy(source)[None].cuda()).latent_dist
            mean = posterior.mode()
            # One fixed standard-normal draw for ALL cases; no seed selection.
            rng = torch.Generator(device="cuda").manual_seed(0)
            sampled = posterior.sample(generator=rng)
            mode_wave = vae.decode(mean).sample[0].cpu().numpy()
            sample_wave = vae.decode(sampled).sample[0].cpu().numpy()
        values = {"reference": source, "mode": mode_wave, "sample": sample_wave}
        if baseline is not None:
            values.update(baseline=baseline, candidate=candidate)
        if any(w.shape[0] != 2 or not np.isfinite(w).all() for w in values.values()):
            raise ValueError("invalid codec output")
        np.savez(
            args.output / f"{name}.npz",
            mean=mean.cpu().numpy(),
            std=posterior.std.cpu().numpy(),
            sampled=sampled.cpu().numpy(),
            **values,
        )
        record = {
            "name": name,
            **metadata,
            "latent_shape": list(mean.shape),
            "posterior_std_rms": float(posterior.std.square().mean().sqrt()),
            "errors": {
                k: metric(source, w) for k, w in values.items() if k != "reference"
            },
            "signals": {k: signal_stats(w) for k, w in values.items()},
        }
        records.append(record)
        for label, wave in values.items():
            outputs.append((f"{name}-{label}.wav", wave))
        auditions.append(
            (
                name + "-comparison.wav",
                [source, mode_wave, baseline]
                if baseline is not None
                else [source, mode_wave, sample_wave],
            )
        )
        print(name, record["errors"], flush=True)

    for obj in corpus["objects"]:
        oid = obj["object_id"]
        for contact in obj["contacts"]:
            index = contact["index"]
            ref = contact["reference"]
            source = shared.teacher_audio(Path(ref["path"]), ref["sha256"])
            comparison = []
            for kind in ("baseline", "candidate"):
                row = by_key[(oid, index, kind)]
                if pilot.sha256(Path(row["wav"])) != row["wav_sha256"]:
                    raise ValueError("generated waveform hash mismatch")
                rate, wave = wavfile.read(row["wav"])
                if (
                    rate != 44100
                    or wave.shape != (131072, 2)
                    or wave.dtype != np.float32
                ):
                    raise ValueError("invalid generation layout")
                comparison.append(wave.T / generated["shared_gain"])
            run(
                f"object-{oid:02d}-contact-{index}",
                source,
                {
                    "object_id": oid,
                    "contact_index": index,
                    "local_fit_role": obj["local_fit_role"],
                    "reference_sha256": ref["sha256"],
                },
                *comparison,
            )
    for name, value in controls().items():
        run("control-" + name, value, {"local_fit_role": "synthetic_control"})
    gain = min(1.0, 0.98 / max(float(abs(w).max()) for _, w in outputs))
    for name, value in outputs:
        wavfile.write(args.output / name, 44100, (value.T * gain).astype(np.float32))
    for name, parts in auditions:
        joined = np.concatenate(
            [x for w in parts for x in [w.T * gain, np.zeros((22050, 2))]]
        )
        wavfile.write(args.output / name, 44100, joined.astype(np.float32))
    summary = {}
    for role in ("train", "development"):
        selected = [r for r in records if r["local_fit_role"] == role]
        summary[role] = {
            k: float(np.mean([r["errors"][k] for r in selected]))
            for k in ("mode", "sample", "baseline", "candidate")
        }
        summary[role]["codec_mode_beats_baseline"] = sum(
            r["errors"]["mode"] < r["errors"]["baseline"] for r in selected
        )
        summary[role]["sample_beats_mode"] = sum(
            r["errors"]["sample"] < r["errors"]["mode"] for r in selected
        )
    cohort.save(
        args.output / "result.json",
        {
            "rows": records,
            "summary": summary,
            "shared_gain": gain,
            "codec_weight_sha256": pilot.WEIGHTS["model.safetensors"],
            "loaded": loaded,
            "script_sha256": pilot.sha256(Path(__file__)),
            "input_sha256": generated["input_sha256"],
            "teacher_window": "2.98s unnormalized stereo, matched prior fit; no post-decode trimming",
            "scope": "codec diagnostic with reference input, NOT source-free generation; no training",
            "sample_seed": 0,
            "comparison_order_real": ["reference", "mode", "baseline"],
            "comparison_order_control": ["reference", "mode", "sample"],
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == "__main__":
    main()
