# SPEC-00: Продуктовый контракт

| Поле | Значение |
|---|---|
| ID | SPEC-00 |
| Статус | Proposed |
| Версия | 1.5 |
| Владелец | Product Architecture |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [INDEX-001](README.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-012](adr/012-standalone-authority-and-acyclic-dependencies.md), [ADR-017](adr/017-artifact-first-ai-content-generation.md) |
| Связанные документы | [SPEC-12](12-vertical-slice-conformance.md), [Packet 1.4 history](README.md) |
| Заменяет | SPEC-00 v1.4 после human approval exact candidate hash |

## Назначение

Next Engine — независимый open-source runtime и toolchain для системных single-player RPG, в которых generic RPG rules, иерархический agent AI и физическое воплощение персонажей являются first-class capabilities. Продукт оптимизируется для больших streaming worlds, воспроизводимых simulation tests, offline AI degradation и расширяемого контента, а не для универсального рынка engine genres.

## Обязательный продуктовый результат v1

Conforming v1 MUST:

1. запускать один и тот же cooked project на Windows x86_64 и Linux x86_64;
2. поддерживать interactive `game`, deterministic `headless`, `tools` и optional separate `ai-host` processes;
3. реализовывать generic Character, Item, Quest, Dialogue, Faction, InteractiveObject и WorldChunk без legacy-specific runtime types;
4. сохранять и загружать versioned state, а также воспроизводить external command stream;
5. продолжать корректный игровой loop без сети, LLM и `ai-host`;
6. иметь physics-authoritative avatar на highest LOD и безопасные physics LOD transitions;
7. выполнять Luau gameplay и Wasm plugins только через capabilities и WorldCommand validation;
8. cook/validate/inspect neutral assets и bounded importer output;
9. реализовывать first-party стрельбу и магию как ordinary mechanic packages через тот же public SDK, что community mods;
10. предоставлять machine-readable authoring context, package tests/replay и safe AgentChangeSet workflow для мододелов/coding agents;
11. добавлять physical creature archetypes и progressive motor skills через public packages с безопасным policy switching и prototype/certified paths;
12. позволять CPU-only coding agent на Linux без display выполнять affected tests/diagnostics и формировать portable capture jobs;
13. принимать observable changes только по automatic evidence + hash-bound human review artifacts без обязательного interactive launch;
14. позволять начать portable core/tooling разработку на `aarch64-apple-darwin` через local-only host gate, не объявляя macOS shipping support;
15. проходить единый набор gates [SPEC-12](12-vertical-slice-conformance.md).

## Product invariants

- Gameplay state MUST изменяться только принятыми `WorldCommand`.
- У каждого изменяемого состояния MUST быть ровно один authoritative owner.
- Simulation correctness MUST NOT зависеть от renderer frame rate, LLM response или network availability.
- RuntimeEntityId MUST оставаться ephemeral; persistent contracts используют PersistentId/AssetId.
- Vendor types MUST NOT пересекать engine-owned backend boundary.
- Imported proprietary source data MUST NOT попадать в engine repository или distributable.
- Baseline renderer MUST запускаться без RT и mesh shaders.
- Release artifact MUST быть воспроизводимо связан с source/content/model/license manifests.
- First-party gameplay mechanics MUST NOT использовать hidden API, недоступный community package.
- First-party physical creatures/policies MUST использовать тот же archetype, certification и authoring contract, что community package.
- Gameplay learning MUST NOT дообучать neural weights в runtime; progression изменяет RPG state и выбирает immutable evaluated policy route.
- Coding agent MUST проходить те же capability, validation, test, provenance и human/owner review paths, что человек.
- Mandatory acceptance MUST NOT требовать monitor, input device, hidden window, interactive runtime или manual-only checklist.
- Required test subset MUST определяться engine-owned impact resolver; agent/author может только расширить его.
- Human approval MUST NOT override failed automatic gate и действует только для exact changeset/evidence hashes.
- Developer-host compatibility MUST NOT считаться shipping/platform conformance; Mac training smoke MUST NOT заменять physics correspondence или `PhysicalCertified` gates.
- `PhysicalCertified` promotion MUST оставаться `AwaitingCapability`, пока не пройден `TRAIN-RTX-01` на поддерживаемом Linux/NVIDIA host; `PrototypeFallback` от этого не блокируется.
- Proposed architecture packet MUST NOT считаться Accepted по факту agent-generated diff или passing automatic checks; promotion требует human approval exact candidate hash и атомарного status/supersession update.
- Required distributed package MUST иметь content hash и trusted publisher signature; explicit unsigned local install остаётся untrusted/local-only и не может повысить capability ceiling.
- AI-generated image/mesh/material/world output MUST оставаться untrusted authoring candidate до provenance, normalization, common cook, resolved scenarios и human review; provider account/model никогда не является runtime source of truth.
- Отсутствие network ImageGen или generation GPU MAY блокировать создание нового candidate, но MUST NOT блокировать existing-content cook, game/headless correctness или offline play.

