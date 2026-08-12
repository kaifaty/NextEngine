#!/usr/bin/env python3
"""Run the hash-closed TRAIN-5 one-clip deterministic PPO overfit."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import traceback
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from isaaclab.app import AppLauncher


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--generation-manifest", type=Path, required=True)
    parser.add_argument("--training-profile", type=Path, required=True)
    parser.add_argument("--environment-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--output-root", type=Path, required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--iterations", type=int)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    for path in (
        args.generation_manifest,
        args.training_profile,
        args.environment_profile,
        args.descriptor,
        args.gate_report,
        args.usd,
    ):
        if not path.resolve().is_file():
            raise FileNotFoundError(path)
    if not args.corpus_root.resolve().is_dir():
        raise FileNotFoundError(args.corpus_root)
    output_root = args.output_root.resolve()
    if output_root == REPOSITORY_ROOT or REPOSITORY_ROOT in output_root.parents:
        raise ValueError("training output must remain outside the repository")

    from next_lab.reference_ppo import TinyReferencePpoProfile

    training_profile = TinyReferencePpoProfile.load(args.training_profile.resolve())
    document = dict(training_profile.document)
    execution = dict(document["execution"])
    if args.iterations is not None:
        if args.iterations <= 0 or args.iterations > execution["iterations"]:
            raise ValueError("iteration override must be positive and no larger than frozen budget")
        execution["iterations"] = args.iterations
        document["execution"] = execution
        training_profile = TinyReferencePpoProfile(
            document=document,
            sha256=training_profile.sha256,
        )
    input_hashes = {
        "training_profile": _sha256(args.training_profile.resolve()),
        "environment_profile": _sha256(args.environment_profile.resolve()),
        "descriptor": _sha256(args.descriptor.resolve()),
        "gate_report": _sha256(args.gate_report.resolve()),
        "usd": _sha256(args.usd.resolve()),
        "generation_manifest": _sha256(args.generation_manifest.resolve()),
    }
    _validate_profile_inputs(training_profile.document, input_hashes)
    generation = json.loads(args.generation_manifest.resolve().read_text(encoding="utf-8"))
    _validate_generation(generation, input_hashes, training_profile.sha256)
    run_dir = output_root / args.run_id
    run_dir.mkdir(parents=True, exist_ok=False)
    manifest_path = run_dir / "run-manifest.json"
    config_hash = hashlib.sha256(
        _canonical_json(
            {
                "training_profile_sha256": training_profile.sha256,
                "resolved_execution": execution,
                "input_hashes": input_hashes,
                "generation_manifest_hash": generation["manifest_hash"],
            }
        )
    ).hexdigest()
    manifest: dict[str, Any] = {
        "schema_version": 1,
        "schema_id": "nextengine.training.humanoid-reference-overfit-run.v1",
        "status": "running",
        "claim": "TrainingExecutionOnly",
        "run_id": args.run_id,
        "created_at_utc": datetime.now(timezone.utc).isoformat(),
        "training_generation_id": generation["training_generation_id"],
        "training_generation_manifest_hash": generation["manifest_hash"],
        "training_profile_sha256": training_profile.sha256,
        "resolved_config_hash": config_hash,
        "resolved_execution": execution,
        "input_hashes": input_hashes,
        "repository": _repository_state(),
        "training_runs": 1,
        "optimizer_steps": 0,
        "samples": 0,
        "learned_policy_claim": False,
    }
    _write_json(manifest_path, manifest)
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
        from next_lab.reference_ppo import TinyReferencePpoTrainer

        seed = int(execution["seed"])
        torch.manual_seed(seed)
        torch.cuda.manual_seed_all(seed)
        torch.use_deterministic_algorithms(
            bool(execution["torch_deterministic_algorithms"])
        )
        cfg = NextEngineReferenceDirectEnvCfg()
        cfg.scene.num_envs = int(execution["num_envs"])
        cfg.sim.device = execution["device"]
        cfg.seed = seed
        cfg.fixed_clip_id = document["scope"]["clip_id"]
        cfg.fixed_start_frame = int(document["scope"]["start_frame"])
        cfg.fixed_horizon_motor_ticks = int(document["scope"]["horizon_motor_ticks"])
        environment = NextEngineReferenceDirectEnv(
            cfg,
            descriptor_path=str(args.descriptor.resolve()),
            profile_path=str(args.environment_profile.resolve()),
            corpus_root=str(args.corpus_root.resolve()),
            gate_report_path=str(args.gate_report.resolve()),
        )
        trainer = TinyReferencePpoTrainer(
            environment,
            training_profile,
            run_dir / "metrics.jsonl",
        )
        evaluation_episodes = int(document["evaluation"]["episodes"])
        initial_evaluation = trainer.evaluate_deterministic(evaluation_episodes)
        records = trainer.train()
        final_evaluation = trainer.evaluate_deterministic(evaluation_episodes)
        accepted = (
            final_evaluation["reference_complete_count"]
            > initial_evaluation["reference_complete_count"]
            and final_evaluation["mean_episode_length"]
            > initial_evaluation["mean_episode_length"]
        )
        checkpoint_path = run_dir / "final-checkpoint.pt"
        torch.save(
            {
                "schema_version": 1,
                "training_profile_sha256": training_profile.sha256,
                "resolved_config_hash": config_hash,
                "environment_profile_sha256": input_hashes["environment_profile"],
                "descriptor_sha256": input_hashes["descriptor"],
                "usd_sha256": input_hashes["usd"],
                "model": trainer.model.state_dict(),
                "optimizer": trainer.optimizer.state_dict(),
                "optimizer_steps": trainer.optimizer_steps,
                "samples": trainer.samples,
            },
            checkpoint_path,
        )
        manifest.update(
            {
                "status": "completed",
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "claim": "TinyDeterministicOneClipOverfitOnly" if accepted else "TrainingExecutionOnly",
                "optimizer_steps": trainer.optimizer_steps,
                "samples": trainer.samples,
                "learned_policy_claim": accepted,
                "overfit_acceptance": "PASS" if accepted else "FAIL",
                "initial_evaluation": initial_evaluation,
                "final_evaluation": final_evaluation,
                "final_metrics": records[-1],
                "checkpoint": {
                    "file": checkpoint_path.name,
                    "sha256": _sha256(checkpoint_path),
                },
                "metrics": {
                    "file": "metrics.jsonl",
                    "sha256": _sha256(run_dir / "metrics.jsonl"),
                    "record_count": len(records),
                },
            }
        )
        _write_json(manifest_path, manifest)
        print(json.dumps(manifest, indent=2, sort_keys=True), flush=True)
    except BaseException as error:
        manifest.update(
            {
                "status": "failed",
                "completed_at_utc": datetime.now(timezone.utc).isoformat(),
                "error": {"type": type(error).__name__, "message": str(error)[:2000]},
            }
        )
        _write_json(manifest_path, manifest)
        traceback.print_exc()
        raise
    finally:
        if environment is not None:
            environment.close()
        if simulation_app is not None:
            simulation_app.close()


def _validate_profile_inputs(
    profile: Mapping[str, Any], hashes: Mapping[str, str]
) -> None:
    if (
        profile["environment_profile_sha256"] != hashes["environment_profile"]
        or profile["descriptor_sha256"] != hashes["descriptor"]
        or profile["usd_sha256"] != hashes["usd"]
    ):
        raise ValueError("tiny PPO profile input hash mismatch")


def _validate_generation(
    generation: Mapping[str, Any], hashes: Mapping[str, str], training_profile_sha256: str
) -> None:
    if (
        generation.get("schema") != "nextengine.humanoid-training-generation.v1"
        or generation.get("status") != "train-5-preacceptance"
    ):
        raise ValueError("training generation is not authorized for TRAIN-5 overfit")
    hash_input = dict(generation)
    expected_manifest_hash = hash_input.pop("manifest_hash", None)
    actual_manifest_hash = hashlib.sha256(
        json.dumps(
            hash_input, ensure_ascii=True, separators=(",", ":"), sort_keys=True
        ).encode("utf-8")
    ).hexdigest()
    if expected_manifest_hash != actual_manifest_hash:
        raise ValueError("training generation manifest hash mismatch")
    admitted = {
        (entry.get("kind"), entry.get("sha256"))
        for entry in generation.get("admitted_inputs", [])
    }
    required = {
        ("reference_tracker_profile", hashes["environment_profile"]),
        ("compiled_descriptor_file", hashes["descriptor"]),
        ("isaac_biomechanics_usd", hashes["usd"]),
        ("tiny_overfit_profile", training_profile_sha256),
        ("train_4_gate_report", hashes["gate_report"]),
    }
    if not required.issubset(admitted):
        raise ValueError("training generation does not admit the exact overfit inputs")


def _repository_state() -> dict[str, Any]:
    commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=REPOSITORY_ROOT,
        text=True,
        check=True,
        capture_output=True,
    ).stdout.strip()
    dirty = bool(
        subprocess.run(
            ["git", "status", "--porcelain"],
            cwd=REPOSITORY_ROOT,
            text=True,
            check=True,
            capture_output=True,
        ).stdout
    )
    if dirty:
        raise RuntimeError("tiny overfit requires a clean implementation commit")
    return {"commit": commit, "dirty": False}


def _canonical_json(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def _write_json(path: Path, value: Mapping[str, Any]) -> None:
    payload = json.dumps(value, indent=2, sort_keys=True) + "\n"
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(payload, encoding="utf-8")
    temporary.replace(path)


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


if __name__ == "__main__":
    main()
