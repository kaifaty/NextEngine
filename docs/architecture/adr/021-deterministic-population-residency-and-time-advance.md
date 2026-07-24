# ADR-021: Deterministic population residency и time advance

| Поле | Значение |
|---|---|
| ID | ADR-021 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | World Services Team |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Agent Intelligence Team, RPG Framework Team, Physical Embodiment Team, Verification & Evidence Team, Release Engineering |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR-021 принят в architecture packet 1.7 как upstream decision для World Services population/calendar contract. Он переносит единственную authoritative simulation-calendar state из legacy RPG save layout в World Services, фиксирует tier/schedule/time-advance semantics и сохраняет distinct RPG, Agent, Runtime and Physical owners. Принятие решения не означает implementation completion, gate `PASS` или vertical conformance.

## Контекст

Accepted architecture already defines Runtime fixed ticks/residency, WorldChunk streaming, RPG aggregates, Agent fallback, Physical LOD and basic calendar/weather/reservation services. Она не задавала durable population record, единый контракт stepped/bulk world-time advance или fail-closed перенос calendar state, исторически записанной внутри RPG save segment.

Без решения implementation может:

- держать calendar одновременно в RPG и World Services;
- менять schedule outcome от wall time, visibility, I/O или worker order;
- считать unload/despawn удалением durable subject и выдать ему новый ID при загрузке;
- копировать RPG/Agent/Physical fields в population record;
- разрешать abstract combat/contact/traversal без соответствующей owner validation;
- применять time skip отдельным mutation path, расходящимся со stepped simulation.

## Решение

### Authority и owner separation

1. World Services Team является единственным владельцем `WorldCalendarStateV1`, durable population membership, logical region/home, schedule cursor, abstract activity, logical residency tier и time-advance plan.
2. RPG Framework Team остаётся единственным владельцем Character, item/inventory/equipment, quest/dialogue, faction/relationship, interactive-object and other domain aggregates.
3. Agent Intelligence Team остаётся единственным владельцем plans, goals, attention, habits, memory и pending intent.
4. Runtime Team владеет fixed `SimulationTick`, command/system ordering, `RuntimeEntityId` и resident PersistentId↔runtime mapping.
5. Physical Embodiment Team владеет body pose, contacts, constraints, traversal outcome, motor state и physical LOD.
6. Asset & Persistence Team владеет save generation/migration transaction и segment encoding, но не semantic values внутри owner segments.

Один и тот же subject `PersistentId` MUST проходить через `Dormant`, `Abstract`, `Simulated` и `Active`; ephemeral `RuntimeEntityId` возникает только у resident representation и никогда не заменяет durable identity. Population records могут хранить revision/hash references на immutable owner views, но не mutable копии owner state.

World Services tier и Physical LOD являются разными state machines. Calendar/tier decision не пишет pose, contact, RPG aggregate, Agent plan или runtime mapping напрямую; он создаёт revision-bound proposal/command для соответствующего owner и общего commit point.

### WorldCalendarStateV1

`WorldCalendarStateV1` является engine-owned authoritative public value:

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

`epoch_world_tick <= world_tick`, numerator is non-zero, denominator is non-zero, and all arithmetic is checked. Calendar mapping is exact:

```text
calendar_unit(t) =
  epoch_calendar_unit
  + floor((t - epoch_world_tick) * calendar_units_per_world_tick_num
          / calendar_units_per_world_tick_den)
```

World Services `world_tick` is in-world calendar authority. Runtime `SimulationTick` remains the transaction/scheduler clock. Они nominally distinct; implicit conversion is forbidden. Any mapping between them is versioned, integer, hash-bound project compatibility data.

Host time zone, locale, daylight-saving rule, process uptime, renderer time and wall clock are non-authoritative. An absent persisted state cannot be reconstructed as tick zero or “now”.

### Atomic RPG-save → World-Services migration

Migration ID is `world-calendar-rpg-to-world-services-v1`. It is a pure ordered transform on an immutable source generation and executes before any world mutation:

1. validate source `SaveManifest`, segment hashes and recognized RPG/World Services schemas;
2. decode the legacy RPG calendar, normalize every persisted field to exact `WorldCalendarStateV1` and validate calendar definition/hash, integer bounds and schedule cursors;
3. stage a World Services segment with the normalized state;
4. stage an RPG segment with all legacy calendar value fields removed;
5. add a non-authoritative migration receipt containing migration ID and before/after segment hashes, never a duplicate calendar value;
6. validate the complete staged generation, cross-segment PersistentId closure and first due world-time boundary;
7. atomically publish one new generation, then and only then permit world load.

