#!/usr/bin/env python3
"""ADR-111 final-only V7 weights on V8; no optimizer and no source mutations."""

from __future__ import annotations

import argparse
import copy
import importlib.metadata
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

import numpy as np
import torch
from next_lab.canonical_ppo import observation_scales, shard_root
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from next_lab.motor_lab_client import normalized_action_to_raw
from rsl_rl.modules import ActorCritic
from tensordict import TensorDict

from lab.scripts.canonical_walking_ppo import (
    artifact_hashes,
    evaluate,
    finite_policy,
    seed_root,
)
from lab.scripts.cpu_walking_contact_audit import analyze, lifted_support_report, plot

ROOT = Path(__file__).resolve().parents[2]
PROFILE = ROOT / "lab/profiles/canonical-walking-corrected-evaluation.v1.json"
TRAINING_PROFILE = ROOT / "lab/profiles/canonical-rsl-rl-walking.v3.json"


def read_json(path):
    return json.loads(path.read_text())


def require_hash(path, expected):
    actual = sha256_file(path)
    if actual != expected:
        raise ValueError(f"hash mismatch: {path}")
    return actual


def check_compatibility(source, target):
    """Allow only the exact descriptor field changes admitted by ADR-110."""
    repaired = copy.deepcopy(target)
    if source["training_descriptor_id"] != (
        "nextengine.canonical.humanoid-biomechanics-forward-start-stop.v7"
    ) or target["training_descriptor_id"] != (
        "nextengine.canonical.humanoid-biomechanics-forward-start-stop.v8"
    ):
        raise ValueError("requires exact V7 to V8 descriptor identities")
    if (
        len(source["environment_profiles"]) != 1
        or len(target["environment_profiles"]) != 1
    ):
        raise ValueError("requires one environment per descriptor")
    a, b = source["environment_profiles"][0], repaired["environment_profiles"][0]
    if (
        a["profile_id"]
        != "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v7"
        or b["profile_id"]
        != "nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v8"
    ):
        raise ValueError("environment identity mismatch")
    command = b["command_profile"]
    changes = {
        "kind": "fixed-forward-start-stop-v3",
        "profile_id": "nextengine.motor.command.biomechanics-forward-start-stop.v3",
        "ramp_down_tick": 990,
        "applied_action_indices_inclusive": [0, 1199],
        "final_zero_action_indices_inclusive": [1020, 1199],
    }
    expected_command = {**a["command_profile"], **changes}
    if command != expected_command:
        raise ValueError("unexpected command change")
    b["command_profile"] = a["command_profile"]
    for key in (
        "profile_id",
        "manifest_hash",
        "command_schedule_profile_hash",
        "correspondence_profile_hash",
    ):
        b[key] = a[key]
    repaired["training_descriptor_id"] = source["training_descriptor_id"]
    if repaired != source:
        raise ValueError(
            "incompatible body/action/observation/reward/safety descriptor"
        )
    np.testing.assert_array_equal(
        observation_scales(source), observation_scales(target)
    )


