# Product-goal → check map

| Поле | Значение |
|---|---|
| ID | TRACE-001 |
| Статус | Accepted |
| Версия | 3.3 |
| Последняя проверка | 2026-07-30 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |
| Заменяет | TRACE-001 3.2 |

## Назначение

Эта карта помогает выбрать небольшой набор проверок по наблюдаемой цели
продукта. Она не пытается перечислить каждое предложение всех SPEC и не
создаёт отдельную систему допуска изменений.

Canonical check names и правила conditional запуска определены в
[SPEC-12](12-vertical-slice-conformance.md).

## Карта

| Product goal | Основные checks | Наблюдаемый сигнал | Relevant specs |
|---|---|---|---|
| Быстро получать полезную обратную связь | `fast` | workspace checks и focused tests завершаются с ясной причиной failure | SPEC-09, SPEC-15 |
| Играть в малый RPG loop полностью offline | `play` | boot, movement, exact-query pickup, equip, package melee игрока и canonical NPC planner command, interaction, bounded NPC/quest outcome и canonical two-chunk transition с возвратом работают без network/`ai-host` | SPEC-00, SPEC-01, SPEC-05, SPEC-06, SPEC-13, SPEC-14, SPEC-18, SPEC-19, SPEC-20, SPEC-25, SPEC-26, SPEC-27 |
| Открывать и закрывать production application exactly once | `fast`, `play`, `persistence-replay`; conditional `platform` | `game`, `headless` и runtime-bearing `tools` проходят один session coordinator; first-seen interactive event совпадает с current registered host/capability binding и непрерывной per-source sequence. Exact duplicate схлопывается, collision/gap и illegal combined lifecycle batch fail closed при whole-batch preflight до source cursor, tick и session mutation; stale close отклоняется до forced checkpoint. Legal suspend/resume/close edges публикуются атомарно вместе с full bounded request/event archive, exact archived retry возвращает прежний event/progress/receipt, а archive exhaustion и publication fault сохраняют prior in-memory/durable generation | SPEC-01, SPEC-17, SPEC-29, ADR-028, ADR-035 |
| Восстанавливать active live run без ложного replay | `fast`, `play`, `persistence-replay`; conditional `platform` | каждый committed 30 Hz tick даёт complete in-memory live/presentation snapshot; same-session durable checkpoint публикуется на tick `0`, затем каждые `30` ticks и forced на suspend/close. Recovery manifest связывает и typed-декодирует ровно шесть payloads: Runtime, RPG, physics, world streaming, input resolver и presentation/camera; exact bytes, active runtime revision, presentation target и roots проверяются до restore. Crash восстанавливает newest complete checkpoint с rollback не более `29` ticks, без wall-time catch-up, и публикует fresh presentation epoch/sequence `0` с camera cuts; current+previous raw generations плюс ordered chain до `64` recovery evidence entries с exact prior durable snapshots и lifecycle/close/live object closures сохраняют bounded read-only history, malformed/tampered/overflow closure fail closed | SPEC-01, SPEC-17, SPEC-21, SPEC-29, SPEC-30, ADR-035 |
| Сохранять authoritative parity composition roots | `play`, `persistence-replay` | `game` с null presentation и `headless` выдают одинаковые authoritative state/command-ledger roots; headless host-object count равен нулю, production dependency graph не содержит verification | SPEC-01, SPEC-15, SPEC-17, SPEC-21, SPEC-29 |
| Сохранять authoritative world и воспроизводить bug | `persistence-replay` | `WorldCheckpointV4`/`ReplayManifestV5` плюс отдельный world-streaming owner segment сохраняют inventory membership, equipment, collected state, character health, pending chunk transition, `InputMappingReceiptV2`, ordered `TargetingIntentV1` → `AuthoritativeTargetingQueryV1`, exact `PhysicsQueryBatchV1`/results и query/result/targeting-trace compare hashes вместе с command/event/state/ledger roots; V4 остаётся legacy exact-only runner по ADR-034, corrupt или bootstrap RPG V3 input отклоняется до partial activation | SPEC-02, SPEC-03, SPEC-06, SPEC-14, SPEC-18, SPEC-19, SPEC-21, SPEC-22, SPEC-25, SPEC-26, SPEC-27, ADR-034 |
| Детерминированно загружать и выгружать chunks | `play`, `persistence-replay`; conditional `performance` | worker completion permutations дают один staging/commit hash; duplicate/corrupt required group и injected publication fault сохраняют прежнюю active generation; unload/reload не воскрешает durable RPG state | SPEC-03, SPEC-20, SPEC-23, SPEC-25, ADR-021, ADR-026 |
| Загружать neutral content и public mechanic packages | `content-package` | validate/cook/resolve/atomic publish/production load; first-party interaction, combat, Luau scripted-melee и Wasm Component packages проходят тот же public capability, affordance, effect-request и RPG command path | SPEC-03, SPEC-07, SPEC-13, SPEC-17, SPEC-24 |
| Готовить и отображать minimal neutral render content | `content-package`; conditional `platform`, `performance`; `v1-package` при packaging change | exact mesh/material/texture/profile revisions детерминированно cook/activate в canonical catalog и derived meshlet payloads; B0 строит стабильный CPU visible list и conventional indexed-indirect draws из checked-in offline SPIR-V, missing material выбирает declared fallback. Copied package `headless --live-ticks 0` проходит production tick-0/checkpoint/close path, copied `game` выполняет bounded interactive frame; их authoritative state/ledger roots и snapshot/host-object expectations совпадают, а disposable sibling smoke state не входит в опубликованный inventory | SPEC-04, SPEC-12, SPEC-17, SPEC-24, SPEC-29, SPEC-30 |
| Преобразовывать player controls в camera-independent authoritative actions | `play`, `persistence-replay`; conditional `platform` | live `game` и headless scenario пропускают equivalent normalized authoritative controls/actions через один exact ActionMap/InputContext resolver и получают одинаковые canonical actions/ingress assignments; `ReplayManifestV5` связывает V2 mapping receipts и persisted-assignment `ClosestPoint` query facts по exact physics snapshot, а live-only relative mouse, camera/view/depth и renderer cadence не меняют command, state или ledger roots | SPEC-01, SPEC-03, SPEC-05, SPEC-15, SPEC-18, SPEC-22, SPEC-26, SPEC-29, SPEC-30, ADR-034 |
| Безопасно обрабатывать untrusted input | `fast`, `content-package` | malformed bounds/hash/version, Luau sandbox escape, Wasm ambient import/forged handle, forbidden capability, trap и instruction/fuel/allocation/live-memory/host-call/command budget exhaustion отклоняются до mutation; package/plugin state и circuit round-trip exact | SPEC-07, SPEC-10, SPEC-11, SPEC-13, SPEC-24, ADR-014 |
| Диагностировать regression без private test backdoor | relevant check + local scenario | stable diagnostic содержит first divergence; minimized replay сохраняет failure identity | SPEC-09, SPEC-15 |
| Автоматизировать operational output без text oracle | relevant JSON-producing command | stdout декодируется как ровно один versioned typed `RunReportV1`, `DiagnosticReportV1` или command report; progress идёт в stderr, stable code проверяется полем | SPEC-09, SPEC-15, SPEC-29 |
| Сохранять player-facing presentation | `play`; optional capture | exact revision-bound mesh/material references, bounds и typed integer/fixed-point third-person camera входят в complete in-memory 30 Hz fixed-step `PresentationSnapshotV2`/B0 plan; renderer при 30/60/144 Hz cadence повторяет последний snapshot с одинаковыми gameplay roots, а authoritative restart начинает fresh epoch/sequence `0` с previous=current camera cuts. Приватные Vulkan float view-projection/depth и UI/camera/animation/audio/render change не меняют authority | SPEC-04, SPEC-08, SPEC-18, SPEC-28, SPEC-29, SPEC-30, ADR-035 |
| Поддерживать shipping platform, которую реально изменили | conditional `platform` | релевантный smoke запускается на affected Windows/Linux target; public API остаётся engine-owned | SPEC-04, SPEC-17, SPEC-29 |
| Закрыть v1 без ложного platform success | `v1-closure` и `v1-package` внутри `native-gate-run` на каждом target; затем `native-gate-compare` | каждый clean same-commit Windows/Linux run сохраняет свой `PASS` и честный remote-target `NOT_RUN`; только пара native target reports с matching project/content/mechanics/WIT/extension/state/ledger roots даёт `native_gate_ready = true`, не подменяя остальные критерии R1/v1 | SPEC-04, SPEC-07, SPEC-12, SPEC-15, SPEC-17, SPEC-29, ADR-030 |
| Не ухудшать затронутый hot path | conditional `performance` | declared numeric scenario остаётся в допустимом regression threshold без изменения gameplay result | SPEC-05, SPEC-06, SPEC-08, SPEC-16, SPEC-23, SPEC-26, SPEC-27, SPEC-30 |
| Сохранять deterministic fallback для AI/physical policy | `play`, `persistence-replay`; `performance` при изменении hot path | absent/bad optional service или model route приводит к declared local fallback без blocked tick и duplicate command | SPEC-05, SPEC-06, SPEC-14, SPEC-27 |
| Проверять заменяемость grounded-capsule physics backend | `physics-collision --backend compare`, `persistence-replay --backend reference\|physx`, `physics-backend-parity`; conditional `platform`, `performance` | reference и PhysX дают exact canonical pose/contact phase/order/checkpoint/replay result; activation fallback не меняет tick, а runtime failure сохраняет previous checkpoint | SPEC-05, SPEC-21, SPEC-26, ADR-033 |
| Не смешивать Gothic data с engine runtime | `content-package` только при изменении importer/neutral contract | isolated importer output проходит bounds/provenance validation; runtime/packages не содержат legacy/protected bytes | SPEC-10, SPEC-11, SPEC-24 |
| Развивать живые quest opportunities без сетевой зависимости | `play`, `persistence-replay` | NPC intent/world need становится Quest только после admission; direct/solicited/contextual/public disclosure, activation, autonomous outcome и exact recorded candidate/template fallback проходят fixed decision boundary, общий RPG command path и replay | SPEC-31 |
| Дать конфликтующим богам независимо реагировать на игрока | `play`, `persistence-replay`, `content-package` | один committed act может улучшить qualitative standing у одного бога и ухудшить у другого; все боги читают один pre-decision snapshot, spillover затрагивает только eligible target, offers/covenants используют явные transitions, cross-context result атомарен, а replay не вызывает модель | SPEC-31, ADR-031 |

## Как выбирать checks

1. Назвать изменённое product behavior и relevant subsystem.
2. Найти соответствующую строку карты.
3. Запустить объединение основных checks.
4. Добавить `platform` или `performance` только при выполнении их trigger из
   SPEC-12.
5. Для нового failure path добавить negative scenario в тот же check.
6. В handoff записать `Pass`, `Fail` и `NotRun(reason)` вместе с остающимся
   продуктовым риском.

Если изменение не попадает ни в одну строку, сначала формулируется новый
наблюдаемый product goal и малый check. Добавлять глобальную бюрократическую
матрицу для этого не требуется.

## REQ/FAIL identifiers

Существующие `REQ-*` и `FAIL-*` identifiers MAY оставаться в subsystem SPEC
как editorial anchors и удобные search labels. Они:

- не образуют глобальный реестр;
- не требуют reserved ranges или непрерывной нумерации;
- не определяют набор ProductCheck;
- не являются proof completeness, release status или product quality;
- MAY быть удалены или переименованы при обычном редактировании relevant
  документа, если ссылки рядом обновлены.

Новая архитектурная формулировка SHOULD предпочитать прямую ссылку на section,
public contract и observable ProductCheck, а не выделять новый глобальный
номер.
