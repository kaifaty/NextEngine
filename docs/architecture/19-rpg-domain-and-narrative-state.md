# SPEC-19: RPG domain и narrative state

| Поле | Значение |
|---|---|
| ID | SPEC-19 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Gameplay Extensibility Team, Agent Intelligence Team, Asset & Persistence Team, World Services Team, Security & Governance Team, Verification & Evidence Team |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## История принятия

SPEC принят в architecture packet 1.7 вместе с ADR-020 как generic RPG-domain foundation contract. Принятие документа не объявляет runtime implementation, любой RPG gate, `vertical-v1` или release readiness пройденными и не выбирает ECS, database, scripting, plugin либо AI backend.

## Назначение и invariants

SPEC-19 отделяет authoritative RPG domain от mechanics/script/plugin/AI/presentation execution и задаёт versioned contracts для character, item/inventory/equipment, quest, dialogue, faction/membership/relationship и interactive-object state.

- Каждый RPG aggregate MUST использовать общий revisioned engine-owned envelope, иметь ровно одного owner и изменяться только явной typed operation.
- RPG state MUST изменяться только внутри validated `WorldCommand` transaction.
- Multi-aggregate operation MUST validate, stage and commit atomically through immutable `RpgTransactionPlan`; partial state, event или receipt publication запрещена.
- Equal command, aggregate revisions, definitions and immutable facts MUST produce identical plan, write set, event order and state root in `game`, `headless` and `capture-worker`.
- Content definitions, localized text, AI memory, presentation, script globals, mechanic reducer state, world calendar and backend caches MUST NOT become a second RPG source of truth.
- First-party and community packages MUST use the same typed RPG operations, capabilities and validation path. Hidden first-party operation, mutable store or bypass API запрещены.
- Save/load and supported `N-2 → N` migration MUST be copy-on-write, fail closed and preserve the original generation on any incompatibility.
- Mechanics, scripts, plugins, AI, physical/world services and presentation MAY read authorized immutable views and submit proposals; they cannot construct a trusted transaction plan or write aggregate storage.

## Source of truth и ownership

| Authoritative state | Единственный owner/source of truth | Не является source |
|---|---|---|
| Character attributes, resources, life state and proficiency | RPG Framework `Character` aggregate | physics pose, animation state, AI memory |
| Item definition, quantity, durability and bounded custom state | RPG Framework `Item` aggregate | render attachment, package definition cache |
| Inventory capacity, reservations and item membership | RPG Framework `Inventory` aggregate | UI list, ECS query index, Item view cache |
| Equipment slot assignment | RPG Framework `Equipment` aggregate | Character/equipment projection, motor route, visual socket |
| Quest state, typed variables, participants and causal history | RPG Framework `Quest` aggregate | dialogue text, script coroutine, planner goal |
| Dialogue session, choices, commitments and transcript policy | RPG Framework `Dialogue` aggregate | TTS playback, provider thread, free-form memory |
| Faction state and faction-to-faction policy | RPG Framework `Faction` aggregate | presentation label, package definition cache |
| Character membership and rank in one faction | RPG Framework `FactionMembership` aggregate | Character/Faction membership index |
| Directed relationship dimensions | RPG Framework `Relationship` aggregate | AI sentiment/recollection, UI label |
| Interactive-object gameplay state | RPG Framework `InteractiveObject` aggregate | physics contact, UI panel, World Services reservation |
| Mechanics definitions and namespaced mechanic state | SPEC-13 Mechanics Registry/Runtime | RPG fields outside a typed operation |
| Calendar, time, schedules, population and reservations | World Services | RPG save segment or Character fields |

RPG Framework Team owns aggregate schemas, state machines, invariant validation, transaction planning and committed RPG event schemas. Runtime Team owns command admission, deterministic commit point, atomic publication, receipt and event-ID derivation. Asset & Persistence Team owns segment serialization, copy-on-write migration orchestration and atomic save generation, but not RPG meaning. A cache or index derived from these owners MUST be reconstructible and MUST NOT accept independent writes.

RPG segments MUST NOT retain a mutable calendar, time, schedule or population copy. Migration from any legacy RPG calendar representation removes those fields before the new RPG segment is eligible for publication; World Services remains the sole authority for the corresponding value.

## Common aggregate contract

Every durable RPG record uses `RpgAggregateEnvelopeV1`:

