# Глоссарий Next Engine

| Поле | Значение |
|---|---|
| ID | GLOSSARY-001 |
| Статус | Accepted |
| Версия | 1.4 |
| Владелец | Architecture Working Group |
| Последняя проверка | 2026-07-22 |
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
| **WorldCommand** | Единственный валидируемый запрос на изменение authoritative gameplay state. Содержит schema version, issuer, target PersistentId, preconditions и deterministic payload. |
| **DomainEvent** | Неизменяемый факт об уже принятом изменении domain state. Не является альтернативным mutable API. |
| **PresentationSnapshot** | Read-only снимок состояния для renderer/audio/UI/interpolation. Не может быть записан обратно как authoritative state. |
| **AgentIntent** | Высокоуровневое намерение AI, ещё не имеющее права изменять мир. Проходит policy/rules validation и преобразуется в WorldCommand либо rejection. |
| **PhysicalAvatarIntent** | Ограниченный по времени запрос locomotion/posture/manipulation к motor controller; не задаёт physics pose напрямую. |
| **MotorObservation** | Versioned numeric observation, вычисленная из physics state и разрешённого gameplay context. |
| **MotorAction** | Ограниченный versioned набор joint targets/torques или controller parameters, прошедший safety clamp. |
| **PhysicsBackend** | Engine-owned interface для worlds, bodies, articulations, queries, contacts и state snapshots. |
| **ContactEvent** | Нормализованное backend-independent begin/persist/end событие с continuity `contact_id`, PersistentId участников, body slots, material tags, point/normal, relative velocity, impulse/effective mass и physics tick. |
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
| **NeutralAuthoringModel** | Engine-owned промежуточная asset/world schema между source importers и cooker. |
| **NeutralImportModel** | Строго ограниченный переносимый результат внешнего importer, являющийся подмножеством NeutralAuthoringModel с provenance. |
| **WorldChunk** | Версионированная единица streaming с bounds, dependencies, PersistentId namespace и content hashes. |
| **SaveManifest** | Корень сохранения: schema versions, build compatibility, world revision, loaded chunks, content hashes и ordered state segments. |
| **ReplayManifest** | Корень replay: build/schema/model hashes, seed, initial snapshot и ordered external WorldCommand stream. |
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
| **EvidenceBaselineManifest** | Immutable human-approved scenario/profile/toolchain/content semantic/media reference; generated candidate не является approved baseline. |
| **EvidenceBundleManifest** | Root content-addressed closure, объединяющий automatic results, impact, diagnostics, replay, metrics, raw media roots, review artifacts и baselines. |
| **HumanReviewDecision** | Attested immutable `Approved`, `Rejected` или `NeedsChanges`, привязанный к exact changeset/evidence/baseline hashes и authorized human role. |
| **HumanReviewRequired** | Impact state, при котором successful automatic gates недостаточны и exact changeset требует valid HumanReviewDecision. |
| **Capability** | Явно выданное право script/plugin/process на именованную операцию или data view. Default — deny. |
| **MechanicPackageId** | Стабильный namespaced ID gameplay/mod package; версия и publisher identity являются отдельными полями. |
| **MechanicPackageManifest** | Author-declared metadata, dependencies, capabilities, exports, state schemas/migrations, provenance, licenses и tests package. |
| **MechanicsLock** | Generated exact package closure: versions, hashes, dependency/patch order, granted capabilities и schemas, используемые save/replay/runtime. |
| **AbilityDefinition** | Immutable cooked описание активной/пассивной способности: requirements, targeting, costs, cooldown, phases, effects и cues. |
| **AbilityInstance** | Runtime state machine конкретного выполнения AbilityDefinition с causal command, phase, targets и interruption state. |
| **MechanicAffordance** | Machine-readable описание способности для planner/authoring tools: preconditions, target, cost/time/risk, expected outcome range и failures. |
| **EffectRequest** | Валидируемое предложение применить semantic effect от source к target в заданном context. |
| **EffectTransaction** | Канонический atomic результат effect pipeline над RPG/Mechanics state; либо committed полностью, либо не applied. |
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
| **Vertical slice** | Минимальная играбельная цепочка, одновременно доказывающая ключевые архитектурные границы и fallback paths. |

`Entity`, `object handle`, `GUID` и `resource ID` не должны использоваться в публичном контракте без уточнения одного из нормативных ID выше.
