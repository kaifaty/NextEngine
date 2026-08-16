# SPEC-02: Runtime, ECS и модель данных

| Поле | Значение |
|---|---|
| ID | SPEC-02 |
| Статус | Accepted |
| Версия | 2.1 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md) |
| Заменяет | SPEC-02 2.0; admits the R4c AgentPlanning consumer and paired Agent/Memory publication without adding a stage |

## Source of truth и ownership

Core runtime владеет fixed tick clocks, entity residency, RuntimeEntityId
mapping, system schedule, command transaction log, DomainEvent order и snapshot
publication. Exact active project configuration приходит только из immutable
`ProjectLockV3`; RPG Framework владеет aggregate state/transaction semantics,
Player Experience — action/UI/camera presentation state. World Services owns
the current derived calendar, bounded routine projection, durable
`PopulationTierV1`, logical placement and graph-navigation service from
SPEC-20/ADR-072; they use existing stages 6/9. Agent Runtime and Memory Service
own the paired cognition snapshots from SPEC-32/ADR-073; cognition consumes the
existing AgentPlanning stage 7 and internal Outcome stage 9, without adding a
Runtime stage.
Domain component values принадлежат профильному context,
а ECS storage — только механизм размещения. ECS implementation не определяет
публичную semantics.

## Идентификаторы

| Тип | Representation contract | Lifetime | Разрешённые границы |
|---|---|---|---|
| `RuntimeEntityId` | opaque, generational, process-local | spawn→despawn одного runtime | только Rust runtime facade |
| `PersistentId` | opaque 128-bit, canonical 16 bytes / lowercase hex text | между runs внутри поддерживаемого format/project closure | saves, chunks, replay, Luau/WIT/AI views |
| `AssetId` | opaque 128-bit logical identifier | между recook; revision через manifest | authoring/cooked/runtime public contracts |

Неявные conversions запрещены. Resolver MUST обнаруживать absent, unloaded, tombstoned и duplicate PersistentId как разные outcomes. Imported IDs используют namespace derived из importer schema ID + source logical identity, а не путь пользователя.

Runtime-created durable object получает `PersistentId` только из `world_namespace`, causal computed `command_id`, canonical `spawn_slot` и versioned `record_kind` по ADR-022. Exact retry идемпотентно возвращает тот же ID; tombstone запрещает reuse, а совпадение с иной provenance завершает instance как `PERSISTENT_ID_COLLISION`. Explicit ID разрешён только validated cooker/import/load/migration boundary и недоступен script/plugin/AI/scenario input.

## Runtime data flow

`NormalizedControlEvent → PlayerActionFrame / async proposals → command candidates → capability/schema/domain validation → canonical WorldCommand transaction (включая internal RpgTransactionPlan) → DomainEvent → physics/world outcomes → state hash/save delta → PresentationSnapshot`. Единственные mutable edges находятся внутри owned fixed-tick stages; обратный поток от UI/camera/event/presentation consumer возможен только новым candidate command на будущий tick.

## Simulation time domains

| Domain | v1 policy | Может менять authoritative state |
|---|---|---|
| Input sampling | bounded normalized controls → deterministic current/next tick `PlayerActionFrame` | нет |
| Gameplay fixed tick | default 30 Hz, project MAY choose 20/60 before save/replay creation | да, через schedule |
| Physics substep | default 120 Hz, project profile MAY choose 60/240 до создания save/replay; integer multiple gameplay tick | physical state only |
| Motor inference | default 60 Hz, declared integer divisor physics rate; MotorAction удерживается PD/safety layer между inference ticks | MotorAction only |
| Presentation | variable rate + interpolation | нет |
| Async/tool/AI | deadline-based, nondeterministic completion | нет; только proposal queue |

Tick rates и divisors MUST входить в project/save/replay manifests. Runtime не допускает fractional drift: clocks используют integer ticks/rational conversion.

## Нормативный system order

Для каждого gameplay tick порядок MUST быть:

