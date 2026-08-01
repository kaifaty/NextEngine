# ADR-038: Versioned production-worker handoff diagnostic

| Поле | Значение |
|---|---|
| ID | ADR-038 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-01 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-29](../29-platform-host-and-application-session.md), [SPEC-30](../30-presentation-extraction-and-render-content.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md) |
| Заменяет | Узко заменяет diagnostic-scenario часть tooling contract [ADR-036](036-thoth-reference-performance-profile.md): main-to-simulation-worker handoff больше не является неизмеряемым gap и получает отдельный versioned `production-worker-soak`. THOTH profile, hard timing authority, budgets, baseline и representative R2–R5 requirements ADR-036 не меняются. |
| Заменён | не заменён |

## Контекст

`interactive-frame-soak` измеряет production Vulkan path над статическими
immutable render inputs. Он намеренно не приписывает renderer fixture задержки
между main callback и simulation worker. До этого решения bounded queue,
`next-simulation`, fixed-step application advance, shared snapshot publication
и main-side read оставались отдельным diagnostic gap.

Проверка этого handoff через verification-only копию очереди не доказывает
поведение `next_game`. Постоянная instrumentation в игровом hot path, напротив,
создала бы overhead даже при выключенной диагностике. Нужен opt-in scenario над
тем же production worker boundary, который сохраняет operational timings и
exact counts, но не делает wall clock частью gameplay authority.

## Решение

### 1. Отдельный versioned report-only scenario

Repository performance CLI поддерживает:

```text
cargo run --release -p xtask -- performance \
  --scenario production-worker-soak \
  --mode report \
  [--output <directory>]
```

`production-worker-soak.v1` выполняет не менее `240` FIFO main callbacks при
declared `60 Hz` cadence через production-owned bounded queue,
`next-simulation`, `FixedStepLiveSchedulerV1`, application coordinator и shared
immutable `PresentationSnapshotV2` handoff. Scenario имеет собственный
versioned scenario hash, machine-readable methodology version и
scenario-specific details schema.

Scenario всегда `REPORT_ONLY`. Запрос того же scenario с `--mode gate`
возвращает `NOT_RUN` со stable diagnostic. Он не является representative R2
workload, не создаёт hard budget и не закрывает B-12.

### 2. Production path и выключенный fast path

Diagnostic использует тот же application-owned worker и bounded
`sync_channel`, что и game composition root. Verification crate MAY сравнивать
результаты, но не владеет очередью и не входит в production execution path.

Timing buffers, queue telemetry, clocks и processed-callback atomics создаются
только для opt-in diagnostic. Обычный `next_game` не выполняет diagnostic clock
reads и atomic counter updates. Callback sequence в diagnostic остаётся
operational metadata и не выбирает simulation work, target tick или commit
order.

### 3. Exact bounded observations

Diagnostic сохраняет bounded raw samples и exact counts для:

- main-side queue send wait;
- message age в worker dequeue;
- ordinary и durable-checkpoint fixed steps;
- shared snapshot publication-lock и main read-lock wait;
- publication callback sequence, snapshot generation lag и freshness;
- queue high-water, submitted/processed callbacks, publications, drops и
  reorder observations.

Diagnostic send и dequeue observation линеаризованы вокруг того же bounded
channel operation. Поэтому queue high-water означает exact channel occupancy,
а не оценку outstanding application work. Snapshot и callback sequence,
которая его опубликовала, читаются из одной lock-protected generation и не могут
образовать torn sidecar pair.

Completed report требует exact submitted/processed/sample counts, FIFO order,
нулевые drop/reorder и согласованное число fixed-step/publication samples.
Любое расхождение даёт `PERFORMANCE_MEASUREMENT_INVALID`, а не частичный timing
claim.

### 4. Authority и lifecycle

Wall time, lock wait, queue depth, thread completion и freshness являются
operational tooling data. Они не входят в command/event/state/ledger roots и не
влияют на scheduling или authoritative outcome. Diagnostic сохраняет final
authoritative state, command archive, identity-index и ledger hashes; focused
parity scenario сравнивает их с serial production advance.

Worker проходит production launch/resume, fixed-step и exact close paths.
Ошибка workload не разрешает оставлять detached worker: terminal worker outcome
получается до удаления scratch state. Durable close failure возвращается как
stable tool error и сохраняет prior complete session generation.

## Product impact

Разработчик получает воспроизводимую границу между renderer-only frame timings
и реальным simulation handoff. Отчёт показывает queue pressure, worker cost и
snapshot freshness без постоянной цены в shipping fast path и без ложного hard
performance verdict.

Изменение не добавляет gameplay contract, не меняет authoritative scheduler,
save/replay schema или THOTH baseline. Existing smoke, long-session и
interactive-frame reports сохраняют свои роли.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | FIFO worker/serial parity, 30/60/144 Hz cadence, bounded-queue saturation, checkpoint/lifecycle attribution, snapshot sidecar consistency и disabled diagnostic fast path | Exact roots/counts, high-water достигает declared capacity без drop/reorder, ordinary/checkpoint samples классифицированы, disabled path не создаёт clocks/atomics | Исправить production worker boundary; не публиковать diagnostic claim |
| `performance --scenario production-worker-soak --mode report` | Не менее `240` FIFO callbacks через bounded game queue и shared snapshot handoff | Versioned raw samples/details, exact callback/step/publication counts, zero drop/reorder, unchanged authoritative roots, verdict `REPORT_ONLY` | Вернуть stable measurement/tool error; не синтезировать частичный report |
| `performance --scenario production-worker-soak --mode gate` | Попытка использовать diagnostic как hard gate | Stable `NOT_RUN`; B-12 остаётся open | Использовать будущий representative R2–R5 workload на compatible THOTH baseline |
| `play` / `persistence-replay` | Game/headless и profiler off/on permutations | Accepted commands, final state и ledger roots exact; renderer cadence и diagnostics не меняют authority | Отключить optional diagnostic и сохранить production application path |

## Рассмотренные варианты

- **Оставить handoff diagnostic gap.** Отклонено: renderer fixture не показывает
  queue pressure, fixed-step worker cost или shared-snapshot freshness.
- **Добавить simulation timing в `interactive-frame-soak`.** Отклонено: static
  renderer fixture и production worker имеют разные inputs и ownership.
- **Создать verification-only worker.** Отклонено: копия не доказывает поведение
  game composition root и может разойтись с production lifecycle.
- **Всегда собирать timings в `next_game`.** Отклонено: optional observability не
  должна добавлять clocks/atomics в выключенный hot path.
- **Сделать scenario hard gate.** Отклонено: reference fixture не является
  representative R2–R5 workload и не имеет calibrated budget.

## Последствия

- SPEC-09, traceability и roadmap ссылаются на ADR-038 для production-worker
  diagnostic semantics.
- ADR-036 остаётся authority для THOTH fingerprint, baseline и hard timing
  policy; заменена только его diagnostic-scenario часть.
- Scenario-specific raw samples и details остаются bounded tooling data и не
  входят в `crates/contracts`.
- B-12 остаётся `OPEN`, пока не пройдут documented representative workloads,
  calibration и shipping-target requirements ADR-036.
