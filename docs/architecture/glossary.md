# Глоссарий Next Engine

| Поле | Значение |
|---|---|
| ID | GLOSSARY-001 |
| Статус | Accepted |
| Версия | 4.1 |
| Последняя проверка | 2026-08-17 |
| Нормативные зависимости | INDEX-001, [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-047](adr/047-simple-application-session-and-save-on-close.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-056](adr/056-deterministic-strategic-agent-and-belief-driven-goap.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](adr/059-event-sourced-physx-continuation-reconstruction.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md) |
| Дополнительные зависимости V4.0 | [SPEC-36](36-functional-tissue-condition-and-injury.md), [SPEC-37](37-character-embodiment-and-surface-deformation.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Заменяет | GLOSSARY-001 4.0; adds the bounded R5e capsule procedural decision terms without changing public save/replay schemas |

Термины ниже имеют одинаковый смысл во всех RFC, schemas, CLI и diagnostics. Публичные контракты MUST использовать эти имена или явно версионированные производные.

| Термин | Нормативное определение |
|---|---|
| **Next Engine** | Временное нейтральное имя независимого специализированного AI-first RPG engine. |
| **Core runtime** | Детерминированное ядро scheduling, ECS facade, команд, событий, snapshots и persistence hooks. |
| **RPG framework** | Generic доменная модель персонажей, предметов, квестов, диалогов, фракций и интерактивных объектов. |
| **Backend** | Заменяемая реализация engine-owned interface. Vendor-типы не входят в interface. |
| **DeveloperHostTier** | Поддержанный host для portable source/tool development и локальных ProductCheck, не создающий shipping-platform claim; v1 включает `macOS-aarch64` по ADR-011. |
| **TrainingCapabilityStatus** | `Available`, `Unavailable` либо `Rejected` для exact hardware/backend profile; сообщает только доступность bounded training run и не выводится из наличия model file. |
| **RuntimeEntityId** | Эфемерный идентификатор живой ECS entity. Действует только в пределах одного runtime instance и не сериализуется. |
| **PersistentId** | Стабильный 128-bit opaque ID сущности или логического объекта между save/load, chunks и replay. Не кодирует ECS layout или vendor handle. |
| **AssetId** | Стабильная ссылка на логический asset; конкретная cooked revision определяется manifest и content hash. |
| **ContentHash** | SHA-256 канонических cooked bytes и параметров cooker, используемый для immutable bundle addressing. |
| **ProjectAuthoringV7** | Current editable `nextengine.project-authoring.v7` intent consumed only by the cooker; it carries typed body-schema, routine, 100-record population/navigation, cognition and systemic activity catalogs and is not runtime authority. |
| **ProjectLockV3** | Current immutable exact project closure over authoring, schema/content/world/mechanics and runtime/launch/platform profile hashes plus allowed presentation targets; it contains no resolver, recovery or storage policy. |
| **ActivatedProjectV8** | Complete validated current project closure atomically published from one exact `ProjectLockV3`, all referenced manifests/records and exact body-schema/routine/population/navigation/cognition/activity catalog bindings. |
| **WorldPopulationSnapshotV1** | World Services segment containing 100 stable logical population records with revision, home/current region/node and tier; it owns no physical pose. |
| **NavigationRoutePlanV1** | Revision-bound engine-owned ordered graph path with integer cost and plan hash; advisory for physical traversal and authoritative only for validated abstract transfer. |
| **AgentCognitionCatalogV1** | Required current content root binding one existing population subject to integer evaluation cadence, bounded retrieval/GOAP limits, stable goal/action/fact IDs and sorted seed beliefs. |
| **AgentCognitionSnapshotV1** | Separate Agent Runtime owner state for active/suspended goals, plan, private task, pending intent, hysteresis, decision RNG and last epistemic hash. |
| **AgentMemorySnapshotV1** | Separate Memory Service owner state for the current bounded semantic beliefs, provenance/contradiction, recorded structured speech acts, revision and last retrieval tick. |
| **WorldActivitySnapshotV1** | Separate World Services authority for one owner-validated `Unassigned → Assigned → Working → Completed` activity; it stores no RPG resource, currency, item or physical pose. |
| **ReplayManifestV10** | Current-only replay envelope requiring the nine non-routine Runtime/RPG/Physics/streaming/population/activity/Agent/Memory/physical-animation owners and permitting only routine to be absent; reference alpha compares ten full-tuple descriptors and the application root. |
| **SchemaDescriptorV1** | Immutable engine-owned schema declaration со stable schema/field IDs, wire shape, encoding, role, compatibility policy and canonical hash; storage/backend layout не является schema. |
| **SchemaRegistryManifestV2** | Exact current set of bounded schema descriptors/current refs bound by `ProjectLockV3`; it contains no historical window or migration DAG. |
| **ContentManifestV1** | Immutable catalog root of exact neutral asset revisions, provenance, current dependency closure and domain root; runtime resolves no floating content revision. |
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
| **PresentationSnapshotV2** | Atomically published immutable scene/camera/semantic-UI plus bounded cue/environment-hash projection with snapshot epoch, sequence and stable object keys. Renderer/UI consume it read-only; it never writes back or enters gameplay hashes. |
| **PlatformCapabilitySetV1** | Canonical normalized engine-owned host capability/limit set selected before runtime staging; it contains no native device, extension or backend type. |
| **PlatformTimebaseV1** | Checked monotonic native-sample normalization profile for platform diagnostics/order; it cannot compute simulation/world tick. |
| **PlatformEventV1** | Bounded typed normalized platform fact with stable host/source sequence and identity; callback order is not authority. |
| **NormalizedControlEventV1** | Bounded engine-owned semantic control value from a private device adapter; native object, wall time and target tick are excluded. |
| **ApplicationSessionManifestV2** | Immutable composition-root session closure over exact project/launch/platform/runtime/schema/content and presentation-target hashes, without recovery/storage/shutdown policy. |
| **ApplicationSessionStateV2** | Runtime-owned revisioned state in the closed `Created → CompositionStaged → RuntimeStaged → Active ↔ Suspended → Quiescing → Finalizing → Closed` lifecycle. |
| **ApplicationCoordinator** | Production application service в `next_application`, связывающий exact project activation, Runtime-owned session plans, Assets-owned atomic durable publication, reference-game execution, presentation extraction, close, Save/Load and replay; сам не становится source of truth. |
| **SessionStore** | Assets-owned two-slot store with atomic `CURRENT` and one canonical `session.snapshot.v4.bin` per selected slot; snapshot keeps current state, last lifecycle record and one close journal/receipt. |
| **ReferenceGame** | First-party product owner в `next_reference_game`: authored reference source, stable IDs, Runtime bootstrap, scripted vertical slice and presentation bindings, без test assertions или temporary fault fixtures. |
| **CloseSessionJournalV2** | Durable two-stage `Prepared \| SavePublished` close journal; Prepared binds one immutable save image and SavePublished prevents a second generation on retry. |
| **CloseSessionReceiptV2** | Terminal receipt binding one close request/session to the single published final-save generation. |
| **ActionMapManifest** | Immutable map stable device-independent action IDs to bounded semantic controls, contexts, conflict policy and accessibility metadata. |
| **PlayerActionFrame** | Canonical immutable ordered device-independent action input for one ingress sample; it contains no physical-device identity, wall time, target tick, camera transform or backend object. |
| **TargetingIntent** | Non-authoritative assigned player proposal containing quantized aim and gameplay query/profile identifiers; authoritative targeting is reconstructed from the assigned-tick snapshot, never rendered camera/depth data. |
| **UiSemanticSnapshot** | Immutable stable screen/control-role/action-affordance projection for UI; widget trees, localized strings as identity and mutable domain references are excluded. |
| **PlayerPreferenceProfile** | Local versioned `PresentationOnly` bindings/accessibility/localization/presentation settings whose resulting canonical action frame may enter production input but whose stored values are not gameplay authority. |
| **AgentIntent** | Высокоуровневое намерение AI, ещё не имеющее права изменять мир. Проходит policy/rules validation и преобразуется в WorldCommand либо rejection. |
| **Strategic Agent** | Детерминированный Agent Runtime слой, выбирающий `что` и `почему` делать NPC из bounded epistemic state; execution выполняют domain owners через proposals и `WorldCommand`. |
| **Epistemic View** | Immutable revision-bound совокупность perception, beliefs, memory и разрешённых self/owner projections, доступная конкретному NPC; hidden authoritative world state в неё не входит. |
| **Belief** | Semantic утверждение NPC с subject/predicate/value, confidence, provenance, learned/verified revision и contradiction state; не является world truth. |
| **Drive View** | Derived fixed-point pressure для goal selection, вычисленное из authoritative owner resources и agent-owned hysteresis; не отдельная mutable needs authority. |
| **Aspiration** | Long-horizon authored tendency, создающая medium-horizon goal candidates, но не исполняющая action напрямую. |
| **Semantic Affordance** | Revision-bound planning projection owner-specific возможности с preconditions/effects/cost and typed execution; current bounded set is logical-route request, hold-position, social-exchange commit, activity wait and systemic settlement. It is not an executable callback or mutation right. |
| **Speech Act** | Bounded semantic NPC/player communication proposal с participants, topic/claim, provenance, confidence и expiry; generated wording не является gameplay authority, and only an owner commit creates a domain result. |
| **Commitment** | RPG-owned revisioned agreement binding issuer, recipient, work, workplace, currency and wage through the closed Offered/Accepted/Fulfilled/Cancelled lifecycle; speech or Agent state cannot mutate it directly. |
| **Tier Cognition Work** | Canonically ordered due-work classification mapping Active/Simulated/Abstract/Dormant records to FullEvaluation/ReducedEvaluation/AbstractMaintenanceOnly/DormantWakeCheckOnly with zero fabricated outcomes. |
| **Bounded Bulk Time** | Reference-game request for 1–4096 ordinary simulation ticks that stops at the first observable boundary or budget exhaustion and must match stepped execution roots/counts at the returned boundary. |
| **Decision Trace** | Bounded immutable non-authoritative diagnostic goal scores, cited beliefs, selected plan, task outcome и replan reason для одного Strategic Agent boundary. |
| **PhysicalAvatarIntent** | Ограниченный по времени запрос locomotion/posture/manipulation к motor controller; не задаёт physics pose напрямую. |
| **BodySchema** | Immutable versioned heterogeneous Physical Interaction Graph со stable schema-scoped body-node/joint-edge/actuator/effector/attachment identities and semantic roles; из одной exact revision выводятся physics descriptors, motor layouts, cached static morphology input, safety limits и replay compatibility. Не содержит current pose или mutable overlays. |
| **BodySchemaAsset** | Current exact content wrapper binding one `BodySchemaV1` generation to its `AssetId`, record revision/hash and admitted deterministic compiler-profile identity; it is immutable project content, not pose or instance state. |
| **BodyInstanceProjection** | Immutable revision-bound effective projection exact BodySchema plus morphology/equipment/stats/damage/fatigue/attachment owners; Physical Embodiment компилирует mass/inertia/ROM/actuator/sensor facts, но не получает ownership исходных mutable fields. |
| **BodyTissueSchema** | Future immutable content-addressed functional anatomy bound to one exact BodySchema: stable tissue/group/break-site identities, sparse capability dependencies, allowed structural variants and presentation bindings. Exact V1 wire remains Proposed. |
| **BodyConditionState** | Future RPG-owned durable typed tissue integrity/continuity/structural/recovery state. Mechanics, Physics, Motor and presentation may consume immutable views or submit proposals but cannot own or mutate it directly. Exact aggregate remains Proposed. |
| **SystemicCondition** | Future RPG-owned bounded whole-body ladder for the first functional-anatomy profile: stable, impaired, critical, unconscious and dead semantics. It deliberately replaces separate player-facing pain/shock/blood pools in the first vertical; exact wire remains Proposed. |
| **FunctionalMuscleGroup** | Bounded gameplay/control abstraction that contributes directionally to one or more actuator axes; it constrains capability in the joint-actuated profile and is not automatically a physical muscle actuator. |
| **BodyCapabilityEnvelope** | Reconstructible immutable Physical Embodiment projection intersecting BodySchema safety with committed condition/stats/equipment/fatigue/structural facts; consumed by Motor observation and fixed safety/PD, never a second durable owner. Exact wire remains Proposed. |
| **BodyTreatmentStage** | Future semantic stabilization, repair or rehabilitation transition over RPG-owned body condition. Medicine and magic use the same validated proposal/command path; exact treatment contracts remain Proposed. |
| **BodyStatusProjection** | Future qualitative read-only UI view of region, function, attachment/structure, systemic band and next treatment stage. It cannot diagnose from presentation or reveal hidden NPC truth. Exact wire remains Proposed. |
| **BodyProjectionRoots** | Reconstructible canonical witness binding exact schema/instance/compiler inputs to emitted physics descriptors, observation/action layouts and actuator-safety limits. It is rederived at activation/restore and is not a mutable save owner. |
| **CapsuleProceduralMotorControllerV1** | Current immutable/stateless R5e `CapsuleAnimation` controller bound to exact body projection, action-layout, actuator-safety and capsule-locomotion roots. It admits a closed cardinal command or emits zero horizontal recovery while Physics reports vertical instability; it never owns pose or saved phase. |
| **CapsuleMotorDecisionV1** | Reconstructible per-tick R5e evidence binding source Physics tick/body revision, requested/applied cardinal direction, clamp bits and exact controller/projection/safety roots. It is not a canonical save owner or a replacement for the general SPEC-27 `MotorActionV1`. |
| **CanonicalEnvironmentReplay** | Byte-exact engine-owned reset/step/action/observation/snapshot/root continuation для locked CPU PhysX build profile. Worker/slot completion order и vendor caches не входят в result. |
| **MotorReplayPrefix** | Bounded ordered episode-origin reset plus every canonical post-safety 240 Hz effort and per-motor-tick physics witness used to reconstruct hidden PhysX continuation in a fresh scene; policy/PD re-execution is separate parity evidence. |
| **EvaluatorCorrespondence** | Bounded comparison canonical CPU execution с accelerated GPU/trainer mirror. Это проверка близости trajectories/contact/done, а не разрешение GPU быть replay authority. |
| **StatisticalTrainingOutcome** | Multi-seed quality/stability/throughput result. Он не доказывает exact replay, evaluator parity или безопасный runtime artifact. |
| **MotorTrainingEnvironment** | Bounded long-lived reset/step environment, использующий production descriptors, actuation, PhysX stepping и immutable canonical records; reward/trainer не получает прямой mutation path. |
| **MotorSkillCommand** | Bounded planner-facing skill/phase/style/cancel/fallback command между PhysicalAvatarIntent и low-level policy; не содержит raw joint actuation и не доказывает outcome. Exact unconsumed V1 shape remains Proposed. |
| **ContactPlan** | Revision-bound ordered desired effectors, surfaces, target frames, activation windows and force/sliding envelopes; это reference/proposal, а committed Physics остаётся единственным доказательством контакта. Exact unconsumed V1 shape remains Proposed. |
| **Physical skill contract** | Conceptual composition of `PhysicalAvatarIntent + MotorSkillCommand + ContactPlan + PhysicalActionChunk`; not a fifth record. It carries stable IDs, reference frames, numeric targets, constraints and evidence predicates, never natural language in the physical hot path. |
| **PhysicalActionChunk** | Bounded normally 250–1000 ms root/CoM/effector/object/contact/force/support reference from authored motion, motion matching, procedural or optional learned generator; low-level control stays closed-loop and the chunk never owns physical pose or stores accepted joint actions. Exact unconsumed V1 shape remains Proposed. |
| **MotorAdaptationProfile** | Explicit bounded action-response history schema, cadence, state/latent segment and reset/remap rules for no-gradient dynamics adaptation; known engine parameters remain direct observation inputs. Exact unconsumed V1 shape remains Proposed. |
| **MotorObservation** | Versioned numeric observation, вычисленная из physics state и разрешённого gameplay context. |
| **MotorAction** | Ограниченный versioned набор joint targets/torques или controller parameters, прошедший safety clamp. |
| **MotorObservationSchemaV1** | Exact engine-owned observation tensor dtype/rank/shape/feature order/unit/normalization contract bound to a policy route. |
| **MotorActionSchemaV1** | Exact engine-owned action tensor/channel/unit/bounds contract; every learned or procedural candidate passes the same fixed-point safety clamp. |
| **MotorInferenceBatchV1** | Closed inference batch canonically ordered by `(motor_tick, PersistentId, PolicyId)`; worker/completion/wall-time order cannot select action. |
| **PolicyStateRecordV1** | Canonical bounded recurrent motor state with exact subject/policy/bundle/schema/route identity and explicit reset/migration/save rules. |
| **PhysicsBackend** | Engine-owned interface для worlds, bodies, articulations, queries, contacts и state snapshots. |
| **ContactEvent** | Нормализованное backend-independent begin/persist/end событие с continuity `contact_id`, PersistentId участников, body slots, material tags, point/normal, relative velocity, impulse/effective mass и physics tick. |
| **PhysicsCanonicalSnapshotV2** | Current portable engine-owned canonical grounded-capsule checkpoint; native backend snapshot bytes are not authoritative. |
| **PhysicsCoordinateProfileV1** | Exact right-handed metres/kilograms/seconds/radians and quantization profile used at the public physics boundary. |
| **RootMotionIntentV1** | Revision-bound animation-produced locomotion proposal accepted only through the normal motor/command validation path; never a pose teleport. |
| **PhysicalIkConstraintV1** | Bounded authoritative physical IK constraint resolved inside declared physics/motor order. |
| **PresentationIkRequestV1** | Presentation-only IK request that may alter render pose but cannot change contacts, gameplay or physics snapshot. |
| **CrossTargetProjectionRoot** | Exact root canonical quantized physical projection для cross-target replay comparison. Raw backend samples могут иметь отдельно declared tolerance, но IDs, event classes, gameplay outcomes и projection root остаются exact. |
| **RenderPose** | Read-only presentation pose, построенная из authoritative physics pose либо animation pose согласно physics LOD. |
| **EstimatedMuscleRecruitment** | Presentation-only allocation of applied joint effort to functional groups under a declared visual objective. It is not actual muscle activation and cannot drive damage, fatigue, healing, strength, Motor state or replay authority. |
| **CharacterEmbodimentManifest** | Future immutable content closure for render rig mapping, skinned surfaces, pose/load correctives, tissue/wound variants, secondary motion, LOD and complete presentation fallback. Exact V1 wire remains Proposed. |
| **InjuryPresentationProfile** | Project/local presentation selection `Reduced`, `Realistic` or `Graphic` over the same committed condition and topology; it changes no gameplay, physical or replay authority. |
| **Physics LOD** | Разрешённый уровень embodied simulation: full articulation, simplified active ragdoll, capsule/animation или abstract simulation. |
| **CreatureArchetypeManifest** | Public immutable manifest, связывающий generic Character, physical archetype, AgentArchetypeDefinition, mechanic packages, provenance и validation metadata без package-specific runtime type. |
| **PhysicalArchetypeBundle** | Cooked BodySchema/compiled descriptor/LOD/capability/policy package одной physical morphology revision с exact fallbacks и validation references. |
| **MorphologyFamilyId** | Stable namespaced ID семейства совместимых body/motor representations; не означает совместимость без exact PolicyCompatibilityKey. |
| **MotorSkillId** | Stable namespaced ID физически исполняемого навыка, независимый от конкретной model revision. |
| **SkillProficiency** | RPG-owned unsigned fixed-point `u16` 0…10 000, сериализуемый уровень владения skill; не neural weight. |
| **MotorPerformanceEnvelope** | Измеримые distribution-level bounds physical skill по точности, контактам, времени, энергии, балансу, recovery и safety. |
| **MotorCapabilityView** | Read-only planner/mechanics view состояния skill route: `Unavailable`, `NoviceFallback`, `PendingActivation` или `Active`. |
| **MotorPolicyBundleManifest** | Immutable evaluator-format-neutral manifest model bytes, schemas, family/topology envelope, explicit state, runtime cost, provenance, evaluation и fallback; provider/device/library types excluded. |
| **PolicyCompatibilityKey** | Canonical hash BodySchema/compiled revision, morphology family, topology bucket/mask, observation/action/normalization/adaptation/state schemas, actuator profile и runtime/training correspondence profile. |
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
| **SaveManifest** | Корень сохранения: schema versions, build compatibility, world revision, loaded chunks, content hashes и ordered state segments. |
| **ReplayManifest** | Корень replay: initial checkpoint, world/runtime/content hashes, named RNG state, `CommandLedgerV2` snapshot/root и ordered `ClosedIngressBatchV1`/`ClosedCommandAdmissionBatchV2` boundaries со всеми bounded authenticated/decodable command envelopes, включая duplicates, conflicts и deterministic rejections. |
| **RunManifest** | Локальный машиночитаемый результат одного run: scenario, build/config hashes, metrics, diagnostics и optional debug artifacts; не является глобальным product status. |
| **TestScenarioManifest** | Immutable bounded scenario: exact fixtures/hashes, seeds/clocks, production actions, declared faults, probes/assertions, limits и optional capture. |
| **ScenarioAction** | Production input/validated command/lifecycle/declared fault action, назначенная на simulation tick; не mutable test hook. |
| **ProbeSpec** | Read-only selector documented state/event/diagnostic/normalized metric, не влияющий на schedule или state hash. |
| **AssertionSpec** | Versioned `Exact`, `Tolerance`, `Distribution`, `Invariant` или `PresentationMetric` oracle с explicit threshold и stable failure code. |
| **OffscreenPresentationTarget** | Engine render target в GPU images/readback без PlatformHost window/surface/swapchain/display server. |
| **CapturePlan** | Optional scenario-owned camera/audio/view/overlay/timeline/output specification для reproducible local debug media. |
| **CaptureJobManifest** | Portable immutable displayless worker job с exact content/replay/profile/CapturePlan hashes и bounded local output requirements. |
| **Capability** | Явно выданное право script/plugin/process на именованную операцию или data view. Default — deny. |
| **MechanicPackageId** | Стабильный namespaced ID gameplay/mod package; версия и publisher identity являются отдельными полями. |
| **MechanicPackageManifestV1** | Current canonical package identity/version/content hash plus declared capabilities and exact ability/effect definitions. |
| **MechanicsLock** | Generated exact package closure: versions, hashes, dependency/patch order, granted capabilities и schemas, используемые save/replay/runtime. |
| **AbilityDefinition** | Immutable cooked описание активной/пассивной способности: requirements, targeting, costs, cooldown, phases, effects и cues. |
| **MechanicAffordance** | Machine-readable описание способности для planner/authoring tools: preconditions, target, cost/time/risk, expected outcome range и failures. |
| **EffectRequest** | Валидируемое предложение применить semantic effect от source к target в заданном context. |
| **PresentationCue** | Semantic non-authoritative запрос gameplay package на VFX/audio/UI/camera feedback. |
| **ai-host** | Отдельный процесс для LLM, embeddings, ASR и TTS. Не участвует в deterministic simulation tick. |
| **Headless runtime** | Тот же core/RPG runtime без renderer и интерактивного platform shell, предназначенный для validation, replay и tests. |
| **Cooker** | Детерминированный tool, преобразующий validated NeutralAuthoringModel в immutable content-addressed bundles. |
| **ProductCheck** | Небольшая воспроизводимая инженерная проверка наблюдаемого product behavior. Canonical kinds: `fast`, `play`, `persistence-replay`, `content-package`, conditional `platform` и conditional `performance`; выбор определяется затронутой областью, а результат описывает только exact run. |
| **GameplayBudgetMatrix** | Единая integer-microsecond матрица per-tick subsystem ceilings, integrated limits, cadence и measurement profile; отдельный subsystem benchmark не может переопределить её суммарный budget. |
| **Vertical slice** | Минимальная играбельная цепочка, используемая `play` ProductCheck для проверки ключевого RPG loop и fallback paths. |

`Entity`, `object handle`, `GUID` и `resource ID` не должны использоваться в публичном контракте без уточнения одного из нормативных ID выше.
