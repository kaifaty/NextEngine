---
name: nextengine-architecture
description: "Use for NextEngine changes to public contracts, authority boundaries, persistence/replay guarantees, cross-subsystem dependency direction, accepted SPEC/ADR semantics, or product-level roadmap scope and status. Do not use for bounded lab experiments, model training/tuning, ordinary local implementation, or status reporting that preserves those boundaries. Russian triggers include: архитектура движка, публичный контракт, ADR, SPEC, граница авторитета, детерминизм реплея, схема сохранений, продуктовый roadmap."
---

# Next Engine architecture workflow

Apply the [shared execution guidance](../astra-guidance.md) once per task alongside this skill; it governs process defaults in the references too.

All paths are relative to the workspace containing `docs/architecture/`.
Architecture protects product semantics; it must not turn a reversible
experiment into a documentation program.
Shared outcome, documentation and verification policy lives in
[AGENTS.md](../../../AGENTS.md). Apply it without duplicating its checklists.

## Classify before loading architecture

Classify the immediate change, not the broad topic:

- **Architecture change:** alters a public contract, technical source of truth,
  authority boundary, cross-subsystem dependency, persistence/replay guarantee,
  Accepted decision or product-level roadmap fact. Use the full workflow below.
- **Bounded implementation or lab experiment:** preserves those semantics and
  has an existing fallback. Read the routing row to identify the boundary and
  focused check, plus only the governing document needed for that boundary. Do
  not create an ADR, replace a roadmap or load every related document.
- **Status or diagnosis:** inspect exact code/results needed for the claim. Do
  not turn the answer into an architecture change.

Admission gates restrict claims and promotion, not creation of clearly labelled
experimental artifacts. A lab candidate may be rendered, played or compared
without being represented as shipped or validated.

## Full architecture workflow

1. Read `docs/architecture/agent-routing.md` and find the row(s) matching the
   semantic change.
2. Read every SPEC/ADR governing the changed semantics in full. Search snippets
   are discovery aids, not authority. When no row matches, use the index and
   glossary to locate the actual owner; do not load unrelated documents.
3. On conflict apply the precedence defined in `AGENTS.md`; this skill does
   not establish a separate hierarchy for workflow or profile ADRs.
4. Treat Proposed technology as an experiment. State its fallback and do not
   present it as shipped before the affected product check passes.
5. Implement gameplay mutations through production commands and committed
   events; preserve public boundaries and avoid test-only mutation backdoors.
6. Run the risk-scoped checks from the routing row before handoff or readiness
   claims. A commit itself is not a validation gate.
7. For a product-level scope, order, stage or exit-criterion change, update the
   existing `docs/roadmap.md`. An individual experiment pass/fail is evidence,
   not grounds for a new numbered roadmap.
8. For a semantic architecture change, add a superseding ADR and update affected
   SPECs, the architecture index, traceability and routing in one coherent change.

## Mathematical selection boundary

When a semantic choice depends on an unresolved stability, convergence,
conservation, conditioning, error-bound or physical-model claim, use
`nextengine-mathematical-research` for that bounded discriminator, then resume
the contract decision here. A settled engineering choice does not require a
separate research campaign.

## Maintenance

The routing table remains the task-to-document and product-check authority.
Update this skill only when its trigger boundary or architecture workflow
changes.
