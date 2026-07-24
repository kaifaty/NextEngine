# SPEC-01: Системная архитектура

| Поле | Значение |
|---|---|
| ID | SPEC-01 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## Bounded contexts

| Context | Владеет | Не владеет / разрешённый вход |
|---|---|---|
| Core runtime | fixed tick clocks, schedule, deterministic job/result admission, memory/resource residency facade, application-session lifecycle, ECS facade, runtime entity residency, command transaction, events, snapshots | project resolution, schema/content authority, calendar/population/topology, RPG meaning, physics internals, presentation state |
| RPG framework | revisioned characters/items/quests/dialogues/factions/interactions, typed operations, transaction planning/validation | direct ECS mutation из scripts/AI, calendar/population, presentation |
| Project composition | authored ProjectManifest resolution, exact ProjectCompositionLock, configuration classes и atomic pre-world activation | OS/session lifecycle, mutable world state, runtime floating dependency resolution |
| Player experience | device-independent actions, input contexts, semantic UI, camera intent, localization и accessibility preferences | gameplay/target authority, OS events/handles, renderer resources |
| Presentation | platform normalization adapter, canonical `PresentationSnapshotV2` staging/publication at the Runtime-declared boundary, bounded `PresentationConsumptionStateV1`, material/shader/color/VFX realization and reconstructible renderer/UI/audio/VFX caches | gameplay/physics/RPG/action authoritative state, session lifecycle, native types in public contracts |
| Physical embodiment | engine-owned physics worlds/canonical snapshots, physical pose, motor observation/action/state/safety, policy resolver/supervisor, animation/root-motion/retarget/IK bridge, contacts, physical certification | quests, inventory, skill proficiency, habits/LLM planning, presentation IK as physics authority |
| Agent intelligence | perception records, intent planning, AI memory policy | final command acceptance, motor/physics tick |
| World services | WorldCalendarStateV1, population membership/schedule cursor, abstract activity, `WorldPartitionManifestV1` logical topology, durable spatial placement/tombstones, region/reservation/navigation facts | immutable content publication, RPG aggregate state, Agent plan, runtime entity mapping, physical pose |
| Gameplay extensibility | mechanic registry/lock, abilities/effects/statuses, namespaced state, package resolver/reducers, authoring SDK | direct RPG/physics/ECS mutation, hidden first-party API |
| Asset & tool chain | `SchemaRegistryManifestV1`, compatibility/migration graph, neutral schemas, `ContentManifestV1`, validation, cooking, immutable bundles/variants and inspectors | proprietary source semantics после cooking; World Services durable placement/tier |
| Verification & evidence | scenario/impact/evidence schemas, runner orchestration, displayless capture jobs и review admission | gameplay truth, subsystem thresholds, agent-selected omissions |
| External importer | legacy parsing, provenance, mapping в NeutralImportModel | engine runtime, cooked bundle loading, embedded legacy VM |

Cross-context вызов MUST идти через engine-owned versioned contract. Общая база данных или shared mutable object между contexts запрещены.

## Public boundary

Публичная системная граница состоит только из versioned schemas и engine-owned facades в `crates/contracts`: nominal IDs, ProjectManifest/ProjectCompositionLock, `SchemaDescriptorV1`/`SchemaRegistryManifestV1`, job/resource/admission descriptors, `ContentManifestV1`/neutral asset and bundle descriptors, `WorldPartitionManifestV1`/persistent spatial-object values, `PlatformCapabilitySetV1`/`PlatformEventV1`/`ApplicationSessionManifestV1`/`CloseSessionOperationJournalV1`/`RecoverySessionLinkV1`, PlayerActionFrame, typed RPG operations/views, WorldCalendar/population values, WorldCommand/DomainEvent, physics body/shape/material/joint/query/contact/`PhysicsCanonicalSnapshotV1`, motor observation/action/state and animation/root-motion/retarget/IK descriptors, `PresentationSnapshotV2`/material/shader-interface/color/VFX manifests, immutable queries/snapshots, package manifests и process/plugin protocols. ECS storage, allocator/task/thread handles, OS/window/input/filesystem/device objects, shader/model compiler objects, package-manager state, importer records, backend objects, database connections и vendor-типы являются private implementation details соответствующего context и не могут пересекать эту границу.

