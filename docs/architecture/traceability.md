# Product-goal → check map

| Поле | Значение |
|---|---|
| ID | TRACE-001 |
| Статус | Accepted |
| Версия | 4.3 |
| Последняя проверка | 2026-08-06 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-038](adr/038-versioned-production-worker-handoff-diagnostic.md), [ADR-039](adr/039-tooling-only-process-wide-system-global-allocator-measurement.md), [ADR-040](adr/040-fixed-tls-sharded-global-allocator-measurement.md), [ADR-041](adr/041-owner-thread-quiescent-global-allocator-measurement.md), [ADR-042](adr/042-unobserved-deallocation-system-pass-through.md), [ADR-043](adr/043-codegen-proven-non-reentrant-count-bearing-allocator-callbacks.md), [ADR-044](adr/044-neutral-text-catalog-and-locale-fallback.md), [ADR-045](adr/045-low-overhead-hard-performance-evidence.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md) |
| Заменяет | TRACE-001 4.2 |

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
| Открывать и закрывать production application exactly once | `fast`, `play`, `persistence-replay`; conditional `platform` | `game`, `headless` и runtime-bearing `tools` проходят один session coordinator; platform event admission сохраняет host/capability identity и per-source sequence. В snapshot хранится только последний lifecycle request/event. Close journal проходит `Prepared → SavePublished`: immutable save image повторно публикуется byte-identical после crash, а `SavePublished` завершает close без второй generation | SPEC-01, SPEC-17, SPEC-29, ADR-028, ADR-035, ADR-047 |
| Восстанавливать active live run без ложного replay | `fast`, `play`, `persistence-replay`; conditional `platform` | SessionStore хранит два чередующихся `session.snapshot.v4.bin` и `CURRENT`, без object packs/archive. Durable world публикуется только manual Save и save-on-close; Suspend не сохраняет, crash теряет прогресс после последнего Save. Restart и explicit Load выбирают newest compatible Save, остаются `Suspended`, создают fresh presentation epoch/sequence `0` и ждут явного Resume | SPEC-01, SPEC-03, SPEC-17, SPEC-21, SPEC-29, SPEC-30, ADR-047 |
| Сохранять authoritative parity composition roots | `play`, `persistence-replay` | `game` с null presentation и `headless` выдают одинаковые authoritative state/command-ledger roots; headless host-object count равен нулю, production dependency graph не содержит verification | SPEC-01, SPEC-15, SPEC-17, SPEC-21, SPEC-29 |
| Сохранять authoritative world и воспроизводить bug | `persistence-replay` | `WorldCheckpointV4`/`ReplayManifestV5` плюс отдельный world-streaming owner segment сохраняют inventory membership, equipment, collected state, character health, pending chunk transition, `InputMappingReceiptV2`, ordered `TargetingIntentV1` → `AuthoritativeTargetingQueryV1`, exact `PhysicsQueryBatchV1`/results и query/result/targeting-trace compare hashes вместе с command/event/state/ledger roots; Replay V5 валидирует closure напрямую, а V4 возвращает typed unsupported-version до nested decode | SPEC-02, SPEC-03, SPEC-06, SPEC-14, SPEC-18, SPEC-19, SPEC-21, SPEC-22, SPEC-25, SPEC-26, SPEC-27, ADR-034, ADR-046 |
| Детерминированно загружать и выгружать chunks | `play`, `persistence-replay`; conditional `performance` | worker completion permutations дают один staging/commit hash; duplicate/corrupt required group и injected publication fault сохраняют прежнюю active generation; unload/reload не воскрешает durable RPG state | SPEC-03, SPEC-20, SPEC-23, SPEC-25, ADR-021, ADR-026 |
| Загружать neutral content и public mechanic packages | `content-package` | validate/cook/resolve/atomic publish/production load; first-party interaction, combat, Luau scripted-melee и Wasm Component packages проходят тот же public capability, affordance, effect-request и RPG command path | SPEC-03, SPEC-07, SPEC-13, SPEC-17, SPEC-24 |
| Отображать localized UI text из stable text IDs | `content-package`, `fast` | text catalogs cook/validate как PresentationOnly manifest entries; fallback closure (unique locales, single source-locale root, acyclic terminated chains) fail-closed до publication; deterministic resolution по declared chain с typed args; missing entry/locale/argument даёт `LOCALIZATION_RESOURCE_MISSING` плюс readable placeholder без смены content/command IDs; pseudo-locale — обычный catalog | SPEC-18, SPEC-24, ADR-044 |
| Готовить и отображать minimal neutral render content | `content-package`; conditional `platform`, `performance`; `visual-smoke`; `v1-package` при packaging change | exact mesh/material/texture/profile revisions детерминированно cook/activate в canonical catalog и derived meshlet payloads; `B0ShaderInterfaceV2` принимает position/UV/optional SNORM16 normals и outdoor frame block, а separate checked-in world/sky/UI/shadow SPIR-V suites строят stable indexed-indirect scene с authored/flat-normal fallback, directional/hemisphere/fog и optional stabilized sampled-depth shadow. Missing material и unavailable shadow allocation выбирают declared presentation fallback без смены authority. Displayless BMP/manifest capture остаётся human evidence, не correctness oracle. Copied package `headless --live-ticks 0` проходит production tick-0/checkpoint/close path, copied `game` выполняет bounded interactive frame; их authoritative state/ledger roots и snapshot/host-object expectations совпадают, а disposable sibling smoke state не входит в опубликованный inventory | SPEC-04, SPEC-12, SPEC-17, SPEC-18, SPEC-24, SPEC-29, SPEC-30 |
| Преобразовывать player controls в camera-independent authoritative actions | `play`, `persistence-replay`; conditional `platform` | live `game` и headless scenario пропускают equivalent normalized authoritative controls/actions через один exact ActionMap/InputContext resolver и получают одинаковые canonical actions/ingress assignments; `ReplayManifestV5` связывает V2 mapping receipts и persisted-assignment `ClosestPoint` query facts по exact physics snapshot, а live-only relative mouse, camera/view/depth и renderer cadence не меняют command, state или ledger roots | SPEC-01, SPEC-03, SPEC-05, SPEC-15, SPEC-18, SPEC-22, SPEC-26, SPEC-29, SPEC-30, ADR-034 |
| Безопасно обрабатывать untrusted input | `fast`, `content-package` | malformed bounds/hash/version, Luau sandbox escape, Wasm ambient import/forged handle, forbidden capability, trap и instruction/fuel/allocation/live-memory/host-call/command budget exhaustion отклоняются до mutation; package/plugin state и circuit round-trip exact | SPEC-07, SPEC-10, SPEC-11, SPEC-13, SPEC-24, ADR-014 |
| Диагностировать regression без private test backdoor | relevant check + local scenario | stable diagnostic содержит first divergence; minimized replay сохраняет failure identity | SPEC-09, SPEC-15 |
| Автоматизировать operational output без text oracle | relevant JSON-producing command | stdout декодируется как ровно один versioned typed `RunReportV1`, `DiagnosticReportV1` или command report; progress идёт в stderr, stable code проверяется полем | SPEC-09, SPEC-15, SPEC-29 |
| Сохранять player-facing presentation | `play`; optional capture | exact revision-bound mesh/material references, bounds и typed integer/fixed-point third-person camera входят в complete in-memory 30 Hz fixed-step `PresentationSnapshotV2`/B0 plan; renderer при 30/60/144 Hz cadence повторяет последний snapshot с одинаковыми gameplay roots, а authoritative restart начинает fresh epoch/sequence `0` с previous=current camera cuts. Приватные Vulkan float view-projection/depth и UI/camera/animation/audio/render change не меняют authority | SPEC-04, SPEC-08, SPEC-18, SPEC-28, SPEC-29, SPEC-30, ADR-035 |
| Поддерживать shipping platform, которую реально изменили | conditional `platform` | релевантный smoke запускается на affected Windows/Linux target; public API остаётся engine-owned | SPEC-04, SPEC-17, SPEC-29 |
| Закрыть v1 без ложного platform success | `v1-closure` и `v1-package` внутри `native-gate-run` на каждом target; затем `native-gate-compare` | каждый clean same-commit Windows/Linux run сохраняет свой `PASS` и честный remote-target `NOT_RUN`; только пара native target reports с matching project/content/mechanics/WIT/extension/state/ledger roots даёт `native_gate_ready = true`, не подменяя остальные критерии R1/v1 | SPEC-04, SPEC-07, SPEC-12, SPEC-15, SPEC-17, SPEC-29, ADR-030 |
| Не ухудшать затронутый hot path | conditional `performance` | versioned run сохраняет raw samples, nearest-rank p50/p95/p99, methodology, full fingerprint и exact authoritative roots. `long-session-soak` показывает 3 600-tick history growth и mandatory checkpoint cost при exact driver/application ledger-root parity; renderer-only `interactive-frame-soak` отделён от versioned `production-worker-soak`, который измеряет bounded game queue, `next-simulation`, fixed-step application и shared snapshot handoff с exact counts и zero drop/reorder. ADR-039 фиксирует отдельный tooling-only process/PID/window-scoped `System` allocation counter; ADR-040 сохраняет exact sharded admission для foreign counted calls, ADR-041 после retained enabled `+15.64%` failure задаёт RMW-free const-TLS owner path, а ADR-042 после retained `+5.88%` исключает unobserved `dealloc` из measurement state machine. Candidate-6 сохранил roots/resource/inactive checks, но enabled `+4.81%` снова нарушил `3%`; ADR-043 разрешил удалить per-call recursion flag только на admitted pinned Windows source+IR+ASM/backend proof, implementation это доказательство прошла, но единственный candidate-7 дал inactive `-0.50%` `PASS` и enabled `+4.30%` `FAIL`. Gross successful `alloc`/`alloc_zeroed`/`realloc` traffic не является live/peak RSS, shipping roots его не линкуют, а до новой material hypothesis и candidate `PASS` counter честно `NOT_RUN`. Linux остаётся `NOT_RUN` до `LNX-006` и отдельного Accepted target-extension ADR. Только compatible `release` baseline на полном `ref-win-thoth-v1` даёт hard timing verdict; diagnostics имеют `REPORT_ONLY`, wrong host/workload/preflight — `NOT_RUN`. Representative `r2-alpha-render` загружает `projects/reference-alpha` через production authoring/cook/activation и измеряет exploration, combat и UI/dialogue отдельно в primary/fallback: шесть окон по 600 warm-up + 3 600 measured frames с independent budgets, Vulkan timestamps, V3 logical/process/device evidence и authoritative roots. Его report mode остаётся `REPORT_ONLY`; R2 hard verdict требует clean ten-run THOTH baseline, а R3–R5 workloads до реализации возвращают `NOT_RUN`. Absolute overrun или >=5% regression с 95% interval — `FAIL`; smoke/soak/worker diagnostic и Accepted boundaries не закрывают B-12 | SPEC-04, SPEC-05, SPEC-06, SPEC-08, SPEC-09, SPEC-12, SPEC-16, SPEC-23, SPEC-26, SPEC-27, SPEC-30, ADR-016, ADR-036, ADR-038, ADR-039, ADR-040, ADR-041, ADR-042, ADR-043, ADR-045 |
| Сохранять deterministic fallback для AI/physical policy | `play`, `persistence-replay`; `performance` при изменении hot path | absent/bad optional service или model route приводит к declared local fallback без blocked tick и duplicate command | SPEC-05, SPEC-06, SPEC-14, SPEC-27 |
| Проверять заменяемость grounded-capsule physics backend | `physics-collision --backend compare`, `persistence-replay --backend reference\|physx`, `physics-backend-parity`; conditional `platform`, `performance` | reference и PhysX дают exact canonical pose/contact phase/order/checkpoint/replay result; activation fallback не меняет tick, а runtime failure сохраняет previous checkpoint | SPEC-05, SPEC-21, SPEC-26, ADR-033 |
| Не смешивать Gothic data с engine runtime | `content-package` только при изменении importer/neutral contract | isolated importer output проходит bounds/provenance validation; runtime/packages не содержат legacy/protected bytes | SPEC-10, SPEC-11, SPEC-24 |

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

### Performance V3 evidence clarification

For the hot-path performance goal, ADR-045 replaces only the mandatory active
allocator-counter clause. Current hard evidence is
`PerformanceRunV3`/`PerformanceResourceCountersV3`: canonical logical charges,
Windows peak working set and process I/O, conservative device-allocation
ceiling, Vulkan timestamps, profiler integrity and exact authoritative roots.
`ProcessAllocationCounterV1` is optional; absent evidence is valid, while a
present partial or inconsistent payload is `NOT_RUN`. V2 remains readable only
as historical evidence. Representative workloads, ten-run THOTH baselines and
absolute budgets remain required; no Linux or B-12 closure follows from the
schema change.

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
