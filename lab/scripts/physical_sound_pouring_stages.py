"""Report-only flow-time error and frozen-field splice discriminator.

No training or target waveform enters sampling. Spliced trajectories are
counterfactuals, not samples certified to follow either learned distribution.
"""

from __future__ import annotations

import argparse
import hashlib
from pathlib import Path

import numpy as np
import physical_sound_pouring_compare as compare
import physical_sound_pouring_pilot as p
import soundfile as sf
import torch


class StagedField(torch.nn.Module):
    def __init__(self, early, late, switch_time=0.25):
        super().__init__()
        if not 0 <= switch_time <= 1:
            raise ValueError("invalid switch time")
        self.early, self.late, self.switch_time = early, late, switch_time

    def forward(self, x, time, controls):
        if not torch.all(time == time[0]):
            raise ValueError("stage probe requires a single integration time")
        return (self.early if float(time[0]) < self.switch_time else self.late)(
            x, time, controls
        )


def endpoint_oracle(target, xt, time):
    """Reference-only control with the exact endpoint, never neural inference."""
    return (target - xt) / (1 - time)


@torch.inference_mode()
def flow_errors(row, models):
    patches = [compare.phase_crop(row, phase) for phase in ("first", "middle")]
    target = torch.from_numpy(np.stack([x[0] for x in patches])[:, None]).cuda()
    c = torch.from_numpy(np.stack([x[1] for x in patches])).cuda()
    wrong = c.clone()
    wrong[:, 4] = c.flip(0)[:, 4]
    noise = torch.randn(
        target.shape,
        generator=torch.Generator(device="cuda").manual_seed(2718),
        device="cuda",
    )
    expected = target - noise
    rows = []
    for t in (0, 0.01, 0.05, 0.1, 0.25, 0.5, 0.75, 0.95):
        xt = (1 - t) * noise + t * target
        time = torch.full((2,), t, dtype=torch.float32, device="cuda")
        oracle = endpoint_oracle(target, xt, t)
        wrong_oracle = endpoint_oracle(target.flip(0), xt, t)
        correct = float((oracle - expected).square().mean())
        swapped = float((wrong_oracle - expected).square().mean())
        rows.append(
            {
                "item_id": row["item_id"],
                "container_id": row["container_id"],
                "kind": "two-endpoint-oracle",
                "time": t,
                "correct_velocity_mse": correct,
                "swapped_velocity_mse": swapped,
                "phase_advantage": swapped - correct,
            }
        )
        for kind, model in models.items():
            correct = float((model(xt, time, c) - expected).square().mean())
            swapped = float((model(xt, time, wrong) - expected).square().mean())
            rows.append(
                {
                    "item_id": row["item_id"],
                    "container_id": row["container_id"],
                    "kind": kind,
                    "time": t,
                    "correct_velocity_mse": correct,
                    "swapped_velocity_mse": swapped,
                    "phase_advantage": swapped - correct,
                }
            )
    return rows


def run(source, base, candidate, output):
    source, base, candidate, output = (
        x.resolve() for x in (source, base, candidate, output)
    )
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[2]):
        raise ValueError("use a fresh external output")
    torch.set_num_threads(4)
    rows, _ = p.load_source(source)
    loaded = {
        k: compare.load_model(d, "cuda")
        for k, d in (("base", base), ("matched", candidate))
    }
    expected_ids = [r["item_id"] for r in rows if r["role"] == "train"]
    for _, m in loaded.values():
        if (
            m["train_ids"] != expected_ids
            or m["source_sha256"]
            != hashlib.sha256((source / "source.json").read_bytes()).hexdigest()
        ):
            raise ValueError("source/training exposure mismatch")
    models = {k: m for k, (m, _) in loaded.items()}
    report = {
        "status": "complete",
        "scope": "disclosed stage ablation, not generalization/admission",
        "models": {k: m for k, (_, m) in loaded.items()},
        "flow_errors": [],
        "rows": [],
    }
    output.mkdir(parents=True)
    for row in compare.select_rows(rows, True):
        report["flow_errors"].extend(flow_errors(row, models))
    fields = {
        **models,
        "base-early": StagedField(models["base"], models["matched"]),
        "base-late": StagedField(models["matched"], models["base"]),
    }
    selected = {}
    for row in rows:
        if row["role"] == "unseen_container":
            selected.setdefault(row["container_id"], row)
    tags = {
        "status": "complete",
        "seconds": p.SAMPLES / p.RATE,
        "cases": [],
        "rows": [],
        "controls": [],
    }
    preview = []
    for idx, row in enumerate(selected.values()):
        for phase in ("first", "middle"):
            oracle, c, real, offset = compare.phase_crop(row, phase)
            pending = [("real", None, real), ("oracle", 314, p.decode(oracle, 314))]
            for seed in (314, 2718, 1618):
                for kind, model in fields.items():
                    pending.append(
                        (kind, seed, p.decode(p.sample(model, c, seed), 314))
                    )
            for kind, seed, wave in pending:
                if not np.isfinite(wave).all() or abs(wave).max() > 0.98:
                    raise ValueError("invalid waveform; no automatic gain correction")
                key = f"{idx}-{phase}-{kind}-{seed}"
                path = output / (key + ".wav")
                sf.write(path, wave, p.RATE, subtype="PCM_16")
                record = {
                    "item_id": row["item_id"],
                    "container_id": row["container_id"],
                    "phase": phase,
                    "kind": kind,
                    "seed": seed,
                    "decoder_seed": 314,
                    "switch_time": 0.25,
                    "controls": c.tolist(),
                    "start_sample": offset,
                    "wav": str(path),
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                    "gain": 1,
                    **p.metrics(wave, real),
                }
                report["rows"].append(record)
                tags["cases"].append({"id": key, "diagnostic_id": "water-pour"})
                tags["rows"].append({**record, "case": len(tags["cases"]) - 1})
                if idx == 0 and (kind == "real" or seed == 2718):
                    preview.extend([wave, np.zeros(p.RATE // 2)])
            print(
                {"object": row["container_id"], "phase": phase, "clips": len(pending)},
                flush=True,
            )
    path = output / "comparison.wav"
    sf.write(path, np.concatenate(preview), p.RATE, subtype="PCM_16")
    report["comparison"] = {
        "wav": str(path),
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
        "gain": 1,
        "order": "first glass recording; first then middle; real/base/matched/base-early/base-late; generator2718, decoder314",
    }
    p.save_json(output / "result.json", report)
    p.save_json(output / "tag-input.json", tags)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("source", "base", "candidate", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run(args.source, args.base, args.candidate, args.output)