```text
RpgAggregateEnvelopeV1 {
  aggregate_kind,
  persistent_id,
  schema_version,
  revision,
  definition_ref,
  provenance_hash,
  payload,
  payload_hash
}
```

- `aggregate_kind` is the closed enum `Character`, `Item`, `Inventory`, `Equipment`, `Quest`, `Dialogue`, `Faction`, `FactionMembership`, `Relationship` or `InteractiveObject`.
- `persistent_id` is a `PersistentId`; `RuntimeEntityId`, names and storage keys are forbidden as durable identity.
- `schema_version` is a positive `u32`. `revision` is a `u64` optimistic-concurrency counter.
- A definition-backed record uses exact `DefinitionRefV1 { asset_id: AssetId, content_hash: ContentHash }`. A kind whose identity is created entirely by a transaction uses the declared `None` variant, not an omitted field.
- `provenance_hash` binds the canonical creation/import/migration provenance record required by the aggregate kind; an explicitly provenance-free kind uses the declared `None` variant.
- `payload` is the closed typed schema selected by `aggregate_kind`; arbitrary field paths, dynamic maps as schema extension and backend values are forbidden.
- `payload_hash` is the canonical hash of the envelope payload and participates in plan/read-set validation.

Every mutating operation names the exact expected revision of each target. A committed transaction increments every changed aggregate from `r` to `r + 1` exactly once even when several operations touch it; an unchanged or rejected aggregate retains `r`. Revision overflow rejects the whole transaction as `RPG_REVISION_EXHAUSTED`. Migration changes schema representation but preserves the domain revision and causal history; it does not masquerade as gameplay mutation.

Public RPG views are immutable projections of an exact envelope revision. They expose only capability-authorized fields and definition references. ECS components/storage, raw pointers, mutable references, task/future handles, OS/window objects, database rows/connections, VM objects and vendor/backend handles MUST NOT appear in `crates/contracts` or any RPG public schema. Contexts MUST NOT share a mutable RPG database or object graph.

## Aggregate payloads

### Character

The Character envelope binds its archetype `DefinitionRefV1`. `CharacterPayloadV1` contains bounded fixed-point attributes/resources with policy IDs, explicit life-state ID, fixed-point namespaced `SkillProficiency`, and references to its Inventory and Equipment aggregate IDs. Faction memberships, directed relationships and equipped-slot maps are immutable query projections of their owning aggregates, not mutable Character copies.

Physical contact, motor outcome or Agent intent may propose a typed character operation but cannot write health, stamina, proficiency or life state. Character destruction does not delete items, quests, memberships or relationships implicitly; a complete transaction must state every affected operation.

### Item, Inventory и Equipment

The Item envelope binds its item-definition reference. `ItemPayloadV1` contains quantity, durability, bounded custom-state schema and provenance. `InventoryPayloadV1` contains owner, capacity/policy, reservations and the canonical item-membership set. `EquipmentPayloadV1` contains character, slot-policy definition and canonical slot-to-item assignments.

Each item belongs to at most one Inventory membership. Equipment assignment is owned only by Equipment and refers to an eligible item; Item/Character views derive the assignment. Membership and slot collections are sorted by `PersistentId` and stable slot ID respectively. Stack split/merge derives new or tombstoned IDs from the validator-computed command identity and declared operation/spawn slots according to ADR-022; a caller cannot inject a runtime-created ID.

Transfer/equipment validation checks exact Inventory, Equipment and Item revisions, reservations, capacity, stack compatibility, slot policy, skill/capability and immutable physical availability. A successful transaction publishes all affected revisions once; rejection publishes none. Renderer or motor may reject/degrade its presentation/route independently, but cannot silently revert committed assignment. Gameplay rollback is a new command transaction.

### Quest

The Quest envelope binds the exact definition whose versioned schema declares states, stable transition IDs, typed variables, participants, preconditions, effects and completion/failure semantics. `QuestPayloadV1` records current state, typed variables, participants and bounded causal history required for diagnostics and migration.

A transition is idempotent by the validator-computed command identity and expected revisions. Missing definition, illegal transition, stale participant or failed multi-aggregate effect rejects the whole transaction. Script, package or AI may propose a transition only by stable transition ID.

### Dialogue и commitments

