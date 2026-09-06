"""TRAIN-only pole/field discriminator, no new optimization or generation claim.

All six TRAIN bodies at their first fixed contact. Teacher mode counts/ranks
are intentionally supplied to this diagnostic. Four combinations isolate learned
poles and learned signed fields at a common count; these are oracle-assisted
controls, NEVER standalone predictions. Actual source-free WAVs stay untouched.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import physical_sound_objectfolder2 as teacher
import physical_sound_objectfolder2_shared as shared
import torch
from physical_sound_sonicgauss_waveform_fit import audio_metrics
from safetensors.torch import load_file
from scipy.io import wavfile


def diagnose(args):
    rows = json.loads((args.data / "train.json").read_text())["rows"]
    if (
        len(rows) != 6
        or {r["object_id"] for r in rows} != shared.TRAIN
        or any(r["role"] != "train" for r in rows)
    ):
        raise ValueError("exact TRAIN-only projection required")
    info = json.loads((args.fit / "fit.json").read_text())
    if teacher.sha(args.fit / "model.safetensors") != info["weights_sha256"]:
        raise ValueError("weights changed")
    model = shared.SharedStudent()
    model.load_state_dict(load_file(args.fit / "model.safetensors"), strict=True)
    model.eval().requires_grad_(False)
    args.output.mkdir(parents=True)
    results = []
    for row in rows:
        path = args.data / row["file"]
        if teacher.sha(path) != row["sha256"]:
            raise ValueError("teacher changed")
        with np.load(path, allow_pickle=False) as saved:
            data = dict(saved)
        count = len(data["frequency"])
        rank = ((torch.arange(count) + 0.5) / count)[:, None]
        with torch.no_grad():
            encoded = model.encode(
                torch.tensor(data["cloud"][None]), torch.tensor(data["features"][None])
            ).expand(count, -1)
            poles = model.mode_parameters(encoded, rank).exp().numpy().astype(float)
            field = (
                (
                    model.signed_field(
                        encoded,
                        rank,
                        torch.tensor(data["contacts"][:1]).expand(count, -1),
                    ).sinh()
                    * model.gain_scale
                )
                .numpy()
                .T.astype(float)
            )
        waves = {}
        for name, frequency, damping, gains in (
            ("teacher", data["frequency"], data["damping"], data["gains"][0]),
            ("learned_poles", poles[:, 0], poles[:, 1], data["gains"][0]),
            ("learned_field", data["frequency"], data["damping"], field),
            ("both_oracle_count", poles[:, 0], poles[:, 1], field),
        ):
            waves[name] = teacher.waveform(gains, frequency, damping, [1, 1, 1])
        metrics = {
            name: audio_metrics(waves["teacher"], wave)
            for name, wave in waves.items()
            if name != "teacher"
        }
        gain = min(1.0, 0.5 / max(float(abs(w).max()) for w in waves.values()))
        gallery = np.concatenate([w * gain for w in waves.values()]).astype(np.float32)
        if not np.isfinite(gallery).all():
            raise ValueError("nonfinite diagnostic")
        name = f"train-{row['object_id']}-teacher-poles-field-both.wav"
        wavfile.write(args.output / name, teacher.RATE, gallery)
        result = {
            "object_id": row["object_id"],
            "metrics": metrics,
            "oracle_modes": count,
            "modal_gain_rms_ratio": float(
                np.sqrt(
                    np.mean(field**2) / np.mean(data["gains"][0].astype(float) ** 2)
                )
            ),
            "comparison": name,
            "common_gain": gain,
            "sha256": teacher.sha(args.output / name),
        }
        results.append(result)
        print(json.dumps(result), flush=True)
    summary = {
        kind: {
            metric: float(np.mean([r["metrics"][kind][metric] for r in results]))
            for metric in ("spectrum", "envelope", "level")
        }
        for kind in ("learned_poles", "learned_field", "both_oracle_count")
    }
    shared.write_json(
        args.output / "diagnostic.json",
        {
            "rows": results,
            "summary": summary,
            "weights_sha256": info["weights_sha256"],
            "scope": "posthoc six TRAIN first contacts; oracle-count/rank pole-field swaps, not new network or source-free generation",
        },
    )
    print(json.dumps(summary, indent=2), flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--data", type=Path, required=True)
    parser.add_argument("--fit", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    torch.set_num_threads(4)
    diagnose(parser.parse_args())
