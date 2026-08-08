# Agent routing: какие документы читать перед изменением

| Поле | Значение |
|---|---|
| ID | ROUTE-001 |
| Статус | Accepted |
| Версия | 1.3 |
| Последняя проверка | 2026-08-08 |

Детерминированная маршрутизация от типа задачи к обязательным документам.
Назначение — не дать агенту (или человеку) начать изменение, не прочитав
регулирующий SPEC/ADR. Это навигационный слой над [README](README.md):
при расхождении применяются precedence rules из README, а не этот файл.

## Как пользоваться

1. Найти строку (или несколько), соответствующую задаче. Изменение, попадающее
   в несколько строк, наследует документы и checks всех этих строк.
2. Прочитать указанные документы **полностью** через прямое чтение файла.
   Search snippets и ranking scores — discovery aids, не authority.
3. При конфликте семантики применять precedence из README (новый superseding
   Accepted ADR → ADR-030 для workflow → профильный technical ADR → subsystem
   SPEC → SPEC-00 → glossary).
4. Запустить указанные product checks (`fast` нужен всегда, остальные — по
   строке) и в handoff сообщить каждый check как passed / failed / not run.
5. Если задача не попадает ни в одну строку — читать полный индекс в
   [README](README.md) и [glossary](glossary.md). Для roadmap-чувствительных
   задач дополнительно читать [roadmap](../roadmap.md).

## Routing-таблица

| Зона задачи (триггеры) | Читать SPEC | Читать ADR | Product check |
|---|---|---|---|
| Workflow, checks, handoff, процесс разработки | [README](README.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md) | ADR-030 | fast |
| Public contracts, stable IDs, commands/events, snapshots, manifests (`crates/contracts`) | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md) + SPEC затронутой подсистемы | ADR-002 | fast + checks затронутой области |
| Детерминизм, replay, command identity, ledger, save/load, persistence | [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md) | ADR-022, ADR-046 | persistence-replay |
| Schema registry, миграции, совместимость версий данных | [SPEC-22](22-schema-registry-compatibility-and-migration.md) | ADR-025, ADR-046, ADR-048 | persistence-replay |
| ECS, runtime data model, fixed stages, scheduling | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | ADR-022 | persistence-replay |
| Assets, streaming, persistence, content catalog, bundles, neutral asset schemas | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | ADR-026, ADR-014, ADR-044 | content-package |
| Rendering, Vulkan, shaders, presentation extraction, render content | [SPEC-04](04-rendering-and-platform.md), [SPEC-30](30-presentation-extraction-and-render-content.md) | ADR-003, ADR-028 (+ ADR-035) | play (+ platform при host/packaging) |
| Physics world, collision, constraints, queries, canonical snapshots | [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-05](05-physics-animation-and-motor-control.md) | ADR-027, ADR-032 | play, persistence-replay |
| PhysX backend (Proposed track) | [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) | ADR-033 (fallback — reference backend) | play, platform |
| Skeletal animation, retargeting, IK | [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-05](05-physics-animation-and-motor-control.md) | ADR-027 | play |
| Motor control, policies, deterministic inference, training lifecycle | [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md) | ADR-009, ADR-013 | performance (model runtime) |
| AI agents, perception, memory, LLM/process boundary | [SPEC-06](06-ai-agents-perception-and-memory.md) | ADR-005 | play |
| Dialogue, model packs (Proposed track) | [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md) | ADR-017 | play |
| Current RPG domain and quests | [SPEC-19](19-rpg-domain-and-narrative-state.md) | ADR-020 | play |
| Future narrative director or divine-standing proposal | SPEC-31 (Proposed intent only) | ADR-029/ADR-031 (Superseded; no current obligation), ADR-046 | none until a production consumer exists |
| World simulation, population lifecycle, time advance | [SPEC-20](20-world-simulation-and-population-lifecycle.md) | ADR-021 | persistence-replay |
| Luau/Wasm scripting, plugins, mod packages, gameplay mechanics authoring | [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | ADR-008, ADR-014 | content-package |
| Player interaction, UI, camera, localization, accessibility | [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md) | ADR-019, ADR-044 | play |
| Project composition, configuration, application lifecycle | [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md) | ADR-018, ADR-048 | fast, content-package |
| Platform host, application session, presentation authority, Save/Load/close | [SPEC-29](29-platform-host-and-application-session.md) | ADR-028, ADR-035, ADR-047 | play, persistence-replay, platform |
| Jobs, memory, resource residency, I/O backpressure, performance budgets | [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md) | ADR-016, ADR-036, ADR-038, ADR-045, ADR-049 | performance |
| Tooling, SDK, observability | [SPEC-09](09-tooling-sdk-and-observability.md) | — | fast |
| Gothic importer boundary, neutral artifacts | [SPEC-10](10-gothic-importer-boundary.md) | ADR-001 | content-package |
| Security, licensing, governance, secrets, provenance | [SPEC-11](11-security-licensing-and-governance.md) | ADR-001 | fast |
| Audio, navigation, world services | [SPEC-08](08-audio-navigation-and-world-services.md) | — | play |
| Roadmap: scope, stage, blocker, exit criterion, очередь работ | [roadmap](../roadmap.md) + SPEC затронутой подсистемы | по строке подсистемы | по строке подсистемы |

## Historical-only — не authority

ADR-004, ADR-006, ADR-007, ADR-010, ADR-012, ADR-015, ADR-023, ADR-024,
ADR-029, ADR-031, ADR-039, ADR-040, ADR-041, ADR-042, ADR-043
полностью superseded; [evidence register](evidence-register.md) и review
packets 1.0–1.9 (`docs/reviews/`) — historical snapshots. Их MAY читать как
контекст, но они не разрешают и не блокируют изменения.

## Proposed — не shipped

SPEC-16/ADR-017 (dialogue model packs), narrative/divine intent formerly in
ADR-029/ADR-031, и PhysX backend из ADR-033 — Proposed.
Не представлять как реализованное; при работе рядом указывать fallback и
bounded evaluation path. `docs/plans/` и `docs/development/` — рабочие
материалы и research notes, не normative architecture.

## Поддержка этого файла

Обновлять routing-таблицу в том же change, который добавляет, supersede или
меняет статус SPEC/ADR либо меняет product checks. Редакционная правка
(ссылки, статусы) нового ADR не требует.
