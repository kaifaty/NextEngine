# ADR-096: Active-kernel Linux performance cohort

| Поле | Значение |
|---|---|
| ID | ADR-096 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-28 |
| Последняя проверка | 2026-08-28 |
| Нормативные зависимости | [SPEC-04](../04-rendering-and-platform.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md), [ADR-091](091-linux-release-performance-authority.md), [ADR-092](092-dimensional-relative-performance-comparison.md), [ADR-093](093-deterministic-r5-worker-placement.md), [ADR-094](094-confidence-gated-relative-warnings.md) |
| Заменяет | Узко заменяет ADR-091 profile ID `ref-linux-b550i-3950x-rtx3080-v1`, exact patch-kernel value `7.0.0-29-generic` и требование нового Accepted profile revision при каждом изменении kernel patch. Остальные hardware, OS family, driver, toolchain, workload, budget, preflight, exact-root и evidence semantics ADR-091/092/093/094 остаются Accepted. |
| Заменён | не заменён |

## Контекст

ADR-091 одновременно использовал полную версию ядра как наблюдаемый
fingerprint одного evidence set и как долгоживущий admission pin. После
обычного Ubuntu kernel update это потребовало бы загружать старый установленный
patch-релиз, хотя baseline и gate уже сравнивают полный fingerprint
byte-for-byte. Такая boot ceremony не добавляет воспроизводимости внутри
кампании и задерживает R7c без продуктовой пользы.

Ослаблять сравнимость нельзя: отчёты на разных ядрах не должны образовывать
один baseline или сравниваться одним gate. Нужно разделить поддержку текущего
host family и exact identity конкретной кампании.

## Решение

### Profile V2 и допустимое активное ядро

Current profile ID становится `ref-linux-b550i-3950x-rtx3080-v2`.
Hardware, hostname, RAM/storage lower bounds, BIOS, NVIDIA driver, performance
governor, Ubuntu release, Rust toolchain и target triple остаются exactly as in
ADR-091.

OS build допускается, когда host probe публикует canonical строку
`26.04; kernel 7.0.0-<revision>-generic`, где `<revision>` — положительное
decimal integer без leading zero. Текущее `7.0.0-30-generic`, более раннее
`7.0.0-29-generic` и последующие patch revisions той же Ubuntu generic kernel
family являются отдельными допустимыми campaign cohorts. `lowlatency`, другой
kernel family, другой Ubuntu release или malformed value дают
`PERF_OS_MISMATCH`.

Routine patch update внутри этой family не требует boot rollback, редактирования
старого evidence или нового ADR. Переход на другой OS/kernel family, hardware,
BIOS, GPU driver, governor, storage identity, Rust toolchain либо target triple
по-прежнему требует явного successor profile decision.

### Exact campaign identity

Полная фактически наблюдаемая `os_build`, включая exact kernel revision,
остаётся частью `PerformanceTargetFingerprintV1`. Все десять calibration
reports одного workload обязаны иметь byte-for-byte equal full fingerprints.
Fixed three-run gate обязан иметь тот же fingerprint, что baseline.

Если active kernel или любой другой fingerprint field меняется между
independent runs, набор несовместим и начинается новая campaign; completed
negative evidence не удаляется и не перелабеливается. Один R7c release claim
по-прежнему требует R2–R5 evidence на одном exact clean commit и одном
фактическом fingerprint cohort.

Profile ID и hard-host component всех четырёх scenario hashes advance to V2.
Поэтому V1 reports/baselines остаются strict historical evidence и не могут
войти в V2 campaign даже при совпадающем фактическом kernel.

### Неизменённая evidence policy

Performance V6, methodology `nextengine-performance-v11`, canonical budgets,
ten-report baseline, isolated fixed three-run gate, CI-gated warning semantics,
deterministic R5 worker placement, profiler/resource integrity, exact roots и
no-retry policy не меняются. Старый kernel-30 collector attempt остаётся
invalid из-за omitted exact target ID, неправильной output shape и incomplete
R2 run; это решение не превращает его в evidence.

## Рассмотренные варианты

- **Загружать pinned kernel 29.** Отклонено: baseline/gate equality уже
  обеспечивает exact cohort, а rollback добавляет операционную церемонию без
  нового сравнимого сигнала.
- **Не записывать kernel version.** Отклонено: изменение среды стало бы
  невидимым и mixed campaign могла бы пройти.
- **Разрешить mixed patch kernels в одном baseline.** Отклонено: это ослабляет
  независимую evidence unit и скрывает platform drift.
- **Переписать V1 evidence как V2.** Отклонено: historical bytes и их исходная
  admission semantics неизменяемы.

## Последствия

- R7c может выполняться на текущем активном Ubuntu generic kernel без отката.
- Kernel patch остаётся точным наблюдаемым фактом и compatibility boundary
  конкретной кампании, но не долгоживущим boot requirement.
- Fresh V2 R2–R5 baselines/gates обязательны; V1 results не закрывают B-12.
- Числовые product budgets и gameplay/authoritative semantics не меняются.

## Product checks

| Check | Expected | Fallback |
|---|---|---|
| Focused fingerprint tests | Kernel 29 and current kernel 30 are independently admitted under V2; another family/flavour/malformed revision rejects | Keep R7c open |
| Baseline compatibility test | Ten individually admitted reports with one different patch kernel reject as `PERF_BASELINE_RUN_INCOMPATIBLE` | Start a new exact cohort; never mix |
| R2–R5 campaign | One clean commit and one full observed fingerprint across every baseline/gate member produce the existing hard verdicts | Preserve the first invalid/negative set and fix the cause on a successor commit/set |
