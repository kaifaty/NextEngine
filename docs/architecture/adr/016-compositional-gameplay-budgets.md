# ADR-016: Compositional gameplay budgets

| Поле | Значение |
|---|---|
| ID | ADR-016 |
| Статус | Accepted |
| Версия | 1.2 |
| Дата решения | 2026-07-23 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

## Частичное supersession ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежний общий admission lifecycle. Compositional budget model и
deterministic workload остаются targeted performance contract для изменений,
которые materially затрагивают tick time, scheduling или related resource use.

## Контекст

Независимые subsystem p95/p99 thresholds не складываются: каждый subsystem
может уложиться в свой limit, пока integrated gameplay tick систематически
превышает общий budget. Reference 100-NPC workload также не воспроизводим, если
due work выбирается wall clock, worker load или случайным iteration order.

## GameplayBudgetMatrix

Engine-owned versioned `GameplayBudgetMatrix` имеет simulation rate 30 Hz и
integer microsecond limits:

| Budget row/stage ID | p95_us | p99_us |
|---|---:|---:|
| core-command-rpg | 1500 | 2000 |
| mechanics | 2000 | 4000 |
| perception-world-services | 1000 | 1500 |
| agent-planning | 1250 | 1500 |
| navigation | 1250 | 1500 |
| state-hash-save-delta-snapshot | 500 | 500 |
| streaming-staging-commit | 250 | 500 |
| reserved-headroom | 250 | 500 |
| TOTAL | 8000 | 12000 |

Rows являются mutually exclusive stage spans. Child span time включается ровно
в одну row; nested instrumentation вычитается при построении exclusive
duration. Work вне известных rows получает `UNOWNED_GAMEPLAY_SPAN` и делает
измерение недействительным. Reserved headroom является declared unallocated
allowance, а не executable stage.

Matrix canonical, hash-bound к run и содержит profile ID, target triple,
build/config/content hashes, reference hardware ID, tick rate,
warm-up/measured counts, workload manifest hash и methodology version.

## Deterministic 100-NPC workload

Reference fixture содержит exact 100 `PersistentId` и fixed membership:

| Class | Count | Cadence at 30 Hz | Period ticks |
|---|---:|---:|---:|
| active | 16 | 10 Hz | 3 |
| near | 32 | 2 Hz | 15 |
| background | 52 | 0.5 Hz | 60 |

Membership хранится в workload manifest, сортируется по `PersistentId` и не
выводится из current distance/camera во время measured run. Для каждого NPC
cadence phase:

```text
phase = u64_le(first_8_bytes(SHA256(
  "nextengine.npc-cadence-phase.v1\0" || PersistentId[16]
))) mod period_ticks
```

NPC due на tick, когда `tick mod period_ticks == phase`. Manifest сохраняет
computed phase, а validator пересчитывает её. Wall clock, renderer frame,
worker index, completion order и load не меняют membership, phase или due
count. Due tasks сортируются по `(service_stage, class_id, PersistentId)` до
dispatch; results возвращаются через deterministic staging order.

## Project overrides

Project может заменить non-headroom row limits только versioned policy, если:

- сумма p95 rows не превышает `8_000` microseconds;
- сумма p99 rows не превышает `12_000` microseconds;
- reserved headroom остаётся не меньше `250/500` microseconds;
- ни одна stage row не исчезает и unowned time не маскируется;
- integrated reference scenario повторяется на exact override hash.

Изолированный subsystem result не переносится на override. Изменение workload,
methodology, reference hardware или total limits требует нового ADR, а не local
threshold edit.

## Reference performance scenario

На declared reference 8-core CPU profile release build:

1. загружает exact neutral fixture, workload и policy;
2. выполняет `1_000` warm-up ticks вне percentiles;
3. выполняет `10_000` consecutive measured gameplay ticks;
4. записывает per-row exclusive microseconds, total gameplay time, due-work
   counts, queue depth, defers, starvation age и budget-overrun counters;
5. вычисляет nearest-rank percentiles без dropping outliers;
6. сравнивает каждую row и integrated total с matrix, проверяет deterministic
   due-work trace и отсутствие unowned spans.

Ожидаемый результат: `total p95 <= 8_000 us`, `total p99 <= 12_000 us`, все
rows внутри limits, нет starvation, dropped due work или watchdog marker,
expected due counts exact. Queue/defer policy имеет finite deterministic maximum
age; превышение даёт `PERF_QUEUE_STARVATION` независимо от total percentile.

Agent-planning и navigation rows измеряют всю due work на tick, включая
scheduled entities/jobs и queue handling, а не per-agent/per-query allowance,
который можно умножить сверх matrix. Mechanics использует ту же fixture, spans
и measured interval. Хороший isolated result не компенсирует integrated
overrun.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Integrated reference workload | Default/override sums valid; deterministic workload; каждая row и total внутри limits; нет starvation, dropped due work, watchdog marker или unowned span | Оптимизировать без изменения authoritative semantics; иначе не заявлять соответствующий performance target |
| Agent-planning row со всеми due jobs | Membership/due order exact, row obeys matrix, deterministic deferral имеет finite maximum age | Использовать in-process planner или bounded deterministic deferral; не выбирать route по wall time |
| Navigation row со всеми due queries | Query order, queue depth и starvation bounds deterministic; required work не теряется | Использовать engine-owned graph navigation и bounded deterministic queue |
| Mechanics row в том же interval | Command/mechanic workload учитывается в одной row без double counting | Reject или deterministically defer optional package work; required work не отбрасывается |
| Invalid sums, reduced headroom, worker-order permutation, starvation, dropped work или unowned span | Каждый случай даёт stable diagnostic и делает scenario result недействительным | Исправить policy, instrumentation или scheduling и повторить focused run |

## Последствия

Subsystem budgets являются частями одного integrated matrix, а не независимыми
разрешениями превысить total. Reference workload используется только когда
изменение затрагивает performance; unrelated development не зависит от
доступности exact reference hardware.
