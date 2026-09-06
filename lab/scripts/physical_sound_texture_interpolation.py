"""Discriminate coefficient-to-timbre interpolation from neural mapping failure.

Same disclosed48-TRAIN/60-new-surface development split. No model fitting.
Stationary central750ms controls, NOT full-event or physical-quality admission.
Eguchi et al. CC BY4; neural comparison: Powered by Stability AI/TangoFlux.
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

import numpy as np
import physical_sound_texture_surface as surface
import soundfile as sf
from scipy.signal import resample_poly

flow, spectrum = surface.flow, surface.flow.spectrum


def projection(value, low, high):
    value, low, high = [np.asarray(x, np.float64) for x in (value, low, high)]
    if value.shape != low.shape or high.shape != low.shape or value.ndim != 1:
        raise ValueError("matching one-dimensional descriptors required")
    if not all(np.isfinite(x).all() for x in (value, low, high)):
        raise ValueError("nonfinite descriptors")
    delta = high - low
    norm = float(delta @ delta)
    if norm <= 1e-20:
        raise ValueError("indistinguishable interpolation endpoints")
    alpha = float(np.clip((value - low) @ delta / norm, 0, 1))
    return alpha, float(np.linalg.norm(value - (low + alpha * delta)))


def coefficients(item):
    return np.array([item["static_friction"], item["dynamic_friction"]])


def blend(low, high, alpha):
    if not np.isfinite(alpha) or not 0 <= alpha <= 1:
        raise ValueError("convex interpolation only")
    return (1 - alpha) * low + alpha * high


def endpoints(row, table, training, train_spectra):
    # Deliberately requires only TRAIN audio; no development target is passed.
    if len(training) != 48 or any(surface.role(r) != "train" for r in training):
        raise ValueError("exact original48 TRAIN rows required")
    expected = {
        r["id"]
        for r in surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        if surface.role(r) == "train"
    }
    if {r["id"] for r in training} != expected or np.shape(train_spectra) != (48, 513):
        raise ValueError("TRAIN identity/layout mismatch")
    if not np.isfinite(train_spectra).all():
        raise ValueError("invalid TRAIN spectra")
    item = table[row["texture_id"]]
    peers = sorted(
        (table[t] for t in surface.TRAIN if table[t]["category"] == item["category"]),
        key=lambda p: p["static_friction"],
    )
    if len(peers) != 2:
        raise ValueError("two TRAIN surfaces per category required")
    values = []
    for peer in peers:
        values.append(
            spectrum.interpolate(
                {**row, "texture_id": peer["texture_id"]}, training, train_spectra
            )
        )
    alpha, off_line = projection(coefficients(item), *(coefficients(p) for p in peers))
    return (
        values[0],
        values[1],
        {
            "low_texture": peers[0]["texture_id"],
            "high_texture": peers[1]["texture_id"],
            "coefficient_alpha": alpha,
            "coefficient_off_line_distance": off_line,
        },
    )


def shape_error(a, b):
    delta = np.asarray(a, np.float64)[1:] - np.asarray(b, np.float64)[1:]
    return float(np.std(delta))


def oracle_alpha(target, low, high):
    # Target-aided diagnostic of the two-spectrum span, never a generator input.
    centered = [
        np.asarray(x, np.float64)[1:] - np.mean(x[1:]) for x in (target, low, high)
    ]
    if np.linalg.norm(centered[1] - centered[2]) < 1e-10:
        return 0.5
    return projection(*centered)[0]


def publish(output, name, mono):
    stereo = np.repeat(resample_poly(mono, 2, 1)[:, None], 2, axis=1)
    entry, pcm = flow.codec.publish(output / f"{name}.wav", stereo)
    return entry, resample_poly(pcm.mean(1), 1, 2), pcm


def read_neural(entry, root, row):
    path = Path(entry["wav"]).resolve()
    if (
        not path.is_relative_to(root)
        or path.stat().st_size > 8_000_000
        or flow.codec.sha(path) != entry["sha256"]
    ):
        raise ValueError("invalid neural PCM identity")
    wave, rate = sf.read(path, always_2d=True)
    if (
        rate != 44100
        or wave.shape != (entry["frames"], 2)
        or not np.isfinite(wave).all()
    ):
        raise ValueError("invalid neural PCM")
    first, count = round(row["crop_start_seconds"] * rate), round(0.75 * rate)
    crop = wave[first : first + count].mean(1)
    if len(crop) != count:
        raise ValueError("incomplete moving crop")
    return resample_poly(crop, 1, 2)


def run(source_path, neural_path, output):
    source_path, neural_path = source_path.resolve(), neural_path.resolve()
    if neural_path.stat().st_size > 5_000_000:
        raise ValueError("oversized neural result")
    neural = json.loads(neural_path.read_text())
    if (
        neural["status"] != "complete"
        or neural["source_sha256"] != flow.codec.sha(source_path)
        or len(neural["rows"]) != 768
    ):
        raise ValueError("matching completed neural development result required")
    rows, waves, _, _, _ = surface.load_data(source_path)
    table = surface.metadata(source_path)
    training = [r for r in rows if r["role"] == "train"]
    train_spectra = np.stack(
        [
            spectrum.spectrum(w)
            for r, w in zip(rows, waves, strict=True)
            if r["role"] == "train"
        ]
    )
    output = flow.fresh(output)
    shutil.copyfile(__file__, output / "executed-script.py")
    report = {
        "status": "running",
        "scope": "posthoc disclosed development; stationary diagnostic, not neural training/quality admission",
        "source_sha256": flow.codec.sha(source_path),
        "neural_result_sha256": flow.codec.sha(neural_path),
        "playback_gain": 50,
        "training_ids": [r["id"] for r in training],
        "rows": [],
        "predictions": [],
        "references": [],
    }
    flow.save(output / "result.json", report)
    try:
        neural_index = {(r["id"], r["seed"], r["variant"]): r for r in neural["rows"]}
        if len(neural_index) != 768:
            raise ValueError("duplicate neural rows")
        real_index = {
            r["id"]: spectrum.spectrum(w) for r, w in zip(rows, waves, strict=True)
        }
        previews = []
        for row, wave in zip(rows, waves, strict=True):
            if row["role"] != "unseen_surface":
                continue
            low, high, info = endpoints(row, table, training, train_spectra)
            target = spectrum.spectrum(wave)
            alpha_oracle = oracle_alpha(target, low, high)
            predictions = {
                "coefficient": blend(low, high, info["coefficient_alpha"]),
                "category_mean": blend(low, high, 0.5),
                "target_aided_oracle_mix": blend(low, high, alpha_oracle),
                "target_aided_self_spectrum": target,
            }
            other_id = row["id"].rsplit("_", 1)[0] + "_" + str(1 - row["repeat"])
            report["predictions"].append(
                {
                    "id": row["id"],
                    "texture_id": row["texture_id"],
                    **info,
                    "target_aided_alpha": alpha_oracle,
                    "repeat_shape_difference_db": shape_error(
                        target, real_index[other_id]
                    ),
                    **{
                        arm + "_shape_db": shape_error(p, target)
                        for arm, p in predictions.items()
                    },
                }
            )
            real_entry, real, real_pcm = publish(output, row["id"] + "-real", wave * 50)
            report["references"].append({"id": row["id"], **real_entry})
            for seed in (314, 2718):
                candidates = {
                    arm: spectrum.synthesize(p, 0.75, seed) * 50
                    for arm, p in predictions.items()
                }
                for arm in ("descriptor", "category_only"):
                    candidates["neural_" + arm] = read_neural(
                        neural_index[row["id"], seed, arm], neural_path.parent, row
                    )
                pcms = {}
                for arm, candidate in candidates.items():
                    entry, published, pcm = publish(
                        output, f"{row['id']}-{arm}-{seed}", candidate
                    )
                    pcms[arm] = pcm
                    report["rows"].append(
                        {
                            "id": row["id"],
                            "texture_id": row["texture_id"],
                            "seed": seed,
                            "variant": arm,
                            "reference_audio_input": arm.startswith("target_aided"),
                            **entry,
                            "moving_shape_rmse_db": shape_error(
                                spectrum.spectrum(published), spectrum.spectrum(real)
                            ),
                            "moving_level_absolute_error_db": abs(
                                flow.codec.level(published) - flow.codec.level(real)
                            ),
                        }
                    )
                if (
                    row["commanded_speed_mm_s"] == 40
                    and row["commanded_normal_force_N"] == 0.5
                    and row["repeat"] == 0
                    and seed == 314
                ):
                    group = []
                    for pcm in (
                        real_pcm,
                        pcms["neural_descriptor"],
                        pcms["coefficient"],
                        pcms["category_mean"],
                        pcms["target_aided_oracle_mix"],
                        pcms["target_aided_self_spectrum"],
                    ):
                        group.extend((pcm, np.zeros((round(0.25 * 44100), 2))))
                    report[f"surface_{row['texture_id']}_comparison"], _ = (
                        flow.codec.publish(
                            output / f"surface-{row['texture_id']}-comparison.wav",
                            np.concatenate(group),
                        )
                    )
                    previews.extend(group)
            print(json.dumps({"evaluated": row["id"]}), flush=True)
        report["comparison"], _ = flow.codec.publish(
            output / "comparison.wav", np.concatenate(previews)
        )
        report["comparison_order"] = (
            "Oak/Steel/Frosted glass: real/neural descriptor/coefficient interpolation/category mean/TARGET-AIDED oracle mix/TARGET-AIDED self spectrum; central750ms each"
        )
        report["summary"] = []
        for texture in surface.HELD:
            for arm in (
                "coefficient",
                "category_mean",
                "target_aided_oracle_mix",
                "target_aided_self_spectrum",
                "neural_descriptor",
                "neural_category_only",
            ):
                rr = [
                    r
                    for r in report["rows"]
                    if r["texture_id"] == texture and r["variant"] == arm
                ]
                report["summary"].append(
                    {
                        "texture_id": texture,
                        "variant": arm,
                        "count": len(rr),
                        **{
                            key: float(np.mean([r[key] for r in rr]))
                            for key in (
                                "moving_shape_rmse_db",
                                "moving_level_absolute_error_db",
                            )
                        },
                    }
                )
        report["status"] = "complete"
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--neural-result", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.neural_result, args.output)
