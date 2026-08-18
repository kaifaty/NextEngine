# Windows validation backlog

| Поле | Значение |
|---|---|
| Статус | `DEFERRED / PRE-R7` operational checklist, не нормативная архитектура |
| Последнее обновление | 2026-08-18 |
| Active development host | Native Linux x86_64 GNU/Vulkan |
| Deferred target | Native Windows x86_64 MSVC/Vulkan |

## Назначение

Этот документ хранит bounded действия, которые требуют native Windows host.
По [ADR-082](../architecture/adr/082-linux-first-development-and-deferred-windows-host.md)
Windows остаётся v1 shipping target, но сейчас не запускается и не является
условием handoff для Linux feature-development.

Текущий roadmap продолжается на Linux. Отсутствие Windows run не превращается
в `PASS`: Windows-specific, THOTH, paired Windows/Linux и release claims
остаются `NotRun(WindowsHostDeferred)` либо открытыми до explicit pre-R7
bring-up.

## Execution policy

1. Частые и затронутые `fast`, `play`, `persistence-replay`,
   `content-package`, `platform` и report-only `performance` checks выполняются
   на Linux вместе с work package.
2. Изменение Windows-specific кода или общего
   platform/renderer/physics/package/performance boundary добавляет ровно один
   bounded item в очередь ниже, если существующий item его не покрывает.
3. До отдельного bring-up Windows commands и THOTH calibration/hard gates не
   запускаются. Их отсутствие не блокирует текущие R5/R6 Linux increments.
4. Historical Windows packages/reports действуют только для записанного exact
   commit и не переносятся на новый changeset.
5. Перед R7/v1 выбирается новый clean release-candidate commit. Обе native
   halves, packages и `native-gate-compare` выполняются на этом exact commit.
6. Cross-compilation, Wine и VM smoke могут помогать диагностике, но не
   заменяют native Windows target evidence.
7. Generated reports, packages, state, logs и caches не коммитятся.

## Deferred queue

| ID | Статус | Когда активировать | Действие на Windows | Открытый claim |
|---|---|---|---|---|
| `WIN-001` | `DEFERRED` | Explicit pre-R7 Windows bring-up | На выбранном clean commit выполнить полный `native-gate-run`, перенести complete target bundle, выполнить matching Linux half и `native-gate-compare`. | R1/B-01 и paired release evidence |
| `WIN-002` | `DEFERRED` | После фиксации нового THOTH host/toolchain/driver profile | Собрать десять valid clean release Performance V5 runs для каждого required R2–R5 workload и затем один fixed three-run v8 hard gate без retry-to-green. | B-12/R7 hard performance |
| `WIN-003` | `DEFERRED` | После material package/runtime change либо release-candidate freeze | Пересобрать Windows package и повторить representative R2 manual visual/session acceptance на current content. | Current Windows player-facing acceptance |
| `WIN-004` | `DEFERRED` | PhysX Stage 0 readiness review | Выполнить Windows half platform/replay/R5 performance matrix и required correspondence route на принятом SDK/profile. | PhysX Stage 0/default-readiness |

## Bring-up entry conditions

Windows work начинается только после нового Accepted ADR, который:

- явно возвращает Windows в active target matrix;
- фиксирует exact native host, OS/toolchain, GPU/driver и THOTH applicability;
- определяет новый clean candidate commit и совместимость historical evidence;
- перечисляет applicable ProductChecks и output locations;
- сохраняет Linux current checks и запрещает перенос exact-commit results.

После этого backlog items переводятся в active task state по одному. Уже
опубликованный `FAIL` сохраняется; повтор выполняется только как новый run с
новым identity/output и не relabel-ит прежний evidence.

## Current non-claims

- Linux hardware correctness не является Windows correctness.
- Linux `REPORT_ONLY` timing не является THOTH hard timing.
- Historical Windows `PASS` не подтверждает текущий commit.
- Отложенные `WIN-*` items не блокируют текущую Linux R5/R6 разработку, но
  соответствующие R1/R7/v1 claims остаются открытыми.
