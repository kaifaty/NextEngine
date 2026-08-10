# Agent routing: какие документы читать перед изменением

| Поле | Значение |
|---|---|
| ID | ROUTE-001 |
| Статус | Accepted |
| Версия | 2.9 |
| Последняя проверка | 2026-08-10 |

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
4. Запустить указанные product checks для executable scope (`fast` нужен для
   code/build/schema/generated-data changes, остальные — по строке) и в handoff
   сообщить каждый check как passed / failed / not run.
5. Если задача не попадает ни в одну строку — читать полный индекс в
   [README](README.md) и [glossary](glossary.md). Для roadmap-чувствительных
   задач дополнительно читать [roadmap](../roadmap.md).

## Validation scope

ADR-030/SPEC-12 documentation-only cheap path имеет приоритет над check column
ниже: если diff содержит только human-readable documentation/agent guidance и
не меняет executable code, build/configuration, schemas, generated fixtures,
package manifests или runtime-consumed data, запускать Cargo/`host-check` не
нужно. Выполняются `git diff --check` и direct link/path/ID validation;
executable checks получают `NotRun(NoExecutableChange)`. Documentation,
сопровождающая implementation, наследует обычные checks затронутой строки.

## Routing-таблица

| Зона задачи (триггеры) | Читать SPEC | Читать ADR | Product check |
|---|---|---|---|
| Workflow, checks, handoff, процесс разработки | [README](README.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md) | ADR-030 | docs-only: `git diff --check` + direct reference validation; executable change: risk-scoped `fast` |
| Public contracts, stable IDs, commands/events, snapshots, manifests (`crates/contracts`) | [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md) + SPEC затронутой подсистемы | ADR-002 | fast + checks затронутой области |
| Детерминизм, replay, command identity, ledger, save/load, persistence | [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md) | ADR-022, ADR-046; ADR-059 for PhysX continuation | persistence-replay |
| Schema registry, миграции, совместимость версий данных | [SPEC-22](22-schema-registry-compatibility-and-migration.md) | ADR-025, ADR-046, ADR-048 | persistence-replay |
| ECS, runtime data model, fixed stages, scheduling | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | ADR-022, ADR-051 when world streaming stage is affected | persistence-replay |
| Assets, persistence, neutral content and current partition manifest | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | ADR-026, ADR-014, ADR-044, ADR-048, ADR-051 | content-package |
| Rendering, Vulkan, shaders, presentation extraction, render content | [SPEC-04](04-rendering-and-platform.md), [SPEC-30](30-presentation-extraction-and-render-content.md) | ADR-003, ADR-028 (+ ADR-035) | play (+ platform при host/packaging) |
| Physics world, collision, constraints, queries, canonical snapshots and PhysX SDK/FFI/backend | [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md), [SPEC-05](05-physics-animation-and-motor-control.md) | ADR-027, ADR-032, ADR-058, ADR-059, ADR-062, ADR-063, ADR-064 | fast, play, persistence-replay, platform; conditional performance |
| Skeletal animation, retargeting, IK | [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-05](05-physics-animation-and-motor-control.md) | ADR-027 | play |
| Motor control, BodySchema/instance overlays, skill/contact/motion hierarchy, policy families, deterministic inference and replay | [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md) | ADR-013, ADR-027, ADR-057, ADR-058, ADR-059, ADR-062, ADR-063, ADR-064 | play, persistence-replay; content-package for BodySchema/content change; conditional performance for motor/physics hot path |
| Deterministic humanoid training substrate, vector environments, motor-lab and Isaac correspondence | [SPEC-35](35-deterministic-humanoid-training-substrate.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md) | ADR-058, ADR-059, ADR-057, ADR-053, ADR-062, ADR-063, ADR-064 | fast, host-check, persistence-replay, platform, conditional performance, BODY-SCHEMA-P1, PHYS-JOINT-P1, PHYS-SNAPSHOT-P1, MOTOR-SCHEDULE/SAFETY/STATE/ENV-P1, MOTOR-LOCOMOTION-ENV-P1, MODEL-DATAPLANE, MODEL-MIRROR |
| AI agents, perception, memory, deterministic Strategic Agent and LLM/process boundary | [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md) | ADR-005, ADR-056, ADR-046 | play; persistence-replay for state, content-package for authored seeds/affordances, conditional performance for 100-NPC work |
| First-party model-training environments, trajectories, datasets, runs, export, consolidation and accelerated mirrors | [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), plus the lane SPEC | ADR-053, ADR-030, ADR-046, plus the lane ADR | SPEC-35 standing/flat-command reset-step-recorder and mirror use current MOTOR-LOCOMOTION-ENV/MODEL-DATAPLANE/MIRROR gates; all other lanes remain Proposed until a consumer |
| Future R4c/R4d deterministic NPC cognition, beliefs, goals, GOAP, social behavior and systemic economy vertical (Proposed promotion track) | [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | ADR-056, ADR-020, ADR-021, ADR-022, ADR-046 | none until production consumers exist; future fast, play, persistence-replay, content-package, STRATEGIC-* and conditional performance |
| Future learned NPC strategic/tactical behavior policy, Hope-inspired state and training (optional R8 Proposed track) | [SPEC-33](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-06](06-ai-agents-perception-and-memory.md) | ADR-050, ADR-053, ADR-054, ADR-056, ADR-046 | none until an optional R8 production consumer exists; future per-role BEHAVIOR-* plus applicable MODEL-* checks |
| Future learned Motor System profiles: learned humanoid MVP, adaptation, motion generation, equipment/injury/weapon/parkour and additional policy families (Proposed track) | [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md) | ADR-053, ADR-057, ADR-058 | deterministic substrate is current under SPEC-35; learned quality/export and advanced families remain future MOTOR/POLICY/ANIM-HYBRID + applicable MODEL-* gates |
| Dialogue, model packs (Proposed track) | [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md) | ADR-017 | play |
| Current RPG domain and quests | [SPEC-19](19-rpg-domain-and-narrative-state.md) | ADR-020, ADR-046 | play |
| Current bounded four-region/64-chunk partition, pinned generation and paired fixed-stage commit | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | ADR-026, ADR-048, ADR-051 | play, content-package, persistence-replay, performance smoke + r3-multiregion-streaming |
| R4a derived calendar and authored relay-keeper routine (Proposed promotion track) | [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-29](29-platform-host-and-application-session.md) | ADR-008, ADR-016, ADR-019, ADR-021 (Accepted ownership/determinism invariants; retired schemas are historical), ADR-022, ADR-025, ADR-030, ADR-034, ADR-046, ADR-047, ADR-048, ADR-051, ADR-052 | none until the production consumer exists; promotion runs fast, play, persistence-replay, content-package and conditional performance smoke/report-only |
| Future R4b population tiers, placement/transfer and 100-NPC workload (Proposed track) | [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | ADR-016, ADR-021 (Accepted invariants only), ADR-046 | none until exact production population/navigation consumers exist |
| Future narrative director or divine-standing proposal | [SPEC-31](31-autonomous-quest-lifecycle-and-narrative-director.md) | ADR-029/ADR-031 (Superseded; historical context), ADR-046 | none until a production consumer exists |
| Luau/Wasm scripting, plugins, mod packages, gameplay mechanics authoring | [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | ADR-008, ADR-014 | content-package |
| Player interaction, UI, camera, localization, accessibility | [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md) | ADR-019, ADR-044 | play |
| Project composition, exact lock and atomic activation | [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md) | ADR-018, ADR-048 | fast, content-package |
| Platform host, application session, presentation authority, Save/Load/close | [SPEC-29](29-platform-host-and-application-session.md) | ADR-028, ADR-035, ADR-047 | play, persistence-replay, platform |
| Current performance budgets and evidence | [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md) for R5 | ADR-016, ADR-036, ADR-038, ADR-045, ADR-049, ADR-060, ADR-061, ADR-062, ADR-063 | performance |
| Future generic scheduler/resource framework after bounded R3 (Proposed track) | [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md) | ADR-046, ADR-051 (scope exclusion) | none until a second production consumer exists |
| Tooling, SDK, observability | [SPEC-09](09-tooling-sdk-and-observability.md) | — | fast |
| Gothic importer boundary, neutral artifacts | [SPEC-10](10-gothic-importer-boundary.md) | ADR-001 | content-package |
| Security, licensing, governance, secrets, provenance | [SPEC-11](11-security-licensing-and-governance.md) | ADR-001 | fast |
| Audio and current bounded world services | [SPEC-08](08-audio-navigation-and-world-services.md) | — | play |
| Future navigation query/cook/100-NPC path (Proposed track) | [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | ADR-016, ADR-046 | none until a production navigation consumer exists |
| Roadmap: scope, stage, blocker, exit criterion, очередь работ | [roadmap](../roadmap.md) + SPEC затронутой подсистемы | по строке подсистемы | по строке подсистемы |

## Historical-only — не authority

ADR-004, ADR-006, ADR-007, ADR-009, ADR-010, ADR-012, ADR-015, ADR-023, ADR-024,
ADR-029, ADR-031, ADR-033, ADR-039, ADR-040, ADR-041, ADR-042, ADR-043
и ADR-055
полностью superseded; [evidence register](evidence-register.md) и review
packets 1.0–1.9 (`docs/reviews/`) — historical snapshots. Их MAY читать как
контекст, но они не разрешают и не блокируют изменения.

## Proposed — не shipped

SPEC-16/ADR-017 (dialogue model packs), SPEC-20/ADR-052 (R4a derived calendar
and authored routine), the navigation candidate in SPEC-08, SPEC-23
(future generic jobs/resource work after bounded R3), SPEC-31 (narrative/divine intent formerly in
ADR-029/ADR-031), SPEC-32 (future deterministic R4c/R4d consumer schemas),
SPEC-33/SPEC-34 and ADR-050/ADR-053/ADR-054 (optional R8 learned
strategic/tactical NPC behavior and training data plane), exact unconsumed
learned-Motor profiles beyond the SPEC-35 substrate under
ADR-057/SPEC-14/27/28/34 — Proposed.
Не представлять как реализованное; при работе рядом указывать fallback и
bounded evaluation path. `docs/plans/` и `docs/development/` — рабочие
материалы и research notes, не normative architecture.

## Поддержка этого файла

Обновлять routing-таблицу в том же change, который добавляет, supersede или
меняет статус SPEC/ADR либо меняет product checks. Редакционная правка
(ссылки, статусы) нового ADR не требует.
