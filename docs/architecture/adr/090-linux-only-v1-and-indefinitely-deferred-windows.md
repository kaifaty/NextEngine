# ADR-090: Linux-only v1 and indefinitely deferred Windows

| Поле | Значение |
|---|---|
| ID | ADR-090 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-20 |
| Последняя проверка | 2026-08-20 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-04](../04-rendering-and-platform.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-29](../29-platform-host-and-application-session.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-001](001-product-repository-license-and-platforms.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-082](082-linux-first-development-and-deferred-windows-host.md) |
| Заменяет | Полностью заменяет host/release policy ADR-082. Узко заменяет Windows как обязательную v1 shipping target и Windows/Linux paired release evidence в ADR-001, ADR-011, ADR-036, ADR-045, ADR-060..063, ADR-084..089, SPEC-00/04/09/12/15/29/35 и roadmap. Linux active-host semantics сохраняются. |
| Заменён | не заменён |

## Контекст

ADR-082 перенёс текущую разработку на native Linux, но сохранил Windows как
обязательную v1 shipping target и сделал explicit Windows bring-up
предпосылкой R7. Доступного Windows host сейчас нет, его возвращение не
планируется и не должно удерживать готовый Linux product в неопределённом
pre-release состоянии.

Это уже не временная перестановка работ. Windows выводится из текущей
продуктовой границы на неопределённый срок. Исторические Windows реализации и
отчёты полезны как локализованная техническая история, но не создают текущего
обязательства по поддержке, проверке или выпуску.

## Решение

### V1 platform boundary

Native `x86_64-unknown-linux-gnu` с Vulkan 1.3 и X11 либо Wayland является:

- единственной v1 shipping target;
- единственным active development/release host;
- обязательной target для install/package/platform и финального R7 evidence.

Windows x86_64/MSVC/Vulkan получает статус
`OUT_OF_SCOPE / INDEFINITELY_DEFERRED`:

- Windows package, native run, THOTH calibration, Windows acceptance и
  Windows/Linux compare не входят в R7 или v1 criteria;
- отсутствие Windows evidence не является blocker, `NOT_RUN` либо release
  non-claim для Linux v1;
- существующий Windows adapter/tooling MAY оставаться в дереве, но не получает
  current support, compatibility или maintenance promise;
- новые work packages не обязаны накапливать Windows follow-up и не должны
  резервировать под него WIP.

Historical Windows packages/reports сохраняют только exact-commit meaning.
Они не подтверждают текущий код и не должны называться current target
support.

### R7 release boundary

R7 становится Linux-only release-candidate этапом. Он не добавляет новые
подсистемы и состоит из следующих bounded outcomes:

1. **R7a — Linux release authority.** Удалить обязательную Windows slot из
   текущего `v1-closure`/native-gate release verdict, version-нуть изменившиеся
   отчёты и доказать, что один native Linux bundle может честно завершить v1
   closure. Реализованный comparator MAY остаться dormant utility, но не
   участвует в Linux release verdict.
2. **R7b — release package and clean install.** Freeze final project/content,
   собрать reproducible Linux package, проверить ELF/glibc/Vulkan/SDL
   prerequisites и запустить скопированные `game`, `headless` и tools из
   изолированного clean-install окружения.
3. **R7c — Linux performance authority.** Принять отдельный exact Linux
   release profile и числовую acceptance policy, затем собрать совместимые
   clean baselines/gates для затронутых representative workloads. Frozen
   THOTH/Windows budgets не переносятся на Linux автоматически.
4. **R7d — final product hardening.** На release candidate пройти gameplay,
   save/load/replay, corrupted-input, long-session, renderer/input/audio,
   lifecycle/recovery и offline fallback paths; закрывать только
   release-blocking defects.
5. **R7e — distribution closure.** Проверить notices, provenance, dependency
   inventory, protected-data absence, getting-started/troubleshooting,
   versioning и reproducible manifest, затем выполнить финальный Linux
   acceptance run.

