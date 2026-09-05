#!/usr/bin/env python3
"""One disposable fixed-buffer PPO update versus zero-LR; never saves a policy."""

from __future__ import annotations

import argparse
import copy
import importlib.metadata
import json
import subprocess
import time
from collections import Counter
from pathlib import Path

import numpy as np
import torch
from next_lab.canonical_ppo import TerminalObservationPPO
from next_lab.isaac_training import (
    atomic_write_json,
    require_external_path,
    sha256_file,
)
from next_lab.ppo_diagnostics import (
    distribution_metrics,
    observe_ppo_update,
    summarize_update,
)
from rsl_rl.modules import ActorCritic
from tensordict import TensorDict
from torch.distributions import Normal

from lab.scripts.canonical_walking_ppo import artifact_hashes, finite_policy, make_env
from lab.scripts.evaluate_corrected_walking import PROFILE as MATRIX
from lab.scripts.evaluate_corrected_walking import (
    ROOT,
    read_json,
    require_hash,
    validate_closed_source,
)

PROFILE = ROOT / "lab/profiles/canonical-ppo-gradient-diagnostic.v1.json"


def check_returns(storage, final_value, gamma, lam):
    """Independent reverse GAE recurrence over retained corrected rewards."""
    advantage = torch.zeros_like(final_value)
    expected = torch.empty_like(storage.returns)
    for tick in reversed(range(storage.num_transitions_per_env)):
        following = (
            final_value
            if tick == storage.num_transitions_per_env - 1
            else storage.values[tick + 1]
        )
        live = 1.0 - storage.dones[tick].float()
        delta = storage.rewards[tick] + live * gamma * following - storage.values[tick]
        advantage = delta + live * gamma * lam * advantage
        expected[tick] = advantage + storage.values[tick]
    torch.testing.assert_close(storage.returns, expected, rtol=1e-6, atol=1e-5)
    return float((storage.returns - expected).abs().max())


@torch.no_grad()
def buffer_metrics(policy, storage):
    obs = TensorDict(
        {"policy": storage.observations["policy"].flatten(0, 1)},
        [storage.num_envs * storage.num_transitions_per_env],
    )
    mean = policy.act_inference(obs)
    std = policy.std.expand_as(mean)
    actions = storage.actions.flatten(0, 1)
    return distribution_metrics(
        mean,
        std,
        storage.mu.flatten(0, 1),
        storage.sigma.flatten(0, 1),
        Normal(mean, std).log_prob(actions).sum(-1),
        storage.actions_log_prob.flatten(),
        0.2,
    )


def run_arm(algorithm, name, seed, expected_steps):
    """Both arms start from identical data/model/Adam state and consume equal RNG."""
    if name not in ("zero-effective-lr", "source-adaptive-lr"):
        raise ValueError("unknown diagnostic arm")
    before = {
        key: value.clone() for key, value in algorithm.policy.state_dict().items()
    }
    pre = buffer_metrics(algorithm.policy, algorithm.storage)
    handle = None
    if name == "zero-effective-lr":

        def zero_step(optimizer, args, kwargs):
            for group in optimizer.param_groups:
                group["lr"] = 0.0

        handle = algorithm.optimizer.register_step_pre_hook(zero_step)
    try:
        torch.manual_seed(seed)
        with observe_ppo_update(algorithm) as records:
            losses = algorithm.update()
    finally:
        if handle is not None:
            handle.remove()
    if len(records) != expected_steps:
        raise ValueError("unexpected diagnostic optimizer step count")
    finite_policy(algorithm.policy)
    after = algorithm.policy.state_dict()
    if name == "zero-effective-lr" and any(
        not torch.equal(before[key], after[key]) for key in before
    ):
        raise ValueError("zero-LR control changed policy state")
    if any(
        not torch.equal(before[key], after[key])
        for key in before
        if "normalizer" in key
    ):
        raise ValueError("normalization changed during gradient update")
    # Storage.clear resets the cursor only; retained tensors still contain this batch.
    return {
        "arm": name,
        "pre": pre,
        "post": buffer_metrics(algorithm.policy, algorithm.storage),
        "losses": losses,
        "minibatches": records,
        "summary": summarize_update(records),
        "policy_max_parameter_delta": max(
            float((before[key] - after[key]).abs().max()) for key in before
        ),
        "normalizer_state_unchanged": True,
    }


