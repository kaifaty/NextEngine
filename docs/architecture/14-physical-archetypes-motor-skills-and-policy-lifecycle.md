# SPEC-14: Physical archetypes, motor skills и policy lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-14 |
| Статус | Accepted |
| Версия | 1.3 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-015](adr/015-evidence-trust-fixture-separation-and-attestation.md) |
| Заменяет | отсутствует |

## Назначение и invariants

Subsystem позволяет first-party и community authors добавлять новый creature archetype вместе с data-driven body, fallback controllers, pretrained motor policies, gameplay skills, AI habits и conformance fixtures без native engine code. Foundation/expert policy model является общей public boundary; hidden first-party physical API запрещён.

Runtime training отсутствует. Neural weights immutable и content-addressed. Игровое изучение skill меняет RPG proficiency и разрешённый policy route, но не веса. V1 policy switch сохраняет body topology; polymorph, mounts, possession и human↔monster replacement находятся вне scope.

## Source of truth и ownership

| Состояние | Единственный owner/source of truth | Разрешённый вход |
|---|---|---|
| Creature/physical/skill definitions и model bytes | Immutable cooked content registry по AssetId/revision/hash | validator/cooker publish |
| Physical pose, velocity, contacts, topology | PhysicsBackend согласно SPEC-05 | accepted MotorAction/physics transaction |
| Skill proficiency | RPG Framework, serialized `SkillProficiency` | validated `LearnSkill`/progression WorldCommand |
| Active policy route, transition и bounded recurrent state | Motor Runtime `PolicySupervisor` | deterministic resolver + accepted transition |
| Habits, working plan и tactical preferences | Agent Runtime | `AgentArchetypeDefinition`, perception, memory, planner |
| Damage, stamina, cooldown, inventory и quest effects | RPG/Mechanics Runtime | EffectRequest/WorldCommand transaction |
| Certification decision | Physical Embodiment Release gate + signed evidence packet | immutable gate results |

Один package может агрегировать ссылки на эти definitions, но не получает ownership чужого mutable state. Model output никогда не изменяет proficiency, gameplay effects или AgentIntent.

## Public boundary и normative data flow

Public contracts: `CreatureArchetypeManifest`, `PhysicalArchetypeBundle`, `MotorPolicyBundleManifest`, `PolicyCompatibilityKey`, `MotorSkillDefinition`, `SkillProficiency`, `MotorPerformanceEnvelope`, `MotorCapabilityView`, `PolicyActivationPlan`, `ActivePolicyRoute`, `PolicyState` и certification/run manifests.

```text
authoring sources + model artifacts + provenance
  → validate body/policies/skills/behavior references
  → deterministic cook + immutable content hashes
  → CreatureArchetypeManifest
  → spawn generic Character + selected physical LOD
  → planner requests ability/PhysicalAvatarIntent
  → PolicyResolver(body, equipment, proficiency, topology, intent)
  → PolicyActivationPlan → PolicySupervisor
  → accepted MotorAction → PhysicsBackend
  → ContactEvent/PhysicalOutcome → Mechanics Runtime
```

Package code не получает mutable articulation, raw ECS ID, model session pointer, direct joint buffer, filesystem/network access или gameplay-state write path.

## Creature и physical archetype contracts

`CreatureArchetypeManifest` MUST содержать stable archetype AssetId/revision и ссылки на:

- generic Character archetype и initial RPG skill loadout;
- `PhysicalArchetypeBundle`;
- `AgentArchetypeDefinition`;
- required/optional mechanic package IDs и ability grants;
- provenance/licenses, certification status и test scenarios.

`PhysicalArchetypeBundle` MUST содержать:

- `PhysicalArchetypeId`, `MorphologyFamilyId`, body schema/revision/hash;
- complete PhysicalBodyDescriptor, canonical units/frames и actuator profile;
- render skeleton mapping, collision/material assets и damage-region mapping;
- LOD projections FullArticulation → SimplifiedActiveRagdoll → CapsuleAnimation → Abstract;
- limb, grip, equipment и topology-mask capabilities;
- foundation/recovery/procedural controller references;
- motor skill catalog, policy bundles, evaluation suites и provenance;
- `PhysicalCertificationLevel` и signed evidence root when certified.