The source save remains immutable recovery input. No failure may publish only one staged segment, mutate the loaded world, advance either clock or choose a calendar value from wall time/defaults.

| Source save state | Deterministic action | Required result / diagnostic |
|---|---|---|
| Valid legacy RPG calendar; no World Services calendar | Normalize, move to World Services, remove RPG values and atomically publish | one new valid generation; no diagnostic |
| Valid `WorldCalendarStateV1`; no legacy RPG calendar | No transform | load exact state without revision/value change |
| Both states present and canonical normalized values are exact | Publish cleanup generation containing only World Services authority | exact value/revision retained; no diagnostic |
| Both states present and any normalized field differs | Reject before publication | `WORLD_CALENDAR_AUTHORITY_CONFLICT`; source unchanged |
| Both states absent | Reject; do not create a default | `WORLD_CALENDAR_STATE_MISSING`; source unchanged |
| Claimed World Services state malformed, or either schema/definition/mapping unsupported, invalid or overflowing | Reject even when the other representation appears usable | `WORLD_CALENDAR_MIGRATION_INCOMPATIBLE`; source unchanged |
| Staging write/hash, cross-segment validation or atomic publish fault | Discard complete staging generation | `WORLD_CALENDAR_MIGRATION_ABORTED`; source and unloaded world unchanged |

This matrix is exhaustive. No best-effort preference, “newest timestamp”, partial field merge or automatic fallback from a malformed claimed authoritative segment is allowed.

### Population tiers и lifecycle

Population tiers are ordered `Dormant < Abstract < Simulated < Active`:

| Tier | Contract |
|---|---|
| `Dormant` | Durable record persists; no work is due before an exact wake condition/world tick. |
| `Abstract` | Non-resident/lightweight record evaluates only declared generic abstract activities. |
| `Simulated` | Resident reduced-cadence owner work preserves every mandatory transition. |
| `Active` | Resident full due owner work; Physical subsystem separately chooses a valid physical LOD. |

Tier selection uses declared region residency, interaction/quest importance, wake horizon, required resolution/capabilities and hash-bound budget profile. It does not use camera visibility, renderer FPS, wall time, worker identity or I/O completion order.

Runtime spawn/despawn changes only resident representation. World Services registration/tombstone changes durable population membership and requires a validated production command. Chunk load cannot re-register an existing subject; unload cannot tombstone it.

Tier/region transition is `Prepare → Validate → Commit → Stabilize`:

- `Prepare` captures immutable expected revisions and stages required content/owner views.
- `Validate` resolves all capabilities, no-contact/no-interaction conditions and complete owner mutation set.
- `Commit` atomically publishes World Services tier/logical location and any Runtime residency change through the common command transaction.
- `Stabilize` reconstructs non-authoritative caches only. Cache failure pins the committed safe tier and cannot rewrite owner state.

Any authoritative failure before `Commit` retains the complete source tier, owner records and Runtime mapping. No partial despawn, duplicate PersistentId, lost mandatory activity or physics-authoritative teleport is permitted.

### Schedule evaluation и unsupported abstract outcome

Each immutable schedule activity declares:

- integer due intervals and exact condition facts;
- desired activity/logical region/resource;
- required resolution tier and owner fact revisions;
- positive retry interval and maximum deferral world tick;
- optional separately validated fallback activity;
- named authoritative RNG stream only when stochastic choice is explicit.

Due work is ordered by `(due_world_tick, region_id, subject PersistentId, schedule_id, operation_ordinal)`. Evaluation reads only `WorldCalendarStateV1` and immutable revisioned facts.

When the current tier cannot validate an outcome:

1. if deterministic tier policy and prerequisites admit the declared minimum sufficient tier, emit exactly one upgrade proposal and retain outcome/cursor;
2. otherwise, before maximum deferral, emit `DeferredUnsupported` with checked `next_retry_world_tick = current_world_tick + min(retry_interval, maximum_deferral_tick - current_world_tick)`;
3. at maximum deferral, keep the activity in `DeferredUnsupported`, stop the advance plan before the outcome and emit `WORLD_ABSTRACT_OUTCOME_BLOCKED`.

Unsupported work never emits success, advances its schedule cursor, mutates an RPG/Agent/Physical owner, synthesizes contact/traversal/combat/dialogue/inventory/quest/relationship result or reads wall time. A fallback activity is evaluated as its own activity and is not fabricated completion of the blocked one.

### Stepped и bounded bulk time