The Dialogue envelope binds the exact definition reference and uses its `PersistentId` as session identity. `DialoguePayloadV1` records participants, node/state, stable available choice IDs, pending/committed commitments, transcript policy and bounded causal history. Presentation text, audio timing and provider state are non-authoritative.

A choice or generated candidate may create typed operation proposals. Quest, trade, inventory, relationship or faction statement becomes committed narrative fact only when the corresponding operations commit in one transaction. A rejected proposal produces an explicit rejection/fallback and no speculative commitment.

### Faction, membership и relationships

The Faction envelope binds the exact definition reference. `FactionPayloadV1` contains generic faction state and directed faction-policy references. `FactionMembershipPayloadV1` contains Character/Faction IDs, membership state, rank and policy definition. `RelationshipPayloadV1` contains source/target PersistentIds, versioned fixed-point dimensions, bounds and policy definition.

Character and Faction expose membership only as derived indexes. Symmetry is never implicit: an inverse relationship is a separate aggregate and requires an explicit operation in the same transaction when policy demands it. AI memory may retain recollection but cannot overwrite the current relationship. Bulk reputation expands into bounded typed operations with contiguous slots before plan construction.

### InteractiveObject

The InteractiveObject envelope binds the exact definition reference. `InteractiveObjectPayloadV1` contains explicit state-machine state, capabilities, owner/region references and the committed reservation reference, when present. Physical overlap is an immutable candidate fact only. Use/lock/open/activate/consume validates object state, actor capability, World Services reservation and physical preconditions before plan construction.

## Typed engine-owned operations

An RPG `WorldCommand` payload is `RpgCommandV1`. It contains a bounded sequence of the following closed `RpgOperationV1` variants:

| Variant | Typed semantic payload |
|---|---|
| `AdjustCharacterAttribute` | Character ref, attribute ID, checked fixed-point delta and policy definition |
| `AdjustCharacterResource` | Character ref, resource ID, checked fixed-point delta and policy definition |
| `TransitionCharacterLifeState` | Character ref, stable transition ID and required immutable facts |
| `SetSkillProficiency` | Character ref, skill ID, expected/current/new fixed-point values and progression policy |
| `CreateItemInstance` | Item definition, quantity/custom-state values and causal spawn slot |
| `UpdateItemInstance` | Item ref, checked quantity/durability change and stable custom-state transition ID |
| `DestroyItemInstance` | Item ref, tombstone reason and required source-membership removal operation |
| `TransferItem` | Item, optional source/destination Inventory refs, quantity and capacity/reservation policy; at least one side MUST be present |
| `SplitItemStack` | Source Item/Inventory refs, quantity and causal spawn slot |
| `MergeItemStacks` | Source/target Item and Inventory refs with exact stack-policy definition |
| `AssignEquipment` | Equipment, Item and relevant Inventory refs, stable slot ID and slot policy |
| `TransitionQuest` | Quest ref, stable transition ID, typed variable updates and participant refs |
| `AdvanceDialogue` | Dialogue ref, stable choice/transition ID and typed commitment operation references |
| `ChangeFactionMembership` | Membership, Character and Faction refs, stable membership/rank transition ID |
| `AdjustRelationship` | Relationship ref, dimension ID, checked fixed-point delta and policy definition |
| `TransitionInteractiveObject` | Object ref, actor ref, stable transition ID and reservation/physical fact refs |

Every variant has a common prefix: contiguous `operation_slot: u32`, sorted unique target refs `(aggregate_kind, PersistentId, expected_revision)`, exact definition/policy hashes and its typed payload. Slots MUST equal `0..operation_count-1`. An intra-command reference may point only to an earlier operation or declared causal spawn slot; duplicate, gap, forward or cyclic reference rejects the command. `RpgCommandV1` and `CanonicalCommandBodyV2` MUST NOT contain `command_id`; the authoritative causal command ID is a validator result computed under ADR-022.

Packages use definition IDs and typed values inside these variants; they cannot add a raw aggregate patch or private variant. Adding a variant or payload field is an engine-owned versioned contract change with migration and gate updates. First-party combat, magic, crafting and trade have no alternate Rust/store path.

## Immutable `RpgTransactionPlan`

`RpgTransactionPlan` is the versioned engine-owned `crates/contracts` handoff from RPG Framework validation to Runtime. Only RPG Framework validation may construct it. It is immutable after construction and contains:

