# Windows validation backlog

| Поле | Значение |
|---|---|
| Статус | `ARCHIVED / OUT_OF_SCOPE_INDEFINITE` historical checklist, не active roadmap queue |
| Последнее обновление | 2026-08-20 |
| Active development host | Native Linux x86_64 GNU/Vulkan |
| Out-of-scope target | Native Windows x86_64 MSVC/Vulkan |

## Назначение

Этот документ сохраняет historical bounded действия, которые требовали native
Windows host. По [ADR-090](../architecture/adr/090-linux-only-v1-and-indefinitely-deferred-windows.md)
Windows больше не является v1/R7 target и отложен вне current scope на
неопределённый срок.

Очередь ниже не является active debt, release blocker или обещанием поддержки.
Отсутствие Windows run не превращается в `PASS`, но и не является Linux v1
non-claim. Исторические Windows результаты сохраняют только exact-commit
meaning.

## Execution policy

1. Частые и затронутые `fast`, `play`, `persistence-replay`,
   `content-package`, `platform` и report-only `performance` checks выполняются
   на Linux вместе с work package.
2. Новые work packages не добавляют Windows follow-up в этот файл.
3. Windows commands и THOTH calibration/hard gates не запускаются в current
   roadmap. Их отсутствие не блокирует Linux handoff или release.
4. Historical Windows packages/reports действуют только для записанного exact
   commit и не переносятся на новый changeset.
5. R7/v1 использует только native Linux release candidate; paired compare не
   участвует.
6. Cross-compilation, Wine и VM smoke могут помогать диагностике, но не
   заменяют native Windows target evidence.
7. Generated reports, packages, state, logs и caches не коммитятся.

## Deferred queue

| ID | Статус | Когда активировать | Действие на Windows | Открытый claim |
|---|---|---|---|---|
| `WIN-001` | `ARCHIVED` | Future ADR-090 re-entry decision only | Historical full native-gate/pair recipe. | No current claim |
| `WIN-002` | `ARCHIVED` | Future ADR-090 re-entry decision only | Historical THOTH baseline/gate recipe. | No current claim; B-12 is now Linux R7c |
| `WIN-003` | `ARCHIVED` | Future ADR-090 re-entry decision only | Historical Windows package/manual acceptance recipe. | No current claim |
| `WIN-004` | `ARCHIVED` | Future ADR-090 re-entry decision only | Historical Windows Stage 0 matrix recipe. | No current claim |

## Re-entry conditions

Windows work can return only after a new product decision and Accepted ADR that:

- explicitly adds Windows shipping/support scope and a separate roadmap slot;
- фиксирует exact native host, OS/toolchain, GPU/driver и THOTH applicability;
- identifies an available native owner/host and treats historical evidence as
  historical rather than compatible by default;
- перечисляет applicable ProductChecks и output locations;
- сохраняет Linux current checks и запрещает перенос exact-commit results.

Only then may selected items become a new active task state. Уже
опубликованный `FAIL` сохраняется; повтор выполняется только как новый run с
новым identity/output и не relabel-ит прежний evidence.

## Current non-claims

- Linux hardware correctness не является Windows correctness.
- Linux `REPORT_ONLY` timing не является THOTH hard timing.
- Historical Windows `PASS` не подтверждает текущий commit.
- Archived `WIN-*` items do not block current Linux development, R7 or v1 and
  do not constitute a support commitment.