Reference `neutral.quadruped.v1` использует generated primitive visuals, articulated torso/head и четыре двухсегментные конечности. Tail и jaw articulation отсутствуют; bite contact принадлежит head damage/contact region. Fixture не использует Gothic-derived data.

## Certification levels

| Level | Разрешённое поведение | Обязательные доказательства |
|---|---|---|
| `PrototypeFallback` | CapsuleAnimation и/или procedural physical controller; learned policy optional; package явно не обещает FullArticulation conformance | schema/body/asset validation, fallback scenario, provenance/license checks |
| `PhysicalCertified` | Full/Simplified physical tiers и перечисленные learned skills | foundation stand/locomotion/recovery, every declared skill suite, MOTOR/PHYS/POLICY gates, performance budget, failure media и signed evidence packet |

Project policy MAY запретить prototype creatures в shipping content. Prototype promotion сохраняет CreatureArchetype identity, но повышает revision и заменяет exact bundle hash. Signature или first-party status не обходят certification gates. `PrototypeFallback` MAY разрабатываться без RTX; `PhysicalCertified` MUST дополнительно иметь `TRAIN-RTX-01=PASS`. Mac `TRAIN-MAC-P0` является только toolchain smoke и не может подменить этот capability/correspondence evidence.

## Motor skills и proficiency

`MotorSkillId` — stable namespaced ID, например `org.nextengine.skill.axe.one-handed` или `org.example.creature.bite`. `SkillProficiency` сериализуется как unsigned fixed-point `u16` в диапазоне `0…10_000`; floating normalization для model input является derived value, не save source.

`MotorSkillDefinition` MUST содержать:

- supported morphology families, body/topology masks и required limb/grip capabilities;
- required equipment/tool tags, mechanic ability/affordance links и supported PhysicalAvatarIntent kinds;
- ordered proficiency bands с integer inclusive bounds;
- candidate policy routes и compatibility requirements для каждого band;
- declared novice/procedural fallback либо explicit `Unavailable`;
- `MotorPerformanceEnvelope` per band;
- switch restrictions, evaluation suite, provenance и failure behavior.

`MotorPerformanceEnvelope` задаёт distribution-level thresholds: target error, valid-contact success, time-to-contact/complete, energy/torque, balance loss/fall, recovery и safety bounds. Он не задаёт damage formula.

`MotorCapabilityView` имеет состояния:

- `Unavailable` — compatible evaluated route отсутствует; planner не вызывает physical ability;
- `NoviceFallback` — разрешён специально trained novice/generic или procedural route;
- `PendingActivation` — skill известен, candidate загружается/ждёт safe point; previous/novice route остаётся authoritative;
- `Active` — route committed и прошёл stabilization.

Запуск expert вне declared body/equipment/topology/proficiency/training envelope запрещён. «Плохо владеет оружием» MUST быть намеренно измеренным novice behavior, не случайным OOD failure.

## Foundation, experts и composition

Foundation policy каждой morphology family MUST обеспечивать declared subset balance, locomotion, posture, reach/grip и recovery либо делегировать отсутствующее действие deterministic procedural controller. Skill route может иметь один из видов:

| Route kind | Contract |
|---|---|
| `ConditionedMode` | тот же foundation model; skill/proficiency/mode входят в declared observation schema |
| `ResidualSkillAdapter` | bounded residual поверх foundation action для explicit joint mask; overlap с другим residual запрещён в v1 |
| `ExclusiveExpert` | один full-body expert заменяет foundation action producer после safe handoff; foundation/recovery остаётся fallback |

Одновременно может быть активен максимум один `ExclusiveExpert`. Arbitrary averaging, learned runtime routing, implicit priority и composition незнакомых outputs запрещены.

`MotorPolicyBundleManifest` MUST содержать:

