---
name: using-determinism-and-replay
description: Use only when designing or materially changing replay architecture, seed/RNG ownership, snapshot semantics, divergence localization, or cross-process/cross-machine equivalence guarantees. Do not use for a seeded offline experiment, deterministic content cooker, pure transform, model training run, or focused repeatability test under the existing contract.
---

# Using Determinism and Replay

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

This skill handles changes to the meaning or architecture of replay. It is not a
default reproducibility checklist.

## Scope gate

If the task is a bounded implementation under an existing determinism contract,
stop here: apply that contract and add the smallest focused repeatability test.
Do not emit a determinism specification or numbered artifact set.

For an actual architecture change, first state the equivalence predicate: under
which inputs and environment are two runs considered equal—identical bytes,
logical equality with named tolerances, or statistical equivalence?

## Read only the affected channel

| Changed concern | Reference |
| --- | --- |
| Equivalence class/vocabulary | [determinism-vs-reproducibility.md](determinism-vs-reproducibility.md) |
| Seed derivation or RNG ownership | [seed-governance.md](seed-governance.md), then [rng-isolation-patterns.md](rng-isolation-patterns.md) if needed |
| Snapshot contents/cadence | [snapshot-strategy.md](snapshot-strategy.md) |
| First-difference localization | [divergence-detection-and-localisation.md](divergence-detection-and-localisation.md) |
| Replay/branching lifecycle | [replay-infrastructure-design.md](replay-infrastructure-design.md) |
| Threads/processes/scheduling | [determinism-under-concurrency.md](determinism-under-concurrency.md) |
| Floating-point behavior | [floating-point-determinism.md](floating-point-determinism.md) |
| GPU behavior | [gpu-determinism.md](gpu-determinism.md) |
| Time/network/external effects | [external-effects-substitution.md](external-effects-substitution.md) |
| Canonical snapshot bytes | [canonical-state-encoding-for-replay.md](canonical-state-encoding-for-replay.md) |
| Property checks | [property-tests-as-determinism-checks.md](property-tests-as-determinism-checks.md) |
| Cost/benefit boundary | [cost-of-determinism.md](cost-of-determinism.md) |

Load multiple channels only when the requested semantic change actually couples
them. Do not re-emit unaffected specifications.

## Implement and verify

- Update the existing governing contract rather than creating a parallel
  determinism document.
- Implement the smallest changed boundary and a test vector or replay check that
  observes it.
- Localize a divergence before redesigning the system.
- Do not demand cross-device byte identity when the product contract requires
  only logical or statistical equivalence.
- Report the achieved equivalence and remaining environment assumptions.

A comprehensive replay architecture review may use all relevant references only
when the user explicitly requests that review or a genuinely new substrate is
being designed.
