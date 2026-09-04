#!/usr/bin/env python3
"""Report-only paired noise discriminator for the closed final CPU V5 policy."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import numpy as np
import torch
from next_lab.canonical_ppo import CanonicalVecEnv
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from rsl_rl.modules import ActorCritic

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    run = require_external_path(args.run, ROOT, label="closed run")
    output = require_external_path(args.output, ROOT, label="audit", must_exist=False)
    if output.exists():
        raise ValueError("audit output must be fresh")
    manifest = json.loads((run / "run-manifest.json").read_text())
    if manifest["status"] != "completed":
        raise ValueError("audit requires a completed run")
    checkpoint = run / "model_249.pt"
    for path in (checkpoint, run / "evaluation.json", run / "metrics.jsonl"):
        if sha256_file(path) != manifest["artifacts"][path.name]:
            raise ValueError(f"closed artifact hash mismatch: {path.name}")
    descriptor_path = Path(manifest["descriptor_path"])
    headless = Path(manifest["headless_path"])
    for path, key in (
        (descriptor_path, "descriptor_sha256"),
        (headless, "headless_sha256"),
    ):
        if sha256_file(path) != manifest[key]:
            raise ValueError(f"input hash mismatch: {key}")
    descriptor = json.loads(descriptor_path.read_text())
    state = torch.load(checkpoint, map_location="cpu", weights_only=True)[
        "model_state_dict"
    ]
    cases = []
    torch.set_num_threads(1)
    for multiplier in (0.0, 1.0, 0.25):
        env = CanonicalVecEnv(
            headless,
            descriptor,
            num_envs=16,
            shards=1,
            run_root="21" * 32,
            device="cpu",
        )
        generator = torch.Generator().manual_seed(2001)
        active = np.ones(16, dtype=bool)
        records = [
            {
                "slot": slot,
                "ticks": 0,
                "single_support_ticks": [0, 0],
                "moving_single_support_ticks": [0, 0],
                "longest_single_support_run": [0, 0],
                "contact_patterns": {},
                "terminal": None,
            }
            for slot in range(16)
        ]
        current = np.zeros((16, 2), dtype=int)
        try:
            policy = ActorCritic(
                env.get_observations(),
                {"policy": ["policy"], "critic": ["policy"]},
                23,
                **manifest["profile"]["policy"],
            )
            policy.load_state_dict(state, strict=True)
            policy.eval()
            with torch.inference_mode():
                for _ in range(1200):
                    action = policy.act_inference(env.get_observations())
                    action += (
                        multiplier
                        * policy.std
                        * torch.randn((16, 23), generator=generator)
                    )
                    env.step(action)
                    for slot in np.flatnonzero(active):
                        result = env.last_steps[slot]
                        record = records[slot]
                        record["ticks"] += 1
                        flags = result.contact_flags
                        pattern = "".join(str(int(flag)) for flag in flags)
                        record["contact_patterns"][pattern] = (
                            record["contact_patterns"].get(pattern, 0) + 1
                        )
                        for side in range(2):
                            single = bool(flags[side] and not flags[1 - side])
                            current[slot, side] = (
                                current[slot, side] + 1 if single else 0
                            )
                            record["single_support_ticks"][side] += int(single)
                            record["moving_single_support_ticks"][side] += int(
                                single and np.any(result.command_raw)
                            )
                            record["longest_single_support_run"][side] = max(
                                record["longest_single_support_run"][side],
                                int(current[slot, side]),
                            )
                        record["final_forward_m"] = float(
                            result.root_position_micrometres[2] / 1e6
                        )
                        if result.terminated or result.truncated:
                            record["terminal"] = (
                                result.terminal_reason_id or "time-limit"
                            )
                            active[slot] = False
                    if not active.any():
                        break
            cases.append({"noise_multiplier": multiplier, "episodes": records})
        finally:
            env.close()
    report = {
        "schema": "nextengine.canonical-walking-exploration-audit.v1",
        "claim": "report-only first episodes; no training, selection or quality admission",
        "source_run_manifest_sha256": sha256_file(run / "run-manifest.json"),
        "checkpoint_sha256": sha256_file(checkpoint),
        "tool_sha256": sha256_file(Path(__file__)),
        "descriptor_sha256": sha256_file(descriptor_path),
        "headless_sha256": sha256_file(headless),
        "common_noise_seed": 2001,
        "shards": 1,
        "slots": 16,
        "maximum_ticks": 1200,
        "cases": cases,
    }
    atomic_write_json(output, report)
    for case in cases:
        print(
            json.dumps(
                {
                    "noise_multiplier": case["noise_multiplier"],
                    "mean_ticks": float(
                        np.mean([item["ticks"] for item in case["episodes"]])
                    ),
                    "single_support_ticks": np.sum(
                        [item["single_support_ticks"] for item in case["episodes"]],
                        axis=0,
                    ).tolist(),
                    "moving_single_support_ticks": np.sum(
                        [
                            item["moving_single_support_ticks"]
                            for item in case["episodes"]
                        ],
                        axis=0,
                    ).tolist(),
                },
                sort_keys=True,
            )
        )


if __name__ == "__main__":
    main()
