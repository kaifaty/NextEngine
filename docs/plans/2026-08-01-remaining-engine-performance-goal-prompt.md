# /goal: оставшиеся доработки производительности Next Engine

Работай автономно в `D:\Projects\NextEngine` и доведи до конца ближайший
evidence-driven performance increment движка. Не останавливайся после анализа:
профилируй production paths, реализуй только подтверждённые улучшения, проверяй
детерминизм и продолжай до терминального условия ниже.

Используй skills `qmd`, `using-rust-engineering` (лист
`performance-and-profiling`), `using-rust-workspaces` и
`using-determinism-and-replay` (листы `cost-of-determinism`,
`snapshot-strategy`, `canonical-state-encoding-for-replay`). Соблюдай корневой
`AGENTS.md`.

## Цель

Закрыть оставшиеся локально исполнимые performance gaps после commit
`61c493b` и получить ещё одно существенное ускорение реальных production paths
без изменения authoritative результата:

1. измерить настоящий main/render → simulation-worker → presentation handoff;
2. добавить точные process-wide allocator counters в performance report;
3. по profiler evidence оптимизировать обычный live tick, checkpoint
   materialization/durable publication и production worker/render path;
4. сохранить exact game/headless/replay roots, bounded recovery и offline
   correctness;
5. оставить R2–R5 hard workloads, ten-run calibration и codegen promotion
   честно отложенными до выполнения их documented prerequisites.

Каждая завершённая фаза MUST заканчиваться отдельным Git-коммитом. После
успешного коммита сразу переходи к следующей фазе; не жди ручного подтверждения.

## Исходная точка

- Branch при создании плана: `codex/architecture-foundation-promotion`.
- Baseline HEAD: `61c493b` (`perf: scale live runtime history`).
- Worktree в этой точке был чистым; branch опережал remote на девять commits.
- Уже реализованы: prepared tick без полного driver fork, transaction deltas
  для ledger/archive/identity history, packed session objects по ADR-037,
  shared immutable snapshot paths, incremental checkpoint validation, Vulkan
  frame slots/cache, reference-physics shared geometry, opt-in Thin LTO/PGO
  workflow, lazy public `TickReport`, chunked COW receipt window и monotonic
  retry/collision fast path.
- Последний exact authoritative root во всех сравниваемых soak runs:
  `5e45825e1a627113902640184c00bf448964bb2b8daf10d6e0947d40fd5a6e17`.
- Последние median run-p95 после `61c493b`: driver window `3.427 s`, driver
  prepare `1 960 us`, driver commit `330 us`, checkpoint materialization
  `41 445 us`, ordinary application tick `2 278 us`, application window
  `5.117 s`, checkpoint tick примерно `89 453 us`, application checkpoint
  window примерно `3.201 s`.
- Identity-root probe и checkpoint tails остаются noise-sensitive.
- Все эти измерения — `REPORT_ONLY`: preflight не достиг одновременно idle CPU
  `<5%` и `20 GiB` free RAM. Они не являются ten-run hard calibration и не
  закрывают B-12.
- Известные измерительные пробелы: allocator fields в
  `PerformanceResourceCountersV1` остаются `None`; static
  `interactive-frame-soak` не измеряет production simulation worker handoff.

Перед началом проверь фактический `HEAD`, branch и `git status`. Не считай этот
snapshot безусловно актуальным: если репозиторий уже продвинулся, используй
текущий код и заново установи baseline. Не переключай branch и не push без
прямого запроса пользователя.

## Нормативный контекст

До редактирования выполни structured QMD query в collection
`nextengine-docs` с явными `intent`, `lex` и `vec`, затем полностью прочитай
кандидаты. Минимум:

- `docs/architecture/README.md`;
- `00-product-contract.md`, `01-system-architecture.md`, `glossary.md`;
- ADR-030, ADR-036 и ADR-037;
- для runtime/ledger: SPEC-21 и ADR-022;
- для persistence/checkpoints: SPEC-03, SPEC-29 и ADR-035;
- для jobs/resources: SPEC-23 и ADR-026;
- для worker/presentation/render: SPEC-30, ADR-028 и ADR-035;
- для tooling/checks: SPEC-09, SPEC-12 и SPEC-15;
- актуальный `docs/roadmap.md` и `docs/development/performance-codegen.md`.