## Process topology

| Процесс | Состав | Обязательность | Isolation/failure |
|---|---|---|---|
| `game` | common application session, core, RPG, physics, presentation, scripting/plugin host, asset streaming | v1 MUST | authoritative local simulation; exactly-once close/save and recovery diagnostics |
| `headless` | та же application session/core/RPG/physics authority без renderer/platform window/display/surface | v1 MUST | deterministic gates, replay и server-like simulation tests; 0 interactive host attempts |
| `tools` | cooker, validator, scenario/impact/evidence/review CLI, mechanic/mod/agent authoring, inspectors, package/replay CLI, optional local MCP adapter | v1 MUST; MCP optional | пишет только staging/output/approved project roots; atomic publish/changeset apply |
| `capture-worker` | common application session/simulation/replay + displayless renderer/audio capture, no interactive PlatformHost window/display/input/surface | v1 MUST для observable evidence; worker location optional | short-lived immutable job; device/encoder/cache crash сохраняет CPU evidence and authoritative roots, publishes no partial bundle |
| `editor` | будущий клиент tools/runtime contracts | v1 MUST NOT требоваться | отдельный ADR после v1 |
| `ai-host` | LLM, embeddings, ASR, TTS adapters | optional | crash/timeout → deterministic in-process fallback |
| Gothic importer | отдельный repository/process/distributable | vertical-slice integration gate | untrusted input boundary; только neutral output |

`game`, `headless` и `capture-worker` MUST использовать один exact ProjectCompositionLock, `ApplicationSessionManifestV1` lifecycle, action-frame/command validator, `SchemaRegistryManifestV1`, `ContentManifestV1`, resource/streaming policies, `WorldPartitionManifestV1`, physics/motor/animation contracts, persistence code и simulation system order. Capture worker отличается только `DisplaylessOffscreen` presentation target/artifact sink; headless target is `None`. Compile-time feature differences не могут менять domain semantics.

## Host и shipping tiers

| Tier | Разрешённый scope | Не доказывает |
|---|---|---|
| `DeveloperHostTier/macOS-aarch64` | `contracts`, portable `runtime`, `verification`, `headless`, `xtask`, docs/boundary checks и bounded `TRAIN-MAC-P0` | macOS game/render/package support, Windows/Linux vertical conformance, runtime/training physics correspondence, `PhysicalCertified` |
| `ShippingTarget/windows-x86_64` | v1 game/headless/tools/package и platform conformance | Linux package либо training capability |
| `ShippingTarget/linux-x86_64` | v1 game/headless/tools/package, CPU headless agent workflow и displayless capture на declared GPU workers | Windows package либо RTX trainer capability |
| `TrainingCapability/local-rtx` | full physical-policy training/correspondence/certification после `TRAIN-RTX-01` | shipping conformance без соответствующих VS gates |

Bootstrap MUST выполнять один project-owned local command surface. Future CI вызывает те же `xtask`, scenario и lab application services и не является отдельным owner semantics. Отсутствие remote/CI не даёт права пропускать gate; unavailable shipping/training hardware фиксируется как `AwaitingCapability`.

## Source-of-truth matrix

