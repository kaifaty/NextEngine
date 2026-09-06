"""Fixed family-wide mesh3/4 experiment; retains failures, not adaptive fitting.

TRAIN uses mesh3, DEV uses mesh4, with paired numerical checks for every body.
Roles, shape/contact grids and subsequent neural training settings stay fixed.
This is synthetic vibration, not real-object sound validation.
"""

from __future__ import annotations

import argparse
import itertools
import multiprocessing
import time
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np
import physical_sound_modal3d_convergence as convergence
import physical_sound_modal3d_pilot as pilot
from scipy.io import wavfile


def one_body(job):
    role, index, shape, contacts, output = job
    start = time.monotonic()
    points = np.array(
        list(
            itertools.product(np.linspace(0.1, 1, 10), (0.1, 0.5, 0.9), (0.1, 0.5, 0.9))
        )
    )
    levels = [
        pilot.solve(shape, contacts, refinement=level, sample_points=points)
        for level in (3, 4)
    ]
    pair = convergence.compare(*levels)
    errors = []
    for ci in range(len(contacts)):
        waves = [
            pilot.render_wave(
                *pilot.physical_modes(d["omega"], d["gains"][ci], 0.18, 64e9, 2230)
            )
            for d in levels
        ]
        errors.append(convergence.diagnostics.audio_metrics(waves[1], waves[0]))
        # Full same-gain numerical comparison for every body/contact, including failures.
        if not all(np.isfinite(w).all() for w in waves):
            raise ValueError("nonfinite teacher sound")
        gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in waves))
        wavfile.write(
            output / f"{role}-{index:02d}-contact-{ci}-mesh-pair.wav",
            pilot.RATE,
            np.concatenate(
                [v for w in waves for v in (w * gain, np.zeros(pilot.RATE // 2))]
            ).astype(np.float32),
        )
    pair["audio_metrics"] = errors
    pair["locally_stable"] = convergence.locally_stable(
        pair, [r["spectrum"] for r in errors]
    )
    for level, modes in zip((3, 4), levels, strict=True):
        np.savez(
            output / f"{role}-{index:02d}-mesh{level}.npz",
            shape=shape,
            contacts=contacts,
            **modes,
        )
    used_level = 3 if role == "train" else 4
    selected = levels[used_level - 3]
    name = f"{role}-{index:02d}.npz"
    np.savez(output / name, shape=shape, contacts=contacts, **selected)
    return {
        "role": role,
        "index": index,
        "shape": shape,
        "file": name,
        "sha256": pilot.integrity.sha256(output / name),
        "refinement": used_level,
        "max_residual": selected["max_residual"],
        "pair": pair,
        "seconds": time.monotonic() - start,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    args.output.mkdir(parents=True)
    jobs = [
        (role, i, shape, contacts, args.output)
        for role, shapes, contacts in (
            ("train", pilot.TRAIN_SHAPES, pilot.TRAIN_CONTACTS),
            ("development", pilot.DEV_SHAPES, pilot.DEV_CONTACTS),
        )
        for i, shape in enumerate(shapes)
    ]
    rows = []
    with ProcessPoolExecutor(
        max_workers=3, mp_context=multiprocessing.get_context("spawn")
    ) as pool:
        for row in pool.map(one_body, jobs):
            rows.append(row)
            pilot.storage.save(args.output / "progress.json", {"rows": rows})
            print(
                "paired teacher",
                row["role"],
                row["index"],
                "stable",
                row["pair"]["locally_stable"],
                "seconds",
                round(row["seconds"], 2),
                flush=True,
            )
    pilot.storage.save(
        args.output / "data.json",
        {
            "rows": rows,
            "solver": "scikit-fem12.0.2 vector P2 tetrahedra, clamped x=0; E=rho=L=1",
            "train_refinement": 3,
            "development_refinement": 4,
            "stable_bodies": sum(r["pair"]["locally_stable"] for r in rows),
            "script_sha256": pilot.integrity.sha256(Path(__file__)),
            "solver_script_sha256": pilot.integrity.sha256(Path(pilot.__file__)),
            "scope": "all48 original shapes retained regardless of local stability; no role changes; own synthetic vibration, not continuum or real-audio validation",
        },
    )


if __name__ == "__main__":
    main()