QMD index на момент создания этого промта отставал: он не видел актуальные
ADR-036/037 в README и обрывал новый хвост roadmap. Если это повторяется, не
обновляй индекс без запроса; используй QMD как discovery, затем `rg` и полные
direct reads текущих файлов, явно записав fallback в handoff.

## Неподвижные границы

- Не меняй command/event/state/ledger/archive/replay roots ради скорости.
- Gameplay mutation остаётся только через validated `WorldCommand` и atomic
  production transaction; никаких test-only mutation backdoors.
- `game` и `headless` сохраняют одинаковые validation, scheduling,
  persistence и replay semantics.
- Async/worker work получает immutable revision-bound inputs, возвращает
  bounded result через fixed commit point и не держит mutable authority через
  `await`/frame boundary.
- Presentation, renderer cadence, wall clock, profiler и allocator counters не
  входят в gameplay authority.
- Не ослабляй tick `0`/каждые `30` ticks/forced checkpoint cadence,
  current+previous complete-generation retention, rollback `<=29` ticks,
  fail-closed load и durable close semantics без нового Accepted ADR.
- Не меняй canonical encoding/hash projection. Merkle/incremental hash,
  authoritative delta checkpoint или новый snapshot format — semantic change:
  сначала новый superseding ADR, затем SPEC/schema/migration/traceability и
  equivalence coverage в том же коммите.
- Private caches допустимы только как reconstructible state. Cache hit/miss не
  может менять accepted result; restart/corruption tests обязаны обходить cache.
- Workspace `unsafe_code = "forbid"` сохраняется. Новый unsafe/FFI boundary
  запрещён без отдельного Accepted ADR и узкой reviewed crate boundary.
- Не добавляй global allocator replacement, `target-cpu=native`, nightly SIMD,
  BOLT или default LTO/PGO. ADR-036 прямо не разрешает их promotion без полной
  representative evidence.
- Не добавляй vendor types в public contracts и не создавай второй source of
  truth.
- Не коммить `target/`, profiles, captures, generated reports, caches,
  datasets, checkpoints, credentials или protected/imported assets.
- Не расширяй scope до Semantic UI, audio, multi-region world, 100 NPC или
  16-avatar motor stack только ради benchmark fixture.

## Performance decision protocol

Для каждой оптимизации используй один цикл:

1. На чистом phase-start commit собери release baseline и profiler/allocation/
   I/O attribution на production path. Малый synthetic microbenchmark допустим
   только как дополнительная локализация, не как product verdict.
2. Сформулируй один hotspot и одну проверяемую гипотезу. Не объединяй
   независимые оптимизации в один changeset.
3. Реализуй минимальную bounded правку с positive/failure/equivalence tests.
4. Сравни минимум три независимых before и три after release
   `REPORT_ONLY` run одинаковой методикой. Все raw samples сохраняются; outlier
   removal и retry-to-green запрещены.
5. Принимай алгоритмическую оптимизацию, если targeted median run-p95 улучшился
   минимум на `5%`, либо exact allocation/I/O work уменьшился минимум на `10%`
   при timing в noise band `<2%`; ни одна другая critical median p95 metric не
   должна ухудшиться на `>=5%`.
6. Exact authoritative roots, driver/application ledger/archive parity,
   profiler on/off parity и save/replay equivalence обязательны независимо от
   timing.
7. Identity-root probe с тремя samples на run сам по себе не является
   достаточным основанием. Оптимизируй его только если sampling/trace evidence
   показывает material contribution к полному checkpoint/window.
8. Если candidate не проходит правило, аккуратно убери только собственные
   незакоммиченные изменения через `apply_patch`; не используй destructive
   reset/checkout. Запиши вывод и выбери следующий hotspot.

Dirty pre-commit runs остаются только discovery evidence. После commit запусти
clean confirmation на exact commit. Hard baseline/gate допускается только по
ADR-036: clean `release`, полный THOTH fingerprint/preflight/counters и real
representative workload.

## Git protocol — обязателен после каждой фазы