| Состояние | Единственный owner/source of truth | Реплики/кэши | Разрешённая запись |
|---|---|---|---|
| Exact project composition/configuration | ProjectCompositionLock, atomically published Asset & Tool Chain closure | resolver/cache/inspector views | validated pre-world activation only |
| Gameplay/RPG state | RPG Framework aggregate registry внутри RpgTransactionPlan/command transaction | presentation snapshot, inspector views | validated WorldCommand typed operation |
| Player action/UI/camera/localization/accessibility | Player Experience manifests/session presentation state | widget/render/device caches | normalized-control resolution or local presentation preference; never direct gameplay write |
| Calendar/population/schedule/abstract activity | World Services WorldCalendarStateV1 + population segment | spatial/nav/inspector projections | validated WorldCommand/WorldAdvancePlan commit |
| ECS residency/runtime mapping | Core runtime | diagnostic indices | runtime scheduler/spawn/despawn transaction |
| Physical pose, constraints, queries and active contacts | Physical Embodiment engine-owned physics world + `PhysicsCanonicalSnapshotV1` | backend native state, RenderPose, contact-derived DomainEvents | fixed step + validated canonical MotorAction/topology transaction |
| Motor policy route/transition/recurrent state | Motor Runtime PolicySupervisor + exact SPEC-27 schemas | evaluator session, capability/inspector views | canonical batch/safety/route commit or deterministic procedural fallback |
| Animation/root-motion/IK state | Physical Embodiment animation contract; physics owns physical IK result | render/skin cache, presentation IK delta | fixed graph/retarget/IK order; root motion only validated intent |
| Agent working plan/habits/tactical preferences | In-process Agent Runtime | inspector snapshot | deterministic planner + AgentArchetypeDefinition |
| RPG motor-skill proficiency | RPG Framework Character state | MotorCapabilityView/AI/mechanics query | validated progression WorldCommand |
| Mechanic package registry/state | MechanicsLock + cooked MechanicRegistry; runtime instances/state — Mechanics Runtime transaction store | package reducers/inspectors read scoped views | validated MechanicDeltaProposal committed by WorldCommand transaction |
| Durable AI memory records/indexes | Engine memory service/save segment; не содержит authoritative RPG relationship/quest/dialogue fields | `ai-host` retrieval/embedding cache | validated memory proposal/compaction transaction from DomainEvent/facts |
| Assets и world chunks | Cooked content manifest + immutable bundles | CPU/GPU caches | cooker publish only |
| Schema/compatibility/migration authority | `SchemaRegistryManifestV1` + immutable descriptors and unique migration DAG | generated decoders, compatibility reports | validated registry publication/copy-on-write migration only |
| Runtime jobs and resources | closed job/resource policy plus deterministic admission/commit generations | worker queues, allocator/I/O/cache telemetry | Runtime admission at declared commit points; no worker direct publish |
| World topology and durable spatial placement | `WorldPartitionManifestV1` + World Services placement/tombstone segment | spatial/residency indexes and ephemeral runtime mapping | validated WorldCommand/streaming admission transaction |
| Save state | Last atomically committed SaveManifest + segments | temporary migration copy | persistence transaction |
| Application session | Runtime `ApplicationSessionStateV1`, full-request-bound `CloseSessionOperationJournalV1`, final-save ledger/receipts and `RecoverySessionLinkV1` | OS process/window state, launcher UI | validated atomic transition; journal-proven exactly-once close/save/recovery under one exact project activation |
| Presentation | Rendering Team atomically published `PresentationSnapshotV2` + exact content/profile + bounded CPU `PresentationConsumptionStateV1` | GPU/UI/VFX/cache/device state | canonical extraction/consumption and cache reconstruction only; no simulation write or acknowledged one-shot replay |
| Import provenance | External importer manifest until neutral export; cooked provenance subset thereafter | inspector index | importer/cooker only |
| Required verification plan | ImpactResolver + VerificationPolicyManifest → ChangeImpactManifest | CI/MCP/human views | generated only; author can add, not remove |
| Evidence bundle state | EvidenceBundleManifest + content-addressed artifacts | static dossier/cache | verifier atomic publish |
| Qualitative approval | `HumanReviewDecisionV2` + `AttestationEnvelopeV2`; only verified `Approve` with automatic `PASS` admission-eligible | changeset admission view | authorized human reviewer only |

Ни одна строка не допускает dual write. Если backend поддерживает собственную mutable копию, она считается implementation cache и MUST быть восстанавливаема из указанного source.