def validate_closed_source(run, generation_path, target_path, matrix):
    """Read-only preflight. All artifacts must be closed before torch.load."""
    generation_hash = require_hash(generation_path, matrix["source_generation_sha256"])
    generation = read_json(generation_path)
    if (
        generation["schema"] != "nextengine.canonical-ppo-generation.v1"
        or Path(generation["run_path"]).resolve() != run.resolve()
    ):
        raise ValueError("source generation/run identity mismatch")
    manifest_path = run / "run-manifest.json"
    manifest_hash = sha256_file(manifest_path)
    manifest = read_json(manifest_path)
    if manifest.get("status") != "completed" or manifest.get("mode") != "train":
        raise ValueError("source training must be completed and closed")
    if (
        manifest["schema"] != "nextengine.canonical-ppo-run.v1"
        or manifest["generation_manifest_sha256"] != generation_hash
    ):
        raise ValueError("source run schema/generation mismatch")
    if manifest["repository_dirty"] or manifest["input_checkpoint"] is not None:
        raise ValueError("source must be the clean fresh-weight run")
    profile_hash = require_hash(
        TRAINING_PROFILE, matrix["source_training_profile_sha256"]
    )
    profile = read_json(TRAINING_PROFILE)
    if (
        manifest["profile"] != profile
        or generation["profile"] != profile
        or profile["evaluation"] != matrix["evaluation"]
    ):
        raise ValueError("source profile or evaluation matrix mismatch")
    closure = generation["closure"]
    for key in (
        "repository_commit",
        "profile_sha256",
        "descriptor_sha256",
        "headless_sha256",
        "dependencies",
    ):
        if manifest[key] != closure[key]:
            raise ValueError(f"source closure mismatch: {key}")
    if manifest["profile_sha256"] != profile_hash:
        raise ValueError("source training profile hash mismatch")
    source_path = require_external_path(
        Path(manifest["descriptor_path"]), ROOT, label="source descriptor"
    )
    require_hash(source_path, matrix["source_descriptor_sha256"])
    if manifest["descriptor_sha256"] != matrix["source_descriptor_sha256"]:
        raise ValueError("source descriptor closure mismatch")
    require_hash(target_path, matrix["target_descriptor_sha256"])
    source, target = read_json(source_path), read_json(target_path)
    check_compatibility(source, target)
    artifacts = manifest["artifacts"]
    required = {"metrics.jsonl", "evaluation.json", matrix["source_checkpoint_name"]}
    required.update(f"evaluation-{seed}.npz" for seed in matrix["evaluation"]["seeds"])
    if not required.issubset(artifacts):
        raise ValueError("source artifact closure is incomplete")
    for relative, digest in artifacts.items():
        path = Path(relative)
        if (
            path.is_absolute()
            or ".." in path.parts
            or not (run / path).resolve().is_relative_to(run.resolve())
        ):
            raise ValueError("source artifact path escapes run")
        require_hash(run / path, digest)
    expected_steps = profile["num_envs"] * profile["steps_per_env"]
    count = 0
    with (run / "metrics.jsonl").open() as stream:
        for index, line in enumerate(stream):
            row = json.loads(line)
            if (
                row["iteration"] != index
                or row["total_timesteps"] != (index + 1) * expected_steps
            ):
                raise ValueError("source metric sequence mismatch")
            count += 1
    if (
        count != profile["iterations"]
        or matrix["source_checkpoint_iteration"] != count - 1
        or matrix["source_checkpoint_name"] != f"model_{count - 1}.pt"
    ):
        raise ValueError("source is not the declared final checkpoint")
    require_hash(manifest_path, manifest_hash)
    return {
        "source_manifest_sha256": manifest_hash,
        "source_generation_sha256": generation_hash,
        "source_checkpoint_sha256": artifacts[matrix["source_checkpoint_name"]],
        "source_descriptor_path": str(source_path),
        "source_descriptor_sha256": matrix["source_descriptor_sha256"],
        "target_descriptor_sha256": matrix["target_descriptor_sha256"],
        "profile": profile,
        "target_descriptor": target,
    }


