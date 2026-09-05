# ADR-112: Prospective validated walking training

| Field | Value |
| --- | --- |
| ID | ADR-112 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-05 |
| Dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-109](109-observable-sole-lift-and-return.md), [ADR-111](111-final-weight-corrected-walking-evaluation.md) |
| Supersedes | ADR-109/110/111's exclusion of a successor optimizer and checkpoint selection only for the new run below; all old experiments and results remain frozen |
| Superseded by | [ADR-113](113-explicit-known-walking-candidate-reuse.md), blanket exclusion of separate known-candidate reuse only; this fresh run remains frozen |

## Evidence and decision

The [closed regression investigation](../../development/r8b-final-policy-regression-research-2026-09-05.md)
finds that V7's final model fails even though a predeclared earlier diagnostic
walked. Exact-buffer controls show that within-update adaptive learning rate
increases a local policy jump; fixed 1e-5 reduces peak minibatch mean KL from
0.02339 to 0.00681. This is not proof of the complete historical cause or an
optimal fresh-training rate. Periodic separate evaluation and retaining a
validated checkpoint are standard practical safeguards, not a statistical claim.

Admit one fresh canonical V8/CUDA PPO run under
[canonical-rsl-rl-walking.v4.json](../../../lab/profiles/canonical-rsl-rl-walking.v4.json).
Keep the exact V8 descriptor/body/actions/reward/controller/safety, network,
seed 44, 128 slots/eight shards, 32 steps/update, 10,000-update ceiling
(40,960,000 transitions), and 14,400-second wall ceiling. Change the learning
rate policy to fixed 1e-5. There is no initialization, resume, sweep, corpus,
Isaac training, physics override or retrospective selection of old weights.
Old V7 training and ADR-111 final-only evaluation remain failed evidence.

## Prospective selection and unchanged task

Before launch freeze validation after every 100 updates: zero-based indices
99, 199, ..., 9999. Save each checkpoint before evaluation. Evaluate its exact
weights and normalization with deterministic mean actions on fresh canonical
scenes for all five seeds 1001..1005. Restore training mode and Python/NumPy/
Torch CPU/CUDA RNG afterward; evaluation must not change model/normalizer state.
No validation trajectory enters PPO storage or the optimizer.

Use every ADR-111 physical gate unchanged: 1,200 safe ticks, >=3 m forward,
velocity MAE <=0.2 m/s, exactly 180 final applied zero commands, stop speed MAE
<=0.1 m/s, >=8 consecutive single-support ticks per side and >=2 switches.
Replay every evaluated action tape in the frozen native auditor; require exact
trajectory and lineage, all-four-substep exclusive loaded sole support and
positive canonical integer post-step whole-box height of the free foot.
Retain the raw presence result separately. No tolerance or safety relaxation.

Select the **first** checkpoint passing the entire five-episode matrix and
stop optimization immediately. Record its hash, iteration and full evaluation
closure; never substitute the last state or select by reward alone. If none
passes, retain the failed final matrix and declare the bounded run failed
quality. Protocol, numerical, missing-artifact, wall or evaluator failures
close execution as failed, not a skipped validation or implicit retry.

This is an explicitly selection-based nominal lesson, not an unbiased
held-out estimate. The five nominal resets are identical physical starts, not
robustness diversity. Passing demonstrates only bounded forward/start-stop
walking. Natural style, arbitrary commands, terrain, recovery, standing
retention on another body, mirror correspondence and runtime/export remain
separate; no current gameplay route is activated.

## Verification, cost and rollback

Require focused checkpoint/policy mismatch, incomplete-matrix, native-lineage,
mode/RNG/state preservation and selection/early-stop tests, existing exact
support/geometry and timeout tests, and the full native adapter control with
resets before the clean-commit generation freeze. Enable the already controlled
passive KL/clip/gradient telemetry only in this new profile. Bind headless and
auditor hashes in the profile and generation closure.

No native/public contract changes are made; ADR-110's native checks and Linux
host-check remain evidence for the unchanged plant. Use focused Python tests
and static checks here. Each validation is O(5 * 1,200) bounded native steps
plus an equal replay; at most 100 validations retain their artifacts externally.
Rollback retires the new run/profile, preserving every checkpoint and prior
failed result. A failed run requires diagnosis rather than an unchanged retry.