В начале фазы сохрани `git status --short` и не трогай чужие изменения. Если
dirty files пересекаются с фазой и нельзя безопасно разделить edits — остановись
и запроси пользователя.

В конце каждой успешно завершённой фазы:

1. выполни focused tests и обязательные ProductCheck этой фазы;
2. выполни `cargo run -p xtask -- host-check`;
3. выполни `git diff --check` и просмотри полный diff;
4. stage только явный список файлов через `git add -- <paths>`; не используй
   `git add .`;
5. проверь `git diff --cached --check` и `git diff --cached`;
6. создай ровно один atomic commit с сообщением, указанным в фазе или более
   точным conventional equivalent;
7. сообщи hash коммита и результаты checks, затем продолжай.

Не используй `--no-verify`, `commit --amend`, rebase, reset, force push или
смешивание unrelated user changes. Пустой/skipped/deferred investigation не
является завершённой фазой и не получает пустой commit. Если clean
post-commit confirmation обнаружил реальную регрессию, не переписывай history:
сделай отдельный corrective/revert commit после соответствующих checks.

## Фаза 0 — безопасный bootstrap плана

1. Проверь branch/HEAD/status, последние performance commits и актуальность
   чисел выше.
2. Если единственный untracked/modified файл — этот goal prompt, проверь его и
   закоммить отдельно как `docs(perf): record remaining optimization goal`.
3. Если prompt уже находится в HEAD, не создавай пустой commit и начинай фазу 1.
4. Любые другие pre-existing changes принадлежат пользователю: не stage их.

Commit checkpoint: отдельный docs commit только если prompt ещё не committed.

## Фаза 1 — production simulation-worker diagnostic

Расширь report-only performance tooling так, чтобы оно измеряло не static
render input, а реальный `next_game` path из `apps/game/src/main.rs`:

`main/render callback → bounded sync queue → next-simulation worker → fixed-step
application advance → shared PresentationSnapshot publication → render read`.

Минимальные bounded raw metrics:

- main-thread queue send/wait;
- message queue age на worker dequeue и queue high-watermark;
- worker fixed-step advance, отдельно ordinary и durable-checkpoint frames;
- snapshot publication lock/wait;
- main-thread snapshot read/wait;
- snapshot sequence lag/freshness в rendered frames;
- exact counts submitted/processed/fixed steps/snapshot publications, без
  dropped or reordered messages.

Используй preallocated bounded buffers или fixed counters. `Instant` и thread
timing остаются operational metadata и не участвуют в scheduling. Diagnostic
должен запускать production composition root либо production-owned reusable
path; `crates/verification` не становится production dependency. Не дублируй
в verification фальшивый worker. Измерь минимум `240` FIFO frames, не
подставляй single-threaded `FixedStepLiveSchedulerV1` за реальный worker и не
оставляй `unowned_spans = 0` hardcoded без наблюдения внутренних worker/
checkpoint phases.

Если exact worker нельзя переиспользовать без копирования, сначала извлеки его
из `apps/game/src/main.rs` в private production module за той же application
границей; app остаётся composition root, public gameplay contract не
расширяется. Version/methodology/scenario hash нового diagnostic обязан явно
измениться; не переопределяй старый report молча.

Сохрани static `interactive-frame-soak` как renderer-only diagnostic либо
явно версионируй его methodology. Новый/расширенный worker workload остаётся
`REPORT_ONLY` и не называется R2 workload. Добавь FIFO, queue saturation,
cadence `30/60/144 Hz`, checkpoint, close retry и exact-root parity tests.

Focused checks минимум: `next_game`, application, desktop adapter, performance
tooling tests; `play`; `persistence-replay`; Windows `platform` если доступен;
оба report-only frame/worker scenarios; host-check.

Commit checkpoint: `perf(game): measure production worker handoff`.

## Фаза 2 — точные allocator counters

Закрой единственный missing required process counter из ADR-036:
`allocator_allocated_bytes` и `allocator_allocation_count` должны быть точными
scenario-scoped delta counters вокруг измеряемого workload на поддержанном
performance root. End-of-run `WorkingSet64` не является allocator counter или
peak RSS и не может подменить эти поля.

Требования:

- после фазы 1 проверь, какой процесс реально исполняет каждый workload;
  counter не должен считать только wrapper, если горячая работа живёт в child
  process;
