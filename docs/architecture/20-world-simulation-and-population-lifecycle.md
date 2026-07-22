# SPEC-20: World simulation и population lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-20 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Agent Intelligence Team, RPG Framework Team, Physical Embodiment Team, Verification & Evidence Team, Release Engineering |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-007](adr/007-identities-persistence-and-replay.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md) |
| Заменяет | отсутствует |

## Статус предложения

Этот документ входит в post-1.5 foundation-completeness proposal. Он не изменяет Accepted packet 1.4, remediation candidate 1.5 или SPEC-16/ADR-017. Он задаёт future acceptance contract only after atomic promotion SPEC-17…20 and ADR-018…021. До этого lifecycle state — `AwaitingReview`.

## Назначение и invariants

SPEC-20 определяет deterministic lifecycle durable population across loaded, reduced and unloaded world regions without duplicating RPG, AI, physical or streaming authority.

- `PersistentId` and durable RPG state MUST survive residency/LOD transitions; runtime spawn/despawn MUST NOT mean narrative creation/deletion.
- World simulation tier MAY change computation fidelity/cadence but MUST NOT change mandatory quest, ownership, schedule or causal outcomes.
- Calendar/time advance, schedules, region transfer and spawn/despawn decisions MUST occur at declared deterministic commit points.
- Core residency, RPG aggregates, Agent plans, physical pose and World Services schedule/population state MUST have distinct owners.
- Wall clock, renderer visibility, async I/O completion and unordered worker completion MUST NOT determine authoritative transition.
- Missing chunk/route/capability or budget pressure MUST produce bounded defer/fallback, never teleport, duplicate entity or lost durable state.
- `game` and `headless` MUST produce exact mandatory population/domain outcomes from the same inputs.

## Source of truth и ownership

| State | Единственный owner/source of truth | Reconstructible/non-authority |
|---|---|---|
| Simulation calendar/time and declared time-advance plan | World Services | wall clock, renderer time |
| Population membership, region/home and schedule cursor | World Services `PopulationRecord` | spatial index, nav crowd cache |
| Runtime entity residency/mapping | Core Runtime | diagnostic indexes |
| Character/item/quest/faction/domain state | RPG Framework | population summary fields |
| Active plan/attention/habits | Agent Runtime | abstract schedule intent |
| Physical pose/contact/traversal outcome | Physical Embodiment | abstract region/location |
| WorldChunk content/spawn definitions and publish state | Asset & Persistence | staged I/O/decompression results |
| RoutePlan/reservation/spatial query | SPEC-08 World Services subservices | actual pose/domain ownership |

Population record contains references/projections needed to schedule work but cannot duplicate mutable RPG attributes, inventory, quest state, Agent plan or physical pose. Cross-owner transition uses immutable views and validated proposals/commands.

## Public contracts

### PopulationRecord

`PopulationRecord` contains schema version, subject PersistentId, archetype/definition AssetId + hash, current region/home anchor, `WorldResidencyTier`, schedule ID/cursor, last committed world tick, importance/interaction flags, declared abstract capabilities and revision.

Record MAY cache immutable summary hashes/revisions of owning domains to reject stale work. It MUST NOT contain mutable copies of health, inventory, quest, relationship, motor recurrent state or physics transform.

### ScheduleDefinition

`ScheduleDefinition` is cooked immutable content with schedule ID/version/hash, integer calendar intervals, conditions over documented immutable facts, desired activity/region/resource, priority, maximum deferral, fallback activity and transition policy IDs.

Schedule evaluation uses simulation calendar, subject/region revisions and named deterministic RNG stream only when definition explicitly declares stochastic choice. Equal inputs produce the same proposal. Schedule does not teleport or mutate RPG/physics state; it emits bounded activity/region/reservation/interaction proposals for future commit.

### WorldResidencyTier

World population tiers are distinct from SPEC-05 physical LOD:

| Tier | Required behavior | Forbidden shortcut |
|---|---|---|
| `Active` | resident entity; full due RPG/AI/world work and selected physical LOD | direct schedule write to pose/domain |
| `Simulated` | resident/reduced-cadence deterministic domain/agent/world work; physical tier may be capsule/abstract as allowed | skipping due mandatory transition |
| `Abstract` | non-resident or lightweight record; closed-form/event-step schedule and declared generic outcomes | invented physics/contact or package-specific hidden simulation |
| `Dormant` | no due work until exact wake condition/time; durable record persists | deleting/tombstoning entity on unload |