## Scope и non-goals

В v1 не входят full scene editor, multiplayer/network replication, macOS game/render package или shipping support, consoles, mobile, обязательный cloud account, полный Gothic II compatibility, Daedalus/Ikarus/LeGo runtime, перенос OpenGothic code и публикация игровых Gothic assets. Portable developer-host tier на Apple Silicon разрешён ADR-011 и не расширяет shipping promise. SDK фиксируется только до контрактов, нужных vertical slice; весь Rust/Luau/WIT API преждевременно не замораживается.

## Пользователи и поддерживаемые workflows

| Роль | Обязательный workflow | Не обещается v1 |
|---|---|---|
| Игрок | install → offline launch → play → save/load | account/cloud sync |
| Gameplay author | typed generation/manual source → validated neutral data + Luau → cook → test/replay/evidence | визуальный full editor; обязательный cloud account |
| Engine developer | portable Rust workspace на Mac/Linux/Windows → local host-check → impact plan → CPU headless gates → optional capture job → Windows/Linux platform package | macOS shipping package; stable ABI для внутренних crates |
| Plugin author | WIT SDK → capability manifest → validate/package | native DLL injection |
| Мододел | public mechanics/physical SDK → package/prototype → scenarios/replay/certification → local install/share | implicit load-order monkey patching или self-certification |
| Coding agent | exact context bundle → scoped changeset → resolved affected tests → diagnose/minimize → evidence/review/apply | privileged ECS/shell/runtime mutation или self-approval |
| Researcher | body/skill config → offline train/export → ONNX parity/runtime suite → certification review | LLM-controlled physics или runtime weight learning |
| Importer user | указать локальную установку → inspect → neutral export → cook | bundled copyrighted content |

## Ownership и публичная граница

Этот Proposed product contract является source of truth candidate для scope, compatibility promise и release definition; Product Architecture владеет его изменением. Она не владеет subsystem state. Внешняя граница продукта состоит из project/verification/scenario/impact/evidence manifests, generation recipe/job/result/provenance/normalization manifests, cooked bundle schema, MechanicsLock/package schemas, creature/physical/policy/skill manifests, save/replay schemas, Luau capability API, WIT plugin worlds, AuthoringContextBundle/AgentChangeSet, HumanReviewDecision, `ai-host` IPC и CLI/JSON contracts. Внутренние Rust crates, ECS layout, provider SDK/model types и backend APIs не являются стабильным SDK до отдельного ADR.

## Product data flow

`manual/generated/imported data + mechanic/creature/policy packages → generation/provenance/normalization when applicable → validation/resolution → cooker + MechanicsLock → immutable bundles → game/headless → WorldCommand/DomainEvent → save/replay/presentation`. Для development: `AgentChangeSet → ImpactResolver → CPU scenarios → optional displayless capture → EvidenceBundle → human review when required`. Optional generation workers, `ai-host`, Luau, Wasm и agent tooling могут предлагать assets/commands/changesets, но не обходят validation. Presentation/capture consumers, generation и training tools не возвращают mutable model/gameplay state в simulation.

## Failure semantics

Ошибка optional feature MUST приводить к bounded degradation: generation unavailable/manual or prior asset, AI fallback, acoustic fallback, lower render/physics tier или disabled plugin. Ошибка authoritative data — corrupt save, incompatible schema, missing required bundle, invalid command — MUST быть fail-closed с structured diagnostic и без частично применённого состояния. Panic/crash одного generation/tool/ai-host process MUST NOT повреждать source assets, published content или последний валидный save.

## Implementation conformance

Implementation может заявить `vertical-v1 conformance` только когда все обязательные gates SPEC-12 имеют `PASS`, artifacts проходят hash/provenance validation и traceability не содержит незакрытых требований. Частичный demo не является v1 conformance. Непрохождение implementation gate не переписывает принятую architecture baseline: оно опровергает backend/implementation hypothesis и запускает указанный fallback/ADR process.
