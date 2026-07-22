# SPEC-19: RPG domain и narrative state

| Поле | Значение |
|---|---|
| ID | SPEC-19 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | RPG Framework Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Gameplay Extensibility Team, Agent Intelligence Team, Asset & Persistence Team, World Services Team, Security & Governance Team, Verification & Evidence Team |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-006](adr/006-scripting-and-plugin-model.md), [ADR-007](adr/007-identities-persistence-and-replay.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md) |
| Заменяет | отсутствует |

## Статус предложения

Этот документ входит в post-1.5 foundation-completeness proposal и не изменяет Accepted packet 1.4, remediation candidate 1.5 или dialogue/model proposal SPEC-16/ADR-017. Он детализирует existing RPG ownership without accepting implementation. До atomic promotion SPEC-17…20 и ADR-018…021 lifecycle state — `AwaitingReview`.

## Назначение и invariants

SPEC-19 отделяет authoritative RPG domain от script/plugin execution и задаёт versioned contracts для character, item/inventory/equipment, quest, dialogue, faction/relationship и interactive-object state.

- Каждый RPG aggregate MUST иметь `PersistentId`, schema/revision, exactly one owner and explicit command/state transition rules.
- RPG state MUST изменяться only inside validated WorldCommand transaction.
- Multi-aggregate operations MUST be atomic; no partial inventory/trade/quest/dialogue commit.
- Content definitions, localized text, AI memory, presentation, script globals and mechanic reducer state MUST NOT become a second RPG source of truth.
- First-party and community mechanics MUST use the same EffectRequest/WorldCommand/domain validation path.
- Save/load/migration MUST preserve original data on failure and identify exact aggregate/definition/revision mismatch.
- Dialogue/model proposals MAY improve expression but cannot create commitments or domain mutation before validation/commit.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является source |
|---|---|---|
| Character attributes/resources/life/proficiency | RPG Framework Character aggregate | physics pose, animation state, AI memory |
| Item instance, quantity, durability, owner/location | RPG Framework Item/inventory aggregates | render attachment, package definition cache |
| Equipment assignment | RPG transaction connecting Character and Item revisions | motor policy route, visual socket |
| Quest state/variables/participants/causal history | RPG Framework Quest aggregate | dialogue text, script coroutine, planner goal |
| Dialogue session/choices/commitments/transcript policy | RPG Framework Dialogue aggregate | TTS playback, provider thread, free-form memory |
| Faction membership/rank and directed relationships | RPG Framework | AI sentiment cache, presentation labels |
| InteractiveObject gameplay state/occupancy | RPG Framework object aggregate + accepted reservation reference | physics contact, UI panel state |
| Mechanic definitions/state | SPEC-13 Mechanics Registry/Runtime | RPG aggregate fields outside declared operation |
| Schedule/population/reservation service state | World Services | RPG character state |

RPG Framework Team owns aggregate schemas, invariants, transaction planning and committed DomainEvents. Runtime owns transaction envelope/order/commit mechanics. Persistence owns segment serialization/migration orchestration but not domain meaning.

## Public boundary

Public RPG contracts are immutable aggregate/query views, typed command payloads, validation/rejection values and committed events. Each view contains schema version, aggregate PersistentId, revision and only capability-authorized fields. Internal ECS components, database rows, mutable references, VM objects and backend handles are forbidden.

Definition references use `AssetId + resolved ContentHash`. Runtime aggregate references use PersistentId and expected revision. Human/localized names never identify aggregates or transitions.

## Aggregate contracts

### Character

Character contains:

- PersistentId, archetype AssetId/content revision and aggregate revision;
- fixed-point attributes/resources with declared bounds and regeneration policy IDs;
- versioned life-state ID and explicit transition rules;
- fixed-point namespaced SkillProficiency map per SPEC-14;
- inventory container reference and equipped-slot map;
- faction memberships/ranks and directed relationship dimensions;
- active quest/dialogue/interaction references;
- embodiment PersistentId/reference, never physics/backend handle.

Physical contact/motor outcome may propose RPG effect/command but cannot write health, stamina, inventory or life state directly. Character destruction does not automatically delete durable items/quests/relationships; an explicit domain policy transaction resolves them.

