# SPEC-19: Current RPG domain state

| Поле | Значение |
|---|---|
| ID | SPEC-19 |
| Статус | Accepted |
| Версия | 2.0 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-19 version 1.4 speculative migration/narrative/divine feature clauses |

## Authority

RPG Framework is the sole owner of durable character, item, inventory,
equipment, quest, dialogue, faction, membership, relationship and interactive
object values. Physics, mechanics, AI, scripts and presentation consume
immutable revision-bound views and submit proposals; they do not mutate RPG
storage.

Every accepted change enters as a `WorldCommand`, is validated into one
immutable `RpgTransactionPlanV1`, and atomically publishes its complete write
set, receipt and ordered `DomainEvent` batch. A failed or stale plan publishes
nothing. First-party and Luau/Wasm packages use the same operations and
capability checks.

## Current aggregate format

Current durable records use `RpgAggregateEnvelopeV1`:

```text
RpgAggregateEnvelopeV1 {
  aggregate_kind,
  persistent_id,
  schema_version,
  revision,
  definition_ref,
  provenance,
  payload,
  payload_hash
}
```

`PersistentId` is durable; `RuntimeEntityId`, ECS rows, names and storage
positions are not. Definition-backed records bind exact `AssetId +
ContentHash`. Revisions use optimistic concurrency and increment exactly once
per changed aggregate in a committed transaction. Overflow rejects the entire
transaction.

The current closed payload enum contains `Character`, `Item`, `Inventory`,
`Equipment`, `Quest`, `Dialogue`, `Faction`, `FactionMembership`,
`Relationship`, `DivineStanding` and `InteractiveObject` records. This list is
the current serialized envelope surface, not a promise that every high-level
feature is implemented. In particular, the small `DivineStandingPayloadV1`
record does not imply the narrative director, pantheon, offers/covenants or
atomic divine-judgment feature set removed from current obligations by
ADR-046.

Current payload ownership is simple:

- Character owns bounded resources, skills and inventory/equipment references.
- Item owns quantity, durability and bounded custom bytes.
- Inventory owns capacity, membership and reservations; Equipment alone owns
  slot assignment.
- Quest owns its current stable state ID; Dialogue owns participants/current
  node.
- Faction, membership and directed relationship records own their declared
  values.
- InteractiveObject owns its stable state ID and optional linked item.
- Calendar/population, physical pose, mechanics reducer state, AI memory,
  localized text and presentation are not copied into RPG authority.

All collections and payloads are bounded, canonical and validated before
hashing/publication. Public views contain no mutable reference, ECS/backend/VM
handle, database row, task/future or OS object.

## Current operations

`RpgCommandV1` contains a bounded contiguous sequence of exactly these current
`RpgOperationPayloadV1` variants:

1. `AdvanceDialogue`;
2. `TransitionQuest`;
3. `AdjustRelationship`;
4. `SetSkillProficiency`;
5. `TransferItem`;
6. `AssignEquipment`;
7. `TransitionInteractiveObject`;
8. `AdjustCharacterResource`.

Each operation binds stable target IDs and exact expected revisions. Operation
slots are `0..n-1`; target sets and definition/policy hashes are sorted unique.
Unknown variants, missing/extra targets, zero/invalid transfer, invalid skill
range, self-targeted/zero resource adjustment or stale revision rejects the
complete command.

Packages cannot submit raw field patches, construct a trusted plan or add a
private operation variant. New operations require a production gameplay
consumer and contract/check update under ADR-046.

## Transaction and event order

RPG validation snapshots the canonical read set, checks definitions, policies,
revisions and immutable physical/world facts, computes complete replacement
envelopes and event drafts, then hashes `RpgTransactionPlanV1`. Runtime rechecks
the read set at the fixed commit point and publishes all or none.

Read/write sets sort by aggregate kind and `PersistentId`. Event drafts use
operation slot plus their closed event-local order; container iteration,
subscriber order, worker completion and hash-map order cannot choose event IDs
or state roots. Subscribers cannot re-enter the current transaction; an event
may only cause a future command proposal.

## Persistence and compatibility

`RpgSnapshotV2` stores canonical current aggregate envelopes and participates
in `RuntimeSnapshotV3`/`WorldCheckpointV4`, Save and Replay closure. Load
validates schema version, definitions, bounds, hashes, revisions and references
before replacing world state.

Pre-v1 RPG formats are current-only. Older or unknown schema returns
`RPG_SCHEMA_UNSUPPORTED` and preserves source bytes/current world. There is no
`RpgMigrationRegistryV1`, N-1/N-2 reader requirement or current copy-on-write
migration ProductCheck. A migration contract is added only after the first
publicly supported v1 format has a real successor.

## Diagnostics and checks

Stable failures include schema unsupported, aggregate missing, revision stale/
exhausted, operation order invalid, transition invalid, ownership conflict,
definition mismatch, plan stale, event order invalid and transaction aborted.
Every failure retains the previous state/ledger roots.

`play` exercises the implemented dialogue, quest, relationship, inventory,
equipment, interactive-object and character-resource operations through the
reference loop. `persistence-replay` proves current snapshot/save/replay roots,
typed rejection of retired formats and no partial mutation. Focused RPG tests
cover all eight operations, canonical order, stale/conflict and atomic-fault
cases.

Future autonomous quest/narrative/divine behavior is Proposed in SPEC-31 and
is not a prerequisite, current check or accepted feature contract here.
