# Next Engine: архитектурная baseline

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Accepted |
| Версия | 2.21 |
| Последняя проверка | 2026-08-09 |
| Заменяет | INDEX-001 version 2.20 |

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

ADR-010, ADR-015, ADR-023, ADR-024 and allocator ADR-039–043, retired evidence
register и старые review packets являются historical-only. Lightweight
traceability — навигационная карта, не admission authority.

## Индекс документов

| ID | Документ | Статус |
|---|---|---|
| SPEC-00 | [Product contract](00-product-contract.md) | Accepted |
| SPEC-01 | [System architecture](01-system-architecture.md) | Accepted |
| SPEC-02 | [Runtime, ECS и data model](02-runtime-ecs-and-data.md) | Accepted |
| SPEC-03 | [Assets, current world streaming и persistence](03-assets-world-streaming-and-persistence.md) | Accepted |
| SPEC-04 | [Rendering и platform](04-rendering-and-platform.md) | Accepted |
| SPEC-05 | [Physics, animation и motor control](05-physics-animation-and-motor-control.md) | Accepted |
| SPEC-06 | [AI agents, perception и memory](06-ai-agents-perception-and-memory.md) | Accepted |
| SPEC-07 | [RPG, scripting и plugins](07-rpg-scripting-and-plugins.md) | Accepted |
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted current audio/partition ownership; navigation sections are a Proposed R4b candidate |
| SPEC-09 | [Current tooling и observability](09-tooling-sdk-and-observability.md) | Accepted |
| SPEC-10 | [Gothic importer boundary](10-gothic-importer-boundary.md) | Accepted |
| SPEC-11 | [Runtime safety и license hygiene](11-security-licensing-and-governance.md) | Accepted |
| SPEC-12 | [Product checks и playable slice](12-vertical-slice-conformance.md) | Accepted |
| SPEC-13 | [Gameplay mechanics и mod packages](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Accepted |
| SPEC-14 | [Physical archetypes, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Accepted |
| SPEC-15 | [Local testing, headless scenarios и debugging](15-headless-testing-agent-validation-and-human-evidence.md) | Accepted |
| SPEC-16 | [Text-canonical multimodal dialogue и model packs](16-text-canonical-multimodal-dialogue-and-model-packs.md) | Proposed |
| SPEC-17 | [Direct project composition и activation](17-project-composition-configuration-and-application-lifecycle.md) | Accepted |
| SPEC-18 | [Player interaction, UI, camera, localization и accessibility](18-player-interaction-ui-camera-localization-and-accessibility.md) | Accepted |
| SPEC-19 | [Current RPG domain state](19-rpg-domain-and-narrative-state.md) | Accepted |
| SPEC-20 | [Derived world calendar and authored NPC routines](20-world-simulation-and-population-lifecycle.md) | Proposed; implementation-ready R4a candidate, not current until its production consumer and checks exist |
| SPEC-21 | [Deterministic runtime primitives, command ledger и causal identity](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | Accepted |
| SPEC-22 | [Current schema registry и format compatibility](22-schema-registry-compatibility-and-migration.md) | Accepted |
| SPEC-23 | [Future generic jobs and resource work](23-jobs-memory-resource-residency-and-io-backpressure.md) | Proposed |
| SPEC-24 | [Current neutral content и package closure](24-content-catalog-bundle-and-neutral-asset-schemas.md) | Accepted |
| SPEC-25 | [Current bounded world partition и streaming boundary](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | Accepted |
| SPEC-26 | [Physics world, collision, constraints, queries и snapshots](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) | Accepted |
| SPEC-27 | [Motor observation, action и deterministic inference](27-motor-observation-action-and-deterministic-inference.md) | Accepted |
| SPEC-28 | [Skeletal animation, retargeting и IK](28-skeletal-animation-retargeting-and-ik.md) | Accepted |
| SPEC-29 | [Platform host и simple application session](29-platform-host-and-application-session.md) | Accepted |
| SPEC-30 | [Presentation snapshot, camera, UI и render content](30-presentation-extraction-and-render-content.md) | Accepted |
| SPEC-31 | [Future narrative director и divine agency intent](31-autonomous-quest-lifecycle-and-narrative-director.md) | Proposed |
| SPEC-32 | [NPC cognition, intention lifecycle and deterministic behavior inference](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md) | Proposed |
| SPEC-33 | [Behavior-policy training, evaluation and deployment lifecycle](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md) | Proposed |
| GLOSSARY-001 | [Glossary](glossary.md) | Accepted |
| EVIDENCE-001 | [Evidence register](evidence-register.md) | Superseded; historical pointer under ADR-030 |
| TRACE-001 | [Lightweight traceability](traceability.md) | Accepted; navigation reference |

## Индекс ADR

| ID | Решение | Статус |
|---|---|---|
| ADR-000 | [ADR template](adr/000-template.md) | Draft |
| ADR-001 | [Product, repositories, license и platforms](adr/001-product-repository-license-and-platforms.md) | Accepted; process clauses partially superseded by ADR-030 |
| ADR-002 | [Rust-first core, FFI и ECS facade](adr/002-rust-first-ffi-and-ecs-facade.md) | Accepted; reviewed unsafe remains confined to FFI/backend boundaries under ADR-033/ADR-049 |
| ADR-003 | [Vulkan renderer и shader toolchain](adr/003-vulkan-renderer-and-shader-toolchain.md) | Accepted |
| ADR-004 | [Physics-avatar backend boundary](adr/004-physics-avatar-backend-boundary.md) | Superseded |
| ADR-005 | [Offline-first AI process boundary](adr/005-offline-first-ai-process-boundary.md) | Accepted |
| ADR-006 | [Scripting и plugin model](adr/006-scripting-and-plugin-model.md) | Superseded |
| ADR-007 | [Identities, persistence и replay](adr/007-identities-persistence-and-replay.md) | Superseded |
| ADR-008 | [Mechanics/mod package и agent authoring](adr/008-mechanics-mod-package-and-agent-authoring-model.md) | Accepted package/no-private-path invariants; unconsumed agent-authoring/CLI/MCP obligations superseded by ADR-046 |
| ADR-009 | [Foundation policies и progressive motor skills](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-010 | [Artifact-first validation и human review](adr/010-artifact-first-headless-validation-and-review.md) | Superseded by ADR-030 |
| ADR-011 | [macOS developer host и staged training](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-012 | [Deterministic command identity и replay V1](adr/012-deterministic-command-identity-and-replay.md) | Superseded |
| ADR-013 | [Physical-avatar authority boundary](adr/013-self-contained-physical-avatar-boundary.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-014 | [Deterministic extensions и package integrity](adr/014-deterministic-extensions-and-package-trust.md) | Accepted; process clauses partially superseded by ADR-030 |
| ADR-015 | [Evidence trust и attestation V1](adr/015-evidence-trust-fixture-separation-and-attestation.md) | Superseded by ADR-030 |
| ADR-016 | [Compositional gameplay budgets](adr/016-compositional-gameplay-budgets.md) | Accepted |
| ADR-017 | [Multimodal dialogue и model packs](adr/017-text-canonical-multimodal-dialogue-and-replaceable-model-packs.md) | Proposed |
| ADR-018 | [Project composition и configuration](adr/018-authoritative-project-composition-and-configuration.md) | Accepted configuration/activation invariants; resolver/catalog clauses superseded by ADR-048 |
| ADR-019 | [Player actions и presentation authority](adr/019-canonical-player-actions-and-presentation-authority.md) | Accepted |
| ADR-020 | [RPG domain authority](adr/020-rpg-domain-authority-and-extension-boundary.md) | Accepted ownership/transaction invariants; pre-v1 N-2 migration clause superseded by ADR-046 |
| ADR-021 | [Population residency и time advance](adr/021-deterministic-population-residency-and-time-advance.md) | Accepted core invariants; unconsumed population/calendar schemas and migration superseded by ADR-046 |
| ADR-022 | [Command identity V2, ledger и causal identity](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) | Accepted |
| ADR-023 | [HumanReviewDecisionV2 и offline attestation](adr/023-human-review-decision-v2-and-offline-attestation.md) | Superseded by ADR-030 |
| ADR-024 | [Requirement/gate/evidence/profile closure](adr/024-requirement-gate-evidence-and-profile-closure.md) | Superseded by ADR-030 |
| ADR-025 | [Schema, content и migration authority](adr/025-schema-content-and-migration-authority.md) | Accepted; generic pre-v1 N-2 support window partially superseded by ADR-046 |
| ADR-026 | [Deterministic work, resources и streaming admission](adr/026-deterministic-work-resource-and-streaming-admission.md) | Accepted immutable-staging/commit invariants; generic jobs/resource subsystem superseded by ADR-046 |
| ADR-027 | [Physics, motor и animation layering](adr/027-physics-motor-and-animation-layering.md) | Accepted |
| ADR-028 | [Platform session и presentation authority](adr/028-platform-session-and-presentation-authority.md) | Accepted platform normalization/authority; recovery/storage/close clauses superseded by ADR-047 |
| ADR-029 | [RPG-owned quest graph и narrative director](adr/029-rpg-owned-quest-graph-and-optional-narrative-director.md) | Superseded by ADR-046; future intent Proposed |
| ADR-030 | [Product-first development и lightweight validation](adr/030-product-first-development-and-lightweight-validation.md) | Accepted |
| ADR-031 | [RPG-owned divine standing и atomic pantheon judgment](adr/031-rpg-owned-divine-standing-and-atomic-pantheon-judgment.md) | Superseded by ADR-046; future intent Proposed |
| ADR-032 | [Grounded capsule physics checkpoint version boundary](adr/032-grounded-capsule-physics-checkpoint-version-boundary.md) | Accepted; designation of V4 as current generated replay partially superseded by ADR-034 |
| ADR-033 | [PhysX grounded-capsule parity и ограниченная FFI-граница](adr/033-physx-grounded-capsule-parity-ffi-boundary.md) | Accepted; PhysX backend остаётся Proposed; current reviewed unsafe allowlist is FFI-only after ADR-049 |
| ADR-034 | [Player targeting replay V5 и exact mapping provenance](adr/034-player-targeting-replay-v5-and-mapping-provenance.md) | Accepted; legacy V4/V1 retention superseded by ADR-046 |
| ADR-035 | [Bounded live recovery, platform-host binding и presentation cut](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md) | Accepted host binding and presentation cut; checkpoint/archive recovery clauses superseded by ADR-047 |
| ADR-036 | [THOTH reference performance profile и hard timing authority](adr/036-thoth-reference-performance-profile.md) | Accepted; THOTH target, budgets, baseline and no-retry authority remain; allocator clauses are superseded by ADR-049 |
| ADR-037 | [Packed session object storage](adr/037-packed-session-object-storage.md) | Superseded by ADR-047 |
| ADR-038 | [Versioned production-worker handoff diagnostic](adr/038-versioned-production-worker-handoff-diagnostic.md) | Accepted; узко заменяет diagnostic-scenario часть ADR-036 без изменения THOTH hard timing authority или B-12 closure |
| ADR-039 | [Tooling-only process-wide System GlobalAlloc measurement boundary](adr/039-tooling-only-process-wide-system-global-allocator-measurement.md) | Superseded by ADR-049 |
| ADR-040 | [Fixed TLS-sharded GlobalAlloc measurement protocol](adr/040-fixed-tls-sharded-global-allocator-measurement.md) | Superseded by ADR-049 |
| ADR-041 | [Owner-thread quiescent GlobalAlloc measurement fast path](adr/041-owner-thread-quiescent-global-allocator-measurement.md) | Superseded by ADR-049 |
| ADR-042 | [Unobserved deallocation System pass-through](adr/042-unobserved-deallocation-system-pass-through.md) | Superseded by ADR-049 |
| ADR-043 | [Codegen-proven non-reentrant count-bearing allocator callbacks](adr/043-codegen-proven-non-reentrant-count-bearing-allocator-callbacks.md) | Superseded by ADR-049 |
| ADR-044 | [Neutral text-catalog schema и deterministic locale fallback](adr/044-neutral-text-catalog-and-locale-fallback.md) | Accepted; versioned neutral catalogs, deterministic fallback closure и PresentationOnly localization semantics |
| ADR-045 | [Low-overhead hard performance evidence](adr/045-low-overhead-hard-performance-evidence.md) | Accepted low-overhead evidence and no-retry policy; allocator retention and V2/V3 tooling schemas superseded by ADR-049 |
| ADR-046 | [Consumer-driven contracts and current-only alpha formats](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) | Accepted; current-only pre-v1 formats, production-consumer admission and removal of unconsumed future obligations |
| ADR-047 | [Simple application session and save-on-close](adr/047-simple-application-session-and-save-on-close.md) | Accepted; two-slot current-state snapshot and two-stage close journal replace session recovery archives/object packs |
| ADR-048 | [Direct exact project lock](adr/048-direct-exact-project-lock.md) | Accepted; current-only authoring/registry/package formats and `ProjectLockV3` replace project resolver/catalog/policy closure |
| ADR-049 | [Performance evidence without allocator instrumentation](adr/049-performance-evidence-without-allocator-instrumentation.md) | Accepted; Performance V4 and low-overhead resource evidence replace allocator instrumentation and V2/V3 readers |
| ADR-050 | [Hierarchical NPC cognition and learned behavior-policy boundary](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md) | Proposed; two independent learned roles with exact per-seed decisions and deterministic planner fallback |
| ADR-051 | [R3a packaged chunk streaming commit boundary](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md) | Accepted; one pinned packaged fetch/decode/validate/two-tick commit vertical, without generic scheduler/resource promotion |
| ADR-052 | [Derived world calendar and authored routine vertical](adr/052-derived-world-calendar-and-authored-routine-vertical.md) | Proposed; R4a candidate for one relay-keeper routine, exact derived calendar and separate World Services segment |

## Proposed tracks

- SPEC-16/ADR-017 — optional text-canonical multimodal dialogue/model packs.
- SPEC-20/ADR-052 — implementation-ready R4a candidate: exact derived
  calendar plus one authored relay-keeper `Duty/Rest` routine. It remains
  Proposed and creates no current schema/check until the production consumer,
  save/replay path and mapped ProductChecks land in the promotion changeset.
- Navigation sections of SPEC-08 — bounded R4b graph/tile candidate; no current
  navigation API or gate until a production consumer is admitted.
- SPEC-23 — future generic scheduler/resource work after the completed bounded
  R3 partition; no generic scheduler/resource framework is accepted.
- ADR-033 — optional PhysX 5.9.0 grounded-capsule backend до полной parity на
  Windows/Linux; reference остаётся default.
- SPEC-31 narrative director, generated quest graph and divine-standing intent
  formerly described by ADR-029/ADR-031; they have no current implementation
  obligation and return only with a concrete production consumer.
- SPEC-32/SPEC-33/ADR-050 — future R4 hierarchical NPC cognition: separate
  strategic/tactical learned policies, intention/recurrent-state lifecycle and
  offline training/deployment. Both policies require one production consumer
  and passing ProductCheck before promotion; utility/HTN remains the mandatory
  deterministic fallback and LLM/audio remain optional.

Этот deferred track сохраняет deterministic offline fallback, engine-owned
contracts и untrusted proposal boundaries. Его принятие выполняется обычным
repository workflow.

Deferred narrative ideas are not current contracts or roadmap gates.

## Historical note

Architecture packet 1.0–1.9 records сохраняются как исторические snapshots
состояния review; они могут содержать proposed, pending, accepted и
superseded формулировки своего момента и не являются текущим workflow или
authority.

`incubator/gothic-importer/` остаётся ignored independent nested repository и
process boundary. Game installations, imported output, protected assets,
datasets, checkpoints, training runs, model outputs, captures и secrets не
принадлежат parent repository.
