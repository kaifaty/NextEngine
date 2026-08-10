from __future__ import annotations

import hashlib
import json
import math
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

import numpy as np


@dataclass(frozen=True)
class CorrespondenceThresholds:
    joint_position_rmse_rad: float = 0.02
    root_position_rmse_m: float = 0.03
    root_velocity_rmse_mps: float = 0.05
    contact_occupancy_agreement: float = 0.98
    done_tick_agreement: float = 0.95


REQUIRED_KEYS = (
    "joint_position_rad",
    "root_position_m",
    "root_velocity_mps",
    "contact_occupancy",
    "done_tick",
)


def evaluate_correspondence(
    cpu: dict[str, np.ndarray[Any, Any]],
    gpu: dict[str, np.ndarray[Any, Any]],
    *,
    thresholds: CorrespondenceThresholds = CorrespondenceThresholds(),
    minimum_episodes: int = 256,
    minimum_motor_steps: int = 600,
) -> dict[str, Any]:
    for key in REQUIRED_KEYS:
        if key not in cpu or key not in gpu or cpu[key].shape != gpu[key].shape:
            raise ValueError(f"trajectory key/shape mismatch: {key}")
    episode_count = int(cpu["joint_position_rad"].shape[0])
    motor_steps = int(cpu["joint_position_rad"].shape[1])
    if episode_count < minimum_episodes or motor_steps < minimum_motor_steps:
        raise ValueError("trajectory does not meet the correspondence sample floor")
    for key in REQUIRED_KEYS[:3]:
        if not np.isfinite(cpu[key]).all() or not np.isfinite(gpu[key]).all():
            raise ValueError(f"non-finite trajectory values: {key}")
    metrics = {
        "joint_position_rmse_rad": _rmse(cpu["joint_position_rad"], gpu["joint_position_rad"]),
        "root_position_rmse_m": _rmse(cpu["root_position_m"], gpu["root_position_m"]),
        "root_velocity_rmse_mps": _rmse(cpu["root_velocity_mps"], gpu["root_velocity_mps"]),
        "contact_occupancy_agreement": float(
            np.mean(cpu["contact_occupancy"] == gpu["contact_occupancy"])
        ),
        "done_tick_agreement": float(np.mean(cpu["done_tick"] == gpu["done_tick"])),
    }
    gates = {
        "joint_position": metrics["joint_position_rmse_rad"] <= thresholds.joint_position_rmse_rad,
        "root_position": metrics["root_position_rmse_m"] <= thresholds.root_position_rmse_m,
        "root_velocity": metrics["root_velocity_rmse_mps"] <= thresholds.root_velocity_rmse_mps,
        "contact_occupancy": metrics["contact_occupancy_agreement"]
        >= thresholds.contact_occupancy_agreement,
        "done_tick": metrics["done_tick_agreement"] >= thresholds.done_tick_agreement,
    }
    return {
        "schema_version": 1,
        "check": "MODEL-MIRROR",
        "status": "passed" if all(gates.values()) else "failed",
        "episodes": episode_count,
        "motor_steps_per_episode": motor_steps,
        "metrics": metrics,
        "thresholds": asdict(thresholds),
        "gates": gates,
    }


def evaluate_files(cpu_path: Path, gpu_path: Path, store_root: Path) -> tuple[dict[str, Any], Path]:
    with np.load(cpu_path, allow_pickle=False) as archive:
        cpu = {key: archive[key] for key in REQUIRED_KEYS}
    with np.load(gpu_path, allow_pickle=False) as archive:
        gpu = {key: archive[key] for key in REQUIRED_KEYS}
    report = evaluate_correspondence(cpu, gpu)
    report["inputs"] = {
        "cpu_sha256": _file_hash(cpu_path),
        "gpu_sha256": _file_hash(gpu_path),
    }
    output = store_root.resolve() / "correspondence" / report["inputs"]["cpu_sha256"][:16]
    output.mkdir(parents=True, exist_ok=True)
    path = output / "model-mirror-report.json"
    temporary = path.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, path)
    return report, path


def _rmse(left: np.ndarray[Any, Any], right: np.ndarray[Any, Any]) -> float:
    return math.sqrt(float(np.mean(np.square(left.astype(np.float64) - right.astype(np.float64)))))


def _file_hash(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()
