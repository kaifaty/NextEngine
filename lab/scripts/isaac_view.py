#!/usr/bin/env python3
"""Continuously view one closed NextEngine policy checkpoint in Isaac Sim."""

from __future__ import annotations

import argparse
import copy
import json
import os
import time
from pathlib import Path
from typing import Any

from isaaclab.app import AppLauncher

from next_lab.isaac_training import (
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    closed_checkpoint_history,
    latest_closed_checkpoint,
    load_active_training_generation,
    require_external_path,
    sha256_file,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
MOTOR_HZ = 60.0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Open a continuously running 3D view of one trained humanoid policy."
    )
    parser.add_argument("--generation-index", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--runs-root", type=Path)
    parser.add_argument("--checkpoint", type=Path)
    parser.add_argument("--browser-assets", type=Path)
    parser.add_argument("--no-open-browser", action="store_true")
    parser.add_argument("--seed", type=int)
    parser.add_argument("--status-interval", type=int, default=60)
    parser.add_argument(
        "--max-steps",
        type=int,
        default=0,
        help="Stop after this many motor steps; zero runs until the window closes.",
    )
    parser.add_argument(
        "--unthrottled",
        action="store_true",
        help="Run as fast as rendering allows instead of pacing at 60 motor steps/s.",
    )
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.max_steps < 0:
        raise ValueError("max-steps must be non-negative")
    if args.status_interval <= 0:
        raise ValueError("status-interval must be positive")

    generation_index = require_external_path(
        args.generation_index, REPOSITORY_ROOT, label="active training generation"
    )
    generation = load_active_training_generation(generation_index)
    profile = IsaacTrainingProfile.load(args.profile.resolve())
    seed = profile.evaluation["seeds"][0] if args.seed is None else args.seed
    config = ResolvedTrainingConfig.from_profile(
        profile,
        num_envs=1,
        steps_per_env=1,
        iterations=1,
        save_interval=1,
        seed=seed,
    )
    descriptor = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")
    generation.manifest.require_input(profile, sha256_file(descriptor), sha256_file(usd))
    browser_assets = None
    if args.browser_assets is not None:
        browser_assets = require_external_path(
            args.browser_assets, REPOSITORY_ROOT, label="WebGL viewer assets"
        )
    checkpoint = resolve_checkpoint(args, generation.manifest.generation_id)
    parent = validate_closed_checkpoint(checkpoint, generation.manifest.generation_id)
    validate_checkpoint_artifacts(parent, profile, descriptor, usd)
    checkpoint_history = closed_checkpoint_history(
        checkpoint, generation.manifest.generation_id
    )
    checkpoint_by_name = {candidate.name: candidate for candidate in checkpoint_history}

    print(
        json.dumps(
            {
                "checkpoint": str(checkpoint),
                "checkpoint_count": len(checkpoint_history),
                "mode": "3d-policy-viewer",
                "seed": seed,
                "status": "starting",
            },
            indent=2,
            sort_keys=True,
        ),
        flush=True,
    )

    simulation_app = None
    wrapped = None
    browser_viewer = None
    try:
        os.environ["NEXTENGINE_HUMANOID_USD"] = str(usd)
        args.device = config.device
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        import torch
        from isaaclab_rl.rsl_rl import RslRlVecEnvWrapper

        from next_lab.browser_policy_viewer import BrowserPolicyViewer
        from next_lab.isaac_env import (
            NextEngineHumanoidDirectEnv,
            NextEngineHumanoidDirectEnvCfg,
            engine_quaternion_xyzw_from_isaac_wxyz_tensor,
            isaac_prim_name,
        )
        from next_lab.isaac_rl import GuardedOnPolicyRunner, build_agent_cfg

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = 1
        env_cfg.seed = seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.episode_ordinal_start = profile.evaluation.get("episode_ordinal_start", 0)
        env_cfg.sim.device = config.device
        env_cfg.viewer.eye = (3.5, 3.5, 2.25)
        env_cfg.viewer.lookat = (0.0, 0.0, 0.75)
        env_cfg.viewer.origin_type = "asset_root"
        env_cfg.viewer.env_index = 0
        env_cfg.viewer.asset_name = "humanoid"

        environment = NextEngineHumanoidDirectEnv(
            env_cfg,
            descriptor_path=str(descriptor),
        )
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)
        runner = GuardedOnPolicyRunner(
            wrapped,
            copy.deepcopy(build_agent_cfg(config).to_dict()),
            log_dir=None,
            device=config.device,
        )
        runner.load(str(checkpoint), load_optimizer=False, map_location=config.device)
        policy = runner.get_inference_policy(device=config.device)
        observations, _ = wrapped.reset()
        active_checkpoint = checkpoint

        if browser_assets is not None:
            browser_viewer = BrowserPolicyViewer(browser_assets)
            configuration = browser_configuration(
                environment,
                active_checkpoint,
                checkpoint_history,
                isaac_prim_name,
            )
            url = browser_viewer.start(
                configuration,
                open_browser=not args.no_open_browser,
            )
            browser_viewer.publish(
                policy_state(
                    environment,
                    0.0,
                    0,
                    False,
                    active_checkpoint.name,
                    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
                )
            )
            print(f"3D browser viewer is running at {url}", flush=True)
            print("Close this terminal or press Ctrl+C to stop.", flush=True)
        else:
            print(
                "Native 3D viewer is running. Close the Isaac Sim window or press Ctrl+C to stop.",
                flush=True,
            )
        step = 0
        with torch.no_grad():
            while simulation_app.is_running() and (args.max_steps == 0 or step < args.max_steps):
                if browser_viewer is not None:
                    requested = browser_viewer.take_checkpoint_request()
                    if requested is not None and requested != active_checkpoint.name:
                        active_checkpoint = checkpoint_by_name[requested]
                        runner.load(
                            str(active_checkpoint),
                            load_optimizer=False,
                            map_location=config.device,
                        )
                        policy = runner.get_inference_policy(device=config.device)
                        observations, _ = wrapped.reset()
                        step = 0
                        browser_viewer.publish(
                            policy_state(
                                environment,
                                0.0,
                                step,
                                False,
                                active_checkpoint.name,
                                engine_quaternion_xyzw_from_isaac_wxyz_tensor,
                            )
                        )
                        print(
                            json.dumps(
                                {
                                    "checkpoint": str(active_checkpoint),
                                    "status": "switched",
                                },
                                sort_keys=True,
                            ),
                            flush=True,
                        )
                        continue
                started = time.perf_counter()
                actions = policy(observations)
                require_finite("policy action", actions)
                observations, rewards, dones, _ = wrapped.step(actions)
                step += 1

                done = bool(dones[0].item())
                state = policy_state(
                    environment,
                    float(rewards[0].item()),
                    step,
                    done,
                    active_checkpoint.name,
                    engine_quaternion_xyzw_from_isaac_wxyz_tensor,
                )
                if browser_viewer is not None:
                    browser_viewer.publish(state)
                if step == 1 or step % args.status_interval == 0 or done:
                    print_status(state)

                if not args.unthrottled:
                    remaining = 1.0 / MOTOR_HZ - (time.perf_counter() - started)
                    if remaining > 0.0:
                        time.sleep(remaining)
    except KeyboardInterrupt:
        print("3D viewer stopped by user.", flush=True)
    finally:
        if browser_viewer is not None:
            browser_viewer.close()
        if wrapped is not None:
            wrapped.close()
        if simulation_app is not None:
            simulation_app.close()


