#!/usr/bin/env python3
"""One frozen CPU PhysX / CUDA PPO experiment, with exact native adapter control."""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import json
import random
import subprocess
import sys
import time
from collections import Counter, deque
from dataclasses import fields
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
import torch
from next_lab.canonical_ppo import CanonicalVecEnv, TerminalObservationPPO, shard_root
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from next_lab.motor_lab_client import MotorLabClient
from rsl_rl.modules import ActorCritic

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/canonical-rsl-rl-walking.v1.json"


def seed_root(seed):
    return hashlib.sha256(
        f"nextengine.canonical-walking.v1:{seed}".encode()
    ).hexdigest()


def make_env(
    headless, descriptor, profile, *, device=None, seed=None, num_envs=None, shards=None
):
    return CanonicalVecEnv(
        headless,
        descriptor,
        num_envs=num_envs or profile["num_envs"],
        shards=shards or profile["shards"],
        run_root=seed_root(profile["seed"] if seed is None else seed),
        device=device or profile["device"],
    )


def adapter_control(headless, descriptor, profile):
    """Raw native controls use the same partition/root but independent processes."""
    slots = profile["num_envs"] // profile["shards"]
    count = 2 * slots
    env = make_env(
        headless, descriptor, profile, device="cpu", num_envs=count, shards=2
    )
    controls = []
    transitions = 0
    terminals = 0
    digest = hashlib.sha256()
    try:
        for shard in range(2):
            control = MotorLabClient(
                headless,
                profile["environment_profile_id"],
                slots,
                shard_root(seed_root(profile["seed"]), shard),
            )
            controls.append(control)
            reset = sorted(
                control.reset(list(range(slots))), key=lambda item: item.vector_slot
            )
            np.testing.assert_array_equal(
                env.raw[shard * slots : (shard + 1) * slots],
                np.stack([item.observation_raw for item in reset]),
            )
        rng = np.random.default_rng(193)
        for tick in range(160):
            # Includes zero control and unsafe exploration to exercise independent resets.
            actions = np.zeros((count, 23), dtype=np.float32)
            actions[1:] = rng.uniform(-0.8, 0.8, size=(count - 1, 23)).astype(
                np.float32
            )
            ordinals = env.ordinals.copy()
            _obs, rewards, _dones, extras = env.step(torch.from_numpy(actions))
            for shard, control in enumerate(controls):
                batch = control.step_normalized(
                    ordinals[slots * shard : slots * (shard + 1)],
                    actions[slots * shard : slots * (shard + 1)],
                )
                for expected in batch:
                    index = shard * slots + expected.vector_slot
                    actual = env.last_steps[index]
                    for field in fields(expected):
                        np.testing.assert_equal(
                            getattr(actual, field.name),
                            getattr(expected, field.name),
                            err_msg=f"tick={tick} slot={index} {field.name}",
                        )
                    digest.update(actual.step_root)
                    assert rewards[index].item() == expected.reward_total_q16 / 65536
                    np.testing.assert_array_equal(
                        extras["terminal_observation"]["policy"][index].numpy(),
                        (expected.observation_raw / env.scales).astype(np.float32),
                    )
                ended = [
                    item.vector_slot
                    for item in batch
                    if item.terminated or item.truncated
                ]
                if ended:
                    terminals += len(ended)
                    for reset in control.reset(ended):
                        index = slots * shard + reset.vector_slot
                        np.testing.assert_array_equal(
                            env.raw[index], reset.observation_raw
                        )
                        assert env.ordinals[index] == reset.episode_ordinal
                transitions += len(batch)
        if terminals == 0:
            raise RuntimeError("adapter control did not exercise autoreset")
        return {
            "status": "passed",
            "transitions": transitions,
            "terminals": terminals,
            "step_roots_sha256": digest.hexdigest(),
            "all_step_fields_exact": True,
        }
    finally:
        env.close()
        for control in controls:
            control.close()


def finite_policy(policy):
    for name, tensor in policy.state_dict().items():
        if not torch.isfinite(tensor).all():
            raise RuntimeError(f"non-finite policy state: {name}")


