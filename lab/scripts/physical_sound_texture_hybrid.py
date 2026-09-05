"""Source-free timed neural friction with TRAIN-only spectral-shape calibration.

Offline hybrid experiment, not new neural weights, runtime or quality admission.
Fixed rubber probe. Eguchi et al./CC BY4; Powered by Stability AI/TangoFlux.
"""

from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path

import numpy as np
import physical_sound_texture_interpolation as interpolation
import soundfile as sf
import torch
from safetensors.numpy import load_file, save_file
from scipy.ndimage import gaussian_filter1d
from scipy.signal import fftconvolve, firwin2, freqz, resample_poly

surface = interpolation.surface
flow, spectrum, event = surface.flow, surface.flow.spectrum, surface.event
FORMAT = "texture-train-spectrum-bank-v1"
TAPS = 513
FILTER_REVISION = "dc-preserving-fir-v2"


def cook(source_path, output):
    rows, waves, _, _, manifest = surface.load_data(source_path)
    table = surface.metadata(source_path)
    selected = [
        (r, w) for r, w in zip(rows, waves, strict=True) if r["role"] == "train"
    ]
    weights = output / "spectra.safetensors"
    save_file(
        {"spectra": np.stack([spectrum.spectrum(w) for _, w in selected])}, weights
    )
    meta = {
        "format": FORMAT,
        "source_sha256": flow.codec.sha(source_path),
        "table_sha256": surface.TABLE_SHA,
        "spectra_sha256": flow.codec.sha(weights),
        "training_rows": [r for r, _ in selected],
        "surfaces": [table[t] for t in surface.TRAIN],
        "source_license": manifest["license"],
        "scope": "48-TRAIN central750ms log spectra; no development audio values",
    }
    flow.save(output / "spectra.json", meta)
    return load_bank(output)


def load_bank(directory):
    path, weights = directory / "spectra.json", directory / "spectra.safetensors"
    if path.stat().st_size > 100000 or weights.stat().st_size > 200000:
        raise ValueError("oversized spectrum bank")
    meta = json.loads(path.read_text())
    if (
        meta["format"] != FORMAT
        or meta["table_sha256"] != surface.TABLE_SHA
        or flow.codec.sha(weights) != meta["spectra_sha256"]
    ):
        raise ValueError("spectrum bank identity mismatch")
    values = load_file(weights)
    if set(values) != {"spectra"}:
        raise ValueError("unexpected bank arrays")
    bank = values["spectra"]
    table = {r["texture_id"]: r for r in meta["surfaces"]}
    expected = {
        r["id"]: r
        for r in surface.source.conditions(True, {t: str(t) for t in surface.SURFACES})
        if surface.role(r) == "train"
    }
    rows = meta["training_rows"]
    if (
        bank.shape != (48, 513)
        or not np.isfinite(bank).all()
        or (bank < -180).any()
        or (bank > 0).any()
    ):
        raise ValueError("invalid spectrum values")
    if (
        len(rows) != 48
        or {r["id"] for r in rows} != set(expected)
        or len(table) != 6
        or len(meta["surfaces"]) != 6
        or set(table) != set(surface.TRAIN)
    ):
        raise ValueError("bank split changed")
    for row in rows:
        if row["role"] != "train" or any(
            row[k] != v for k, v in expected[row["id"]].items() if k != "texture_name"
        ):
            raise ValueError("bank condition metadata changed")
    for item in table.values():
        surface.features(
            item["category"],
            item["static_friction"],
            item["dynamic_friction"],
            np.full(32, 40),
            np.full(32, 0.5),
        )
    return bank, meta


