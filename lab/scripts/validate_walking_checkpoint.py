"""Prospective V8 checkpoint validation; no optimizer or source-run mutation."""

from __future__ import annotations

import json
import random
import subprocess
from pathlib import Path

import numpy as np
import torch
from next_lab.canonical_ppo import shard_root
from next_lab.isaac_training import atomic_write_json, sha256_file
from next_lab.motor_lab_client import normalized_action_to_raw


def validate_checkpoint(
    policy, headless, auditor, descriptor, profile, output, checkpoint
):
    # Local imports keep the old frozen trainer/evaluator entry points independent.
    from lab.scripts.canonical_walking_ppo import artifact_hashes, evaluate, seed_root
    from lab.scripts.evaluate_corrected_walking import corrected_episode

    saved = torch.load(
        checkpoint, map_location=next(policy.parameters()).device, weights_only=True
    )["model_state_dict"]
    if saved.keys() != policy.state_dict().keys() or any(
        not torch.equal(value, policy.state_dict()[key]) for key, value in saved.items()
    ):
        raise ValueError("checkpoint does not match evaluated policy")
    if (
        len(profile["evaluation"]["seeds"]) != 5
        or len(set(profile["evaluation"]["seeds"])) != 5
    ):
        raise ValueError("requires five distinct declared episode seeds")
    del saved
    output.mkdir(parents=True, exist_ok=False)
    weights_hash = sha256_file(checkpoint)
    executable_hashes = {str(p): sha256_file(p) for p in (headless, auditor)}
    state = {k: v.detach().clone() for k, v in policy.state_dict().items()}
    training = policy.training
    rng = (random.getstate(), np.random.get_state(), torch.get_rng_state())
    cuda_rng = torch.cuda.get_rng_state_all() if torch.cuda.is_available() else None
    manifest = {
        "schema": "nextengine.prospective-walking-validation.v1",
        "status": "running",
        "checkpoint_path": str(checkpoint),
        "checkpoint_sha256": weights_hash,
        "executables": executable_hashes,
        "profile": profile,
        "claim": "nominal forward/start-stop validation; no held-out robustness or runtime promotion",
    }
    atomic_write_json(output / "run-manifest.json", manifest)
    try:
        raw = evaluate(policy, headless, descriptor, profile, output)
        atomic_write_json(output / "raw-presence-evaluation.json", raw)
        records = []
        for episode in raw["episodes"]:
            seed = episode["seed"]
            npz = output / f"evaluation-{seed}.npz"
            with np.load(npz, allow_pickle=False) as data:
                evaluation = {key: data[key].copy() for key in data.files}
            root = shard_root(seed_root(seed), 0)
            tape = output / f"actions-{seed}.json"
            trace_path = output / f"native-{seed}.json"
            atomic_write_json(
                tape,
                {
                    "profile_id": profile["environment_profile_id"],
                    "run_root_bytes": list(bytes.fromhex(root)),
                    "retain_frames": True,
                    "action_q1_30": [
                        normalized_action_to_raw(
                            evaluation["action"], 23, q1_30=True
                        ).tolist()
                    ],
                    "source_evaluation_sha256": sha256_file(npz),
                    "source_checkpoint_sha256": weights_hash,
                },
            )
            with (output / f"native-{seed}.log").open("x") as log:
                subprocess.run(
                    [str(auditor), str(tape), str(trace_path)],
                    check=True,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    timeout=600,
                )
            trace = json.loads(trace_path.read_text())
            if (
                trace["action_tape_sha256"] != sha256_file(tape)
                or trace["profile_id"] != profile["environment_profile_id"]
                or trace["run_root"] != root
                or trace["executable_sha256"] != executable_hashes[str(auditor)]
            ):
                raise ValueError("native validation lineage mismatch")
            result, report, _, _ = corrected_episode(
                episode, trace, descriptor, evaluation, profile["evaluation"]
            )
            atomic_write_json(output / f"support-{seed}.json", report)
            records.append(result)
        if [r["seed"] for r in records] != profile["evaluation"]["seeds"]:
            raise ValueError("incomplete prospective evaluation matrix")
        if any(
            not torch.equal(value, policy.state_dict()[key])
            for key, value in state.items()
        ):
            raise RuntimeError("validation mutated policy or normalization")
        if sha256_file(checkpoint) != weights_hash:
            raise ValueError("validation checkpoint changed")
        if any(sha256_file(Path(p)) != h for p, h in executable_hashes.items()):
            raise ValueError("validation executable changed")
        result = {
            "status": "passed" if all(r["passed"] for r in records) else "failed",
            "episodes": records,
            "checkpoint_sha256": weights_hash,
            "claim": manifest["claim"],
        }
        atomic_write_json(output / "evaluation.json", result)
        manifest.update(status="completed", evaluation_status=result["status"])
        return result
    except BaseException as error:
        manifest.update(status="failed", error=f"{type(error).__name__}: {error}")
        raise
    finally:
        policy.train(training)
        random.setstate(rng[0])
        np.random.set_state(rng[1])
        torch.set_rng_state(rng[2])
        if cuda_rng is not None:
            torch.cuda.set_rng_state_all(cuda_rng)
        manifest["artifacts"] = artifact_hashes(output)
        atomic_write_json(output / "run-manifest.json", manifest)