- instrumentation должна быть process-wide, bounded, thread-safe и не
  аллоцировать/логировать из allocation callback;
- предпочти safe tooling-only, runtime-enabled wrapper над тем же System
  allocator с atomics; не меняй allocator semantics или shipping allocator;
- define exact semantics для successful allocation/reallocation bytes and
  count, zero-sized allocation, deallocation, counter overflow и before/after
  snapshot; optional live/peak values храни отдельными diagnostic fields;
- report/gate fail closed при отсутствии hook, overflow, nested measurement
  или process mismatch;
- versioned report явно хранит counter scope/process identity; scenario window
  не включает несвязанные project/bootstrap/smoke phases. Если hard profile
  требует peak host residency, измеряй настоящий scenario/process peak, а не
  masquerading end-of-run RSS;
- profiler/counter path не влияет на authoritative roots; overhead остаётся
  `<=3%` и reserved instrumentation memory `<=64 MiB`;
- добавь known-count tests, multithreaded tests, delta/reset isolation,
  overflow/failure tests и JSON/hard-counter validation.

Сначала докажи, что hook возможен без workspace `unsafe` и без замены
allocator. Если требуется новый unsafe/global-allocation boundary или изменение
обычных shipping roots, не прячь это в implementation commit: сделай отдельную
architecture subphase с новым Accepted ADR, fallback `NOT_RUN` и отдельным
commit; только затем реализуй hook следующей фазой.

Focused checks минимум: затронутые crate tests, profiler on/off root parity,
`performance --scenario smoke --mode report`, production-worker report,
`performance --scenario long-session-soak --mode report`, host-check.

Commit checkpoint: `perf(tooling): add exact allocator counters`.

## Фаза 3 — обычный live tick: iterative hotspot removal

На baseline после фаз 1–2 профилируй `long-session-soak.v3` и production worker
path. Выбери самый широкий remaining ordinary-tick stack. Возможные кандидаты —
только список для проверки, не заранее выбранное решение:

- full/COW clones оставшихся owner collections в prepare/commit;
- повторная canonical encoding/hash работа одного runtime generation;
- per-tick heap churn и временные `Vec`/`String`/maps;
- physics/RPG/presentation staging, которое можно перевести на shared immutable
  backing или bounded owner delta;
- lock/channel contention, подтверждённый production-worker diagnostic.

Для каждого принятого hotspot создай отдельную subphase `3.N`, повтори полный
decision protocol и сделай отдельный commit
`perf(<owner>): remove <measured-hotspot>`. Не объединяй две независимые
гипотезы. После каждого commit обязательно прогони diagnostic
`0/4095/4096/4097`, чтобы не вернуть O(history) clone/scan и exact eviction
осталась прежней.

Останови цикл ordinary tick после двух подряд исследованных кандидатов, которые
не достигают acceptance rule, либо когда remaining top stack меньше `10%`
ordinary-tick/window cost и его усложнение не оправдано.

Focused checks каждой subphase: owner crate tests, contracts/runtime/
application/reference-game tests по затронутой границе, history-scaling
diagnostic, `play`, `persistence-replay`, три before/after soak и host-check.

Commit checkpoint: один atomic commit на каждую успешно принятую `3.N`.

## Фаза 4 — checkpoint materialization и durable publication

Раздели `application checkpoint ~seconds` на CPU materialization, canonical
hash/validation, bytes written, число file/barrier operations, staging
read-back, rename/pointer publication и scheduler wait. Не оптимизируй
identity-root probe, пока attribution не покажет его существенную долю.

Сначала выполни instrumentation-only subphase `4.0`: bounded spans/counters для
materialize, recovery closure, durable snapshot canonicalization, pack build,
pre/post prune, staged write+sync, bytes/files/barriers, complete readback/hash/
path validation, generation rename и `CURRENT` write/sync/switch. Timing не
влияет на schedule, overflow/unowned phase даёт `NOT_RUN`, methodology/schema
versioned. Commit: `perf(application): attribute checkpoint publication`.

Разрешённые направления при наличии evidence:

- повторное использование уже validated immutable canonical component/object
  bytes внутри exact generation;