- PolicyId/version/model SHA-256, ONNX opset и required runtime features;
- model kind/route kind, morphology/body/topology/actuator compatibility;
- observation/action/normalization schema hashes и control/inference rates;
- supported skill/proficiency/equipment envelope;
- joint mask и residual/action bounds, если применимо;
- feed-forward либо versioned bounded recurrent `PolicyState` schema/reset/handoff;
- training simulator/build/config/dataset provenance и license disclosure;
- golden corpus, runtime/training evaluation manifests, runtime cost и declared fallback.

`PolicyCompatibilityKey` является canonical hash body revision, MorphologyFamilyId, topology mask, observation/action/normalization schemas, actuator profile и runtime/training correspondence profile. Partial match запрещён.

## Deterministic route resolution

`PolicyResolver` является pure deterministic engine function:

```text
(PhysicalArchetype revision, equipment tags, SkillProficiency,
 topology/limb mask, requested intent, exact policy catalog)
    → Result<PolicyActivationPlan, StableRejection>
```

Selection order задаётся exact skill definition: compatible active route → compatible higher-preference candidate → declared novice/procedural fallback → `Unavailable`. Tie-breaker — explicit candidate order затем PolicyId; runtime discovery/load order не влияет на выбор.

Resolver не оценивает neural output и не меняет RPG state. Mechanics/AI читают `MotorCapabilityView`, но actual ability authorization повторно проверяется на command tick.

## PolicySupervisor и switching

State machine:

```text
Requested → Loading → CompatibilityChecked → AwaitSafePoint
→ ShadowWarmup → Commit → Stabilize → Active
```

- `Loading` проверяет exact content hash и создаёт bounded inference session вне motor/physics critical section.
- `CompatibilityChecked` требует exact PolicyCompatibilityKey, finite metadata, supported runtime/opset и available fallback.
- `AwaitSafePoint` удерживает previous authoritative route. Switch запрещён при fall/recovery, active strike/contact, grip transition, unstable support, stale observation, topology transaction или LOD transition.
- `ShadowWarmup` выполняет candidate без actuation минимум 2 и максимум 8 motor ticks, проверяя finite/action/confidence/state bounds.
- `Commit` происходит на motor-tick boundary и атомарно заменяет ActivePolicyRoute; physics pose/velocity не копируется и не телепортируется.
- `Stabilize` применяет normal actuator rate/energy/contact guards; нарушение откатывает previous route либо recovery controller.

Policy switch не является LOD или topology transition. Изменение body schema/MorphologyFamilyId/topology mask во время route transition запрещено. После трёх transition failures за session candidate circuit-break-ится для entity и emits diagnostic.

## Skill progression data flow

```text
LearnSkill WorldCommand
  → RPG validates and commits SkillProficiency
  → SkillCapabilityChanged DomainEvent
  → PolicyResolver creates PolicyActivationPlan
  → PolicySupervisor loads/waits/warms/commits
  → ActivePolicyRouteChanged DomainEvent
  → planner/mechanics observe updated MotorCapabilityView
```

Skill может быть committed, пока route `PendingActivation`; в этот период previous/novice route остаётся единственным actuator source. Ability, требующая новый active route, получает stable `MOTOR_ROUTE_PENDING`, а planner выбирает wait/fallback. Runtime не откатывает RPG learning из-за transient load/safe-point delay.

Reference axe fixture использует `SkillProficiency=0` и declared safe novice route. `LearnSkill` устанавливает `6000`, после чего trained route активируется на ближайшем safe point. Damage/stamina modifiers выполняются отдельными Mechanics Runtime definitions и EffectRequests.

## Persistence и replay

Save segment MUST фиксировать creature/physical revisions, SkillProficiency, ActivePolicyRoute hash, transition state, source/candidate policy hashes и bounded PolicyState when declared. Replay manifest дополнительно фиксирует exact model catalog/content hashes; route change является derived DomainEvent и сравнивается как oracle.

Feed-forward policy reset не требует durable hidden state. Stateful policy MUST иметь versioned fixed-size PolicyState schema, canonical serialization и explicit reset/handoff. Unknown state schema, missing exact required policy, incompatible compatibility key или corrupt model fails before world mutation and preserves original save.

Declared optional fallback MAY использоваться при load только если project/save policy заранее разрешает downgrade; новый fallback hash записывается в migrated copy и делает replay несовместимым с original manifest. Silent downgrade запрещён.

