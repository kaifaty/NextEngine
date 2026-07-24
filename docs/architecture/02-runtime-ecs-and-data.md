# SPEC-02: Runtime, ECS и модель данных

| Поле | Значение |
|---|---|
| ID | SPEC-02 |
| Статус | Accepted |
| Версия | 1.7 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-01](01-system-architecture.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

Core runtime владеет fixed tick clocks, entity residency, RuntimeEntityId mapping, system schedule, command transaction log, DomainEvent order и snapshot publication. Exact active project configuration приходит только из immutable `ProjectCompositionLock`; World Services владеет calendar/population state, RPG Framework — aggregate state/transaction semantics, Player Experience — action/UI/camera presentation state. Domain component values принадлежат профильному context, а ECS storage — только механизм размещения. ECS implementation не определяет публичную semantics.

## Идентификаторы

| Тип | Representation contract | Lifetime | Разрешённые границы |
|---|---|---|---|
| `RuntimeEntityId` | opaque, generational, process-local | spawn→despawn одного runtime | только Rust runtime facade |
| `PersistentId` | opaque 128-bit, canonical 16 bytes / lowercase hex text | между runs и content revisions согласно migration | saves, chunks, replay, Luau/WIT/AI views |
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
6. deterministic World Services calendar/population/schedule update через revision-bound `WorldAdvancePlan`;
7. deterministic agent planning и PhysicalAvatarIntent generation;
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

`ScenarioAction` входит в runtime только через production `VirtualInputEvent`, WorldCommand/MechanicCommand proposal, lifecycle или declared adapter-fault boundary SPEC-15. Test runner не получает mutable ECS/component store. Fixture state создаётся cooker/load transaction до first tick.

`ProbeSpec` разрешает только documented immutable snapshot/query, ordered DomainEvent/diagnostic и normalized metrics. Probe execution не меняет schedule/access sets, RNG, queue ordering или state hash. Missing selector/revision возвращает stable typed result, не raw pointer/RuntimeEntityId.

## Async I/O и task scheduling

Task получает owned immutable inputs и cancellation token. Completion включает request ID, input content hashes, result hash и typed error. Commit stage MUST отклонить устаревший результат, если precondition revision изменилась. Cancellation/timeout не оставляет partially registered asset/entity.

## ECS facade и Bevy candidate

Facade предоставляет query/system registration, declared read/write sets, lifecycle hooks и snapshot extraction. Она MUST NOT обещать Bevy archetype/layout/event semantics. Bevy ECS/app crates — `Proposed` по [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md); exact version pin не влияет на schemas.

## Failure semantics

- Invalid external command → `CommandReceiptV1` rejection/structured diagnostic без `DomainEvent`; simulation продолжается.
- Internal invariant violation, duplicate PersistentId или transaction rollback failure → simulation instance останавливается до следующего tick; создаётся crash/replay capsule.
- Late async result → `STALE_REVISION`, без side effect.
- Event subscriber overrun → consumer отключается/деградирует по policy; authoritative event order сохраняется.
- State hash divergence в replay → немедленный fail с первым divergent tick и component ownership diff.
- Test-only mutable access attempt или probe side effect → architecture violation; run invalid, changeset blocked.
- Deterministic scenario repeat divergence → `NONDETERMINISTIC_RESULT`; diagnostic replay сохраняется, retry не считается pass.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| ECS-P1 | Bevy PoC из ADR-002 | Все thresholds ADR-002 | manifest/bench/API report | собственная facade implementation |
| RUNTIME-01 | 10 000 ticks × 100 seed fixtures, два повтора | exact command/event order и final state hash на одной target triple | replay report | blocking schedule fix |
| RUNTIME-02 | fuzz commands/schemas/preconditions 24 CPU-hours | 0 panic/UB/partial commit; все rejects stable-coded | fuzz corpus + sanitizer logs | blocking validator fix |
| RUNTIME-03 | async completion permutations, 1 000 runs | final state exact; stale results 100% rejected | permutation report | serialize commit queue |
| RUNTIME-04 | public boundary scan | 0 RuntimeEntityId в serialized/WIT/Luau/IPC schemas; 0 Bevy/vendor public types | schema/API scan | blocking boundary refactor |
| RUNTIME-05 | production vs scenario control-plane scan | 100% scenario mutations проходят VirtualInputEvent/validated command/lifecycle/declared fault adapter; 0 mutable ECS/backend test API; probes preserve exact state hash | API/architecture scan, replay/probe report | remove test backdoor; block merge |
| RUNTIME-06 | V2 body/claim arrival/duplicate/collision/gap/retry/exhaustion/priority/Ingress-Outcome permutations | 100% corpus даёт exact computed IDs/full body hashes, `CommandLedgerV2` root, `CommandStreamLedgerV2` states/reservations/4096-window/receipts, stage trace и state root; no sequence wrap; re-entry всегда `OUTCOME_REENTRY_FORBIDDEN` | versioned command corpus, ledger snapshots/receipt roots, stage traces, state roots | blocking validator/order fix |
| RUNTIME-07 | causal spawn retry/slots/tombstone/provenance collisions | 100% golden IDs exact; collision всегда fatal до alternate ID/mutation | spawn vectors, save/reload ledger, collision corpus | blocking identity/migration fix |
| CANON-01 | JCS/CanonicalBinaryV1/path/domain/Merkle vectors Win/Linux | 100% bytes/hashes exact; invalid Unicode/float/key/path vectors rejected | neutral vectors, raw-byte/hash reports | blocking canonicalization fix |

Runtime Team владеет gates; Release Engineering принимает artifacts.