- streaming hash/encode без полного временного buffer;
- bounded scratch/capacity reuse без сохранения скрытого authoritative state;
- private pack/index layout, уменьшающий durable barriers, при полном final
  logical-hash validation и legacy raw-load compatibility;
- подготовка immutable revision-bound data до commit, если fixed publication
  boundary и stale-result rejection не меняются.

Сначала проверь уже найденные current-code candidates в этом порядке; paths и
line numbers могут сдвинуться, поэтому подтверждай symbols через `rg`:

1. `WorldCheckpointCanonicalComponentsV1` хранит bytes, но большие runtime/
   RPG/physics segment hashes могут повторно вычисляться для base state root,
   streaming state root и replay descriptors. Сохрани exact derived segment
   hashes/lengths при первом проходе и строй прежние Merkle formulas из них;
   сравни с независимым `SaveSegmentDescriptor::for_bytes` reference.
2. `ApplicationCoordinator::publish_current` повторно SHA-256-хэширует уже
   проверенные shared bytes при `SessionObjectV1::new`, а publish/cache helpers
   глубоко clone snapshot/cache. Сохрани prehashed immutable
   `SessionObjectV1`/metadata-only validated cache вместо потери hash.
3. `ContentStore::publish_inner` делает deep `publication.files.clone()` после
   того, как `SessionPublicationV1::packing_plan` уже скопировал objects в pack.
   Валидируй/write borrowed file refs; затем, только если profiler подтверждает,
   пиши Arc-срезы последовательно или используй segmented/scatter construction
   того же exact concatenated pack без giant intermediate `Vec`. Сохрани
   обязательный полный reread и проверку каждого logical object/hash.
4. `RuntimeState::materialize_command_ledger` вычисляет full identity root, а
   trusted checkpoint validation может заново хэшировать/сканировать тот же
   body. Opaque generation-bound validated-root artifact допустим только если
   decode/load/tamper path независимо делает full recompute и exact root bytes
   не меняются. Сначала убери лишний canonical-layout length traversal через
   доказанную exact fixed-record-size формулу; более сильный вариант — один
   ordered traversal, который одновременно feed-ит root hasher и ledger
   encoder. Никогда не заменяй это публичным `skip_validation` bool и сохрани
   regression `incremental_checkpoint_rejects_a_re_rooted_identity_command_id_forgery`.
5. `RuntimeSnapshotV3::canonical_bytes_and_ledger_bytes_validated`,
   `CommandBodyArchiveV1::canonical_bytes` и generic segment encoding могут
   clone whole-history buffers. Добавь borrowed/streaming exact encoder только
   с golden old/new byte equality и independent decode/hash verification.

Каждый подтверждённый пункт — отдельная subphase `4.N`, отдельные before/after
runs и отдельный owner-specific commit. Не реализуй весь список одним diff.

Запрещено без нового ADR: ослабить flush/atomicity, сдвинуть checkpoint cadence,
считать RAM cache durable, публиковать checkpoint после authoritative commit,
ввести authoritative delta schema или global mutable pack pool без crash-safe
protocol. Если лучшая измеренная идея меняет Accepted semantics ADR-035/037,
сначала отдельная architecture subphase и commit с новым superseding ADR,
обновлёнными SPEC/README/traceability/roadmap; реализацию делай следующей фазой
и отдельным commit.

Failure coverage: missing/corrupt/truncated/overlapping pack, wrong object hash,
stale cache/current pointer, publication fault на каждом durable step,
current+previous retention, process restart, close retry, save/load/replay and
snapshot equivalence.

Focused checks: assets/contracts/runtime/application tests,
`persistence-replay`, `play`, три before/after `long-session-soak`, host-check.

Commit checkpoint: один atomic commit на каждую успешно принятую `4.N`, например
`perf(persistence): remove duplicate publication copies`.

## Фаза 5 — replay без полного runtime fork на каждом tick

Профилируй replay отдельно от live tick. Текущий кандидат для проверки:
`RuntimeReplayDriver::stage_replay_tick` всё ещё может делать
`fork_from_checkpoint()` на каждом tick, а application replay повторно строит,
валидирует и кодирует checkpoint/segments для нескольких compare values.