Tier selection uses declared deterministic inputs: region residency, interaction/quest importance, wake horizon, capability requirements and budget profile. Camera visibility MAY request presentation/streaming interest but cannot alone authorize domain downgrade/upgrade.

### AbstractActivityState

`AbstractActivityState` contains activity ID, start/next-decision ticks, origin/target region, schedule revision, required capability/facts revisions, progress representation, deterministic RNG stream state when used and causal event references. Only fields owned by World Services are stored; any RPG outcome remains a pending proposal until domain commit.

Abstract activity supports only public generic operations declared by engine/packages. If outcome needs contact, path traversal, precise combat, dialogue or interactive-object validation unavailable at the tier, activity defers, upgrades residency or resolves only a content-declared non-physical fallback.

### WorldAdvancePlan

`WorldAdvancePlan` is immutable, bounded work for interval `[from_tick, to_tick]`: ordered due schedule decisions, wake/sleep/tier proposals, region transfers, domain command candidates, reservations and next cursors. It includes input revision hashes and maximum operation count.

Bulk time skip MUST use the same transition definitions/order as stepped simulation. It MAY batch intervals proven free of external decisions, but must stop at earliest boundary that can emit observable/domain result. Plan is validated against current revisions at commit; stale plan is discarded/recomputed, not partially applied.

### RegionTransfer

`RegionTransfer` contains subject, source/target region revisions, logical arrival constraints, schedule/activity cause and required streaming/route/capability facts. Abstract transfer commits logical region only when policy allows. Active physical transfer completes only through traversal/interaction outcomes; region service cannot teleport physics-authoritative avatar.

## Population lifecycle

Normative transition:

`Absent definition → Registered → Dormant/Abstract → Simulated → Active → Quiescing → lower tier or Tombstoned`.

- `Registered` validates unique PersistentId, definition/content hashes and initial domain record before publish.
- Upgrade `Prepare → Validate → Commit → Stabilize` resolves chunks, domain/agent/physical descriptors and reconstructs caches before resident mapping publication.
- Downgrade quiesces ingress, commits/aborts due commands, snapshots each owner segment and proves no forbidden physical/interaction state before despawn.
- Chunk unload cannot force downgrade while required interaction/contact/transaction is unresolved; it defers or keeps required data resident.
- Tombstone is explicit durable domain/world operation. Unload/despawn never implies tombstone.

Transition failure retains source tier/record and emits diagnostic. Repeated failure may pin a safe tier according to profile, but cannot discard due mandatory outcome.

## Time, schedules и offline progression

Simulation time is integer ticks plus calendar mapping from SPEC-08. Schedule cadence and next-decision tick are serialized. Time skip input is a validated production command/policy, not wall-clock elapsed duration.

For each advance boundary:

1. collect due records by `(next_tick, region_id, subject PersistentId, schedule_id)`;
2. read immutable owner views at declared revision;
3. evaluate schedule/activity into bounded proposals;
4. validate reservations/domain/region transitions;
5. commit accepted operations in common runtime order;
6. update World Services cursor only for committed/deferred specified result;
7. emit DomainEvents and next-decision tick.

Offline/abstract progression can change RPG state only through the same commands and package operations available to active simulation. It cannot synthesize quest success, inventory transfer, combat/contact or relationship change without the declared validator/preconditions.

## Streaming, save и replay

WorldChunk spawn records define initial registration, not repeatable spawn each load. Persistent population registry and tombstones decide whether a durable entity already exists. Transient population definition must explicitly declare non-durable lifecycle and cannot be referenced by save/quest/package persistent state.

Save records population records/schedule cursors/abstract activities in World Services segment; RPG/Agent/Motor/Runtime segments retain their own fields. Load validates cross-segment PersistentIds, revisions, content/schedule definitions and tier capability. Missing required definition/state schema fails before world mutation.

Replay records external time-advance commands and accepted WorldCommands; derived schedule/tier events are optional oracle. First divergence reports tick, subject, tier/schedule input revisions and planned/committed operation diff.

## Budgets и concurrency

Work queues are bounded and ordered. Each profile declares maximum due records/operations per tick, cadence per tier and deterministic overflow policy. Overflow may defer only records whose maximum deferral and mandatory outcome constraints allow it. Dropped/overdue work is counted with stable diagnostic; silent omission forbidden.

Worker tasks receive immutable record/view inputs and return revision-bound proposals. Completion enters staging queue; commit order is canonical and independent of worker completion. No mutable ECS/world/RPG access crosses async boundary.

