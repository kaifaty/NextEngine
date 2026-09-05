"""Cached paired source/codec/generator DC and AC diagnostic; no neural fitting.

Disclosed physical-sound lab only. Original signals and old metrics retained.
Eguchi et al./CC BY4; Powered by Stability AI/TangoFlux. Not a quality judge.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_texture_hybrid as hybrid
from scipy.signal import resample_poly

surface, flow = hybrid.surface, hybrid.flow


def components(mono):
    x = np.asarray(mono, np.float64)
    if x.ndim != 1 or len(x) < 441 or not np.isfinite(x).all():
        raise ValueError("finite shared-band mono samples required")
    mean, total, ac = float(x.mean()), float(np.mean(x * x)), float(np.var(x))
    blocks = x[: len(x) // 441 * 441].reshape(-1, 441)
    block_mean_power = float(np.mean(blocks.mean(1) ** 2))
    return {
        "mean": mean,
        "absolute_mean": abs(mean),
        "total_power": total,
        "dc_power": mean**2,
        "ac_power": ac,
        "dc_fraction": mean**2 / max(total, 1e-18),
        "total_dbfs": float(10 * np.log10(max(total, 1e-18))),
        "ac_dbfs": float(10 * np.log10(max(ac, 1e-18))),
        "block_mean_power_fraction": block_mean_power
        / max(float(np.mean(blocks**2)), 1e-18),
    }


def envelope(mono, treatment):
    x = np.asarray(mono, np.float64)
    if x.ndim != 1 or len(x) < 441 or not np.isfinite(x).all():
        raise ValueError("finite shared-band event required")
    if treatment not in ("original", "event_mean_removed", "block_means_removed"):
        raise ValueError("unknown diagnostic, not a selectable admission metric")
    if treatment == "event_mean_removed":
        x = x - x.mean()
    blocks = x[: len(x) // 441 * 441].reshape(-1, 441)
    if treatment == "block_means_removed":
        blocks = blocks - blocks.mean(1, keepdims=True)
    return 10 * np.log10(np.maximum(np.mean(blocks**2, axis=1), 1e-18))


def compare(candidate, reference):
    if candidate.shape != reference.shape:
        raise ValueError("paired full-event shape mismatch")
    return {
        name + "_envelope_mae_db": float(
            np.mean(abs(envelope(candidate, name) - envelope(reference, name)))
        )
        for name in ("original", "event_mean_removed", "block_means_removed")
    }


def read_report(root, directory):
    path = root / directory / "result.json"
    if path.stat().st_size > 5_000_000:
        raise ValueError("oversized prior report")
    report = json.loads(path.read_text())
    if report["status"] != "complete":
        raise ValueError("completed prior artifact required")
    return report, path.parent, flow.codec.sha(path)


def moving_comparison(candidate, reference):
    """Fixed moving-window diagnostics, not a full-event acceptance replacement."""
    if candidate.shape != (16538,) or reference.shape != candidate.shape:
        raise ValueError("two source-defined750ms/shared22.05kHz windows required")
    a, b = [envelope(x, "original") for x in (candidate, reference)]
    delta = (
        flow.spectrum.spectrum(candidate)[1:] - flow.spectrum.spectrum(reference)[1:]
    )
    return {
        "bins": len(a),
        "ordered_envelope_mae_db": float(np.mean(abs(a - b))),
        "envelope_distribution_w1_db": float(np.mean(abs(np.sort(a) - np.sort(b)))),
        "level_absolute_error_db": abs(
            flow.codec.level(candidate) - flow.codec.level(reference)
        ),
        "shape_rmse_db": float(np.std(delta)),
        "candidate_envelope_std_db": float(np.std(a)),
        "reference_envelope_std_db": float(np.std(b)),
    }


def repeat_check(root, output):
    root = root.resolve()
    prior, prior_root, prior_sha = read_report(
        root, "texture-windowed-field-2026-09-05"
    )
    source = root / "cluster-surface-transfer-grid-2026-09-05" / "result.json"
    if prior["model"]["source_sha256"] != flow.codec.sha(source):
        raise ValueError("source/model lineage mismatch")
    rows, _, _, _, manifest = surface.load_data(source)
    originals = {r["id"]: r for r in manifest["rows"]}
    metadata = {r["id"]: r for r in rows}
    output = flow.fresh(output)
    report = {
        "status": "running",
        "scope": "posthoc moving-window real-repeat diagnostic; not a quality threshold or temporal-event test",
        "source_sha256": flow.codec.sha(source),
        "windowed_result_sha256": prior_sha,
        "new_training_steps": 0,
        "rows": [],
    }
    flow.save(output / "result.json", report)

    def original(row):
        entry = originals[row["id"]]
        path = flow.spectrum.checked(entry["audio"], manifest, source.parent)
        wave, rate = surface.event.sf.read(path, always_2d=True)
        if rate != 44100 or wave.shape[1] != 1 or not np.isfinite(wave).all():
            raise ValueError("finite mono44100 source required")
        return np.repeat(wave * 50, 2, axis=1)

    def central(wave, row):
        start = round(row["crop_start_seconds"] * 44100)
        crop = wave[start : start + 33075]
        if len(crop) != 33075:
            raise ValueError("incomplete source-defined moving window")
        return resample_poly(crop.mean(1), 1, 2)

    try:
        previews = []
        for ref in prior["references"]:
            row = metadata[ref["id"]]
            pair = next(
                r
                for r in rows
                if r["texture_id"] == row["texture_id"]
                and r["commanded_speed_mm_s"] == row["commanded_speed_mm_s"]
                and r["commanded_normal_force_N"] == row["commanded_normal_force_N"]
                and r["repeat"] == 1 - row["repeat"]
            )
            real = hybrid.checked_wave(ref, prior_root)
            repeated = original(pair)
            target = central(real, row)
            for seed in (314, 2718):
                waves = {"real_repeat": repeated}
                for variant in ("global", "windowed"):
                    entry = next(
                        r
                        for r in prior["rows"]
                        if r["id"] == row["id"]
                        and r["seed"] == seed
                        and r["variant"] == variant
                    )
                    waves[variant] = hybrid.checked_wave(entry, prior_root)
                for name, wave in waves.items():
                    report["rows"].append(
                        {
                            "id": row["id"],
                            "repeat_id": pair["id"],
                            "texture_id": row["texture_id"],
                            "seed": seed,
                            "speed_mm_s": row["commanded_speed_mm_s"],
                            "variant": name,
                            **moving_comparison(
                                central(wave, pair if name == "real_repeat" else row),
                                target,
                            ),
                        }
                    )
                if (
                    row["texture_id"] == 76
                    and row["commanded_normal_force_N"] == 0.5
                    and seed == 314
                ):
                    # Full recordings/decodes at their original lengths, no time/gain matching.
                    for wave in (real, *waves.values()):
                        previews.extend((wave, np.zeros((11025, 2))))
        if len(report["rows"]) != 144:
            raise ValueError("exact24-record/two-seed/three-arm diagnostic required")
        report["comparison"], _ = flow.codec.publish(
            output / "comparison.wav", np.concatenate(previews)
        )
        report["comparison_order"] = (
            "Frosted glass20/60mm/s,.5N,seed314: real/repeated-real/global/windowed; full events, source lengths differ"
        )
        report["status"] = "complete"
    except BaseException as error:
        report.update(status="failed", error=f"{type(error).__name__}: {error}")
        flow.save(output / "result.json", report)
        raise
    flow.save(output / "result.json", report)


def run(root, output):
    root = root.resolve()
    names = {
        "codec": "texture-codec-controls-2026-09-05",
        "timed": "texture-full-event-evaluation-2026-09-05",
        "surface": "texture-surface-transfer-2026-09-05",
        "hybrid": "texture-hybrid-dc-2026-09-05",
    }
    reports = {key: read_report(root, name) for key, name in names.items()}
    old_grid = root / "cluster-texture-training-grid-2026-09-05" / "result.json"
    new_grid = root / "cluster-surface-transfer-grid-2026-09-05" / "result.json"
    for name in ("codec", "timed"):
        if reports[name][0]["source_sha256"] != flow.codec.sha(old_grid):
            raise ValueError("original-grid lineage mismatch")
    if (
        reports["surface"][0]["source_sha256"] != flow.codec.sha(new_grid)
        or reports["hybrid"][0]["neural_result_sha256"] != reports["surface"][2]
    ):
        raise ValueError("expanded-grid lineage mismatch")
    if reports["codec"][0]["vae_sha256"] != flow.CODEC_SHA:
        raise ValueError("codec identity changed")
    rows, _, _, _, _ = surface.load_data(new_grid)
    output = flow.fresh(output)
    report = {
        "status": "running",
        "scope": "posthoc cached DC/AC causal diagnostic; no new weights or acceptance metric",
        "prior_reports": {k: sha for k, (_, _, sha) in reports.items()},
        "codec_rows": [],
        "paired_rows": [],
    }
    flow.save(output / "result.json", report)
    try:
        codec, codec_root, _ = reports["codec"]
        if len(codec["rows"]) != 432 or codec["arms"]["shared"] != flow.GAIN:
            raise ValueError(
                "complete original24-case/3-channel/2-gain codec audit required"
            )
        for entry in codec["rows"]:
            wave = hybrid.checked_wave(entry, codec_root)
            # Common published50x original units; fractions are scale-invariant.
            mono = resample_poly(wave.mean(1), 1, 2) * (
                50 / codec["arms"][entry["arm"]]
            )
            report["codec_rows"].append(
                {k: entry[k] for k in ("id", "channel", "arm", "variant")}
                | components(mono)
            )
        timed, timed_root, _ = reports["timed"]
        current, current_root, _ = reports["surface"]
        corrected, corrected_root, _ = reports["hybrid"]
        if (
            len(timed["references"]) != 60
            or len(current["rows"]) != 768
            or len(corrected["rows"]) != 576
        ):
            raise ValueError("incomplete paired evaluation")
        previews = []
        for row in rows:
            if (
                row["texture_id"] not in surface.source.TEXTURES
                or row["role"] == "train"
            ):
                continue
            source = next(r for r in timed["references"] if r["id"] == row["id"])
            real_wave = hybrid.checked_wave(source["real"], timed_root)
            reconstructed = hybrid.checked_wave(source["codec"], timed_root)
            first = round(row["crop_start_seconds"] * 44100)
            last = first + 33075
            for seed in (314, 2718):
                prior = next(
                    r
                    for r in timed["rows"]
                    if r["id"] == row["id"]
                    and r["seed"] == seed
                    and r["variant"] == "timed"
                )
                present = next(
                    r
                    for r in current["rows"]
                    if r["id"] == row["id"]
                    and r["seed"] == seed
                    and r["variant"] == "descriptor"
                )
                filtered = next(
                    r
                    for r in corrected["rows"]
                    if r["id"] == row["id"]
                    and r["seed"] == seed
                    and r["variant"] == "hybrid"
                )
                waves = {
                    "real": real_wave,
                    "reference_aided_codec314": reconstructed,
                    "timed_generator": hybrid.checked_wave(prior, timed_root),
                    "surface_generator": hybrid.checked_wave(present, current_root),
                    "hybrid": hybrid.checked_wave(filtered, corrected_root),
                }
                for name, wave in waves.items():
                    if wave.shape != real_wave.shape:
                        raise ValueError("unaligned waveform")
                    central = resample_poly(wave[first:last].mean(1), 1, 2)
                    if len(central) != 16538:
                        raise ValueError("incomplete common central window")
                    report["paired_rows"].append(
                        {
                            "id": row["id"],
                            "texture_id": row["texture_id"],
                            "role": row["role"],
                            "seed": seed,
                            "variant": name,
                            **components(central),
                            **compare(
                                resample_poly(wave.mean(1), 1, 2),
                                resample_poly(real_wave.mean(1), 1, 2),
                            ),
                        }
                    )
                if (
                    seed == 314
                    and row["commanded_speed_mm_s"] == 40
                    and row["commanded_normal_force_N"] == 0.5
                    and row["repeat"] == 0
                ):
                    group = []
                    for wave in waves.values():
                        group.extend((wave, np.zeros((11025, 2))))
                    report[f"surface_{row['texture_id']}_comparison"], _ = (
                        flow.codec.publish(
                            output / f"surface-{row['texture_id']}-comparison.wav",
                            np.concatenate(group),
                        )
                    )
                    previews.extend(group)
        if len(report["paired_rows"]) != 360:
            raise ValueError(
                "exact36-record/two-seed/five-variant paired audit required"
            )
        report["comparison"], _ = flow.codec.publish(
            output / "comparison.wav", np.concatenate(previews)
        )
        report["comparison_order"] = (
            "Nyatoh/Stainless steel/Float glass,40mm/s,.5N,repeat0,seed314: real/reference-aided-codec314/timed-generator/surface-generator/hybrid"
        )
        report["summaries"] = []
        for table, fields in (
            ("codec_rows", ("channel", "arm", "variant")),
            ("paired_rows", ("variant",)),
        ):
            keys = sorted({tuple(r[k] for k in fields) for r in report[table]})
            for key in keys:
                selected = [
                    r for r in report[table] if tuple(r[k] for k in fields) == key
                ]
                result = {
                    "table": table,
                    **dict(zip(fields, key, strict=True)),
                    "count": len(selected),
                }
                measures = [
                    "absolute_mean",
                    "dc_fraction",
                    "block_mean_power_fraction",
                    "total_dbfs",
                    "ac_dbfs",
                ]
                if table == "paired_rows":
                    measures += [
                        name + "_envelope_mae_db"
                        for name in (
                            "original",
                            "event_mean_removed",
                            "block_means_removed",
                        )
                    ]
                for measure in measures:
                    values = [r[measure] for r in selected]
                    result[measure] = {
                        "mean": float(np.mean(values)),
                        "median": float(np.median(values)),
                        "max": float(max(values)),
                    }
                report["summaries"].append(result)
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
    parser.add_argument("--repeat-check", action="store_true")
    args = parser.parse_args()
    (repeat_check if args.repeat_check else run)(args.lab_root, args.output)
