#!/usr/bin/env python3
"""Exercise exact-reference reset, observation, reward, and terminals in Isaac."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import traceback
from pathlib import Path

from isaaclab.app import AppLauncher


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--clip-id", default="cmu104-start-right")
    parser.add_argument("--start-frame", type=int, default=0)
    parser.add_argument("--horizon", type=int, default=64)
    parser.add_argument("--num-envs", type=int, default=4)
    parser.add_argument("--steps", type=int, default=16)
    parser.add_argument("--output", type=Path)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.num_envs <= 0 or args.steps <= 0 or args.horizon <= 0:
        raise ValueError("num-envs, steps, and horizon must be positive")
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(args.usd.resolve())
    simulation_app = None
    environment = None
    try:
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import numpy as np
        import torch

        from next_lab.isaac_reference_env import (
            NextEngineReferenceDirectEnv,
            NextEngineReferenceDirectEnvCfg,
        )
        from next_lab.reference_tracker import (
            ReferenceCorpus,
            ReferenceTrackerProfile,
            build_observation,
            reference_state,
        )

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = args.num_envs
        cfg.sim.device = args.device
        cfg.seed = 120_812
        cfg.fixed_clip_id = args.clip_id
        cfg.fixed_start_frame = args.start_frame
        cfg.fixed_horizon_motor_ticks = args.horizon
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.profile.resolve()),
            corpus_root=str(args.corpus_root.resolve()),
            gate_report_path=str(args.gate_report.resolve()),
        )
        observations, _ = environment.reset()
        policy = observations["policy"]
        if policy.shape != (args.num_envs, 435) or not torch.isfinite(policy).all():
            raise RuntimeError("Isaac reference reset observation is invalid")
        profile = ReferenceTrackerProfile.load(args.profile.resolve())
        corpus = ReferenceCorpus(
            profile, args.corpus_root.resolve(), args.gate_report.resolve()
        )
        clip = corpus.load_clip(args.clip_id)
        expected = build_observation(
            reference_state(clip, args.start_frame), clip, args.start_frame
        )
        actual = environment.last_raw_observation[0].detach().cpu().numpy()
        difference = np.abs(actual.astype(np.int64) - expected.astype(np.int64))
        slices = {
            "root_quaternion": (0, 4),
            "root_linear_velocity": (4, 7),
            "root_angular_velocity": (7, 10),
            "joint_position": (10, 33),
            "joint_velocity": (33, 56),
            "previous_target": (56, 79),
            "dynamic_contacts": (79, 86),
            "phase_and_reference": (86, 435),
        }
        reset_differences = {
            name: int(np.max(difference[start:end]))
            for name, (start, end) in slices.items()
        }
        if any(
            reset_differences[name] > tolerance
            for name, tolerance in {
                "root_quaternion": 256,
                "root_linear_velocity": 4,
                "root_angular_velocity": 4,
                "joint_position": 4,
                "joint_velocity": 8,
                "previous_target": 1,
                "phase_and_reference": 256,
            }.items()
        ):
            raise RuntimeError("Isaac exact-reference reset exceeds correspondence tolerance")
        zero = torch.zeros((args.num_envs, 23), device=environment.device)
        terminal_count = 0
        reward_minimum = float("inf")
        reward_maximum = float("-inf")
        executed_steps = 0
        first_step_trace = None
        for _ in range(args.steps):
            observations, rewards, terminated, truncated, _ = environment.step(zero)
            if not torch.isfinite(observations["policy"]).all() or not torch.isfinite(
                rewards
            ).all():
                raise RuntimeError("Isaac reference step produced non-finite output")
            terminal_count += int(torch.sum(terminated | truncated).item())
            reward_minimum = min(reward_minimum, float(torch.min(rewards).item()))
            reward_maximum = max(reward_maximum, float(torch.max(rewards).item()))
            executed_steps += 1
            if first_step_trace is None:
                first_step_trace = {
                    "pre_physics_joint_position_microradians": environment.last_step_pre_physics_action_joint_position_microradians[
                        0
                    ].tolist(),
                    "pre_physics_joint_velocity_microradians_per_second": environment.last_step_pre_physics_action_joint_velocity_microradians_per_second[
                        0
                    ].tolist(),
                    "command_reference_target_microradians": environment.last_step_command_reference_target_microradians[
                        0
                    ].tolist(),
                    "applied_target_microradians": environment.last_step_applied_target_microradians[
                        0
                    ].tolist(),
                    "joint_position_microradians": environment.last_step_action_joint_position_microradians[
                        0
                    ].tolist(),
                    "joint_velocity_microradians_per_second": environment.last_step_action_joint_velocity_microradians_per_second[
                        0
                    ].tolist(),
                    "velocity_excess_by_action_channel_microradians_per_second": environment.last_step_velocity_excess_by_action_channel[
                        0
                    ].tolist(),
                    "hard_rom_excess_by_action_channel_microradians": environment.last_step_hard_rom_excess_by_action_channel[
                        0
                    ].tolist(),
                    "terminal_reason": int(
                        environment.last_step_terminal_reason[0].item()
                    ),
                    "failure_joint_velocity": bool(
                        environment.last_step_failure_joint_velocity[0].item()
                    ),
                    "failure_effort_envelope": bool(
                        environment.last_step_failure_effort_envelope[0].item()
                    ),
                    "failure_hard_rom": bool(
                        environment.last_step_failure_hard_rom[0].item()
                    ),
                    "failure_hard_impact": bool(
                        environment.last_step_failure_hard_impact[0].item()
                    ),
                    "failure_self_collision": bool(
                        environment.last_step_failure_self_collision[0].item()
                    ),
                }
        report = {
            "schema_version": 1,
            "check": "MOTOR-REFERENCE-ENV-P1-ISAAC-SANITY",
            "status": "PASS",
            "claim": "IsaacResetObservationRewardTerminalSanityOnly",
            "profile_sha256": _sha256(args.profile.resolve()),
            "descriptor_sha256": _sha256(args.descriptor.resolve()),
            "usd_sha256": _sha256(args.usd.resolve()),
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "clip_id": args.clip_id,
            "start_frame": args.start_frame,
            "horizon_motor_ticks": args.horizon,
            "physics_velocity_limit_basis_points": environment.physics_velocity_limit_basis_points,
            "num_envs": args.num_envs,
            "executed_motor_steps": executed_steps,
            "terminal_count": terminal_count,
            "reward_minimum": reward_minimum,
            "reward_maximum": reward_maximum,
            "first_step_trace": first_step_trace,
            "reset_maximum_absolute_difference_raw": reset_differences,
            "dynamic_contact_difference_disposition": "sensor state is empty until the first physics step",
            "training_runs": 0,
            "optimizer_steps": 0,
            "learned_policy_claim": False,
        }
        if args.output is not None:
            output = args.output.resolve()
            output.parent.mkdir(parents=True, exist_ok=True)
            payload = json.dumps(report, indent=2, sort_keys=True) + "\n"
            temporary = output.with_suffix(output.suffix + ".tmp")
            temporary.write_text(payload, encoding="utf-8")
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
