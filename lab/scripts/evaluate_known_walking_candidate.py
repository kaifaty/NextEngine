"""Explicit exploratory reuse of one known checkpoint; old final result stays failed."""

import argparse
import importlib.metadata
import json
import subprocess
from pathlib import Path

import torch
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from rsl_rl.modules import ActorCritic
from tensordict import TensorDict

from lab.scripts.canonical_walking_ppo import ROOT, artifact_hashes, finite_policy
from lab.scripts.evaluate_corrected_walking import (
    PROFILE as FINAL_PROFILE,
)
from lab.scripts.evaluate_corrected_walking import (
    read_json,
    require_hash,
    validate_closed_source,
)
from lab.scripts.validate_walking_checkpoint import validate_checkpoint

PROFILE = ROOT / "lab/profiles/known-walking-candidate.v1.json"


def candidate_checkpoint(run, source_manifest, cfg):
    if (
        cfg["source_checkpoint_iteration"] != 3999
        or cfg["source_checkpoint_name"] != "model_3999.pt"
    ):
        raise ValueError("only the explicitly identified known candidate is admitted")
    name = cfg["source_checkpoint_name"]
    if source_manifest["artifacts"].get(name) != cfg["source_checkpoint_sha256"]:
        raise ValueError("candidate is not closed by source manifest")
    path = run / name
    require_hash(path, cfg["source_checkpoint_sha256"])
    return path


def main():
    parser = argparse.ArgumentParser(__doc__)
    for name in (
        "run",
        "generation",
        "target-descriptor",
        "headless",
        "auditor",
        "output",
    ):
        parser.add_argument("--" + name, type=Path, required=True)
    args = parser.parse_args()
    run = require_external_path(args.run, ROOT, label="source run")
    generation = require_external_path(args.generation, ROOT, label="source generation")
    target = require_external_path(args.target_descriptor, ROOT, label="descriptor")
    output = require_external_path(
        args.output, ROOT, label="candidate evaluation", must_exist=False
    )
    if output.exists() or output.is_relative_to(run):
        raise ValueError("output must be new and outside source run")
    if subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True
    ).strip():
        raise ValueError("candidate evaluation requires a clean recorded commit")
    cfg = read_json(PROFILE)
    # Preserve the old final-only validator intact and verify the entire closed run.
    closed = validate_closed_source(run, generation, target, read_json(FINAL_PROFILE))
    for key in (
        "source_generation_sha256",
        "source_manifest_sha256",
        "source_descriptor_sha256",
        "target_descriptor_sha256",
    ):
        if closed[key] != cfg[key]:
            raise ValueError(f"candidate source mismatch: {key}")
    source_manifest = read_json(run / "run-manifest.json")
    if source_manifest["profile_sha256"] != cfg["source_training_profile_sha256"]:
        raise ValueError("candidate source training profile mismatch")
    checkpoint_path = candidate_checkpoint(run, source_manifest, cfg)
    headless, auditor = (
        args.headless.resolve(strict=True),
        args.auditor.resolve(strict=True),
    )
    require_hash(headless, cfg["headless_sha256"])
    require_hash(auditor, cfg["auditor_sha256"])
    profile = {
        **closed["profile"],
        "environment_profile_id": cfg["target_environment_profile_id"],
    }
    if profile["evaluation"] != cfg["evaluation"]:
        raise ValueError("candidate physical gate mismatch")
    versions = {k: importlib.metadata.version(k) for k in profile["dependencies"]}
    if versions != profile["dependencies"]:
        raise ValueError("candidate dependency mismatch")
    torch.set_num_threads(1)
    torch.backends.cuda.matmul.allow_tf32 = False
    torch.backends.cudnn.allow_tf32 = False
    device = profile["device"]
    obs = TensorDict({"policy": torch.zeros(1, 88, device=device)}, [1])
    policy = ActorCritic(
        obs, {"policy": ["policy"], "critic": ["policy"]}, 23, **profile["policy"]
    ).to(device)
    checkpoint = torch.load(checkpoint_path, map_location=device, weights_only=True)
    if (
        type(checkpoint["iter"]) is not int
        or checkpoint["iter"] != cfg["source_checkpoint_iteration"]
    ):
        raise ValueError("candidate checkpoint iteration mismatch")
    policy.load_state_dict(checkpoint["model_state_dict"], strict=True)
    finite_policy(policy)
    del checkpoint
    manifest = {
        "schema": "nextengine.known-walking-candidate-run.v1",
        "status": "running",
        "profile": cfg,
        "profile_sha256": sha256_file(PROFILE),
        "repository_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "script_sha256": sha256_file(Path(__file__)),
        "source_run": str(run),
        "source_generation": str(generation),
        "target_descriptor": str(target),
        "optimizer_steps": 0,
        "claim": "explicitly selected known candidate; nominal walking only, not held-out statistics, old-run success or runtime promotion",
    }
    output.mkdir(parents=True, exist_ok=False)
    atomic_write_json(output / "run-manifest.json", manifest)
    try:
        result = validate_checkpoint(
            policy,
            headless,
            auditor,
            closed["target_descriptor"],
            profile,
            output / "evaluation",
            checkpoint_path,
        )
        require_hash(run / "run-manifest.json", cfg["source_manifest_sha256"])
        require_hash(checkpoint_path, cfg["source_checkpoint_sha256"])
        result["claim"] = manifest["claim"]
        atomic_write_json(output / "evaluation.json", result)
        manifest.update(status="completed", evaluation_status=result["status"])
        print(json.dumps(result, sort_keys=True), flush=True)
    except BaseException as error:
        manifest.update(status="failed", error=f"{type(error).__name__}: {error}")
        raise
    finally:
        manifest["artifacts"] = artifact_hashes(output)
        atomic_write_json(output / "run-manifest.json", manifest)


if __name__ == "__main__":
    main()
