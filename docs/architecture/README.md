# Next Engine: нормативный пакет архитектурных спецификаций

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Proposed |
| Версия | 1.5 |
| Владелец | Architecture Working Group |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | отсутствуют |
| Заменяет | Packet 1.4 после human approval exact candidate hash |

Этот каталог задаёт архитектуру независимого AI-first RPG engine. Git baseline Packet 1.4 остаётся последним Accepted packet. Текущее содержимое — **Packet 1.5 candidate**: его новые решения и семантически изменённые SPEC имеют статус `Proposed` и не становятся baseline до одобрения уполномоченным человеком exact changeset hash.

Packet 1.5 закрывает локальную нормативную authority, canonical command ordering, baseline/attestation lifecycle, package trust, FFI exceptions, integrated performance, governance enforcement и artifact-first AI-assisted content generation. Он не утверждает implementation conformance, shipping readiness или human approval.

## Нормативный язык и приоритет

**MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT** и **MAY** имеют смысл RFC 2119/8174 только в верхнем регистре.

При конфликте действует: новый Accepted superseding ADR → SPEC-12 release gates → owning subsystem SPEC → SPEC-00 → glossary. Proposed ADR не заменяет Accepted решение. Evidence register сообщает состояние технологии, но не создаёт решение.

## Authority и граф зависимостей

Активный граф обязан быть локальным и ациклическим:

`INDEX/GLOSSARY → ADR → SPEC-00/01 → SPEC-02…11 → SPEC-15 → SPEC-13 → SPEC-16 → SPEC-14 → SPEC-12 → TRACE/EVIDENCE`.

Поле `Нормативные зависимости` содержит только локальные более ранние authority. Обратные, информационные и provenance-ссылки находятся в `Связанные документы` или обычном тексте и не создают нормативное ребро. Внешний материал становится normative только после exact frozen migration с hash/provenance/license.

## Статусы и supersession

- `Draft` и `Proposed` не входят в Accepted baseline.
- `Proposed` требует owner, воспроизводимый gate, threshold, evidence и fallback.
- `Accepted` обязателен для conforming implementation.
- `Rejected` и `Superseded` сохраняют историю решения.
- Accepted ADR не меняет смысл; замена создаёт новый ADR. После approval старый ADR получает `Superseded` и двустороннюю ссылку.
- Агент может подготовить candidate и evidence, но не может подписать review, принять packet или повысить baseline.

## Порядок чтения

