# Next Engine: архитектурная baseline

| Поле | Значение |
|---|---|
| ID | INDEX-001 |
| Статус | Accepted |
| Версия | 2.46 |
| Последняя проверка | 2026-08-18 |
| Заменяет | INDEX-001 2.45; records the current bounded R5g authored pose-corrective and cadence/deformation-LOD route without promoting injury or full embodiment |

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

Чистая documentation-only правка без executable/build/schema/generated-data
изменений использует cheap path: `git diff --check` и direct validation
изменённых ссылок, путей и identifiers. Она не запускает Cargo или
`host-check` по умолчанию. Для локального code change `fast` может состоять из
format/lint/focused tests затронутого package и boundary scan. Полный
workspace `host-check` нужен только для cross-cutting/public-contract/workspace
изменения, неясной области влияния либо явного требования пользователя/плана.
Создание Git commit само по себе не запускает и не требует ProductCheck:
проверки относятся к final handoff/readiness claim или explicit request, а не
к сохранению промежуточного checkpoint.

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

Product intent for functional anatomy and final visible characters is recorded
in the non-contractual but approved
[PRODUCT-FA-001 brief](../product/functional-anatomy-and-character-embodiment.md).
ADR-075 and SPEC-18/36/37 translate that intent into normative architecture.

ADR-009, ADR-010, ADR-015, ADR-023, ADR-024, ADR-033, ADR-039–043 and ADR-055, retired evidence
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
| SPEC-08 | [Audio, navigation и world services](08-audio-navigation-and-world-services.md) | Accepted audio plus bounded engine-owned graph/query baseline; broader navmesh/traversal remains Proposed |
| SPEC-09 | [Current tooling и observability](09-tooling-sdk-and-observability.md) | Accepted |
| SPEC-10 | [Gothic importer boundary](10-gothic-importer-boundary.md) | Accepted |
| SPEC-11 | [Runtime safety и license hygiene](11-security-licensing-and-governance.md) | Accepted |
| SPEC-12 | [Product checks и playable slice](12-vertical-slice-conformance.md) | Accepted; commit-independent risk-scoped validation and documentation-only cheap path |
| SPEC-13 | [Gameplay mechanics и mod packages](13-gameplay-mechanics-mod-packages-and-agent-authoring.md) | Accepted |
| SPEC-14 | [Physical archetypes, BodySchema, motor skills и policy lifecycle](14-physical-archetypes-motor-skills-and-policy-lifecycle.md) | Accepted hierarchy/ownership; fixed-humanoid BodySchema V1 is current through SPEC-35, advanced skill/adaptation/family profiles remain Proposed |
| SPEC-15 | [Local testing, headless scenarios и debugging](15-headless-testing-agent-validation-and-human-evidence.md) | Accepted |
| SPEC-16 | [Text-canonical multimodal dialogue и model packs](16-text-canonical-multimodal-dialogue-and-model-packs.md) | Proposed |
| SPEC-17 | [Direct project composition и activation](17-project-composition-configuration-and-application-lifecycle.md) | Accepted |
| SPEC-18 | [Player interaction, UI, camera, localization и accessibility](18-player-interaction-ui-camera-localization-and-accessibility.md) | Accepted; future qualitative body-status projection remains consumer-driven |
| SPEC-19 | [Current RPG domain state](19-rpg-domain-and-narrative-state.md) | Accepted |
| SPEC-20 | [World calendar, authored routines and bounded population lifecycle](20-world-simulation-and-population-lifecycle.md) | Accepted R4a calendar/routine, R4b population/navigation, R4c cognition consumer and bounded R4d activity/tier/bulk-time vertical |
| SPEC-21 | [Deterministic runtime primitives, command ledger и causal identity](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) | Accepted |
| SPEC-22 | [Current schema registry и format compatibility](22-schema-registry-compatibility-and-migration.md) | Accepted |
| SPEC-23 | [Future generic jobs and resource work](23-jobs-memory-resource-residency-and-io-backpressure.md) | Proposed |
| SPEC-24 | [Current neutral content и package closure](24-content-catalog-bundle-and-neutral-asset-schemas.md) | Accepted |
| SPEC-25 | [Current bounded world partition и streaming boundary](25-world-partition-streaming-admission-and-persistent-spatial-objects.md) | Accepted |
| SPEC-26 | [Physics world, collision, constraints, queries и snapshots](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) | Accepted |
| SPEC-27 | [Motor observation, action и deterministic inference](27-motor-observation-action-and-deterministic-inference.md) | Accepted generic tensor/state/safety/replay baseline; exact adaptation/reference profile evolution is Proposed |
| SPEC-28 | [Skeletal animation, retargeting и IK](28-skeletal-animation-retargeting-and-ik.md) | Accepted |
| SPEC-29 | [Platform host и simple application session](29-platform-host-and-application-session.md) | Accepted |
| SPEC-30 | [Presentation snapshot, camera, UI и render content](30-presentation-extraction-and-render-content.md) | Accepted |
| SPEC-31 | [Future narrative director и divine agency intent](31-autonomous-quest-lifecycle-and-narrative-director.md) | Proposed |
| SPEC-32 | [Deterministic Strategic Agent cognition and social behavior](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md) | Accepted R4c cognition core under ADR-073 plus bounded R4d social/work/economy and tier-cadence vertical under ADR-074 |
| SPEC-33 | [Behavior-policy training, evaluation and deployment lifecycle](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md) | Proposed optional R8 quality track |
| SPEC-34 | [Model-training environments, trajectories and consolidation lifecycle](34-model-training-environments-trajectories-and-consolidation-lifecycle.md) | Proposed common lifecycle; bounded standing/flat-command/curriculum V2 and biomechanics reference-tracker V3 records are current through SPEC-35 and ADR-064/065/067/070; SPEC-44 does not yet add a world-solver lane |
| SPEC-35 | [Deterministic humanoid training substrate](35-deterministic-humanoid-training-substrate.md) | Accepted PhysX-only fixed 23-DoF standing, flat-command/curriculum and biomechanics reference-tracking environments; profiles authorize implementation, not learned quality, runtime policy or R5 completion |
| SPEC-36 | [Functional tissue condition, injury and structural body changes](36-functional-tissue-condition-and-injury.md) | Accepted functional-anatomy product, ownership, treatment, player/NPC parity and fallback semantics; exact contracts/vertical remain Proposed |
| SPEC-37 | [Character embodiment, surface deformation and injury presentation](37-character-embodiment-and-surface-deformation.md) | Accepted realistic third-person target and current R5g exact base-rig/LBS/pose-corrective/fallback plus bounded cadence/deformation-LOD route; load/injury, severity matrix and advanced deformers remain Proposed |
| SPEC-38 | [Proposed continuum material physics](38-continuum-material-physics.md) | Proposed post-v1 local water/deformable-terrain track; CPU DFSPH reference, GPU correspondence and MLS-MPM terrain are not current runtime contracts |
| SPEC-39 | [Proposed layered physical-world model](39-layered-physical-world.md) | Proposed owner/coupling/commit model for composing rigid, continuum, living-structure and thermochemical state without a universal solver or second writer |
| SPEC-40 | [Proposed structural vegetation physics](40-structural-vegetation-physics.md) | Proposed sparse tree graph, CPU structural oracle, section-cell cutting, PhysX handoff, exact persistence and forest-LOD track; V0A decisions are closed and V0B calibration remains open |
| SPEC-41 | [Proposed world-substrate composition](41-world-substrate-composition.md) | Proposed successor stage-8 `WorldDynamicsStep`, runtime-owned DAG, exact identity, epoch persistence and fail-stop transaction; current schedule remains unchanged |
| SPEC-42 | [Proposed arcane substrate and physical magic](42-arcane-substrate-and-physical-magic.md) | Proposed fixed-point Arcane owner with analytical maximum debit, successor-stage PhysX coupling and epoch persistence; numeric A0B remains open |
| SPEC-43 | [Proposed thermochemical material processes](43-thermochemical-material-processes.md) | Proposed enthalpy/phase owner with per-interface sub-LSB residuals and atomic parcel topology; numeric/profile T0B remains open |
| SPEC-44 | [Proposed neural-assisted world simulation](44-neural-assisted-world-simulation.md) | Proposed N0/N1 report/shadow-only research; runtime advice requires a later safety-certificate-backed Accepted ADR |
| GLOSSARY-001 | [Glossary](glossary.md) | Accepted |
| EVIDENCE-001 | [Evidence register](evidence-register.md) | Superseded; historical pointer under ADR-030 |
| TRACE-001 | [Lightweight traceability](traceability.md) | Accepted; navigation reference |