```text
RpgTransactionPlan {
  schema = "nextengine.rpg-transaction-plan.v1",
  causal_command_id,
  canonical_command_body_hash,
  project_composition_lock_hash,
  schema_registry_hash,
  definition_policy_hashes[],
  ordered_operations[],
  ordered_read_set[],
  ordered_write_set[],
  ordered_event_drafts[],
  budget_policy_hash,
  plan_hash
}
```

- `causal_command_id` is the validator-computed ADR-022 result, never a caller-supplied payload field.
- `ordered_operations` uses ascending contiguous `operation_slot`.
- `ordered_read_set` and `ordered_write_set` sort by `(aggregate_kind_tag, PersistentId bytes)`. A read entry binds revision and canonical state hash. A write entry binds before/after revision, before/after hash and the complete staged envelope.
- `ordered_event_drafts` contains only events derivable from the complete write set. An event draft is not a published `DomainEvent`.
- `definition_policy_hashes` are unique and byte-sorted. `plan_hash` covers every preceding field in canonical encoding.
- The plan contains no mutable reference, ECS/entity handle, database transaction, task handle, VM object, callback or backend/vendor value.

Validation and publication use exactly this sequence:

1. common command admission validates envelope/schema/size/principal/capability and computes command identity;
2. RPG validation checks operation slots, exact project/definition/policy compatibility and bounded inputs;
3. validator snapshots all target aggregates in read-set order and checks existence, expected revisions, ownership and reservations;
4. aggregate-local state machines, numeric bounds and tombstone rules are checked;
5. cross-aggregate preconditions are evaluated in read-set order against immutable mechanics, physical and World Services facts bound to their revisions/hashes;
6. validator expands every operation into one complete read/write/event set and canonicalizes event order;
7. the complete plan is encoded and hashed, then its replacement envelopes and event drafts are staged copy-on-write in an isolated transaction buffer;
8. at the declared Runtime commit point, Runtime rechecks the bound read revisions/hashes and atomically publishes the whole write set, terminal receipt and committed event batch; any mismatch or failure discards the buffer and publishes no RPG state or event.

No subscriber may re-enter the current transaction. A committed event may cause a future `WorldCommand` proposal only. Crash recovery sees either the previous atomic checkpoint or the fully committed plan; staging is never an authoritative shared store.

## Stable event order

Each operation schema assigns contiguous `event_local_slot: u16` values. Before commit, event drafts are sorted by the exact tuple:

```text
(
  operation_slot,
  event_local_slot,
  event_schema_id_nfc_utf8,
  primary_aggregate_kind_tag,
  primary_persistent_id_bytes
)
```

Duplicate tuple, missing local slot or schema/order mismatch aborts the plan as `RPG_EVENT_ORDER_INVALID`. The zero-based position in this sorted sequence is the canonical ADR-022 event slot used with event schema and full body hash to derive `DomainEventId`. Storage iteration, subscriber registration, worker completion and hash-map order never choose event order.

## Proposal and read-only boundaries

- SPEC-13 mechanics and reducers submit `EffectRequest`, `MechanicDeltaProposal` or `RpgCommandV1` proposals; they cannot write aggregates or construct/modify `RpgTransactionPlan`.
- Luau and Wasm receive capability-filtered immutable views and a bounded command/proposal sink. VM globals and plugin memory are not RPG state.
- `AgentIntent`, planner, dialogue/model output and AI memory are untrusted proposals. Deterministic validators retain final authority.
- Physical Embodiment and World Services expose revision/hash-bound immutable facts and receive future proposals/events; neither stores RPG fields.
- UI/presentation reads immutable semantic projections and emits action-derived command candidates. Localized text, animation, camera and audio cannot pre-commit narrative fact.
- Tools, scenarios and tests use the production `WorldCommand`/`RpgCommandV1` path and read-only probes; a mutable test or inspector backdoor is forbidden.
- First-party code uses the same public operation variants, capability checks, diagnostics and gates as community packages. A private domain service, direct mutable ECS access or shared mutable database is non-conforming.

## Persistence и copy-on-write migrations

`RpgSaveSegmentV1` contains aggregate records sorted by `(aggregate_kind_tag, PersistentId bytes)`, exact definition hashes, domain revisions, causal/idempotency history required by the domain and cross-segment PersistentId references. It excludes `RuntimeEntityId`, localized labels, VM/coroutine state, presentation/audio state, backend objects and World Services-owned calendar/time/population state.

