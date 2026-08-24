---
name: nextengine-architecture
description: "Route cross-cutting NextEngine architecture/roadmap via SPEC/ADRs; choose optimal algorithms/data structures. Use for contracts, commands/events, determinism/replay/persistence/migrations, schemas/ECS, assets/streaming, rendering/platform/physics/animation/motor, ML/AI/RPG, scripting/plugins/UI/sessions, jobs/memory/performance, tooling/importer/licensing/security, ProductChecks and ADR/SPEC/roadmap changes. Russian: архитектура, спеки/ADR, детерминизм/реплей/сохранения/миграции, контракты, алгоритмы/структуры данных/производительность, физика/рендер/анимация, плагины, роадмап/продукт-чеки."
---

# Next Engine architecture workflow

All doc paths below are relative to the NextEngine workspace root — the
directory containing `docs/architecture/`. Resolve them against the current
workspace; never assume a fixed absolute path, drive letter or checkout
location.

## Procedure

1. Read `docs/architecture/agent-routing.md` and find the row(s) matching the
   task. A change matching multiple rows inherits the documents and product
   checks of all of them.
2. Read every listed SPEC/ADR **in full** with direct file reads. Never act on
   search snippets or ranking scores. When no row matches or in doubt, read the
   full index in `docs/architecture/README.md` plus `glossary.md`.
3. On semantic conflict apply precedence from the README: newer superseding
   Accepted ADR → ADR-030 (workflow and product checks) → profile technical
   ADR → subsystem SPEC → SPEC-00 → glossary.
4. Treat `Proposed` documents (SPEC-16/ADR-017, PhysX backend ADR-033) as not
   shipped: state the fallback and bounded evaluation path. Treat the
   historical-only list in the routing file (ADR-004/006/007/010/012/015/023/
   024, evidence register, review packets) as context, never as authority.
5. Before planning, implementing, optimizing or reviewing a solution, apply
   the selection rules below.
6. Implement through production paths only: gameplay state changes via
   validated `WorldCommand` transactions, `DomainEvent` from committed changes,
   no test-only mutation backdoors, no retry-to-green.
7. A Git commit is a checkpoint, not a validation gate: never run checks merely
   because a commit is about to be created. Before final handoff/readiness
   claims, run the risk-scoped product checks mapped by the routing row;
   documentation-only work uses the SPEC-12 cheap path, while executable work
   uses applicable `fast`, `play`, `persistence-replay`, `content-package`,
   `platform` and `performance` checks. Report each as passed / failed / not run
   with the remaining product risk.
8. Roadmap-sensitive work (scope, stage, blocker, exit criterion, subsystem
   status): also read `docs/roadmap.md` first and update it in the same change
   when the completed work materially changes its facts.
9. Semantic architecture change: add a new ADR naming what it supersedes and
   update the affected SPECs, `docs/architecture/README.md` index,
   `traceability.md` and `agent-routing.md` in the same change.

## Solution selection

- Define the required result, actual input sizes and load, hard constraints and
  limiting resource before choosing a design.
- Keep settled engineering selection here. When a choice depends on an
  unresolved claim about stability, convergence, conservation, conditioning,
  error bounds, solver feasibility or physical-model validity, use
  `$nextengine-mathematical-research` to return a claim-scoped report; then
  resume this workflow for any semantic, roadmap or ProductCheck decision.
- Prefer the simplest architecture that satisfies current product,
  architecture, safety and verification requirements. Avoid speculative
  abstractions, unnecessary layers, ceremonies and coordination structures.
  In plans and comparisons, default to the smallest practical path with the
  best result-to-effort ratio; add architectural complexity only for a
  demonstrated constraint or risk.
- For significant choices, compare correctness, worst- and expected-case time,
  memory, constant factors and target measurements. Optimize the limiting
  resource; do not use code readability as a selection criterion.
- Use proven algorithms and data structures from olympiad and competitive
  programming as a production toolbox. Check the standard library and already
  accepted dependencies first; implement a custom variant when no suitable
  option exists or it is more efficient under the declared constraints.
- Briefly record non-trivial alternatives, time and space complexity, and the
  benchmark when the result depends on the target profile.
- Keep correctness, determinism, safety, public contracts and testability as
  hard constraints. The ban on overengineering limits unjustified architecture
  and process; it does not forbid an algorithm made complex by real constraints.

## Maintenance

The routing table is the single source of truth for task-to-document and check
mapping; do not mirror its rows here. After adding or superseding a SPEC/ADR,
update `agent-routing.md` in that change. Update this skill only when its
trigger scope, workflow or engineering-selection rules change.