1. [Product contract](00-product-contract.md), [system architecture](01-system-architecture.md), [glossary](glossary.md).
2. Активные Accepted ADR и Proposed ADR текущего candidate.
3. SPEC-02…11 в порядке индекса.
4. [Headless testing и evidence](15-headless-testing-agent-validation-and-human-evidence.md).
5. [Mechanics/packages](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [AI content generation](16-ai-assisted-world-and-asset-generation.md) и [physical policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md).
6. [Vertical conformance](12-vertical-slice-conformance.md), затем [traceability](traceability.md) и [evidence register](evidence-register.md).

## Индекс нормативных документов

| ID | Документ | Статус | Владелец | Нормативные зависимости |
|---|---|---|---|---|
| SPEC-00 | [Продуктовый контракт](00-product-contract.md) | Proposed | Product Architecture | INDEX-001, ADR-001, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012, ADR-017 |
| SPEC-01 | [Системная архитектура](01-system-architecture.md) | Proposed | Core Architecture | SPEC-00, ADR-001, ADR-002, ADR-005, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012, ADR-016, ADR-017 |
| SPEC-02 | [Runtime, ECS и модель данных](02-runtime-ecs-and-data.md) | Proposed | Runtime Team | SPEC-01, ADR-007, ADR-013, ADR-016 |
| SPEC-03 | [Assets, streaming и persistence](03-assets-world-streaming-and-persistence.md) | Accepted | Asset & Persistence Team | SPEC-02, ADR-007 |
| SPEC-04 | [Rendering и platform](04-rendering-and-platform.md) | Accepted | Rendering Team | SPEC-01, SPEC-03, ADR-003 |
| SPEC-05 | [Physics, animation и motor control](05-physics-animation-and-motor-control.md) | Accepted | Physical Embodiment Team | SPEC-02, ADR-004, ADR-009, RESEARCH-001 |
| SPEC-06 | [AI agents, perception и memory](06-ai-agents-perception-and-memory.md) | Accepted | Agent Intelligence Team | SPEC-02, ADR-005 |
| SPEC-07 | [RPG, scripting и plugins](07-rpg-scripting-and-plugins.md) | Proposed | RPG Framework Team | SPEC-02, SPEC-06, ADR-015 |
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted | World Services Team | SPEC-02, SPEC-05 |
| SPEC-09 | [Tooling, SDK и observability](09-tooling-sdk-and-observability.md) | Proposed | Developer Experience Team | SPEC-02, SPEC-03, ADR-011, ADR-014 |
| SPEC-10 | [Граница Gothic importer](10-gothic-importer-boundary.md) | Accepted | Importer Team | SPEC-03, ADR-001, ADR-007, ADR-011 |
| SPEC-11 | [Security, licensing и governance](11-security-licensing-and-governance.md) | Proposed | Security & Governance Team | SPEC-00/01/07/09, ADR-001/008/009/014/015/016/017 |
| SPEC-15 | [Headless testing, agent validation и human evidence](15-headless-testing-agent-validation-and-human-evidence.md) | Proposed | Verification & Evidence Team | SPEC-01…11, ADR-014 |
| SPEC-13 | [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Proposed | Gameplay Extensibility Team | SPEC-02/03/05/07/09/11/15, ADR-008/014/015 |
| SPEC-16 | [AI-assisted world and asset generation](16-ai-assisted-world-and-asset-generation.md) | Proposed | Asset & Persistence Team | SPEC-01/03/09/11/15/13, ADR-017 |
| SPEC-14 | [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Proposed | Physical Embodiment Team | SPEC-03/05/06/07/09/11/13/15, ADR-009/011/014 |
| SPEC-12 | [Vertical-slice conformance](12-vertical-slice-conformance.md) | Proposed | Release Engineering | SPEC-00…11/13/14/15/16, ADR-011…017 |
| GLOSSARY-001 | [Глоссарий](glossary.md) | Proposed | Architecture Working Group | INDEX-001 |
| TRACE-001 | [Матрица трассируемости](traceability.md) | Proposed | Release Engineering | SPEC-12 |
| EVIDENCE-001 | [Реестр доказательств](evidence-register.md) | Proposed | Architecture Working Group | активные и candidate ADR |

## Индекс ADR-решений

| ID | Решение | Статус |
|---|---|---|
| ADR-001 | [Продукт, репозитории, лицензия и платформы](adr/001-product-repository-license-and-platforms.md) | Accepted |
| ADR-002 | [Rust-first core, FFI и ECS facade](adr/002-rust-first-ffi-and-ecs-facade.md) | Accepted |
| ADR-003 | [Vulkan renderer и compiler-neutral shaders](adr/003-vulkan-renderer-and-shader-toolchain.md) | Accepted |
| ADR-004 | [Physics-authoritative avatars и replaceable backend](adr/004-physics-avatar-backend-boundary.md) | Accepted |
| ADR-005 | [Offline-first AI и process boundary](adr/005-offline-first-ai-process-boundary.md) | Accepted |
| ADR-006 | [Luau gameplay и Wasm plugins](adr/006-scripting-and-plugin-model.md) | Accepted |
| ADR-007 | [Stable IDs, persistence и replay](adr/007-identities-persistence-and-replay.md) | Accepted |
| ADR-008 | [Mechanics/mod package и agent-ready authoring](adr/008-mechanics-mod-package-and-agent-authoring-model.md) | Accepted |
| ADR-009 | [Pretrained foundation policies и progressive motor skills](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md) | Accepted |
| ADR-010 | [Artifact-first headless validation и human review](adr/010-artifact-first-headless-validation-and-review.md) | Accepted |
| ADR-011 | [macOS developer host и staged training capability](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted |
| ADR-012 | [Standalone authority и ациклические зависимости](adr/012-standalone-authority-and-acyclic-dependencies.md) | Proposed |
| ADR-013 | [Canonical WorldCommand ordering](adr/013-canonical-world-command-ordering.md) | Proposed |
| ADR-014 | [Artifact-first review, baselines и attestation v2](adr/014-artifact-first-review-baselines-and-attestation-v2.md) | Proposed |
| ADR-015 | [Luau/Wasm package trust v2](adr/015-luau-wasm-package-trust-v2.md) | Proposed |
| ADR-016 | [Rust-first audited FFI boundary v2](adr/016-rust-first-audited-ffi-boundary-v2.md) | Proposed |
| ADR-017 | [Artifact-first AI-assisted content generation](adr/017-artifact-first-ai-content-generation.md) | Proposed |

## Ненормативный template и frozen annex

- Template: [ADR-NNN](adr/000-template.md), не является ADR-решением.
- `RESEARCH-001`: [Physical Avatar Research Specification](research/physical-avatar-research-spec.md), exact frozen external annex. Нормативное применение ограничено engine-owned boundary ADR-004/SPEC-05; OpenGothic-specific statements не становятся runtime contract. Provenance и hash находятся в [MIGRATION_PROVENANCE](../../MIGRATION_PROVENANCE.md).

## Packet summary

| Metric | Value |
|---|---:|
| Markdown documents | 40 |
| Subsystem SPEC files | 17 |
| Decision ADR files | 17 |
| Vertical gates | 15 |
| Requirements | 95 |
| Failure paths | 40 |
| Technology rows | 28 |
| Proposed technology rows | 19 |

Packet 1.5 candidate не принят, пока exact changeset hash не прошёл human architecture review. При отсутствии reviewer capability результат остаётся `Proposed/AwaitingCapability`; Accepted Packet 1.4 не переписывается. После одобрения promotion changeset одновременно меняет candidate document statuses, ADR supersession backlinks и packet baseline declaration.
