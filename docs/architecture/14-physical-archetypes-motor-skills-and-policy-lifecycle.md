# SPEC-14: Physical archetypes, motor skills и policy lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-14 |
| Статус | Accepted |
| Версия | 2.1 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [ADR-009](adr/009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-14 2.0; marks the unimplemented population-tier input as future R4b rather than current API |

## Назначение и invariants

Subsystem позволяет first-party и community authors добавлять новый creature archetype вместе с data-driven body, fallback controllers, pretrained motor policies, gameplay skills, AI habits и product-check fixtures без native engine code. Foundation/expert policy model является общей public boundary; hidden first-party physical API запрещён.

Runtime training отсутствует. Neural weights immutable и content-addressed. Игровое изучение skill меняет RPG proficiency и разрешённый policy route, но не веса. V1 policy switch сохраняет body topology; polymorph, mounts, possession и human↔monster replacement находятся вне scope.

[SPEC-27](27-motor-observation-action-and-deterministic-inference.md) является
единственным owner exact observation/action dtype, shape, unit, normalization,
canonical batching, recurrent-state и procedural-fallback contracts.
[SPEC-28](28-skeletal-animation-retargeting-and-ik.md) владеет
skeleton/clip/graph/retarget/root-motion/IK contracts. Этот документ связывает
их с archetype/policy lifecycle, но не создаёт альтернативный tensor, state,
animation graph или inference route.

## Source of truth и ownership

| Состояние | Единственный owner/source of truth | Разрешённый вход |
|---|---|---|
| Creature/physical/skill definitions и model bytes | Immutable cooked content registry по AssetId/revision/hash | validator/cooker publish |
| Physical pose, velocity, contacts, topology | Physical Embodiment engine-owned physics world; PhysicsBackend является private compute adapter | accepted MotorAction/physics transaction |
| Skill proficiency | RPG Framework, serialized `SkillProficiency` | validated `LearnSkill`/progression WorldCommand |
| Active policy route, transition и complete `PolicyStateRecordV1` | Motor Runtime `PolicySupervisor` | deterministic resolver + accepted transition/state commit |
| Habits, working plan и tactical preferences | Agent Runtime | `AgentArchetypeDefinition`, perception, memory, planner |
| Damage, stamina, cooldown, inventory и quest effects | RPG/Mechanics Runtime | EffectRequest/WorldCommand transaction |
| Future durable population identity, schedule и `WorldResidencyTier` | Proposed World Services R4b track in SPEC-20/ADR-046 | no current view/API; after promotion it remains a committed world-service view, never physical-policy output |
| Physical simulation LOD и pose fidelity | Physical Embodiment LOD coordinator | deterministic selection constrained by committed residency view and physical profile |
| Physical support status | Immutable bundle revision and deterministic product-check results | `Prototype` or `Supported` |

Один package может агрегировать ссылки на эти definitions, но не получает ownership чужого mutable state. Model output никогда не изменяет proficiency, gameplay effects или AgentIntent.

## Public boundary и normative data flow

Public contracts: `CreatureArchetypeManifest`, `PhysicalArchetypeBundle`, `MotorPolicyBundleManifest`, `PolicyCompatibilityKey`, `MotorSkillDefinition`, `SkillProficiency`, `MotorPerformanceEnvelope`, `MotorCapabilityView`, `PolicyActivationPlan`, `ActivePolicyRoute`, SPEC-27 `PolicyStateRecordV1`, `PolicyStateCommitV1` и support-check/run manifests.

```text
authoring sources + model artifacts + provenance
  → validate body/policies/skills/behavior references
  → deterministic cook + immutable content hashes
  → CreatureArchetypeManifest
  → future R4b World Services may publish committed WorldResidencyTier
  → spawn/materialize generic Character + independently selected physical LOD
  → planner requests ability/PhysicalAvatarIntent
  → PolicyResolver(body, equipment, proficiency, topology, intent)
  → PolicyActivationPlan → PolicySupervisor
  → accepted MotorAction → PhysicsBackend
  → ContactEvent/PhysicalOutcome → Mechanics Runtime
```

Package code не получает mutable articulation, raw ECS ID, model session pointer, direct joint buffer, filesystem/network access или gameplay-state write path.

After its own R4b promotion, `WorldResidencyTier` and physical LOD remain
separate enums, owners and state machines. Residency may then decide whether
durable population is materialized and which World Services work is due; it
never selects a pose, actuator route or model. Physical Embodiment may consume
that committed view but cannot write tier/schedule/population state. Until
that promotion there is no tier input, transition command, persistence field
or replay assertion in the current physical contract.

## Creature и physical archetype contracts

`CreatureArchetypeManifest` MUST содержать stable archetype AssetId/revision и ссылки на:

- generic Character archetype и initial RPG skill loadout;
- `PhysicalArchetypeBundle`;
- `AgentArchetypeDefinition`;
- required/optional mechanic package IDs и ability grants;
- provenance/license notices, support status и test scenarios.

`PhysicalArchetypeBundle` MUST содержать:

- `PhysicalArchetypeId`, `MorphologyFamilyId`, body schema/revision/hash;
- complete PhysicalBodyDescriptor, canonical units/frames и actuator profile;
- render skeleton mapping, collision/material assets и damage-region mapping;
- LOD projections FullArticulation → SimplifiedActiveRagdoll → CapsuleAnimation → Abstract;
- limb, grip, equipment и topology-mask capabilities;
- foundation/recovery/procedural controller references;
- motor skill catalog, policy bundles, evaluation suites и provenance;
- `PhysicalSupportLevel` (`Prototype` или `Supported`) и exact checked bundle revision.

Reference `neutral.quadruped.v1` использует generated primitive visuals, articulated torso/head и четыре двухсегментные конечности. Tail и jaw articulation отсутствуют; bite contact принадлежит head damage/contact region. Fixture не использует Gothic-derived data.

## Product support levels

| Level | Разрешённое поведение | Обязательные проверки |
|---|---|---|
| `Prototype` | CapsuleAnimation и/или procedural physical controller; learned policy optional; FullArticulation может быть отключён | schema/body/asset validation, deterministic fallback scenario, provenance/license validation |
| `Supported` | Перечисленные Full/Simplified tiers и learned skills доступны для exact bundle revision | foundation stand/locomotion/recovery, every declared skill suite, MOTOR/PHYS/POLICY product checks, performance budget и deterministic fallback |

Project content policy MAY запретить prototype creatures в конкретной поставке. Переход `Prototype` → `Supported` сохраняет `CreatureArchetype` identity, но повышает revision и заменяет exact bundle hash. First-party status не обходит failed product check. Training origin and hardware остаются provenance; support status определяется поведением exact shipped bundle на supported runtime targets. Отсутствие training hardware не блокирует engine или `Prototype`.

## Motor skills и proficiency

`MotorSkillId` — stable namespaced ID, например `org.nextengine.skill.axe.one-handed` или `org.example.creature.bite`. `SkillProficiency` сериализуется как raw basis-point `u16` в диапазоне `0…10_000`. [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) profile MUST зарегистрировать два exact `FixedPointDescriptorV1`: durable raw descriptor с `storage_bits=16`, `signed=false`, `fractional_bits=0`, bounds `0…10_000`; и canonical normalized descriptor с `storage_bits=32`, `signed=false`, `fractional_bits=16`, bounds `0…65_536`. Оба используют `NearestTiesToEven`/`RejectTransaction`.

Canonical normalization вычисляет `normalized_raw = round_ties_to_even((i128(raw_u16) * 65_536) / 10_000)` exact integer algorithm SPEC-21. Band selection и route resolution сравнивают raw basis points с raw manifest thresholds. `MotorObservation` и replay oracle содержат derived `normalized_raw`; save хранит единственный durable raw value и `RuntimeDeterminismProfileV1.numeric_profile_hash`, покрывающий оба descriptors, а loader заново вычисляет normalization до world mutation. Если model tensor требует float, versioned normalization schema MUST задавать target width и exact IEEE round-to-nearest-ties-to-even conversion из rational `normalized_raw / 65_536`; tensor float остаётся derived evaluator input, а не route/save authority. Platform-native cast/division, implicit epsilon, saturation и raw float как save source запрещены; out-of-range/intermediate overflow rejected до route mutation. Descriptor/conversion boundary vectors входят в `NUMERIC-P1` corpus и `normalization_schema_hash`.

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
- exact raw/normalized proficiency and observation `FixedPointDescriptorV1`, `AuthoritativeNumericProfileV1` hash и `NUMERIC-P1` boundary-vector hash;
- supported skill/proficiency/equipment envelope;
- joint mask и residual/action bounds, если применимо;
- exact SPEC-27 state schema width `S`, включая `S = 0`, и versioned reset/handoff rules;
- training simulator/build/config/dataset provenance и license disclosure;
- golden corpus, runtime/training evaluation manifests, runtime cost и declared fallback.

`PolicyCompatibilityKey` является canonical hash body revision, MorphologyFamilyId, topology mask, exact SPEC-27 observation/action/normalization/state schemas, actuator profile и runtime/training correspondence profile. Partial match запрещён. Inference batch order is exactly `(motor_tick, PersistentId, PolicyId)`; worker order, measured duration and cache/session identity are excluded.

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

- `Loading` проверяет exact content hash и создаёт bounded inference session вне motor/physics critical section. Completion является неauthoritative task result: она проходит [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md) deterministic staging/merge и становится наблюдаемой только на declared motor-tick commit point.
- `CompatibilityChecked` требует exact PolicyCompatibilityKey, finite metadata, supported runtime/opset и available fallback.
- `AwaitSafePoint` удерживает previous authoritative route. Switch запрещён при fall/recovery, active strike/contact, grip transition, unstable support, stale observation, topology transaction или LOD transition.
- `ShadowWarmup` выполняет candidate без actuation минимум 2 и максимум 8 motor ticks, проверяя finite/action/confidence/state bounds.
- `Commit` происходит на motor-tick boundary и атомарно заменяет ActivePolicyRoute; physics pose/velocity не копируется и не телепортируется.
- `Stabilize` применяет normal actuator rate/energy/contact guards; нарушение откатывает previous route либо recovery controller.

Worker completion order, measured load, elapsed wall time и wall timeout MUST NOT выбирать candidate/previous/fallback route либо activation tick. `PolicyActivationPlan` фиксирует logical motor-tick eligibility/deadline и canonical fault semantics; только staged compatible result на declared commit point либо canonical logical fault signal может продвинуть state machine, оставить previous route или включить declared retry/fallback. Wall watchdog MAY отменить зависшую load/inference работу; exact run получает `Fail` и не может пройти POLICY/replay check.

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

`LearnSkill` и вызванные им internal activation outcomes используют `CanonicalCommandBodyV2`, two-phase validation и exact retry/receipt semantics [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md); policy subsystem не создаёт отдельный command path.

Skill может быть committed, пока route `PendingActivation`; в этот период previous/novice route остаётся единственным actuator source. Ability, требующая новый active route, получает stable `MOTOR_ROUTE_PENDING`, а planner выбирает wait/fallback. Runtime не откатывает RPG learning из-за transient load/safe-point delay.

Reference axe fixture использует `SkillProficiency=0` и declared safe novice route. `LearnSkill` устанавливает exact raw value `6000`, после чего trained route активируется на первом canonical safe point, разрешённом logical `PolicyActivationPlan`; load wall time не выбирает этот tick. Damage/stamina modifiers выполняются отдельными Mechanics Runtime definitions и EffectRequests.

## Persistence и replay

Save segment MUST фиксировать creature/physical revisions, SkillProficiency,
ActivePolicyRoute hash, transition state, source/candidate policy hashes,
complete canonical SPEC-27 `PolicyStateRecordV1`, его exact
`authoritative_state_hash` и допустивший запись `PolicyStateCommitV1` для каждой
active policy route. Replay manifest дополнительно фиксирует exact model
catalog/content hashes; route change является derived DomainEvent и
сравнивается как oracle.

`S = 0` означает отсутствие `PolicyStateInput`/`PolicyStateOutput` tensors и
durable recurrent vector, а не отсутствие owner record. Для feed-forward и
stateful policy `PolicyStateRecordV1` остаётся authoritative и содержит exact
subject/policy/bundle/schema identity и generations, active-route и
action-schema hashes, state tick/vector, previous applied action
tick/hash/raw channel values, consecutive learned-unavailable counter и
fallback phase. Единственный normative `authoritative_state_hash` MUST
покрывать canonical bytes полного record со всеми этими
future-action-affecting fields; отдельный recurrent-only hash запрещён.

До inference request/result validation, save publication/load или replay
restore/comparison consumer MUST canonicalize полный record, recompute
`authoritative_state_hash` и exact-compare его до чтения любого covered field.
После learned, hold либо recovery validation `MotorActionV1`, полный следующий
`PolicyStateRecordV1` и `PolicyStateCommitV1` MUST публиковаться атомарно.
Commit связывает current/request
`prior_authoritative_state_hash`, recomputed
`next_authoritative_state_hash`, `applied_action_hash` и exact
route/state generations; action без соответствующего full-record commit либо
частичная публикация запрещены.

Save/load/save и replay restore MUST сохранять exact canonical record bytes,
`authoritative_state_hash`, atomic commit link и state/action roots как при
`S > 0`, так и при `S = 0`. Loader/replay rehydrator проверяет каждый full
record и непрерывность
`prior_authoritative_state_hash` → `next_authoritative_state_hash` до use или
publication. Unknown state schema, missing exact required policy,
incompatible compatibility key, corrupt model, full-record hash mismatch или
broken commit chain fails before world mutation and preserves immutable source
save, prior active generation и prior committed root. Every unavailable/unsafe
inference path uses the manifest-bound deterministic procedural fallback;
wall-time miss can fail a product check but cannot choose an authoritative action.

Declared optional fallback MAY использоваться при load только если project/save policy заранее разрешает downgrade; новый fallback hash записывается в migrated copy и делает replay несовместимым с original manifest. Silent downgrade запрещён.

## Agent archetype и habits boundary

`AgentArchetypeDefinition` содержит behavior traits, routine templates, sensory profile, tactical preferences, memory/relationship priors и initial skill loadout. Stalking, circling, territory guard, wounded retreat и preferred attack изменяют utility/HTN choice и создают AgentIntent/InvokeAbility, но не MotorAction.

Optional learned tactical policy остаётся за SPEC-06 AgentIntent boundary, имеет deterministic planner fallback и не входит в MotorPolicyBundleManifest. Motor layer не читает biography, quest state, free-form memory или LLM output.

## Authoring and training intent

There is no current public physical-policy CLI or agent-authoring protocol.
Future R5/R6 tooling may expose validate, train, export and route-test
operations, but it must call the same engine-owned validators and cannot create
a status/mutation bypass. MCP, agent context bundles, change-set schemas and
approval ceremonies are not motor architecture.

Training flow:

```text
body validation → procedural baseline → generated environment
→ foundation curriculum → novice/skill expert curriculum
→ holdout evaluation → runtime-backend correspondence
→ ONNX export/parity → transition suite → runtime product checks
```

Training algorithm является backend detail. PPO, SAC, imitation, motion priors или distillation MAY использоваться, если создают одинаковые normative artifacts и проходят product checks. Isaac Lab — `Proposed`; fallback — engine-owned headless physics lab.

На `DeveloperHostTier/macOS-aarch64` workflow ограничен schema/body validation, procedural baseline, generated deterministic 2-DoF smoke, tiny optimization, ONNX export и parity через `TRAIN-MAC-P0`; MPS failure использует declared CPU fallback. Полное foundation/creature/weapon training MAY требовать отдельный accelerator host, но runtime support определяется только exact exported artifact, correspondence и product checks на supported targets. Device/toolchain записываются в RunManifest и не выводятся из наличия model file.

Distributed weights являются licensed package content. Raw datasets, unrestricted checkpoints, protected assets и local training caches не коммитятся в engine repository.

## Failure semantics

- Missing/corrupt/incompatible model → reject before actuation; previous or declared recovery/procedural route.
- Unsupported skill/equipment/topology/proficiency → novice fallback only if explicitly evaluated; otherwise `Unavailable`.
- Manifest-declared logical load deadline либо canonical load-fault signal → remain previous route; retry/circuit-break cadence задаётся bounded integer motor ticks, no motor tick stall.
- Load/inference wall-watchdog trip → cancel uncommitted work, retain previous/safe route и mark exact run `Fail`; elapsed wall time не создаёт deterministic fallback или activation decision.
- ShadowWarmup/action/state violation → discard candidate session/state, previous route unchanged.
- Unsafe point persists → route remains pending with diagnostic; gameplay MAY use declared novice/wait fallback.
- Transition commit fault → atomic rollback to previous route; no partial joint ownership.
- `PolicyStateRecordV1` schema/identity mismatch on load or replay → stable `MOTOR_STATE_IDENTITY_MISMATCH`; full-record hash or commit-chain mismatch → stable `MOTOR_STATE_HASH_MISMATCH`. Both fail closed before covered-field use and preserve original save/prior committed root, including `S = 0`.
- Missing behavior adapter/ai-host → deterministic AgentArchetype planner fallback; motor remains correct.
- Required support-check input or provenance missing → package remains `Prototype`.
- Runtime-training request → capability denial; no mutable model bytes or optimizer state in game process.
- MPS unavailable/unsupported operation → CPU smoke fallback с diagnostic; отсутствие MPS не блокирует repository/prototype work и не считается MPS `Pass`.
- Required training hardware unavailable → training job does not start; engine and procedural `Prototype` continue to work.

## Product checks

| ID | Сценарий | Ожидаемый результат | Fallback |
|---|---|---|---|
| EMB-01 | `Prototype`/`Supported` bundle validation corpus | 100% required references, hashes and provenance resolve; unsupported status claims rejected; 0 package-specific native/public types | retain `Prototype` or reject bundle revision |
| POLICY-01 | policy compatibility, ONNX parity and runtime/training suite | 100% compatibility mismatches rejected; MOTOR-P1 threshold; `NUMERIC-P1` conversion vectors pass; 100 held-out episodes outcome delta ≤2 percentage points | previous model, procedural recovery or training-backend fallback |
| POLICY-02 | 1 000 valid/blocked/faulted transitions | 100% unsafe points deferred; 0 teleport/actuator/safety violation; valid candidate commits within 30 motor ticks after first eligible safe point; route/state exact under completion permutations | pin previous route and circuit-break candidate |
| SKILL-01 | one-handed axe, 1 000 held-out episodes per proficiency band | route result exact for all boundary vectors; novice valid-contact success 20–60%; trained ≥80%; trained median contact time ≥20% lower; 0 safety violations; damage changes only through `EffectRequest` | reject skill revision and retain novice route |
| CREATURE-01 | neutral quadruped `Prototype` → `Supported` | stand/locomotion/recovery aggregate ≥90%; bite/lunge valid contact ≥75%; 0 hidden/native gameplay dependency; procedural fallback remains loadable | retain capsule/procedural `Prototype` |
| BEHAVIOR-01 | habits/tactics with ai-host/adapter present and absent | 0 direct MotorAction/gameplay mutation from habits; all actions pass AgentIntent/WorldCommand; scenario outcomes exact offline | deterministic authored utility/HTN profile |
| TRAIN-P1 | pinned training backend export/deployment | ADR-009 thresholds; license/SBOM complete; export reproducible from pinned config except declared stochastic metrics | engine-owned headless physics lab |
| TRAIN-MAC-P0 | generated deterministic 2-DoF development smoke | 4 fixed seeds; 8 envs ×256 steps; 1 000 observations; PyTorch/ONNX max abs error ≤`1e-5`; 0 NaN/Inf; ≤10 minutes on MPS or declared CPU fallback | CPU smoke; stop this training lane on export/parity failure |

These checks define runtime and authoring behavior; they are not organizational approval.