def evaluate(policy, headless, descriptor, profile, output):
    cfg = profile["evaluation"]
    records = []
    policy.eval()
    for seed in cfg["seeds"]:
        env = make_env(headless, descriptor, profile, seed=seed, num_envs=1, shards=1)
        positions, contacts, commands, velocities, actions, quaternions, joints = (
            [],
            [],
            [],
            [],
            [],
            [],
            [],
        )
        try:
            with torch.inference_mode():
                for _ in range(cfg["max_steps"]):
                    action = policy.act_inference(env.get_observations())
                    env.step(action)
                    result = env.last_steps[0]
                    positions.append(result.root_position_micrometres / 1e6)
                    contacts.append(result.contact_flags)
                    commands.append(result.command_raw / 1e6)
                    velocities.append(
                        result.root_linear_velocity_micrometres_per_second / 1e6
                    )
                    actions.append(action[0].cpu().numpy())
                    quaternions.append(result.root_quaternion_q1_30_xyzw / (1 << 30))
                    joints.append(result.joint_position_microradians / 1e6)
                    if result.terminated or result.truncated:
                        break
            position, contact, command, velocity = map(
                np.asarray, (positions, contacts, commands, velocities)
            )
            support_runs = [0, 0]
            current_side, current_length, previous_qualified, switches = (
                None,
                0,
                None,
                0,
            )
            for flags in contact:
                side = (0 if flags[0] else 1) if sum(flags) == 1 else None
                current_length = (
                    current_length + 1
                    if side is not None and side == current_side
                    else 1
                )
                current_side = side
                if side is not None:
                    support_runs[side] = max(support_runs[side], current_length)
                    if current_length == cfg["minimum_single_support_run_ticks"]:
                        if (
                            previous_qualified is not None
                            and side != previous_qualified
                        ):
                            switches += 1
                        previous_qualified = side
            full = (
                result.truncated
                and not result.terminated
                and len(position) == cfg["max_steps"]
            )
            forward = float(position[-1, 2] - position[0, 2])
            error = float(np.mean(np.abs(velocity[:, 2] - command[:, 1])))
            stop_speed = float(
                np.mean(np.linalg.norm(velocity[-cfg["stop_ticks"] :, [0, 2]], axis=1))
            )
            gates = {
                "complete_safe_episode": full,
                "forward_travel": forward >= cfg["minimum_forward_displacement_m"],
                "velocity_tracking": error <= cfg["maximum_forward_velocity_mae_mps"],
                "final_zero_command_ticks": full
                and bool(np.all(command[-cfg["stop_ticks"] :] == 0)),
                "stopped": full and stop_speed <= cfg["maximum_stop_speed_mae_mps"],
                "bilateral_single_support": min(support_runs)
                >= cfg["minimum_single_support_run_ticks"],
                "alternating_support": switches >= cfg["minimum_support_switches"],
            }
            np.savez_compressed(
                output / f"evaluation-{seed}.npz",
                root_position_m=position,
                contact_occupancy=contact,
                command=command,
                root_velocity_mps=velocity,
                action=np.asarray(actions),
                root_quaternion_xyzw=np.asarray(quaternions),
                joint_position_rad=np.asarray(joints),
            )
            records.append(
                {
                    "seed": seed,
                    "ticks": len(position),
                    "terminal_reason": result.terminal_reason_id,
                    "forward_displacement_m": forward,
                    "velocity_mae_mps": error,
                    "stop_speed_mae_mps": stop_speed,
                    "longest_single_support_ticks": support_runs,
                    "support_switches": switches,
                    "gates": gates,
                    "passed": all(gates.values()),
                }
            )
        finally:
            env.close()
    return {
        "status": "passed" if all(item["passed"] for item in records) else "failed",
        "claim": "bounded canonical walking evaluation; no runtime or mirror promotion",
        "episodes": records,
    }


