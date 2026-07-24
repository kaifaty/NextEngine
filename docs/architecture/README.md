# Next Engine: нормативный пакет архитектурных спецификаций

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Accepted |
| Версия | 1.8 |
| Review candidate | 1.9 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | отсутствуют |
| Заменяет | отсутствует |

Этот каталог задаёт архитектурную baseline независимого AI-first RPG-движка, временно называемого **Next Engine**. Он не описывает перенос OpenGothic и не меняет контракты действующего C++20 runtime. Документы написаны так, чтобы после review пакет можно было перенести без смысловых изменений в отдельный engine monorepo.

Architecture packet version 1.8 объединяет перечисленные в индексе версии SPEC/ADR в один review set. До отдельного approval exact root в будущем [ARCH-REVIEW-1.8](../reviews/architecture/packet-1.8.md) это candidate, а admitted baseline остаётся packet 1.7. Статус `Accepted` внутри candidate фиксирует proposed target state; он **не** утверждает, что implementation прошла gates SPEC-12. Такой результат называется отдельно `vertical-v1 implementation conformance`.

Packet 1.5 атомарно принял ADR-012…ADR-016, superseded ADR-004/006/007/010 и синхронизировал command identity/encoding, self-contained physical boundary, extension trust, evidence attestation, compositional budgets, SPEC, glossary, evidence register и traceability. Это architecture promotion; оно не создаёт runtime implementation, reviewer credentials, gate PASS или release claim.

Packet 1.5.1 является admitted редакционным patch к Accepted packet 1.5. Он нормализует bootstrap authority labels `Product Architecture` и `Core Architecture` до единого `Repository Owner` там, где речь идёт об изменении документа, review или архитектурном gate ownership. Patch не передаёт Repository Owner mutable subsystem state, не меняет public/runtime semantics, requirements, failure paths, vertical gates, technology status и не принимает Proposed tracks.

Admitted packet 1.6 перепрофилировал dialogue promotion в P0 integrity remediation. ADR-022 заменил самоссылочную V1 command identity и неполный ledger; SPEC-21 зафиксировал clock, RNG, schedule, ordered work и numeric substrate; ADR-023 ввёл подписываемые `Approve|Reject|NeedsChanges` и V2 attestation; ADR-024 замкнул requirement→gate→evidence→VS/profile graph; SPEC-07 синхронизирован с exact 1 800-tick breaker; `PERF-01` стал blocking child VS-12. Это architecture admission, не runtime implementation или gate PASS.

Dialogue/model track SPEC-16/ADR-017 остаётся **Deferred Proposed**. Его `REQ-079`…`REQ-086` и `FAIL-025`…`FAIL-030` навсегда reserved и никогда не переиспользуются. RESEARCH-002 остаётся ненормативным `Draft`; model rows остаются `Proposed`.

Admitted packet 1.7 атомарно перевёл foundation-completeness track SPEC-17…SPEC-20 и ADR-018…ADR-021 в `Accepted`, не завися от решения по SPEC-16. Его exact allocations — `REQ-087`…`REQ-102` и `FAIL-031`…`FAIL-038`; approved exact root — `a8c8f8820cafb135f6e9e513d20fcbe38fd175352f5808965e5ddd9c442855ba`.

Consolidated candidate packet 1.8 добавляет весь оставшийся decision-complete P0 substrate: SPEC-22…25 schema/jobs/content/world, SPEC-26…28 physics/motor/animation, SPEC-29/30 platform/session/presentation и decisions ADR-025…028. Его exact allocations — `REQ-112`…`REQ-147` и `FAIL-044`…`FAIL-061`; он не принимает ни одной external technology, не создаёт implementation gate `PASS` и заменяет три промежуточных human admission-точки одной final exact-root decision.

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
9. Для integrity remediation packet 1.6 — [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) и [ADR-024](adr/024-requirement-gate-evidence-and-profile-closure.md). Их требования являются implementation contracts, а не заявлением runtime/gate PASS.
10. Для Deferred Proposed dialogue/model track — [RESEARCH-002](research/npc-dialogue-model-landscape.md), [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md) и [ADR-017](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md).
11. Для packet-1.7 foundation completeness — [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), ADR-018, ADR-019, ADR-020 и ADR-021.
12. Для consolidated packet-1.8 P0 closure — [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-29](29-platform-host-and-application-session.md), [SPEC-30](30-presentation-extraction-and-render-content.md) и ADR-025…ADR-028.

## Индекс нормативных документов