def predict(
    bank, meta, category, static, dynamic, speed, force, *, category_mean=False
):
    surface.features(category, static, dynamic, np.full(32, speed), np.full(32, force))
    spectrum.features(
        spectrum.TEXTURES[surface.CATEGORIES.index(category)], speed, force
    )
    peers = sorted(
        [r for r in meta["surfaces"] if r["category"] == category],
        key=lambda r: r["static_friction"],
    )
    if len(peers) != 2:
        raise ValueError("two TRAIN peers required")
    values = []
    for peer in peers:
        by_force = [
            spectrum.interpolate(
                {
                    "texture_id": peer["texture_id"],
                    "commanded_normal_force_N": f,
                    "commanded_speed_mm_s": speed,
                },
                meta["training_rows"],
                bank,
            )
            for f in (0.5, 1.0)
        ]
        values.append(interpolation.blend(*by_force, (force - 0.5) / 0.5))
    alpha, distance = interpolation.projection(
        [static, dynamic], *(interpolation.coefficients(p) for p in peers)
    )
    return interpolation.blend(*values, 0.5 if category_mean else alpha), {
        "coefficient_alpha": alpha,
        "off_line_distance": distance,
    }


def calibration_window(physical, speed, frames):
    surface.validate_features(physical)
    indices = np.flatnonzero(physical[3] * 20 + 40 >= speed * 0.9)
    if len(indices) < 16:
        raise ValueError("insufficient steady requested motion")
    center = (indices[0] + indices[-1] + 1) * event.HOP / 2
    first, count = round(center - 0.375 * event.RATE), round(0.75 * event.RATE)
    if first < 0 or first + count > frames:
        raise ValueError("incomplete generated moving window")
    return first, first + count


def anchor(native, physical, speed, predicted_db):
    if (
        native.ndim != 2
        or native.shape[1] != 2
        or not 32 * event.HOP <= len(native) <= 256 * event.HOP
        or not np.isfinite(native).all()
    ):
        raise ValueError("bounded finite full neural decode required")
    if (
        np.shape(predicted_db) != (513,)
        or not np.isfinite(predicted_db).all()
        or np.min(predicted_db) < -180
        or np.max(predicted_db) > 0
    ):
        raise ValueError("bounded TRAIN-predicted spectrum required")
    first, last = calibration_window(physical, speed, len(native))
    own = resample_poly(native[first:last].mean(1), 1, 2)
    if np.var(native[first:last].mean(1)) <= 1e-18:
        raise ValueError("silent or DC-only generator cannot be spectrally calibrated")
    own_db = spectrum.spectrum(own)
    difference = np.asarray(predicted_db, np.float64) - own_db
    difference -= difference[1:].mean()  # Shape only, not target absolute level.
    requested_db = gaussian_filter1d(np.clip(difference, -12, 12), 2)
    frequencies = np.fft.rfftfreq(1024, 1 / 22050)
    taps = firwin2(
        TAPS,
        np.r_[frequencies, 13000, 22050],
        np.r_[10 ** (requested_db / 20), 1, 1],
        fs=44100,
    )
    _, response = freqz(taps, worN=frequencies, fs=44100)
    power = 10 ** (own_db.astype(np.float64) / 10)
    # Intrinsic EQ gain: preserve the model's predicted shared-band moving power.
    # This uses only its own generated PSD, NOT real audio or per-file headroom.
    energy_gain = float(
        np.sqrt(power[1:].sum() / (power[1:] * np.abs(response[1:]) ** 2).sum())
    )
    if not np.isfinite(energy_gain) or not 0.25 <= energy_gain <= 4:
        raise ValueError("unsafe intrinsic filter gain")
    taps *= energy_gain
    # Welch removes segment means. Its shape/energy estimate cannot authorize
    # changing DC. Project the FIR onto unity DC with a smooth low-pass kernel.
    # Unlike changing the central tap, this does not alter the whole passband.
    dc_before = float(taps.sum())
    dc_kernel = np.hanning(TAPS)
    taps += (1 - taps.sum()) * dc_kernel / dc_kernel.sum()
    delay = (TAPS - 1) // 2
    # Remove fixed FIR latency, not a source-fitted acoustic shift. Keep extra tail.
    corrected = fftconvolve(
        native.astype(np.float64), taps[:, None], mode="full", axes=0
    )[delay:]
    if not np.isfinite(corrected).all():
        raise ValueError("nonfinite filtered decode")
    after = resample_poly(corrected[first:last].mean(1), 1, 2)
    return corrected, {
        "filter_revision": FILTER_REVISION,
        "uncorrected_dc_gain": dc_before,
        "dc_gain": float(taps.sum()),
        "tap_count": TAPS,
        "fixed_latency_compensation_samples": delay,
        "retained_extra_tail_samples": delay,
        "calibration_first_sample": first,
        "calibration_last_sample": last,
        "requested_correction_limit_db": 12,
        "intrinsic_energy_gain": energy_gain,
        "moving_level_change_db": flow.codec.level(after) - flow.codec.level(own),
        "applied_source_fitted_shift_samples": 0,
    }