def train(headless, descriptor, profile, output):
    random.seed(profile["seed"])
    np.random.seed(profile["seed"])
    torch.manual_seed(profile["seed"])
    torch.set_num_threads(1)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    env = make_env(headless, descriptor, profile)
    start = time.monotonic()
    returns, lengths = deque(maxlen=100), deque(maxlen=100)
    totals = np.zeros(env.num_envs)
    episode_lengths = np.zeros(env.num_envs, dtype=np.int64)
    try:
        obs = env.get_observations()
        policy = ActorCritic(
            obs,
            {"policy": ["policy"], "critic": ["policy"]},
            env.num_actions,
            **profile["policy"],
        ).to(profile["device"])
        algorithm = TerminalObservationPPO(
            policy, device=profile["device"], **profile["algorithm"]
        )
        algorithm.init_storage(
            "rl", env.num_envs, profile["steps_per_env"], obs, [env.num_actions]
        )
        policy.train()
        for iteration in range(profile["iterations"]):
            if time.monotonic() - start > profile["wall_seconds"]:
                raise TimeoutError("frozen optimizer wall budget exhausted")
            tick_start = time.monotonic()
            reasons = Counter()
            with torch.inference_mode():
                for _ in range(profile["steps_per_env"]):
                    actions = algorithm.act(obs)
                    obs, reward, done, extras = env.step(actions)
                    algorithm.process_env_step(obs, reward, done, extras)
                    totals += reward.cpu().numpy()
                    episode_lengths += 1
                    for index in np.flatnonzero(done.cpu().numpy()):
                        returns.append(float(totals[index]))
                        lengths.append(int(episode_lengths[index]))
                        totals[index] = 0
                        episode_lengths[index] = 0
                        reasons[
                            env.last_steps[index].terminal_reason_id or "time-limit"
                        ] += 1
                algorithm.compute_returns(obs)
            collect_end = time.monotonic()
            losses = algorithm.update()
            finite_policy(policy)
            if not all(np.isfinite(float(value)) for value in losses.values()):
                raise RuntimeError("non-finite PPO loss")
            record = {
                "iteration": iteration,
                "total_timesteps": (iteration + 1)
                * env.num_envs
                * profile["steps_per_env"],
                "losses": losses,
                "learning_rate": algorithm.learning_rate,
                "mean_action_noise_std": float(policy.std.mean().detach().cpu()),
                "mean_episode_return": float(np.mean(returns)) if returns else None,
                "mean_episode_length": float(np.mean(lengths)) if lengths else None,
                "terminal_reasons": dict(reasons),
                "collection_seconds": collect_end - tick_start,
                "learning_seconds": time.monotonic() - collect_end,
                "steps_per_second": env.num_envs
                * profile["steps_per_env"]
                / (time.monotonic() - tick_start),
            }
            with (output / "metrics.jsonl").open("a") as stream:
                stream.write(json.dumps(record, sort_keys=True, allow_nan=False) + "\n")
            print(json.dumps(record, sort_keys=True), flush=True)
            if (iteration + 1) % profile[
                "save_interval"
            ] == 0 or iteration + 1 == profile["iterations"]:
                checkpoint = output / f"model_{iteration}.pt"
                torch.save(
                    {
                        "model_state_dict": policy.state_dict(),
                        "optimizer_state_dict": algorithm.optimizer.state_dict(),
                        "iter": iteration,
                        "infos": {"resume_supported": False},
                    },
                    checkpoint,
                )
        evaluation = evaluate(policy, headless, descriptor, profile, output)
        atomic_write_json(output / "evaluation.json", evaluation)
        return evaluation
    finally:
        env.close()


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("mode", choices=("check-adapter", "freeze", "train"))
    parser.add_argument("--headless", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--generation-index", type=Path)
    args = parser.parse_args()
    headless = args.headless.resolve(strict=True)
    descriptor_path = require_external_path(args.descriptor, ROOT, label="descriptor")
    output = require_external_path(args.output, ROOT, label="run", must_exist=False)
    if output.exists():
        raise ValueError("run output must be new")
    profile = json.loads(PROFILE.read_text())
    if sha256_file(descriptor_path) != profile["descriptor_sha256"]:
        raise ValueError("frozen descriptor file hash mismatch")
    descriptor = json.loads(descriptor_path.read_text())
    versions = {
        name: importlib.metadata.version(name) for name in profile["dependencies"]
    }
    if versions != profile["dependencies"]:
        raise ValueError(f"pinned learner dependency mismatch: {versions}")
    commit = subprocess.check_output(
        ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
    ).strip()
    dirty = bool(
        subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=ROOT, text=True
        ).strip()
    )
    if args.mode in {"train", "freeze"} and dirty:
        raise ValueError("training requires a clean recorded commit")
    if args.mode == "train" and not torch.cuda.is_available():
        raise RuntimeError("frozen learner requires CUDA")
    closure = {
        "repository_commit": commit,
        "profile_sha256": sha256_file(PROFILE),
        "descriptor_sha256": sha256_file(descriptor_path),
        "headless_sha256": sha256_file(headless),
        "dependencies": versions,
    }
    generation = None
    if args.mode == "train":
        if args.generation_index is None:
            raise ValueError("training requires an external frozen generation index")
        index_path = require_external_path(
            args.generation_index, ROOT, label="generation index"
        )
        index = json.loads(index_path.read_text())
        generation_path = require_external_path(
            Path(index["manifest_path"]), ROOT, label="generation"
        )
        if sha256_file(generation_path) != index["manifest_sha256"]:
            raise ValueError("generation file hash mismatch")
        generation = json.loads(generation_path.read_text())
        if generation["closure"] != closure or generation["profile"] != profile:
            raise ValueError("generation input closure mismatch")
        if generation["schema"] != "nextengine.canonical-ppo-generation.v1":
            raise ValueError("unsupported generation schema")
        if output != Path(generation["run_path"]).resolve():
            raise ValueError("output is not the sole admitted generation run")
    output.mkdir(parents=True, exist_ok=False)
    manifest = {
        "schema": "nextengine.canonical-ppo-run.v1",
        "status": "running",
        "mode": args.mode,
        "started_at_utc": datetime.now(timezone.utc).isoformat(),
        "repository_commit": commit,
        "repository_dirty": dirty,
        "profile": profile,
        "profile_sha256": sha256_file(PROFILE),
        "descriptor_path": str(descriptor_path),
        "descriptor_sha256": sha256_file(descriptor_path),
        "headless_path": str(headless),
        "headless_sha256": sha256_file(headless),
        "python": sys.version,
        "python_executable": sys.executable,
        "dependencies": versions,
        "run_root": seed_root(profile["seed"]),
        "input_checkpoint": None,
        "generation_id": "r8b-canonical-walking-v1",
        "run_id": output.name,
        "authority": "ADR-107 bounded R&D; no mirror or runtime promotion; no resume",
    }
    if generation is not None:
        manifest["generation_index_sha256"] = sha256_file(index_path)
        manifest["generation_manifest_sha256"] = sha256_file(generation_path)
    atomic_write_json(output / "run-manifest.json", manifest)
    try:
        manifest["adapter_control"] = adapter_control(headless, descriptor, profile)
        if args.mode == "freeze":
            generation_path = output / "generation-manifest.json"
            atomic_write_json(
                generation_path,
                {
                    "schema": "nextengine.canonical-ppo-generation.v1",
                    "closure": closure,
                    "profile": profile,
                    "run_path": str(output / "runs/TRAIN-1"),
                    "adapter_control": manifest["adapter_control"],
                },
            )
            atomic_write_json(
                output / "active-generation.json",
                {
                    "manifest_path": str(generation_path),
                    "manifest_sha256": sha256_file(generation_path),
                },
            )
        if args.mode == "train":
            manifest["evaluation_status"] = train(
                headless, descriptor, profile, output
            )["status"]
        manifest["status"] = "completed"
    except BaseException as error:
        manifest["status"] = "failed"
        manifest["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        manifest["artifacts"] = {
            path.name: sha256_file(path)
            for path in sorted(output.iterdir())
            if path.is_file() and path.name != "run-manifest.json"
        }
        manifest["completed_at_utc"] = datetime.now(timezone.utc).isoformat()
        atomic_write_json(output / "run-manifest.json", manifest)


if __name__ == "__main__":
    main()
