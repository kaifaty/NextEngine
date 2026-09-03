---
name: nextengine-architecture
description: "Use for NextEngine changes to public contracts, authority boundaries, persistence/replay guarantees, cross-subsystem dependency direction, accepted SPEC/ADR semantics, or product-level roadmap scope and status. Do not use for bounded lab experiments, model training/tuning, ordinary local implementation, or status reporting that preserves those boundaries. Russian triggers include: архитектура движка, публичный контракт, ADR, SPEC, граница авторитета, детерминизм реплея, схема сохранений, продуктовый roadmap."
---

# Next Engine architecture workflow

All paths are relative to the workspace containing `docs/architecture/`.
Architecture protects product semantics; it must not turn a reversible
experiment into a documentation program.

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
3. On conflict apply repository precedence: newer superseding Accepted ADR,
   workflow/profile ADR, subsystem SPEC, SPEC-00, then glossary.
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

## Outcome and solution selection

- Define the primary user-observable deliverable before supporting machinery.
  For an audio task, a WAV or running scene is primary; a validator, manifest,
  protocol or roadmap is supporting work unless explicitly requested.
- Each bounded iteration should produce that artifact or a decisive blocker.
  After two supporting-only checkpoints, stop adding infrastructure and switch
  to the smallest end-to-end vertical slice, reporting any reduced claim.
- Prefer the simplest design satisfying current product, safety, determinism and
  verification requirements. Add abstractions, layers or coordination only for
  a demonstrated constraint.
- Compare alternatives only on criteria that can change the decision. Record
  complexity or benchmarks only when material to the chosen design.
- Prefer working code and focused tests over a planning document. A reversible
  local experiment needs no pre-implementation protocol unless it opens
  protected evidence, consumes a substantial irreversible budget or the user
  explicitly requested preregistration.
- Apply hard constraints at the boundary actually being changed; do not expand a
  local experiment into a redesign of adjacent systems.

## Documentation guard

- Maintain one stable roadmap. Revise or replace it only when the objective,
  product sequencing or governing semantics change, not after each negative run.
- Prefer one implementation/result commit. Do not create separate freeze,
  protocol, conformance and result commits for one reversible experiment.
- Write a report when it records a reusable decision. File count, hashes and
  repeat-exact runs are not progress toward a different primary deliverable.

## Maintenance

The routing table remains the task-to-document and product-check authority.
Update this skill only when its trigger boundary or architecture workflow
changes.