## Индекс ADR

| ID | Решение | Статус |
|---|---|---|
| ADR-000 | [ADR template](adr/000-template.md) | Draft |
| ADR-001 | [Product, repositories, license и platforms](adr/001-product-repository-license-and-platforms.md) | Accepted; process clauses partially superseded by ADR-030 |
| ADR-002 | [Rust-first core, FFI и ECS facade](adr/002-rust-first-ffi-and-ecs-facade.md) | Accepted; reviewed unsafe remains confined to FFI/backend boundaries under ADR-058/ADR-049 |
| ADR-003 | [Vulkan renderer и shader toolchain](adr/003-vulkan-renderer-and-shader-toolchain.md) | Accepted |
| ADR-004 | [Physics-avatar backend boundary](adr/004-physics-avatar-backend-boundary.md) | Superseded |
| ADR-005 | [Offline-first AI process boundary](adr/005-offline-first-ai-process-boundary.md) | Accepted; planner wording partially superseded by ADR-056, process/offline boundaries preserved |
| ADR-006 | [Scripting и plugin model](adr/006-scripting-and-plugin-model.md) | Superseded |
| ADR-007 | [Identities, persistence и replay](adr/007-identities-persistence-and-replay.md) | Superseded |
| ADR-008 | [Mechanics/mod package и agent authoring](adr/008-mechanics-mod-package-and-agent-authoring-model.md) | Accepted package/no-private-path invariants; unconsumed agent-authoring/CLI/MCP obligations superseded by ADR-046 |
| ADR-009 | [Foundation policies и progressive motor skills](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md) | Superseded by ADR-057; current authority ADR-066 |
| ADR-010 | [Artifact-first validation и human review](adr/010-artifact-first-headless-validation-and-review.md) | Superseded by ADR-030 |
| ADR-011 | [macOS developer host и staged training](adr/011-macos-developer-host-local-verification-and-staged-training.md) | Accepted; certification clauses partially superseded by ADR-030 |
| ADR-012 | [Deterministic command identity и replay V1](adr/012-deterministic-command-identity-and-replay.md) | Superseded |
| ADR-013 | [Physical-avatar authority boundary](adr/013-self-contained-physical-avatar-boundary.md) | Accepted authority boundary; process clauses partially superseded by ADR-030 and backend fallback by ADR-058 |
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
| ADR-027 | [Physics, motor и animation layering](adr/027-physics-motor-and-animation-layering.md) | Accepted ownership/layering; backend-neutral candidate/fallback wording partially superseded by ADR-058 |
| ADR-028 | [Platform session и presentation authority](adr/028-platform-session-and-presentation-authority.md) | Accepted platform normalization/authority; recovery/storage/close clauses superseded by ADR-047 |
| ADR-029 | [RPG-owned quest graph и narrative director](adr/029-rpg-owned-quest-graph-and-optional-narrative-director.md) | Superseded by ADR-046; future intent Proposed |
| ADR-030 | [Product-first development и lightweight validation](adr/030-product-first-development-and-lightweight-validation.md) | Accepted |
| ADR-031 | [RPG-owned divine standing и atomic pantheon judgment](adr/031-rpg-owned-divine-standing-and-atomic-pantheon-judgment.md) | Superseded by ADR-046; future intent Proposed |
| ADR-032 | [Grounded capsule physics checkpoint version boundary](adr/032-grounded-capsule-physics-checkpoint-version-boundary.md) | Accepted; designation of V4 as current generated replay partially superseded by ADR-034 |
| ADR-033 | [PhysX grounded-capsule parity и ограниченная FFI-граница](adr/033-physx-grounded-capsule-parity-ffi-boundary.md) | Superseded by ADR-058 |
| ADR-034 | [Player targeting replay V5 и exact mapping provenance](adr/034-player-targeting-replay-v5-and-mapping-provenance.md) | Accepted; legacy V4/V1 retention superseded by ADR-046 |
| ADR-035 | [Bounded live recovery, platform-host binding и presentation cut](adr/035-bounded-live-recovery-platform-host-and-presentation-cut.md) | Accepted host binding and presentation cut; checkpoint/archive recovery clauses superseded by ADR-047 |
| ADR-036 | [THOTH reference performance profile и hard timing authority](adr/036-thoth-reference-performance-profile.md) | Accepted; THOTH target, baseline and no-retry authority remain; allocator clauses are superseded by ADR-049, preflight thresholds by ADR-060/ADR-061, driver/R5 workload details by ADR-062 and relative evidence unit by ADR-063 |
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
| ADR-049 | [Performance evidence without allocator instrumentation](adr/049-performance-evidence-without-allocator-instrumentation.md) | Accepted allocator-removal and low-overhead resource evidence; Performance V4/methodology identity are superseded by ADR-063 |
| ADR-050 | [Optional learned strategic and tactical behavior-policy boundary](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md) | Proposed optional R8 role policies with exact applied-decision parity and ADR-056 fallback |
| ADR-051 | [R3a packaged chunk streaming commit boundary](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md) | Accepted; one pinned packaged fetch/decode/validate/two-tick commit vertical, without generic scheduler/resource promotion |
| ADR-052 | [Derived world calendar and authored routine vertical](adr/052-derived-world-calendar-and-authored-routine-vertical.md) | Accepted; one relay-keeper routine, exact derived calendar, joint owner commit and current-only Replay V6 |
| ADR-053 | [Engine-native model training and immutable artifact boundary](adr/053-engine-native-model-training-and-immutable-artifact-boundary.md) | Proposed; canonical headless environment, accelerated mirrors and immutable candidate bundles |
| ADR-054 | [Bounded strategic adaptation and two-tier sleep](adr/054-bounded-strategic-adaptation-and-two-tier-sleep.md) | Proposed; Hope-inspired explicit bounded state, deterministic runtime consolidation and offline child bundles |
| ADR-055 | [Mamba-2 physical motion foundation profile](adr/055-mamba2-physical-motion-foundation-profile.md) | Superseded by ADR-057; current authority ADR-066 |
| ADR-056 | [Deterministic Strategic Agent and belief-driven GOAP](adr/056-deterministic-strategic-agent-and-belief-driven-goap.md) | Accepted; belief-driven Utility + bounded GOAP closes R4/v1 without learned models |
| ADR-057 | [Hierarchical learnable Motor System and policy-family architecture](adr/057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) | Superseded by ADR-066 |
| ADR-058 | [PhysX-only deterministic humanoid training substrate](adr/058-physx-only-deterministic-humanoid-training-substrate.md) | Accepted sole production backend and fixed-humanoid Stage 0; cutover/completion require Windows/Linux and replay/performance gates |
| ADR-059 | [Event-sourced PhysX continuation reconstruction](adr/059-event-sourced-physx-continuation-reconstruction.md) | Accepted bounded reset + post-safety effort prefix; partially supersedes ADR-058 direct continuation-import assumption |
| ADR-060 | [Relaxed THOTH performance preflight](adr/060-relaxed-thoth-performance-preflight.md) | Accepted 10 GiB free-RAM threshold; below-15% load and methodology v5 are superseded by ADR-061 |
| ADR-061 | [Forty-percent THOTH load preflight](adr/061-forty-percent-thoth-load-preflight.md) | Accepted CPU/GPU start-load below 40%; methodology identity is superseded by ADR-063 |
| ADR-062 | [R5 PhysX humanoid performance authority](adr/062-r5-physx-humanoid-performance-authority.md) | Accepted 16-slot 23-DoF PhysX workload, 1/4/8-worker budgets and driver 610.88; single-run relative gate/methodology v7 are superseded by ADR-063 |
| ADR-063 | [Run-level performance evidence and fixed gate batches](adr/063-run-level-performance-evidence-and-fixed-gate-batches.md) | Accepted Performance V5/methodology v8, ten independent baseline runs, fixed three-run gate batches and run-level relative bootstrap |
| ADR-064 | [Canonical flat-command locomotion environment](adr/064-canonical-flat-command-locomotion-environment.md) | Accepted engine-owned 23-DoF flat locomotion commands, root-local observation, Q16 reward, partial reset/checkpoint lifecycle, protocol/mirror v2 boundary; no learned-policy claim |
| ADR-065 | [Curriculum flat-command locomotion profile](adr/065-curriculum-flat-command-locomotion-profile.md) | Accepted engine-owned episode-ordinal curriculum, sharper Q16 reward/support shaping and full-stage held-out evaluation; V1 remains unchanged and learned quality remains unproven |
| ADR-066 | [Contact-centric physical skills and morphology-conditioned motor architecture](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) | Accepted no-text physical-skill hierarchy, heterogeneous BodySchema, `PhysicalActionChunk`, family graph-controller and deterministic rollout boundaries; exact learned profiles remain Proposed |
| ADR-067 | [Stage 0 profile identity and curriculum hash closure](adr/067-stage0-profile-identity-and-curriculum-hash-closure.md) | Accepted frozen standing/flat-command V1 identity, per-profile translator hash and exact-zero curriculum support semantics |
| ADR-068 | [Static morphology cache and action-chunk field closure](adr/068-static-morphology-cache-and-action-chunk-field-closure.md) | Accepted dedicated static morphology cache identity and one Proposed `PhysicalActionChunk` field set; no learned route promoted |
| ADR-069 | [Biomechanics BodySchema V2 and explicit solver projection](adr/069-biomechanics-body-schema-v2-and-solver-projection.md) | Accepted current-only biomechanics schema with full inertia, explicit solver carriers/projection, collider/contact roles and hard safety closure; V1 unchanged |
| ADR-070 | [Biomechanics reference-tracking training environment](adr/070-biomechanics-reference-tracking-training-environment.md) | Accepted current-only TRAIN-5 locomotion tracker profile, corpus-bound V3 manifest, fixed 435-channel observation and 23-channel residual action; no quality or runtime-policy claim |
| ADR-071 | [Canonical physics-material lineage](adr/071-canonical-physics-material-lineage.md) | Accepted successor material/combine contracts, biomechanics compiled/mirror lineage and explicit ABI 4 material input; no PhysX or training claim |
| ADR-072 | [Deterministic population tiers and graph-navigation vertical](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md) | Accepted R4b 100-record World Services owner and 64-node graph; its six-owner/Replay V7 format boundary is superseded by ADR-073 |
| ADR-073 | [Deterministic cognition owner vertical](adr/073-deterministic-cognition-owner-vertical.md) | Accepted R4c semantic beliefs, fixed-point Utility, bounded GOAP and paired Agent/Memory owners; its V5/V6/Replay V8 boundary is superseded by ADR-074 |
| ADR-074 | [Systemic Strategic Agent owner vertical](adr/074-systemic-strategic-agent-owner-vertical.md) | Accepted bounded R4d structured social/work/economy path, activity owner, tier cognition, bulk-time equivalence, V6/V7 content and nine-owner Replay V9 |
| ADR-075 | [Product-grounded functional anatomy and character embodiment](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) | Accepted functional gameplay abstraction and third-person semantics; R5g base-rig/LBS plus bounded pose-corrective/deformation-LOD projection is current, while condition/injury schemas and the complete lower-limb severity matrix remain Proposed |
| ADR-076 | [Continuum material physics track](adr/076-continuum-material-physics-track.md) | Proposed multi-lane continuum strategy, partially narrowed by ADR-081; no current backend/schema/save claim |
| ADR-077 | [Layered physical world and living-structures track](adr/077-layered-physical-world-and-living-structures-track.md) | Proposed destructible-tree profile, partially narrowed by ADR-081; V0B remains open and there is no current backend/schema/save claim |
| ADR-078 | [World substrate and arcane physical-interaction track](adr/078-world-substrate-and-arcane-physical-interaction-track.md) | Proposed telekinesis-first architecture, partially narrowed by ADR-081; numeric A0B remains open and no current schema/runtime claim exists |
| ADR-079 | [Thermochemical material-process track](adr/079-thermochemical-material-process-track.md) | Proposed enthalpy-first parcel owner, partially narrowed by ADR-081; no current schema/runtime claim |
| ADR-080 | [Neural assistance as bounded proposals](adr/080-neural-assistance-as-bounded-proposals.md) | Proposed report/shadow-only research after classical promotion, narrowed by ADR-081 |
| ADR-081 | [World-dynamics gap closure and promotion guardrails](adr/081-world-dynamics-gap-closure-and-promotion-guardrails.md) | Accepted promotion guardrails for successor scheduling, composition, exact continuation, float execution, fault/capacity/budget semantics and domain closures; no current runtime/schema activation |