def main():
    parser = argparse.ArgumentParser(__doc__)
    parser.add_argument("--run", type=Path, required=True)
    parser.add_argument("--generation", type=Path, required=True)
    parser.add_argument("--target-descriptor", type=Path, required=True)
    parser.add_argument("--headless", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    cfg, matrix = read_json(PROFILE), read_json(MATRIX)
    run = require_external_path(args.run, ROOT, label="closed source")
    out = require_external_path(
        args.output, ROOT, label="diagnostic output", must_exist=False
    )
    if out.exists() or out.is_relative_to(run):
        raise ValueError("diagnostic output must be fresh and outside source")
    if subprocess.check_output(
        ["git", "status", "--porcelain"], cwd=ROOT, text=True
    ).strip():
        raise ValueError("requires a clean recorded commit")
    closed = validate_closed_source(
        run, args.generation, args.target_descriptor, matrix
    )
    for key in ("source_checkpoint_sha256", "source_manifest_sha256"):
        if closed[key] != cfg[key]:
            raise ValueError(f"diagnostic source mismatch: {key}")
    require_hash(args.headless, cfg["headless_sha256"])
    profile = copy.deepcopy(closed["profile"])
    versions = {key: importlib.metadata.version(key) for key in profile["dependencies"]}
    if versions != profile["dependencies"]:
        raise ValueError("diagnostic dependency mismatch")
    if (profile["num_envs"], profile["shards"], profile["steps_per_env"]) != (
        cfg["num_envs"],
        cfg["shards"],
        cfg["buffer_steps"],
    ):
        raise ValueError("diagnostic batch differs from frozen training profile")
    profile["environment_profile_id"] = matrix["target_environment_profile_id"]
    out.mkdir(parents=True, exist_ok=False)
    manifest = {
        "schema": "nextengine.ppo-gradient-diagnostic-run.v1",
        "status": "running",
        "profile": cfg,
        "profile_sha256": sha256_file(PROFILE),
        "repository_commit": subprocess.check_output(
            ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True
        ).strip(),
        "script_sha256": sha256_file(Path(__file__)),
        "dependencies": versions,
        "source_run": str(run),
        "target_descriptor_sha256": matrix["target_descriptor_sha256"],
        "claim": "disposable numerical diagnostic only; no candidate, continuation or learned evaluation",
    }
    atomic_write_json(out / "run-manifest.json", manifest)
    start = time.monotonic()
    env = None
    try:
        torch.set_num_threads(1)
        torch.backends.cuda.matmul.allow_tf32 = False
        torch.backends.cudnn.allow_tf32 = False
        device = profile["device"]
        obs = TensorDict(
            {"policy": torch.zeros((cfg["num_envs"], 88), device=device)},
            [cfg["num_envs"]],
        )
        policy = ActorCritic(
            obs, {"policy": ["policy"], "critic": ["policy"]}, 23, **profile["policy"]
        ).to(device)
        checkpoint = torch.load(
            run / matrix["source_checkpoint_name"],
            map_location=device,
            weights_only=True,
        )
        if checkpoint["iter"] != 9999:
            raise ValueError("not final checkpoint")
        policy.load_state_dict(checkpoint["model_state_dict"], strict=True)
        algorithm = TerminalObservationPPO(
            policy, device=device, **profile["algorithm"]
        )
        algorithm.optimizer.load_state_dict(checkpoint["optimizer_state_dict"])
        rates = {group["lr"] for group in algorithm.optimizer.param_groups}
        if len(rates) != 1:
            raise ValueError("source optimizer has unequal rates")
        algorithm.learning_rate = rates.pop()
        source_parameters = {
            key: value.clone() for key, value in policy.named_parameters()
        }
        policy.train()
        torch.manual_seed(cfg["action_noise_seed_after_loading"])
        env = make_env(
            args.headless,
            closed["target_descriptor"],
            profile,
            seed=cfg["environment_seed"],
        )
        warm_actions, warm_reasons = [], Counter()
        with torch.inference_mode():
            for tick in range(cfg["roll_in_steps"]):
                if time.monotonic() - start > cfg["wall_seconds"]:
                    raise TimeoutError("diagnostic wall ceiling")
                action = policy.act(env.get_observations())
                obs, _, _, _ = env.step(action)
                policy.update_normalization(obs)
                warm_actions.append(action.cpu().numpy().copy())
                warm_reasons.update(
                    item.terminal_reason_id
                    for item in env.last_steps
                    if item.terminated or item.truncated
                )
                if (tick + 1) % 300 == 0:
                    print(
                        json.dumps(
                            {"roll_in_steps": tick + 1, "terminals": dict(warm_reasons)}
                        ),
                        flush=True,
                    )
        algorithm.init_storage("rl", cfg["num_envs"], cfg["buffer_steps"], obs, [23])
        (
            observations_after,
            final_observations,
            timeout_values,
            rewards,
            timeouts,
            motor_ticks,
        ) = [], [], [], [], [], []
        with torch.inference_mode():
            for _ in range(cfg["buffer_steps"]):
                action = algorithm.act(obs)
                obs, reward, done, extras = env.step(action)
                final_value = policy.evaluate(extras["terminal_observation"]).squeeze(
                    -1
                )
                observations_after.append(obs["policy"].clone())
                final_observations.append(
                    extras["terminal_observation"]["policy"].clone()
                )
                timeout_values.append(final_value.clone())
                rewards.append(reward.clone())
                timeouts.append(extras["time_outs"].clone())
                motor_ticks.append([item.motor_tick for item in env.last_steps])
                algorithm.process_env_step(obs, reward, done, extras)
            final_value = policy.evaluate(obs).detach()
            algorithm.compute_returns(obs)
            recurrence_error = check_returns(
                algorithm.storage, final_value, algorithm.gamma, algorithm.lam
            )
        env.close()
        env = None  # Updated copies are never applied to a native environment.
        for key, value in policy.named_parameters():
            if not torch.equal(source_parameters[key], value):
                raise ValueError("roll-in modified trainable parameters")
        tensors = {
            key: getattr(algorithm.storage, key).cpu().numpy()
            for key in (
                "actions",
                "rewards",
                "dones",
                "values",
                "returns",
                "advantages",
                "actions_log_prob",
                "mu",
                "sigma",
            )
        }
        tensors["observations"] = algorithm.storage.observations["policy"].cpu().numpy()
        for key, values in (
            ("observations_after", observations_after),
            ("final_observations", final_observations),
            ("timeout_values", timeout_values),
            ("raw_rewards", rewards),
            ("timeouts", timeouts),
        ):
            tensors[key] = torch.stack(values).cpu().numpy()
        tensors["final_value"] = final_value.cpu().numpy()
        tensors["motor_ticks"] = np.asarray(motor_ticks)
        tensors.update(
            {
                key: value.cpu().numpy()
                for key, value in policy.state_dict().items()
                if "normalizer" in key
            }
        )
        np.savez_compressed(out / "buffer.npz", **tensors)
        np.savez_compressed(out / "roll-in-actions.npz", action=np.stack(warm_actions))
        results = []
        for name in cfg["arms"]:
            arm = copy.deepcopy(algorithm)
            result = run_arm(
                arm, name, cfg["update_rng_seed"], cfg["adam_steps_per_arm"]
            )
            atomic_write_json(out / f"{name}.json", result)
            results.append(
                {
                    "arm": name,
                    "pre": result["pre"],
                    "post": result["post"],
                    "summary": result["summary"],
                }
            )
            print(json.dumps(results[-1]), flush=True)
            del arm
        if time.monotonic() - start > cfg["wall_seconds"]:
            raise TimeoutError("diagnostic wall ceiling")
        require_hash(
            run / matrix["source_checkpoint_name"], cfg["source_checkpoint_sha256"]
        )
        require_hash(run / "run-manifest.json", cfg["source_manifest_sha256"])
        require_hash(args.headless, cfg["headless_sha256"])
        manifest.update(
            status="completed",
            results=results,
            recurrence_max_error=recurrence_error,
            moving_buffer_samples=int(
                np.count_nonzero(tensors["observations"][..., 80])
            ),
            buffer_motor_tick_range=[
                int(np.min(motor_ticks)),
                int(np.max(motor_ticks)),
            ],
            candidate_weights_saved=False,
            elapsed_seconds=time.monotonic() - start,
        )
    except BaseException as error:
        manifest.update(status="failed", error=f"{type(error).__name__}: {error}")
        raise
    finally:
        if env is not None:
            env.close()
        manifest["artifacts"] = artifact_hashes(out)
        atomic_write_json(out / "run-manifest.json", manifest)


if __name__ == "__main__":
    main()
