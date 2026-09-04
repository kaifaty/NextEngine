#!/usr/bin/env python3
"""Evaluate a closed biomechanics standing or start/stop policy in CPU PhysX."""

from __future__ import annotations

import argparse
import json
import math
import statistics
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import numpy as np
import torch
from torch import nn

from next_lab.isaac_training import (
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    atomic_write_json,
    load_active_training_generation,
    require_external_path,
    require_generation_output_path,
    sha256_file,
    validate_checkpoint_artifacts,
    validate_closed_checkpoint,
)
from next_lab.motor_lab_client import (
    BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
    BIOMECHANICS_STANDING_PROFILE_ID,
    MotorLabClient,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
Q1_30_ONE = 1 << 30


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generation-index", type=Path, required=True)
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--headless", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--episodes", type=int, default=5)
    parser.add_argument("--max-steps", type=int, default=3_600)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.episodes <= 0 or args.max_steps <= 0:
        raise ValueError("episodes and max-steps must be positive")
    generation_index = require_external_path(
        args.generation_index, REPOSITORY_ROOT, label="active training generation"
    )
    generation = load_active_training_generation(generation_index)
    profile_path = args.profile.resolve()
    profile = IsaacTrainingProfile.load(profile_path)
    if profile.environment_profile_id not in {
        BIOMECHANICS_STANDING_PROFILE_ID,
        BIOMECHANICS_FORWARD_START_STOP_PROFILE_ID,
    }:
        raise ValueError(
            "CPU policy evaluator accepts only biomechanics standing or forward start/stop"
        )
    descriptor_path = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd_path = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")
    generation.manifest.require_input(
        profile, sha256_file(descriptor_path), sha256_file(usd_path)
    )
    checkpoint_path = require_external_path(
        args.checkpoint, REPOSITORY_ROOT, label="checkpoint"
    )
    parent = validate_closed_checkpoint(
        checkpoint_path, generation.manifest.generation_id
    )
    validate_checkpoint_artifacts(
        parent, profile, descriptor_path, usd_path
    )
    headless_path = args.headless.resolve()
    if not headless_path.is_file():
        raise FileNotFoundError(f"headless executable does not exist: {headless_path}")
    output_root = require_generation_output_path(
        require_external_path(
            args.output_root,
            REPOSITORY_ROOT,
            label="CPU evaluation root",
            must_exist=False,
        ),
        generation,
        label="CPU evaluation root",
    )
    output_root.mkdir(parents=True, exist_ok=True)

    descriptor = json.loads(descriptor_path.read_text(encoding="utf-8"))
    actor, mean, std = load_actor(checkpoint_path)
    scales = observation_scales(descriptor)
    checkpoint_hash = sha256_file(checkpoint_path)
    evaluation_id = (
        datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
        + f"-cpu-{checkpoint_hash[:12]}"
    )
    evaluation_dir = output_root / evaluation_id
    evaluation_dir.mkdir(parents=False, exist_ok=False)
    manifest_path = evaluation_dir / "evaluation-manifest.json"

    run_root = ResolvedTrainingConfig.from_profile(profile).run_root_hex
    episode_returns: list[float] = []
    episode_lengths: list[int] = []
    terminal_reasons: Counter[str] = Counter()
    terminal_dispositions: Counter[str] = Counter()
    minimum_root_heights: list[float] = []
    maximum_tilts_degrees: list[float] = []
    final_backward_leans_degrees: list[float] = []
    contact_occupancies: list[float] = []
    action_absolute_maxima: list[float] = []

    with MotorLabClient(headless_path, profile.environment_profile_id, 1, run_root) as client:
        if client.descriptor.maximum_episode_steps != args.max_steps:
            raise ValueError(
                "max-steps must equal the closed environment bound: "
                f"{client.descriptor.maximum_episode_steps}"
            )
        for _ in range(args.episodes):
            reset = client.reset([0])[0]
            observation = policy_observation(reset.observation_raw, scales)
            episode_return = 0.0
            minimum_height = math.inf
            maximum_tilt = 0.0
            contact_samples = 0
            action_absolute_maximum = 0.0
            final_pitch = 0.0
            final_step = None
            with torch.inference_mode():
                for _step in range(args.max_steps):
                    normalized = (observation - mean) / (std + 1.0e-2)
                    action = actor(normalized).cpu().numpy()
                    if not np.isfinite(action).all():
                        raise RuntimeError("CPU policy produced non-finite action")
                    result = client.step_normalized(
                        [reset.episode_ordinal], action
                    )[0]
                    episode_return += result.reward_total_q16 / 65_536.0
                    minimum_height = min(
                        minimum_height,
                        result.root_position_micrometres[1] / 1_000_000.0,
                    )
                    tilt, pitch = root_tilt_and_pitch_degrees(
                        result.root_quaternion_q1_30_xyzw
                    )
                    maximum_tilt = max(maximum_tilt, tilt)
                    final_pitch = pitch
                    contact_samples += int(result.contact_flags.sum())
                    action_absolute_maximum = max(
                        action_absolute_maximum, float(np.max(np.abs(action)))
                    )
                    final_step = result
                    observation = policy_observation(result.observation_raw, scales)
                    if result.terminated or result.truncated:
                        break
            if final_step is None:
                raise RuntimeError("CPU policy evaluation produced no steps")
            disposition = "terminated" if final_step.terminated else "truncated"
            if not (final_step.terminated or final_step.truncated):
                disposition = "budget-exhausted"
            terminal_dispositions[disposition] += 1
            terminal_reasons[final_step.terminal_reason_id or "none"] += 1
            episode_returns.append(episode_return)
            episode_lengths.append(final_step.motor_tick)
            minimum_root_heights.append(minimum_height)
            maximum_tilts_degrees.append(maximum_tilt)
            final_backward_leans_degrees.append(final_pitch)
            contact_occupancies.append(
                contact_samples / (2.0 * final_step.motor_tick)
            )
            action_absolute_maxima.append(action_absolute_maximum)

        result: dict[str, Any] = {
            "schema": "nextengine.motor.cpu-policy-evaluation.v1",
            "status": "completed",
            "claim": "CanonicalCpuPhysXPolicyEvaluation",
            "evaluation_id": evaluation_id,
            "completed_at_utc": datetime.now(timezone.utc).isoformat(),
            "training_generation_id": generation.manifest.generation_id,
            "training_generation_manifest_hash": generation.manifest.manifest_hash,
            "repository": repository_state(),
            "profile_id": profile.profile_id,
            "training_profile_sha256": sha256_file(profile_path),
            "environment_profile_id": client.descriptor.profile_id,
            "environment_manifest_hash": client.descriptor.manifest_hash.hex(),
            "run_root_hex": run_root,
            "checkpoint": {
                "path": str(checkpoint_path),
                "sha256": checkpoint_hash,
                "parent_run_id": parent.get("run_id"),
            },
            "episodes": args.episodes,
            "maximum_episode_steps": args.max_steps,
            "returns": summary(episode_returns),
            "lengths": summary(episode_lengths),
            "termination_dispositions": dict(sorted(terminal_dispositions.items())),
            "terminal_reasons": dict(sorted(terminal_reasons.items())),
            "minimum_root_height_metres": summary(minimum_root_heights),
            "maximum_root_tilt_degrees": summary(maximum_tilts_degrees),
            "final_signed_pitch_degrees": summary(final_backward_leans_degrees),
            "two_sole_contact_occupancy": summary(contact_occupancies),
            "policy_action_absolute_maximum": summary(action_absolute_maxima),
        }
        atomic_write_json(manifest_path, result)
        print(json.dumps(result, indent=2, sort_keys=True), flush=True)


