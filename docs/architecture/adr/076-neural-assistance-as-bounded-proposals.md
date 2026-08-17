# ADR-076: Neural assistance as bounded proposals

| Field | Value |
|---|---|
| ID | ADR-076 |
| Status | Proposed |
| Version | 1.0 |
| Decision date | 2026-08-17 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-36](../36-continuum-material-physics.md), [SPEC-37](../37-layered-physical-world.md), [SPEC-39](../39-world-substrate-composition.md), [SPEC-41](../41-thermochemical-material-processes.md), [SPEC-42](../42-neural-assisted-world-simulation.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md) |
| Supersedes | none |
| Superseded by | none |

## Context

The imported neural-assisted physics paper proposes learned warm starts,
closures and surrogates guarded by classical correction and fallback. That is
useful research direction, but a generic "fall back to the classical solver"
after a failed learned solve would be a hidden retry. GPU classical authority,
tolerance determinism and mid-run solver switching also conflict with current
CPU canonical authority and exact-root rules.

Next Engine already has immutable model/dataset lineage under SPEC-34/ADR-053
and domain-specific deterministic fallbacks for learned behavior. World
physics needs a stricter rule because an accelerator can perturb the solver
trajectory, coupling receipts and persistence roots.

## Proposed decision

### A model is not a world owner or backend

Adopt SPEC-42. A learned component may produce bounded, revision-bound advice
for one already promoted classical owner. It owns no authoritative state and
cannot emit commands, events, contacts, phase, topology or checkpoints.

N0 diagnostics and report-only N1 proposals are the only initial trust tiers.
Learned corrections, constitutive surrogates, region evolution and end-to-end
world simulation require new consumer-backed decisions and receive no implied
permission from this ADR.

### Classical reference, production and persistence come first

No dataset or model work closes a missing numeric profile, reference oracle,
production solver, exact persistence or performance baseline. The first study
starts only after a target owner exposes a measured iterative bottleneck.

SPEC-36 base water V1 has no warm start. A learned DFSPH warm start, if later
selected, is a separate post-promotion branch and does not alter W1-W6 or GPU
authority. The same independence applies to vegetation and thermochemical
roadmaps.

### Proposal before solve; no retry after admission

The model output is canonicalized and validated before the classical solver
starts. Missing, invalid, incompatible or out-of-range output selects the
deterministic default initialization inside the same owner/backend.

After an admitted proposal enters the solve, non-convergence or invariant
failure rejects the step. Runtime cannot rerun from the default, reduce the
timestep, race several models or switch CPU/GPU/backend. This preserves failure
semantics instead of turning proposal acceptance into retry-to-green.

### Exact roots, not approximate similarity

A runtime proposal profile must preserve exact canonical owner roots,
outcome/event order and failure classification against the default classical
path on the frozen corpus, targets and continuation windows. Tolerance metrics
remain report-only. If exact closure cannot be shown, the model stays N0 or
shadow-only regardless of speed.

Training artifacts follow SPEC-34/ADR-053: hash-closed data, immutable bundle,
fixed splits, external heavy storage, no online training or hidden recurrent
state. Current `ModelLaneV1` is not expanded without a real consumer.

### Correctness fallback is the unchanged classical path

An absent accelerator uses the target owner's normal deterministic default
initialization. This does not authorize a different material law, cheaper
solver, frozen state or alternative backend. The optional bundle can affect
cost only; it cannot be required for correct gameplay or restore.

## Failure and stop conditions

Malformed or unavailable output is proposal absence before solve. An admitted
proposal followed by solver failure is an owner-step failure with no automatic
retry. Model lineage mismatch rejects the accelerator before use and leaves the
classical owner available.

After two evidence-backed optimization cycles without a predeclared end-to-end
p95/p99 improvement and exact non-regression, the branch stops. We do not lower
classical gates, accept approximate roots, enlarge the budget or grant
model/GPU authority without a new decision.

## Alternatives rejected

- Learned solver as a peer owner: creates a second writer and ambiguous
  persistence.
- GPU classical baseline as automatic authority: conflicts with the selected
  CPU canonical paths and correspondence-only mirrors.
- Retry the classical default after learned non-convergence: changes failure
  semantics and can hide deterministic faults.
- Tolerance-based authoritative equivalence: insufficient for command/event/
  checkpoint roots.
- Online adaptation: creates hidden future-affecting state and unreproducible
  lineage.

## Consequences

- SPEC-42 and this ADR remain `Proposed`; no model lane, schema, provider or
  runtime dependency is added.
- Neural work is optional and downstream of the classical owner, never a gate
  for its promotion.
- `WORLD-DYNAMICS-P1` counts promoted state owners, not models; neural
  assistance cannot satisfy its owner threshold.
- The next action is selecting one post-promotion bottleneck and freezing N0
  instrumentation, not training a model.