Если attribution подтверждает hotspot, переведи replay на тот же production
`PreparedRuntimeTick`/generation validation substrate: сначала stage и сравни
все recorded inputs/stages/queries/results, затем infallible commit. Любое
несовпадение обязано оставить live runtime полностью неизменным. Верни один
validated checkpoint + canonical-components bundle и выведи state root,
ledger/archive hashes и segment hashes из него один раз.

Обязательные tests: first-divergence на каждом compare point, malformed/
tampered replay до mutation, stale generation, collision/retry history,
save→load→continue, game/headless parity и old/new exact final roots. Не
ослабляй read-only v1 replay semantics и не добавляй branching replay.

Focused checks: runtime/application replay tests, `persistence-replay`, `play`,
targeted replay performance before/after и host-check.

Commit checkpoint: `perf(replay): prepare ticks without runtime forks`.

## Фаза 6 — оптимизация production worker/render handoff

Используй метрики фазы 1. Оптимизируй только реально dominant wait/copy/
contention surface. Сохрани bounded FIFO input, exact fixed-step order, latest
immutable presentation semantics, retry-capable durable `Closed` barrier и
renderer independence от gameplay state.

До изменения channel/lock/handoff собери совместимые production-worker runs:
предпочтительно десять, если preflight позволяет, и не меньше набора, который
надёжно отделяет hotspot от шума по правилам этого goal. Не меняй bounded
blocking channel, не coalesce/drain события и не вводи `ArcSwap`/latest-value
handoff по догадке. Останови subphase без code commit, если измеряемая доля
кандидата меньше `2%`, неотличима от шума или исправление меняет accepted
event/lifecycle semantics. Coalescing допустим только при доказанной
семантической эквивалентности FIFO/lifecycle и измеренном выигрыше.

Допустимые кандидаты после подтверждения profiler:

- убрать оставшийся owned snapshot/camera/render-input clone;
- сократить `RwLock` critical section или заменить private handoff на более
  подходящий bounded latest-value primitive;
- убрать ненужное blocking ожидание, не меняя accepted input order или
  checkpoint boundary;
- reuse exact-key render plan/resources только как reconstructible cache;
- исправить frame-slot/acquire alias/wait только при сохранении device-loss и
  stale-image safety.

Acceptance дополнительно требует: `0` lost/reordered work messages, bounded
queue high-watermark, exact final roots, no presentation feedback into
simulation, no lifecycle/deadlock regression, no `>=5%` regression renderer
critical-path p95/p99 и material improvement production-worker target metric.

Focused checks: game/application/presentation/render/desktop tests, `play`,
`persistence-replay`, `platform`, static frame soak, production-worker soak,
host-check.

Commit checkpoint: `perf(game): reduce production worker latency` либо более
точное сообщение.

## Фаза 7 — immediate-scope closure

После последних code commits заново собери на clean HEAD минимум три release
run каждого доступного diagnostic: `long-session-soak`, static
`interactive-frame-soak` и production-worker soak. Сверь их с phase-start
baseline, exact roots и acceptance policy.

Если факты materially изменили состояние B-12/measurement foundation или
implementation queue, обнови `docs/roadmap.md` и этот файл краткой таблицей
completed/deferred результатов, затем закоммить
`docs(perf): record measured optimization checkpoint`. Если факты не
изменились, не делай пустой docs commit: включи документацию в последний
material code phase.

Если новая performance boundary требует native Linux validation, в той же
material phase добавь точную pending-запись в
`docs/development/linux-validation-backlog.md`: команда, prerequisite,
ожидаемый артефакт и критерий закрытия. Отсутствующий Linux host остаётся
`NOT_RUN`; не подменяй native correctness синтетическим `PASS`.

Не публикуй `PerformanceBaselineV1` из трёх runs и не переводись из
`REPORT_ONLY` в `PASS`. Не подгоняй preflight ожиданиями/retries. Если host не
готов, оставь точный `NOT_RUN`/`REPORT_ONLY` reason.

## Отложенные activation phases — не реализовывать раньше продукта

Эти фазы являются остальной частью performance roadmap, но не входят в
немедленный terminal condition. Активируй каждую только когда production
prerequisites уже существуют в current HEAD. Каждый workload и каждое codegen
promotion — отдельная фаза и отдельный commit.