Every external time advance is a validated production command. World Services constructs one immutable bounded plan for `[from_world_tick, to_world_tick]` with expected owner revisions, canonical ordered operations, maximum operation count and plan hash. `from_world_tick` must equal the current calendar state.

Stepped and bulk modes call the same boundary evaluator, canonical plan partition and command validators. A plan closes immediately before the first boundary whose operations would exceed `maximum_operation_count`, or at the requested target when none does; a single boundary larger than the limit rejects without commit. Bulk mode MAY coalesce only a span proven to contain no schedule/wake/reservation expiry/transfer, authoritative RNG draw, external decision or observable/domain result. It stops at the earliest such boundary. The modes MUST produce exact equal calendar revision, command, event, owner-state and schedule-cursor hashes.

Each plan validates all expected revisions and operation bounds before atomic commit and increments calendar revision exactly once. Stale/invalid plan is discarded whole. A long request uses the same contiguous plan boundaries in both modes; `world_tick` advances only to the last committed plan, so a later failure has explicit deterministic partial progress rather than an unrecorded skip.

Async workers receive immutable inputs and return revision-bound proposals through a canonically ordered staging queue. Worker completion order cannot choose due work, tier, operation order or commit result.

### Budget behavior

Population/world-service work participates in the single ADR-016 `GameplayBudgetMatrix`; no per-NPC or per-tier budget can multiply beyond its mutually exclusive integrated owner row. Overflow may defer only work whose positive retry interval, maximum deferral and mandatory-outcome contract permit it. Starvation, silent drop, reordering and unowned span fail the run.

## Рассмотренные варианты

- Calendar remains in RPG save — `Rejected`: schedule/time authority would be stored under the wrong owner and encourage cross-domain writes.
- Calendar is duplicated in RPG and World Services — `Rejected`: creates two mutable authorities and ambiguous migration/replay.
- Wall-clock offline progression — `Rejected`: machine-, locale- and downtime-dependent and not replayable.
- Complete Character/Agent/Physical state copied into population service — `Rejected`: violates single-owner state.
- Despawn/unload means delete, then respawn from chunk — `Rejected`: breaks PersistentId, save references and causal history.
- Visibility or distance alone selects authoritative tier — `Rejected`: presentation state would change mandatory gameplay.
- Bulk time uses bespoke domain mutation — `Rejected`: diverges from ordinary validators/commands.
- Unsupported abstract action guesses a success/failure — `Rejected`: fabricates owner state without required evidence.
- Best-effort stale-plan or mixed calendar migration — `Rejected`: admits partial or ambiguous causal history.

## Последствия

- World Services gains explicit calendar, population, schedule, activity, tier and time-plan schemas.
- RPG saves lose legacy calendar value fields; a hash-bound atomic migration preserves the original generation on every failure.
- Runtime resident mapping, Physical LOD, RPG aggregates and Agent plans remain separately owned and joined only by immutable views plus validated transactions.
- Abstract simulation stays intentionally bounded. Missing resolution can reduce performance through upgrade or a defer that blocks further advance, but cannot silently reduce correctness.
- Save/replay includes additional World Services revisions and plan diagnostics sufficient to identify the first divergent world tick/subject/operation.
- This decision chooses no navigation, physics, ECS, crowd, database or other backend technology.

## Implementation gates и fallback

Conforming implementation MUST pass all three exact gates:

| Gate | Blocking decision contract | Deterministic fallback |
|---|---|---|
| `WORLD-POP-P1` | Ownership, tier equivalence, one PersistentId and upgrade/defer-without-fabrication corpus | retain/pin safe tier or reject invalid schedule |
| `WORLD-TIME-P1` | Stepped/bulk exact equivalence, exhaustive calendar migration matrix, stale-plan atomic rejection and zero wall-clock authority | use bounded stepped advance or block incompatible load/time skip |
| `WORLD-RESIDENCY-P1` | Durable identity and all-or-nothing lifecycle across tier/chunk/save/load fault permutations | retain source tier, Runtime mapping, registry and prior save |

`NONDETERMINISTIC_RESULT` is a failure and retry cannot turn it green. Architecture acceptance does not create gate evidence or `PASS`.

## Supersession

Moving calendar/population ownership into RPG, Agent, Runtime residency cache or Physical state; changing one-PersistentId semantics; permitting wall-clock or visibility authority; fabricating unsupported abstract outcomes; allowing destructive unload/respawn; or introducing non-atomic legacy calendar migration requires a new ADR that explicitly supersedes ADR-021 and synchronizes affected SPEC, traceability and evidence contracts.
