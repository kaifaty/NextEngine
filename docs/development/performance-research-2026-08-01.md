# Ресерч: методы существенного повышения производительности Next Engine

Дата: 2026-08-01. Статус: исследовательская заметка, не ADR и не roadmap commitment.
Ресерч выполнен после завершения ADR-043 (candidate-7: enabled `+4.30%` FAIL,
allocator metric disabled, B-12 OPEN). Все числа ниже — из `docs/roadmap.md`
(B-12 measurement log, REPORT_ONLY runs на Windows, шумный preflight).

## Измеренная база (наши реальные горячие точки)

| Метрика | Текущее значение | Контекст |
|---|---|---|
| Ordinary application tick p95 | 2 278 µs | single-thread simulation |
| Checkpoint application tick p95 | ~94 ms | save hitch, виден пользователю |
| Checkpoint materialization p95 | ~41 ms | уже снижено со 131 873 µs (−66%) transaction-delta package |
| Driver prepare p95 | 1 960 µs | |
| Application 1 200-tick window | 5.117 s | |
| Frame critical path p95/p99 | 125/215 µs | renderer уже быстрый |
| GPU p95 | 10 µs | |
| Frame-slot-wait p95 | 7 093 µs | вероятно present-bound |
| Allocator callbacks / kernel | 15 253 608 | budget ≈ 3.253 ns/callback (3%) |

Открытые блокеры: B-04 (job/resource substrate), B-06 (world = 2 chunks),
B-12 (allocator metric + calibration + representative R2–R5 workloads).
Ограничения: ADR-036/039 запрещают замену shipping allocator и ослабление
3% budget; ADR-036 запрещает BOLT; retry-to-green запрещён.

## Тир 1 — самые большие рычаги

### 1. Checkpoint/persistence pipeline: дельты + copy-on-write + async write-behind

**Почему это №1:** checkpoint tick ~94 ms и materialization ~41 ms — на два
порядка больше ordinary tick (2.3 ms) и на порядок больше frame budget.
Это единственная измеренная операция, которая гарантированно видна игроку
как рывок при сохранении.

Методы (подтверждены внешней практикой):

- **Incremental delta checkpointing.** Ключевое наблюдение из DeltaBox
  (arXiv 2605.22781): последовательные чекпойнты почти идентичны — дублировать
  надо только изменения. Их change-based C/R снизил latency с сотен ms до
  ~14 ms (порядок величины). Мы уже прошли часть пути (transaction-delta
  package: materialization 131 873 → 44 794 µs, −66%); следующий шаг —
  field/chunk-level dirty tracking, чтобы materialization не обходил и не
  хешировал неизменённое состояние.
- **Copy-on-write state capture.** COW-снимок (Arc-based, как CowState в
  juncture, или page-level COW в DeltaFS/DeltaCR) позволяет снять снимок
  за O(1)–O(dirty) на commit point, а сериализацию вынести из critical path.
  Важный нюанс из DeltaCR: post-snapshot COW faults размазывают latency по
  hot path — их гасят фоновым "async-warm" проходом. Для нас аналог —
  фоновая материализация вне application tick.
- **Async write-behind / durability modes.** Практика agent-checkpoint
  систем (LangGraph/juncture): Durability modes Sync/Async/Exit и
  incremental `put_writes` — запись дельты сразу по завершении шага,
  маленький payload, линейная накопительная стоимость вместо периодического
  full dump. Наш cadence `0/30/60` и durable schemas уже есть; write-behind
  убирает 94 ms из tick, оставляя durable guarantee через staging queue —
  это соответствует нашему правилу "async work returns immutable
  revision-bound results through staging queues".
- **Append-only delta channels.** Bounded checkpoint storage: per-field
  delta step (serialize only writes added that step) + периодический
  compact/snapshot. Стыкуется с B-05 (migration DAG, copy-on-write migration).

**Ожидаемый эффект:** checkpoint tick 94 ms → цель < 10 ms на текущем
контенте; materialization 41 ms → O(dirty) вместо O(world). Это самый
большой пользовательски-видимый выигрыш из всех доступных.