def load_actor(checkpoint_path: Path) -> tuple[nn.Sequential, torch.Tensor, torch.Tensor]:
    checkpoint = torch.load(checkpoint_path, map_location="cpu", weights_only=False)
    state = checkpoint.get("model_state_dict")
    if not isinstance(state, dict):
        raise ValueError("checkpoint has no model_state_dict")
    actor = nn.Sequential(
        nn.Linear(84, 256),
        nn.ELU(),
        nn.Linear(256, 128),
        nn.ELU(),
        nn.Linear(128, 64),
        nn.ELU(),
        nn.Linear(64, 23),
    )
    actor.load_state_dict(
        {
            key.removeprefix("actor."): value
            for key, value in state.items()
            if key.startswith("actor.")
        },
        strict=True,
    )
    actor.eval()
    mean = state["actor_obs_normalizer._mean"].to(dtype=torch.float32)
    std = state["actor_obs_normalizer._std"].to(dtype=torch.float32)
    if mean.shape != (1, 84) or std.shape != (1, 84):
        raise ValueError("checkpoint actor observation normalizer shape mismatch")
    return actor, mean, std


def observation_scales(descriptor: dict[str, Any]) -> np.ndarray:
    joints = {joint["joint_id"]: joint for joint in descriptor["joints"]}
    ordered = [joints[actuator["joint_id"]] for actuator in descriptor["actuators"]]
    position = [
        max(abs(joint["soft_limit_microradians"][0]), abs(joint["soft_limit_microradians"][1]))
        for joint in ordered
    ]
    velocity = [joint["maximum_velocity_microradians_per_second"] for joint in ordered]
    scales = np.asarray(
        [Q1_30_ONE] * 4
        + [2_000_000] * 3
        + [2_000_000] * 3
        + position
        + velocity
        + position
        + [1_000_000] * 3
        + [1, 1],
        dtype=np.float64,
    )
    if scales.shape != (84,) or np.any(scales <= 0):
        raise ValueError("descriptor produced an invalid observation scale")
    return scales


def policy_observation(raw: np.ndarray, scales: np.ndarray) -> torch.Tensor:
    values = np.asarray(raw, dtype=np.float64)
    if values.shape != (84,):
        raise ValueError("CPU observation width mismatch")
    normalized = values / scales
    if not np.isfinite(normalized).all():
        raise RuntimeError("CPU observation contains non-finite values")
    return torch.from_numpy(normalized.astype(np.float32, copy=False)).unsqueeze(0)


def root_tilt_and_pitch_degrees(quaternion_q1_30: np.ndarray) -> tuple[float, float]:
    x, y, z, w = np.asarray(quaternion_q1_30, dtype=np.float64) / Q1_30_ONE
    up_y = max(-1.0, min(1.0, 1.0 - 2.0 * (x * x + z * z)))
    tilt = math.degrees(math.acos(up_y))
    pitch = math.degrees(math.atan2(2.0 * (w * x + y * z), 1.0 - 2.0 * (x * x + y * y)))
    return tilt, pitch


def summary(values: list[float] | list[int]) -> dict[str, float | int]:
    return {
        "count": len(values),
        "mean": statistics.mean(values),
        "standard_deviation": statistics.pstdev(values),
        "minimum": min(values),
        "maximum": max(values),
    }


def repository_state() -> dict[str, Any]:
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    status = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return {"commit": commit, "dirty": bool(status.strip())}


if __name__ == "__main__":
    main()
