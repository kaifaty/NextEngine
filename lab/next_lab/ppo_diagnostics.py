"""Passive, scoped observation of the pinned feed-forward RSL-RL PPO update.

No extra actor sampling, backward pass, optimizer step or global monkeypatch.
This is private lab telemetry, not a new optimizer or an experiment admission.
"""

from __future__ import annotations

import math
from contextlib import contextmanager
from importlib.metadata import version

import torch


@torch.no_grad()
def distribution_metrics(mean, std, old_mean, old_std, log_prob, old_log_prob, clip):
    """KL(old Normal || current Normal); ratio clipping, not loss clipping.

    Use float64 only in this detached report. Unlike the scheduler's stabilized
    expression, the analytic KL is zero for exactly identical distributions.
    """
    tensors = (mean, std, old_mean, old_std)
    if (
        mean.ndim != 2
        or mean.numel() == 0
        or any(t.shape != mean.shape for t in tensors)
        or log_prob.shape != (mean.shape[0],)
        or old_log_prob.shape != log_prob.shape
        or not 0 < clip < 1
        or any(not torch.isfinite(t).all() for t in (*tensors, log_prob, old_log_prob))
        or (std <= 0).any()
        or (old_std <= 0).any()
    ):
        raise ValueError("invalid Gaussian diagnostic inputs")
    mean, std, old_mean, old_std = (t.detach().double() for t in tensors)
    kl = (
        torch.log(std / old_std)
        + (old_std.square() + (old_mean - mean).square()) / (2 * std.square())
        - 0.5
    ).sum(-1)
    # Compare log ratios directly to avoid overflow in exp for extreme policies.
    log_ratio = log_prob.detach().double() - old_log_prob.detach().double()
    clipped = (log_ratio < math.log1p(-clip)) | (log_ratio > math.log1p(clip))
    return {
        "analytic_kl_mean": float(kl.mean()),
        "analytic_kl_max": float(kl.max()),
        "ratio_clip_fraction": float(clipped.double().mean()),
        "absolute_log_ratio_max": float(log_ratio.abs().max()),
        "mean_action_saturation_fraction": float((mean.abs() >= 1).double().mean()),
        "action_std_min": float(std.min()),
        "action_std_max": float(std.max()),
    }


@contextmanager
def observe_ppo_update(algorithm):
    """Yield one report per pre-step minibatch; restore all local hooks on exit.

    Supported only for RSL-RL 3.1.2, non-recurrent/no-RND/no-symmetry/single-device
    PPO. Each backward parameter hook must run once per batch: this is not a
    generic gradient-accumulation observer. No callback modifies a gradient.
    """
    policy, storage = algorithm.policy, algorithm.storage
    if (
        version("rsl-rl-lib") != "3.1.2"
        or policy.is_recurrent
        or algorithm.rnd
        or algorithm.symmetry
        or algorithm.is_multi_gpu
        or "mini_batch_generator" in vars(storage)
    ):
        raise ValueError("unsupported or already-observed PPO update")
    original = storage.mini_batch_generator
    records, handles, gradients = [], [], {}
    current = None

    def batches(*args, **kwargs):
        nonlocal current
        for batch in original(*args, **kwargs):
            if len(batch) != 10:
                raise ValueError("pinned PPO minibatch contract changed")
            gradients.clear()
            current = batch
            count = len(records)
            yield batch
            if len(records) != count + 1:
                raise RuntimeError("expected exactly one optimizer step per batch")
            current = None

    def gradient_hook(name):
        def observe(gradient):
            if current is None or name in gradients:
                raise RuntimeError("unexpected gradient outside single backward")
            gradients[name] = gradient.detach().double().square().sum()
            # Returning None preserves the original gradient.

        return observe

    @torch.no_grad()
    def before_step(optimizer, args, kwargs):
        if current is None or not gradients:
            raise RuntimeError("optimizer step has no observed minibatch/backward")
        _, actions, _, _, _, old_log_prob, old_mean, old_std, _, _ = current
        report = distribution_metrics(
            policy.action_mean,
            policy.action_std,
            old_mean,
            old_std,
            policy.get_actions_log_prob(actions),
            old_log_prob.squeeze(-1),
            algorithm.clip_param,
        )
        report["samples"] = actions.shape[0]
        report["learning_rate"] = optimizer.param_groups[0]["lr"]
        for label, prefix in (
            ("gradient_norm", ""),
            ("actor_gradient_norm", "actor."),
            ("critic_gradient_norm", "critic."),
        ):
            terms = [
                value for name, value in gradients.items() if name.startswith(prefix)
            ]
            report[label] = float(torch.stack(terms).sum().sqrt()) if terms else 0.0
        post_clip = [
            p.grad.detach().double().square().sum()
            for p in policy.parameters()
            if p.grad is not None
        ]
        report["post_clip_gradient_norm"] = float(torch.stack(post_clip).sum().sqrt())
        if not all(math.isfinite(value) for value in report.values()):
            raise RuntimeError("non-finite PPO diagnostic")
        records.append(report)

    storage.mini_batch_generator = batches
    try:
        for name, parameter in policy.named_parameters():
            if parameter.requires_grad:
                handles.append(parameter.register_hook(gradient_hook(name)))
        handles.append(algorithm.optimizer.register_step_pre_hook(before_step))
        yield records
    finally:
        for handle in handles:
            handle.remove()
        del storage.mini_batch_generator


def summarize_update(records):
    if not records:
        raise ValueError("no observed PPO minibatches")
    samples = sum(row["samples"] for row in records)
    result = {"minibatches": len(records), "sample_visits": samples}
    for key in records[0]:
        if key != "samples":
            result[f"{key}_average"] = (
                sum(row[key] * row["samples"] for row in records) / samples
            )
            result[f"{key}_maximum"] = max(row[key] for row in records)
    return result