Integrated budget follows SPEC-12 authoritative profile. If ADR-016 later becomes Accepted, this subsystem adopts its one compositional GameplayBudgetMatrix without changing ownership or allowing per-agent multiplication beyond total row budget.

## Stable diagnostics и failure semantics

| Code / failure | Required outcome |
|---|---|
| `POPULATION_RECORD_INVALID` | Reject registration/load; no partial domain/entity creation |
| `POPULATION_ID_COLLISION` | Security/determinism fatal for conflicting provenance; never choose random replacement |
| `POPULATION_TIER_UNAVAILABLE` | Retain/pin source safe tier or declared lower non-lossy fallback |
| `POPULATION_TRANSITION_BLOCKED` | Defer with exact forbidden condition; no partial despawn/upgrade |
| `SCHEDULE_DEFINITION_MISSING` | Fail required load/project; optional subject uses explicit idle fallback only |
| `SCHEDULE_PLAN_STALE` | Discard whole plan and recompute from current revisions |
| `WORLD_ADVANCE_BUDGET_EXCEEDED` | Deterministically defer allowed work and report overdue set; never reorder/drop mandatory work |
| `REGION_TRANSFER_INVALID` | Retain source logical/physical state; request future route/stream validation |
| `WORLD_STATE_SCHEMA_MISMATCH` | Fail load before mutation and preserve original save |
| `NONDETERMINISTIC_RESULT` | Gate fails at first population/schedule/tier divergence; retry cannot turn green |

## Proposed gates

| Gate | Owner | Reproducible command/scenario | Pass threshold | Required evidence | Fallback |
|---|---|---|---|---|---|
| `WORLD-POP-P1` | World Services + RPG Framework | `next gate WORLD-POP-P1 --scenario population-schedules-v1 --records 1000 --ticks 10000` | 1,000 records ×10,000 ticks, 100 repeats: exact schedule/proposal/accepted-command/final domain hashes; 0 owner duplicate writes; mandatory outcome exact across Active/Simulated/Abstract variants | population/schedule manifests, owner views, command/event/state hashes, tier traces | pin safe tier/cadence; block invalid schedule |
| `WORLD-TIME-P1` | World Services + Runtime | `next gate WORLD-TIME-P1 --scenario stepped-vs-bulk-time --seeds 100` | stepped and bounded bulk advance converge to exact mandatory state/event/cursor hashes; 100% stale plans rejected atomically; no wall-clock reads | advance plans, schedule traces, replay hashes, clock/API scan | stepped fixed-tick advance; block time skip |
| `WORLD-RESIDENCY-P1` | Runtime + Asset & Persistence | `next gate WORLD-RESIDENCY-P1 --scenario population-residency-churn --cycles 10000` | 10,000 chunk/tier/load/save cycles: 0 duplicate/lost PersistentIds, dangling durable refs, partial transitions or uncommitted due outcomes; source tier retained on every injected failure | resolver/population/save manifests, transition/fault traces, memory report | pin current tier/chunk; retain prior save/registry |

## Foundation requirement aliases

| Alias | Primary owner | Future acceptance contract |
|---|---|---|
| `FND-WORLD-R1` | World Services | Population, schedule, residency, RPG, AI and physical fields have distinct owners |
| `FND-WORLD-R2` | World Services | Active/Simulated/Abstract/Dormant tiers preserve mandatory domain outcomes |
| `FND-WORLD-R3` | World Services | Stepped and bounded bulk time use one deterministic schedule/command path |
| `FND-WORLD-R4` | Runtime Team | Spawn/despawn/chunk transitions preserve PersistentId, save and replay correctness |
| `FND-WORLD-F1` | World Services | Stale/blocked/budgeted schedule or tier work defers atomically without lost mandatory state |
| `FND-WORLD-F2` | Runtime Team | Residency/load/schema failure retains source tier/registry/save with no partial entity lifecycle |

## Promotion contract

Promotion occurs only with SPEC-17…19 and ADR-018…021 in one reviewed transaction. It updates SPEC-00/01/02/03/05/06/07/08/09/11/12/14/15, glossary and traceability; maps WORLD-POP/TIME/RESIDENCY gates into VS-01/02/04/05/11; and preserves exactly fifteen VS gates.

Foundation alias allocation follows SPEC-17: REQ-087…102 and FAIL-031…038 after accepted SPEC-16, or REQ-079…094 and FAIL-025…032 after its formal rejection/withdrawal. Pending SPEC-16 blocks promotion but not review. No external technology row is added.
