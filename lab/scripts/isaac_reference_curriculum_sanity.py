#!/usr/bin/env python3
"""Validate hash-derived multi-clip/phase reset selection in Isaac."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import traceback
from collections import Counter
from pathlib import Path

from isaaclab.app import AppLauncher


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--clip-id", action="append", required=True)
    parser.add_argument("--horizon", type=int, required=True)
    parser.add_argument("--num-envs", type=int, default=16)
    parser.add_argument("--reset-rounds", type=int, default=8)
    parser.add_argument("--rng-run-root-hex", required=True)
    parser.add_argument("--output", type=Path, required=True)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if (
        args.horizon <= 0
        or args.num_envs <= 0
        or args.reset_rounds <= 0
        or len(args.rng_run_root_hex) != 64
        or len(args.clip_id) != len(set(args.clip_id))
    ):
        raise ValueError("invalid reference curriculum sanity configuration")
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(args.usd.resolve())
    simulation_app = None
    environment = None
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch

        from next_lab.isaac_reference_env import (
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
            _select_curriculum_episode,
        )

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = args.num_envs
        cfg.sim.device = args.device
        cfg.seed = 120_812
        cfg.fixed_horizon_motor_ticks = args.horizon
        cfg.eligible_clip_ids = tuple(args.clip_id)
        cfg.phase_randomization = True
        cfg.rng_run_root_hex = args.rng_run_root_hex
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.profile.resolve()),
            corpus_root=str(args.corpus_root.resolve()),
            gate_report_path=str(args.gate_report.resolve()),
        )
        environment.reset_episode_sequence()
        clip_counts: Counter[str] = Counter()
        phase_counts: Counter[str] = Counter()
        run_root = bytes.fromhex(args.rng_run_root_hex)
        frame_counts = tuple(clip.frame_count for clip in environment.reference_clips)
        checked = 0
        for episode_ordinal in range(args.reset_rounds):
            observation, _ = environment.reset()
            if not torch.isfinite(observation["policy"]).all():
                raise RuntimeError("curriculum reset produced a non-finite observation")
            clip_indices = environment._clip_index.detach().cpu().tolist()
            start_frames = environment._cursor.detach().cpu().tolist()
            terminal_frames = environment._terminal_frame.detach().cpu().tolist()
            for vector_slot, actual in enumerate(
                zip(clip_indices, start_frames, terminal_frames, strict=True)
            ):
                expected = _select_curriculum_episode(
                    run_root=run_root,
                    episode_ordinal=episode_ordinal,
                    vector_slot=vector_slot,
                    clip_frame_counts=frame_counts,
                    horizon_motor_ticks=args.horizon,
                )
                if actual != expected:
                    raise RuntimeError("Isaac curriculum reset selection mismatch")
                clip_index, start_frame, terminal_frame = actual
                clip_id = args.clip_id[clip_index]
                clip_counts[clip_id] += 1
                phase_counts[f"{clip_id}:{start_frame}"] += 1
                if terminal_frame - start_frame != args.horizon:
                    raise RuntimeError("curriculum terminal frame mismatch")
                checked += 1
        if len(clip_counts) != len(args.clip_id) or len(phase_counts) <= len(clip_counts):
            raise RuntimeError("curriculum sanity did not cover every clip and varied phases")
        report = {
            "schema_version": 1,
            "check": "MOTOR-REFERENCE-ENV-P1-CURRICULUM-SELECTION-SANITY",
            "status": "PASS",
            "claim": "HashDerivedTrainClipPhaseSelectionOnly",
            "profile_sha256": _sha256(args.profile.resolve()),
            "descriptor_sha256": _sha256(args.descriptor.resolve()),
            "usd_sha256": _sha256(args.usd.resolve()),
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "rng_run_root_hex": args.rng_run_root_hex,
            "eligible_clip_ids": args.clip_id,
            "horizon_motor_ticks": args.horizon,
            "num_envs": args.num_envs,
            "reset_rounds": args.reset_rounds,
            "checked_episode_selections": checked,
            "clip_selection_counts": dict(sorted(clip_counts.items())),
            "distinct_clip_phase_count": len(phase_counts),
            "optimizer_steps": 0,
            "training_runs": 0,
            "learned_policy_claim": False,
        }
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        temporary = output.with_suffix(output.suffix + ".tmp")
        temporary.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        temporary.replace(output)
        print(json.dumps(report, indent=2, sort_keys=True), flush=True)
    except BaseException:
        traceback.print_exc()
        raise
    finally:
        if environment is not None:
            environment.close()
        if simulation_app is not None:
            simulation_app.close()


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    main()
