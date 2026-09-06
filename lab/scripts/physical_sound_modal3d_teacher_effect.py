"""One fixed shared fit and audible old/new teacher-effect evaluation.

No checkpoint selection, parameter sweep or promotion. Training is a separate
process with a TRAIN-only manifest; standalone rendering reads no FEM or audio.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

import numpy as np
import physical_sound_modal3d_pilot as pilot
from scipy.io import wavfile


def train_projection(data, root):
    rows = [r for r in data["rows"] if r["role"] == "train"]
    if len(rows) != 36 or {tuple(r["shape"]) for r in rows} != set(pilot.TRAIN_SHAPES):
        raise ValueError("exact original TRAIN grid required")
    return {
        "rows": [
            {
                "role": "train",
                "file": str((root / r["file"]).resolve()),
                "sha256": r["sha256"],
            }
            for r in rows
        ],
        "scope": "TRAIN only; no DEV responses, mesh diagnostics or checkpoint selection",
    }


def stage(name, output, extra, traced=False):
    command = [
        sys.executable,
        str(Path(pilot.__file__).resolve()),
        name,
        "--output",
        str(output),
        *map(str, extra),
    ]
    if traced:
        command = [
            "strace",
            "-f",
            "-e",
            "trace=openat,connect",
            "-o",
            str(output.parent / f"{name}-access.log"),
            *command,
        ]
    env = os.environ.copy()
    env.pop("PYTHONPATH", None)
    env.update(OMP_NUM_THREADS="4", OPENBLAS_NUM_THREADS="4")
    subprocess.run(command, env=env, check=True)


def run(args):
    args.output.mkdir(parents=True)
    data = json.loads((args.data / "data.json").read_text())
    train_root = args.output / "train-input"
    train_root.mkdir()
    pilot.storage.save(train_root / "data.json", train_projection(data, args.data))
    stage("fit", args.output / "fit", ["--data", train_root], traced=True)
    stage("render", args.output / "render", ["--fit", args.output / "fit"], traced=True)
    for label, generated in (("old", args.baseline), ("new", args.output / "render")):
        stage(
            "evaluate",
            args.output / f"{label}-evaluation",
            ["--data", args.data, "--generated", generated],
        )
    old, new = [
        json.loads(
            (args.output / f"{label}-evaluation" / "evaluation.json").read_text()
        )
        for label in ("old", "new")
    ]
    summary = {
        key: {
            "old": old["summary"][key]["neural"],
            "neural": new["summary"][key]["neural"],
            "wins": sum(
                n["neural"][key] < o["neural"][key]
                for o, n in zip(old["rows"], new["rows"], strict=True)
            ),
        }
        for key in old["summary"]
    }
    reference_rows = {r["index"]: r for r in data["rows"] if r["role"] == "development"}
    parts = []
    for old_row, new_row in zip(old["rows"], new["rows"], strict=True):
        key = (old_row["shape_index"], old_row["contact_index"])
        if key != (new_row["shape_index"], new_row["contact_index"]):
            raise ValueError("evaluation ordering mismatch")
        index, ci = key
        name = f"case-{index:02d}-contact-{ci}"
        with np.load(
            args.data / reference_rows[index]["file"], allow_pickle=False
        ) as d:
            reference = pilot.render_wave(
                *pilot.physical_modes(d["omega"], d["gains"][ci], 0.18, 64e9, 2230)
            )
        waves = [reference]
        for root in (args.baseline, args.output / "render"):
            with np.load(root / (name + ".npz"), allow_pickle=False) as d:
                waves.append(
                    pilot.render_wave(
                        *pilot.physical_modes(d["omega"], d["gains"], 0.18, 64e9, 2230)
                    )
                )
        parts.append((name, waves))
    gain = min(
        1.0, 0.98 / max(float(abs(w).max()) for _, waves in parts for w in waves)
    )
    for name, waves in parts:
        wavfile.write(
            args.output / (name + "-teacher-effect.wav"),
            pilot.RATE,
            np.concatenate(
                [v for w in waves for v in (w * gain, np.zeros(pilot.RATE // 2))]
            ).astype(np.float32),
        )
    result = {
        "summary": summary,
        "all_five_mean_metrics_improve_over_old": pilot.beats_baseline(summary, "old"),
        "new_model_baseline_decision": new["decision"],
        "locally_stable_bodies": data["stable_bodies"],
        "total_bodies": len(data["rows"]),
        "comparison_gain": gain,
        "data_sha256": pilot.integrity.sha256(args.data / "data.json"),
        "baseline_render_sha256": pilot.integrity.sha256(args.baseline / "render.json"),
        "fit_sha256": pilot.integrity.sha256(args.output / "fit" / "fit.json"),
        "script_sha256": pilot.integrity.sha256(Path(__file__)),
        "scope": "mesh4 reference -> old neural -> corrected-teacher neural; one unchanged-design fit; exposed DEV, no pristine holdout, no radiation/real-audio/material admission",
    }
    pilot.storage.save(args.output / "effect.json", result)
    print(json.dumps(result, indent=2), flush=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "baseline", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    run(args)


if __name__ == "__main__":
    main()
