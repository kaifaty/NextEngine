---
name: nextengine-architecture
description: Route Next Engine development tasks to the governing architecture documents before making changes. Use for any architecture-sensitive, cross-cutting, or roadmap-sensitive work in the NextEngine repository (the workspace containing docs/architecture/): public contracts in crates/contracts, WorldCommand/DomainEvent, determinism, replay, command identity, save/load persistence, schema migrations, ECS runtime, assets/streaming/bundles, renderer/Vulkan/shaders, physics/collision, animation/IK, motor control/ML policies/inference, AI agents/perception/LLM boundary, RPG domain/quests/divine standing/narrative director, Luau/Wasm scripting/plugins/mod packages, UI/camera/localization/accessibility, project composition/configuration, platform host/application session, jobs/memory/allocator/performance budgets, tooling/SDK/observability, gothic importer boundary, licensing/security, product checks, ADR/SPEC changes, roadmap stages/blockers/exit criteria. Триггеры на русском: архитектура движка, спеки, ADR, детерминизм, реплей, сохранения, миграции схем, контракты, квесты, физика, рендер, анимация, скрипты, плагины, роадмап, продукт-чеки. Prevents acting without reading the governing SPEC/ADR first.
---

# Next Engine architecture router

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
5. Implement through production paths only: gameplay state changes via
   validated `WorldCommand` transactions, `DomainEvent` from committed changes,
   no test-only mutation backdoors, no retry-to-green.
6. Run the product checks mapped by the routing row (`fast` always; `play`,
   `persistence-replay`, `content-package`, `platform`, `performance` per row).
   In the handoff report each relevant check as passed / failed / not run with
   the remaining product risk.
7. Roadmap-sensitive work (scope, stage, blocker, exit criterion, subsystem
   status): also read `docs/roadmap.md` first and update it in the same change
   when the completed work materially changes its facts.
8. Semantic architecture change: add a new ADR naming what it supersedes and
   update the affected SPECs, `docs/architecture/README.md` index,
   `traceability.md` and `agent-routing.md` in the same change.

## Maintenance

The routing table lives in the repository and is the single source of truth;
this skill only encodes the procedure. After adding or superseding a SPEC/ADR,
update `agent-routing.md` in that change — do not edit this skill.