Current engine schema `N` MUST read exact RPG schemas `N`, `N-1` and `N-2`. `RpgMigrationRegistryV1` provides exactly one adjacent pure migration, including an explicit identity migration when representation is unchanged, for every supported aggregate kind and required definition/state transition:

```text
N-2 --migration[N-2,N-1]--> N-1 --migration[N-1,N]--> N
```

Load/migration follows these rules:

1. source bytes and current save generation remain read-only;
2. validator selects one unambiguous adjacent chain from the exact `ProjectCompositionLock`; missing, duplicate or skipped edge fails closed;
3. records are copied into an isolated working generation and transformed by ascending schema step, then `(aggregate_kind_tag, PersistentId bytes)`;
4. each step validates definition/state/transition mappings, fixed-point bounds, references and cross-aggregate invariants before the next step;
5. migration preserves aggregate domain revisions, causal/idempotency history and tombstones and emits no gameplay `DomainEvent`;
6. after all records reach `N`, the complete segment and every cross-segment reference are validated and hashed;
7. only then may Persistence atomically publish a new save generation. Any decode, transform, validation, hash, I/O or crash fault discards the working generation and leaves original bytes/hash and prior published generation unchanged.

Input older than `N-2`, a missing required definition/migration or an invalid record returns `RPG_MIGRATION_REQUIRED` or the more specific diagnostic. Package uninstall/export follows the same copy-on-write rule and cannot silently discard RPG fields or causal history.

## Compositional budget ownership

Command admission, RPG validation, `RpgTransactionPlan` construction, staging and commit are measured only in ADR-016 `core-command-rpg` owner spans. Mechanics, AI and World Services proposal work remains in its own mutually exclusive budget row. No RPG gate redefines the integrated `GameplayBudgetMatrix`, and passing an RPG functional gate does not substitute for `PERF-01`.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `RPG_SCHEMA_UNSUPPORTED` | Reject command/load before snapshot mutation |
| `RPG_AGGREGATE_NOT_FOUND` | Reject command; no substitute by name or runtime ID |
| `RPG_REVISION_STALE` | Reject complete transaction with current revision hint |
| `RPG_REVISION_EXHAUSTED` | Reject complete transaction; do not wrap revision |
| `RPG_OPERATION_ORDER_INVALID` | Reject gap, duplicate, forward/cyclic reference; no plan |
| `RPG_TRANSITION_INVALID` | Reject illegal character/quest/dialogue/faction/object transition; no event |
| `RPG_OWNERSHIP_CONFLICT` | Reject inventory/equipment/trade transaction atomically |
| `RPG_RESERVATION_INVALID` | Reject interaction; retry only as a future command |
| `RPG_DEFINITION_MISMATCH` | Fail command/load before mutation; preserve save/project |
| `RPG_PLAN_STALE` | Discard staged plan after commit-point revision/hash recheck |
| `RPG_EVENT_ORDER_INVALID` | Abort before event/state publication |
| `RPG_TRANSACTION_ABORTED` | Discard staged mutation/event set; committed state unchanged |
| `RPG_MIGRATION_REQUIRED` | Fail closed with exact schema/definition path; preserve original bytes |
| `RPG_COMMITMENT_REJECTED` | Do not present stateful claim as fact; use declared rejection/fallback |
| `NONDETERMINISTIC_RESULT` | Gate fails at first divergent plan/aggregate/event; no retry-to-green |

## Accepted gate descriptors

