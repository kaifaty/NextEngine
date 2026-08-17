# ADR-080: Neural assistance as bounded proposals

| Field | Value |
|---|---|
| ID | ADR-080 |
| Status | Proposed |
| Version | 1.1 |
| Decision date | 2026-08-17 |
| Last verified | 2026-08-17 |
| Normative dependencies | [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-38](../38-continuum-material-physics.md), [SPEC-39](../39-layered-physical-world.md), [SPEC-41](../41-world-substrate-composition.md), [SPEC-43](../43-thermochemical-material-processes.md), [SPEC-44](../44-neural-assisted-world-simulation.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md) |
| Supersedes | none |
| Superseded by | Partially [ADR-081](081-world-dynamics-gap-closure-and-promotion-guardrails.md): N1 is report/shadow-only; runtime consumption requires a later certificate-backed Accepted ADR. |

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

Adopt SPEC-44 as narrowed by ADR-081. A learned component may produce bounded,
revision-bound report/shadow proposals for one already promoted classical
owner. Production never consumes them. The model owns no authoritative state
and cannot emit commands, events, contacts, phase, topology or checkpoints.

N0 diagnostics and report-only N1 proposals are the only initial trust tiers.
Learned corrections, constitutive surrogates, region evolution and end-to-end
world simulation require new consumer-backed decisions and receive no implied
permission from this ADR.

### Classical reference, production and persistence come first

No dataset or model work closes a missing numeric profile, reference oracle,
production solver, exact persistence or performance baseline. The first study
starts only after a target owner exposes a measured iterative bottleneck.

SPEC-38 base water V1 has no warm start. A learned DFSPH warm start, if later
selected, is a separate post-promotion branch and does not alter W1-W6 or GPU
authority. The same independence applies to vegetation and thermochemical
roadmaps.

### Production default first; proposals are shadow-only

Production always runs the unchanged deterministic default initialization and
classical solver. After authority is selected, a bounded shadow evaluator may
canonicalize one proposal and apply it to a non-authoritative solver copy for
comparison. Missing, late, invalid or unavailable output simply removes that
shadow sample.

Shadow non-convergence or invariant failure is telemetry. It cannot reject,
delay, retry or change the production step, race model candidates or switch the
production CPU/GPU/backend.

### Exact roots, not approximate similarity

An N1 research profile compares exact canonical owner roots, outcome/event
order and failure classification against the default classical path on the
frozen corpus, targets and continuation windows. Tolerance metrics remain
report-only. Passing the corpus does not authorize runtime consumption.

A future runtime proposal requires a consumer-backed Accepted ADR and a
mechanically checkable admissibility certificate proving identical canonical
result and failure class for every proposal in the admitted domain. Corpus
agreement alone is insufficient.

Training artifacts follow SPEC-34/ADR-053: hash-closed data, immutable bundle,
fixed splits, external heavy storage, no online training or hidden recurrent
state. Current `ModelLaneV1` is not expanded without a real consumer.

### Production authority is the unchanged classical path

The target owner always uses its normal deterministic default initialization.
This does not authorize a different material law, cheaper solver, frozen state
or alternative backend. Under this ADR the optional bundle affects only
shadow telemetry and cannot be required for activation, gameplay or restore.

## Failure and stop conditions

Malformed or unavailable output is shadow absence. A shadow solver failure is
recorded without affecting the owner step. Model lineage mismatch rejects the
shadow bundle and leaves the classical owner unchanged.

After two evidence-backed optimization cycles without a predeclared end-to-end
p95/p99 improvement and exact non-regression, the branch stops. We do not lower
classical gates, accept approximate roots, enlarge the budget or grant
model/GPU authority without a new decision.

## Alternatives rejected

- Learned solver as a peer owner: creates a second writer and ambiguous
  persistence.
- GPU classical baseline as automatic authority: conflicts with the selected
  CPU canonical paths and correspondence-only mirrors.
- Feed a corpus-validated proposal into production and retry on failure:
  changes failure semantics and lacks the ADR-081 mechanical certificate.
- Tolerance-based authoritative equivalence: insufficient for command/event/
  checkpoint roots.
- Online adaptation: creates hidden future-affecting state and unreproducible
  lineage.

## Consequences

- SPEC-44 and this ADR remain `Proposed`; no model lane, schema, provider or
  runtime dependency is added.
- N0/N1 work is optional, report/shadow-only and downstream of the classical
  owner, never a gate for its promotion or part of production work.
- `WORLD-DYNAMICS-P1` counts promoted state owners, not models; neural
  assistance cannot satisfy its owner threshold.
- The next action is selecting one post-promotion bottleneck and freezing N0
  instrumentation, not training a model.