## Agent archetype и habits boundary

`AgentArchetypeDefinition` содержит behavior traits, routine templates, sensory profile, tactical preferences, memory/relationship priors и initial skill loadout. Stalking, circling, territory guard, wounded retreat и preferred attack изменяют utility/HTN choice и создают AgentIntent/InvokeAbility, но не MotorAction.

Optional learned tactical policy остаётся за SPEC-06 AgentIntent boundary, имеет deterministic planner fallback и не входит в MotorPolicyBundleManifest. Motor layer не читает biography, quest state, free-form memory или LLM output.

## Authoring, training и agent workflow

Normative CLI/JSON contracts:

| Команда | Результат |
|---|---|
| `next physical new|describe|validate|pack <archetype>` | scaffold, resolved body/LOD/capability graph, validation и deterministic bundle |
| `next policy inspect|verify <policy>` | manifest/schema/provenance/runtime support и golden parity report |
| `next policy route-test <archetype> --matrix <fixture>` | exhaustive resolver/capability matrix |
| `next policy transition-test <archetype> --suite <id>` | safe/blocked/fault transition runs + media manifest |
| `next policy certify <archetype> --evidence <run>` | immutable certification candidate; не self-approving |
| `lab generate-env|train|evaluate|compare|export-onnx|certify` | backend-neutral research application service и structured RunManifest |

Training flow:

```text
body validation → procedural baseline → generated environment
→ foundation curriculum → novice/skill expert curriculum
→ holdout evaluation → runtime-backend correspondence
→ ONNX export/parity → transition suite → certification review
```

Training algorithm является backend detail. PPO, SAC, imitation, motion priors или distillation MAY использоваться, если создают одинаковые normative artifacts и проходят gates. Isaac Lab — `Proposed`; fallback — engine-owned headless physics lab.

На `DeveloperHostTier/macOS-aarch64` workflow ограничен schema/body validation, procedural baseline, generated deterministic 2-DoF smoke, tiny optimization, ONNX export и parity через `TRAIN-MAC-P0`; MPS failure использует declared CPU fallback. Foundation/creature/weapon training, runtime-backend correspondence и certification выполняются только после `TRAIN-RTX-01` на поддерживаемом local Linux/NVIDIA host. Состояние capability входит в RunManifest и не выводится из наличия model file.

AuthoringContextBundle MUST включать body/skill/policy/TestScenario/CapturePlan schemas, compatibility/ownership/output-category matrix, resolved references, training/evaluation configs, diagnostic fix-its, budgets и minimal neutral examples. Agent может предлагать descriptors/config/reward/scenario changes и запускать bounded experiments. ImpactResolver автоматически классифицирует physical/motor/animation как HumanReviewRequired. Promotion model weights, certification, baseline, capability/license/provenance changes всегда требует explicit owner + SPEC-15 human evidence review; agent не может self-certify/self-approve.

Distributed weights являются licensed package content. Raw datasets, unrestricted checkpoints, protected assets и local training caches не коммитятся в engine repository.

## Failure semantics

