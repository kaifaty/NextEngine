#!/usr/bin/env python3
"""Audit randomized reference-phase resets without training or a learned policy."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import traceback
from pathlib import Path

from isaaclab.app import AppLauncher


DEFAULT_RUN_ROOT_HEX = (
    "b94845b6f5bc0afcc103d4545bc1b90c"
    "c790f7965ba2322c8fcdb23f3c6f7d91"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--clip-id", default="cmu104-start-right")
    parser.add_argument("--horizon", type=int, default=11)
    parser.add_argument("--num-envs", type=int, default=256)
    parser.add_argument("--minimum-episodes", type=int, default=4096)
    parser.add_argument("--reset-safety-window-motor-ticks", type=int, default=1)
    parser.add_argument("--run-root-hex", default=DEFAULT_RUN_ROOT_HEX)
    parser.add_argument("--output", type=Path, required=True)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if (
        args.horizon <= 0
        or args.num_envs <= 0
        or args.minimum_episodes <= 0
        or args.reset_safety_window_motor_ticks <= 0
        or len(args.run_root_hex) != 64
    ):
        raise ValueError("invalid phase-safety audit bounds")
    for path in (args.descriptor, args.profile, args.gate_report, args.usd):
        if not path.resolve().is_file():
            raise FileNotFoundError(path)
    if not args.corpus_root.resolve().is_dir():
        raise FileNotFoundError(args.corpus_root)
    bytes.fromhex(args.run_root_hex)

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
        )

        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = args.num_envs
        cfg.sim.device = args.device
        cfg.seed = 120_812
        cfg.eligible_clip_ids = (args.clip_id,)
        cfg.phase_randomization = True
        cfg.rng_run_root_hex = args.run_root_hex
        cfg.fixed_horizon_motor_ticks = args.horizon
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.profile.resolve()),
            corpus_root=str(args.corpus_root.resolve()),
            gate_report_path=str(args.gate_report.resolve()),
        )
        observations, _ = environment.reset()
        if not torch.isfinite(observations["policy"]).all():
            raise RuntimeError("phase-safety reset observation is non-finite")

        clip = environment.reference_clip
        valid_start_count = clip.frame_count - args.horizon
        if valid_start_count <= 0:
            raise ValueError("phase-safety horizon exceeds the selected clip")
        phase_counts = [0] * valid_start_count
        zero = torch.zeros(
            (args.num_envs, 23), dtype=torch.float32, device=environment.device
        )
        branch_counts = {
            "reference_complete": 0,
            "failure": 0,
            "tracking_lost": 0,
            "hard_rom": 0,
            "joint_safety": 0,
            "joint_velocity": 0,
            "effort_envelope": 0,
            "hard_impact": 0,
            "self_collision": 0,
            "forbidden_contact": 0,
            "world_bounds": 0,
            "fall": 0,
            "non_finite": 0,
            "truncated": 0,
        }
        terminal_tick_counts: dict[str, dict[str, int]] = {
            branch: {} for branch in branch_counts
        }
        reset_window_safety_failure_count = 0
        hard_rom_channel_counts: dict[str, int] = {}
        hard_rom_channel_maxima: dict[str, dict[str, int]] = {}
        reset_window_hard_rom_channel_counts: dict[str, int] = {}
        reset_window_hard_rom_channel_maxima: dict[str, dict[str, int]] = {}
        velocity_channel_counts: dict[str, int] = {}
        velocity_channel_maxima: dict[str, dict[str, int]] = {}
        reset_window_velocity_channel_counts: dict[str, int] = {}
        reset_window_velocity_channel_maxima: dict[str, dict[str, int]] = {}
        forbidden_contact_mask_counts: dict[str, int] = {}
        contact_pair_counts = {
            "hard_impact": {},
            "self_collision": {},
            "forbidden_contact": {},
        }
        reset_window_contact_pair_counts = {
            "hard_impact": {},
            "self_collision": {},
            "forbidden_contact": {},
        }
        contact_pair_first_examples: dict[str, dict[str, int]] = {}
        maximum_hard_rom_excess = 0
        completed_episodes = 0
        executed_motor_steps = 0
        maximum_motor_steps = args.minimum_episodes * (args.horizon + 1)
        while completed_episodes < args.minimum_episodes:
            observations, _, terminated, truncated, _ = environment.step(zero)
            executed_motor_steps += 1
            if not torch.isfinite(observations["policy"]).all():
                raise RuntimeError("phase-safety step observation is non-finite")
            done = terminated | truncated
            done_ids = torch.nonzero(done, as_tuple=False).flatten()
            if done_ids.numel() == 0:
                if executed_motor_steps >= maximum_motor_steps:
                    raise RuntimeError("phase-safety audit made no bounded progress")
                continue

            starts = environment.last_step_episode_start_frame[done_ids]
            success = environment.last_step_success[done_ids]
            failure = environment.last_step_failure[done_ids]
            tracking = environment.last_step_failure_tracking_lost[done_ids]
            hard_rom = environment.last_step_failure_hard_rom[done_ids]
            joint_safety = environment.last_step_failure_joint_safety[done_ids]
            joint_velocity = environment.last_step_failure_joint_velocity[done_ids]
            effort_envelope = environment.last_step_failure_effort_envelope[done_ids]
            hard_impact = environment.last_step_failure_hard_impact[done_ids]
            self_collision = environment.last_step_failure_self_collision[done_ids]
            forbidden = environment.last_step_failure_forbidden_contact[done_ids]
            world_bounds = environment.last_step_failure_world_bounds[done_ids]
            fall = environment.last_step_failure_fall[done_ids]
            non_finite = environment.last_step_failure_non_finite[done_ids]
            hard_channels = environment.last_step_hard_rom_action_channel[done_ids]
            hard_excess = environment.last_step_hard_rom_excess_microradians[done_ids]
            hard_excess_by_channel = (
                environment.last_step_hard_rom_excess_by_action_channel[done_ids]
            )
            velocity_excess_by_channel = (
                environment.last_step_velocity_excess_by_action_channel[done_ids]
            )
            action_joint_positions = (
                environment.last_step_action_joint_position_microradians[done_ids]
            )
            reference_frames = environment.last_step_reference_frame[done_ids]
            elapsed_ticks = environment.last_step_episode_elapsed_motor_ticks[done_ids]
            forbidden_masks = environment.last_step_forbidden_contact_mask[done_ids]
            contact_pair_masks = {
                "hard_impact": environment.last_step_hard_impact_pair_mask[
                    done_ids
                ],
                "self_collision": environment.last_step_self_collision_pair_mask[
                    done_ids
                ],
                "forbidden_contact": environment.last_step_forbidden_contact_pair_mask[
                    done_ids
                ],
            }
            for index in range(done_ids.numel()):
                start = int(starts[index].item())
                if not 0 <= start < valid_start_count:
                    raise RuntimeError("phase-safety audit observed an invalid start frame")
                phase_counts[start] += 1
                branch_counts["reference_complete"] += int(success[index].item())
                branch_counts["failure"] += int(failure[index].item())
                branch_counts["tracking_lost"] += int(tracking[index].item())
                branch_counts["hard_rom"] += int(hard_rom[index].item())
                branch_counts["joint_safety"] += int(joint_safety[index].item())
                branch_counts["joint_velocity"] += int(joint_velocity[index].item())
                branch_counts["effort_envelope"] += int(
                    effort_envelope[index].item()
                )
                branch_counts["hard_impact"] += int(hard_impact[index].item())
                branch_counts["self_collision"] += int(self_collision[index].item())
                branch_counts["forbidden_contact"] += int(forbidden[index].item())
                branch_counts["world_bounds"] += int(world_bounds[index].item())
                branch_counts["fall"] += int(fall[index].item())
                branch_counts["non_finite"] += int(non_finite[index].item())
                branch_counts["truncated"] += int(truncated[done_ids[index]].item())
                tick = int(elapsed_ticks[index].item())
                if tick <= 0 or tick > args.horizon:
                    raise RuntimeError("phase-safety audit observed an invalid terminal tick")
                tick_key = str(tick)
                branch_values = {
                    "reference_complete": success[index],
                    "failure": failure[index],
                    "tracking_lost": tracking[index],
                    "hard_rom": hard_rom[index],
                    "joint_safety": joint_safety[index],
                    "joint_velocity": joint_velocity[index],
                    "effort_envelope": effort_envelope[index],
                    "hard_impact": hard_impact[index],
                    "self_collision": self_collision[index],
                    "forbidden_contact": forbidden[index],
                    "world_bounds": world_bounds[index],
                    "fall": fall[index],
                    "non_finite": non_finite[index],
                    "truncated": truncated[done_ids[index]],
                }
                for branch, value in branch_values.items():
                    if value:
                        counts = terminal_tick_counts[branch]
                        counts[tick_key] = counts.get(tick_key, 0) + 1
                if tick <= args.reset_safety_window_motor_ticks and (
                    hard_rom[index]
                    or joint_safety[index]
                    or hard_impact[index]
                    or self_collision[index]
                    or forbidden[index]
                    or world_bounds[index]
                    or fall[index]
                    or non_finite[index]
                ):
                    reset_window_safety_failure_count += 1
                maximum_hard_rom_excess = max(
                    maximum_hard_rom_excess, int(hard_excess[index].item())
                )
                if hard_rom[index]:
                    key = str(int(hard_channels[index].item()))
                    hard_rom_channel_counts[key] = hard_rom_channel_counts.get(key, 0) + 1
                    for channel in range(hard_excess_by_channel.shape[1]):
                        excess = int(hard_excess_by_channel[index, channel].item())
                        if excess <= 10:
                            continue
                        channel_key = str(channel)
                        previous = hard_rom_channel_maxima.get(channel_key)
                        if previous is None or excess > previous["excess_microradians"]:
                            hard_rom_channel_maxima[channel_key] = {
                                "excess_microradians": excess,
                                "episode_start_frame": start,
                                "reference_frame": int(reference_frames[index].item()),
                                "terminal_motor_tick": tick,
                                "observed_position_microradians": int(
                                    action_joint_positions[index, channel].item()
                                ),
                            }
                        if tick <= args.reset_safety_window_motor_ticks:
                            reset_window_hard_rom_channel_counts[channel_key] = (
                                reset_window_hard_rom_channel_counts.get(channel_key, 0)
                                + 1
                            )
                            previous_reset = reset_window_hard_rom_channel_maxima.get(
                                channel_key
                            )
                            if (
                                previous_reset is None
                                or excess > previous_reset["excess_microradians"]
                            ):
                                reset_window_hard_rom_channel_maxima[channel_key] = {
                                    "excess_microradians": excess,
                                    "episode_start_frame": start,
                                    "reference_frame": int(reference_frames[index].item()),
                                    "terminal_motor_tick": tick,
                                    "observed_position_microradians": int(
                                        action_joint_positions[index, channel].item()
                                    ),
                                }
                if forbidden[index]:
                    key = str(int(forbidden_masks[index].item()))
                    forbidden_contact_mask_counts[key] = (
                        forbidden_contact_mask_counts.get(key, 0) + 1
                    )
                for category, pair_masks in contact_pair_masks.items():
                    pair_indices = torch.nonzero(
                        pair_masks[index], as_tuple=False
                    ).flatten().tolist()
                    for pair_index in pair_indices:
                        pair_id = environment.contact_pair_ids[pair_index]
                        counts = contact_pair_counts[category]
                        counts[pair_id] = counts.get(pair_id, 0) + 1
                        example_key = f"{category}:{pair_id}"
                        contact_pair_first_examples.setdefault(
                            example_key,
                            {
                                "episode_start_frame": start,
                                "reference_frame": int(reference_frames[index].item()),
                                "terminal_motor_tick": tick,
                            },
                        )
                        if tick <= args.reset_safety_window_motor_ticks:
                            reset_counts = reset_window_contact_pair_counts[category]
                            reset_counts[pair_id] = reset_counts.get(pair_id, 0) + 1
                if joint_velocity[index]:
                    for channel in range(velocity_excess_by_channel.shape[1]):
                        excess = int(
                            velocity_excess_by_channel[index, channel].item()
                        )
                        if excess <= 0:
                            continue
                        channel_key = str(channel)
                        velocity_channel_counts[channel_key] = (
                            velocity_channel_counts.get(channel_key, 0) + 1
                        )
                        previous = velocity_channel_maxima.get(channel_key)
                        if (
                            previous is None
                            or excess
                            > previous["excess_microradians_per_second"]
                        ):
                            velocity_channel_maxima[channel_key] = {
                                "excess_microradians_per_second": excess,
                                "episode_start_frame": start,
                                "reference_frame": int(
                                    reference_frames[index].item()
                                ),
                                "terminal_motor_tick": tick,
                            }
                        if tick <= args.reset_safety_window_motor_ticks:
                            reset_window_velocity_channel_counts[channel_key] = (
                                reset_window_velocity_channel_counts.get(
                                    channel_key, 0
                                )
                                + 1
                            )
                            previous_reset = reset_window_velocity_channel_maxima.get(
                                channel_key
                            )
                            if (
                                previous_reset is None
                                or excess
                                > previous_reset[
                                    "excess_microradians_per_second"
                                ]
                            ):
                                reset_window_velocity_channel_maxima[channel_key] = {
                                    "excess_microradians_per_second": excess,
                                    "episode_start_frame": start,
                                    "reference_frame": int(
                                        reference_frames[index].item()
                                    ),
                                    "terminal_motor_tick": tick,
                                }
            completed_episodes += done_ids.numel()

        missing_start_frames = [
            frame for frame, count in enumerate(phase_counts) if count == 0
        ]
        full_horizon_safety_failure_count = (
            branch_counts["hard_rom"]
            + branch_counts["joint_safety"]
            + branch_counts["hard_impact"]
            + branch_counts["self_collision"]
            + branch_counts["forbidden_contact"]
            + branch_counts["world_bounds"]
            + branch_counts["fall"]
            + branch_counts["non_finite"]
        )
        report = {
            "schema_version": 1,
            "check": "MOTOR-REFERENCE-ENV-P1-ISAAC-PHASE-SAFETY-AUDIT",
            "status": (
                "PASS"
                if not missing_start_frames
                and reset_window_safety_failure_count == 0
                else "FAIL"
            ),
            "claim": "IsaacRandomizedPhaseResetSafetyOnly",
            "profile_sha256": _sha256(args.profile.resolve()),
            "isaac_velocity_guard_profile_sha256": environment.reference_profile.document[
                "termination"
            ].get("isaac_velocity_guard_profile_sha256"),
            "isaac_physics_velocity_limit_basis_points": environment.physics_velocity_limit_basis_points,
            "descriptor_sha256": _sha256(args.descriptor.resolve()),
            "usd_sha256": _sha256(args.usd.resolve()),
            "gate_report_sha256": _sha256(args.gate_report.resolve()),
            "corpus_manifest_sha256": json.loads(
                (args.corpus_root.resolve() / "corpus-manifest.json").read_bytes()
            )["manifest_sha256"],
            "tool_sha256": _sha256(Path(__file__).resolve()),
            "clip_id": args.clip_id,
            "reference_frame_count": clip.frame_count,
            "horizon_motor_ticks": args.horizon,
            "valid_start_frame_count": valid_start_count,
            "distinct_start_frame_count": sum(count > 0 for count in phase_counts),
            "missing_start_frames": missing_start_frames,
            "phase_episode_counts": {
                str(frame): count for frame, count in enumerate(phase_counts)
            },
            "requested_minimum_episode_count": args.minimum_episodes,
            "observed_episode_count": completed_episodes,
            "executed_vector_motor_steps": executed_motor_steps,
            "num_envs": args.num_envs,
            "run_root_hex": args.run_root_hex,
            "zero_residual_action": True,
            "terminal_branch_counts": branch_counts,
            "terminal_tick_counts": {
                branch: dict(sorted(counts.items(), key=lambda item: int(item[0])))
                for branch, counts in terminal_tick_counts.items()
            },
            "maximum_hard_rom_excess_microradians": maximum_hard_rom_excess,
            "hard_rom_action_channel_counts": dict(sorted(hard_rom_channel_counts.items())),
            "hard_rom_action_channel_maxima": dict(
                sorted(hard_rom_channel_maxima.items())
            ),
            "reset_window_hard_rom_action_channel_counts": dict(
                sorted(reset_window_hard_rom_channel_counts.items())
            ),
            "reset_window_hard_rom_action_channel_maxima": dict(
                sorted(reset_window_hard_rom_channel_maxima.items())
            ),
            "velocity_action_channel_counts": dict(
                sorted(velocity_channel_counts.items())
            ),
            "velocity_action_channel_maxima": dict(
                sorted(velocity_channel_maxima.items())
            ),
            "reset_window_velocity_action_channel_counts": dict(
                sorted(reset_window_velocity_channel_counts.items())
            ),
            "reset_window_velocity_action_channel_maxima": dict(
                sorted(reset_window_velocity_channel_maxima.items())
            ),
            "forbidden_contact_mask_counts": dict(
                sorted(forbidden_contact_mask_counts.items())
            ),
            "contact_pair_counts": {
                category: dict(sorted(counts.items()))
                for category, counts in contact_pair_counts.items()
            },
            "reset_window_contact_pair_counts": {
                category: dict(sorted(counts.items()))
                for category, counts in reset_window_contact_pair_counts.items()
            },
            "contact_pair_first_examples": dict(
                sorted(contact_pair_first_examples.items())
            ),
            "reset_safety_window_motor_ticks": args.reset_safety_window_motor_ticks,
            "reset_window_safety_failure_count": reset_window_safety_failure_count,
            "full_horizon_safety_failure_count": full_horizon_safety_failure_count,
            "full_horizon_safety_disposition": (
                "ReportOnlyUntrainedZeroResidualBaseline"
            ),
            "tracking_loss_disposition": "ReportOnlyUntrainedZeroResidualBaseline",
            "optimizer_steps": 0,
            "training_runs": 0,
            "learned_policy_claim": False,
        }
        output = args.output.resolve()
        output.parent.mkdir(parents=True, exist_ok=True)
        payload = (json.dumps(report, indent=2, sort_keys=True) + "\n").encode("utf-8")
        if output.exists() and output.read_bytes() != payload:
            raise ValueError(f"refusing to overwrite a different audit: {output}")
        if not output.exists():
            temporary = output.with_suffix(output.suffix + ".tmp")
            temporary.write_bytes(payload)
            temporary.replace(output)
        print(json.dumps(report, indent=2, sort_keys=True), flush=True)
        if report["status"] != "PASS":
            raise RuntimeError("randomized phase reset safety audit failed")
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
