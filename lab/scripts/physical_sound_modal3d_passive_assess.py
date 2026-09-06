"""Coupled passive-field sound versus FEM, convex interpolation and old NN."""

from __future__ import annotations

import argparse
import itertools
import json
from pathlib import Path

import numpy as np
import physical_sound_modal3d_impact as impact
import physical_sound_modal3d_pilot as pilot
from scipy.interpolate import RegularGridInterpolator
from scipy.io import wavfile


def port_matrix(amplitudes):
    return np.einsum("pm,qm->mpq", amplitudes, amplitudes)


def interpolate(train):
    axes = [sorted({s[i] for s in pilot.TRAIN_SHAPES}) for i in range(3)]
    caxes = [sorted(set(pilot.TRAIN_CONTACTS[:, i])) for i in range(2)]
    if len(train) != 36 or {tuple(r["shape"]) for r in train} != set(
        pilot.TRAIN_SHAPES
    ):
        raise ValueError("exact complete TRAIN grid required")
    omega = np.empty((4, 3, 3, 8))
    matrix = np.empty((4, 3, 3, 3, 3, 8, 2, 2))
    for row in train:
        idx = tuple(axes[i].index(float(row["shape"][i])) for i in range(3))
        omega[idx] = row["omega"]
        matrices = np.array([port_matrix(row["port_modes"][[c, -1]]) for c in range(9)])
        matrix[idx] = matrices.reshape(3, 3, 8, 2, 2)
    return RegularGridInterpolator(axes, omega), RegularGridInterpolator(
        axes + caxes, matrix
    )


def coupled(omega, matrix, config):
    w = pilot.physical_modes(omega, np.ones(8), 0.18, config["target_young"], 2230)[0]
    physical_matrix = matrix / (2230 * 0.18**3)
    collision = impact.collision(config, w, physical_matrix[:, 0, 0])
    wave = impact.render_force(w, physical_matrix[:, 0, 1], collision)
    return wave, collision