## Monorepo layering

Целевой engine monorepo MUST разделять public contracts и adapters минимум на логические группы:

```text
crates/contracts          stable schemas and nominal IDs
crates/core-runtime       schedule, command bus, ECS facade
crates/resource-runtime   jobs, memory/residency, I/O staging and backpressure
crates/rpg                generic RPG domain
crates/physics-api        engine-owned backend interface
crates/physics-*          vendor adapters
crates/physical-archetypes body/skill/policy manifests, resolver and supervisor
crates/render-api         render graph/descriptors/capabilities
crates/render-vulkan      Vulkan backend
crates/agent-runtime      deterministic AI and ai-host client
crates/mechanics          abilities, effects, statuses, package registry/state
crates/scripting          Luau host and capability surface
crates/plugin-host        WIT/Wasm host
crates/assets             schema registry, neutral content/bundles, streaming, persistence
crates/verification       scenario, impact, assertions, evidence contracts/services
tools/*                   cooker, validators, mechanic/mod/agent SDK, inspectors, packaging
apps/game, apps/headless, apps/capture-worker  composition roots
lab/*                     isolated offline training application, не runtime dependency
```

Dependency direction MUST point from composition roots/backends to contracts, never from contracts to vendors. `cargo deny`/architecture tests MUST enforce forbidden dependency edges.

## Cross-context data flow

1. Platform adapter publishes a closed `PlatformEventV1` batch with `NormalizedControlEventV1`; Player Experience creates device-independent PlayerActionFrame, and only Runtime ingress assigns a tick. AI/script/plugin/world-service work remains proposal.
2. Action-derived intent и другие proposals превращаются в typed command candidates; RPG/mechanics/security validators разрешают issuer capability, project/package lock/schema/content/partition revisions, skill proficiency, motor capability, preconditions и budgets.
3. Core runtime упорядочивает принятые WorldCommand в fixed simulation tick; multi-aggregate RPG work использует immutable RpgTransactionPlan, а async jobs/resources публикуют только canonically admitted revision-bound results.
4. Domain systems и World Services меняют только owned state, публикуют stable ordered DomainEvent и не принимают UI/camera/worker completion как authority.
5. Deterministic PolicyResolver/PolicySupervisor canonically batches `(motor_tick, PersistentId, PolicyId)`, validates learned/procedural candidate through one safety clamp, and physics publishes canonical snapshot/query/contact results.
6. Animation evaluates fixed graph/retarget/IK order; root motion returns only validated intent, while presentation IK remains non-authoritative.
7. At the Runtime-declared extraction boundary Rendering Team stages and
   atomically publishes `PresentationSnapshotV2`; Runtime/domain owners publish
   only their immutable source projections and persistence/replay deltas.
8. Presentation/material/color/VFX/cache adapters, inspectors and `ai-host` read only designated views; no derived result returns as authority.
9. Verification runner reads immutable scenario/actions/probes; displayless capture worker replays exact common session/runtime and evidence/review pipeline never writes world state.

## Scheduling и concurrency boundary

Authoritative mutation выполняется только внутри объявленных simulation stages. Async I/O, shader/model compilation, AI IPC, archive parsing и asset decompression MUST завершаться bounded revision-bound result в staging queue; admission, cancellation, memory pin/lease, eviction и backpressure следуют accepted deterministic resource policy, а результат становится видимым только на deterministic commit point. Worker task не может удерживать mutable ECS reference через await или frame boundary, выбирать commit order либо публиковать partial registry/content/world state.

## Failure semantics

- Backend init failure выбирает declared fallback либо завершает запуск до создания mutable world.
- `ai-host` failure не останавливает simulation.
- Renderer device loss invalidates presentation caches and rebuilds atomically
  from exact content/profile/`PresentationSnapshotV2` plus validated bounded
  CPU `PresentationConsumptionStateV1`; acknowledged one-shots are not replayed,
  and policy MAY request typed suspend, but authoritative
  session/simulation/snapshot roots remain unchanged.
