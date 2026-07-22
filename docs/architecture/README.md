# Next Engine: нормативный пакет архитектурных спецификаций

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Accepted |
| Версия | 1.4 |
| Владелец | Architecture Working Group |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | отсутствуют |
| Заменяет | отсутствует |

Этот каталог задаёт архитектурную baseline независимого AI-first RPG-движка, временно называемого **Next Engine**. Он не описывает перенос OpenGothic и не меняет контракты действующего C++20 runtime. Документы написаны так, чтобы после review пакет можно было перенести без смысловых изменений в отдельный engine monorepo.

Architecture packet version 1.4 объединяет перечисленные в индексе версии RFC/ADR в один review set. Статус `Accepted` фиксирует decision baseline, достаточную для standalone Mac-first bootstrap; он **не** утверждает, что ещё не созданная implementation прошла gates SPEC-12. Такой результат называется отдельно `vertical-v1 implementation conformance`.

## Нормативный язык

Ключевые слова **MUST**, **MUST NOT**, **SHOULD**, **SHOULD NOT** и **MAY** имеют нормативный смысл RFC 2119/8174 только когда написаны заглавными буквами:

- **MUST / MUST NOT** — обязательное условие conformance;
- **SHOULD / SHOULD NOT** — рекомендуемое условие, отступление требует записанного обоснования и компенсирующей проверки;
- **MAY** — допустимый, но необязательный вариант.

При конфликте действует следующий приоритет: более новый Accepted ADR, явно заменяющий старое решение → `12-vertical-slice-conformance.md` для release gates → профильный subsystem RFC → product contract → glossary. Evidence register сообщает факты, но сам по себе не принимает архитектурных решений.

## Статусы и изменение решений

Для RFC и ADR разрешены статусы `Draft`, `Proposed`, `Accepted`, `Rejected`, `Superseded`. Для внешней технологии дополнительно используется только один из тех же четырёх решающих статусов: `Accepted`, `Proposed`, `Rejected` или `Superseded`.

- `Draft` не входит в baseline.
- `Proposed` требует измеримого gate и заранее указанного fallback.
- `Accepted` обязателен для conforming implementation.
- `Rejected` хранит отрицательное решение и причину.
- `Superseded` содержит ссылку на заменяющий документ.

Accepted ADR не редактируется так, чтобы изменить смысл решения. Изменение оформляется новым ADR с полем `Заменяет`, затем синхронно обновляются зависимые RFC, evidence register и traceability. Редакционные исправления без изменения смысла разрешены с увеличением patch-версии.

Каждый RFC MUST содержать ID, статус, версию, владельца, дату проверки, нормативные зависимости и сведения о supersession. Каждый `Proposed` backend MUST иметь владельца gate, команду или воспроизводимый сценарий, числовой pass/fail threshold, перечень evidence artifacts и fallback.

## Порядок чтения

1. [Product contract](00-product-contract.md) и [system architecture](01-system-architecture.md).
2. [Glossary](glossary.md) — единые имена контрактов и идентификаторов.
3. RFC 02–11 в порядке индекса ниже.
4. [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md).
5. [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md).
6. [Headless testing, agent validation и human evidence](15-headless-testing-agent-validation-and-human-evidence.md).
7. [Vertical-slice conformance](12-vertical-slice-conformance.md).
8. [Traceability](traceability.md), [evidence register](evidence-register.md) и [ADR](adr/000-template.md).

## Индекс нормативных документов

