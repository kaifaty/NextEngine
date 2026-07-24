# SPEC-00: Продуктовый контракт

| Поле | Значение |
|---|---|
| ID | SPEC-00 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [INDEX-001](README.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-008](adr/008-mechanics-mod-package-and-agent-authoring-model.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

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
13. принимать observable changes только по automatic evidence `PASS` + hash-bound `HumanReviewDecisionV2`/`AttestationEnvelopeV2` без обязательного interactive launch;
14. позволять начать portable core/tooling разработку на `aarch64-apple-darwin` через local-only host gate, не объявляя macOS shipping support;
15. разрешать authored ProjectManifest в один exact ProjectCompositionLock, общий для `game`, `headless` и `capture-worker`;
16. принимать device-independent PlayerActionFrame и обеспечивать semantic UI/camera/localization/accessibility без presentation authority над gameplay;
17. хранить generic RPG state в revisioned single-owner aggregates и применять multi-aggregate typed operations только атомарным RpgTransactionPlan;
18. сохранять World Services-owned calendar/population state и одинаковые mandatory outcomes при stepped/bulk time и разрешённых residency tiers;
19. активировать один immutable `SchemaRegistryManifestV1` с совместимостью и unique copy-on-write migration path для каждого authoritative schema;
20. выполнять jobs, memory/resource residency и I/O только через bounded deterministic admission/backpressure contracts без authoritative drop;
21. cook/load один `ContentManifestV1` и neutral scene/mesh/material/texture/skeleton/animation/audio/collision/navigation/world schemas без source/vendor types;
22. сохранять один `WorldPartitionManifestV1`, durable spatial objects/cross-chunk references и exact tier/streaming admission semantics;
23. исполнять physics только через engine-owned body/shape/material/joint/query/contact/`PhysicsCanonicalSnapshotV1` contracts с canonical units/order и явной exact/quantized/tolerance classification;
24. принимать motor observation/action/state только по exact schemas, canonical `(motor_tick, PersistentId, PolicyId)` batching, deterministic safety clamp и procedural fallback, а animation root motion — только как validated intent;
25. запускать `game`, `headless` и `capture-worker` через один `ApplicationSessionManifestV1`/closed lifecycle, причём headless/capture-worker не создают interactive window/surface/display;
26. публиковать только immutable `PresentationSnapshotV2`, neutral material/shader/color/VFX contracts и reconstructible caches без обратной записи presentation в simulation;
27. проходить единый набор gates [SPEC-12](12-vertical-slice-conformance.md).

## Product invariants

- Gameplay state MUST изменяться только принятыми `WorldCommand`.
- У каждого изменяемого состояния MUST быть ровно один authoritative owner.
- Simulation correctness MUST NOT зависеть от renderer frame rate, LLM response или network availability.
- RuntimeEntityId MUST оставаться ephemeral; persistent contracts используют PersistentId/AssetId.
- Vendor types MUST NOT пересекать engine-owned backend boundary.
- Imported proprietary source data MUST NOT попадать в engine repository или distributable.
- Baseline renderer MUST запускаться без RT и mesh shaders.
- Release artifact MUST быть воспроизводимо связан с source/content/model/license manifests.
- Runtime MUST активировать только immutable exact ProjectCompositionLock; floating dependency resolution, environment/user override authoritative configuration и partial registry publication запрещены.
- Device binding, UI/widget state, camera interpolation, localized text и accessibility preferences MUST NOT становиться gameplay/target authority.
- RPG aggregate, population/calendar, Agent plan, runtime residency и physical pose MUST сохранять отдельных единственных owners.
- Schema field IDs MUST быть immutable и non-reusable; incompatible либо
  ambiguous migration MUST fail closed over a complete copy without publishing
  any partial registry/save/content generation.
- Async job completion, cache warmth, memory pressure and I/O order MUST NOT
  select authoritative outcome. Required work is bounded/deferred/rejected with
  an exact diagnostic, never silently dropped.
- Neutral content, target variants, partition topology, durable spatial
  placement and cross-chunk references MUST remain engine-owned public schemas;
  source/importer, OS, ECS, vendor and backend types remain private.
- Physics/motor/animation public contracts MUST be engine-owned and backend-free.
  Physics/query/contact/snapshot order and applied motor actions are exact after
  declared quantization; diagnostic tolerance cannot select or waive gameplay.
- Platform/session callbacks, wall time, device loss and renderer cadence MUST
  NOT select input tick, session outcome, physical action or gameplay result.
  Close/save is exactly-once and displayless roots create no interactive host.
- Presentation snapshots/materials/shader interfaces/color/VFX/cache manifests
  are immutable/reconstructible. Bounded CPU
  `PresentationConsumptionStateV1` is presentation-session recovery metadata,
  remains outside invalidatable caches and gameplay saves/hashes, and prevents
  acknowledged one-shot replay. Presentation state never writes back to
  simulation; exact pixels are claimed only for pinned capture.
- First-party gameplay mechanics MUST NOT использовать hidden API, недоступный community package.
- First-party physical creatures/policies MUST использовать тот же archetype, certification и authoring contract, что community package.
- Gameplay learning MUST NOT дообучать neural weights в runtime; progression изменяет RPG state и выбирает immutable evaluated policy route.
- Coding agent MUST проходить те же capability, validation, test, provenance и human/owner review paths, что человек.
- Mandatory acceptance MUST NOT требовать monitor, input device, hidden window, interactive runtime или manual-only checklist.
- Required test subset MUST определяться engine-owned impact resolver; agent/author может только расширить его.
- Только verified `HumanReviewDecisionV2::Approve` при всех required automatic gates `PASS` может допустить exact changeset/evidence hashes; `Reject` и `NeedsChanges` остаются signed non-admitting feedback и не могут override automatic failure.
- Developer-host compatibility MUST NOT считаться shipping/platform conformance; Mac training smoke MUST NOT заменять physics correspondence или `PhysicalCertified` gates.
- `PhysicalCertified` promotion MUST оставаться `AwaitingCapability`, пока не пройден `TRAIN-RTX-01` на поддерживаемом Linux/NVIDIA host; `PrototypeFallback` от этого не блокируется.

## Scope и non-goals

В v1 не входят full scene editor, multiplayer/network replication, macOS game/render package или shipping support, consoles, mobile, обязательный cloud account, полный Gothic II compatibility, Daedalus/Ikarus/LeGo runtime, перенос OpenGothic code и публикация игровых Gothic assets. Portable developer-host tier на Apple Silicon разрешён ADR-011 и не расширяет shipping promise. SDK фиксируется только до контрактов, нужных vertical slice; весь Rust/Luau/WIT API преждевременно не замораживается.

## Пользователи и поддерживаемые workflows

| Роль | Обязательный workflow | Не обещается v1 |
|---|---|---|
| Игрок | install → offline launch → play → save/load | account/cloud sync |
| Gameplay author | validated neutral data + Luau → cook → test/replay | визуальный full editor |
| Engine developer | portable Rust workspace на Mac/Linux/Windows → local host-check → impact plan → CPU headless gates → optional capture job → Windows/Linux platform package | macOS shipping package; stable ABI для внутренних crates |
| Plugin author | WIT SDK → capability manifest → validate/package | native DLL injection |
| Мододел | public mechanics/physical SDK → package/prototype → scenarios/replay/certification → local install/share | implicit load-order monkey patching или self-certification |
| Coding agent | exact context bundle → scoped changeset → resolved affected tests → diagnose/minimize → evidence/review/apply | privileged ECS/shell/runtime mutation или self-approval |
| Researcher | body/skill config → offline train/export → ONNX parity/runtime suite → certification review | LLM-controlled physics или runtime weight learning |
| Importer user | указать локальную установку → inspect → neutral export → cook | bundled copyrighted content |

## Ownership и публичная граница

Этот Accepted product contract является source of truth для scope, compatibility promise и release definition; Repository Owner владеет его изменением, но не владеет subsystem state. Внешняя граница продукта состоит из ProjectManifest/ProjectCompositionLock, `SchemaDescriptorV1`/`SchemaRegistryManifestV1`, job/resource/backpressure descriptors, `ContentManifestV1`/neutral bundle schemas, `WorldPartitionManifestV1`/durable spatial-object contracts, player action/semantic projection, typed RPG aggregate/operation, world calendar/population, engine-owned physics/query/contact/snapshot and motor/animation schemas, `ApplicationSessionManifestV1`/`CloseSessionOperationJournalV1`/`RecoverySessionLinkV1`/normalized platform events, `PresentationSnapshotV2`/material/shader-interface/color/VFX contracts, verification/scenario/impact/evidence manifests, MechanicsLock/package schemas, creature/policy/skill manifests, save/replay schemas, Luau capability API, WIT plugin worlds, AuthoringContextBundle/AgentChangeSet, `HumanReviewDecisionV2`/`AttestationEnvelopeV2`, `ai-host` IPC и CLI/JSON contracts. Внутренние Rust crates, ECS layout, task handles, OS/window/device/compiler objects, importer records и backend APIs не являются стабильным SDK до отдельного ADR.

## Product data flow

`source/imported data + mechanic/creature/policy packages → deterministic ProjectManifest resolution → ProjectCompositionLock + SchemaRegistryManifestV1 + ContentManifestV1 + MechanicsLock → immutable neutral bundles/WorldPartitionManifestV1 → bounded resource/streaming admission → ApplicationSessionManifestV1 for game/headless/capture-worker → PlatformEventV1/PlayerActionFrame/proposals → WorldCommand/RpgTransactionPlan/DomainEvent/WorldAdvancePlan → physics/motor/animation owner commits → save/replay + immutable PresentationSnapshotV2 → material/color/VFX/cache presentation`. Для development: `AgentChangeSet → ImpactResolver → CPU scenarios → optional displayless capture → EvidenceBundle → HumanReviewDecisionV2/AttestationEnvelopeV2 when required`. Optional `ai-host`, Luau, Wasm и agent tooling могут предлагать commands/changesets, но не обходят validation. Platform/presentation/capture consumers, async workers и training tools не возвращают mutable model/gameplay state в simulation.

## Failure semantics

Ошибка optional feature MUST приводить к bounded degradation: AI/procedural-motor fallback, acoustic fallback, lower render/physics/animation tier, declared content/material/VFX variant либо disabled plugin. Ошибка authoritative data — corrupt save, incompatible/ambiguous schema or migration, missing required bundle/resource/partition/physics/policy reference, invalid command — MUST быть fail-closed с structured diagnostic и без частично применённого registry/content/world/physics/save/session состояния. Queue/resource pressure MUST deterministically defer or reject bounded work and MUST NOT drop an authoritative operation. Platform/session/device/cache fault сохраняет последний complete session/save/snapshot и exactly-once receipt; presentation fallback не меняет gameplay. Panic/crash одного tool/worker/ai-host process MUST NOT повреждать source assets, active world generation или последний валидный save.

## Implementation conformance

Implementation может заявить `vertical-v1 conformance` только когда все обязательные gates SPEC-12 имеют `PASS`, artifacts проходят hash/provenance validation и traceability не содержит незакрытых требований. Частичный demo не является v1 conformance. Непрохождение implementation gate не переписывает принятую architecture baseline: оно опровергает backend/implementation hypothesis и запускает указанный fallback/ADR process.