def corrected_episode(raw, trace, descriptor, evaluation, cfg):
    if len(trace["frames"]) != 1 or len(trace["cases"]) != 1:
        raise ValueError("native replay requires exactly one episode")
    frames, case = trace["frames"][0], trace["cases"][0]
    report, clearance, impulses = analyze(trace, descriptor, evaluation)
    support = lifted_support_report(frames, descriptor)
    if (
        raw["ticks"] != len(frames)
        or case["ticks"] != len(frames)
        or raw["terminal_reason"] != case["terminal"]
    ):
        raise ValueError("native terminal/count mismatch")
    # Independent closed-form oracle includes each APPLIED action, not next obs.
    commands = []
    for tick in range(len(frames)):
        speed = (
            0
            if tick < 120
            else min(500_000, (tick - 119) * 16_666)
            if tick < 990
            else max(0, 500_000 - (tick - 989) * 16_666)
        )
        commands.append([0, speed, 0])
    np.testing.assert_array_equal([f["command_raw"] for f in frames], commands)
    position, velocity, command = (
        evaluation[key] for key in ("root_position_m", "root_velocity_mps", "command")
    )
    if not all(np.isfinite(a).all() for a in (position, velocity, command)):
        raise ValueError("non-finite evaluation trajectory")
    full = (
        len(frames) == cfg["max_steps"]
        and case["terminal"] == "terminal.timeout"
        and case["safety_error"] is None
        and all(f["completed_physics_substeps"] == 4 for f in frames)
    )
    forward = float(position[-1, 2] - position[0, 2])
    error = float(np.mean(np.abs(velocity[:, 2] - command[:, 1])))
    stop = float(
        np.mean(np.linalg.norm(velocity[-cfg["stop_ticks"] :, [0, 2]], axis=1))
    )
    gates = {
        "complete_safe_episode": full,
        "forward_travel": forward >= cfg["minimum_forward_displacement_m"],
        "velocity_tracking": error <= cfg["maximum_forward_velocity_mae_mps"],
        "final_zero_command_ticks": full
        and bool(np.all(command[-cfg["stop_ticks"] :] == 0)),
        "stopped": full and stop <= cfg["maximum_stop_speed_mae_mps"],
    }
    gait = support["load_and_lift"]
    gates["bilateral_single_support"] = (
        min(gait["longest_continuous_ticks_by_stance_side"])
        >= cfg["minimum_single_support_run_ticks"]
    )
    gates["alternating_support"] = (
        gait["switches_between_runs_of_at_least_eight_ticks"]
        >= cfg["minimum_support_switches"]
    )
    corrected = {
        **raw,
        "gates": gates,
        "passed": all(gates.values()),
        "forward_displacement_m": forward,
        "velocity_mae_mps": error,
        "stop_speed_mae_mps": stop,
        "longest_single_support_ticks": gait["longest_continuous_ticks_by_stance_side"],
        "support_switches": gait["switches_between_runs_of_at_least_eight_ticks"],
    }
    report["lifted_support"] = support
    return corrected, report, clearance, impulses


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("mode", choices=("check-inputs", "evaluate"))
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--generation", type=Path, required=True)
    parser.add_argument("--target-descriptor", type=Path, required=True)
    parser.add_argument("--headless", type=Path)
    parser.add_argument("--auditor", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    run = require_external_path(args.run, ROOT, label="source run")
    generation = require_external_path(args.generation, ROOT, label="source generation")
    target_path = require_external_path(
        args.target_descriptor, ROOT, label="target descriptor"
    )
    matrix = read_json(PROFILE)
    closed = validate_closed_source(run, generation, target_path, matrix)
    if args.mode == "check-inputs":
        print(
            json.dumps(
                {
                    k: v
                    for k, v in closed.items()
                    if k not in ("profile", "target_descriptor")
                },
                indent=2,
            )
        )
        return
    if args.headless is None or args.auditor is None or args.output is None:
        raise ValueError("evaluation requires headless, auditor and fresh output")
    headless = args.headless.resolve(strict=True)
    auditor = args.auditor.resolve(strict=True)
    output = require_external_path(
        args.output, ROOT, label="evaluation", must_exist=False
    )
    if output.exists() or output.is_relative_to(run):
        raise ValueError("evaluation output must be fresh and outside source run")
    if subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True
    ).strip():
        raise ValueError("evaluation requires a clean recorded commit")
    profile = closed["profile"]
    versions = {key: importlib.metadata.version(key) for key in profile["dependencies"]}
    if versions != profile["dependencies"]:
        raise ValueError("evaluation dependency mismatch")
    torch.set_num_threads(1)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    device = profile["device"]
    obs = TensorDict({"policy": torch.zeros((1, 88), device=device)}, [1])
    policy = ActorCritic(
        obs, {"policy": ["policy"], "critic": ["policy"]}, 23, **profile["policy"]
    ).to(device)
    checkpoint = torch.load(
        run / matrix["source_checkpoint_name"], map_location=device, weights_only=True
    )
    if (
        type(checkpoint["iter"]) is not int
        or checkpoint["iter"] != matrix["source_checkpoint_iteration"]
    ):
        raise ValueError("checkpoint iteration mismatch")
    policy.load_state_dict(checkpoint["model_state_dict"], strict=True)
    finite_policy(policy)
    del checkpoint
    target = closed["target_descriptor"]
    inference_profile = {
        **profile,
        "environment_profile_id": matrix["target_environment_profile_id"],
    }
    manifest = {
        "schema": "nextengine.corrected-walking-evaluation-run.v1",
        "status": "running",
        "started_at_utc": datetime.now(timezone.utc).isoformat(),
        "repository_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "matrix": matrix,
        "matrix_sha256": sha256_file(PROFILE),
        "matrix_path": str(PROFILE),
        "source_run": str(run),
        "source_generation_path": str(generation),
        "target_descriptor_path": str(target_path),
        "inputs": {
            k: v for k, v in closed.items() if k not in ("profile", "target_descriptor")
        },
        "executables": {str(p): sha256_file(p) for p in (headless, auditor)},
        "evaluator_sha256": sha256_file(Path(__file__)),
        "dependencies": versions,
        "claim": "bounded forward/start-stop only; no robustness, mirror or runtime promotion",
    }
    output.mkdir(parents=True, exist_ok=False)
    atomic_write_json(output / "run-manifest.json", manifest)
    try:
        raw = evaluate(policy, headless, target, inference_profile, output)
        finite_policy(policy)
        atomic_write_json(output / "raw-presence-evaluation.json", raw)
        records = []
        for episode in raw["episodes"]:
            seed = episode["seed"]
            npz = output / f"evaluation-{seed}.npz"
            with np.load(npz, allow_pickle=False) as data:
                evaluation = {k: data[k].copy() for k in data.files}
            root = shard_root(seed_root(seed), 0)
            tape_path = output / f"actions-{seed}.json"
            trace_path = output / f"native-{seed}.json"
            atomic_write_json(
                tape_path,
                {
                    "profile_id": matrix["target_environment_profile_id"],
                    "run_root_bytes": list(bytes.fromhex(root)),
                    "retain_frames": True,
                    "action_q1_30": [
                        normalized_action_to_raw(
                            evaluation["action"], 23, q1_30=True
                        ).tolist()
                    ],
                    "source_evaluation_sha256": sha256_file(npz),
                    "source_checkpoint_sha256": closed["source_checkpoint_sha256"],
                },
            )
            with (output / f"native-{seed}.log").open("x") as log:
                subprocess.run(
                    [str(auditor), str(tape_path), str(trace_path)],
                    check=True,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    timeout=600,
                )
            trace = read_json(trace_path)
            if (
                trace["action_tape_sha256"] != sha256_file(tape_path)
                or trace["profile_id"] != matrix["target_environment_profile_id"]
                or trace["run_root"] != root
                or trace["executable_sha256"] != manifest["executables"][str(auditor)]
            ):
                raise ValueError("native replay lineage mismatch")
            result, report, clearance, impulses = corrected_episode(
                episode, trace, target, evaluation, matrix["evaluation"]
            )
            audit_path = output / f"support-{seed}"
            audit_path.mkdir()
            atomic_write_json(audit_path / "report.json", report)
            plot(target, evaluation, trace, clearance, impulses, audit_path)
            records.append(result)
            print(json.dumps(result, sort_keys=True), flush=True)
        if [r["seed"] for r in records] != matrix["evaluation"]["seeds"]:
            raise ValueError("incomplete corrected evaluation matrix")
        for path, digest in manifest["executables"].items():
            require_hash(Path(path), digest)
        require_hash(run / "run-manifest.json", closed["source_manifest_sha256"])
        require_hash(
            run / matrix["source_checkpoint_name"], closed["source_checkpoint_sha256"]
        )
        result = {
            "status": "passed" if all(r["passed"] for r in records) else "failed",
            "profile_id": matrix["profile_id"],
            "claim": manifest["claim"],
            "episodes": records,
        }
        atomic_write_json(output / "evaluation.json", result)
        manifest["status"] = "completed"
        manifest["evaluation_status"] = result["status"]
    except BaseException as error:
        manifest["status"] = "failed"
        manifest["error"] = f"{type(error).__name__}: {error}"
        raise
    finally:
        manifest["artifacts"] = artifact_hashes(output)
        manifest["completed_at_utc"] = datetime.now(timezone.utc).isoformat()
        atomic_write_json(output / "run-manifest.json", manifest)


if __name__ == "__main__":
    main()
