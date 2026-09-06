"""Bounded timing/attenuation/distribution discriminator; no model fitting.

Exact scalar optima are diagnostics of an objective, NOT gain corrections.
Synthetic sounds demonstrate a scoring counterexample, not neural generation.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_sonicgauss_codec_probe as codec
import physical_sound_sonicgauss_cohort as cohort
import physical_sound_sonicgauss_shared_fit as shared
from scipy.io import wavfile
from scipy.signal import correlate, correlation_lags, stft


def magnitudes(wave):
    if wave.ndim != 1 or not np.isfinite(wave).all() or len(wave) < 2048:
        raise ValueError("expected long finite mono")
    return [
        np.abs(stft(wave, 44100, nperseg=n, noverlap=n * 3 // 4)[2]).astype(np.float64)
        for n in (512, 1024, 2048)
    ]


def optimum_gain(reference, generated):
    """Weighted median solves sum_r ||R-gG||_1/||R||_1 for nonnegative g."""
    length = max(len(reference), len(generated))
    ref, gen = [
        magnitudes(np.pad(w, (0, length - len(w)))) for w in (reference, generated)
    ]
    weights, ratios = [], []
    gradient = 0.0
    for r, g in zip(ref, gen, strict=True):
        denominator = r.sum()
        if denominator <= 0:
            raise ValueError("silent reference")
        keep = g > 0
        ratios.append(r[keep] / g[keep])
        weights.append(g[keep] / denominator / 3)
        gradient += float((np.sign(g - r) * g).sum() / denominator / 3)
    ratios, weights = np.concatenate(ratios), np.concatenate(weights)
    if not len(weights) or weights.sum() == 0:
        raise ValueError("silent generator")
    order = np.argsort(ratios)
    index = np.searchsorted(np.cumsum(weights[order]), weights.sum() / 2)
    gain = float(ratios[order[index]])

    def loss(scale):
        return float(
            np.mean(
                [
                    np.abs(r - g * scale).sum() / r.sum()
                    for r, g in zip(ref, gen, strict=True)
                ]
            )
        )

    return {
        "optimal_gain": gain,
        "loss_at_gain_one": loss(1),
        "loss_at_optimum": loss(gain),
        "gradient_at_gain_one": gradient,
    }


def envelope(wave):
    padded = np.pad(wave, (0, (-len(wave)) % 88))
    return np.sqrt(np.mean(padded.astype(float).reshape(-1, 88) ** 2, axis=1))


def lag_frames(reference, generated):
    a, b = envelope(reference), envelope(generated)
    values = correlate(b, a)
    lags = correlation_lags(len(b), len(a))
    keep = abs(lags) <= 125  # fixed +/-249ms search; measurements use 1.995ms frames
    return int(lags[keep][values[keep].argmax()])


def translate(wave, shift, guard=16384):
    if not isinstance(shift, int) or abs(shift) > guard:
        raise ValueError("shift outside guard")
    return np.pad(
        wave, (guard + shift, guard - shift)
    )  # preserves every sample, no wrap/crop


def read_signal(path, sha):
    wave = cohort.audio(Path(path), sha)
    return codec.audible_component(np.stack([wave, wave])).mean(0)


def toy_signals():
    time = np.arange(131418) / 44100
    local = np.maximum(time - 0.1, 0)
    env = (time >= 0.1) * (1 - np.exp(-2000 * local)) * np.exp(-8 * local)
    return [
        (0.05 * env * np.sin(2 * np.pi * f * local)).astype(np.float32)
        for f in (700, 1400, 2800)
    ]


def energy_choices(signals):
    features = [np.concatenate([v.ravel() for v in magnitudes(w)]) for w in signals]
    divisor = float(np.mean([f.sum() for f in features]))
    choices = {
        "correct_empirical_distribution": features,
        "collapsed_first_tone": [features[0]],
        "silence": [np.zeros_like(features[0])],
    }

    def distance(a, b):
        return float(np.abs(a - b).sum() / divisor)

    rows = {}
    for name, values in choices.items():
        attraction = float(np.mean([distance(a, b) for a in features for b in values]))
        repulsion = float(np.mean([distance(a, b) for a in values for b in values]))
        rows[name] = {
            "paired_distance": attraction,
            "generated_pair_distance": repulsion,
            "energy_score": 2 * attraction - repulsion,
        }
    return rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "generated", "output"):
        parser.add_argument("--" + name, required=True, type=Path)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    corpus = json.loads((args.data / "corpus.json").read_text())
    shared.validate_inputs(corpus)
    generated = json.loads((args.generated / "result.json").read_text())
    if generated["input_sha256"] != cohort.pilot.sha256(args.data / "inputs.json"):
        raise ValueError("input binding mismatch")
    lookup = {
        (r["object_id"], r["contact_index"]): r
        for r in generated["rows"]
        if r["variant"] == "baseline"
    }
    if len(lookup) != 60:
        raise ValueError("complete baseline cohort required")
    args.output.mkdir(parents=True)
    rows = []
    for obj in corpus["objects"]:
        for contact in obj["contacts"]:
            source = contact["reference"]
            row = lookup[(obj["object_id"], contact["index"])]
            ref = read_signal(source["path"], source["sha256"])
            gen = read_signal(row["wav"], row["wav_sha256"]) / generated["shared_gain"]
            lag = lag_frames(ref, gen)
            length = max(len(ref), len(gen))
            r, g = [np.pad(w, (0, length - len(w))) for w in (ref, gen)]
            before = optimum_gain(r, g)
            aligned = optimum_gain(translate(r, 0), translate(g, -lag * 88))
            rows.append(
                {
                    "object_id": obj["object_id"],
                    "contact_index": contact["index"],
                    "local_fit_role": obj["local_fit_role"],
                    "lag_frames": lag,
                    "lag_ms": lag * 88000 / 44100,
                    "reference_peak_frame": int(envelope(ref).argmax()),
                    "generated_peak_frame": int(envelope(gen).argmax()),
                    "original": before,
                    "aligned_diagnostic": aligned,
                }
            )
        print("analyzed", obj["object_id"], flush=True)
    signals = toy_signals()
    controls = {
        "exact": optimum_gain(signals[0], signals[0]),
        "correct_but_double_level": optimum_gain(signals[0], signals[0] * 2),
        "correct_but_2ms_late": optimum_gain(
            translate(signals[0], 0), translate(signals[0], 88)
        ),
        "wrong_frequency_same_timing": optimum_gain(signals[0], signals[1]),
    }
    for i, w in enumerate(signals):
        wavfile.write(args.output / f"toy-tone-{i}.wav", 44100, w)
    # No fitted gain, normalization or internet audio is used in this demo.
    mean_wave = np.mean(signals, axis=0).astype(np.float32)
    wavfile.write(args.output / "toy-mean.wav", 44100, mean_wave)
    wavfile.write(
        args.output / "control-comparison.wav",
        44100,
        np.concatenate(
            [
                p
                for w in signals + [mean_wave]
                for p in [w, np.zeros(22050, dtype=np.float32)]
            ]
        ),
    )
    summary = {}
    for role in ("train", "development"):
        subset = [r for r in rows if r["local_fit_role"] == role]
        summary[role] = {
            "count": len(subset),
            "median_abs_lag_ms": float(np.median([abs(r["lag_ms"]) for r in subset])),
        }
        for kind in ("original", "aligned_diagnostic"):
            summary[role][kind] = {
                "mean_loss": float(
                    np.mean([r[kind]["loss_at_gain_one"] for r in subset])
                ),
                "median_optimal_gain": float(
                    np.median([r[kind]["optimal_gain"] for r in subset])
                ),
                "attenuation_direction_count": sum(
                    r[kind]["gradient_at_gain_one"] > 0 for r in subset
                ),
            }
    result = {
        "rows": rows,
        "summary": summary,
        "synthetic_controls": controls,
        "distribution_counterexample": energy_choices(signals),
        "scope": "diagnostic only: optimal gains/alignment NEVER applied to models/WAVs; toy empirical L1 energy score, not full published SED or physical validation",
        "script_sha256": cohort.pilot.sha256(Path(__file__)),
    }
    cohort.save(args.output / "result.json", result)
    print(
        json.dumps(
            {
                k: result[k]
                for k in (
                    "summary",
                    "synthetic_controls",
                    "distribution_counterexample",
                )
            },
            indent=2,
        ),
        flush=True,
    )


if __name__ == "__main__":
    main()