### Item, inventory и equipment

Item instance contains PersistentId, definition AssetId/hash, quantity, durability, bounded custom state schema, current owner/location and revision. Stack merge/split derives deterministic instance/provenance rules and cannot reuse tombstoned IDs.

Inventory/equipment operation declares source/destination container revisions, item revision/quantity, capacity/slot/precondition policy and causal command. Transaction validation checks ownership, reservations, capacity, stack compatibility, required skill/capability and physical availability. Successful transfer updates every affected aggregate once and emits ordered events; rejection updates none.

Equipment visual attachment and motor compatibility follow committed RPG slot assignment. Renderer/motor may reject or degrade their own route, but cannot silently revert committed equipment; any gameplay rollback is a new command transaction.

### Quest

Quest definition declares versioned states, legal transitions, typed variables, participants, preconditions, effects and completion/failure semantics. Runtime Quest aggregate records current state, variables, participants, definition hash, revision and causal event/command history sufficient for diagnostics/migration.

Transition is idempotent by command identity and expected revision. Missing definition, illegal transition, stale participant or failed multi-aggregate effect rejects the whole transaction. Script/AI may propose transition only by stable transition ID.

### Dialogue и commitments

Dialogue aggregate records session ID, participants, definition/node/state, available stable choice IDs, pending/committed commitments, transcript policy, revision and causal history. Presentation text/audio timing is non-authoritative.

Choice or generated candidate may create bounded command proposals. A quest, trade, inventory, relationship or faction statement becomes committed narrative fact only after all corresponding commands commit. Rejected proposal produces explicit rejection/fallback and no speculative commitment.

If SPEC-16/ADR-017 is later Accepted, its CanonicalUtterance/turn protocol specializes dialogue input, model routing and streaming presentation. It does not change RPG ownership, transaction order or commitment rule. Until then, authored text/choice and deterministic fallback from SPEC-06/07 remain baseline.

### Faction и relationships

Faction aggregate contains PersistentId, definition hash, memberships/ranks, generic policies and directed faction relations. Character relationship dimensions are versioned fixed-point values with declared bounds and policy IDs. Symmetry is not implicit: inverse updates require explicit transaction operation.

Relationship/faction changes identify source fact/command, expected revisions and allowed policy. AI memory may store recollection but not overwrite current values. Bulk reputation effects expand to a deterministic ordered operation set before commit.

### InteractiveObject

InteractiveObject contains PersistentId, definition hash, explicit state machine, capabilities, occupancy/reservation reference, owner/region and revision. Physical overlap is candidate fact only. Use/lock/open/activate/consume operations validate object state, actor capability, World Services reservation and physical preconditions before transaction.

## Command validation и transaction model

Every RPG command payload declares schema, operation ID, issuer/capability, target PersistentIds + expected revisions, typed inputs, causal command ID and definition/policy hashes.

Validation order:

1. schema/size/ID and capability;
2. exact project/content/definition compatibility;
3. target existence, expected revisions and owner/reservation;
4. aggregate-local state-machine and bound invariants;
5. cross-aggregate preconditions in sorted PersistentId order;
6. mechanic/physical/world facts through immutable queries;
7. construct complete mutation/event set;
8. atomic commit at Runtime stage or reject all.

DomainEvent is emitted only from commit and contains causal command ID, aggregate IDs/revisions and typed payload. Subscriber may propose future command but cannot re-enter current transaction.

## Mechanics, scripts, plugins и AI boundary

- SPEC-13 Ability/Effect/Status/Projectile/AreaField mechanisms can request declared RPG operations; package reducers cannot write aggregate storage.
- Crafting, combat, magic, trade modifiers and custom progression remain ordinary packages over generic RPG operations and mechanics state.
- Luau/Wasm receive capability-filtered views and command sink. VM globals are not durable RPG state.
- AgentIntent/dialogue/model output is untrusted. Planner reads affordances/views and submits standard operations.
- UI reads immutable semantic views and emits PlayerAction-derived command candidates.

## Persistence и migrations

RPG save segment contains ordered aggregate records, definition hashes, revisions, causal/idempotency ledgers required by domain, and cross-segment PersistentId references. It does not contain ECS IDs, localized labels, script coroutine/VM state or presentation/audio state.