def checked_wave(entry, root):
    path = Path(entry["wav"]).resolve()
    if (
        not path.is_relative_to(root)
        or path.stat().st_size > 8_000_000
        or flow.codec.sha(path) != entry["sha256"]
    ):
        raise ValueError("invalid prior WAV identity")
    wave, rate = sf.read(path, always_2d=True)
    if (
        rate != event.RATE
        or wave.shape != (entry["frames"], 2)
        or not np.isfinite(wave).all()
    ):
        raise ValueError("invalid prior WAV layout")
    return wave


def motion_gate(native, corrected, physical, speed):
    surface.validate_features(physical)
    if (
        not np.isfinite(speed)
        or not 20 <= speed <= 60
        or native.ndim != 2
        or native.shape[1] != 2
        or corrected.shape != (len(native) + (TAPS - 1) // 2, 2)
        or not np.isfinite(native).all()
        or not np.isfinite(corrected).all()
    ):
        raise ValueError("invalid motion-gated correction")
    unfiltered = np.pad(native, ((0, len(corrected) - len(native)), (0, 0)))
    times = (np.arange(physical.shape[1]) + 0.5) * event.HOP / event.RATE
    mask = np.clip(
        np.interp(
            (np.arange(len(corrected)) + 0.5) / event.RATE,
            times,
            physical[3] * 20 + 40,
            left=0,
            right=0,
        )
        / speed,
        0,
        1,
    )
    # Silence/machine background is not a different material's contact response.
    result = unfiltered + mask[:, None] * (corrected - unfiltered)
    return result, {
        "motion_gate": "clip(requested_or_sensor_velocity/commanded_speed,0,1)",
        "unchanged_idle_samples": int(np.count_nonzero(mask == 0)),
    }


def evaluate(source_path, neural_path, output, motion_only=False):
    source_path, neural_path = source_path.resolve(), neural_path.resolve()
    if neural_path.stat().st_size > 5_000_000:
        raise ValueError("oversized previous result")
    previous = json.loads(neural_path.read_text())
    if (
        previous["status"] != "complete"
        or previous["source_sha256"] != flow.codec.sha(source_path)
        or len(previous["rows"]) != 768
    ):
        raise ValueError("matching completed surface trial required")
    output = flow.fresh(output)
    shutil.copyfile(__file__, output / "executed-script.py")
    report = {
        "status": "running",
        "filter_revision": FILTER_REVISION,
        "motion_only": motion_only,
        "scope": "source-free hybrid calibration; no neural fitting/quality admission",
        "neural_result_sha256": flow.codec.sha(neural_path),
        "references": [],
        "rows": [],
    }
    flow.save(output / "result.json", report)
    try:
        bank, meta = cook(source_path, output)
        report["bank"] = meta
        rows, _, _, _, manifest = surface.load_data(source_path)
        originals = {r["id"]: r for r in manifest["rows"]}
        table = surface.metadata(source_path)
        real_references = {r["id"]: r for r in previous["references"]}
        previews = []
        for row in rows:
            if row["role"] != "unseen_surface" and not (
                row["texture_id"] in surface.source.TEXTURES and row["role"] != "train"
            ):
                continue
            entry = originals[row["id"]]
            position, force = [
                np.genfromtxt(
                    spectrum.checked(entry[k], manifest, source_path.parent),
                    delimiter=",",
                    names=True,
                )
                for k in ("position", "force")
            ]
            real = checked_wave(real_references[row["id"]], neural_path.parent)
            real_receipt, _ = flow.codec.publish(output / f"{row['id']}-real.wav", real)
            report["references"].append({"id": row["id"], **real_receipt})
            item = table[row["texture_id"]]
            db, prediction = predict(
                bank,
                meta,
                item["category"],
                item["static_friction"],
                item["dynamic_friction"],
                row["commanded_speed_mm_s"],
                row["commanded_normal_force_N"],
            )
            average, _ = predict(
                bank,
                meta,
                item["category"],
                item["static_friction"],
                item["dynamic_friction"],
                row["commanded_speed_mm_s"],
                row["commanded_normal_force_N"],
                category_mean=True,
            )
            for seed in (314, 2718):
                prior = next(
                    r
                    for r in previous["rows"]
                    if r["id"] == row["id"]
                    and r["seed"] == seed
                    and r["variant"] == "descriptor"
                )
                native = checked_wave(prior["full"], neural_path.parent)
                times = (
                    (np.arange(len(native) // event.HOP) + 0.5) * event.HOP / event.RATE
                )
                physical = surface.builder(table)(
                    row["texture_id"],
                    event.position_speed(position, times),
                    np.interp(times, force["time"], force["force"]),
                )
                variants = {
                    "neural": (native, {}),
                    "hybrid": anchor(native, physical, row["commanded_speed_mm_s"], db),
                    "hybrid_category_mean": anchor(
                        native, physical, row["commanded_speed_mm_s"], average
                    ),
                }
                pcms = {}
                for name, (wave, details) in variants.items():
                    if motion_only and name != "neural":
                        wave, gating = motion_gate(
                            native, wave, physical, row["commanded_speed_mm_s"]
                        )
                        details = {**details, **gating}
                    receipt, pcm = event.publish(
                        output, f"{row['id']}-{name}-{seed}", wave, len(real)
                    )
                    pcms[name] = pcm
                    report["rows"].append(
                        {
                            "id": row["id"],
                            "texture_id": row["texture_id"],
                            "role": row["role"],
                            "seed": seed,
                            "variant": name,
                            "prediction": prediction,
                            "filter": details,
                            **receipt,
                            **surface.metrics(
                                pcm, real, {"row": row, "physical": physical}
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
                        real,
                        pcms["neural"],
                        pcms["hybrid"],
                        pcms["hybrid_category_mean"],
                    ):
                        group.extend((pcm, np.zeros((round(0.25 * event.RATE), 2))))
                    report[f"surface_{row['texture_id']}_comparison"], _ = (
                        flow.codec.publish(
                            output / f"surface-{row['texture_id']}-comparison.wav",
                            np.concatenate(group),
                        )
                    )
                    if row["role"] == "unseen_surface":
                        previews.extend(group)
            print(json.dumps({"evaluated": row["id"]}), flush=True)
            flow.save(output / "result.json", report)
        report["comparison"], _ = flow.codec.publish(
            output / "comparison.wav", np.concatenate(previews)
        )
        report["comparison_order"] = (
            "Oak/Steel/Frosted glass,40mm/s,.5N,repeat0,seed314: real/neural/hybrid/hybrid-category-mean"
        )
        report["summary"] = []
        for texture in surface.HELD + (None,):
            for arm in ("neural", "hybrid", "hybrid_category_mean"):
                rr = [
                    r
                    for r in report["rows"]
                    if r["variant"] == arm
                    and (
                        r["texture_id"] == texture
                        if texture is not None
                        else r["role"] != "unseen_surface"
                    )
                ]
                summary = {"texture_id": texture, "variant": arm, "count": len(rr)}
                for key in (
                    "moving_shape_rmse_db",
                    "moving_level_absolute_error_db",
                    "envelope_db_mae",
                    "pre_contrast_error_db",
                    "post_contrast_error_db",
                    "onset_error_seconds",
                    "offset_error_seconds",
                ):
                    values = [r[key] for r in rr if r[key] is not None]
                    summary[key] = float(np.mean(values)) if values else None
                    summary[key + "_count"] = len(values)
                report["summary"].append(summary)
        report["status"] = "complete"
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


def render(
    model_path,
    bank_path,
    output,
    category,
    static,
    dynamic,
    speed=40.0,
    force=0.5,
    seed=314,
    motion_only=False,
):
    torch.set_num_threads(4)
    bank, meta = load_bank(bank_path)
    models, model_meta = surface.load_models(model_path, "cuda")
    if meta["source_sha256"] != model_meta["source_sha256"]:
        raise ValueError("spectrum/model source mismatch")
    predicted, prediction = predict(bank, meta, category, static, dynamic, speed, force)
    physical, request = event.profile(
        spectrum.TEXTURES[surface.CATEGORIES.index(category)], speed, force
    )
    physical = surface.features(
        category, static, dynamic, physical[3] * 20 + 40, physical[4] * 0.25 + 0.75
    )
    vae, _ = flow.load_codec("cuda")
    native = surface.generate(models["descriptor"], vae, physical, seed)
    corrected, details = anchor(native, physical, speed, predicted)
    if motion_only:
        corrected, gating = motion_gate(native, corrected, physical, speed)
        details.update(gating)
    output = flow.fresh(output)
    count = round(request["duration_seconds"] * event.RATE)
    raw_entry, raw = event.publish(output, "neural", native, count)
    receipt, hybrid = event.publish(output, "hybrid", corrected, count)
    comparison, _ = flow.codec.publish(
        output / "comparison.wav",
        np.concatenate((raw, np.zeros((round(0.25 * event.RATE), 2)), hybrid)),
    )
    flow.save(
        output / "result.json",
        {
            "status": "complete",
            "filter_revision": FILTER_REVISION,
            "motion_only": motion_only,
            "reference_audio_input": False,
            "sensor_input": False,
            "neural": raw_entry,
            "hybrid": receipt,
            "comparison": comparison,
            "comparison_order": "neural/hybrid",
            "model": model_meta,
            "bank_sha256": meta["spectra_sha256"],
            "category": category,
            "static_friction": static,
            "dynamic_friction": dynamic,
            "request": {k: v for k, v in request.items() if k != "texture"},
            "seed": seed,
            "prediction": prediction,
            "filter": details,
        },
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path)
    parser.add_argument("--neural-result", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--model", type=Path)
    parser.add_argument("--bank", type=Path)
    parser.add_argument("--category", choices=surface.CATEGORIES, default="Glass")
    parser.add_argument("--static", type=float)
    parser.add_argument("--dynamic", type=float)
    parser.add_argument("--speed", type=float, default=40)
    parser.add_argument("--force", type=float, default=0.5)
    parser.add_argument("--seed", type=int, default=314)
    parser.add_argument("--motion-only", action="store_true")
    args = parser.parse_args()
    if args.model:
        if not args.bank or args.static is None or args.dynamic is None:
            parser.error("--bank, --static and --dynamic required")
        render(
            args.model,
            args.bank,
            args.output,
            args.category,
            args.static,
            args.dynamic,
            args.speed,
            args.force,
            args.seed,
            args.motion_only,
        )
    else:
        if not args.source or not args.neural_result:
            parser.error("--source and --neural-result required")
        evaluate(args.source, args.neural_result, args.output, args.motion_only)