## Proposed tracks

- SPEC-38/ADR-076 — post-v1 local water and deformable-material research; CPU
  DFSPH is the candidate water authority, GPU remains correspondence-only and
  the main R8 integration track stays inactive until the serial oracle passes.
- SPEC-39/SPEC-40/ADR-077 — post-v1 layered physical-world and living-
  structures research; the selected tree vertical is blocked on V0B numeric,
  material and corpus calibration before solver code.
- SPEC-41/SPEC-42/ADR-078 — post-v1 world-substrate and arcane-physical
  composition; the telekinesis architecture is closed, numeric A0B is open,
  and thermochemical/continuum/vegetation/Vital/Identity lanes remain independent.
- SPEC-43/ADR-079 — post-v1 thermochemical material research; a sealed
  enthalpy/ice calorimetry profile is first, T0B calibration blocks code, and
  combustion/atmosphere/physical/arcane couplers remain independent.
- SPEC-44/ADR-080 — optional N0/N1 report/shadow assistance downstream of a
  promoted classical owner; models own no state and cannot affect production
  work, roots or failure classes under ADR-081.
- SPEC-16/ADR-017 — optional text-canonical multimodal dialogue/model packs.
- Broader navigation sections of SPEC-08 — navmesh cooking, dynamic overlays,
  tactical/physical path following and optional Recast adapter remain Proposed
  beyond the current ADR-072 graph/query and abstract-transfer baseline.
