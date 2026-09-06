---
name: using-determinism-and-replay
description: Design or inspect determinism and replay boundaries, including seed/RNG ownership, snapshots, divergence localization, concurrency, floating-point behavior and external effects. Use for recoverable simulation behavior or cross-process/machine reproducibility. In NextEngine, use its architecture skill for normative contracts and product checks.
---

# Using Determinism and Replay

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

Design or inspect the boundary that makes past behavior reproducible. Establish
the required equivalence, inputs and state ownership from the actual product
contract. In NextEngine, route normative changes through
[nextengine-architecture](../nextengine-architecture/SKILL.md); generic replay
tiers never replace Accepted SPEC/ADR or admit an Isaac mirror as authority.

## Establish the claim

Identify the workload, equivalence predicate, platform/version envelope and the
observable that would falsify the claim. Distinguish bit equality, logical
equivalence and statistical reproducibility. Reuse settled repository definitions
instead of reopening them or asking the user to restate them.

Read the relevant topic reference for the affected source of divergence:
seed lineage, RNG ownership, snapshot completeness, comparison points, replay
mode, concurrency, floating-point operations, GPU kernels, external effects or
canonical encoding. Do not load the entire catalog for a localized defect.

## Preserve the substantive invariants

- Bind seeds and RNG streams to stable logical identities; capture enough state
  to restore continuation rather than merely reseeding.
- Separate authoritative state from reconstructible caches and record the
  external inputs needed by replay. Do not consult live effects during replay.
- Declare compare points and canonical representation; localize divergence to
  the first differing operation before changing tolerances or schedules.
- Distinguish read-only replay from branching replay and verify the capability
  actually claimed. Include snapshot versions and compatibility boundaries.
- Verify relevant scheduling, arithmetic, library and hardware constraints.
  A GPU setting or finite successful sample does not prove portable bit equality.
- Preserve minimized failing inputs. Never hide deterministic failures with
  sleeps, retries, implicit tolerances or post-hoc frame selection.

## Scope design and verification

Use the repository's existing documents and tests. Numbered spec sets, tiers,
cross-pack agents and generic consistency gates in references are optional
planning aids for a requested full-system design. They do not mandate new
documents, dependencies, CI or stricter equivalence than the product requires.
Missing optional slash commands do not block direct inspection and probes.

For an implementation change, run the relevant reproduction and non-regression
checks, including persistence/replay checks when authoritative state changes.
Choose property tests or wider platform comparisons when they test a material
claim. Do not weaken a frozen contract to meet performance; report the conflict
and follow the normal semantic decision workflow. State the exact evidence,
supported equivalence envelope and untested boundaries in the handoff.

## Topic references

Read only the reference relevant to the next decision. Paths are relative to this skill.

- [determinism-vs-reproducibility.md](determinism-vs-reproducibility.md) — Fixing the vocabulary; bit-exact vs logical-equivalence vs statistical; choosing a class
- [seed-governance.md](seed-governance.md) — Seeds as inputs; storage, propagation, derivation, audit; the `time.time()` anti-pattern
- [rng-isolation-patterns.md](rng-isolation-patterns.md) — Per-component RNGs, hierarchical seeding, the "one big RNG" anti-pattern, RNG ownership
- [snapshot-strategy.md](snapshot-strategy.md) — Full vs delta vs event-sourced; tradeoffs at different tick rates; what's in the snapshot, what isn't
- [divergence-detection-and-localisation.md](divergence-detection-and-localisation.md) — Compare-points, state hashing, binary-search bisection, the first-differing-op rule
- [replay-infrastructure-design.md](replay-infrastructure-design.md) — Read-only vs branching replay, rewind primitives, replay loop architecture, lifecycle
- [determinism-under-concurrency.md](determinism-under-concurrency.md) — Lockstep, recorded schedule, schedule-independent computation; per-strategy tradeoffs; schedule-sensitive operation catalog
- [floating-point-determinism.md](floating-point-determinism.md) — Reduction order, FMA, BLAS pinning, denormal mode, transcendental policy, ε for non-bit-exact classes
- [gpu-determinism.md](gpu-determinism.md) — cuDNN flags, atomic-float kernels, TF32 policy, NCCL determinism, driver pinning, cross-device replay
- [external-effects-substitution.md](external-effects-substitution.md) — Time, IO, network, third-party calls; the Effects layer; record-and-replay vs deterministic-function vs (test-only) mocks
- [canonical-state-encoding-for-replay.md](canonical-state-encoding-for-replay.md) — The bytes problem for snapshots; cross-link to `axiom-audit-pipelines:canonical-encoding-for-fingerprinting`; per-tick hashing patterns; tensor canonicalisation; snapshot envelope schema
- [property-tests-as-determinism-checks.md](property-tests-as-determinism-checks.md) — Replay equivalence, seed isolation, snapshot round-trip, restore idempotence, fork-and-converge, schedule independence; Hypothesis / proptest patterns
- [cost-of-determinism.md](cost-of-determinism.md) — Performance hit, library compatibility loss, refactoring overhead, operational discipline, cognitive load; when *not* to pay; the trade record
