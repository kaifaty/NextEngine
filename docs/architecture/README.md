# Next Engine: архитектурная baseline

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Accepted |
| Версия | 2.7 |
| Последняя проверка | 2026-07-30 |
| Заменяет | INDEX-001 version 2.6 |

Этот каталог задаёт архитектуру независимого AI-first open-source RPG engine.
Next Engine не является переносом OpenGothic и не является general-purpose
engine. Rust остаётся portable core, Windows x86_64 и Linux x86_64 — v1 shipping
targets, а Apple Silicon macOS — developer-host tier.

## Product-first precedence

[ADR-030](adr/030-product-first-development-and-lightweight-validation.md) —
действующий process и validation contract. При конфликте применяется:

1. более новый Accepted ADR, который явно supersedes ADR-030;
2. ADR-030 для development workflow и product checks;
3. профильный Accepted technical ADR;
4. relevant subsystem SPEC;
5. [product contract](00-product-contract.md);
6. [glossary](glossary.md).

Старые release, review, certification и admission workflows полностью
superseded ADR-030. Их catalogs и packets MAY использоваться только как
historical reference или optional test recipes и не разрешают либо блокируют
changeset, package, model или release.

## Technical baseline

Product-first не означает ослабление runtime correctness. Обязательны:

- ровно один source of truth для каждого mutable authoritative field;
- gameplay mutation только через validated engine-owned `WorldCommand` и atomic
  transaction boundary;
- committed `DomainEvent` и immutable projections вместо прямого доступа к
  domain storage;
- fixed simulation stages, deterministic commit points, command identity,
  save/load atomicity и replay equivalence;
- engine-owned public contracts без ECS, OS, vendor, database, importer и VM
  types;
- schema/version/bounds/hash/canonical-encoding/reference/resource validation
  до mutation;
- fail-closed load/admission для corrupt или incompatible authoritative data;
- capability-scoped immutable views и proposal/command sinks для scripts,
  plugins, packages, AI и tools;
- instruction/fuel, allocation, memory, host-call и output limits; trap или
  overrun не публикует partial state;
- deterministic in-process fallback для optional AI/model/backend roles;
- offline-correct mandatory gameplay;
- отсутствие credentials, private keys, protected/imported assets, datasets,
  checkpoints, training runs и generated captures/logs в Git;
- source provenance, basic license и third-party notices.

Authoritative contracts находятся в `crates/contracts`. Dependencies идут от
composition roots и adapters к engine-owned contracts. Runtime, RPG,
presentation, physical embodiment, agent intelligence, gameplay extensibility,
assets/tooling и importer не делят mutable object graph или общий
mutable database.

Production application coordination находится в `crates/application`;
first-party project/source/scenario — в `crates/reference-game`. Runtime
остаётся authority для application-session state, Assets — owner атомарной
durable publication, а `game`, `headless` и runtime-bearing `tools` входят в
эти production paths без зависимости от verification crate.

## Product checks

Изменение проходит минимальный релевантный набор:

| Check | Когда нужен | Минимальный результат |
|---|---|---|
| `fast` | Каждый code change | format, compile/typecheck, lint, focused/unit tests и boundary scan для изменённой области |
| `play` | Gameplay, runtime, UI, input, renderer или composition | representative app/headless flow запускается, загружает проект и выполняет затронутый игровой цикл |
| `persistence-replay` | Commands, IDs, authoritative state, scheduling, save schema или migrations | save→load продолжает мир; focused replay/deterministic comparison проходит; corrupt input отвергается до mutation |
| `content-package` | Asset schema, cooker, importer boundary, package/plugin или distribution | representative content/package validates и cooks/loads; bounds/hash/version, protected-data и basic license-notice checks проходят |
| `platform` | Только renderer, packaging, host integration или OS-specific change | targeted smoke на доступной релевантной platform |
| `performance` | Только material hot-path, physics, renderer, I/O или model-runtime change | targeted benchmark на declared profile без изменения authoritative result |

