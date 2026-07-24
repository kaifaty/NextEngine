# Глоссарий Next Engine

| Поле | Значение |
|---|---|
| ID | GLOSSARY-001 |
| Статус | Accepted |
| Версия | 1.8 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | INDEX-001 |
| Заменяет | отсутствует |

Термины ниже имеют одинаковый смысл во всех RFC, schemas, CLI и diagnostics. Публичные контракты MUST использовать эти имена или явно версионированные производные.

| Термин | Нормативное определение |
|---|---|
| **Next Engine** | Временное нейтральное имя независимого специализированного AI-first RPG engine. |
| **Core runtime** | Детерминированное ядро scheduling, ECS facade, команд, событий, snapshots и persistence hooks. |
| **RPG framework** | Generic доменная модель персонажей, предметов, квестов, диалогов, фракций и интерактивных объектов. |
| **Backend** | Заменяемая реализация engine-owned interface. Vendor-типы не входят в interface. |
| **DeveloperHostTier** | Поддержанный host для portable source/tool development и локальных gates, не создающий shipping/platform conformance claim; v1 включает `macOS-aarch64` по ADR-011. |
| **TrainingCapabilityStatus** | `Available`, `AwaitingCapability` либо `Rejected` для exact hardware/backend profile; не выводится из наличия model file и не является package certification. |
| **AwaitingCapability** | Blocking non-PASS state: обязательный gate известен, но подходящий GPU/encoder/reviewer/training host недоступен; completed independent evidence сохраняется. |
| **RuntimeEntityId** | Эфемерный идентификатор живой ECS entity. Действует только в пределах одного runtime instance и не сериализуется. |
| **PersistentId** | Стабильный 128-bit opaque ID сущности или логического объекта между save/load, chunks и replay. Не кодирует ECS layout или vendor handle. |
| **AssetId** | Стабильная ссылка на логический asset; конкретная cooked revision определяется manifest и content hash. |
| **ContentHash** | SHA-256 канонических cooked bytes и параметров cooker, используемый для immutable bundle addressing. |
| **ProjectManifest** | Versioned authored project intent с typed dependency ranges, roots, profiles, policies и provenance; не является runtime lock и не активируется до deterministic resolution. |
| **ProjectCatalogSnapshot** | Immutable JCS-canonical, externally hash-bound resolver input с полными dependency records, hashes, compatibility/trust/budget metadata и `yanked` state; registry/network/cache state не входит в него. |
| **ProjectCompositionLock** | Immutable content-addressed exact closure project, catalog, engine/schema/content/package/trust/budget/config/model и migration hashes, общая для `game`, `headless` и `capture-worker`; runtime не разрешает из неё floating ranges. |
| **SchemaDescriptorV1** | Immutable engine-owned schema declaration со stable schema/field IDs, wire shape, encoding, role, compatibility policy and canonical hash; storage/backend layout не является schema. |
| **SchemaRegistryManifestV1** | Exact content-addressed set accepted schema descriptors, compatibility entries and migration authority, bound by `ProjectCompositionLock`; partial registry publication forbidden. |
| **ContentManifestV1** | Immutable catalog root exact neutral asset revisions, typed dependency closure, provenance, bundle/blob hashes and target-variant policy; runtime resolves no floating content revision. |
| **ContentBundleV1** | Logical immutable bundle descriptor plus exact bounded blob set with canonical root and atomic publication; archive/container layout is private. |
| **VariantFallbackPlanV1** | Lock-bound canonical trace from one requested target profile through its declared same-target capability fallback pointers to the first supported effective profile; runtime never re-scores or re-traverses it after activation. |
| **MemoryBudgetProfileV1** | Hash-bound finite owner/pool/charge/reserve limits for runtime/tool memory admission; measured allocator/RSS/device state cannot choose authoritative outcome. |
| **ComputeResourceKeyV1** | Qualified nominal identity of a reconstructible compute/content resource generation; not an unqualified resource ID, pointer, task or backend handle. |
| **ComputeResourcePinV1** | Bounded owner/reason/lifetime record preventing eviction of an exact qualified resource generation; unrelated to `WorldResidencyTier`. |
| **ProjectResolutionConflictReport** | Canonically ordered deterministic resolver failure over exact manifest/catalog/profile hashes; partial lock или registry при его создании не публикуется. |
| **LaunchProfile** | Versioned composition-root/target/capability/tick/profile descriptor, чьи authoritative values входят в exact project lock, а operational output paths не становятся domain state. |
| **ConfigurationClass** | Exactly-one classification каждого project key как `Authoritative`, `PresentationOnly` или `DeveloperOnly`, определяющая owner, override и save/replay/hash semantics. |
| **CommandStreamId** | Стабильный opaque ID логического потока команд одного principal в exact world identity namespace; вместе с `sequence` участвует в ledger admission и arrival-independent order, но не является runtime task/thread handle. |
| **IssuerPrincipal** | Closed tagged engine-owned identity источника `WorldCommand` (`Player`, `Agent`, `Package`, `Script`, `Plugin`, `Tool` или `InternalSystem`) с versioned payload; не содержит backend/session object. |
| **CanonicalBinaryV1** | Версионированное каноническое бинарное кодирование fixed-width integers, lengths, ordered fields и collections для hash-bound authoritative данных; platform-native layout и unordered iteration запрещены. |
| **CanonicalCommandBodyV2** | Closed canonical command body без `command_id`, arrival metadata и wall time: schema/kind, principal, stream, sequence, target tick/phase, deterministic payload, capabilities и preconditions. Exact canonical bytes являются единственным input domain-separated command-ID V2 hash. |
| **WorldCommandEnvelopeV2** | Public ingress envelope с optional `claimed_command_id`, `CanonicalCommandBodyV2` и non-authoritative transport metadata; computed ID существует только как validator result и проверяется против claim до ledger reservation. |
| **WorldCommand** | Единственный валидируемый запрос на изменение authoritative gameplay state; current public representation — `WorldCommandEnvelopeV2`. Command body не содержит собственного `command_id`. |
| **WorldIdentityManifestV1** | Immutable world namespace manifest, связывающий project/world generation, identity algorithm versions и collision domains для command, event и durable object IDs. |
| **CommandReceiptV1** | Persisted terminal result admitted command либо collision set: exact body reference/hash membership, stream/sequence, committed/rejected/collision outcome, event IDs, finalized tick и transaction-result root. |
| **CommandLedgerV2** | Per-world persisted ledger с per-stream high-watermarks/pending/state, body archive/global ID bindings и ровно `min(finalized_receipt_count, 4096)` recent receipts; после 4 096 finalizations окно содержит ровно 4 096 records. |
| **TickRateProfileV1** | Versioned integer profile gameplay/physics/motor rates; вместе с `IngressAssignmentProfileV1` задаёт current/next cutoff, в котором wall time не определяет authoritative target tick. |
| **RngStreamStateV1** | Persisted state declared ChaCha12 authoritative RNG stream, выводимого из world seed и canonical descriptor; counter/index входит в save/replay, а cross-platform vectors exact. |
| **ScheduleManifestV1** | Immutable system DAG с stable `SystemId`, declared reads/writes/stage, logical shard plans, reducer bindings и canonical topological tie-break; worker completion order не является execution/commit order. |
| **DomainEvent** | Неизменяемый факт об уже принятом изменении domain state. Не является альтернативным mutable API. |
| **PresentationSnapshotV2** | Atomically published immutable scene/camera/semantic-UI/cue projection with snapshot epoch, sequence and stable object keys. Renderer/audio/UI/VFX consume it read-only; it never writes back or enters gameplay hashes. |
| **PresentationConsumptionStateV1** | Bounded Rendering-owned CPU presentation-session recovery state over cue prefix, one-shot acknowledgment root, pending-one-shot map and active continuous-instance map; it survives cache invalidation but never enters gameplay saves/hashes or writes to simulation. |
| **PlatformCapabilitySetV1** | Canonical normalized engine-owned host capability/limit set selected before runtime staging; it contains no native device, extension or backend type. |
| **PlatformTimebaseV1** | Checked monotonic native-sample normalization profile for platform diagnostics/order; it cannot compute simulation/world tick. |
| **PlatformEventV1** | Bounded typed normalized platform fact with stable host/source sequence and identity; callback order is not authority. |
| **NormalizedControlEventV1** | Bounded engine-owned semantic control value from a private device adapter; native object, wall time and target tick are excluded. |
| **ApplicationSessionManifestV1** | Immutable composition-root session closure over exact project/launch/platform/runtime/schema/content/recovery/shutdown/presentation-target hashes. |
| **ApplicationSessionStateV1** | Runtime-owned revisioned state in the closed `Created → CompositionStaged → RuntimeStaged → Active ↔ Suspended → Quiescing → Finalizing → Closed` lifecycle. |
| **CloseSessionOperationJournalV1** | Durable full-request-bound progress journal that lets an exact close retry execute only its next missing edge/save step without revalidating the historical starting revision. |
| **CloseSessionResultV1** | Closed terminal session result `Saved | ClosedUsingLastSafeGeneration`; only `Closed` publishes its receipt, while retry-pending or required-save failure remains typed non-Closed progress in `Finalizing`. |
| **RecoverySessionLinkV1** | Durable link from an immutable failed `Finalizing` session to one new live session, binding the exact prior state/manifest, same project lock and verified last-safe save; it is not a lifecycle edge or new project revision. |
| **ActionMapManifest** | Immutable map stable device-independent action IDs to bounded semantic controls, contexts, conflict policy and accessibility metadata. |
| **PlayerActionFrame** | Canonical immutable ordered device-independent action input for one ingress sample; it contains no physical-device identity, wall time, target tick, camera transform or backend object. |
| **TargetingIntent** | Non-authoritative assigned player proposal containing quantized aim and gameplay query/profile identifiers; authoritative targeting is reconstructed from the assigned-tick snapshot, never rendered camera/depth data. |
| **UiSemanticSnapshot** | Immutable stable screen/control-role/action-affordance projection for UI; widget trees, localized strings as identity and mutable domain references are excluded. |
| **PlayerPreferenceProfile** | Local versioned `PresentationOnly` bindings/accessibility/localization/presentation settings whose resulting canonical action frame may enter production input but whose stored values are not gameplay authority. |
| **AgentIntent** | Высокоуровневое намерение AI, ещё не имеющее права изменять мир. Проходит policy/rules validation и преобразуется в WorldCommand либо rejection. |
| **PhysicalAvatarIntent** | Ограниченный по времени запрос locomotion/posture/manipulation к motor controller; не задаёт physics pose напрямую. |
| **MotorObservation** | Versioned numeric observation, вычисленная из physics state и разрешённого gameplay context. |
| **MotorAction** | Ограниченный versioned набор joint targets/torques или controller parameters, прошедший safety clamp. |
| **MotorObservationSchemaV1** | Exact engine-owned observation tensor dtype/rank/shape/feature order/unit/normalization contract bound to a policy route. |
| **MotorActionSchemaV1** | Exact engine-owned action tensor/channel/unit/bounds contract; every learned or procedural candidate passes the same fixed-point safety clamp. |
| **MotorInferenceBatchV1** | Closed inference batch canonically ordered by `(motor_tick, PersistentId, PolicyId)`; worker/completion/wall-time order cannot select action. |
| **PolicyStateRecordV1** | Canonical bounded recurrent motor state with exact subject/policy/bundle/schema/route identity and explicit reset/migration/save rules. |
| **PhysicsBackend** | Engine-owned interface для worlds, bodies, articulations, queries, contacts и state snapshots. |
| **ContactEvent** | Нормализованное backend-independent begin/persist/end событие с continuity `contact_id`, PersistentId участников, body slots, material tags, point/normal, relative velocity, impulse/effective mass и physics tick. |
| **PhysicsCanonicalSnapshotV1** | Portable engine-owned canonical physical checkpoint over descriptors, bodies, joints, continuation and contact continuity; native backend snapshot bytes are not authoritative. |
| **PhysicsCoordinateProfileV1** | Exact right-handed metres/kilograms/seconds/radians and quantization profile used at the public physics boundary. |
| **RootMotionIntentV1** | Revision-bound animation-produced locomotion proposal accepted only through the normal motor/command validation path; never a pose teleport. |
| **PhysicalIkConstraintV1** | Bounded authoritative physical IK constraint resolved inside declared physics/motor order. |
| **PresentationIkRequestV1** | Presentation-only IK request that may alter render pose but cannot change contacts, gameplay or physics snapshot. |
| **CrossTargetProjectionRoot** | Exact root canonical quantized physical projection для cross-target replay comparison. Raw backend samples могут иметь отдельно declared tolerance, но IDs, event classes, gameplay outcomes и projection root остаются exact. |
| **RenderPose** | Read-only presentation pose, построенная из authoritative physics pose либо animation pose согласно physics LOD. |
| **Physics LOD** | Разрешённый уровень embodied simulation: full articulation, simplified active ragdoll, capsule/animation или abstract simulation. |
| **CreatureArchetypeManifest** | Public immutable manifest, связывающий generic Character, physical archetype, AgentArchetypeDefinition, mechanic packages, provenance и certification без package-specific runtime type. |
| **PhysicalArchetypeBundle** | Cooked body/LOD/capability/policy package одной physical morphology revision с exact descriptors, fallbacks и evidence references. |
| **PhysicalCertificationLevel** | `PrototypeFallback` либо `PhysicalCertified`; уровень заявляет доказанное physical behavior, а не доверие к publisher. |
| **MorphologyFamilyId** | Stable namespaced ID семейства совместимых body/motor representations; не означает совместимость без exact PolicyCompatibilityKey. |
| **MotorSkillId** | Stable namespaced ID физически исполняемого навыка, независимый от конкретной model revision. |
| **SkillProficiency** | RPG-owned unsigned fixed-point `u16` 0…10 000, сериализуемый уровень владения skill; не neural weight. |
| **MotorPerformanceEnvelope** | Измеримые distribution-level bounds physical skill по точности, контактам, времени, энергии, балансу, recovery и safety. |
| **MotorCapabilityView** | Read-only planner/mechanics view состояния skill route: `Unavailable`, `NoviceFallback`, `PendingActivation` или `Active`. |
| **MotorPolicyBundleManifest** | Immutable manifest model bytes, schemas, compatibility, skill envelope, state, runtime cost, provenance, evaluation и fallback. |
| **PolicyCompatibilityKey** | Canonical hash body revision, morphology family, topology mask, observation/action/normalization schemas, actuator profile и runtime/training correspondence profile. |
| **PolicyActivationPlan** | Deterministic resolver result с candidate/previous/fallback routes и preconditions для PolicySupervisor; сам не активирует model. |
| **ActivePolicyRoute** | Motor Runtime-owned committed route, являющийся единственным neural/procedural producer source для разрешённой composition. |
| **PolicyState** | Versioned bounded recurrent state policy с canonical serialization и explicit reset/handoff semantics. |
| **AgentArchetypeDefinition** | Immutable generic AI asset с traits, routines, senses, tactical preferences, memory/relationship priors и initial skill loadout; не содержит MotorAction. |
| **RpgAggregateEnvelopeV1** | Versioned durable wrapper одного RPG aggregate с `PersistentId`, schema/kind, revision, canonical payload и definition/provenance hashes; mutable backend/ECS/presentation state запрещён. |
| **RpgCommandV1** | Closed bounded RPG payload of contiguous typed `RpgOperationV1` slots and revision/hash preconditions; it remains a proposal inside the common `WorldCommand` path. |
| **RpgOperationV1** | Closed typed revision-checked RPG-domain operation; first-party, package, script, plugin, AI and tool proposals use the same variants and validators. |
| **RpgTransactionPlan** | Immutable RPG-Framework-produced all-or-nothing plan with canonical read/write sets, staged aggregate bytes and ordered events; external packages cannot construct or mutate it. |
| **NeutralAuthoringModel** | Engine-owned промежуточная asset/world schema между source importers и cooker. |
| **NeutralImportModel** | Строго ограниченный переносимый результат внешнего importer, являющийся подмножеством NeutralAuthoringModel с provenance. |
| **WorldChunk** | Версионированная единица streaming с bounds, dependencies, PersistentId namespace и content hashes. |
| **WorldPartitionManifestV1** | Immutable exact topology of regions, cells, anchors, chunk bindings and schema/content/resource-policy hashes used by cooker, persistence and streamer. |
| **SpatialPlacementStateRootV1** | Domain-separated canonical root of one mutable durable `WorldPlacementStateV1` generation, distinct from the immutable initial-placement catalog hash and validated across command/save/replay/migration transitions. |
| **PersistentSpatialObjectV1** | World Services-owned durable placement/tombstone record retaining one `PersistentId`; it contains no runtime entity, RPG, Agent, physical-pose or presentation state. |
| **StreamingAdmissionPlanV1** | Immutable revision-bound canonical expansion/rank/admission/eviction/defer plan for whole dependency groups, published only at a deterministic commit point. |
| **TierExecutionProfile** | Project-locked allowed activity/cadence/capability contract for one `WorldResidencyTier`; it does not own job scheduling or physical LOD. |
| **WorldCalendarStateV1** | World Services-owned integer/revision-bound mapping from `world_tick` to simulation calendar units; host locale, timezone and wall clock are never authority. |
| **PopulationRecord** | World Services-owned durable subject record retaining one `PersistentId`, logical region/schedule, declared abstract capabilities and `WorldResidencyTier`, but no RPG, Agent, physical-pose or runtime-entity state. |
| **WorldResidencyTier** | World Services lifecycle level `Dormant`, `Abstract`, `Simulated` or `Active`; it controls durable population execution/residency and is explicitly orthogonal to physical LOD. |
| **WorldAdvancePlanV1** | Immutable bounded revision-checked ordered World Services work for a world-tick interval; stepped and bulk execution must produce the same plan boundaries and committed results. |
| **SaveManifest** | Корень сохранения: schema versions, build compatibility, world revision, loaded chunks, content hashes и ordered state segments. |
| **ReplayManifest** | Корень replay: initial checkpoint, world/runtime/content hashes, named RNG state, `CommandLedgerV2` snapshot/root и ordered `ClosedIngressBatchV1`/`ClosedCommandAdmissionBatchV2` boundaries со всеми bounded authenticated/decodable command envelopes, включая duplicates, conflicts и deterministic rejections. |
| **RunManifest** | Машиночитаемый манифест проверки: scenario, build, configs, hashes, metrics и artifacts. |
| **VerificationPolicyManifest** | Project-owned versioned policy profiles, impact rules, observable categories, budgets, reviewers, baselines, quotas и redaction. |
| **TestScenarioManifest** | Immutable generic deterministic scenario: fixtures, seeds/clocks, actions/faults, probes/assertions, captures, profiles и expected evidence. |
| **ScenarioAction** | Timestamped production input/validated command/lifecycle/declared fault action; не mutable test hook. |
| **ProbeSpec** | Read-only selector documented state/event/diagnostic/normalized metric, не влияющий на schedule или state hash. |
| **AssertionSpec** | Versioned oracle с owner, exact/tolerance/distribution/invariant/presentation/qualitative kind, explicit threshold и stable failure code. |
| **ChangeImpactManifest** | Generated immutable result ImpactResolver: changed nodes/reasons, required suites/profiles/long gates, observable categories, captures и review flag. |
| **OffscreenPresentationTarget** | Engine render target в GPU images/readback без PlatformHost window/surface/swapchain/display server. |
| **CapturePlan** | Scenario-owned camera/audio/view/overlay/timeline/output specification для reproducible media. |
| **CaptureJobManifest** | Portable immutable displayless worker job с exact base/candidate/content/replay/profile/CapturePlan hashes и artifact requirements. |
| **MaterialDefinitionV1** | Neutral immutable material parameter/texture/render-state/shader-interface schema with exact fallback and no compiled pipeline/device object. |
| **MaterialInstanceV1** | Canonical values and texture bindings conforming to one exact material definition; it cannot alter interface or add backend bindings. |
| **ShaderInterfaceManifestV1** | Neutral stage/input/resource/output semantic layout contract validated against target artifacts; compiler/backend objects are excluded. |
| **PresentationPipelineKeyV1** | Domain-separated hash of exact shader artifact/interface, render state, layout, target, specialization and capability-path fields; cache/insertion/device identity is excluded. |
| **SdrColorProfileV1** | Mandatory exact sRGB-D65, linear-compositing, straight-boundary/premultiplied-working alpha and rounding/output-transform contract; pinned capture fixes exact pixels. |
| **VfxCueV1** | Event-derived stable presentation-only VFX lifecycle record with canonical dedupe/cancel/fallback semantics and no simulation write authority. |
| **PresentationCacheManifestV1** | Reconstructible GPU/UI/VFX cache generation over exact content/profile/snapshot inputs; corruption/device loss preserves authoritative state. |
| **EvidenceBaselineManifest** | Immutable human-approved scenario/profile/toolchain/content semantic/media reference; generated candidate не является approved baseline. |
| **EvidenceBundleManifest** | Root content-addressed closure, объединяющий automatic results, impact, diagnostics, replay, metrics, raw media roots, review artifacts и baselines. |
| **AttestationEnvelopeV2** | Closed JCS envelope с exact project/domain/payload type, signed `key_id`, payload/trust/revocation hashes и Ed25519 signature; cryptographic verification отделена от admission. V1 доступен только historical-audit verifier. |
| **ReviewerTrustManifest** | Project-owned offline manifest разрешённых reviewer keys/roles, validity windows и revocations; agent, tool и service keys не получают human-review authority. |
| **HumanReviewDecisionV2** | Closed JCS payload с exact enum `Approve`, `Reject` или `NeedsChanges`, привязанный через `AttestationEnvelopeV2` к project, changeset, evidence, baseline, policy, impact, fixture, automatic summary, `requirement_graph_sha256` и `gate_descriptor_set_sha256`. Все три решения подписываемы; только verified `Approve` вместе с automatic `PASS` admission-eligible; trace/gate drift требует нового payload/decision/signature. |
| **HumanReviewRequired** | Impact state, при котором successful automatic gates недостаточны и exact changeset требует verified `HumanReviewDecisionV2::Approve`; `Reject` и `NeedsChanges` остаются signed non-admitting feedback. |
| **Capability** | Явно выданное право script/plugin/process на именованную операцию или data view. Default — deny. |
| **MechanicPackageId** | Стабильный namespaced ID gameplay/mod package; версия и publisher identity являются отдельными полями. |
| **MechanicPackageManifest** | Author-declared metadata, dependencies, capabilities, exports, state schemas/migrations, provenance, licenses и tests package. |
| **MechanicsLock** | Generated exact package closure: versions, hashes, dependency/patch order, granted capabilities и schemas, используемые save/replay/runtime. |
| **AbilityDefinition** | Immutable cooked описание активной/пассивной способности: requirements, targeting, costs, cooldown, phases, effects и cues. |
| **AbilityInstance** | Runtime state machine конкретного выполнения AbilityDefinition с causal command, phase, targets и interruption state. |
| **MechanicAffordance** | Machine-readable описание способности для planner/authoring tools: preconditions, target, cost/time/risk, expected outcome range и failures. |
| **EffectRequest** | Валидируемое предложение применить semantic effect от source к target в заданном context. |
| **EffectTransaction** | Каноническая atomic composition одного validated `RpgTransactionPlan` и Mechanics Runtime delta; либо committed полностью, либо не applied. |
| **StatusInstance** | Versioned runtime instance временного/постоянного gameplay status со stacking, immunity, duration и source provenance. |
| **MechanicReducer** | Bounded deterministic Luau/Wasm function, преобразующая immutable context, command и own state в MechanicDeltaProposal. |
| **MechanicDeltaProposal** | Неавторитетное предложение namespaced state patch, EffectRequests, future commands, event payloads и presentation cues; authoritative DomainEvent создаётся engine только после commit. |
| **PackagePatch** | Явное изменение чужого definition с target ID/path, expected revision hash и conflict policy; не implicit file override. |
| **AuthoringContextBundle** | Замкнутый machine-readable SDK/context snapshot для человека или coding agent, привязанный к exact engine/project/package hashes. |
| **AgentChangeSet** | Reviewable набор bounded edits/operations с base hashes, provenance, tests, risk flags, dry-run и atomic apply semantics. |
| **AgentPolicy** | Project-owned правила разрешённых authoring roots/tools/budgets и условий automatic/reviewed AgentChangeSet apply. |
| **PresentationCue** | Semantic non-authoritative запрос gameplay package на VFX/audio/UI/camera feedback. |
| **ai-host** | Отдельный процесс для LLM, embeddings, ASR и TTS. Не участвует в deterministic simulation tick. |
| **Headless runtime** | Тот же core/RPG runtime без renderer и интерактивного platform shell, предназначенный для validation, replay и tests. |
| **Cooker** | Детерминированный tool, преобразующий validated NeutralAuthoringModel в immutable content-addressed bundles. |
| **Conformance gate** | Воспроизводимая pass/fail проверка с владельцем, threshold, командой/сценарием и evidence artifacts. |
| **GateDescriptorV1** | Machine-readable descriptor полного gate ID с единственным `descriptor_source_id`, classification/subject, owner, applicability, scenario, threshold, evidence, fallback, requirement/failure links, child/parent links и VS/profile closure. Несовместимое повторное semantic definition является `GATE_DESCRIPTOR_CONFLICT`. |
| **RequirementGraphV1** | Canonical hash-bound projection indexed documents, stable technology IDs, reservations, Accepted requirements/failures, gate descriptors и двусторонних owner→gate→evidence→VS/profile edges. |
| **GameplayBudgetMatrix** | Единая integer-microsecond матрица per-tick subsystem ceilings, integrated limits, cadence и measurement profile; отдельные subsystem gates не могут переопределить её суммарный budget. |
| **Vertical slice** | Минимальная играбельная цепочка, одновременно доказывающая ключевые архитектурные границы и fallback paths. |

`Entity`, `object handle`, `GUID` и `resource ID` не должны использоваться в публичном контракте без уточнения одного из нормативных ID выше.
