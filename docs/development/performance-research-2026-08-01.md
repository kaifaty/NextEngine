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