Focused checks используются во время итерации; широкий local check запускается
перед handoff, когда он существует и релевантен. Недоступная GPU, encoder, RTX
machine или shipping host не блокирует unrelated work. В handoff
указывается, какие checks прошли, не запускались или завершились ошибкой.

Flaky retry-to-green, wall-clock gameplay assertions, silent fallback после
authoritative corruption и mutable test backdoors запрещены. Ни один check не
требует отдельного serialized approval artifact.

## Изменение архитектуры

Semantic change оформляется новым ADR через обычный repository workflow.
Прямой запрос на архитектурное изменение достаточен. Отдельный promotion tool,
organizational approval matrix и generated process graph не требуются.

Редакционная правка MAY обновить ссылки, статус и supersession note без нового
ADR. Новый technology/backend остаётся replaceable implementation за
engine-owned API; для него SHOULD быть указан bounded evaluation path и fallback.

Статусы:

- `Draft` — рабочий материал;
- `Proposed` — candidate semantics, не входящая в Accepted technical baseline;
- `Accepted` — действующее архитектурное решение;
- `Rejected` — отклонённый вариант;
- `Superseded` — historical решение со ссылкой на замену.

Статус документа не является test result или release approval.

## Порядок чтения

1. [ADR-030](adr/030-product-first-development-and-lightweight-validation.md).
2. [Product contract](00-product-contract.md) и
   [system architecture](01-system-architecture.md).
3. [Glossary](glossary.md).
4. Relevant subsystem SPEC и его technical ADR.
5. [Product checks](12-vertical-slice-conformance.md) и
   [local testing/debugging](15-headless-testing-agent-validation-and-human-evidence.md).
6. Для deterministic runtime/save/replay:
   [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md)
   и
   [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md).

ADR-010, ADR-015, ADR-023, ADR-024, retired evidence register и старые review
packets являются historical-only. Lightweight traceability — навигационная
карта, не admission authority.

## Индекс документов

