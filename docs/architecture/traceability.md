# Product-goal → check map

| Поле | Значение |
|---|---|
| ID | TRACE-001 |
| Статус | Accepted |
| Версия | 2.6 |
| Последняя проверка | 2026-07-28 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |
| Заменяет | TRACE-001 2.5 |

## Назначение

Эта карта помогает выбрать небольшой набор проверок по наблюдаемой цели
продукта. Она не пытается перечислить каждое предложение всех SPEC и не
создаёт отдельную систему допуска изменений.

Canonical check names и правила conditional запуска определены в
[SPEC-12](12-vertical-slice-conformance.md).

## Карта

| Product goal | Основные checks | Наблюдаемый сигнал | Relevant specs |
|---|---|---|---|
| Быстро получать полезную обратную связь | `fast` | workspace checks и focused tests завершаются с ясной причиной failure | SPEC-09, SPEC-15 |
| Играть в малый RPG loop полностью offline | `play` | boot, movement, contact-gated pickup, equip, package melee игрока и canonical NPC planner command, interaction, bounded NPC/quest outcome и canonical two-chunk transition с возвратом работают без network/`ai-host` | SPEC-00, SPEC-01, SPEC-06, SPEC-13, SPEC-14, SPEC-19, SPEC-20, SPEC-25, SPEC-27 |
| Сохранять authoritative world и воспроизводить bug | `persistence-replay` | `WorldCheckpointV4`/`ReplayManifestV4` плюс отдельный world-streaming owner segment сохраняют inventory membership, equipment, collected state, character health, pending chunk transition и replay agent command с ожидаемыми command/event/state/ledger roots; corrupt или bootstrap RPG V3 input отклоняется до partial activation | SPEC-02, SPEC-03, SPEC-06, SPEC-14, SPEC-19, SPEC-21, SPEC-22, SPEC-25, SPEC-27 |
| Детерминированно загружать и выгружать chunks | `play`, `persistence-replay`; conditional `performance` | worker completion permutations дают один staging/commit hash; duplicate/corrupt required group и injected publication fault сохраняют прежнюю active generation; unload/reload не воскрешает durable RPG state | SPEC-03, SPEC-20, SPEC-23, SPEC-25, ADR-021, ADR-026 |
| Загружать neutral content и public mechanic packages | `content-package` | validate/cook/resolve/atomic publish/production load; first-party interaction, combat и Luau scripted-melee packages проходят тот же public capability, affordance, effect-request и RPG command path | SPEC-03, SPEC-07, SPEC-13, SPEC-17, SPEC-24 |
| Безопасно обрабатывать untrusted input | `fast`, `content-package` | malformed bounds/hash/version, Luau sandbox escape, forbidden capability, trap и instruction/allocation/live-memory/host-call/command budget exhaustion отклоняются до mutation; package state/circuit round-trip exact | SPEC-07, SPEC-10, SPEC-11, SPEC-13, SPEC-24, ADR-014 |
| Диагностировать regression без private test backdoor | relevant check + local scenario | stable diagnostic содержит first divergence; minimized replay сохраняет failure identity | SPEC-09, SPEC-15 |
| Сохранять player-facing presentation | `play`; optional capture | UI/camera/animation/audio/render change виден в том же gameplay scenario и не меняет authority | SPEC-04, SPEC-08, SPEC-18, SPEC-28, SPEC-30 |
| Поддерживать shipping platform, которую реально изменили | conditional `platform` | релевантный smoke запускается на affected Windows/Linux target; public API остаётся engine-owned | SPEC-04, SPEC-17, SPEC-29 |
| Не ухудшать затронутый hot path | conditional `performance` | declared numeric scenario остаётся в допустимом regression threshold без изменения gameplay result | SPEC-05, SPEC-06, SPEC-08, SPEC-16, SPEC-23, SPEC-26, SPEC-27, SPEC-30 |
| Сохранять deterministic fallback для AI/physical policy | `play`, `persistence-replay`; `performance` при изменении hot path | absent/bad optional service или model route приводит к declared local fallback без blocked tick и duplicate command | SPEC-05, SPEC-06, SPEC-14, SPEC-27 |
| Проверять заменяемость grounded-capsule physics backend | `physics-collision --backend compare`, `persistence-replay --backend reference\|physx`, `physics-backend-parity`; conditional `platform`, `performance` | reference и PhysX дают exact canonical pose/contact phase/order/checkpoint/replay result; activation fallback не меняет tick, а runtime failure сохраняет previous checkpoint | SPEC-05, SPEC-21, SPEC-26, ADR-033 |
| Не смешивать Gothic data с engine runtime | `content-package` только при изменении importer/neutral contract | isolated importer output проходит bounds/provenance validation; runtime/packages не содержат legacy/protected bytes | SPEC-10, SPEC-11, SPEC-24 |
| Развивать живые quest opportunities без сетевой зависимости | `play`, `persistence-replay` | NPC intent/world need становится Quest только после admission; direct/solicited/contextual/public disclosure, activation, autonomous outcome и exact recorded candidate/template fallback проходят fixed decision boundary, общий RPG command path и replay | SPEC-31 |
| Дать конфликтующим богам независимо реагировать на игрока | `play`, `persistence-replay`, `content-package` | один committed act может улучшить qualitative standing у одного бога и ухудшить у другого; все боги читают один pre-decision snapshot, spillover затрагивает только eligible target, offers/covenants используют явные transitions, cross-context result атомарен, а replay не вызывает модель | SPEC-31, ADR-031 |

## Как выбирать checks

1. Назвать изменённое product behavior и relevant subsystem.
2. Найти соответствующую строку карты.
3. Запустить объединение основных checks.
4. Добавить `platform` или `performance` только при выполнении их trigger из
   SPEC-12.
5. Для нового failure path добавить negative scenario в тот же check.
6. В handoff записать `Pass`, `Fail` и `NotRun(reason)` вместе с остающимся
   продуктовым риском.

Если изменение не попадает ни в одну строку, сначала формулируется новый
наблюдаемый product goal и малый check. Добавлять глобальную бюрократическую
матрицу для этого не требуется.

## REQ/FAIL identifiers

Существующие `REQ-*` и `FAIL-*` identifiers MAY оставаться в subsystem SPEC
как editorial anchors и удобные search labels. Они:

- не образуют глобальный реестр;
- не требуют reserved ranges или непрерывной нумерации;
- не определяют набор ProductCheck;
- не являются proof completeness, release status или product quality;
- MAY быть удалены или переименованы при обычном редактировании relevant
  документа, если ссылки рядом обновлены.

Новая архитектурная формулировка SHOULD предпочитать прямую ссылку на section,
public contract и observable ProductCheck, а не выделять новый глобальный
номер.
