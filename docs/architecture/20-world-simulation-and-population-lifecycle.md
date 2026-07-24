# SPEC-20: World simulation и population lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-20 |
| Статус | Accepted |
| Версия | 1.1 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Agent Intelligence Team, RPG Framework Team, Physical Embodiment Team, World Services Team, Verification & Evidence Team, Release Engineering |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-014](adr/014-deterministic-extensions-and-package-trust.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-20 принят в architecture packet 1.7 вместе с foundation contracts SPEC-17…SPEC-19 и ADR-018…ADR-021. Он закрепляет World Services как единственного владельца world calendar, population schedule и logical residency tier, не передавая ему RPG, Agent, Runtime или Physical state. Принятие документа не объявляет `WORLD-POP-P1`, `WORLD-TIME-P1`, `WORLD-RESIDENCY-P1` либо vertical implementation conformance пройденными.

## Назначение и invariants

SPEC-20 определяет deterministic lifecycle durable population across loaded, reduced and unloaded world regions without duplicating RPG, Agent, Runtime, Physical или persistence authority.

- Один `PersistentId` MUST обозначать durable subject во всех residency tiers, saves, replays и owner segments. Tier-local durable identity и замена PersistentId при spawn/despawn запрещены.
- Runtime spawn/despawn MUST NOT означать narrative creation/deletion. Durable registration и tombstone являются отдельными validated operations.
- Tier MAY менять cadence и доступную fidelity, но MUST NOT менять mandatory quest, ownership, schedule или causal outcome.
- `WorldCalendarStateV1`, schedule decisions, logical region transfer и tier decisions MUST изменяться только на declared deterministic commit points.
- Unsupported abstract outcome MUST deterministically request a sufficient tier upgrade or remain deferred. Он MUST NOT fabricate contact, traversal, combat, dialogue, inventory, quest или relationship result.
- Wall clock, renderer visibility/frame rate, async I/O completion и unordered worker completion MUST NOT определять calendar, schedule, tier, transfer или mandatory outcome.
- Missing content, residency prerequisite, capability или budget MUST produce bounded deterministic defer/pin/fail-closed behavior, never teleport, duplicate entity, discard due work or lose durable state.
- `WorldPartitionManifestV1`, durable spatial placement/tombstones,
  `TierExecutionProfile` and streaming admission MUST preserve the same World
  Services ownership and upgrade/defer semantics; content load/unload is not a
  domain outcome.
- `game`, deterministic `headless` и `capture-worker` MUST use the same calendar, schedule, command validation, transition order, save and replay contracts.

## Source of truth и exact ownership

| Mutable state | Единственный owner/source of truth | Allowed foreign projection / forbidden duplicate |
|---|---|---|
| `WorldCalendarStateV1`, time-advance cursor/plan, population membership, `WorldPartitionManifestV1` logical region/home and durable placement/tombstones, schedule cursor, abstract activity, `TierExecutionProfile` and `WorldResidencyTier` | World Services Team | immutable calendar/population/spatial snapshots; no content payload, RPG calendar field or renderer/wall-clock authority |
| Character, item/inventory/equipment, quest/dialogue, faction/relationship, interactive-object and other RPG aggregates | RPG Framework Team | revisioned immutable facts in schedule evaluation; no mutable population summary |
| Goals, plans, attention, habits, memory and pending `AgentIntent` | Agent Intelligence Team | revisioned immutable plan/affordance facts; no schedule-owned Agent plan |
| Fixed `SimulationTick` transaction clock, command/system order, `RuntimeEntityId` and resident PersistentId↔runtime mapping | Runtime Team | World Services tier proposal; no durable population membership in ECS/residency cache |
| Body pose, contacts, constraints, physical traversal outcome, motor route/policy state and physical LOD | Physical Embodiment Team | abstract logical location/capability facts; no population-owned pose or fabricated contact |
| Schema/content/WorldChunk definitions and publish state, owner-segment encoding, save generations and migration orchestration | Asset & Persistence Team | staged immutable inputs; no durable placement/tier/domain ownership over decoded fields |

Runtime `SimulationTick` orders transactions. World Services `world_tick` is the authoritative in-world calendar cursor. The two nominal values MUST NOT be implicitly converted or treated as interchangeable; any project mapping between them is versioned, integer, hash-bound and part of compatibility metadata.

Cross-owner work reads immutable revisioned views and emits proposals or `WorldCommand` candidates. Only each owner writes its fields inside the common atomic transaction. Backend caches, spatial indexes and presentation projections are reconstructible and cannot become a second source of truth.

## World calendar contract

`WorldCalendarStateV1` is an engine-owned public value stored only in the World Services save segment:

```text
WorldCalendarStateV1 {
  1 schema_version: u16 = 1,
  2 world_tick: u64,
  3 calendar_definition_id: AssetId,
  4 calendar_definition_hash: ContentHash,
  5 epoch_world_tick: u64,
  6 epoch_calendar_unit: i64,
  7 calendar_units_per_world_tick_num: u64,
  8 calendar_units_per_world_tick_den: u64,
  9 revision: u64,
  10 last_causal_command_id: Option<CommandId>,
}
```

`epoch_world_tick <= world_tick`; numerator MUST be non-zero. Calendar unit at any admitted tick is computed with checked integer arithmetic:

```text
calendar_unit(t) =
  epoch_calendar_unit
  + floor((t - epoch_world_tick) * calendar_units_per_world_tick_num
          / calendar_units_per_world_tick_den)
```

Overflow, an absent/mismatched definition hash or an invalid denominator fails before mutation. Host locale, time zone, daylight-saving rules and wall time are presentation inputs only and MUST NOT enter this mapping. Calendar revision increments exactly once per committed advance plan; aborted/deferred work cannot advance `world_tick`.

## Population contracts

### PopulationRecord

`PopulationRecord` contains schema version, subject `PersistentId`, archetype/definition `AssetId + ContentHash`, logical region/home anchor resolved in exact `WorldPartitionManifestV1`, `WorldResidencyTier`, `TierExecutionProfile` ID/hash, schedule ID/cursor, last committed world tick, importance/wake facts, declared abstract capabilities and revision.

The record MAY cache immutable owner revision/hash references solely to reject stale work. It MUST NOT contain mutable health, inventory, quest, relationship, Agent plan/memory, motor recurrent state, runtime entity ID or physical transform. The same subject `PersistentId` is retained through every tier transition; a resident `RuntimeEntityId` is ephemeral and never serialized.

### ScheduleDefinition

`ScheduleDefinition` is cooked immutable content with schedule ID/version/hash, integer calendar intervals, conditions over documented immutable facts, desired activity/region/resource, priority, positive retry interval, maximum deferral tick, fallback activity ID and required resolution tier.

Schedule evaluation uses `WorldCalendarStateV1`, subject/region/owner revisions and a named authoritative RNG stream only when the definition explicitly declares stochastic choice. Equal inputs MUST produce the same ordered proposal. A schedule never mutates RPG/Agent/Physical state or teleports a subject; it submits revision-bound proposals for later common validation and commit.

Fallback activity is another declared schedule activity with its own validators and required resolution tier. It is not permission to invent the original outcome.

### WorldResidencyTier

Population residency is ordered `Dormant < Abstract < Simulated < Active` and is orthogonal to physical LOD:

| Tier | Required behavior | Forbidden shortcut |
|---|---|---|
| `Active` | Runtime-resident subject; all due RPG/Agent/world work and selected Physical path run through their owners | schedule writes pose, Agent plan or RPG aggregate |
| `Simulated` | Runtime-resident reduced-cadence deterministic owner work; every due mandatory transition is preserved | skip a due outcome because cadence is lower |
| `Abstract` | Non-resident/lightweight World Services record; only declared generic abstract activities can resolve | invent contact, traversal, combat or package-specific hidden mutation |
| `Dormant` | Durable record persists with no due work before an exact wake condition/world tick | delete/tombstone subject on unload |

Tier selection uses only declared deterministic facts: logical region residency, interaction/quest importance, wake horizon, required resolution/capabilities and hash-bound budget profile. Camera visibility MAY request presentation/streaming interest but cannot authorize a tier or domain change.

Exact cadence, allowed activity classes, required content classes, operation
bound, maximum deferral and fallback tier resolve from immutable
`TierExecutionProfile`. Profile selection is project-locked and cannot depend
on cache warmth, I/O completion, worker count or renderer interest. SPEC-25
classifies each abstract activity as exact, requiring a sufficient tier,
defer-only or presentation-only; no approximate authoritative success class
exists.

### AbstractActivityState и unsupported outcomes

`AbstractActivityState` stores activity ID, start/next-decision/deadline world ticks, origin/target logical region, schedule revision, required owner fact revisions, progress representation, named RNG stream state when used and causal references. It stores only World Services fields; an RPG/Agent/Physical result remains uncommitted until its owner validates the corresponding command.

At every due decision the evaluator MUST follow this order:

1. If the current tier satisfies the declared required resolution and all owner facts are current, evaluate the activity normally.
2. Otherwise, if the deterministic tier policy admits the minimum sufficient tier and its content/capability prerequisites can be staged, emit one revision-bound upgrade proposal and leave the activity outcome/cursor uncommitted.
3. Otherwise, before `maximum_deferral_tick`, emit `DeferredUnsupported` with checked `next_retry_world_tick = current_world_tick + min(retry_interval, maximum_deferral_tick - current_world_tick)`.
4. At the maximum deferral boundary, keep the activity in `DeferredUnsupported`, stop the advance plan before the outcome, retain the exact source tier and emit `WORLD_ABSTRACT_OUTCOME_BLOCKED`.

No branch may read elapsed wall time, substitute a visible nearby entity, choose a random teleport, emit a success event, advance the schedule cursor or mutate an owner aggregate for an unsupported result.

### WorldAdvancePlanV1

`WorldAdvancePlanV1` is immutable bounded work for `[from_world_tick, to_world_tick]`. It includes expected calendar/population/owner revisions, ordered due schedule decisions, wake/tier proposals, logical transfers, domain command candidates, reservations, next cursors, maximum operation count and a canonical plan hash.

The order key is `(due_world_tick, region_id, subject PersistentId, schedule_id, operation_ordinal)`. `from_world_tick` MUST equal current `WorldCalendarStateV1.world_tick`; `to_world_tick` MUST be greater than or equal to it. A stale input, operation-bound overflow or failed command validation rejects the complete plan before commit.

Stepped and bulk time use the same boundary evaluator, operation order and canonical plan partition. A plan closes immediately before the first boundary whose operations would exceed `maximum_operation_count`, or at the requested target when none does; a single boundary larger than the limit rejects without commit. Bulk MAY coalesce only an interval proven to contain no due schedule, wake, reservation expiry, transfer, authoritative RNG draw, external decision or observable/domain result. It MUST stop at the earliest such boundary and produce the same command/event/cursor/state hashes as stepped execution. Each bounded plan commits atomically and increments calendar revision once; a longer request uses the same contiguous plan boundaries in both modes, and the calendar cursor never moves beyond the last committed plan.

### RegionTransfer

`RegionTransfer` contains subject `PersistentId`, expected source/target logical-region and durable-placement revisions from `WorldPartitionManifestV1`, arrival constraints, schedule/activity cause and required content/residency/capability facts. Abstract transfer commits logical region only under declared policy. A physically active transfer completes only from Physical-owned traversal outcome; World Services cannot write or teleport a physical pose.

## Population lifecycle и transition transaction

Normative lifecycle is:

`Absent definition → Registered → Dormant/Abstract → Simulated → Active → Quiescing → lower tier or Tombstoned`.

- `Registered` validates a unique `PersistentId`, definition/provenance hashes and required owner records before publication.
- Upgrade `Prepare → Validate → Commit → Stabilize` stages immutable owner views, validates all expected revisions/capabilities and atomically publishes the World Services tier plus Runtime residency changes at one commit point.
- Downgrade quiesces ingress, resolves/aborts due transactions, validates owner snapshots and proves no forbidden interaction/contact state before runtime despawn.
- Any authoritative failure before commit retains the complete source tier/records/mapping. `Stabilize` may rebuild only non-authoritative caches; its failure cannot alter committed owner state and pins the committed safe tier until rebuild succeeds.
- Chunk unload cannot force downgrade while a required interaction, contact, transaction or mandatory outcome is unresolved; it defers unload or retains required data.
- Tombstone is an explicit durable command. Unload/despawn is never a tombstone and loading a chunk never re-registers an existing durable subject.

## Calendar ownership migration

Legacy saves may contain the calendar mapping in the RPG segment. Migration ID `world-calendar-rpg-to-world-services-v1` is a pure copy transform and MUST run before any world mutation:

1. validate the source `SaveManifest`, segment hashes and recognized source schemas;
2. normalize the legacy RPG calendar into exact `WorldCalendarStateV1` using only persisted integer fields and the pinned calendar definition;
3. stage a World Services segment containing the normalized state and stage an RPG segment with all legacy calendar value fields removed;
4. write a non-authoritative migration receipt containing source/target segment hashes and migration ID, not a duplicate calendar value;
5. validate the complete staged save, cross-segment `PersistentId` closure, schedule cursors and first due boundary;
6. atomically publish a new save generation; only that generation may then be loaded.

The source generation is immutable and remains the recovery generation. No failure may publish one owner segment without the other, mutate the loaded world or infer a value from wall time/default locale.

| Source save state | Required deterministic result | Stable failure |
|---|---|---|
| Valid legacy RPG calendar; World Services calendar absent | Normalize, stage new World Services state, remove legacy RPG values and atomically publish one new generation | none |
| Valid `WorldCalendarStateV1`; legacy RPG calendar absent | Load as current format; no migration or revision change | none |
| Both present and canonical normalized values are exact | Atomically publish cleanup generation with only World Services authority; calendar revision/value unchanged | none |
| Both present but any normalized field differs | Reject before staging publication; preserve source generation | `WORLD_CALENDAR_AUTHORITY_CONFLICT` |
| Both absent | Reject load; never choose tick zero/current wall time | `WORLD_CALENDAR_STATE_MISSING` |
| Claimed World Services state malformed, or legacy schema/definition/mapping unsupported, invalid or overflowing | Reject even if the other copy appears usable; preserve source generation | `WORLD_CALENDAR_MIGRATION_INCOMPATIBLE` |
| Staging write, hash, cross-segment validation or atomic publish fails | Discard staging and retain source generation/world state | `WORLD_CALENDAR_MIGRATION_ABORTED` |

## Save, replay и concurrency

The World Services segment stores `WorldCalendarStateV1`, exact partition
manifest hash, ordered durable placements/tombstones, `PopulationRecord`
values, schedule cursors, abstract activities, tier-profile references and tier
revisions. RPG, Agent, Runtime and Physical segments retain only their owned
fields. Load validates exact schema/content/partition hashes, cross-chunk and
cross-segment `PersistentId` closure, owner revisions, content/schedule
definitions and tier prerequisites before world publication. Ephemeral runtime
mapping, worker queue and cache state are never serialized.

Replay records external calendar-advance commands, accepted `WorldCommand` values and owner events. Derived schedule/tier plans MAY be stored as an oracle. First divergence reports runtime tick, world tick, subject PersistentId, tier/schedule revisions, plan hash and planned/committed operation diff.

All queues are bounded and canonically ordered. Workers receive immutable revisioned inputs and return proposals through deterministic staging; worker/completion order cannot select commit order. No mutable ECS, world, RPG, Agent or Physical access crosses an async boundary.

`GameplayBudgetMatrix` from ADR-016 owns the integrated performance envelope. Due population/world-service work is charged to its existing mutually exclusive owner row; per-subject budgets MUST NOT multiply beyond the integrated row. Overflow may defer only work whose declared maximum deferral and mandatory-outcome rules allow it. Starvation, silent drop and unowned time fail the run.

## Stable diagnostics

| Code / failure | Required outcome |
|---|---|
| `POPULATION_RECORD_INVALID` | Reject registration/load; no partial domain/entity creation |
| `POPULATION_ID_COLLISION` | Fail closed for conflicting provenance; never choose replacement identity |
| `POPULATION_TIER_UNAVAILABLE` | Retain/pin safe tier or enqueue the exact declared upgrade/defer path |
| `POPULATION_TRANSITION_BLOCKED` | Retain source state and report exact failed prerequisite; no partial despawn/upgrade |
| `SCHEDULE_DEFINITION_MISSING` | Fail required load/project; optional subject may use only an explicitly declared idle activity |
| `SCHEDULE_PLAN_STALE` | Discard complete plan and recompute from current immutable revisions |
| `WORLD_ABSTRACT_OUTCOME_BLOCKED` | Keep the unsupported activity deferred and stop before its outcome; retain tier/activity/cursor and never fabricate success |
| `WORLD_ADVANCE_BUDGET_EXCEEDED` | Deterministically defer allowed work and report overdue set; never reorder/drop mandatory work |
| `REGION_TRANSFER_INVALID` | Retain source logical/physical state and wait for valid future facts |
| `WORLD_CALENDAR_AUTHORITY_CONFLICT` | Reject dual unequal owners before load and preserve source save |
| `WORLD_CALENDAR_STATE_MISSING` | Reject load; no default or wall-clock reconstruction |
| `WORLD_CALENDAR_MIGRATION_INCOMPATIBLE` | Reject unsupported/invalid/overflowing legacy or new state before mutation |
| `WORLD_CALENDAR_MIGRATION_ABORTED` | Discard staging; preserve original save generation and unloaded world |
| `WORLD_STATE_SCHEMA_MISMATCH` | Fail load before mutation and preserve original save |
| `WORLD_PARTITION_MISMATCH` | Reject topology/placement/load before publication and preserve the previous manifest/save/active generation |
| `WORLD_STREAM_PLAN_STALE` | Discard complete admission plan; retain prior active/tier/object state and rebuild from current immutable revisions |
| `NONDETERMINISTIC_RESULT` | Fail at first population/schedule/tier divergence; retry cannot turn green |

## Verification gates

| Gate | Primary owner | Contributors | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|---|
| `WORLD-POP-P1` | World Services Team | RPG Framework Team, Agent Intelligence Team | `next gate WORLD-POP-P1 --scenario population-schedules-v1 --records 1000 --ticks 10000` | 1,000 records × 10,000 world ticks × 100 repeats have exact partition/profile/schedule/proposal/accepted-command/final-owner hashes and 0 duplicate owner writes; Active/Simulated/Abstract mandatory outcomes match; every unsupported-resolution fixture upgrades or defers and emits 0 fabricated outcomes | partition/tier-profile/population/schedule manifests, immutable owner views, command/event/state hashes, tier/upgrade/defer traces | pin safe tier/cadence; block invalid schedule/outcome |
| `WORLD-TIME-P1` | World Services Team | Runtime Team, Asset & Persistence Team | `next gate WORLD-TIME-P1 --scenario stepped-vs-bulk-time --seeds 100 --calendar-migration-matrix all` | 100 seeds converge to exact stepped/bulk calendar, command, event, cursor and owner-state hashes; every migration-matrix row returns exact result/code; failed migrations preserve source hash; 100% stale plans reject atomically; 0 authoritative wall-clock reads | advance plans/hashes, schedule traces, migration before/after hashes and fault matrix, replay roots, clock/API scan | stepped bounded advance; block time skip/load |
| `WORLD-RESIDENCY-P1` | Runtime Team | World Services Team, Asset & Persistence Team, Physical Embodiment Team | `next gate WORLD-RESIDENCY-P1 --scenario population-residency-churn --cycles 10000` | 10,000 tier/chunk/load/save cycles retain exactly one PersistentId per subject with exact durable placement/tombstone/cross-chunk closure and 0 duplicate/lost IDs, dangling durable refs, partial transitions or uncommitted mandatory outcomes; every authoritative injected transition failure retains source state | partition/resolver/population/save manifests, placement/tombstone roots, owner revision graph, transition/fault/identity traces, memory report | pin current tier/chunk; retain prior topology/registry/save |

These gates are implementation obligations. Their status remains evidence-driven; architecture acceptance does not create `PASS`.

## Requirements

| ID | Требование | Primary owner | Contributors | Blocking gates |
|---|---|---|---|---|
| REQ-099 | World Services calendar/population/schedule/tier fields, RPG aggregates, Agent state, Runtime clock/residency mapping and Physical state MUST have exactly the distinct owners listed above; every tier and owner segment MUST reference one unchanged subject `PersistentId`. | World Services Team | RPG Framework Team, Agent Intelligence Team, Runtime Team, Physical Embodiment Team | WORLD-POP-P1, WORLD-RESIDENCY-P1 |
| REQ-100 | `Active`, `Simulated`, `Abstract` and `Dormant` with exact `TierExecutionProfile` MUST preserve mandatory domain outcomes; an unsupported resolution MUST deterministically upgrade or defer without fabricated result, cursor advance or wall-time/input-completion authority. | World Services Team | RPG Framework Team, Agent Intelligence Team, Physical Embodiment Team | WORLD-POP-P1, WORLD-RESIDENCY-P1 |
| REQ-101 | `WorldCalendarStateV1`, stepped advance, bounded bulk advance and legacy RPG-calendar migration MUST use one integer, revision-bound schedule/command path and produce exact equivalent committed hashes. | World Services Team | Runtime Team, Asset & Persistence Team | WORLD-TIME-P1 |
| REQ-102 | Runtime spawn/despawn, chunk/resource residency, durable placement/tombstones and tier transitions MUST preserve the subject PersistentId, owner-segment save/replay closure and all-or-nothing authoritative lifecycle publication. | Runtime Team | World Services Team, Asset & Persistence Team, Physical Embodiment Team | WORLD-RESIDENCY-P1 |

## Failure paths

| ID | Trigger | Required result | Primary owner | Contributors | Blocking gates |
|---|---|---|---|---|---|
| FAIL-037 | Stale, unsupported, blocked, budgeted or overdue schedule/time/tier work | Reject the stale plan or retain exact activity/tier/cursor and deterministically upgrade/defer; no lost mandatory state, fabricated outcome, wall-time decision or partial commit. | World Services Team | Runtime Team, RPG Framework Team, Agent Intelligence Team | WORLD-POP-P1, WORLD-TIME-P1, WORLD-RESIDENCY-P1 |
| FAIL-038 | Calendar migration conflict/fault, residency/load/schema failure, PersistentId conflict or interrupted lifecycle transition | Fail before world/owner publication, preserve source tier/registry/save generation and return the exact stable diagnostic; no partial entity lifecycle or second calendar owner. | Runtime Team | World Services Team, Asset & Persistence Team, RPG Framework Team, Physical Embodiment Team | WORLD-TIME-P1, WORLD-RESIDENCY-P1 |

## Technology neutrality

This contract selects no navigation implementation, physics implementation, ECS, database, crowd system or other backend technology. Public values remain engine-owned and versioned; any replaceable implementation is an adapter/cache behind these ownership, determinism, migration and gate contracts.