- Physics backend corruption/panic aborts the uncommitted step and preserves the last canonical checkpoint; incompatible continuation is fatal for the instance and recovery uses the last save/replay/`PhysicsCanonicalSnapshotV1`, never native state.
- Motor evaluator/capability/performance fault never selects by wall time: exact logical hold/recovery/procedural fallback applies or the conformant commit fails.
- Session close/save/finalize fault retains the last complete session/save
  generation; exact retry follows only its durable operation-journal stage and
  returns at most one receipt. Required-save recovery links one new live session
  to the same project lock/activation and verified last-safe generation.
- Tool/importer crash не публикует частичный bundle: staging directory удаляется/карантинируется при следующем запуске.
- Capture worker unavailable/crash → CPU evidence сохраняется, required observable changeset остаётся AwaitingCapability; interactive runtime не используется как fallback gate.
- Missing/tampered V2 review payload/envelope или verified `Reject`/`NeedsChanges` → changeset не admitted; gameplay/project revision не меняется.
- Unsupported developer host → fail before workspace admission с capability diagnostic; не создаётся ложный shipping result.
- Missing `TRAIN-RTX-01` capability → prototype work продолжается, но `PhysicalCertified` promotion остаётся `AwaitingCapability`.

## Архитектурные gates

| Gate | Pass threshold | Evidence | Fallback |
|---|---|---|---|
| ARCH-01 dependency direction | 0 forbidden crate edges и 0 vendor symbols в `contracts` public API | dependency graph + public API report | split adapter/crate до merge |
| ARCH-02 composition-root parity | Одинаковые project/schema/content/resource/partition/session/physics/motor hashes, accepted command/rejection/receipt/DomainEvent sequences и final gameplay hash для 100 fixed scenarios в `game` с null presentation, `headless` и `capture-worker`; 0 composition-root semantic divergence and 0 interactive host attempt from displayless roots | replay/session manifests, lock/registry/admission/physics/motor roots, command/receipt/event traces, host-attempt audit and composition-root graph | blocking; общий authoritative-substrate refactor |
| ARCH-03 process isolation | `ai-host` absent, timeout и bad-version cases, а также kill/restart в 100 deterministic fault points: 0 blocked gameplay tick, 0 game crash, 0 duplicated committed command и 0 mandatory-outcome difference | fault-injection report, process/dependency trace and replay roots | disable ai-host adapter; deterministic in-process planner |
| ARCH-04 single owner | Все mutable schema fields отображены ровно в одну строку ownership registry | generated ownership report | blocking architecture review |
| ARCH-05 first-party mechanics dogfood | Стрельба/магия используют только public package API; 0 private gameplay dependency или bypass | public API/link/package graph report | добавить public primitive через RFC/ADR |
| ARCH-06 first-party physical dogfood | Reference creature/axe policies используют только public physical/package SDK; 0 hidden/native creature dependency | public API/link/archetype/policy graph report | добавить public primitive через RFC/ADR; retain prototype fallback |
| ARCH-07 verification dogfood | Все first-party mechanics/creatures/presentation fixtures используют public TestScenario/Impact/Capture/Evidence contracts; 0 private mutable test API или interactive-only gate | public API/dependency/scenario graph report | expose missing public primitive; block admission |
| ARCH-08 independent composition | Required composition roots depend only on Next Engine-owned contracts and contain 0 OpenGothic/legacy runtime dependency or product-specific bypass; source/dependency direction scan has 0 violations | composition-root graph, source/dependency/public API scan and architecture review record | block architecture admission; split or remove foreign/product-specific boundary |
| HOST-MAC-01 portable developer host | `cargo run -p xtask -- host-check` проходит на clean `aarch64-apple-darwin`; 0 Apple/vendor/importer type в public contracts; 0 CI/remote dependency | host/toolchain manifest, command report, dependency/API scan | fix portable boundary; Mac host claim blocked |

Release Engineering владеет выполнением; профильные команды исправляют нарушения.
