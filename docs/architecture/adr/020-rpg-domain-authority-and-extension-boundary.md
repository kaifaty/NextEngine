# ADR-020: RPG domain authority и extension boundary

| Поле | Значение |
|---|---|
| ID | ADR-020 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-07](../07-rpg-scripting-and-plugins.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-008](008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Контекст

Generic RPG aggregates and Luau/Wasm execution were described in one subsystem, while mechanics packages, AI memory, dialogue generation, presentation and persistence each needed RPG facts. Without a separate authority decision, aggregate transitions, inventory/equipment atomicity, narrative commitments, faction/relationship semantics and migrations could move into scripts, reducer state, AI memory, UI state or a shared mutable database. That would create multiple sources of truth, hidden first-party paths and replay-dependent partial commits.

## Решение

1. `RpgDomainState` is the sole authoritative store of revisioned `Character`, `Item`, `Inventory`, `Equipment`, `Quest`, `Dialogue`, `Faction`, `FactionMembership`, `Relationship` and `InteractiveObject` aggregates.
2. Every aggregate uses an engine-owned typed envelope with `PersistentId`, schema version, `u64` domain revision, exact definition reference and immutable view. A committed transaction increments each changed aggregate exactly once; rejection changes none and overflow fails closed.
3. RPG mutation is expressed only as a closed typed engine-owned operation inside `WorldCommand`. Arbitrary field patching, mutable ECS access, backend/database objects and package-private operation variants are forbidden.
4. Only RPG validation constructs immutable `RpgTransactionPlan` from the validator-computed ADR-022 command identity, exact target revisions, definition/policy hashes and immutable cross-context facts. Caller input never contains its own authoritative command ID or a trusted transaction plan.
5. Runtime executes `validate → stage copy-on-write → commit atomically` at one deterministic commit point. The complete write set, terminal receipt and DomainEvent batch become visible together or not at all; stale revision, conflict, abort or crash cannot publish a subset.
6. Operations have contiguous stable slots; events have operation-local slots and a total schema/aggregate/ID tie-break. Canonical event position is the ADR-022 event slot, so storage iteration, worker completion and subscriber order cannot change event order or ID.
7. Mechanics, Luau, Wasm, AI, physical/world services and presentation receive only capability-filtered immutable views and proposal/command sinks. They cannot write RPG storage, construct/modify the plan, pre-commit a narrative fact or retain a second mutable owner.
8. First-party combat, magic, crafting and trade use the same typed operations, capabilities, validators, migrations and public SDK available to community packages. A hidden Rust service, first-party store or bypass API is invalid.
9. RPG save migration supports the exact adjacent copy-on-write chain `N-2 → N-1 → N`. It preserves domain revisions and causal/idempotency history, emits no gameplay event and publishes a new generation only after all definition, transition, bound, reference and cross-aggregate checks pass. Any failure preserves original bytes and the prior generation.
10. Calendar, time, schedules, population and reservations remain World Services-owned. RPG segments and migrations cannot retain a mutable calendar or population copy.
11. RPG validation/planning/staging/commit work is charged to the ADR-016 `core-command-rpg` budget row; extension proposal work remains separately charged. The integrated performance check measures their combined cost.
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

## Product checks

| Scenario | Expected | Fallback |
|---|---|---|
| Multi-aggregate command, including stale revision, conflict and injected abort | The complete write set, terminal receipt and ordered event batch publish together exactly once, or no RPG revision changes | Reject the command and retain the prior atomic checkpoint |
| `N-2 → N-1 → N` save migration with malformed input or publication fault | A new generation appears only after all bounds, definitions, references and cross-aggregate invariants pass; deterministic replay remains exact | Keep the original bytes and prior generation |
| Script, package, AI or first-party mechanic attempts a private operation or direct store write | The attempt is denied before mutation; first-party and community mechanics use the same public path | Reject the proposal and keep immutable views unchanged |

## Supersession

ADR-020 coexists with ADR-008, ADR-014, ADR-016, ADR-022 and ADR-030 and does not supersede them. Moving RPG authority into script/plugin/mechanics/AI/presentation/world-service state, allowing partial domain transaction, in-place migration, hidden first-party path, shared mutable store or forbidden public backend type requires a new superseding ADR and corresponding updates to affected specifications.
