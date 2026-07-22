# SPEC-01: Системная архитектура

| Поле | Значение |
|---|---|
| ID | SPEC-01 |
| Статус | Accepted |
| Версия | 1.4 |
| Владелец | Core Architecture |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-002](adr/002-rust-first-ffi-and-ecs-facade.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-010](adr/010-artifact-first-headless-validation-and-review.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md) |
| Заменяет | отсутствует |

## Bounded contexts

| Context | Владеет | Не владеет / разрешённый вход |
|---|---|---|
| Core runtime | clocks, schedule, ECS facade, command transaction, events, snapshots | RPG meaning, physics internals, rendering resources |
| RPG framework | characters/items/quests/dialogues/factions/interactions и rule validation | direct ECS mutation из scripts/AI |
| Presentation | renderer, UI, audio presentation и immutable interpolation | gameplay/physics authoritative state |
| Physical embodiment | physics worlds, physical pose, motor, policy resolver/supervisor, animation/physics bridge, contacts, physical certification | quests, inventory, skill proficiency, habits/LLM planning |
| Agent intelligence | perception records, intent planning, AI memory policy | final command acceptance, motor/physics tick |
| Gameplay extensibility | mechanic registry/lock, abilities/effects/statuses, namespaced state, package resolver/reducers, authoring SDK | direct RPG/physics/ECS mutation, hidden first-party API |
| Asset & tool chain | neutral schemas, validation, cooking, bundles, migrations, inspectors | proprietary source semantics после cooking |
| Verification & evidence | scenario/impact/evidence schemas, runner orchestration, displayless capture jobs и review admission | gameplay truth, subsystem thresholds, agent-selected omissions |
| External importer | legacy parsing, provenance, mapping в NeutralImportModel | engine runtime, cooked bundle loading, embedded legacy VM |

Cross-context вызов MUST идти через engine-owned versioned contract. Общая база данных или shared mutable object между contexts запрещены.

## Public boundary

Публичная системная граница состоит только из versioned schemas и engine-owned facades в `crates/contracts`: nominal IDs, WorldCommand/DomainEvent, immutable queries/snapshots, asset/package manifests и process/plugin protocols. ECS storage, task handles, backend objects, database connections и vendor-типы являются private implementation details соответствующего context и не могут пересекать эту границу.

## Process topology

| Процесс | Состав | Обязательность | Isolation/failure |
|---|---|---|---|
| `game` | core, RPG, physics, presentation, scripting/plugin host, asset streaming | v1 MUST | authoritative local simulation; сохраняет recovery diagnostics |
| `headless` | тот же core/RPG/physics contracts без renderer/platform window | v1 MUST | deterministic gates, replay и server-like simulation tests |
| `tools` | cooker, validator, scenario/impact/evidence/review CLI, mechanic/mod/agent authoring, inspectors, package/replay CLI, optional local MCP adapter | v1 MUST; MCP optional | пишет только staging/output/approved project roots; atomic publish/changeset apply |
| `capture-worker` | common simulation/replay + renderer/audio capture, no PlatformHost window/display/input | v1 MUST для observable evidence; worker location optional | short-lived immutable job; GPU/encoder crash сохраняет CPU evidence и не публикует partial bundle |
| `editor` | будущий клиент tools/runtime contracts | v1 MUST NOT требоваться | отдельный ADR после v1 |
| `ai-host` | LLM, embeddings, ASR, TTS adapters | optional | crash/timeout → deterministic in-process fallback |
| Gothic importer | отдельный repository/process/distributable | vertical-slice integration gate | untrusted input boundary; только neutral output |

`game`, `headless` и `capture-worker` MUST использовать один command validator, schema registry, persistence code и simulation system order. Capture worker отличается только presentation target/artifact sink. Compile-time feature differences не могут менять domain semantics.

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
| Gameplay/RPG state | RPG framework внутри command transaction | presentation snapshot, inspector views | validated WorldCommand |
| ECS residency/runtime mapping | Core runtime | diagnostic indices | runtime scheduler/spawn/despawn transaction |
| Physical pose и active contacts | Physics world для active LOD | RenderPose, contact-derived DomainEvents | PhysicsBackend step/motor action |
| Motor policy route/transition/recurrent state | Motor Runtime PolicySupervisor | capability/inspector views | deterministic PolicyResolver + supervisor commit |
| Animation pose для non-physical LOD | Animation state machine | RenderPose | validated animation/controller state |
| Agent working plan/habits/tactical preferences | In-process Agent Runtime | inspector snapshot | deterministic planner + AgentArchetypeDefinition |
| RPG motor-skill proficiency | RPG Framework Character state | MotorCapabilityView/AI/mechanics query | validated progression WorldCommand |
| Mechanic package registry/state | MechanicsLock + cooked MechanicRegistry; runtime instances/state — Mechanics Runtime transaction store | package reducers/inspectors read scoped views | validated MechanicDeltaProposal committed by WorldCommand transaction |
| Durable AI memory records/indexes | Engine memory service/save segment; не содержит authoritative RPG relationship/quest/dialogue fields | `ai-host` retrieval/embedding cache | validated memory proposal/compaction transaction from DomainEvent/facts |
| Assets и world chunks | Cooked content manifest + immutable bundles | CPU/GPU caches | cooker publish only |
| Save state | Last atomically committed SaveManifest + segments | temporary migration copy | persistence transaction |
| Presentation | Presentation subsystem per frame | GPU/audio device state | snapshot consumption only |
| Import provenance | External importer manifest until neutral export; cooked provenance subset thereafter | inspector index | importer/cooker only |
| Required verification plan | ImpactResolver + VerificationPolicyManifest → ChangeImpactManifest | CI/MCP/human views | generated only; author can add, not remove |
| Evidence bundle state | EvidenceBundleManifest + content-addressed artifacts | static dossier/cache | verifier atomic publish |
| Qualitative approval | HumanReviewDecision + SPEC-11 attestation | changeset admission view | authorized human reviewer only |

