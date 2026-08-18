# ADR-036: THOTH reference performance profile и hard timing authority

| Поле | Значение |
|---|---|
| ID | ADR-036 |
| Статус | Accepted |
| Версия | 1.5 |
| Дата решения | 2026-07-30 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-04](../04-rendering-and-platform.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | частично [ADR-016](016-compositional-gameplay-budgets.md) и SPEC-05 `PHYS-P4`: qualifier `reference 8-core CPU` заменяется полным host `ref-win-thoth-v1`; числовые gameplay/physics budgets не меняются |
| Заменён | Diagnostic-scenario часть узко заменена [ADR-038](038-versioned-production-worker-handoff-diagnostic.md). Allocator instrumentation и V2/V3 tooling schemas заменены [ADR-049](049-performance-evidence-without-allocator-instrumentation.md). CPU/GPU idle and free-RAM preflight thresholds are superseded by [ADR-060](060-relaxed-thoth-performance-preflight.md), with the load ceiling subsequently superseded by [ADR-061](061-forty-percent-thoth-load-preflight.md). NVIDIA driver fingerprint and R5/PHYS-P4 workload details are superseded by [ADR-062](062-r5-physx-humanoid-performance-authority.md). [ADR-082](082-linux-first-development-and-deferred-windows-host.md) defers Windows/THOTH execution and moves B-12 from the current R5 loop to the pre-R7 release boundary; the THOTH profile, budgets and evidence design remain frozen rather than active. |

> ADR-063 supersedes raw-frame relative bootstrap and single-run candidate
> evidence with Performance V5 run boundaries and fixed three-run gate batches.
> Current report/baseline schemas are Performance V5 under ADR-063. Older
> schema and allocator passages below are historical context only.
> The 5%/20 GiB preflight values below are also historical; ADR-060 and
> ADR-061 sets the current below-40%/10 GiB thresholds; ADR-062 sets the
> R5 workload; ADR-063 sets the current methodology identity and run-level gate.

## Контекст

Существующие performance checks используют малые two-chunk, one-agent и
five-object fixtures. Они полезны как smoke, но не являются representative
R2–R5 workload и не могут закрыть roadmap blocker B-12. Кроме того, qualifier
`reference 8-core CPU` не задаёт воспроизводимую машину, ОС, GPU, storage,
driver или power policy.

Hard timing verdict должен опираться на один измеримый hardware/software
fingerprint. Cross-machine Windows/Linux timings остаются полезной
диагностикой, но не должны создавать условный `PASS`.

## Решение

### Reference host

Единственный hard timing target имеет ID `ref-win-thoth-v1`:

| Поле | Exact profile |
|---|---|
| Host | `THOTH` |
| CPU | AMD Ryzen 9 3950X, 16 physical cores / 32 logical threads |
| GPU | NVIDIA GeForce RTX 3080, 10,240 MiB VRAM |
| RAM | 32 GiB installed |
| Storage | `WDS100T1X0E-00AFY0`, NVMe 1 TB |
| OS | Windows 11 Pro, build `10.0.26200` |
| NVIDIA driver | `610.88` (superseded value fixed by ADR-062) |
| Power plan | `AMD Ryzen High Performance` |

Hard run использует всю машину. Искусственное ограничение affinity или CPU
count до восьми ядер запрещено. `PHYS-P4` и reference scenario ADR-016
используют полный THOTH host; их числовые thresholds остаются прежними.

BIOS version не задаётся этим ADR, но записывается baseline fingerprint.
Изменение BIOS, OS build, GPU driver, power plan, Rust toolchain, content hash
или methodology invalidates baseline.

### Timing authority и platform scope

- `PASS`/`FAIL` по hard timing принимается только на совместимом
  `ref-win-thoth-v1`, в Cargo profile `release` и относительно совместимого
  baseline.
- Linux performance имеет только `REPORT_ONLY`. Отсутствие Linux timing
  baseline не блокирует hard timing verdict.
- Native Windows/Linux build, platform lifecycle, replay и exact hash
  correctness остаются обязательными shipping checks. Linux `REPORT_ONLY`
  timing не превращает `NotRun` correctness в `Pass`.
- Renderer quality, wall timing, profiler state, allocator/GPU counters и
  captures являются operational/presentation data и не входят в gameplay
  authority.

### B0 renderer budgets

Representative R2 alpha workload имеет два явно независимых результата:

- primary `1920×1080`: frame critical path `p95 <= 14,000 us`,
  `p99 <= 16,670 us`;
- fallback `b0-safe-720p30`, `1280×720`: `p99 <= 33,330 us`.

`frame_critical_path_us` равен
`max(cpu_extract_and_submit_us, gpu_timestamp_duration_us)`. VSync wait
исключается, missed deadlines учитываются отдельно. Fallback `PASS` не
скрывает primary `FAIL`; automatic mid-session quality switching не входит в
v1.

### Tooling contract

Repository command:

```text
cargo run --release -p xtask -- performance \
  --scenario <smoke|long-session-soak|r2-alpha-render|r3-multiregion-streaming|r4-100npc|r5-physics-16> \
  --mode <report|gate> \
  [--target ref-win-thoth-v1] \
  [--baseline <baseline.json>] \
  [--output <directory>]

cargo run --release -p xtask -- performance-baseline \
  --runs <directory-with-exactly-ten-run-directories> \
  --output <directory>
```

Default без аргументов сохраняется как совместимый `smoke/report`.
`long-session-soak` — отдельный report-only diagnostic: он выполняет `3 600`
live ticks через driver и interactive application scheduler, сохраняет три
окна по `1 200` ticks, обязательные 30-tick checkpoint samples и isolated
identity-index/archive-root probes. Как и `smoke`, он не имеет hard budget и
не может закрыть B-12.
`PerformanceRunV2`, `PerformanceMetricV1`, `PerformanceBaselineV2` и
`PerformanceVerdict` являются versioned tooling JSON, не public gameplay
contracts. Run сохраняет commit/cleanliness, toolchain, build profile,
scenario/content hashes, full target fingerprint, preflight, raw samples,
nearest-rank p50/p95/p99, resource/instrumentation summary, authoritative
hashes и methodology. V2 использует `PerformanceResourceCountersV2` и strict
nested `ProcessAllocationCounterV1`. Shapes `PerformanceMetricV1` и
`PerformanceMethodologyV1` не меняются, но значение `methodology_version` в
run обязано быть `nextengine-performance-v2`.

`performance-baseline` читает ровно десять direct-child
`performance-report-v2.json`, сортирует их по имени каталога, отклоняет
symlink/reparse и несовместимые/dirty/`NOT_RUN` runs и публикует один
`performance-baseline-v2.json` через create-then-rename. Existing output не
перезаписывается. Baseline не создаётся, пока profiler parity, preflight и все
required hard counters не валидны.

`gate` разрешён только при одновременном выполнении условий:

- exact `release`, clean commit, target `ref-win-thoth-v1`;
- compatible host и baseline;
- CPU/GPU load ниже 5%, free RAM не менее 20 GiB, thermal checks clear;
- profiler buffer не overflowed, нет dropped/unowned spans;
- representative scenario действительно реализован.

Любое несовпадение даёт `NOT_RUN` со stable diagnostic. Smoke fixture и ещё не
реализованный representative workload также дают `NOT_RUN` в `gate`, а не
условный `PASS`.

### Measurement и regression policy

- Percentiles используют nearest-rank и все samples без удаления outliers.
- Baseline содержит десять чистых запусков одного commit на THOTH.
- Изменение command/event/replay result или authoritative root — немедленный
  `FAIL` независимо от timing.
- Absolute budget overrun на THOTH — `FAIL` без retry-to-green.
- Относительно compatible baseline: `<2%` — noise, `2–5%` — `WARNING`,
  `>=5%` при полностью превышающем 5% deterministic bootstrap 95% interval —
  `FAIL`.
- Runtime profiling включается настройкой в той же release binary. Per-thread
  buffers finite и preallocated; dropped span, overflow или unowned gameplay
  span invalidates run. Declared profiler overhead — не более 3% и 64 MiB.
- Heavy captures/profiles остаются local artifacts и не коммитятся.

### Rollout и B-12

Сначала стабилизируются smoke schema, spans/counters и десять calibration runs.
Hard gate включается только после стабилизации noise и появления
representative workload соответствующего roadmap stage. R2 render workload
появляется вместе с alpha project; R3 scheduler не проектируется раньше
конкретного R2/R3 streaming use case.

B-12 остаётся `OPEN`, пока THOTH не даст hard `PASS` на representative
workload, Linux не даст report-only профиль, а native correctness не пройдёт на
обеих shipping targets. Smoke, fallback-only `PASS` или любой `NOT_RUN` blocker
не закрывают.

Shipping Thin LTO/PGO MAY быть принят только после алгоритмических
оптимизаций, статистически значимого улучшения объединённого R2–R5 workload и
отсутствия >=2% regression любого другого hard scenario. `target-cpu=native`,
nightly portable SIMD, unsafe intrinsics, BOLT и недоказанная глобальная замена
allocator не входят в shipping profile.

## Product impact

Разработчик получает один воспроизводимый источник hard timing verdict и не
может случайно закрыть performance blocker малым fixture или другой машиной.
Linux сохраняет обязательную correctness роль без ложного cross-host timing
oracle. Gameplay/replay semantics и public contracts не меняются.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `performance --scenario smoke --mode report` | Текущие two-chunk/one-agent/five-object/live fixtures | Versioned report содержит все четыре результата, build/methodology/fingerprint и exact roots; timings имеют `REPORT_ONLY` | Исправить tooling; не делать product timing claim |
| `performance --scenario long-session-soak --mode report` | `3 600` live ticks с held movement, periodic camera input и тем же workload через interactive application scheduler | Три 1 200-tick окна, 30-tick in-memory `WorldCheckpoint` state samples, paired application cadence samples без persistence work, isolated identity/archive root probes и exact driver/application ledger-root parity имеют `REPORT_ONLY` | Диагностировать history-dependent growth; не подменять representative R2–R5 gate |
| `performance-baseline --runs <dir> --output <dir>` | Ровно десять clean compatible THOTH reports одного commit | Один strict `PerformanceBaselineV2`; malformed, incomplete, missing-counter или mixed набор отклоняется | Исправить calibration environment; baseline не синтезируется из частичных runs |
| `performance --scenario <R2–R5> --mode gate` | Release run на THOTH с compatible baseline | Exact fingerprint/preflight; absolute и relative verdict; correctness roots неизменны | `NOT_RUN` при несовместимой среде/workload; bounded presentation/LOD fallback не скрывает primary failure |
| `performance` schema/fingerprint tests | Known percentile/bootstrap vectors, malformed JSON/baseline и wrong host | Nearest-rank exact; unknown fields reject; wrong host `NOT_RUN` | Reject baseline/run до timing verdict |
| `play` / `persistence-replay` | Profiler on/off, renderer primary/fallback/headless и worker permutations | Commands, events, replay result и authoritative roots exact | Disable optional profiling/presentation path; timing success не компенсирует divergence |

## Рассмотренные варианты

- **Эмулировать восемь ядер на THOTH.** Отклонено: affinity становится ещё
  одной нестабильной operational переменной и не описывает shipping host.
- **Использовать Linux timings как второй hard gate.** Отклонено: reference
  Linux hardware не зафиксирован; новый hard host требует отдельного ADR.
- **Считать fallback прохождением всего B0.** Отклонено: это скрывает primary
  regression.
- **Разрешить smoke закрывать B-12.** Отклонено: fixture topology прямо не
  представляет R2–R5 product workload.

## Последствия

- SPEC-04/05/09/12, ADR-016, traceability и roadmap ссылаются на этот profile.
- Tooling может появиться раньше representative workload, но возвращает
  честный `NOT_RUN` для недоступного gate.
- Числовые ADR-016, PHYS-P4 и motor inference budgets не изменены.
