#!/usr/bin/env python3
"""Evaluate one TRAIN-5 reference checkpoint with terminal-branch diagnostics."""

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
    parser.add_argument("--training-profile", type=Path, required=True)
    parser.add_argument("--environment-profile", type=Path, required=True)
    parser.add_argument("--descriptor", type=Path, required=True)
    parser.add_argument("--corpus-root", type=Path, required=True)
    parser.add_argument("--gate-report", type=Path, required=True)
    parser.add_argument("--usd", type=Path, required=True)
    parser.add_argument("--checkpoint", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    AppLauncher.add_app_launcher_args(parser)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    paths = (
        args.training_profile,
        args.environment_profile,
        args.descriptor,
        args.gate_report,
        args.usd,
        args.checkpoint,
    )
    for path in paths:
        if not path.resolve().is_file():
            raise FileNotFoundError(path)
    if not args.corpus_root.resolve().is_dir():
        raise FileNotFoundError(args.corpus_root)

    from next_lab.reference_ppo import TinyReferencePpoProfile

    profile = TinyReferencePpoProfile.load(args.training_profile.resolve())
    document = profile.document
    execution = document["execution"]
    if (
        document["environment_profile_sha256"]
        != _sha256(args.environment_profile.resolve())
        or document["descriptor_sha256"] != _sha256(args.descriptor.resolve())
        or document["usd_sha256"] != _sha256(args.usd.resolve())
    ):
        raise ValueError("checkpoint diagnostic input hash mismatch")
    os.environ["NEXTENGINE_HUMANOID_USD"] = str(args.usd.resolve())
    os.environ["CUBLAS_WORKSPACE_CONFIG"] = execution["cublas_workspace_config"]
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
        trainer = TinyReferencePpoTrainer(environment, profile, Path("/dev/null"))
        checkpoint = torch.load(
            args.checkpoint.resolve(), map_location=execution["device"], weights_only=False
        )
        if (
            checkpoint.get("training_profile_sha256") != profile.sha256
            or checkpoint.get("environment_profile_sha256")
            != document["environment_profile_sha256"]
            or checkpoint.get("descriptor_sha256") != document["descriptor_sha256"]
            or checkpoint.get("usd_sha256") != document["usd_sha256"]
        ):
            raise ValueError("checkpoint provenance mismatch")
        trainer.model.load_state_dict(checkpoint["model"], strict=True)
        evaluation = trainer.evaluate_deterministic(
            int(document["evaluation"]["episodes"])
        )
        report = {
            "schema_version": 1,
            "check": "MOTOR-REFERENCE-ENV-P1-CHECKPOINT-DIAGNOSTIC",
            "status": "PASS",
            "claim": "CheckpointDeterministicTerminalDiagnosticOnly",
            "training_profile_sha256": profile.sha256,
            "environment_profile_sha256": _sha256(
                args.environment_profile.resolve()
            ),
            "descriptor_sha256": _sha256(args.descriptor.resolve()),
            "usd_sha256": _sha256(args.usd.resolve()),
            "checkpoint_sha256": _sha256(args.checkpoint.resolve()),
            "source_optimizer_steps": int(checkpoint["optimizer_steps"]),
            "source_samples": int(checkpoint["samples"]),
            "evaluation_optimizer_steps": 0,
            "evaluation": evaluation,
            "learned_policy_quality_claim": False,
            "tool_sha256": _sha256(Path(__file__).resolve()),
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
