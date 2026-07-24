# ADR-020: RPG domain authority и extension boundary

| Поле | Значение |
|---|---|
| ID | ADR-020 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | RPG Framework Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Gameplay Extensibility Team, Agent Intelligence Team, Asset & Persistence Team, World Services Team, Security & Governance Team, Verification & Evidence Team |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR принят в architecture packet 1.7 вместе с RPG-domain foundation contract. Он separates authoritative generic RPG state from extension execution and presentation without selecting an ECS, database, scripting, plugin, AI or UI backend. Принятие решения не объявляет RPG gates, runtime implementation, `vertical-v1` или release readiness пройденными.

## Контекст

Generic RPG aggregates and Luau/Wasm execution were described in one subsystem, while mechanics packages, AI memory, dialogue generation, presentation and persistence each needed RPG facts. Without a separate authority decision, aggregate transitions, inventory/equipment atomicity, narrative commitments, faction/relationship semantics and migrations could move into scripts, reducer state, AI memory, UI state or a shared mutable database. That would create multiple sources of truth, hidden first-party paths and replay-dependent partial commits.

## Решение

1. RPG Framework is the sole owner of revisioned `Character`, `Item`, `Inventory`, `Equipment`, `Quest`, `Dialogue`, `Faction`, `FactionMembership`, `Relationship` and `InteractiveObject` authoritative aggregates.
2. Every aggregate uses an engine-owned typed envelope with `PersistentId`, schema version, `u64` domain revision, exact definition reference and immutable view. A committed transaction increments each changed aggregate exactly once; rejection changes none and overflow fails closed.
3. RPG mutation is expressed only as a closed typed engine-owned operation inside `WorldCommand`. Arbitrary field patching, mutable ECS access, backend/database objects and package-private operation variants are forbidden.
4. Only RPG validation constructs immutable `RpgTransactionPlan` from the validator-computed ADR-022 command identity, exact target revisions, definition/policy hashes and immutable cross-context facts. Caller input never contains its own authoritative command ID or a trusted transaction plan.
5. Runtime executes `validate → stage copy-on-write → commit atomically` at one deterministic commit point. The complete write set, terminal receipt and DomainEvent batch become visible together or not at all; stale revision, conflict, abort or crash cannot publish a subset.
6. Operations have contiguous stable slots; events have operation-local slots and a total schema/aggregate/ID tie-break. Canonical event position is the ADR-022 event slot, so storage iteration, worker completion and subscriber order cannot change event order or ID.
7. Mechanics, Luau, Wasm, AI, physical/world services and presentation receive only capability-filtered immutable views and proposal/command sinks. They cannot write RPG storage, construct/modify the plan, pre-commit a narrative fact or retain a second mutable owner.
8. First-party combat, magic, crafting and trade use the same typed operations, capabilities, validators, migrations and gates available to community packages. A hidden Rust service, first-party store or bypass API is non-conforming.
9. RPG save migration supports the exact adjacent copy-on-write chain `N-2 → N-1 → N`. It preserves domain revisions and causal/idempotency history, emits no gameplay event and publishes a new generation only after all definition, transition, bound, reference and cross-aggregate checks pass. Any failure preserves original bytes and the prior generation.
10. Calendar, time, schedules, population and reservations remain World Services-owned. RPG segments and migrations cannot retain a mutable calendar or population copy.
11. RPG validation/planning/staging/commit spans belong to ADR-016 `core-command-rpg`; extension proposal work stays in its own owner row. Functional RPG gate PASS cannot substitute for integrated `PERF-01`.
12. Public contracts contain only engine-owned nominal IDs, typed values, immutable views/plans/events and canonical hashes. ECS/vendor/backend/OS/VM/database/task objects and a cross-context shared mutable store are forbidden.

## Рассмотренные варианты

- Script-owned quest/dialogue/inventory state — `Rejected`: VM lifecycle, package order and breaker state would become RPG authority.
- Mechanics Runtime owns arbitrary RPG fields — `Rejected`: package state would bypass aggregate ownership, common validation and migration.
- AI memory or generated dialogue updates relationships/commitments directly — `Rejected`: an untrusted proposal would become authoritative fact.
- Presentation/UI/camera state commits dialogue, targeting or inventory outcome — `Rejected`: renderer and input timing would choose gameplay truth.
- Partial multi-aggregate commit with later compensation — `Rejected`: receipts, replay and failure semantics would be ambiguous.
- Shared mutable RPG database across Runtime, Persistence and extensions — `Rejected`: it creates dual writers and leaks storage/backend types.
- Separate first-party hardcoded combat/crafting stores — `Rejected`: it violates ADR-008 dogfooding and denies community packages equivalent capability.
- In-place or best-effort save migration — `Rejected`: a failed step could destroy the only valid generation or silently discard causal history.

## Последствия

- The engine must maintain explicit aggregate, operation, plan, event and migration schemas plus deterministic positive/failure fixtures.
- Packages remain expressive through generic typed operations and mechanics state but cannot add raw RPG fields or private mutation paths.
- Persistence stores more revision, definition and causal metadata, gaining exact stale/conflict/migration diagnostics and atomic recovery.
- Narrative/model and presentation innovation remains isolated from commitment and save authority.
- Replaceable implementations remain behind engine-owned boundaries; this decision accepts no external technology or backend.

## Gates и fallback

The decision is verified by exact blocking gates `RPG-DOMAIN-P1`, `RPG-TRANSACTION-P1` and `RPG-MIGRATION-P1`. Their canonical descriptors are owned by the RPG-domain specification; this ADR does not redefine them. Failing domain schema/operation rejects the command or pins the prior compatible schema, failing plan retains the prior atomic checkpoint, and failing migration retains the original save/prior generation. Deterministic mismatch remains `NONDETERMINISTIC_RESULT` and cannot pass by retry.

## Supersession

ADR-020 coexists with ADR-008, ADR-014, ADR-016, ADR-022 and ADR-023 and does not supersede them. Moving RPG authority into script/plugin/mechanics/AI/presentation/world-service state, allowing partial domain transaction, in-place migration, hidden first-party path, shared mutable store or forbidden public backend type requires a new superseding ADR with synchronized specifications, gates and traceability.
