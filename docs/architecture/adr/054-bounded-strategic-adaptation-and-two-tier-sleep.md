# ADR-054: Bounded strategic adaptation and two-tier sleep

| Field | Value |
|---|---|
| ID | ADR-054 |
| Status | Proposed |
| Version | 1.0 |
| Decision date | 2026-08-09 |
| Last verified | 2026-08-09 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-33](../33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-050](050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md) |
| Supersedes | none while `Proposed` |
| Superseded by | none |

## Context

Strategic NPC behavior benefits from memory on multiple timescales, while the
engine requires every value that can affect a future decision to be bounded,
saveable, hashable and replayable. A literal runtime optimizer, hidden fast
weights or mutable per-NPC model bytes would create a second authority outside
the Agent owner and make equal save/replay inputs insufficient to recover a
decision.

The Nested Learning/Hope work motivates a multi-timescale reference profile,
and sleep-consolidation research motivates an offline consolidation workflow.
These papers are research inputs, not evidence that a particular NPC policy is
safe, higher quality or ready to ship:

- [Nested Learning / Hope](https://arxiv.org/abs/2512.24695)
- [Sleep consolidation](https://arxiv.org/abs/2606.03979)

## Decision candidate

### Hope-inspired reference profile

`HopeInspiredStrategicV1` is a reference training/evaluation profile, not a
public model enum or mandatory runtime architecture. The public boundary stays
the pure role evaluator:

```text
(observation, canonical candidates, prior bounded per-NPC policy state)
  → (one score per candidate, next bounded per-NPC policy state)
```

A GRU with the same input/output/state envelope is the mandatory comparator.
Hope-inspired quality is not claimed until the exact multi-seed held-out,
retention, save/replay and resource gates in SPEC-33/SPEC-34 beat or justify
their trade-off against that comparator and the utility/HTN baseline.

### Externalized adaptation state

Any conceptual fast weight, memory level, consolidation accumulator or other
future-decision-affecting value is represented entirely by versioned bounded
segments in `BehaviorPolicyStateRecordV1`. Every segment has a fixed width,
fixed-point descriptor, reset/consolidation rule and canonical hash coverage.
The complete record is saved and replayed with its NPC.

Runtime gradients, optimizer/scheduler state, base-weight mutation, per-NPC
weight files, hidden session tensors, evaluator-owned cache and shared mutable
memory across NPCs are forbidden. A state value that cannot be externalized,
bounded and deterministically transformed is not an admissible runtime
Hope-inspired feature.

### Runtime sleep consolidation

Runtime sleep is an engine-owned deterministic state transition, not training.
At a declared strategic boundary, the Agent owner MAY run a versioned pure
`RuntimeSleepConsolidationV1` transform over the current bounded policy state
and committed fact summary. It produces a complete next state record and
`consolidation_revision`, or no state changes occur.

Authored NPC sleep is one allowed trigger, alongside explicitly declared
logical boundaries such as a safe long-rest or chapter/session transition. A
wall clock, renderer frame, trainer callback or background completion cannot
trigger it. The authored `Sleep` goal and physical animation are observations
or triggers; they do not own the consolidation state.

Save/load/replay exact-compares pre/post state hashes, consolidation revision,
candidate scores, selected decision and resulting state roots. Oversized,
hidden, non-finite or incompatible state fails before field use and invokes
the declared planner fallback where safe.

### Offline Sleep/Dreaming

Offline Sleep/Dreaming is an `OfflineConsolidationCycle`, not runtime sleep. It
consumes an immutable dataset/trajectory corpus plus an immutable parent bundle
and produces a new immutable child candidate bundle. It never edits the parent,
active project, active world or save.

The consolidation manifest binds parent bundle/hash, corpus manifest/root,
data provenance and consent, seed derivation, algorithm/config/tool hashes,
retention suite, new-task suite and child export hash. The child must pass both
retention and joint Strategic/Tactical evaluation before publication. Failure
or catastrophic forgetting leaves the parent and project lock unchanged.

V1 has no active-session bundle hot swap. Selecting a successful child requires
an explicit exact project-lock change and a new session under ADR-053.

## Alternatives considered

- Literal runtime Hope optimizer — rejected because optimizer and weight
  mutation are hidden authoritative state and conflict with replay.
- Fully stateless strategic model — allowed as a comparator/fallback, but not
  required; it cannot represent the intended bounded adaptation hypothesis.
- Treat authored sleep as the only consolidation trigger — rejected because
  gameplay narrative and technical state lifecycle have different ownership.
- Mutate the active bundle during offline consolidation — rejected because
  artifact, project-lock and save identities would change in place.
- Publish `Hope` as a stable public architecture enum — rejected because the
  research implementation may change while the evaluator contract remains.

## Consequences

- Strategic adaptation becomes explicit data with exact reset, save, replay
  and migration semantics instead of evaluator behavior.
- Runtime sleep remains cheap and deterministic; expensive learning remains
  offline and statistically evaluated.
- The reference profile can be replaced without changing gameplay contracts.
- There is storage/validation cost proportional to the declared fixed state
  width; the profile must justify it against GRU and planner baselines.

## Promotion boundary

ADR-054 remains `Proposed` until one production strategic consumer externalizes
all adaptive state, passes exact strategic tick → runtime consolidation →
save/load/replay checks, rejects hidden/incompatible state, and passes the
pre-registered multi-seed Hope-vs-GRU plus offline retention/joint suites.
Research results or a training-only prototype cannot promote it.