Ни одна строка не допускает dual write. Если backend поддерживает собственную mutable копию, она считается implementation cache и MUST быть восстанавливаема из указанного source.

## Monorepo layering

Целевой engine monorepo MUST разделять public contracts и adapters минимум на логические группы:

```text
crates/contracts          stable schemas and nominal IDs
crates/core-runtime       schedule, command bus, ECS facade
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
crates/assets             neutral schemas, streaming, persistence
crates/verification       scenario, impact, assertions, evidence contracts/services
tools/*                   cooker, validators, mechanic/mod/agent SDK, inspectors, packaging
apps/game, apps/headless, apps/capture-worker  composition roots
lab/*                     isolated offline training application, не runtime dependency
```

Dependency direction MUST point from composition roots/backends to contracts, never from contracts to vendors. `cargo deny`/architecture tests MUST enforce forbidden dependency edges.

## Cross-context data flow

1. Platform input и AI/script/plugin proposals превращаются в typed command candidates.
2. RPG/mechanics/security validators разрешают issuer capability, package lock/schema, skill proficiency, motor capability, preconditions и budgets.
3. Core runtime упорядочивает принятые WorldCommand в fixed simulation tick.
4. Domain systems меняют owned state и публикуют DomainEvent.
5. Deterministic PolicyResolver/PolicySupervisor выбирает compatible immutable motor route; physics получает bounded physical intents/actions и владеет active pose.
6. В конце simulation frame создаются immutable PresentationSnapshot и persistence/replay deltas.
7. Presentation, inspectors и `ai-host` читают только предназначенные views.
8. Verification runner читает immutable scenario/actions/probes; capture worker воспроизводит exact replay, а evidence/review pipeline не пишет world state.

## Scheduling и concurrency boundary

Authoritative mutation выполняется только внутри объявленных simulation stages. Async I/O, shader/model compilation, AI IPC и asset decompression MUST завершаться сообщением в staging queue; результат становится видимым только на deterministic commit point. Worker task не может удерживать mutable ECS reference через await или frame boundary.

## Failure semantics

- Backend init failure выбирает declared fallback либо завершает запуск до создания mutable world.
- `ai-host` failure не останавливает simulation.
- Renderer device loss приостанавливает presentation и восстанавливает GPU caches из immutable assets; authoritative simulation MAY быть поставлена на bounded pause policy, но не реконструируется из GPU state.
- Physics backend corruption/panic является fatal для текущего simulation instance; recovery идёт из последнего save/replay checkpoint, не через продолжение с недостоверной pose.
- Tool/importer crash не публикует частичный bundle: staging directory удаляется/карантинируется при следующем запуске.
- Capture worker unavailable/crash → CPU evidence сохраняется, required observable changeset остаётся AwaitingCapability; interactive runtime не используется как fallback gate.
- Missing/tampered review evidence → changeset не admitted; gameplay/project revision не меняется.
- Unsupported developer host → fail before workspace admission с capability diagnostic; не создаётся ложный shipping result.
- Missing `TRAIN-RTX-01` capability → prototype work продолжается, но `PhysicalCertified` promotion остаётся `AwaitingCapability`.

## Архитектурные gates

| Gate | Pass threshold | Evidence | Fallback |
|---|---|---|---|
| ARCH-01 dependency direction | 0 forbidden crate edges и 0 vendor symbols в `contracts` public API | dependency graph + public API report | split adapter/crate до merge |
| ARCH-02 headless parity | Одинаковый final gameplay hash и DomainEvent sequence для 100 fixed scenarios в game-with-null-presentation и headless | replay manifests | blocking; общий composition code refactor |
| ARCH-03 process isolation | Kill/restart `ai-host` в 100 точках: 0 game crash, 0 duplicated committed command | fault-injection report | disable ai-host adapter |
| ARCH-04 single owner | Все mutable schema fields отображены ровно в одну строку ownership registry | generated ownership report | blocking architecture review |
| ARCH-05 first-party mechanics dogfood | Стрельба/магия используют только public package API; 0 private gameplay dependency или bypass | public API/link/package graph report | добавить public primitive через RFC/ADR |
| ARCH-06 first-party physical dogfood | Reference creature/axe policies используют только public physical/package SDK; 0 hidden/native creature dependency | public API/link/archetype/policy graph report | добавить public primitive через RFC/ADR; retain prototype fallback |
| ARCH-07 verification dogfood | Все first-party mechanics/creatures/presentation fixtures используют public TestScenario/Impact/Capture/Evidence contracts; 0 private mutable test API или interactive-only gate | public API/dependency/scenario graph report | expose missing public primitive; block admission |
| HOST-MAC-01 portable developer host | `cargo run -p xtask -- host-check` проходит на clean `aarch64-apple-darwin`; 0 Apple/vendor/importer type в public contracts; 0 CI/remote dependency | host/toolchain manifest, command report, dependency/API scan | fix portable boundary; Mac host claim blocked |

Release Engineering владеет выполнением; профильные команды исправляют нарушения.