def resolve_checkpoint(args: argparse.Namespace, generation_id: str) -> Path:
    if args.checkpoint is not None:
        return require_external_path(
            args.checkpoint, REPOSITORY_ROOT, label="viewer checkpoint"
        )
    if args.runs_root is None:
        raise ValueError("either --checkpoint or --runs-root is required")
    runs_root = require_external_path(
        args.runs_root, REPOSITORY_ROOT, label="training runs root"
    )
    return latest_closed_checkpoint(runs_root, generation_id)


def browser_configuration(
    environment: Any,
    checkpoint: Path,
    checkpoint_history: list[Path],
    isaac_prim_name: Any,
) -> dict[str, Any]:
    descriptor_bodies = {
        isaac_prim_name(body["body_id"]): body for body in environment.descriptor["bodies"]
    }
    body_names = list(environment.robot.body_names)
    indices = {name: index for index, name in enumerate(body_names)}
    bodies = []
    root_indices = []
    for index, name in enumerate(body_names):
        body = descriptor_bodies.get(name)
        if body is None:
            raise RuntimeError(f"viewer cannot map Isaac body to descriptor: {name}")
        if "parent_body_slot" in body:
            parent_slot = body["parent_body_slot"]
            parent_id = (
                None
                if parent_slot is None
                else environment.descriptor["bodies"][int(parent_slot)]["body_id"]
            )
        else:
            parent_id = body.get("parent_body_id")
        if parent_id is None:
            parent_index = -1
            root_indices.append(index)
        else:
            parent_name = isaac_prim_name(parent_id)
            if parent_name not in indices:
                raise RuntimeError(f"viewer cannot map descriptor parent body: {parent_id}")
            parent_index = indices[parent_name]
        body_id = body["body_id"]
        color = "#60a5fa" if ".left-" in body_id else "#f472b6"
        if ".right-" not in body_id and ".left-" not in body_id:
            color = "#fbbf24"
        colliders = []
        for collider in body["colliders"]:
            geometry = collider["geometry"]
            kind = geometry["kind"]
            if kind == "sphere":
                shape = {
                    "kind": kind,
                    "radius_m": geometry["radius_micrometres"] / 1_000_000.0,
                }
            elif kind == "box":
                shape = {
                    "kind": kind,
                    "half_extents_m": [
                        value / 1_000_000.0
                        for value in geometry["half_extents_micrometres"]
                    ],
                }
            elif kind == "capsule":
                shape = {
                    "kind": kind,
                    "radius_m": geometry["radius_micrometres"] / 1_000_000.0,
                    "half_segment_m": geometry["half_segment_micrometres"]
                    / 1_000_000.0,
                }
            else:
                raise RuntimeError(
                    f"viewer body has unsupported collider geometry: {kind}"
                )
            colliders.append(
                {
                    "color": "#22d3ee" if collider["contact_role"] == 8 else color,
                    "geometry": shape,
                    "local_position_m": [
                        value / 1_000_000.0
                        for value in collider["local_translation_micrometres"]
                    ],
                    "local_quaternion_xyzw": [
                        value / float(1 << 30)
                        for value in collider["local_rotation_q1_30"]
                    ],
                }
            )
        bodies.append(
            {
                "body_id": body_id,
                "colliders": colliders,
                "parent_index": parent_index,
            }
        )
    if len(root_indices) != 1:
        raise RuntimeError("viewer requires exactly one root body")
    return {
        "bodies": bodies,
        "checkpoint": checkpoint.name,
        "checkpoints": [
            {
                "iteration": int(candidate.stem.removeprefix("model_")),
                "name": candidate.name,
            }
            for candidate in checkpoint_history
        ],
        "root_index": root_indices[0],
    }