def assess(args):
    data = json.loads((args.data / "data.json").read_text())
    rendered = json.loads((args.generated / "render.json").read_text())
    train = []
    dev = {}
    for row in data["rows"]:
        path = args.data / row["file"]
        if pilot.integrity.sha256(path) != row["sha256"]:
            raise ValueError("teacher changed")
        with np.load(path, allow_pickle=False) as d:
            record = {k: d[k].copy() for k in d.files}
        if row["role"] == "train":
            train.append(record)
        else:
            dev[row["index"]] = record
    f, m = interpolate(train)
    primary = [r for r in rendered["rows"] if r["name"].startswith("case-")]
    if len(primary) != 48 or {
        (r["shape_index"], r["contact_index"]) for r in primary
    } != set(itertools.product(range(12), range(4))):
        raise ValueError("complete48-case primary render required")
    args.output.mkdir(parents=True)
    rows = []
    for row in rendered["rows"]:
        i, c = row["shape_index"], row["contact_index"]
        target = dev[i]
        config = row["config"]
        refmatrix = port_matrix(target["port_modes"][[c, -1]])
        reference, refcontact = coupled(target["omega"], refmatrix, config)
        with np.load(args.generated / (row["name"] + ".npz"), allow_pickle=False) as n:
            nw, nm = n["omega"].copy(), port_matrix(n["amplitudes"])
        iw = f(target["shape"][None])[0]
        im = m(np.r_[target["shape"], target["contacts"][c]][None])[0]
        predictions = {"interpolation": (iw, im), "neural": (nw, nm)}
        metrics = {"name": row["name"], "shape_index": i, "contact_index": c}
        waves = {}
        for label, (omega, matrix) in predictions.items():
            wave, contact = coupled(omega, matrix, config)
            waves[label] = wave
            metrics[label] = {
                "frequency_relative_mean": float(
                    np.mean(abs(omega / target["omega"] - 1))
                ),
                "port_matrix_relative_frobenius": float(
                    np.linalg.norm(matrix - refmatrix)
                    / max(np.linalg.norm(refmatrix), 1e-10)
                ),
                "contact_self_relative_l1": float(
                    abs(matrix[:, 0, 0] - refmatrix[:, 0, 0]).sum()
                    / max(abs(refmatrix[:, 0, 0]).sum(), 1e-10)
                ),
                "cross_relative_l1": float(
                    abs(matrix[:, 0, 1] - refmatrix[:, 0, 1]).sum()
                    / max(abs(refmatrix[:, 0, 1]).sum(), 1e-10)
                ),
                "impulse_relative_error": abs(
                    contact["impulse"] / refcontact["impulse"] - 1
                ),
                "energy_balance_error": contact["energy_balance_error"],
                **impact.diagnostics.audio_metrics(reference, wave),
            }
        with np.load(
            args.legacy / f"case-{i:02d}-contact-{c}.npz", allow_pickle=False
        ) as old:
            if not np.array_equal(nw, old["omega"]):
                raise ValueError("frozen frequency prediction changed")
            legacy = impact.render_force(
                *pilot.physical_modes(
                    old["omega"].astype(float),
                    old["gains"].astype(float),
                    0.18,
                    config["target_young"],
                    2230,
                ),
                impact.collision(config),
            )
        metrics["legacy"] = impact.diagnostics.audio_metrics(reference, legacy)
        path = args.generated / (row["name"] + ".wav")
        if pilot.integrity.sha256(path) != row["wav_sha256"]:
            raise ValueError("neural WAV changed")
        sr, saved = wavfile.read(path)
        if sr != pilot.RATE or not np.array_equal(
            saved, waves["neural"] * rendered["gain"]
        ):
            raise ValueError("standalone coupled replay differs")
        parts = [reference, waves["interpolation"], legacy, waves["neural"]]
        gain = min(1.0, 0.98 / max(float(abs(w).max()) for w in parts))
        wavfile.write(
            args.output / (row["name"] + "-comparison.wav"),
            pilot.RATE,
            np.concatenate(
                [v for w in parts for v in (w * gain, np.zeros(pilot.RATE // 2))]
            ).astype(np.float32),
        )
        metrics["comparison_gain"] = gain
        rows.append(metrics)
    held = [r for r in rows if r["name"].startswith("case-")]
    summary = {
        k: {
            label: float(np.mean([r[label][k] for r in held]))
            for label in ("interpolation", "neural")
        }
        for k in held[0]["neural"]
    }
    for key in ("spectrum", "envelope", "level"):
        summary[key]["legacy"] = float(np.mean([r["legacy"][key] for r in held]))
        summary[key]["wins_over_legacy"] = sum(
            r["neural"][key] < r["legacy"][key] for r in held
        )
        summary[key]["wins_over_interpolation"] = sum(
            r["neural"][key] < r["interpolation"][key] for r in held
        )
    improved = all(
        summary[k]["neural"] < summary[k][label]
        for k in ("spectrum", "envelope", "level")
        for label in ("legacy", "interpolation")
    )
    improved &= (
        summary["port_matrix_relative_frobenius"]["neural"]
        < summary["port_matrix_relative_frobenius"]["interpolation"]
    )
    result = {
        "rows": rows,
        "summary": summary,
        "frequency_preserved_exact": True,
        "quality_advantage": bool(improved),
        "script_sha256": pilot.integrity.sha256(Path(__file__)),
        "data_sha256": pilot.integrity.sha256(args.data / "data.json"),
        "render_sha256": pilot.integrity.sha256(args.generated / "render.json"),
        "scope": "FEM -> convex TRAIN interpolation -> legacy one-way NN -> passive coupled NN;48 primary cases,11 separate controls; exposedDEV,notpressure/realism;passivity does not imply quality",
    }
    pilot.storage.save(args.output / "assessment.json", result)
    print(
        json.dumps({"summary": summary, "quality_advantage": bool(improved)}, indent=2),
        flush=True,
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("data", "generated", "legacy", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists() or args.output.resolve().is_relative_to(
        Path(__file__).resolve().parents[2]
    ):
        raise ValueError("new external output required")
    assess(args)


if __name__ == "__main__":
    main()