1. ingest immutable `PlayerActionFrame`, other closed input records и completed async proposals по exact current/next cutoff;
2. authenticate/capability check command candidates;
3. validate schema, target, preconditions и RPG rules; multi-aggregate operation строит immutable revision-bound `RpgTransactionPlan`;
4. пересчитать V2 body hash/command ID, выполнить ADR-022 `CommandLedgerV2` admission/deduplication через соответствующий `CommandStreamLedgerV2` и sort `Ingress` commands по `(target_tick, phase, priority_class, issuer_tag, issuer_payload_bytes, sequence, command_id)`;
5. атомарно apply `Ingress` transaction и emit ordered DomainEvent; RPG plan либо полностью коммитит stable mutation/event order, либо не меняет state;
6. применить current deterministic world/streaming commitments and derive the bounded World Services routine/population proposals for the existing stage-9 Outcome batch;
7. build the revision-bound cognition view, run deterministic Utility + bounded GOAP for a due subject and stage only its typed Agent/Memory decision proposal; other deterministic agent planning and PhysicalAvatarIntent generation retain this stage;
8. physics/motor substeps, contact normalization и physical outcomes;
9. сформировать один закрытый `Outcome` batch внутренних commands, пропустить его через тот же validator/order/transaction и запретить same-tick re-entry;
10. despawn/spawn commit и PersistentId resolver update;
11. save/replay delta + state hash;
12. immutable PresentationSnapshot publication.

Systems внутри stage MAY работать параллельно только если access declaration доказывает отсутствие conflicting writes. Schedule graph и tie-break rules MUST быть сериализованы в build metadata.

## Public contracts

### `WorldCommandEnvelopeV2`

`WorldCommandEnvelopeV2` MUST содержать один `CanonicalCommandBodyV2`, optional `claimed_command_id` и только versioned transport-integrity metadata. Body содержит schema/kind, closed tagged `IssuerPrincipalV2`, bound `CommandStreamId`, monotonic per-stream `sequence`, `target_tick`, `phase`, optional target PersistentId, deterministic payload, declared capabilities и preconditions. Body MUST NOT содержать `command_id`, `claimed_command_id`, transport/session framing, arrival/batch/process/worker ordinal, wall-clock timestamp, validator result или validator-owned `priority_class`.

После schema/capability normalization validator кодирует только body по `CanonicalBinaryV1`, вычисляет full 256-bit `body_hash`, затем domain-separated 128-bit `command_id`. Optional claim MUST совпасть с computed ID до ledger reservation. `priority_class` назначается versioned command-kind registry и не выбирается input. Validator возвращает `Accepted{canonical_body, computed_command_id, body_hash}` либо stable rejection code; исключения/строки backend не являются contract.

Command применён атомарно: либо все owned component mutations и events committed, либо ни одно. Command handler MUST NOT выполнять blocking I/O, LLM call или asset load.

World-level `CommandLedgerV2` является authoritative state, содержит один `CommandStreamLedgerV2` на stream и входит в snapshot/save/replay. Каждый per-stream ledger хранит admission high-watermark, закрывающий все меньшие gaps без reuse; exact stream state `Open|CollisionLocked|Exhausted|Closed`; ordered pending reservations; total finalized count; rolling receipt-chain root; и ровно последние `min(finalized_receipt_count, 4096)` `CommandReceiptV1`. Каждая reservation и каждый ordinary receipt сохраняют полный `body_hash`. После 4096 finalizations окно содержит ровно 4096 receipts. Exact retry retained reservation/receipt возвращает прежний result без новых events/mutation; unknown sequence не выше high-watermark после eviction получает `COMMAND_SEQUENCE_FINALIZED`; conflicting body переводит stream в `CollisionLocked`, а `u64` sequence overflow — в `Exhausted`, без wrap или alternate ID. Arrival order, transport batch, process/worker order и wall clock не могут менять admission или commit result. `Outcome` может быть создан только authenticated `InternalSystem`; proposal из `Outcome` переносится минимум на следующий tick, а повторный stage-9 entry даёт `OUTCOME_REENTRY_FORBIDDEN`.

### DomainEvent

Содержит `event_id`, simulation tick, causal command ID, event schema/version и domain payload. Event immutable. Event subscriber может создать candidate command на будущий tick, но не re-enter текущую transaction.

### PresentationSnapshot

Содержит monotonically increasing sequence, source ticks, transforms/poses, visible RPG/UI projections, audio emitters и interpolation metadata. Snapshot является immutable value; slow consumer пропускает промежуточные snapshots, не блокируя simulation.

### Scenario driver и probes