| ID | Документ | Статус | Владелец | Нормативные зависимости |
|---|---|---|---|---|
| SPEC-00 | [Продуктовый контракт](00-product-contract.md) | Accepted | Product Architecture | INDEX-001, ADR-001, ADR-008, ADR-009, ADR-010, ADR-011 |
| SPEC-01 | [Системная архитектура](01-system-architecture.md) | Accepted | Core Architecture | SPEC-00, SPEC-14, SPEC-15, ADR-001, ADR-002, ADR-005, ADR-008, ADR-009, ADR-010, ADR-011 |
| SPEC-02 | [Runtime, ECS и модель данных](02-runtime-ecs-and-data.md) | Accepted | Runtime Team | SPEC-01, SPEC-15, ADR-002, ADR-007 |
| SPEC-03 | [Assets, streaming и persistence](03-assets-world-streaming-and-persistence.md) | Accepted | Asset & Persistence Team | SPEC-02, SPEC-14, SPEC-15, ADR-007 |
| SPEC-04 | [Rendering и platform](04-rendering-and-platform.md) | Accepted | Rendering Team | SPEC-01, SPEC-03, SPEC-15, ADR-003, ADR-010 |
| SPEC-05 | [Physics, animation и motor control](05-physics-animation-and-motor-control.md) | Accepted | Physical Embodiment Team | SPEC-02, SPEC-14, SPEC-15, ADR-004, ADR-009 |
| SPEC-06 | [AI agents, perception и memory](06-ai-agents-perception-and-memory.md) | Accepted | Agent Intelligence Team | SPEC-02, SPEC-13, SPEC-14, ADR-005 |
| SPEC-07 | [RPG, scripting и plugins](07-rpg-scripting-and-plugins.md) | Accepted | RPG Framework Team | SPEC-02, SPEC-06, SPEC-14, ADR-006 |
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted | World Services Team | SPEC-02, SPEC-05, SPEC-15 |
| SPEC-09 | [Tooling, SDK и observability](09-tooling-sdk-and-observability.md) | Accepted | Developer Experience Team | SPEC-02, SPEC-03, SPEC-13, SPEC-14, SPEC-15, ADR-010, ADR-011 |
| SPEC-10 | [Граница Gothic importer](10-gothic-importer-boundary.md) | Accepted | Importer Team | SPEC-03, ADR-001, ADR-007, ADR-011 |
| SPEC-11 | [Security, licensing и governance](11-security-licensing-and-governance.md) | Accepted | Security & Governance Team | все SPEC, ADR-001, ADR-006, ADR-008, ADR-009, ADR-010 |
| SPEC-12 | [Vertical-slice conformance](12-vertical-slice-conformance.md) | Accepted | Release Engineering | SPEC-00…SPEC-11, SPEC-13, SPEC-14, SPEC-15, ADR-011 |
| SPEC-13 | [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Accepted | Gameplay Extensibility Team | SPEC-02, SPEC-03, SPEC-05, SPEC-07, SPEC-09, SPEC-11, SPEC-14, SPEC-15, ADR-008, ADR-010 |
| SPEC-14 | [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Accepted | Physical Embodiment Team | SPEC-03, SPEC-05, SPEC-06, SPEC-07, SPEC-09, SPEC-13, SPEC-15, ADR-009, ADR-010, ADR-011 |
| SPEC-15 | [Headless testing, agent validation и human evidence](15-headless-testing-agent-validation-and-human-evidence.md) | Accepted | Verification & Evidence Team | SPEC-01, SPEC-02, SPEC-03, SPEC-04, SPEC-09, SPEC-11, SPEC-13, ADR-010 |
| GLOSSARY-001 | [Глоссарий](glossary.md) | Accepted | Architecture Working Group | INDEX-001 |
| EVIDENCE-001 | [Реестр доказательств](evidence-register.md) | Accepted | Architecture Working Group | профильные ADR |
| TRACE-001 | [Матрица трассируемости](traceability.md) | Accepted | Release Engineering | SPEC-12 |

## Индекс ADR

| ID | Решение | Статус |
|---|---|---|
| ADR-000 | [Шаблон ADR](adr/000-template.md) | Accepted |
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
| ADR-011 | [macOS developer host, local verification и staged training capability](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted |

## Review и перенос в отдельный репозиторий

Architecture baseline принимается только единым review-пакетом. Частичное одобрение отдельных файлов не делает архитектуру принятой. Initial packet version 1.0 был расширен version 1.1 через ADR-008/SPEC-13, version 1.2 — через ADR-009/SPEC-14, version 1.3 — через ADR-010/SPEC-15, а version 1.4 — через ADR-011 и синхронное уточнение platform/training/importer/traceability contracts. Packet 1.4 содержит 32 Markdown-документа, 16 subsystem SPEC, 15 vertical gates, 78 requirements, 24 failure paths и 23 technology rows. Любое последующее изменение подчиняется ADR/supersession rules. Review MUST подтвердить отсутствие orphan-документов, нерешённых архитектурных вопросов, vendor-типов в публичных контрактах и нетрассируемых vertical-slice требований.

При создании самостоятельного репозитория каталог MUST быть перенесён в `docs/architecture/` с сохранением ID, истории ADR и относительных ссылок. Изменение путей MAY быть отдельным механическим commit; изменение решений в том же commit запрещено. После успешного local migration новый repository становится source of truth, а эта копия остаётся frozen historical source до появления reviewed permanent remote URL.