### D1 — R2 alpha render workload

Prerequisites: настоящий 20–30 minute alpha project, Semantic UI/dialogue
60-second windows и representative 1920×1080 content. Не синтезируй их в
performance tooling. Реализуй primary и `b0-safe-720p30` как независимые
results по ADR-036. Commit: `perf(r2): add representative alpha workload`.

### D2 — R3 multi-region streaming workload

Prerequisites: production jobs/resources vertical, general partition,
four-region/64-chunk project и примерно `150%` residency pressure. Проверь
workers `1/2/8/16`, cancellation/I/O faults и exact roots. Required budgets:
host residency `<= 12 GiB` (`12_884_901_888` bytes), device residency
`<= 5.5 GiB` (`5_905_580_032` bytes), ready-result commit или stale rejection
`<= 2` ticks и commit p95 `<= 2_000 us`. Commit:
`perf(r3): add multiregion streaming workload`.

### D3 — R4 100-NPC workload

Prerequisites: calendar/population/navigation/RPG owner paths и реальный
ADR-016 integrated scenario. Все stage rows exclusive, unowned time invalidates
run, exact roots across restart/worker permutations. Population mix:
`16` active, `32` near и `52` background NPC; integrated simulation p95
`<= 8_000 us`, p99 `<= 12_000 us`. Commit:
`perf(r4): add integrated 100 npc workload`.

### D4 — R5 16-avatar workload

Prerequisites: production physics/animation/procedural motor stack, 120 Hz
physics/60 Hz motor, save/replay owner state и deterministic LOD. Physics p95
`<= 4_000 us`, p99 `<= 6_000 us`; если workload включает inference, его p99
`<= 0.5 ms` на avatar или `<= 2 ms` на batch из `16`. Commit:
`perf(r5): add representative physics workload`.

### D5 — ten-run calibration и hard gate

Только когда D1–D4 существуют и все required counters готовы: на exact clean
THOTH commit собери ровно десять compatible runs каждого workload без
cherry-picking, опубликуй baseline через existing command и отдельно получи
Linux `REPORT_ONLY` плюс mandatory native correctness. Generated reports не
коммить. Обновляй B-12 только если все ADR-036 exit conditions реально
выполнены. Документальный status commit допустим только при изменении факта:
`docs(perf): record hard calibration result`.

### D6 — Thin LTO, затем PGO

Запускай `docs/development/performance-codegen.md` только после D5. Baseline и
candidate используют один exact source commit и все четыре workloads по десять
runs. До первого сравнения добавь в build stamp/comparator канонически
отсортированную provenance Cargo feature set, backend и target triple. Для
representative Vulkan R2 baseline, candidate и PGO training обязаны собираться
с одним semantic feature set, включая `--features desktop-sdl-ash`; текущая
документальная команда без этого feature не является репрезентативной.
Thin LTO и PGO оцениваются отдельно; любой `NOT_RUN`, combined
improvement без статистической значимости или regression `>=2%` оставляет
default `release` неизменным. PGO training включает все четыре workloads.
Promotion `[profile.release]` — отдельное архитектурно осознанное изменение и
отдельный commit; никогда не смешивай его с алгоритмической оптимизацией.

## Терминальное условие текущего /goal

Immediate goal завершён только когда одновременно:

- фазы 1 и 2 закрыли allocator и production-worker measurement gaps;
- для ordinary tick, checkpoint и worker/render path есть актуальная profiler
  attribution;
- все evidence-backed candidates, прошедшие acceptance rule, реализованы и
  каждый завершён отдельным commit;
- цикл каждого доступного hot path остановлен по измеренному diminishing-return
  правилу, а не по догадке;
- clean checks и clean report-only confirmations прошли, exact roots не
  изменились;
- roadmap честно оставляет недоступные D1–D6 и B-12 открытыми;
- working tree чист относительно собственных изменений, а чужие изменения не
  staged и не повреждены.

В финальном ответе перечисли commits по фазам, before/after median p95 и
allocation/I/O deltas, exact root, каждый ProductCheck как `PASS`/`FAIL`/
`NOT_RUN`, QMD fallback, remaining deferred phases и главный residual risk.