Migration is a pure ordered transform on a copy. It validates every referenced definition, state/transition mapping, numeric bound and cross-aggregate invariant before publishing new segment. Missing required definition/migration or any invalid record rejects load and preserves original save. Package uninstall/export follows SPEC-13 but cannot silently discard RPG fields.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `RPG_AGGREGATE_NOT_FOUND` | Reject command; no substitute by name/runtime ID |
| `RPG_REVISION_STALE` | Reject complete transaction with current revision hint |
| `RPG_TRANSITION_INVALID` | Reject illegal state/quest/dialogue/object transition; no event |
| `RPG_OWNERSHIP_CONFLICT` | Reject inventory/equipment/trade transaction atomically |
| `RPG_RESERVATION_INVALID` | Reject interaction; request/retry only as future command |
| `RPG_DEFINITION_MISMATCH` | Fail command/load before mutation; preserve save/project |
| `RPG_TRANSACTION_ABORTED` | Roll back staged mutation/event set; committed state unchanged |
| `RPG_MIGRATION_REQUIRED` | Fail closed with exact schema/definition path; preserve original bytes |
| `RPG_COMMITMENT_REJECTED` | Do not present stateful claim as fact; use authored rejection/fallback |
| `NONDETERMINISTIC_RESULT` | Gate fails at first divergent command/aggregate/event; no retry-to-green |

## Proposed gates

| Gate | Owner | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|
| `RPG-DOMAIN-P1` | RPG Framework | `next gate RPG-DOMAIN-P1 --scenario rpg-aggregate-corpus --commands 10000` | 10,000 valid/invalid character/item/quest/dialogue/faction/object commands produce exact accept/reject/state/event hashes over 100 repeats; bounds/invariants 100% enforced | command corpus, aggregate snapshots, event/state hashes, diagnostics | block invalid definition/command; pin prior schema |
| `RPG-TRANSACTION-P1` | RPG Framework + Runtime | `next gate RPG-TRANSACTION-P1 --scenario rpg-atomic-faults --all-commit-points` | 0 partial item/equipment/trade/quest/dialogue commits across conflict, stale revision, injected abort/crash; exact event order | transaction plan/commit traces, fault matrix, state roots | abort all staged changes; retain prior revisions |
| `RPG-MIGRATION-P1` | RPG Framework + Asset & Persistence | `next gate RPG-MIGRATION-P1 --scenario rpg-save-migrations --versions n-2,n-1,n` | valid fixtures exact expected hashes; 100% missing definition/transition/invalid bounds fail closed; original save hash unchanged on failure | migration corpus/report, before/after hashes, diagnostics | use prior engine/package or explicit export tool |

## Foundation requirement aliases

| Alias | Primary owner | Future acceptance contract |
|---|---|---|
| `FND-RPG-R1` | RPG Framework | Every generic RPG aggregate has one owner, revisioned view and explicit state machine |
| `FND-RPG-R2` | RPG Framework | Inventory/equipment/quest/dialogue/faction/object changes commit atomically through WorldCommand |
| `FND-RPG-R3` | Gameplay Extensibility | First-party/community mechanics use common RPG operations without hidden store or API |
| `FND-RPG-R4` | Asset & Persistence | RPG save segments and migrations preserve definitions, revisions and causal history fail closed |
| `FND-RPG-F1` | RPG Framework | Invalid/stale/conflicting multi-aggregate operation produces no partial mutation/event |
| `FND-RPG-F2` | RPG Framework | Missing/incompatible definition or migration preserves original save and exact prior state |

## Promotion contract

Promotion occurs only with SPEC-17/18/20 and ADR-018…021 in one reviewed transaction. It updates SPEC-00/01/02/03/06/07/08/09/11/12/13/14/15, glossary and traceability; maps RPG-DOMAIN/TRANSACTION/MIGRATION gates into VS-02/03/06/11/13; and preserves exactly fifteen VS gates.

Foundation alias allocation follows SPEC-17: REQ-087…102 and FAIL-031…038 after accepted SPEC-16, or REQ-079…094 and FAIL-025…032 after its formal rejection/withdrawal. Pending SPEC-16 blocks promotion but not review. No external technology row is added.