`ScenarioAction` входит в runtime только через production `VirtualInputEvent`, WorldCommand/MechanicCommand proposal, lifecycle или declared adapter-fault boundary. Test runner не получает mutable ECS/component store. Fixture state создаётся cooker/load transaction до first tick.

`ProbeSpec` разрешает только documented immutable snapshot/query, ordered DomainEvent/diagnostic и normalized metrics. Probe execution не меняет schedule/access sets, RNG, queue ordering или state hash. Missing selector/revision возвращает stable typed result, не raw pointer/RuntimeEntityId.

## Async I/O и task scheduling

Task получает owned immutable inputs и cancellation token. Completion включает request ID, input content hashes, result hash и typed error. Commit stage MUST отклонить устаревший результат, если precondition revision изменилась. Cancellation/timeout не оставляет partially registered asset/entity.

## ECS facade и Bevy candidate

Facade предоставляет query/system registration, declared read/write sets, lifecycle hooks и snapshot extraction. Она MUST NOT обещать Bevy archetype/layout/event semantics. Bevy ECS/app crates — `Proposed` по [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md); exact version pin не влияет на schemas.

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| `COMMAND_REJECTED` | Invalid external command | Return a rejected `CommandReceiptV1` and structured diagnostic without `DomainEvent`; continue simulation. |
| `RUNTIME_INTERNAL_INVARIANT` | Internal invariant violation, duplicate `PersistentId` or transaction rollback failure | Stop the simulation instance before the next tick and create a crash/replay capsule. |
| `STALE_REVISION` | Late async result | Reject without side effects. |
| `EVENT_CONSUMER_OVERRUN` | Event subscriber exceeds its bounded policy | Disconnect or degrade that consumer while preserving authoritative event order. |
| `NONDETERMINISTIC_RESULT` | Replay state-root divergence or deterministic scenario-repeat divergence | Stop at the first divergent tick, report the owned state difference and preserve the diagnostic replay. |
| `SCENARIO_MUTATION_FORBIDDEN` | Test-only mutable access or a probe side effect | Reject the operation and invalidate the execution without mutating authoritative state. |

## Product checks

| ID | Scenario / command | Expected behavior | Fallback |
|---|---|---|---|
| ECS-P1 | Bevy PoC из ADR-002 | Все technical thresholds ADR-002 выполнены без утечки backend semantics через facade. | Use the engine-owned facade implementation. |
| RUNTIME-01 | 10 000 ticks × 100 seed fixtures, два повтора | Exact command/event order и final state hash на одной target triple. | Reject the schedule/profile revision. |
| RUNTIME-02 | Fuzz commands, schemas and preconditions for 24 CPU-hours | 0 panic, UB or partial commit; every rejection is stable-coded. | Reject malformed input before mutation. |
| RUNTIME-03 | 1 000 async-completion permutations | Final state remains exact and every stale result is rejected. | Serialize completion commit through the same deterministic queue. |
| RUNTIME-04 | Public-boundary scan | No `RuntimeEntityId` in serialized/WIT/Luau/IPC schemas and no Bevy/vendor public types. | Keep the backend behind the runtime facade. |
| RUNTIME-05 | Production and scenario control-plane comparison | Every scenario mutation uses `VirtualInputEvent`, validated command, lifecycle or declared fault adapter; probes preserve the state hash. | Remove the mutable test path. |
| RUNTIME-06 | V2 body/claim duplicate, collision, gap, retry, exhaustion, priority and Ingress/Outcome permutations | Exact IDs, full body hashes, ledger state, receipts, stage trace and state root; no sequence wrap; re-entry returns `OUTCOME_REENTRY_FORBIDDEN`. | Reject the incompatible validator/order implementation. |
| RUNTIME-07 | Causal spawn retry, slots, tombstones and provenance collisions | Golden IDs remain exact; collision stops before alternate ID or mutation. | Retain the prior identity state and reject the spawn. |
| RUNTIME-08 | R4c stage-7 proposal, stage-9 command/event and stale joint-owner publication faults | Cognition runs only on authored boundaries, publishes paired Agent/Memory revisions through the ordinary Outcome barrier and never re-enters the tick or partially commits another owner. | Abort the complete prepared generation and retain every prior owner. |
| CANON-01 | JCS/CanonicalBinaryV1/path/domain/Merkle vectors on Windows and Linux | Bytes and hashes are exact; invalid Unicode, float, key and path vectors reject. | Reject noncanonical input. |