Пока R7a не реализован, существующий aggregate `v1-closure` может всё ещё
показывать прежний Windows slot и `shipping_ready = false`. Это известный
implementation gap, а не восстановленный Windows blocker и не разрешение
вручную relabel-ить старый report.

### Performance boundary

`ref-win-thoth-v1`, его baselines и hard gates становятся frozen historical
design outside current v1 scope. Linux report-only runs сохраняют
диагностическую ценность, но сами по себе не становятся release PASS. R7c
обязан либо принять воспроизводимую Linux hard profile/policy, либо оставить
performance release criterion открытым; отсутствие Windows не ослабляет
требование измерить Linux product.

### Re-entry rule

Windows может вернуться только отдельным будущим product decision:

- новый Accepted ADR явно добавляет Windows shipping/support scope;
- roadmap выделяет отдельный stage/work package, не скрытый R7 tail;
- указаны owner, доступный native host, toolchain/driver/runtime profile,
  package policy и applicable ProductChecks;
- historical evidence не переносится вперёд и не relabel-ится.

До выполнения всех четырёх условий Windows work не планируется и не является
долгом текущего Linux release.

## Product impact

Путь к v1 снова зависит только от доступной и реально проверяемой системы.
R7 концентрируется на качестве устанавливаемого Linux продукта: финальном
игровом цикле, сохранении/воспроизведении, package safety, performance,
лицензиях и документации. Выпуск больше не ждёт отсутствующий host и не
выдаёт исторические Windows проверки за поддержку.

Пользователю Windows ничего не обещается в рамках v1. Сохраняющийся код может
оказаться полезной стартовой точкой будущего porting stage, но не является
supported binary или compatibility contract.

## Relevant checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| documentation/link validation | ADR/SPEC/roadmap/README/agent guidance | Linux-only v1 и indefinite Windows non-goal сформулированы одинаково; ссылки и IDs разрешаются | Не активировать новый roadmap state до исправления противоречий |
| future R7a focused + `host-check` | Versioned Linux-only release report/closure implementation | Один complete native Linux bundle может получить release-ready verdict без fabricated Windows result | Оставить R7 planned; не relabel-ить старый V1 report |
| future Linux `native-gate-run`/package | Clean exact release candidate | Linux target bundle, isolated package smoke и required product roots проходят | Не публиковать package |
| future R7c performance | Accepted Linux release profile and representative workloads | Compatible clean baseline/gate и unchanged authoritative roots | Снизить только declared presentation/LOD quality либо оставить release criterion open |

## Рассмотренные варианты

- **Оставить Windows prerequisite без срока.** Отклонено: недоступный host
  превращает R7 в бессрочный внешний blocker и не улучшает Linux продукт.
- **Считать текущий Linux `REPORT_ONLY` полной заменой THOTH.** Отклонено:
  target сменился, поэтому fingerprint, budgets и baseline должны быть приняты
  отдельно, а не унаследованы молча.
- **Удалить Windows код немедленно.** Отклонено: изменение shipping scope не
  требует destructive cleanup; код можно оставить dormant, пока он не мешает
  Linux correctness и maintenance.
- **Сохранить paired report schema и подставлять synthetic Windows PASS.**
  Отклонено: fabricated evidence нарушает fail-closed release semantics.

## Последствия

- R1/B-01 cross-target gap перестаёт блокировать R7/v1.
- Windows backlog архивируется как dormant historical checklist и не является
  очередью текущего roadmap.
- B-12 меняется с Windows/THOTH blocker на необходимость отдельной Linux
  release performance authority в R7c.
- R7 может начаться на текущем Linux host с R7a; pre-R7 Windows decision больше
  не существует.
- Изменение report/closure semantics остаётся implementation work R7a и не
  считается выполненным этой documentation-only фиксацией.

## Supersession

Добавление shipping platform, смена Linux ABI/baseline, отказ от измеримого R7
performance criterion или изменение release-ready aggregation требует нового
Accepted ADR и соответствующего roadmap slot.