| ID | Документ | Статус | Владелец | Нормативные зависимости |
|---|---|---|---|---|
| SPEC-00 | [Продуктовый контракт](00-product-contract.md) | Accepted | Repository Owner | INDEX-001, ADR-001, ADR-008, ADR-009, ADR-011, ADR-023 |
| SPEC-01 | [Системная архитектура](01-system-architecture.md) | Accepted | Repository Owner | SPEC-00, SPEC-14, SPEC-15, ADR-001, ADR-002, ADR-005, ADR-008, ADR-009, ADR-011, ADR-023 |
| SPEC-02 | [Runtime, ECS и модель данных](02-runtime-ecs-and-data.md) | Accepted | Repository Owner | SPEC-01, SPEC-15, ADR-002, ADR-022 |
| SPEC-03 | [Assets, streaming и persistence](03-assets-world-streaming-and-persistence.md) | Accepted | Repository Owner | SPEC-02, SPEC-14, SPEC-15, ADR-022 |
| SPEC-04 | [Rendering и platform](04-rendering-and-platform.md) | Accepted | Repository Owner | SPEC-01, SPEC-03, SPEC-15, ADR-003, ADR-023 |
| SPEC-05 | [Physics, animation и motor control](05-physics-animation-and-motor-control.md) | Accepted | Repository Owner | SPEC-02, SPEC-14, SPEC-15, ADR-009, ADR-013 |
| SPEC-06 | [AI agents, perception и memory](06-ai-agents-perception-and-memory.md) | Accepted | Repository Owner | SPEC-02, SPEC-13, SPEC-14, ADR-005, ADR-016 |
| SPEC-07 | [RPG, scripting и plugins](07-rpg-scripting-and-plugins.md) | Accepted | Repository Owner | SPEC-02, SPEC-06, SPEC-14, ADR-014 |
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted | Repository Owner | SPEC-02, SPEC-05, SPEC-15, ADR-016 |
| SPEC-09 | [Tooling, SDK и observability](09-tooling-sdk-and-observability.md) | Accepted | Repository Owner | SPEC-02, SPEC-03, SPEC-13, SPEC-14, SPEC-15, ADR-011, ADR-023, ADR-024 |
| SPEC-10 | [Граница Gothic importer](10-gothic-importer-boundary.md) | Accepted | Repository Owner | SPEC-03, ADR-001, ADR-011, ADR-022, ADR-023 |
| SPEC-11 | [Security, licensing и governance](11-security-licensing-and-governance.md) | Accepted | Repository Owner | SPEC-00, SPEC-01, SPEC-13, SPEC-14, SPEC-15, ADR-001, ADR-008, ADR-009, ADR-014, ADR-023, ADR-024 |
| SPEC-12 | [Vertical-slice conformance](12-vertical-slice-conformance.md) | Accepted | Repository Owner | SPEC-00…SPEC-11, SPEC-13, SPEC-14, SPEC-15, SPEC-17…SPEC-30, ADR-011, ADR-016, ADR-018…ADR-028 |
| SPEC-13 | [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Accepted | Repository Owner | SPEC-02, SPEC-03, SPEC-05, SPEC-07, SPEC-09, SPEC-11, SPEC-14, SPEC-15, ADR-008, ADR-014, ADR-016, ADR-023 |
| SPEC-14 | [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Accepted | Repository Owner | SPEC-03, SPEC-05, SPEC-06, SPEC-07, SPEC-09, SPEC-13, SPEC-15, ADR-009, ADR-011, ADR-023 |
| SPEC-15 | [Headless testing, agent validation и human evidence](15-headless-testing-agent-validation-and-human-evidence.md) | Accepted | Repository Owner | SPEC-01, SPEC-02, SPEC-03, SPEC-04, SPEC-09, SPEC-11, SPEC-13, ADR-023, ADR-024 |
| SPEC-16 | [Text-canonical multimodal dialogue и model packs](16-text-canonical-multimodal-dialogue-and-model-packs.md) | Proposed | Repository Owner | SPEC-01, SPEC-03, SPEC-06, SPEC-07, SPEC-08, SPEC-09, SPEC-11, SPEC-12, SPEC-15, ADR-005, ADR-022, ADR-023 |
| SPEC-17 | [Project composition, configuration и application lifecycle](17-project-composition-configuration-and-application-lifecycle.md) | Accepted | Repository Owner | SPEC-00, SPEC-01, SPEC-02, SPEC-03, SPEC-07, SPEC-09, SPEC-11, SPEC-15, ADR-002, ADR-011, ADR-014, ADR-016, ADR-018, ADR-022, ADR-023 |
| SPEC-18 | [Player interaction, UI, camera, localization и accessibility](18-player-interaction-ui-camera-localization-and-accessibility.md) | Accepted | Repository Owner | SPEC-00, SPEC-01, SPEC-02, SPEC-04, SPEC-07, SPEC-09, SPEC-11, SPEC-15, SPEC-17, ADR-002, ADR-014, ADR-016, ADR-019, ADR-022, ADR-023 |
| SPEC-19 | [RPG domain и narrative state](19-rpg-domain-and-narrative-state.md) | Accepted | Repository Owner | SPEC-00, SPEC-01, SPEC-02, SPEC-03, SPEC-06, SPEC-07, SPEC-08, SPEC-09, SPEC-11, SPEC-13, SPEC-14, SPEC-15, SPEC-17, ADR-008, ADR-014, ADR-016, ADR-020, ADR-022, ADR-023 |
| SPEC-20 | [World simulation и population lifecycle](20-world-simulation-and-population-lifecycle.md) | Accepted | Repository Owner | SPEC-00, SPEC-01, SPEC-02, SPEC-03, SPEC-05, SPEC-06, SPEC-07, SPEC-08, SPEC-09, SPEC-11, SPEC-14, SPEC-15, SPEC-17, SPEC-19, ADR-009, ADR-014, ADR-016, ADR-021, ADR-022, ADR-023 |
| SPEC-21 | [Deterministic runtime primitives, command ledger и causal identity](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | Accepted | Repository Owner | ADR-022 |
| SPEC-22 | [Schema registry, compatibility и migration](22-schema-registry-compatibility-and-migration.md) | Accepted | Asset & Persistence Team | SPEC-00, SPEC-01, SPEC-03, SPEC-17, SPEC-19, SPEC-21, ADR-018, ADR-020, ADR-022, ADR-024, ADR-025 |
| SPEC-23 | [Jobs, memory, compute-resource residency and I/O backpressure](23-jobs-memory-resource-residency-and-io-backpressure.md) | Accepted | Runtime Team | SPEC-01, SPEC-02, SPEC-03, SPEC-17, SPEC-20, SPEC-21, ADR-016, ADR-021, ADR-022, ADR-024, ADR-026 |
| SPEC-24 | [Content catalog, bundle и neutral asset schemas](24-content-catalog-bundle-and-neutral-asset-schemas.md) | Accepted | Asset & Persistence Team | SPEC-00, SPEC-01, SPEC-03, SPEC-04, SPEC-05, SPEC-08, SPEC-10, SPEC-17, SPEC-21, SPEC-22, ADR-018, ADR-022, ADR-024, ADR-025 |
| SPEC-25 | [World partition, streaming admission and persistent spatial objects](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | Accepted | World Services Team | SPEC-00, SPEC-01, SPEC-02, SPEC-03, SPEC-08, SPEC-17, SPEC-20, SPEC-21, SPEC-23, SPEC-24, ADR-021, ADR-022, ADR-024, ADR-026 |
| SPEC-26 | [Physics world, collision, constraints, queries and canonical snapshots](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) | Accepted | Physical Embodiment Team | SPEC-00, SPEC-01, SPEC-02, SPEC-03, SPEC-05, SPEC-14, SPEC-15, SPEC-17, SPEC-21, SPEC-22, SPEC-23, SPEC-24, ADR-013, ADR-018, ADR-022, ADR-024, ADR-025, ADR-026, ADR-027 |
| SPEC-27 | [Motor observation, action and deterministic inference](27-motor-observation-action-and-deterministic-inference.md) | Accepted | Physical Embodiment Team | SPEC-00, SPEC-01, SPEC-02, SPEC-05, SPEC-06, SPEC-14, SPEC-15, SPEC-17, SPEC-21, SPEC-26, ADR-009, ADR-016, ADR-024, ADR-027 |
| SPEC-28 | [Skeletal animation, retargeting and IK](28-skeletal-animation-retargeting-and-ik.md) | Accepted | Physical Embodiment Team | SPEC-00, SPEC-01, SPEC-03, SPEC-04, SPEC-05, SPEC-14, SPEC-15, SPEC-17, SPEC-18, SPEC-21, SPEC-24, SPEC-26, ADR-022, ADR-023, ADR-024, ADR-027 |
| SPEC-29 | [Platform host and application session](29-platform-host-and-application-session.md) | Accepted | Runtime Team | SPEC-00, SPEC-01, SPEC-02, SPEC-04, SPEC-15, SPEC-17, SPEC-18, SPEC-21, ADR-018, ADR-019, ADR-022, ADR-023, ADR-024, ADR-028 |
| SPEC-30 | [Presentation extraction and render content](30-presentation-extraction-and-render-content.md) | Accepted | Rendering Team | SPEC-00, SPEC-01, SPEC-03, SPEC-04, SPEC-15, SPEC-17, SPEC-18, SPEC-21, SPEC-24, SPEC-26, SPEC-28, SPEC-29, ADR-019, ADR-022, ADR-023, ADR-024, ADR-027, ADR-028 |
| GLOSSARY-001 | [Глоссарий](glossary.md) | Accepted | Repository Owner | INDEX-001 |
| EVIDENCE-001 | [Реестр доказательств](evidence-register.md) | Accepted | Repository Owner | профильные ADR |
| TRACE-001 | [Матрица трассируемости](traceability.md) | Accepted | Repository Owner | SPEC-12 |

## Индекс ADR

| ID | Решение | Статус |
|---|---|---|
| ADR-000 | [Шаблон ADR](adr/000-template.md) | Accepted |
| ADR-001 | [Продукт, репозитории, лицензия и платформы](adr/001-product-repository-license-and-platforms.md) | Accepted |
| ADR-002 | [Rust-first core, FFI и ECS facade](adr/002-rust-first-ffi-and-ecs-facade.md) | Accepted |
| ADR-003 | [Vulkan renderer и compiler-neutral shaders](adr/003-vulkan-renderer-and-shader-toolchain.md) | Accepted |
| ADR-004 | [Physics-authoritative avatars и replaceable backend](adr/004-physics-avatar-backend-boundary.md) | Superseded |
| ADR-005 | [Offline-first AI и process boundary](adr/005-offline-first-ai-process-boundary.md) | Accepted |
| ADR-006 | [Luau gameplay и Wasm plugins](adr/006-scripting-and-plugin-model.md) | Superseded |
| ADR-007 | [Stable IDs, persistence и replay](adr/007-identities-persistence-and-replay.md) | Superseded |
| ADR-008 | [Mechanics/mod package и agent-ready authoring](adr/008-mechanics-mod-package-and-agent-authoring-model.md) | Accepted |
| ADR-009 | [Pretrained foundation policies и progressive motor skills](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md) | Accepted |
| ADR-010 | [Artifact-first headless validation и human review](adr/010-artifact-first-headless-validation-and-review.md) | Superseded |
| ADR-011 | [macOS developer host, local verification и staged training capability](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted |
| ADR-012 | [Deterministic command identity, ordering и replay](adr/012-deterministic-command-identity-and-replay.md) | Superseded |
| ADR-013 | [Self-contained physical-avatar authority boundary](adr/013-self-contained-physical-avatar-boundary.md) | Accepted |
| ADR-014 | [Deterministic extensions и package trust](adr/014-deterministic-extensions-and-package-trust.md) | Accepted |
| ADR-015 | [Evidence trust, fixture separation и offline attestation](adr/015-evidence-trust-fixture-separation-and-attestation.md) | Superseded |
| ADR-016 | [Compositional gameplay budgets](adr/016-compositional-gameplay-budgets.md) | Accepted |
| ADR-017 | [Text-canonical multimodal dialogue и replaceable model packs](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md) | Proposed |
| ADR-018 | [Authoritative project composition и configuration classes](adr/018-authoritative-project-composition-and-configuration.md) | Accepted |
| ADR-019 | [Canonical player actions и presentation authority](adr/019-canonical-player-actions-and-presentation-authority.md) | Accepted |
| ADR-020 | [RPG domain authority и extension boundary](adr/020-rpg-domain-authority-and-extension-boundary.md) | Accepted |
| ADR-021 | [Deterministic population residency и time advance](adr/021-deterministic-population-residency-and-time-advance.md) | Accepted |
| ADR-022 | [Deterministic command identity V2, ledger и causal identity](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) | Accepted |
| ADR-023 | [HumanReviewDecisionV2 и offline attestation](adr/023-human-review-decision-v2-and-offline-attestation.md) | Accepted |
| ADR-024 | [Requirement, gate, evidence и profile closure](adr/024-requirement-gate-evidence-and-profile-closure.md) | Accepted |
| ADR-025 | [Schema, content и migration authority](adr/025-schema-content-and-migration-authority.md) | Accepted |
| ADR-026 | [Deterministic work, compute-resource and streaming admission](adr/026-deterministic-work-resource-and-streaming-admission.md) | Accepted |
| ADR-027 | [Physics, motor and animation authority layering](adr/027-physics-motor-and-animation-layering.md) | Accepted |
| ADR-028 | [Platform session and presentation authority](adr/028-platform-session-and-presentation-authority.md) | Accepted |

## Ненормативные приложения

| ID | Документ | Статус | Назначение |
|---|---|---|---|
| RESEARCH-001 | [Frozen physical-avatar research annex](research/physical-avatar-research-spec.md) | Historical | Проверяемая provenance ADR-013; не является нормативной dependency или Next Engine contract |
| RESEARCH-002 | [NPC dialogue model landscape](research/npc-dialogue-model-landscape.md) | Draft | Current primary-source snapshot; не является technology acceptance или Next Engine contract |

## Packet summary

Summary считает indexed architecture documents без frozen research annex RESEARCH-001. Document/SPEC/ADR/technology counts включают явно помеченные Proposed/Draft additions; requirement и failure counts включают Accepted packet-1.8 candidate rows. Reserved Deferred/future allocations не включаются в Accepted completeness и никогда не переиспользуются.

| Metric | Value |
|---|---:|
| Markdown documents | 65 |
| Subsystem SPEC files | 31 |
| Decision ADR files | 28 |
| Vertical gates | 15 |
| Requirements | 139 |
| Failure paths | 55 |
| Technology rows | 33 |
| Proposed technology rows | 26 |

## Deferred dialogue/model proposal track

Этот track отложен и не входит в packet 1.8. Его decision documents остаются `Proposed`, а allocations permanently reserved. Он состоит из:

- RESEARCH-002 — ненормативный market/Skyrim AI snapshot с Russian-local, multilingual, high-end, remote-opt-in и text-only profiles;
- SPEC-16 — engine-owned `CanonicalUtterance`, turn/stream/model-pack/provider/capability contracts, failures, CLI projections и numerical gates;
- ADR-017 — решение text-as-canonical, per-role replaceability, optional downloadable packs, remote opt-in и replay-without-regeneration;
- десяти `Proposed` model rows в evidence register; cloud services остаются research-only до выбора exact adapter.

Возможная будущая promotion требует нового packet и одной hash-bound decision `architecture.promote` от Repository Owner. `REQ-079`…`REQ-086` и `FAIL-025`…`FAIL-030` уже зарезервированы, не участвуют в Accepted completeness и не могут быть перераспределены даже при окончательном отклонении track. Protocol promotion не переводит model candidate в `Accepted`: exact model files отдельно проходят DIALOGUE/MODEL/license/voice gates. До этого lifecycle — `Deferred Proposed`, а `TextOnlyFallback` остаётся единственным mandatory baseline.

## Packet 1.7 foundation-completeness promotion

Packet 1.7 принял foundation contracts независимо от dialogue/model proposal одним changeset для всех восьми документов:

- SPEC-17/ADR-018 — exact `ProjectManifest → ProjectCompositionLock`, configuration classes и atomic application lifecycle;
- SPEC-18/ADR-019 — Player Experience-owned action mapping, semantic UI, camera targeting boundary, localization и accessibility;
- SPEC-19/ADR-020 — authoritative generic RPG aggregates, atomic domain transactions и fail-closed migrations;
- SPEC-20/ADR-021 — deterministic population schedules, residency tiers, time advance и durable spawn/despawn semantics.

Track имеет fixed Accepted allocations: SPEC-17 `REQ-087`…`REQ-090`/`FAIL-031`…`FAIL-032`, SPEC-18 `REQ-091`…`REQ-094`/`FAIL-033`…`FAIL-034`, SPEC-19 `REQ-095`…`REQ-098`/`FAIL-035`…`FAIL-036`, SPEC-20 `REQ-099`…`REQ-102`/`FAIL-037`…`FAIL-038`. Они не зависят от статуса SPEC-16. Child gates map into existing VS-01…VS-15; VS-16 не создаётся. Ни один external technology row не добавляется.

Exact root `a8c8f8820cafb135f6e9e513d20fcbe38fd175352f5808965e5ddd9c442855ba`
одобрен Repository Owner `Kaifaty`; [ARCH-REVIEW-1.7](../reviews/architecture/packet-1.7.md)
является admitted rollback boundary. Это решение не создало implementation
conformance.

## Consolidated packet 1.8 P0 closure

Packet 1.8 формирует один candidate changeset:

- SPEC-22/ADR-025 — immutable schema/field identity, compatibility classes,
  exact schema-registry manifest, unique migration DAG и full copy-on-write
  publication;
- SPEC-23/ADR-026 — closed job classes, deterministic result admission,
  cancellation trees, memory/residency pins and leases, finite I/O queues,
  archive/decompression limits and no authoritative drop;
- SPEC-24/ADR-025 — `ContentManifestV1`, logical manifest+blob bundle,
  dependency kinds, target variants and neutral scene/mesh/material/texture/
  skeleton/animation/audio/collision/navigation/world schemas;
- SPEC-25/ADR-026 — `WorldPartitionManifestV1`, topology/anchors, canonical
  residency rank, durable placement/tombstones/cross-chunk references and
  `TierExecutionProfile` abstract-equivalence rules;
- SPEC-26/ADR-027 — engine-owned physics world/body/shape/material/joint/query/
  contact/canonical-snapshot contracts, canonical units/order and exact versus
  quantized/tolerance classification;
- SPEC-27/ADR-027 — exact motor observation/action/state schemas, canonical
  inference batching, one safety clamp and deterministic procedural fallback;
- SPEC-28/ADR-027 — skeleton/clip/graph/retarget/root-motion/IK/LOD contracts,
  fixed ordering and explicit physical/presentation authority split;
- SPEC-29/ADR-028 — normalized platform facts, one application-session state
  machine, exactly-once close/save/recovery and displayless root parity;
- SPEC-30/ADR-028 — `PresentationSnapshotV2`, material/shader-interface/
  pipeline-key, exact SDR/optional HDR, VFX cue and reconstructible cache
  contracts.

Fixed Accepted allocations are SPEC-22 `REQ-112`…`REQ-115`/
`FAIL-044`…`FAIL-045`, SPEC-23 `REQ-116`…`REQ-119`/
`FAIL-046`…`FAIL-047`, SPEC-24 `REQ-120`…`REQ-123`/
`FAIL-048`…`FAIL-049`, SPEC-25 `REQ-124`…`REQ-127`/
`FAIL-050`…`FAIL-051`, SPEC-26 `REQ-128`…`REQ-131`/
`FAIL-052`…`FAIL-053`, SPEC-27 `REQ-132`…`REQ-135`/
`FAIL-054`…`FAIL-055`, SPEC-28 `REQ-136`…`REQ-139`/
`FAIL-056`…`FAIL-057`, SPEC-29 `REQ-140`…`REQ-143`/
`FAIL-058`…`FAIL-059`, SPEC-30 `REQ-144`…`REQ-147`/
`FAIL-060`…`FAIL-061`. Child gates map into existing VS-01…VS-15; no new
vertical slice or external technology status is introduced. Only Deferred
SPEC-16 allocations remain reserved.

W3–W5 from the remediation plan are internal review waves inside this one
candidate. This removes intermediate human promotion prompts without weakening
hash binding, automatic validation or final authority. Until automatic checks
and one separately recorded `architecture.promote` bind the exact packet-1.8
root, packet 1.7 remains admitted authority. Agent may prepare candidate and
evidence but cannot create reviewer identity or final approval.

## Deferred P1 backlog

Следующие направления намеренно не входят в packet 1.8 и не получают здесь
requirements, failure IDs, gates, technology status или compatibility promise:

- детальные audio/acoustics contracts и navigation/crowd simulation;
- extension-host ABI, distribution ecosystem и advanced plugin lifecycle;
- release/install/update/rollback profiles;
- detailed importer IPC and NeutralImportModel evolution;
- advanced AI deliberation, perception and memory contracts;
- economy, crafting, social and ecology domain systems;
- full editor, multiplayer, macOS shipping, consoles and mobile.

Это planning backlog, а не нормативная dependency или скрытое acceptance
решение. Каждый пункт требует отдельного owning SPEC/ADR, allocations,
traceability, measurable evidence and a later hash-bound promotion packet.

## Packet 1.5 remediation closure

Эта таблица фиксирует архитектурное закрытие findings в packet 1.5, а не evidence PASS реализации:

| Finding | Нормативное закрытие packet 1.5 | Primary owner | Требуемое gate/failure evidence |
|---|---|---|---|
| 1. Command identity и arrival-independent order | ADR-012: tagged principal, stream ledger, collision policy и полный sort tuple | Runtime Team | `RUNTIME-06`; permutation/collision corpus, ledger, state roots |
| 2. Post-physics same-tick mutation | ADR-012: закрытая `Outcome` phase на stage 9 без re-entry | Runtime Team | `RUNTIME-06`; stage trace и `OUTCOME_REENTRY_FORBIDDEN` |
| 3. Runtime durable identity | ADR-012: causal-command-derived PersistentId, retry/tombstone/collision semantics | Persistence Team | `RUNTIME-07`; spawn vectors, save/reload и collision corpus |
| 4. Canonical bytes, hashes и determinism classes | ADR-012: JCS, `CanonicalBinaryV1`, path/domain/Merkle rules и cross-target classes | Runtime + Persistence Teams | `CANON-01`; Windows/Linux golden vectors |
| 5. Protected/imported fixture leakage | ADR-015: isolated import-smoke, CC0 neutral fixture, cleanup before VS-09 | Importer Team | `PRIVACY-02`; prohibited-root scans и cleanup audit |
| 6. Human evidence trust | ADR-015: Ed25519 envelope, offline trust manifest/revocation и isolated signer | Verification & Evidence Team | `REVIEW-02`; tamper/role/expiry/revocation/agent-key corpus |
| 7. Wall-dependent script outcomes | ADR-014: instruction/fuel/allocation authority, wall trip only `NonConforming` | RPG Framework Team | `SCRIPT-P5`; CPU/load/watchdog permutations |
| 8. Package/plugin signature ambiguity | ADR-014: hash for all, signed trusted distribution, consent-bound unsigned local ceiling | Security & Governance Team | `MOD-P2`; signed/unsigned/invalid-signature matrix |
| 9. External physical normative dependency | ADR-013: complete local ownership/LOD/contact/safety/fallback contract | Physical Embodiment Team | docs-check plus public-contract scan; annex only non-normative |
| 10. Non-compositional performance limits | ADR-016: one 8/12 ms matrix, deterministic 100-NPC cadence и integrated methodology | Release Engineering | `PERF-01`; 1 000 warm-up + 10 000 measured ticks |
| 11. Composite requirement ownership | Promotion manifest below assigns exactly one primary owner and separate contributors | Architecture Working Group | docs-check rejects new composite primary owners |

## Atomic promotion manifest

Packet 1.5 был принят одним hash-bound changeset после automatic checks и единственной recorded decision `architecture.promote` от Repository Owner. Transaction содержит:

1. ADR-004/006/007/010 получили `Superseded` и backlink; ADR-012…016 получили `Accepted`.
2. INDEX/SPEC/glossary/evidence/traceability перешли на packet 1.5; Accepted dependency graph больше не содержит grandfathered external links ADR-004/SPEC-05.
3. SPEC-02/03/10 используют только ADR-012 ordering/identity/encoding; SPEC-05 — ADR-013; SPEC-07/11/13 — ADR-014; SPEC-09/10/11/12/15 — ADR-015; SPEC-06/08/12/13 — ADR-016.
4. REQ-005/011/013/015/019–023/031/032/036/064/072/073/075 и соответствующие FAIL rows получили новые ADR/gate/evidence links без изменения total counts: 78 REQ, 24 FAIL, 15 VS gates, 33 technology rows, из них 26 `Proposed`.
5. Glossary получил `CommandStreamId`, `IssuerPrincipal`, `CanonicalBinaryV1`, `AttestationEnvelope`, `ReviewerTrustManifest`, `GameplayBudgetMatrix`. JCS/Ed25519 libraries не добавлены как technology rows.
6. Traceability заменил `Owner` на `Primary owner` и добавил `Contributors / required approvers`; каждая строка имеет ровно одного primary owner.

| Row | Primary owner | Contributors / required approvers |
|---|---|---|
| REQ-032, FAIL-003 | RPG Framework | Security & Governance |
| REQ-043 | Gameplay Extensibility | Runtime |
| REQ-045, FAIL-009 | Persistence | Gameplay Extensibility |
| REQ-048, FAIL-010 | Developer Experience | Security & Governance |
| REQ-049 | Gameplay Extensibility | Physical Embodiment |
| REQ-051 | Agent Intelligence | Gameplay Extensibility |
| FAIL-007 | Importer | Security & Governance |

Hash-bound record [ARCH-REVIEW-1.5](../reviews/architecture/packet-1.5.md) связывает exact candidate file hashes, automatic checks и одну human promotion decision. Эта запись проверяет closure и структуру bootstrap review, но не заменяет криптографическую проверку личности Repository Owner.

## Packet 1.5.1 editorial ownership closure

Packet 1.5.1 оформляет только bootstrap ownership normalization:

1. authority изменять SPEC/ADR обозначена единым именем `Repository Owner`, согласованным с индексом;
2. subsystem owners, authoritative mutable-state owners, public boundaries, dependency direction и implementation contracts остаются без изменений;
3. Accepted/Proposed/Draft statuses, 78 requirements, 24 failure paths, 15 vertical gates и 33 technology rows не изменяются;
4. dialogue/model track и foundation-completeness track остаются вне patch;
5. patch не создаёт implementation, `vertical-v1`, `PhysicalCertified`, release-readiness или gate-PASS claims.

Hash-bound record [ARCH-REVIEW-1.5.1](../reviews/architecture/packet-1.5.1.md) связывает exact editorial candidate root, automatic checks и отдельную decision `architecture.promote`.

## Packet 1.6 integrity remediation

Packet 1.6 закрыл P0 integrity findings одним admitted boundary:

| Finding | Нормативное закрытие | Primary owner | Gate / evidence |
|---|---|---|---|
| Command-ID self-field, causal identities и finite ledger | ADR-022 + SPEC-21: body without ID, domain-separated V2 ID, world/stream/principal/event identities, high-watermarks и 4 096 receipts | Runtime Team | COMMAND-ID-P1, CAUSAL-ID-P1, COMMAND-LEDGER-P1; V2 golden/collision/restart corpus |
| Clock, RNG, schedule, ordered work и numeric determinism | SPEC-21: exact cutoff, ChaCha12 named streams, stable DAG, ordered reductions/results, checked/fixed-point/quantized profiles | Runtime Team | CLOCK-P1, RNG-P1, SCHEDULE-P1, NUMERIC-P1; cross-platform vectors |
| Human-review enum/envelope conflict | ADR-023: signed `Approve`, `Reject`, `NeedsChanges`; only `Approve` + automatic `PASS` admits; V1 historical-only | Verification & Evidence Team | REVIEW-01, REVIEW-02, SEC-07; payload/envelope/tamper corpus |
| Circuit-breaker conflict | SPEC-07 + ADR-014: third deterministic violation in inclusive 1 800 gameplay ticks; wall watchdog never creates strike/PASS | RPG Framework Team | SCRIPT-P5; ticks 1 799/1 800 and watchdog corpus |
| Orphan/shorthand/status-blind gate mapping | ADR-024: `RequirementGraphV1`, `GateDescriptorV1`, stable reservations, reverse closure and CandidateOnly classification | Verification & Evidence Team | TRACE-01; validator negative corpus |
| Orphan integrated performance gate | ADR-016 + SPEC-12 + traceability: `REQ-111`/`FAIL-043`, `PERF-01` blocking child VS-12 | Release Engineering | PERF-01; independent integrated evidence |

ADR-012 и ADR-015 становятся `Superseded` с backlinks; их V1 artifacts остаются только explicitly historical verification inputs. `ECS-P1`, `RENDER-ASH-P1`, `SHADER-SLANG-P1`, `PLATFORM-P1` и `MCP-P1` классифицированы CandidateOnly и не закрывают Accepted baseline. `RENDER-P1`/`SHADER-P1` остаются implementation-neutral Accepted Vulkan/engine-contract gates. Packet сохраняет 15 vertical gates и не выбирает ECS, binding/compiler, platform либо tooling vendor.

Hash-bound record [ARCH-REVIEW-1.6](../reviews/architecture/packet-1.6.md) содержит approved exact root `0826bd1f005676eec92e19ee70c1d0f176ac8f883fbfc2422027526e30927cd8`; его rollback boundary сохранён, а packet 1.7 позднее admitted отдельно.

## Review и перенос в отдельный репозиторий

Architecture baseline принимается только единым review-пакетом. Частичное одобрение отдельных файлов не делает архитектуру принятой. Initial packet version 1.0 был расширен version 1.1 через ADR-008/SPEC-13, version 1.2 — через ADR-009/SPEC-14, version 1.3 — через ADR-010/SPEC-15, version 1.4 — через ADR-011, version 1.5 — через ADR-012…ADR-016, version 1.5.1 — редакционной нормализацией ownership labels, admitted version 1.6 — SPEC-21/ADR-022…ADR-024 и 9 Accepted requirements/5 failure paths, а admitted version 1.7 — SPEC-17…20/ADR-018…021 и 16 requirements/8 failure paths. Consolidated candidate packet 1.8 добавляет SPEC-22…30/ADR-025…028 и ещё 36 requirements/18 failure paths, сохраняя 15 vertical gates. Current index содержит 65 architecture documents; frozen RESEARCH-001 учитывается отдельно. Любое последующее изменение подчиняется ADR/supersession rules. Review MUST подтвердить отсутствие orphan-документов, нерешённых конфликтов, vendor/OS/ECS/importer типов в публичных контрактах и нетрассируемых requirements.

Packet 1.8 не заявляет `vertical-v1`, `LEGAL-IMPORT-01=PASS`,
`PhysicalCertified`, implementation gate PASS или release readiness. До
Repository Owner approval его exact root admitted authority остаётся packet
1.7; packet 1.9 не формируется до admission 1.8.

При создании самостоятельного репозитория каталог MUST быть перенесён в `docs/architecture/` с сохранением ID, истории ADR и относительных ссылок. Изменение путей MAY быть отдельным механическим commit; изменение решений в том же commit запрещено. После успешного local migration новый repository становится source of truth, а эта копия остаётся frozen historical source до появления reviewed permanent remote URL.
