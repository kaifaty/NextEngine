from __future__ import annotations

import os
from pathlib import Path
from typing import Any

import numpy as np


CORRESPONDENCE_SCHEMA_VERSION = 2


def trajectory_metadata(profile: dict[str, Any]) -> dict[str, np.ndarray[Any, Any]]:
    """Return the exact identity fields required by MODEL-MIRROR-P1."""
    return {
        "profile_id": np.asarray(profile["profile_id"], dtype=np.str_),
        "manifest_hash": np.asarray(profile["manifest_hash"], dtype=np.str_),
        "observation_layout_hash": np.asarray(
            profile["observation_layout_hash"], dtype=np.str_
        ),
        "action_layout_hash": np.asarray(
            profile["action_layout_hash"], dtype=np.str_
        ),
        "command_schedule_profile_hash": np.asarray(
            profile["command_schedule_profile_hash"], dtype=np.str_
        ),
        "reward_profile_hash": np.asarray(
            profile["reward_profile_hash"], dtype=np.str_
        ),
        "termination_profile_hash": np.asarray(
            profile["termination_profile_hash"], dtype=np.str_
        ),
        "rng_derivation_profile_hash": np.asarray(
            profile["rng_derivation_profile_hash"], dtype=np.str_
        ),
        "correspondence_profile_hash": np.asarray(
            profile["correspondence_profile_hash"], dtype=np.str_
        ),
        "reward_component_ids": np.asarray(
            [component["component_id"] for component in profile["reward_components"]],
            dtype=np.str_,
        ),
    }


def write_policy_trajectory(
    path: Path,
    *,
    metadata: dict[str, np.ndarray[Any, Any]],
    joint_position_rad: np.ndarray[Any, Any],
    root_position_m: np.ndarray[Any, Any],
    root_velocity_mps: np.ndarray[Any, Any],
    contact_occupancy: np.ndarray[Any, Any],
    done_tick: np.ndarray[Any, Any],
    reward_total_q16: np.ndarray[Any, Any],
    command_raw: np.ndarray[Any, Any],
) -> Path:
    episodes, motor_steps, joints = joint_position_rad.shape
    expected_shapes = {
        "root_position_m": (episodes, motor_steps, 3),
        "root_velocity_mps": (episodes, motor_steps, 3),
        "contact_occupancy": (episodes, motor_steps, 2),
        "done_tick": (episodes,),
        "reward_total_q16": (episodes, motor_steps),
        "command_raw": (episodes, motor_steps, 3),
    }
    if joints != 23:
        raise ValueError("MODEL-MIRROR trajectory must contain 23 joints")
    values = {
        "root_position_m": root_position_m,
        "root_velocity_mps": root_velocity_mps,
        "contact_occupancy": contact_occupancy,
        "done_tick": done_tick,
        "reward_total_q16": reward_total_q16,
        "command_raw": command_raw,
    }
    for name, shape in expected_shapes.items():
        if values[name].shape != shape:
            raise ValueError(f"{name} shape mismatch: expected {shape}, got {values[name].shape}")
    for name in ("joint_position_rad", "root_position_m", "root_velocity_mps"):
        value = joint_position_rad if name == "joint_position_rad" else values[name]
        if not np.isfinite(value).all():
            raise ValueError(f"non-finite trajectory values: {name}")

    path = path.resolve()
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".tmp")
    payload = {
        "schema_version": np.asarray(CORRESPONDENCE_SCHEMA_VERSION, dtype=np.uint16),
        **metadata,
        "joint_position_rad": np.asarray(joint_position_rad, dtype=np.float32),
        "root_position_m": np.asarray(root_position_m, dtype=np.float32),
        "root_velocity_mps": np.asarray(root_velocity_mps, dtype=np.float32),
        "contact_occupancy": np.asarray(contact_occupancy, dtype=np.bool_),
        "done_tick": np.asarray(done_tick, dtype=np.int64),
        "reward_total_q16": np.asarray(reward_total_q16, dtype=np.int64),
        "command_raw": np.asarray(command_raw, dtype=np.int64),
    }
    try:
        with temporary.open("wb") as output:
            np.savez_compressed(output, **payload)
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)
    return path
