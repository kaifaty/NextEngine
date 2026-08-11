#!/usr/bin/env python3
"""Train the private NextEngine Isaac/RSL-RL mirror with a closed run manifest."""

from __future__ import annotations

import argparse
import copy
import importlib.metadata
import json
import os
import subprocess
import traceback
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from isaaclab.app import AppLauncher

from next_lab.isaac_training import (
    RUN_MANIFEST_SCHEMA,
    IsaacTrainingProfile,
    ResolvedTrainingConfig,
    atomic_write_json,
    checkpoint_records,
    parse_gpu_memory_csv,
    require_external_path,
    sha256_file,
    training_config_hash,
    validate_resume_checkpoint,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--log-root", type=Path, required=True)
    parser.add_argument("--num-envs", type=int)
    parser.add_argument("--steps-per-env", type=int)
    parser.add_argument("--iterations", type=int)
    parser.add_argument("--save-interval", type=int)
    parser.add_argument("--seed", type=int)
    parser.add_argument("--training-device")
    parser.add_argument("--resume", type=Path)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    profile = IsaacTrainingProfile.load(args.profile.resolve())
    config = ResolvedTrainingConfig.from_profile(
        profile,
        num_envs=args.num_envs,
        steps_per_env=args.steps_per_env,
        iterations=args.iterations,
        save_interval=args.save_interval,
        seed=args.seed,
        device=args.training_device,
    )
    descriptor = require_external_path(
        args.descriptor, REPOSITORY_ROOT, label="engine descriptor"
    )
    usd = require_external_path(args.usd, REPOSITORY_ROOT, label="derived humanoid USD")
    log_root = require_external_path(
        args.log_root, REPOSITORY_ROOT, label="training log root", must_exist=False
    )
    log_root.mkdir(parents=True, exist_ok=True)
    descriptor_hash = sha256_file(descriptor)
    usd_hash = sha256_file(usd)
    config_hash = training_config_hash(config, descriptor_hash, usd_hash)
    gpu = query_gpu(config.device)
    if gpu["memory_free_mib"] < profile.min_free_gpu_memory_mib:
        raise RuntimeError(
            "insufficient free GPU memory: "
            f"{gpu['memory_free_mib']} MiB available, "
            f"{profile.min_free_gpu_memory_mib} MiB required"
        )

    parent: dict[str, Any] | None = None
    resume = None
    if args.resume is not None:
        resume = require_external_path(
            args.resume, REPOSITORY_ROOT, label="resume checkpoint"
        )
        parent_manifest = validate_resume_checkpoint(resume, config_hash)
        parent = {
            "checkpoint": str(resume),
            "checkpoint_sha256": sha256_file(resume),
            "run_id": parent_manifest.get("run_id"),
        }

    run_id = (
        datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S.%fZ")
        + "-"
        + config.run_root_hex[:12]
    )
    run_dir = log_root / run_id
    run_dir.mkdir(parents=False, exist_ok=False)
    manifest_path = run_dir / "run-manifest.json"
    manifest: dict[str, Any] = {
        "schema": RUN_MANIFEST_SCHEMA,
        "status": "running",
        "run_id": run_id,
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "repository": repository_state(),
        "profile_path": str(args.profile.resolve()),
        "training_config": config.as_dict(),
        "training_config_hash": config_hash,
        "artifacts": {
            "descriptor": {"path": str(descriptor), "sha256": descriptor_hash},
            "usd": {"path": str(usd), "sha256": usd_hash},
        },
        "gpu_preflight": gpu,
        "parent": parent,
        "checkpoints": [],
    }
    atomic_write_json(manifest_path, manifest)

    simulation_app = None
    wrapped = None
    try:
        os.environ["NEXTENGINE_HUMANOID_USD"] = str(usd)
        args.device = config.device
        app_launcher = AppLauncher(args)
        simulation_app = app_launcher.app

        from isaaclab_rl.rsl_rl import RslRlVecEnvWrapper

        from next_lab.isaac_env import (
            NextEngineHumanoidDirectEnv,
            NextEngineHumanoidDirectEnvCfg,
        )
        from next_lab.isaac_rl import GuardedOnPolicyRunner, build_agent_cfg

        env_cfg = NextEngineHumanoidDirectEnvCfg()
        env_cfg.scene.num_envs = config.num_envs
        env_cfg.seed = config.seed
        env_cfg.run_root_hex = config.run_root_hex
        env_cfg.environment_profile_id = profile.environment_profile_id
        env_cfg.sim.device = config.device
        environment = NextEngineHumanoidDirectEnv(
            env_cfg,
            descriptor_path=str(descriptor),
        )
        wrapped = RslRlVecEnvWrapper(environment, clip_actions=1.0)
        agent_cfg = build_agent_cfg(config)
        runner = GuardedOnPolicyRunner(
            wrapped,
            copy.deepcopy(agent_cfg.to_dict()),
            log_dir=str(run_dir),
            device=config.device,
            metrics_path=run_dir / "metrics.jsonl",
        )
        if resume is not None:
            runner.load(str(resume), load_optimizer=True, map_location=config.device)
            runner.current_learning_iteration += 1
        manifest["packages"] = package_versions()
        atomic_write_json(manifest_path, manifest)

        runner.learn(
            num_learning_iterations=config.iterations,
            init_at_random_ep_len=False,
        )
        manifest.update(
            {
                "status": "completed",
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "samples": config.num_envs * config.steps_per_env * config.iterations,
                "gpu_postflight": query_gpu(config.device),
                "checkpoints": checkpoint_records(run_dir),
            }
        )
        atomic_write_json(manifest_path, manifest)
        print(
            json.dumps(
                {
                    "status": "completed",
                    "run_dir": str(run_dir),
                    "samples": manifest["samples"],
                    "checkpoints": manifest["checkpoints"],
                },
                indent=2,
                sort_keys=True,
            ),
            flush=True,
        )
    except BaseException as error:
        manifest.update(
            {
                "status": "failed",
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "error": {
                    "type": type(error).__name__,
                    "message": str(error)[:2000],
                },
                "checkpoints": checkpoint_records(run_dir),
            }
        )
        atomic_write_json(manifest_path, manifest)
        traceback.print_exc()
        raise
    finally:
        if wrapped is not None:
            wrapped.close()
        if simulation_app is not None:
            simulation_app.close()


def query_gpu(device: str) -> dict[str, int | str]:
    index = int(device.removeprefix("cuda:"))
    result = subprocess.run(
        [
            "nvidia-smi",
            "--query-gpu=index,name,memory.total,memory.free",
            "--format=csv,noheader,nounits",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    for line in result.stdout.splitlines():
        record = parse_gpu_memory_csv(line)
        if record["index"] == index:
            return record
    raise RuntimeError(f"CUDA device is not reported by nvidia-smi: {device}")


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
    return {"root": str(REPOSITORY_ROOT), "commit": commit, "dirty": bool(status)}


def package_versions() -> dict[str, str]:
    versions = {}
    for distribution in ("torch", "rsl-rl", "isaaclab", "isaacsim", "next-lab"):
        try:
            versions[distribution] = importlib.metadata.version(distribution)
        except importlib.metadata.PackageNotFoundError:
            versions[distribution] = "not-installed-as-distribution"
    return versions


if __name__ == "__main__":
    main()
