"""One-shape 3D teacher fidelity discriminator, not new neural training.

Four bounded mesh levels, common-point modal assurance, eight versus twelve
modes and retained full audio. A local stability threshold is not physical
validation, a family-wide error bound or permission to relabel old teachers.
"""

from __future__ import annotations

import argparse
import itertools
import json
import resource
import time
from pathlib import Path

import numpy as np
import physical_sound_modal3d_pilot as pilot
import physical_sound_sonicgauss_waveform_fit as diagnostics
from scipy.io import wavfile
from scipy.optimize import linear_sum_assignment


def assurance(a, b):
    if (
        a.ndim != 2
        or b.ndim != 2
        or a.shape[0] != b.shape[0]
        or not np.isfinite(a).all()
        or not np.isfinite(b).all()
    ):
        raise ValueError("compatible finite sampled modal columns required")
    aa, bb = (a * a).sum(0), (b * b).sum(0)
    if np.any(aa <= 0) or np.any(bb <= 0):
        raise ValueError("zero modal vector cannot be matched")
    return np.clip((a.T @ b) ** 2 / np.outer(aa, bb), 0, 1)


def compare(coarse, fine):
    mac = assurance(coarse["mode_samples"], fine["mode_samples"])
    rows, cols = linear_sum_assignment(-mac)
    if not np.array_equal(rows, np.arange(len(coarse["omega"]))):
        raise ValueError("incomplete modal assignment")
    return {
        "matched_columns": cols.tolist(),
        "matched_assurance": mac[rows, cols].tolist(),
        "sorted_frequency_relative_change": (
            coarse["omega"] / fine["omega"] - 1
        ).tolist(),
        "matched_frequency_relative_change": (
            coarse["omega"] / fine["omega"][cols] - 1
        ).tolist(),
        "first8_participation_relative_l1": (
            abs(coarse["gains"][:, :8] - fine["gains"][:, cols[:8]]).sum(1)
            / np.maximum(abs(fine["gains"][:, cols[:8]]).sum(1), 1e-10)
        ).tolist(),
    }


def locally_stable(pair, spectral_errors):
    # Chosen as a local diagnostic before running levels3/4, not a release gate.
    if not all(
        np.isfinite(values).all()
        for values in (
            pair["matched_frequency_relative_change"],
            pair["matched_assurance"],
            pair["first8_participation_relative_l1"],
            spectral_errors,
        )
    ):
        return False
    return bool(
        np.max(np.abs(pair["matched_frequency_relative_change"][:8])) <= 0.01
        and min(pair["matched_assurance"][:8]) >= 0.98
        and max(pair["first8_participation_relative_l1"]) <= 0.05
        and max(spectral_errors) <= 0.05
    )


def run(args):
    args.output.mkdir(parents=True)
    points = np.array(
        list(
            itertools.product(np.linspace(0.1, 1, 10), (0.1, 0.5, 0.9), (0.1, 0.5, 0.9))
        )
    )
    levels, records, pairs, waves = [], [], [], {}
    for level in (1, 2, 3, 4):
        start = time.monotonic()
        modes = pilot.solve(
            pilot.DEV_SHAPES[0],
            pilot.DEV_CONTACTS,
            refinement=level,
            mode_count=12,
            sample_points=points,
        )
        elapsed = time.monotonic() - start
        np.savez(args.output / f"level-{level}.npz", **modes)
        levels.append(modes)
        for contact in range(4):
            wave = pilot.render_wave(
                *pilot.physical_modes(
                    modes["omega"][:8], modes["gains"][contact, :8], 0.18, 64e9, 2230
                )
            )
            waves[level, contact] = wave
            wavfile.write(
                args.output / f"level-{level}-contact-{contact}.wav", pilot.RATE, wave
            )
        record = {
            "level": level,
            "dofs": int(modes["dofs"]),
            "seconds": elapsed,
            "max_residual": modes["max_residual"],
            "peak_rss_kib": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss,
        }
        records.append(record)
        if level > 1:
            pair = compare(levels[-2], modes)
            pair["from_level"], pair["to_level"] = level - 1, level
            pair["audio_metrics"] = [
                diagnostics.audio_metrics(waves[level, c], waves[level - 1, c])
                for c in range(4)
            ]
            pair["locally_stable"] = locally_stable(
                pair, [m["spectrum"] for m in pair["audio_metrics"]]
            )
            pairs.append(pair)
        pilot.storage.save(
            args.output / "progress.json", {"levels": records, "pairs": pairs}
        )
        print(
            "fidelity level",
            record,
            "stable",
            pairs[-1]["locally_stable"] if pairs else None,
            flush=True,
        )
    final = levels[-1]
    truncation, neural_rows, comparisons = [], [], []
    for c in range(4):
        full = pilot.render_wave(
            *pilot.physical_modes(final["omega"], final["gains"][c], 0.18, 64e9, 2230)
        )
        wavfile.write(
            args.output / f"level-4-twelve-modes-contact-{c}.wav", pilot.RATE, full
        )
        truncation.append(diagnostics.audio_metrics(full, waves[4, c]))
        with np.load(
            args.neural / f"case-00-contact-{c}.npz", allow_pickle=False
        ) as saved:
            omega, gains = saved["omega"].copy(), saved["gains"].copy()
        wave = pilot.render_wave(*pilot.physical_modes(omega, gains, 0.18, 64e9, 2230))
        neural_rows.append(
            {
                "frequency_relative_errors": (omega / final["omega"][:8] - 1).tolist(),
                "input_sha256": pilot.integrity.sha256(
                    args.neural / f"case-00-contact-{c}.npz"
                ),
                **diagnostics.audio_metrics(waves[4, c], wave),
            }
        )
        comparisons.append(
            (
                f"contact-{c}-mesh-and-neural.wav",
                [waves[n, c] for n in (1, 2, 3, 4)] + [wave],
            )
        )
        comparisons.append((f"contact-{c}-eight-vs-twelve.wav", [waves[4, c], full]))
    gain = min(
        1.0, 0.98 / max(float(abs(w).max()) for _, parts in comparisons for w in parts)
    )
    for name, parts in comparisons:
        wavfile.write(
            args.output / name,
            pilot.RATE,
            np.concatenate(
                [v for w in parts for v in (w * gain, np.zeros(pilot.RATE // 2))]
            ).astype(np.float32),
        )
    result = {
        "levels": records,
        "pairs": pairs,
        "truncation_eight_vs_twelve": truncation,
        "neural_vs_level4_eight": neural_rows,
        "comparison_gain": gain,
        "shape": pilot.DEV_SHAPES[0],
        "sample_points": points.tolist(),
        "local_stability": pairs[-1]["locally_stable"],
        "scope": "one synthetic shape/four contacts; no continuum error bound, radiation/real-audio validation, family promotion or retraining",
        "script_sha256": pilot.integrity.sha256(Path(__file__)),
        "solver_script_sha256": pilot.integrity.sha256(Path(pilot.__file__)),
        "neural_inputs": str(args.neural.resolve()),
    }
    pilot.storage.save(args.output / "result.json", result)
    print(
        json.dumps(
            {
                "local_stability": result["local_stability"],
                "last_pair": pairs[-1],
                "truncation": truncation,
                "neural": neural_rows,
            },
            indent=2,
        ),
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--neural", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    run(args)


if __name__ == "__main__":
    main()