- Missing/corrupt/incompatible model → reject before actuation; previous or declared recovery/procedural route.
- Unsupported skill/equipment/topology/proficiency → novice fallback only if explicitly evaluated; otherwise `Unavailable`.
- Load timeout → remain previous route; bounded retry/circuit-break, no motor tick stall.
- ShadowWarmup/action/state violation → discard candidate session/state, previous route unchanged.
- Unsafe point persists → route remains pending with diagnostic; gameplay MAY use declared novice/wait fallback.
- Transition commit fault → atomic rollback to previous route; no partial joint ownership.
- Stateful schema mismatch on load → fail-closed and preserve original save.
- Missing behavior adapter/ai-host → deterministic AgentArchetype planner fallback; motor remains correct.
- Certification artifact/provenance/media missing → package remains PrototypeFallback; status cannot be inferred from signature.
- Runtime-training request → capability denial; no mutable model bytes or optimizer state in game process.
- Missing GPU/encoder/human reviewer → CPU policy/route/safety evidence сохраняется, package promotion остаётся AwaitingCapability; interactive showcase не заменяет review.
- Human approval при failed POLICY/SKILL/CREATURE gate → rejected as AUTO_GATE_NOT_PASS.
- MPS unavailable/unsupported operation → CPU smoke fallback с diagnostic; отсутствие MPS не блокирует repository/prototype work и не считается MPS PASS.
- `TRAIN-RTX-01` unavailable/failed → certification candidate остаётся `AwaitingCapability`; package остаётся `PrototypeFallback`, даже если ONNX и Mac parity прошли.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| EMB-01 | prototype/certified bundle validation corpus | 100% required references/hashes/provenance resolve; 100% invalid certification claims rejected; 0 package-specific native/public types | bundle graph, schema/license/API report | retain PrototypeFallback/reject bundle revision |
| POLICY-01 | policy compatibility, ONNX parity и runtime/training suite | 100% compatibility mismatches rejected; MOTOR-P1 exact threshold; 100 held-out episodes outcome delta ≤2 percentage points | manifest/golden corpus/parity/correspondence RunManifests | previous model/procedural recovery; training backend fallback |
| POLICY-02 | 1 000 valid/blocked/faulted transitions | 100% unsafe points deferred; 0 pose teleport, actuator-limit or safety violation; valid commit ≤500 ms after next eligible safe point; every failure retains previous/recovery route | route timeline, action/physics trace, state hashes, required GIF/MP4 manifest | pin previous route/circuit-break candidate |
| SKILL-01 | one-handed axe, 1 000 held-out episodes per proficiency band | novice valid-contact success 20–60%; trained ≥80%; trained median valid-contact time ≥20% lower; 0 safety violations; gameplay damage changes only through EffectRequest | skill/route manifests, metrics distributions, replay/contact traces, media | reject skill revision/retain novice route |
| CREATURE-01 | neutral quadruped prototype→certified | stand/locomotion/recovery aggregate ≥90%; bite/lunge valid contact ≥75%; 0 hidden/native gameplay dependency; prototype fallback remains loadable | public package graph, scenario metrics, contacts/replays, media | retain prototype/capsule/procedural controller |
| BEHAVIOR-01 | habits/tactics with ai-host/adapter present and absent | 0 direct MotorAction/gameplay mutation from habits; all actions pass AgentIntent/WorldCommand; required scenario outcomes exact offline | planner/intent/command trace, boundary scan, replay | deterministic authored utility/HTN profile |
| TRAIN-P1 | pinned training backend export/deployment | ADR-009 thresholds; complete license/SBOM; artifacts reproducible from pinned config except declared stochastic metrics | environment/export manifests, parity and held-out reports | engine-owned headless physics lab |
| TRAIN-MAC-P0 | generated deterministic 2-DoF development smoke | 4 fixed seeds; 8 envs ×256 steps; 1 000 parity observations; max abs PyTorch/ONNX error ≤`1e-5`; 0 NaN/Inf; ≤10 minutes on MPS or declared CPU fallback | device/toolchain/lock/model/corpus hashes, RunManifest, parity metrics | CPU smoke; block training lane on export/parity failure |
| TRAIN-RTX-01 | local full-training capability preflight | supported Linux/NVIDIA host, RAM ≥32 GiB, VRAM ≥16 GiB, pinned driver/toolchain, offline cache/provenance/license PASS | hardware/driver/toolchain/license/SBOM capability report | remain `AwaitingCapability`; only PrototypeFallback |
| AUTHOR-P1 | cold human/agent physical authoring workflow | reference harness creates quadruped prototype, adds axe skill route, resolves impact, validates/evaluates/captures certification candidate using only public bundles/CLI; 100% internal references resolve; no self-approval; valid HumanReviewDecision required after automatic PASS | context/impact/evidence bundles, AgentChangeSet, command/run/review reports | reviewed public CLI workflow/fix context generator; retain prototype |

Physical Embodiment Team владеет EMB/POLICY/SKILL/CREATURE/TRAIN gates; Agent Intelligence co-owns BEHAVIOR-01, Developer Experience co-owns AUTHOR-P1, Security & Governance co-owns provenance/license/certification evidence. Release Engineering принимает root artifacts.