| ID | Документ | Статус |
|---|---|---|
| SPEC-00 | [Product contract](00-product-contract.md) | Accepted |
| SPEC-01 | [System architecture](01-system-architecture.md) | Accepted |
| SPEC-02 | [Runtime, ECS и data model](02-runtime-ecs-and-data.md) | Accepted |
| SPEC-03 | [Assets, streaming и persistence](03-assets-world-streaming-and-persistence.md) | Accepted |
| SPEC-04 | [Rendering и platform](04-rendering-and-platform.md) | Accepted |
| SPEC-05 | [Physics, animation и motor control](05-physics-animation-and-motor-control.md) | Accepted |
| SPEC-06 | [AI agents, perception и memory](06-ai-agents-perception-and-memory.md) | Accepted |
| SPEC-07 | [RPG, scripting и plugins](07-rpg-scripting-and-plugins.md) | Accepted |
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted |
| SPEC-09 | [Tooling, SDK и observability](09-tooling-sdk-and-observability.md) | Accepted |
| SPEC-10 | [Gothic importer boundary](10-gothic-importer-boundary.md) | Accepted |
| SPEC-11 | [Runtime safety и license hygiene](11-security-licensing-and-governance.md) | Accepted |
| SPEC-12 | [Product checks и playable slice](12-vertical-slice-conformance.md) | Accepted |
| SPEC-13 | [Gameplay mechanics, mod packages и agent authoring](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Accepted |
| SPEC-14 | [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Accepted |
| SPEC-15 | [Local testing, headless scenarios и debugging](15-headless-testing-agent-validation-and-human-evidence.md) | Accepted |
| SPEC-16 | [Text-canonical multimodal dialogue и model packs](16-text-canonical-multimodal-dialogue-and-model-packs.md) | Proposed |
| SPEC-17 | [Project composition, configuration и application lifecycle](17-project-composition-configuration-and-application-lifecycle.md) | Accepted |
| SPEC-18 | [Player interaction, UI, camera, localization и accessibility](18-player-interaction-ui-camera-localization-and-accessibility.md) | Accepted |
| SPEC-19 | [RPG domain и narrative state](19-rpg-domain-and-narrative-state.md) | Accepted |
| SPEC-20 | [World simulation и population lifecycle](20-world-simulation-and-population-lifecycle.md) | Accepted |
| SPEC-21 | [Deterministic runtime primitives, command ledger и causal identity](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | Accepted |
| SPEC-22 | [Schema registry, compatibility и migration](22-schema-registry-compatibility-and-migration.md) | Accepted |
| SPEC-23 | [Jobs, memory, compute residency и I/O backpressure](23-jobs-memory-resource-residency-and-io-backpressure.md) | Accepted |
| SPEC-24 | [Content catalog, bundle и neutral asset schemas](24-content-catalog-bundle-and-neutral-asset-schemas.md) | Accepted |
| SPEC-25 | [World partition, streaming admission и persistent spatial objects](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | Accepted |
| SPEC-26 | [Physics world, collision, constraints, queries и snapshots](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) | Accepted |
| SPEC-27 | [Motor observation, action и deterministic inference](27-motor-observation-action-and-deterministic-inference.md) | Accepted |
| SPEC-28 | [Skeletal animation, retargeting и IK](28-skeletal-animation-retargeting-and-ik.md) | Accepted |
| SPEC-29 | [Platform host и application session](29-platform-host-and-application-session.md) | Accepted |
| SPEC-30 | [Presentation extraction и render content](30-presentation-extraction-and-render-content.md) | Accepted |
| SPEC-31 | [Autonomous quest lifecycle, divine agency и narrative director](31-autonomous-quest-lifecycle-and-narrative-director.md) | Accepted |
| GLOSSARY-001 | [Glossary](glossary.md) | Accepted |
| EVIDENCE-001 | [Evidence register](evidence-register.md) | Superseded; historical pointer under ADR-030 |
| TRACE-001 | [Lightweight traceability](traceability.md) | Accepted; navigation reference |

## Индекс ADR

| ID | Решение | Статус |
|---|---|---|
| ADR-000 | [ADR template](adr/000-template.md) | Draft |
| ADR-001 | [Product, repositories, license и platforms](adr/001-product-repository-license-and-platforms.md) | Accepted; process clauses partially superseded by ADR-030 |
| ADR-002 | [Rust-first core, FFI и ECS facade](adr/002-rust-first-ffi-and-ecs-facade.md) | Accepted |
| ADR-003 | [Vulkan renderer и shader toolchain](adr/003-vulkan-renderer-and-shader-toolchain.md) | Accepted |
| ADR-004 | [Physics-avatar backend boundary](adr/004-physics-avatar-backend-boundary.md) | Superseded |
| ADR-005 | [Offline-first AI process boundary](adr/005-offline-first-ai-process-boundary.md) | Accepted |
| ADR-006 | [Scripting и plugin model](adr/006-scripting-and-plugin-model.md) | Superseded |
| ADR-007 | [Identities, persistence и replay](adr/007-identities-persistence-and-replay.md) | Superseded |
| ADR-008 | [Mechanics/mod package и agent authoring](adr/008-mechanics-mod-package-and-agent-authoring-model.md) | Accepted |
| ADR-009 | [Foundation policies и progressive motor skills](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-010 | [Artifact-first validation и human review](adr/010-artifact-first-headless-validation-and-review.md) | Superseded by ADR-030 |
| ADR-011 | [macOS developer host и staged training](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-012 | [Deterministic command identity и replay V1](adr/012-deterministic-command-identity-and-replay.md) | Superseded |
| ADR-013 | [Physical-avatar authority boundary](adr/013-self-contained-physical-avatar-boundary.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-014 | [Deterministic extensions и package integrity](adr/014-deterministic-extensions-and-package-trust.md) | Accepted; process clauses partially superseded by ADR-030 |
| ADR-015 | [Evidence trust и attestation V1](adr/015-evidence-trust-fixture-separation-and-attestation.md) | Superseded by ADR-030 |
| ADR-016 | [Compositional gameplay budgets](adr/016-compositional-gameplay-budgets.md) | Accepted |
| ADR-017 | [Multimodal dialogue и model packs](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md) | Proposed |
| ADR-018 | [Project composition и configuration](adr/018-authoritative-project-composition-and-configuration.md) | Accepted |
| ADR-019 | [Player actions и presentation authority](adr/019-canonical-player-actions-and-presentation-authority.md) | Accepted |
| ADR-020 | [RPG domain authority](adr/020-rpg-domain-authority-and-extension-boundary.md) | Accepted |
| ADR-021 | [Population residency и time advance](adr/021-deterministic-population-residency-and-time-advance.md) | Accepted |
| ADR-022 | [Command identity V2, ledger и causal identity](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) | Accepted |
| ADR-023 | [HumanReviewDecisionV2 и offline attestation](adr/023-human-review-decision-v2-and-offline-attestation.md) | Superseded by ADR-030 |
| ADR-024 | [Requirement/gate/evidence/profile closure](adr/024-requirement-gate-evidence-and-profile-closure.md) | Superseded by ADR-030 |
| ADR-025 | [Schema, content и migration authority](adr/025-schema-content-and-migration-authority.md) | Accepted |
| ADR-026 | [Deterministic work, resources и streaming admission](adr/026-deterministic-work-resource-and-streaming-admission.md) | Accepted |
| ADR-027 | [Physics, motor и animation layering](adr/027-physics-motor-and-animation-layering.md) | Accepted |
| ADR-028 | [Platform session и presentation authority](adr/028-platform-session-and-presentation-authority.md) | Accepted; active-run cadence, bounded recovery evidence, host binding и presentation-cut clauses partially superseded by ADR-035 |
| ADR-029 | [RPG-owned quest graph и narrative director](adr/029-rpg-owned-quest-graph-and-optional-narrative-director.md) | Accepted |
| ADR-030 | [Product-first development и lightweight validation](adr/030-product-first-development-and-lightweight-validation.md) | Accepted |
| ADR-031 | [RPG-owned divine standing и atomic pantheon judgment](adr/031-rpg-owned-divine-standing-and-atomic-pantheon-judgment.md) | Accepted; partially supersedes ADR-020 aggregate enumeration |
| ADR-032 | [Grounded capsule physics checkpoint version boundary](adr/032-grounded-capsule-physics-checkpoint-version-boundary.md) | Accepted; designation of V4 as current generated replay partially superseded by ADR-034 |
| ADR-033 | [PhysX grounded-capsule parity и ограниченная FFI-граница](adr/033-physx-grounded-capsule-parity-ffi-boundary.md) | Accepted; PhysX backend остаётся Proposed |
| ADR-034 | [Player targeting replay V5 и exact mapping provenance](adr/034-player-targeting-replay-v5-and-mapping-provenance.md) | Accepted |
| ADR-035 | [Bounded live recovery, platform-host binding и presentation cut](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md) | Accepted; partially supersedes ADR-028 |

## Proposed tracks

- SPEC-16/ADR-017 — optional text-canonical multimodal dialogue/model packs.
- ADR-033 — optional PhysX 5.9.0 grounded-capsule backend до полной parity на
  Windows/Linux; reference остаётся default.

Этот deferred track сохраняет deterministic offline fallback, engine-owned
contracts и untrusted proposal boundaries. Его принятие выполняется обычным
repository workflow.

Accepted SPEC-31/ADR-029/ADR-031 определяют intent/need admission into
autonomous Quest opportunities, four deterministic disclosure channels,
system-owned activation/rewards, optional narrative director, RPG-owned divine
standing и atomic conflict resolution for independently judging gods. Этот
architecture status не означает runtime implementation или `PASS` будущих
gameplay checks.

## Historical note

Architecture packet 1.0–1.9 records сохраняются как исторические snapshots
состояния review; они могут содержать proposed, pending, accepted и
superseded формулировки своего момента и не являются текущим workflow или
authority.

`incubator/gothic-importer/` остаётся ignored independent nested repository и
process boundary. Game installations, imported output, protected assets,
datasets, checkpoints, training runs, model outputs, captures и secrets не
принадлежат parent repository.