| Gate | Primary owner | Contributors | Reproducible command/scenario | Pass threshold | Required evidence | Fallback | VS / profile closure |
|---|---|---|---|---|---|---|---|
| `RPG-DOMAIN-P1` | RPG Framework Team | Gameplay Extensibility Team | `next gate RPG-DOMAIN-P1 --scenario rpg-aggregate-corpus --commands 10000 --repeats 100` | All aggregate kinds and operation variants produce exact accept/reject/revision/state/event hashes over 100 repeats; 100% state-machine/bound/revision-overflow cases enforced; public/API/store scan finds 0 hidden first-party path and 0 forbidden public type | command/operation corpus, schema/ownership/API graph, aggregate snapshots, revision/event/state hashes and diagnostics | reject invalid definition/command and pin prior compatible schema; gate remains blocking | VS-03, VS-13 |
| `RPG-TRANSACTION-P1` | RPG Framework Team | Runtime Team | `next gate RPG-TRANSACTION-P1 --scenario rpg-atomic-faults --all-commit-points --roots game,headless,capture-worker` | 0 partial Character/Item/Inventory/Equipment/Quest/Dialogue/Faction/Membership/Relationship/InteractiveObject state, event or receipt publications across stale/conflict/abort/crash faults; exact plan/write/event/state roots and event IDs across roots and worker/order permutations | immutable plan/read/write/event traces, fault/power matrix, receipts, event IDs and state roots | discard staging and retain prior revisions/checkpoint; deterministic mismatch blocks admission | VS-03, VS-06, VS-11, VS-13 |
| `RPG-MIGRATION-P1` | Asset & Persistence Team | RPG Framework Team | `next gate RPG-MIGRATION-P1 --scenario rpg-save-migrations --versions n-2,n-1,n --power-faults all` | Valid `N-2 → N-1 → N`, `N-1 → N` and `N` fixtures reach exact expected hashes; 100% missing/duplicate migration, definition/transition, revision/history, bounds/reference/cross-invariant and power faults fail closed; original hash/generation unchanged on every failure | migration registry/fixtures, before/after segment and definition hashes, revision/causal-history report, cross-reference validation, fault diagnostics and publication audit | use prior compatible engine/package or explicit copy-on-write export; never mutate original save | VS-02, VS-11, VS-13 |

These are `AcceptedBaseline`, `Blocking` gates. Their canonical descriptor source is SPEC-19. Unavailable implementation evidence is non-PASS; no agent, retry or human decision can synthesize gate success.

## Requirements

| Requirement | Нормативное требование | Primary owner | Contributors / required approvers | Blocking gates | Evidence | VS / profile closure |
|---|---|---|---|---|---|---|
| `REQ-095` | Every generic RPG aggregate has one owner, `RpgAggregateEnvelopeV1`, immutable revisioned view and explicit state machine. | RPG Framework Team | Repository Owner | `RPG-DOMAIN-P1` | schema/ownership/API graph, aggregate corpus, revision/state/event hashes | VS-03, VS-13 |
| `REQ-096` | Character/item/inventory/equipment/quest/dialogue/faction/membership/relationship/object changes validate, stage and commit atomically through `WorldCommand` and immutable `RpgTransactionPlan` with canonical event order. | RPG Framework Team | Runtime Team | `RPG-TRANSACTION-P1` | plan/read/write/event traces, fault matrix, receipts and state roots | VS-03, VS-06, VS-11 |
| `REQ-097` | First-party and community mechanics use the same typed RPG operations, capabilities and validators with no hidden operation, store or mutable API. | Gameplay Extensibility Team | RPG Framework Team, Security & Governance Team | `RPG-DOMAIN-P1` | public API/package/store graph, first-party/community operation corpus and denial audit | VS-13 |
| `REQ-098` | RPG save segments preserve exact definitions, domain revisions and causal history and migrate copy-on-write from `N-2`/`N-1` to `N` with fail-closed atomic publication. | Asset & Persistence Team | RPG Framework Team | `RPG-MIGRATION-P1` | migration registry/corpus, before/after hashes, revision/history/cross-reference report and fault audit | VS-02, VS-11, VS-13 |

## Failure paths

| Failure requirement | Failure / trigger | Primary owner | Contributors / required approvers | Нормативный путь / fallback | Blocking gates | Evidence | VS / profile closure |
|---|---|---|---|---|---|---|---|
| `FAIL-035` | Invalid, stale, conflicting or faulted multi-aggregate operation/plan | RPG Framework Team | Runtime Team | Reject or discard the entire staged plan; publish no aggregate revision, DomainEvent or partial receipt result; retain the prior atomic checkpoint. | `RPG-TRANSACTION-P1` | fault/power matrix, plan/receipt/event/state roots and unchanged-state proof | VS-03, VS-06, VS-11, VS-13 |
| `FAIL-036` | Missing/incompatible/ambiguous definition or `N-2 → N` migration, invalid cross-aggregate record or migration publication fault | Asset & Persistence Team | RPG Framework Team | Fail closed before publication, discard the working generation and preserve exact original bytes/hash, prior published generation, revisions and causal history. | `RPG-MIGRATION-P1` | migration negative corpus, original-generation hashes, revision/history report and atomic-publication audit | VS-02, VS-11, VS-13 |
