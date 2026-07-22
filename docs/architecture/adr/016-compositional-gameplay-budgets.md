# ADR-016: Compositional gameplay budgets

| Поле | Значение |
|---|---|
| ID | ADR-016 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Release Engineering |
| Требуемые согласующие | Architecture Working Group, Release Engineering, Runtime Team, RPG Framework Team, Gameplay Extensibility Team, Agent Intelligence Team, World Services Team, Persistence Team |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

Этот ADR задаёт candidate performance contract и не объявляет reference hardware/runtime PASS. Он становится baseline только после approval и синхронного обновления профильных SPEC/gates.

## Контекст

Независимые subsystem p95/p99 thresholds не складываются: каждый subsystem может пройти в изоляции, пока integrated gameplay tick систематически превышает общий budget. Reference 100-NPC workload также не воспроизводим, если due work выбирается wall clock, worker load или случайным iteration order.

## GameplayBudgetMatrix

`VerificationPolicyManifest` содержит versioned `GameplayBudgetMatrix`. Default profile имеет simulation rate 30 Hz и integer microsecond limits:

| Budget owner/stage ID | p95_us | p99_us |
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

Rows являются mutually exclusive owner spans. Child span time включается ровно в одну row; nested instrumentation вычитается при построении exclusive duration. Work вне известных rows получает `UNOWNED_GAMEPLAY_SPAN` и invalidates run. Reserved headroom измеряется как declared unallocated allowance, а не executable owner.

Matrix JCS-canonical, hash-bound к run и содержит profile ID, target triple, build/config/content hashes, reference hardware ID, tick rate, warm-up/measured counts, workload manifest hash и methodology version.

## Deterministic 100-NPC workload

Reference fixture содержит exact 100 PersistentIds и fixed membership:

| Class | Count | Cadence at 30 Hz | Period ticks |
|---|---:|---:|---:|
| active | 16 | 10 Hz | 3 |
| near | 32 | 2 Hz | 15 |
| background | 52 | 0.5 Hz | 60 |

Membership хранится в workload manifest, сортируется по PersistentId и не выводится из current distance/camera во время measured run. Для каждого NPC cadence phase:

```text
phase = u64_le(first_8_bytes(SHA256(
  "nextengine.npc-cadence-phase.v1\0" || PersistentId[16]
))) mod period_ticks
```

NPC due на tick, когда `tick mod period_ticks == phase`. Manifest сохраняет computed phase и validator пересчитывает её. Wall clock, renderer frame, worker index, completion order и load не могут менять membership, phase или due count. Due tasks сортируются по `(service_stage, class_id, PersistentId)` до dispatch; results возвращаются через deterministic staging order.

## Project overrides

Project MAY заменить non-headroom row limits только versioned policy, если одновременно:

- сумма p95 rows не превышает `8_000` microseconds;
- сумма p99 rows не превышает `12_000` microseconds;
- reserved headroom остаётся не меньше `250/500` microseconds;
- ни одна owner row не исчезает и unowned time не маскируется;
- весь integrated `PERF-01` profile повторяется на exact override hash.

Изолированный subsystem PASS не переносится на override. Изменение workload, methodology, reference hardware или total limits требует нового ADR, а не local threshold edit.

## PERF-01 methodology

`PERF-01` выполняет release build на declared reference 8-core CPU profile:

1. загрузить exact neutral fixture/workload/policy manifests;
2. выполнить `1_000` warm-up ticks, не включая их в percentiles;
3. выполнить `10_000` consecutive measured gameplay ticks;
4. записать per-owner exclusive microseconds, total gameplay time, due-work counts, queue depth, defers, starvation age и budget-overrun counters каждого tick;
5. вычислить percentiles nearest-rank over per-tick values без dropping outliers;
6. сравнить каждую row и integrated total с matrix; проверить deterministic due-work trace и отсутствие unowned spans.

Integrated PASS требует `total p95 <= 8_000 us` и `total p99 <= 12_000 us`, все owner rows внутри limits, no starvation, no dropped due work, no watchdog/non-conforming marker и exact expected due counts. Queue/defer policy MUST иметь finite deterministic maximum age; превышение даёт `PERF_QUEUE_STARVATION` независимо от total percentile.

`AI-04` и `NAV-P3` измеряют total work due для соответствующей row на каждом tick, включая all scheduled entities/jobs и queue handling, а не per-agent/per-query budget, который можно умножить сверх matrix. `MECH-05` использует ту же fixture, spans и measured interval. Subsystem-only pass при integrated fail остаётся overall FAIL.

## Gates и failure cases

| Gate | Blocking contract | Required evidence |
|---|---|---|
| `PERF-01` | Default/override sums valid; deterministic workload; all row and total percentiles pass; no starvation/unowned work | Policy/workload manifests, 11 000-tick trace, exclusive spans, due/queue/defer counters, percentile report |
| `AI-04` | Agent planning row включает весь due work и соблюдает matrix | Per-tick membership/due trace and exclusive span |
| `NAV-P3` | Navigation row включает весь due work и bounded deterministic deferral | Query/due/queue/starvation trace and exclusive span |
| `MECH-05` | Mechanics row измерена в integrated profile | Command/mechanic workload trace and exclusive span |

Negative corpus включает invalid default/override sum, reduced headroom, worker-order cadence permutation, subsystem-only pass при integrated fail, deliberate queue starvation, dropped due work, unowned span и owner/total overrun. Каждый случай MUST давать stable diagnostic и блокировать PASS.

## Последствия и синхронизация при принятии

При approval SPEC-06, SPEC-08, SPEC-12, SPEC-13, `VerificationPolicyManifest`, AI-04, NAV-P3, MECH-05, traceability и glossary обновляются атомарно. Release Engineering владеет integrated matrix/methodology; subsystem owners владеют своими rows и evidence. До promotion этот ADR остаётся `Proposed/AwaitingReview`.