- SPEC-23 — future generic scheduler/resource work after the completed bounded
  R3 partition; no generic scheduler/resource framework is accepted.
- SPEC-31 narrative director, generated quest graph and divine-standing intent
  formerly described by ADR-029/ADR-031; they have no current implementation
  obligation and return only with a concrete production consumer.
- SPEC-32 breadth beyond ADR-074 — broad bargaining, episodic/social memory,
  generic jobs markets, crime/faction and macro-economy behavior. The bounded
  structured speech, commitment, activity, work/currency/trade/food and exact
  tier-cadence consumer is current; LLM/audio remain optional.
- SPEC-33/SPEC-34 and ADR-050/ADR-053/ADR-054 — optional R8 learned
  strategic/tactical and training data-plane track. Per-role promotion requires
  immutable artifacts, multi-seed evidence, target parity and complete ADR-056
  fallback; joint promotion is required only by profiles activating both roles.
- ADR-066/ADR-068/ADR-069/ADR-070/ADR-071 Accepted core with Proposed subsections in SPEC-14/SPEC-26/SPEC-27/
  SPEC-28/SPEC-34 — natural language ends before Physical Embodiment; typed physical
  primitives and `PhysicalActionChunk` carry contact/root/CoM/effector meaning.
  The current TRAIN-5 tracker environment still proves no learned MLP quality;
  after that bounded experiment come contact chunks, bounded
  graph/shared-joint within-family transfer, damage/equipment/skills,
  distillation/compiled students and optional candidate rollouts. GRU is the
  first recurrent baseline; Mamba remains an equal-budget experiment. The
  procedural/animation R5 path stays current and independently shippable.
- PRODUCT-FA-001, SPEC-18/SPEC-36/SPEC-37 and ADR-075 — functional anatomy,
  staged treatment, preserved agency, player/NPC parity, qualitative body UI,
  third-person `Reduced`/`Realistic`/`Graphic` presentation and complete
  fallback semantics are Accepted. Exact commands/manifests/controllers and
  the 16/64/distant workload profile remain Proposed beyond the current small
  authored pose-corrective/cadence-LOD subset until the bounded
  unilateral lower-limb production consumer closes `play`,
  `persistence-replay`, `content-package` and applicable platform/performance
  checks. Joint-target plus fixed safety/PD and an authored skinned surface
  remain required fallbacks.

ADR-058/SPEC-35 are an Accepted parallel R&D implementation track. PhysX-only
default and Stage 0 readiness are facts only after the documented atomic
cutover and platform/replay/performance gates; the Accepted architecture alone
does not mark them complete.

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
