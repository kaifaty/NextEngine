# ADR-107: Direct canonical CPU walking learner

| Field | Value |
|---|---|
| ID | ADR-107 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Normative dependencies | [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-106](106-walking-reference-and-leg-clearance-audit.md) |
| Supersedes | ADR-106's no-optimizer sequencing only for one direct canonical CPU V5 experiment; not its Isaac recipes or promotion gates |
| Superseded by | [ADR-108](108-observable-periodic-walking-credit.md) for its one diagnosed V6 successor only; frozen V5 remains unchanged |

## Context and decision

The user requests a learned walking policy, including necessary repairs.
First-substep probes find equal initial efforts and solver mass properties,
but physical divergence persists without ground contact and with explicitly
matched damping, contact offsets and position iterations. Further mirror
tuning is not a demonstrated prerequisite for learning directly on canonical
physics. See [the evidence](../../development/r8b-canonical-walking-learner-2026-09-04.md).

Admit one fresh-weight canonical CPU PhysX experiment using the unchanged
walking V5 environment and its existing eleven reward components. CUDA
evaluates and optimizes the neural network; it does not simulate physics.
No USD, Isaac simulation, reference corpus or inherited checkpoint is an input.
The prior paired alternation gate remains failed, but no longer blocks this
one direct-CPU discriminator. This does not repair or waive MODEL-MIRROR.

The frozen recipe is
`lab/profiles/canonical-rsl-rl-walking.v1.json`: 128 slots in eight independent
16-slot motor-lab processes, 32 steps/update, 250 updates, 1,024,000 samples,
seed 44, maximum optimizer wall budget 3,600 seconds, no resume or sweeps.
The policy architecture, action noise and PPO hyperparameters retain the
Proposed V5 recipe. This isolates direct canonical learning after the body and
action repairs; duration-aware gait reward remains a separate future identity.
Budget exhaustion fails the run and retains its closed prefix.

## Adapter and run closure

Motor-lab owns raw observations, rewards, commands, targets, safety, episode
ordinals and terminations. Python validates every descriptor identity before
stepping, converts observations by descriptor-owned scales and actions to
clamped Q1.30, and resets only ended slots. Shard root derivation and slot
partition are frozen inputs. Timeout bootstrapping uses the final pre-reset
observation and excludes true terminations. Invalid action batches fail before
any shard advances; a worker failure stops the whole run, never retries a step.

Before optimization require focused adapter/timeout tests and a native control
with all raw StepResult fields byte-exact, including independent slot resets.
The `freeze` entry point writes an external generation manifest and index
closing its file SHA-256. It binds clean Git commit, executable/descriptor/profile
file hashes, dependency versions, fixed evaluation matrix and one new run path.
Training rejects changed closure or reused output. This CPU-specific generation
does not fabricate reference-corpus/USD admission. The run manifest closes all
produced checkpoints, metrics and evaluation artifacts on success or failure.
Checkpoints are evaluation artifacts, not complete continuation snapshots.

## Quality and rollback

Evaluate only the predeclared final checkpoint on five seeds 1001–1005, each
up to the exact 1,200-tick bound. Every episode must survive without safety
termination, travel at least 3 m forward, and finish the exact 180 zero-command
ticks. Additional measured gates require velocity MAE <=0.2 m/s, final planar
speed MAE <=0.1 m/s, continuous single support >=8 ticks on each side and at
least two switches between qualified left/right support runs. This rejects
standing/sliding as walking. Reset seeds alone do not establish perturbation
robustness. No checkpoint selection by held-out score.

A completed optimizer budget is not learned quality. Failed evaluation must
produce a diagnosis before another profile/run, not an unchanged retry.
Mirror correspondence, runtime/export admission and broader locomotion remain
unchanged and unpassed. Rollback retires this learner/profile; no canonical
physics/reward setting or old standing artifact changes. Required verification
is focused Python/native adapter tests; existing body/motor checks remain
applicable, broad host-check reports any independent existing blocker separately.