def policy_state(
    environment: Any,
    reward: float,
    step: int,
    done: bool,
    checkpoint_name: str,
    quaternion_mapper: Any | None = None,
) -> dict[str, Any]:
    command = environment._current_command()[0].detach().cpu().tolist()
    right, forward, yaw = (value / 1_000_000.0 for value in command)
    height = float(environment.robot.data.root_pos_w[0, 2].item())
    state = {
        "body_positions_w": environment.robot.data.body_pos_w[0].detach().cpu().tolist(),
        "checkpoint": checkpoint_name,
        "command": {
            "forward_mps": forward,
            "mode": command_mode(right, forward, yaw),
            "right_mps": right,
            "yaw_rps": yaw,
        },
        "episode_reset": done,
        "episode_step": int(environment.episode_length_buf[0].item()),
        "reward": reward,
        "root_height_m": height,
        "viewer_step": step,
    }
    if quaternion_mapper is not None:
        state["body_quaternions_xyzw"] = (
            quaternion_mapper(environment.robot.data.body_quat_w[0])
            .detach()
            .cpu()
            .tolist()
        )
    return state


def print_status(state: dict[str, Any]) -> None:
    command = state["command"]
    print(
        json.dumps(
            {
                "command": {
                    "forward_mps": round(command["forward_mps"], 3),
                    "mode": command["mode"],
                    "right_mps": round(command["right_mps"], 3),
                    "yaw_rps": round(command["yaw_rps"], 3),
                },
                "episode_reset": state["episode_reset"],
                "episode_step": state["episode_step"],
                "reward": round(state["reward"], 4),
                "root_height_m": round(state["root_height_m"], 3),
                "viewer_step": state["viewer_step"],
            },
            sort_keys=True,
        ),
        flush=True,
    )


def command_mode(right: float, forward: float, yaw: float) -> str:
    translating = right != 0.0 or forward != 0.0
    turning = yaw != 0.0
    if translating and turning:
        return "move-and-turn"
    if translating:
        return "move"
    if turning:
        return "turn"
    return "stand"


def require_finite(name: str, value: Any) -> None:
    import torch

    if not torch.isfinite(value).all():
        raise RuntimeError(f"non-finite {name}")


if __name__ == "__main__":
    main()
