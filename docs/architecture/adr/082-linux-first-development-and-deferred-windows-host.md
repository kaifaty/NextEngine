# ADR-082: Linux-first development and deferred Windows host

| Поле | Значение |
|---|---|
| ID | ADR-082 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-18 |
| Последняя проверка | 2026-08-18 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-29](../29-platform-host-and-application-session.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md) |
| Заменяет | Узко заменяет Windows-first execution policy и трактовку THOTH как active development host в ADR-036/060/061, а также требование выполнять Windows/THOTH evidence как текущий R5 development gate. Shipping targets, THOTH fingerprint/budgets, Performance V5/v8, native comparator и exact Windows/Linux release evidence сохраняются, но откладываются до явного Windows bring-up перед R7. |
| Заменён | не заменён |

## Контекст

Текущая разработка выполняется на native Linux x86_64 с Vulkan-capable GPU и
активной desktop session. Этот host уже прошёл полный hardware native gate, но
roadmap, README и часть performance tooling продолжали считать Windows/THOTH
основным developer host, а Linux — отложенной асинхронной проверкой.

Такое расхождение давало ложный `NOT_RUN` для реализованного
`r2-alpha-render`, переносило Linux-проверки за Windows checkpoint и делало
недоступный сейчас Windows host обязательным для текущего R5 feedback loop.
При этом замена release-матрицы на Linux-only была бы другой продуктовой
семантикой: Windows x86_64 остаётся заявленной v1 shipping target.

## Решение

### Active development host

Native `x86_64-unknown-linux-gnu` с системным Vulkan loader, доступным GPU и
X11/Wayland session является текущим active development host. Применимые по
ProductCheck map `fast`, `play`, `persistence-replay`, `content-package` и
затронутые `platform`/`performance` checks MUST выполняться на Linux в том же
work package, которому они принадлежат.

Renderer performance report запускается явно с desktop adapter:

```text
cargo run --locked --release -p xtask --features desktop-sdl-ash -- \
  performance --scenario r2-alpha-render --mode report
```

`r2-alpha-render.v3` MUST выполнять шесть production Vulkan окон на
поддержанном Linux desktop host и возвращать внешний `PASS` с вложенным
`REPORT_ONLY`, если workload и report evidence полны. Linux process peak RSS и
process I/O читаются из native procfs; Vulkan workload добавляет device ceiling
и timestamp evidence. Disabled desktop feature, отсутствующая display/GPU или
неполное evidence возвращают typed `NOT_RUN`, а не Windows-only diagnostic.

### Deferred Windows host

Windows x86_64/MSVC/Vulkan остаётся v1 shipping target и допустимой private
adapter implementation, но не является текущим developer host. До отдельного
Windows bring-up решения:

- Windows checks, THOTH calibration/hard gates и same-commit target compare не
  запускаются и не требуются для текущего Linux feature-development handoff;
- соответствующие target/release claims остаются `NotRun(WindowsHostDeferred)`
  либо открытыми, а не наследуют исторический Windows result;
- Windows-specific work накапливается в bounded backlog и не занимает текущий
  R5/R6 implementation slot;
- исторические Windows packages/reports сохраняют только свой exact-commit
  смысл и не подтверждают новый changeset.

Перед R7/v1 shipping Windows MUST быть явно возвращён в active target matrix,
после чего на одном новом clean commit выполняются обе native halves и
`native-gate-compare`. Возобновление Windows execution и выбор release
performance gate требуют нового superseding ADR, чтобы не смешивать
исторический THOTH baseline с новой host/toolchain/driver policy.

### Performance authority

Linux timings остаются `REPORT_ONLY`; этот ADR не превращает текущую Linux
машину в новый hard reference host и не переносит на неё THOTH budgets.
`ref-win-thoth-v1`, числовые ADR-016/036/062 budgets, ten-run baseline и fixed
three-run V5/v8 gate сохраняются как deferred release design, но не блокируют
функциональное продолжение R5 на Linux. B-12 является R7/release blocker, а не
текущим R5 implementation gate.

Host applicability меняет identity только `r2-alpha-render`: его scenario hash
advance-ится с v2 до v3. Общий Performance V5 wire и methodology
`nextengine-performance-v8` не меняются; старый R2 hash нельзя смешивать с v3
report/baseline.

## Product impact

Разработчик получает короткий реальный Vulkan/performance feedback loop на
доступной системе вместо успешного render planning внутри внешнего
Windows-only `NOT_RUN`. Windows не удаляется из продукта и Linux report не
выдаётся за release evidence. R1 paired closure, Windows acceptance refresh,
hard B-12 и v1 shipping честно остаются отложенными.

Gameplay, save/replay, project/content schemas, public platform contracts и
package ABI не меняются.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | Linux process-counter parsing, R2 scenario identity and strict report tests | VmHWM/read/write bytes parse exactly; v2 R2 hash is incompatible with v3 | Return typed unavailable counter; never fabricate resource evidence |
| `platform` | Current Linux SDL3/ash B0 scene on the connected display | Hardware Vulkan path consumes the current project and preserves authoritative roots | `NotRun` when display/GPU is unavailable; headless remains usable |
| `performance --scenario r2-alpha-render --mode report` | Six Linux production Vulkan windows at 1080p/720p | Outer `PASS`, nested `REPORT_ONLY`, 21,600 measured samples, Linux RSS/I/O, Vulkan queries/device ceiling and exact roots | Typed `NOT_RUN`; no Windows or B-12 claim |
| documentation/link validation | Architecture, roadmap, agent guidance and target backlog | Linux-first/Windows-deferred wording is consistent and all direct references resolve | Keep prior policy until the inconsistency is repaired |

## Рассмотренные варианты

- **Оставить Windows-first policy и считать Linux запуск исключением.**
  Отклонено: доступный production host продолжал бы получать ложный
  Windows-only diagnostic и откладывать релевантную проверку.
- **Сделать текущий Linux host новым hard timing oracle.** Отклонено: машина,
  OS image, driver и noise baseline не приняты как exact release profile;
  report-only evidence достаточно для текущей разработки.
- **Удалить Windows из v1 targets.** Отклонено: пользователь изменил порядок
  разработки, а не продуктовую shipping boundary.
- **Считать исторический Windows result достаточным для новых commits.**
  Отклонено: target evidence остаётся exact-commit и не переносится вперёд.

## Последствия

- README, AGENTS, SPEC-04/09/12/35, routing, traceability и roadmap называют
  Linux текущим development host и Windows deferred target.
- R2 report tooling больше не содержит supported-Windows-only diagnostic;
  scenario hash становится v3.
- Linux report resource evidence получает native peak RSS и I/O counters.
- Windows backlog заменяет прежнюю Linux-deferred очередь; завершённый Linux
  catch-up остаётся историческим exact evidence.
- Ни один Linux report не закрывает Windows, THOTH, paired R1, B-12 или v1.

## Supersession

Возврат Windows в active development/release matrix, изменение v1 shipping
targets или принятие Linux hard reference profile требует нового Accepted ADR
с exact host fingerprint, evidence compatibility и mapped ProductChecks.