### 2. Параллелизм: B-04 job substrate + параллельное исполнение систем

**Почему:** ordinary tick 2 278 µs выполняется single-thread. Все systemic
R4-нагрузки (100 NPC, schedules, navigation) упираются в один поток.
B-04 (`OPEN`) — прямой gate для R3–R5.

Методы:

- **Work-stealing DAG scheduler** (классика: Chase-Lev deques, T1/P + T∞
  bound). Зависимости систем — DAG по declared read/write sets; стадии и
  commit points из нашего architecture contract сохраняются — параллелизм
  внутри стадии, не через неё.
- **Детерминизм — главный риск, и он решаем.** Random work stealing ломает
  воспроизводимость порядка. Подтверждённые подходы: Almost Deterministic
  Work Stealing (SC'19, Shiina) — детерминированное распределение задач в
  порядке serial execution (left-to-right, work-first) + иерархическое
  локализованное stealing; deterministic team-building (B-авторитет) —
  расширение stealing с детерминированной иерархией. Плюс наш собственный
  инструмент из B-04: canonical merge + logical budgets — результаты
  параллельных задач сливаются в каноническом порядке, тогда gameplay
  observable state не зависит от scheduling. Это прямо соответствует
  нашему правилу "authoritative work occurs at fixed stages and commit
  points".
- **SoA + parallel iteration.** Свежий benchmark (arXiv 2606.14919):
  archetype+SoA даёт ~2× throughput против OOP baseline, параллельный
  вариант — выше. Chunk sizing под L1/L2, sparse-set для entity churn,
  archetype graph для дешёвых component transitions.

**Ожидаемый эффект:** на multi-core — кратно на systemic workloads
(R4 100-NPC). Порядок: сначала B-04 substrate (finite queues, canonical
merge, budgets), потом параллелизация систем по одной с replay-parity
проверкой каждой.

### 3. Representative R2–R5 workloads + hard calibration (предусловие)

Все текущие runs — REPORT_ONLY: preflight не достигал idle CPU/20 GiB RAM,
hard calibration и R2–R5 workloads — `NOT_RUN`. Без них **любая** из
оптимизаций выше не может получить hard PASS, а микро-оптимизации рискуют
быть шумом (identity-root probe и checkpoint tails уже noise-sensitive).
Это не оптимизация, но разблокирует hard verdicts для всего остального и
является exit criterion B-12. ThinLTO/PGO workflow уже готов, но не
promoted именно из-за отсутствия ten-run R2–R5.

## Тир 2 — сильные, но вторичные

### 4. Снижение трафика аллокаций на тик (вместо оптимизации счётчика)

15.25M count-bearing callbacks за kernel при budget 3.253 ns/callback.
Candidate-6 (+4.81%) и candidate-7 (+4.30%) показали: микро-оптимизация
counter path упёрлась в ~4.3–4.8% при budget 3%. Альтернативный рычаг —
**уменьшить N (число аллокаций), а не стоимость одной**:

- Per-tick arena/bump allocator для transient tick data (~10× на alloc,
  O(1) free по практике arena allocators), reset на commit point.
- Pooling для повторяющихся transient структур (events, commands, staging
  results).
- Inline storage (small-vector style) для маленьких коллекций на hot path.
- SoA chunk storage снижает и количество аллокаций, и cache misses.

Двойной эффект: ускоряет ordinary tick и снижает counter overhead
пропорционально — может закрыть enabled metric дешевле, чем ещё один
candidate на counter path. Остаётся в рамках ADR-039: shipping allocator
не меняется, меняется трафик. Требует material hypothesis + новый ADR/candidate
только если считать это timing candidate для B-12; как product optimization —
свободно.

### 5. PGO/ThinLTO promotion

Реальные gains: Firefox C++ — 5–10%. В Rust сильно зависит от codegen units:
0.3% при 1 CGU, 1.2% при CGU-per-module, до +4% с тюнингом ThinLTO
`import-instr-limit`. У нас `release-pgo` workflow готов. Gate: representative
R2–R5 ten-run. BOLT/Propeller пост-линк запрещён (ADR-036). Реалистично
ожидать 3–7% на simulation-heavy workloads после promotion.

### 6. Renderer: GPU-driven culling и bindless (когда появится R2+ контент)

Текущий frame critical path 125/215 µs и GPU p95 10 µs — renderer **не**
является bottleneck на текущем two-chunk fixture. Frame-slot-wait p95
7 093 µs вероятно present-bound (vsync) — сначала измерить, не оптимизировать
вслепую. Методы на будущее (R2+ контент): two-phase occlusion culling с HiZ,
meshlets через MDI fallback (работают и без mesh shaders), bindless +
ubershader (VkCmdBindPipeline дорогой при частой смене), >1M объектов
culling < 0.5 ms на современных GPU. У нас уже есть meshlet payloads и
indexed-indirect B0 — естественный следующий шаг GPU culling, но не раньше
появления representative content load.

## Тир 3 — если вернёмся к allocator metric (candidate-8 hypotheses)

Только как material hypothesis для нового candidate (retry-to-green
запрещён): owner fast path без `try_with`/closure на hot callback, слияние
phase+cookie проверок в одну atomic load, убрать `#[inline(never)]` границы
вызовов между counter и owner, дешёвый memory ordering на control load
(relaxed + fence вместо acquire где допустимо). Каждая — отдельный ADR с
codegen proof, как ADR-042/043.

## Что НЕ делать (зафиксированные ограничения)

- Замена shipping allocator (mimalloc и т.п. на Windows быстрее HeapAlloc,
  но запрещено ADR-036/039). Использовать это как аргумент для снижения
  трафика аллокаций, не для swap.
- BOLT/Propeller (ADR-036).
- Ослабление 3% budget (ADR-039).
- Микро-оптимизации без R2–R5 ten-run evidence — риск погони за шумом.
- Параллелизм без canonical merge/deterministic ordering — ломает replay
  contract.

## Рекомендуемая очередь

1. **R2–R5 representative workloads + hard calibration** — разблокирует hard
   verdicts (B-12 exit criterion, PGO promotion, всё остальное).
2. **Checkpoint delta + COW capture + async write-behind** — 94 ms → < 10 ms,
   самый большой видимый игроку выигрыш; стыкуется с B-05.
3. **B-04 job substrate → параллельные системы** с canonical merge и
   replay-parity gate — gate для R3–R5, кратный выигрыш на systemic load.
4. **Allocation traffic reduction** (per-tick arena, pooling, inline storage) —
   ускоряет tick и снимает давление на B-12 enabled metric.
5. **PGO promotion** — сразу после появления R2–R5 ten-run.
6. **GPU culling/bindless** — только когда R2+ контент сделает renderer
   измеримым bottleneck.

## Приложение 2026-08-02: анализ ослабления staged re-read (ADR-037 §3 шаг 3)

Вопрос: можно ли убрать или ослабить полное перечитывание staged session
generation перед atomic rename (~`8.5 ms` p50 на checkpoint на этой системе,
сами чтения с диска — hashing уже убран byte-exact compare'ом).

Проверенные факты по коду:

- `SessionStore::load_current` -> `load_generation_closure(current_id)` не
  имеет fallback на previous generation: повреждённый current = fail closed
  (`crates/assets/src/session.rs:294,386`).
- Application resume/recovery (`coordinator/activation.rs:105-133`,
  `coordinator/recovery.rs:66`) прокидывают ошибку `?` — fallback'а нет и там.
  Previous generation валидируется только ПОСЛЕ успешного decode current.
- Game `SaveStore` fallback имеет (`load_latest` перебирает поколения), но
  session store — нет: его previous generation существует для rollback при
  неудачной НОВОЙ публикации, а не для восстановления после порчи current.

Вывод: staged re-read — load-bearing. Это единственная проверка, которая
гарантирует, что после rename authoritative байты на диске равны
проверенным в памяти. Её ослабление превращает publish-time bounded failure
(rollback, prior current остаётся authoritative, стабильный
`SESSION_STORAGE_UNAVAILABLE`) в load-time unrecoverable failure при
следующем запуске — при живой previous generation на диске.

Рассмотренные варианты ослабления и почему отклонены:

- **Re-read после rename с rollback CURRENT при mismatch.** Открывает crash
  window, в котором CURRENT указывает на непроверенные байты: crash в этом
  окне = corrupt current при следующем запуске без fallback. Именно порядок
  «проверить ДО rename» и есть смысл ADR-037 §3.
- **Hash-only re-read.** Требует чтения тех же байтов — стоимость (сами
  чтения) не меняется, гарантия та же, смысла нет.
- **Re-read только index/manifest.** Не ловит порчу pack/object bytes —
  теряется вся гарантия.
- **Load-time fallback current->previous.** Семантическое изменение recovery:
  молча откатывает session identity (sequence, live_session_id, closure
  chain), может воскресить закрытую сессию как live. Это product-level
  решение, требующее отдельного ADR и product checks; цена ошибки — silent
  session rollback, недопустимая размен для ~`8.5 ms` p50.

Решение: re-read НЕ ослаблять, ADR не писать. Стоимость легитимно атакуется
параллельным чтением файлов staged generation (кандидат parallel I/O) без
изменения semantics — порядок проверок и множество failure modes не меняются.

## Приложение 2026-08-02 (2): parallel encode/write в checkpoint path — ОТКЛОНЕНО release soak

Кандидат: scoped-thread fan-out (`std::thread::scope`, bounded, deterministic
error order, byte-exact assembly) в трёх местах:

1. `WorldCheckpointV4::new_with_canonical_components_internal` — runtime
   validate+encode+ledger hash ∥ rpg encode ∥ physics validate+encode.
2. `state_root_from_segments` — per-segment SHA-256 параллельно.
3. `ContentStore` publish — parallel write+fsync staged files и parallel
   reads в `verify_staged_generation_bytes` (re-read ADR-037 §3 не ослаблен,
   только распараллелен).

Реализация была завершена, contracts `129` + assets `34` tests зелёные,
`host-check` и `persistence-replay` PASS, roots parity в soak runs. Парный
same-hour A/B на эталонной нагруженной системе (1 baseline run против 3
candidate runs, paired потому что cross-batch сравнения на этой системе
шумные): baseline materialization p95 `10 514 µs` совпал со старым
baseline `10 419 µs` (система не деградировала), а кандидат дал
materialization p95 `13 674 µs` (`+30%`), application checkpoint-tick
`+22%`, ordinary tick `+65%` (путь, который кандидат вообще не трогает),
application/live-runtime windows `+27%/+43%`, driver commit `+82%`.
Равномерная регрессия всех метрик, включая незатронутые пути, указывает на
системный механизм: создание 2-4 OS threads на каждый checkpoint на
нагруженной машине стоит дороже, чем экономит параллелизм (scheduler
contention/migrations, per-thread CreateThread overhead под AV, конкуренция
fsync на одном volume). Кандидат полностью откачен; код не сохранён.

Вывод по методике: per-checkpoint thread spawning в authoritative tick path
на reference-системе — анти-паттерн независимо от чистоты реализации.
Параллелизм в этом пути имеет смысл пересматривать только вместе с B-04
job substrate (persistent bounded worker pool по ADR-026, без CreateThread
на checkpoint) и повторным paired A/B. До B-04 — не возвращаться.

## Приложение 2026-08-02 (3): derived-кэш identity-index bindings encode — ПРИНЯТО

Кандидат 4 из очереди (encode-side incremental caching). Анализ показал,
что checkpoint materialization выполняет canonical nested encode всей
bindings map identity index дважды за checkpoint: один раз внутри
`command_identity_index_root` (hash) и второй раз при canonical stream
write. Оба пути идут через `canonical_bytes`/`visit_canonical_bytes`
`CommandIdentityIndexBodyV1`, поэтому достаточно одного derived кэша в
body, без изменения call sites и wire форматов.

Реализация: приватное поле `bindings_concat: OnceLock<Arc<[u8]>>` в
`CommandIdentityIndexBodyV1` хранит count-prefixed canonical encoding
bindings map; собирается лениво через `encode_identity_bindings`,
инвалидируется в `insert_occurrence_incremental` и
`commit_prepared_replacements_deferred`. Ручные `PartialEq`/`Eq`/`Debug`
по четырём публичным полям исключают кэш из сравнений; decode идёт через
`from_parts`, поэтому загруженный индекс всегда cold-cache. Flat ledger
wire encode (`encode_identity_index`) — другой формат (raw поля без
nested headers); попытка переиспользовать кэш там дала
`Decode(UnexpectedEnd)` в round-trip тесте и была откачена, wire path
остался per-binding loop.

Замеры: identity-index-root-probe в soak p50 `458 → 243 µs`, p95
`676 → 372 µs` (`-47%`, стабильно в 3 candidate runs); synthetic probe
@12k bindings: root `2 000 → 1 210 µs`, значение root байт-в-байт прежнее.
Все soak runs дают byte-exact roots (`5e45825e…`).

Парный same-hour A/B против сомнения «шум или регрессия»: первый candidate
run показал elevation всех метрик (materialization p95 `12 817` против
baseline `10 514 µs`, ordinary tick `+38%`) — но elevation была
равномерной, включая незатронутые пути, как и в отклонённом parallel
кандидате. Контрольный эксперимент: 2 baseline runs против 3 candidate
runs в тот же час. Парный третий candidate run: ordinary tick p50/p95
`1446/1780` против baseline `1443/1828`, driver-prepare, checkpoint-tick,
materialization и windows — в пределах run-to-run spread. Вердикт:
elevation в ранних runs — фоновая нагрузка (глобальный сдвиг в одном run,
транзиентный burst в другом), регрессии нет. Кандидат принят.

Методический вывод, подтверждённый второй раз: на нагруженной
reference-системе решения по tick-path кандидатам принимаются только по
paired same-hour A/B с несколькими runs на сторону; одиночный run с
равномерным сдвигом всех метрик (включая незатронутые пути) — признак
фона, а не кода.

Открытые follow-up кандидаты (по убыванию ожидаемого эффекта):

1. `Arc::make_mut` в `insert_occurrence_incremental` и
   `commit_prepared_replacements_deferred` потенциально deep-клонирует
   всю bindings map при shared Arc — подозрение на driver-prepare
   ~1,3 ms/tick, не исследовано. Стоит инструментировать refcount и
   фактические clone counts прежде чем менять.
2. Flat ledger wire encode (`encode_identity_index`) остаётся O(history)
   (~550 µs @12k bindings); кэшировать нельзя без изменения wire формата,
   но можно рассмотреть incremental append, если формат позволяет.
3. Merged replacements root path
   (`command_identity_index_root_with_replacements`) всё ещё стримит
   per-binding visits — кандидат на тот же concat-кэш, если путь горячий.

## Приложение 2026-08-02 (4): per-tick deep-clone identity/causal maps — ПРИНЯТО

Follow-up кандидат 1 (подозрение на `Arc::make_mut` deep-clone). До
инструментирования гипотеза указывала на driver-prepare (~1,3 ms/tick);
пер-децильный анализ raw samples soak показал, что prepare плоский
(~1 264–1 293 µs), а линейно растёт driver-commit: `27 → 323 µs` за
3600 тиков. Урок: сначала пер-децильный тренд, потом гипотеза.

Диагностика временной инструментацией (strong_count + sub-step timings в
`commit_deferred_roots` и archive commit, прогон 900-тикового
verification теста): archive commit уникален (entries sc=1 после drop
base, leaf insert дёшев), а identity bindings Arc достигает
strong_count 4 в точке commit: live + update.base + staged ledger +
snapshot-cache. Источник staged-ссылки — Rust partial-move drop семантика:
`let CommandLedgerV2 { streams, .. } = staged_ledger;` НЕ освобождает
поля за `..` в точке `let` (под pinned toolchain 1.93 они доживают до
конца enclosing scope), поэтому staged identity index и causal registry
удерживали shared владение картами live-поколения через весь commit, и
каждый `Arc::make_mut` deep-клонировал полную bindings map (подтверждено:
900/900 тиков, avg identity commit растёт `14 → 26 → 38 µs` на
n=300/600/900). Четвёртый держатель — materialized-ledger snapshot cache
(`ledger_snapshot_cache`, `state.rs`), пополняемый только при
`snapshot()`/`command_ledger()` на dirty roots: ~30 тиков из 900 (тик
после каждого checkpoint), там clone легитимен по copy-on-write
семантике.

Fix: явное связывание `identity_index`/`causal_identity_registry` из
staged ledger и `drop` до copy-on-write commits (15 строк, семантика
неизменна — tick мутации несут только streams). Regression test:
`Arc::as_ptr` identity/causal карт стабилен через reportless commits
(in-place mutation), негативный контроль подтверждён (без fix тест
падает). Первый вариант теста на post-commit strong_count==1 был слеп:
lingerers дропаются при выходе из функции и финальный count всегда 1 —
guard должен наблюдать поведение В ТОЧКЕ commit, а не после неё.

Парный same-hour A/B (release soak, roots byte-exact `5e45825e…`):
driver-commit p95 `385 → 27 µs` (`-93%`), p50 `178 → 18 µs`, децильный
тренд сплющен `33 → 370` в `15 → 41` (остаточный рост — ~1 легитимный
clone на checkpoint interval из-за snapshot cache). Остальные метрики в
пределах run-to-run spread, root-probe неизменен (К4 кэш не тронут).

Методический вывод: `Arc::make_mut` на shared структурах — это
copy-on-write оптимизация только пока refcount доказуемо равен 1 в точке
мутации; destructure с `..` молча продлевает жизнь неиспользуемых полей.
Для hot commit paths явный `drop` staged поколения дешевле, чем
полагаться на end-of-scope. Известный остаточный паттерн:
`insert_occurrence_incremental` клонирует по построению
(`self.clone()` → refcount 2 → гарантированный deep-clone) — test-only
путь; если станет production-hot, реструктурировать в
check-then-mutate-in-place с pre-committed rollback или persistent map.

## Приложение 2026-08-02 (5): flat ledger wire encode через derived кэш — ПРИНЯТО

Follow-up кандидат 2. В К4 кэш canonical concat был применён только к
nested canonical путям (root/hash/stream); плоский ledger wire encode
(`encode_identity_index`, durable segment в `CommandLedgerV2` field 7 →
checkpoint/save/replay + mandated byte-exactness re-encode на каждом
decode) остался per-binding loop. Ключевое наблюдение: flat формат —
header + concat self-contained per-binding segments (BTreeMap порядок) +
trailing root, поэтому concat кэшируется БЕЗ изменения wire формата
(ранняя заметка «нельзя кэшировать без изменения формата» относилась к
попытке переиспользовать canonical nested concat для flat пути).

Реализация зеркалит К4: второй derived буфер
`bindings_flat: OnceLock<Arc<[u8]>>` в body, общая инвалидация, cold
cache на decode (`from_parts`), `encode_identity_index` =
exact-capacity writer + один `extend_from_slice` из кэша + root.
Byte-exactness зафиксирована reference-тестом (cached encode == fresh
per-binding loop после всех типов мутаций; decode → re-encode parity).
Сопутствующая механика: canonical-visit helpers и merged-root machinery
перенесены `identity_index.rs → hashes.rs` (`pub(super)`) — лимит 1000
строк/файл; перенос не меняет ни байта поведения.

Замеры: synthetic probe @12k bindings — flat encode `~1 480 → ~210 µs`
(`-86%`), 1 116 056 байт byte-exact. На soak scale (3 600 bindings)
экономия ~65 µs/encode ниже шума materialization — парный A/B
подтверждает parity (checkpoint-tick p95 swings `61–138 ms` между
runs — fsync I/O noise; решение по mechanism + probe, не по soak tail).
Память: второй derived буфер ~+1,1 MB @12k на тёплый индекс — принято
как осознанный trade-off (буфер освобождается инвалидацией на мутации).

Открытый follow-up 3: `command_identity_index_root_with_replacements`
(merged view на checkpoint через `PreparedCommandIdentityIndexUpdate::
index_root`) всё ещё кодирует всю merged map streaming'ом. Кандидатный
дизайн: segment-offset индекс поверх canonical concat (splice k
заменённых segments вместо полного re-encode) — стоимость приблизится к
cached root (~250 µs soak scale против ~460 µs свежего encode).

## Приложение 2026-08-02 (6): merged root из committed body — ПРИНЯТО

Follow-up кандидат 3. Анализ показал, что segment-offset splice
(изначальный дизайн из приложения 5) избыточен: на checkpoint
`commit_prepared_replacements` вычислял root streaming'ом merged map, а
snapshot encode следом пересобирал canonical concat той же логической
карты второй раз. Вместо splice по stale кэшу (который к моменту
checkpoint всё равно холодный — ordinary ticks инвалидируют его каждый
тик) root теперь выводится из committed body: committed map ≡ merged
view по построению, cached `command_identity_index_root` выполняет один
encode и оставляет кэш тёплым для snapshot encode. Public
`PreparedCommandIdentityIndexUpdate::index_root()` не тронут.

Parity зафиксирован тестом: committed root == streaming merged root ==
root независимо слитой карты (base + replacements), bodies равны.
Probe @12k (полный checkpoint-цикл): `~5 000 → ~4 000 µs` (`-20%`),
byte-exact. Soak A/B: parity в сопоставимых окнах, один нагруженный run
отброшен по uniform shift незатронутых метрик (driver-prepare `+35%`).

Методический вывод серии: derived-кэш окупается только если он ТЁПЛЫЙ в
точке потребления — трассировать жизненный цикл кэша (кто строит, кто
инвалидирует, кто читает) обязательно до выбора дизайна; «замерить
полный цикл потребителя», а не изолированную операцию.

Итог серии follow-up (К4–Ф3): identity-index bindings на checkpoint
пути encoded ровно один раз на формат (nested canonical + flat wire),
roots из кэша, per-tick commit без deep-clone, driver-commit flat.
Оставшиеся известные резервы: B-04 job substrate для parallel
encode/write (возврат отклонённого К3 на persistent workers),
materialized-ledger snapshot cache (легитимный ~1 clone/interval),
path-A materialize deep-clone (семантически требуемый при BTreeMap;
persistent map — отдельное архитектурное решение, не локальный fix).

## Источники

- DeltaBox: millisecond checkpoint/rollback через change-based DeltaState,
  arXiv 2605.22781 (2026) — delta/COW/async-warm, 14 ms C/R.
- ECS SoA/archetype parallel benchmark, arXiv 2606.14919 — ~2× SoA vs OOP,
  выше в parallel variant; chunk sizing, sparse-set, pooling.
- Almost Deterministic Work Stealing, SC'19 (Shiina et al.) — deterministic
  task allocation + hierarchical localized stealing.
- Work-stealing для mixed-mode parallelism (deterministic team-building) —
  детерминированная иерархия stealing.
- Job Systems for Game Engines (mightyprofessionalgaming.com) — Chase-Lev
  DAG scheduler, T1/P + T∞.
- juncture (LangGraph Rust impl) — CowState, Durability modes
  Sync/Async/Exit, incremental put_writes.
- Bounded Checkpoint Storage for Append-Only Agent State — per-field delta
  step pattern.
- Firefox PGO практика (5–10%); Rust PGO/ThinLTO CGU и import-instr-limit
  тюнинг; Windows allocator comparisons (mimalloc vs HeapAlloc) — как
  аргумент для traffic reduction, не swap.
