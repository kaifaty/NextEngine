# SPEC-14: Physical archetypes, motor skills и policy lifecycle

| Поле | Значение |
|---|---|
| ID | SPEC-14 |
| Статус | Accepted |
| Версия | 2.7 |
| Последняя проверка | 2026-08-12 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](28-skeletal-animation-retargeting-and-ik.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Заменяет | SPEC-14 2.6; adopts the no-text contact-centric physical-skill contract and replaces the unconsumed motion-horizon proposal with `PhysicalActionChunk` |

## Назначение и invariants

Subsystem позволяет first-party и community authors добавлять новый creature
archetype вместе с data-driven body, fallback controllers, learned motor
policies, gameplay skills, AI habits и product-check fixtures без native engine
code. Общая public boundary — versioned `BodySchema`, semantic motor commands,
policy-family metadata и deterministic actuator/safety path, а не одна
universal network. Hidden first-party physical API запрещён.

Runtime training отсутствует. Neural weights immutable и content-addressed.
Игровое изучение skill меняет RPG proficiency и разрешённый policy route, но
не веса. V1 policy switch сохраняет body topology; topology transaction,
polymorph, mounts, possession и human↔monster replacement не являются policy
switch и имеют отдельный physical/content lifecycle.

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

Public contracts: `CreatureArchetypeManifest`, `PhysicalArchetypeBundle`,
semantic `BodySchema`, `BodyInstanceProjection`, `MotorPolicyBundleManifest`,
`PolicyCompatibilityKey`, `MotorSkillDefinition`, `SkillProficiency`,
`MotorPerformanceEnvelope`, `MotorCapabilityView`, `PolicyActivationPlan`,
`ActivePolicyRoute`, SPEC-27 `PolicyStateRecordV1`, `PolicyStateCommitV1` и
support-check/run manifests. SPEC-35 accepts `BodySchemaV1` and
`BodyInstanceProjectionV1` for the fixed Stage 0 humanoid. Exact
`MotorSkillCommandV1`, `ContactPlanV1`, `PhysicalActionChunkV1`,
`MotorAdaptationProfileV1` and advanced overlay/family shapes below remain
Proposed targets, not current registry entries.

```text
authoring sources + model artifacts + provenance
  → validate body/policies/skills/behavior references
  → deterministic cook + immutable content hashes
  → CreatureArchetypeManifest
  → future R4b World Services may publish committed WorldResidencyTier
  → spawn/materialize generic Character + independently selected physical LOD
  → planner requests ability/PhysicalAvatarIntent
  → typed physical primitives + constraints; natural language is already absent
  → Skill Orchestrator → ContactPlan → optional PhysicalActionChunk
  → PolicyResolver(body, equipment, proficiency, topology, intent)
  → PolicyActivationPlan → PolicySupervisor
  → accepted MotorAction → PhysicsBackend
  → ContactEvent/PhysicalOutcome → Mechanics Runtime
```

Package code не получает mutable articulation, raw ECS ID, model session pointer, direct joint buffer, filesystem/network access или gameplay-state write path.

Natural language MAY be compiled into a typed `AgentIntent` at an optional
human/dialogue/`ai-host` boundary. Raw text, tokens and language embeddings are
forbidden after that boundary: the physical path uses only stable IDs, closed
primitive/constraint enums, declared reference frames, numeric targets, masks
and revision-bound evidence predicates. Display names and text provenance do
not participate in motor routing, observation, chunk identity or success proof.

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

- `PhysicalArchetypeId`, `MorphologyFamilyId`, `BodySchema` revision/hash;
- deterministic compiled SPEC-26 descriptor closure, canonical units/frames и
  actuator profile derived from that exact `BodySchema`;
- render skeleton mapping, collision/material assets и damage-region mapping;
- LOD projections FullArticulation → SimplifiedActiveRagdoll → CapsuleAnimation → Abstract;
- limb, grip, equipment и topology-mask capabilities;
- foundation/recovery/procedural controller references;
- motor skill catalog, policy bundles, evaluation suites и provenance;
- `PhysicalSupportLevel` (`Prototype` или `Supported`) и exact checked bundle revision.

Reference `neutral.quadruped.v1` использует generated primitive visuals, articulated torso/head и четыре двухсегментные конечности. Tail и jaw articulation отсутствуют; bite contact принадлежит head damage/contact region. Fixture не использует Gothic-derived data.

## `BodySchema` and instance projection

`BodySchema` is the immutable heterogeneous Physical Interaction Graph from
which physics articulation, motor tensor layout, cached morphology input,
actuator/safety limits and save/replay compatibility are derived together. It
is not only a skeleton hierarchy, is not a neural model and does not contain
mutable physics state.

The Proposed exact target shape is:

```text
BodySchemaV1 {
  schema_id, schema_revision, coordinate_profile_hash,
  body_nodes[], joint_edges[], actuators[], effectors[],
  colliders[], attachment_slots[], symmetry_groups[], capabilities[],
  schema_hash
}
```

Every node, joint, actuator, effector and attachment has a stable
schema-scoped namespaced ID independent of array position or backend handle.
The primary articulation graph is acyclic. Runtime grabs, carried objects,
equipment and cooperative loads create a bounded validated attachment/
constraint overlay; they do not edit immutable source bytes.

Body nodes carry shape/mass/inertia/rest frame and closed semantic roles such
as root, torso, limb, support/manipulation/strike effector, wing or tail. Joint
edges carry endpoints, type/axes/limits, passive dynamics and actuator health/
availability semantics. Effectors, actuators, colliders, attachments and
capability relations are distinct graph record classes with stable IDs.

Static schema facts include mass/inertia/CoM, dimensions, rest frame, collider
summary, semantic role, joint type/axes/limits, passive stiffness/damping,
torque-speed-power envelope, nominal PD profile, break threshold, material and
support/contact capability. Current pose, velocity, contact, external impulse,
joint state, available power, saturation, fatigue, damage, sensor validity and
attached load are dynamic projections.

```text
BodyInstanceProjectionV1 =
    exact BodySchemaV1
  + MorphologyParameters
  + EquipmentOverlay
  + StatsOverlay
  + DamageOverlay
  + FatigueState
  + RuntimeAttachmentGraph
```

Every overlay carries owner revision/hash. RPG/Mechanics remain owner of
stats, equipment, damage and fatigue; Physics remains owner of pose, contacts
and active topology. Physical Embodiment compiles only the immutable effective
mass/inertia/ROM/actuator/sensor projection and its hash. Changing an overlay
never creates a parallel mutable authority.

SPEC-26 owns exact physics descriptor/topology compilation. SPEC-27 owns exact
observation/action/state layout. A topology transaction maps surviving
elements by stable BodySchema IDs, increments the physical/topology revision
and resets incompatible adaptation/generator/router state; it never guesses by
array index or joint name.

Static morphology encoding is reconstructible and MAY be cached per node/edge.
Its identity binds exact BodySchema, compiled descriptor, effective instance,
topology and encoder/profile hashes. It invalidates only on creation or a
declared schema/topology/effective-projection change, including material body/
equipment change or actuator lock/restore; per-tick pose/contact/fatigue updates
do not rebuild it. Cache warmth and completion order never choose an action.

These semantic boundaries are Accepted through ADR-066. ADR-058/SPEC-35 accept
the `BodySchemaV1` and `BodyInstanceProjectionV1` wire records for the fixed
23-DoF Stage 0 humanoid. Non-default equipment, damage, attachment/topology
overlays and additional policy families remain Proposed until their own
production consumers and ProductChecks exist under ADR-046.

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

### Proposed hierarchical skill interfaces

Future consumer-backed `MotorSkillDefinitionV1` extends the existing semantic
definition with an exact command schema, phase graph, contact template, cancel
windows, fallback skills, motion source, proficiency/style curve, energy model
and evaluation profile. It never embeds executable planner callbacks.

The concept called a **physical skill contract** is not a fifth overlapping
record. It is the composition of the existing boundaries: a future
consumer-backed `PhysicalAvatarIntent` specialization names the desired
physical result and immutable constraint/completion/failure profiles;
`MotorSkillCommandV1` selects the skill, phase, style and interruption policy;
`ContactPlanV1` and `PhysicalActionChunkV1` refine short-horizon execution. No
layer in this chain owns the target RPG fact or proves its own success.

The intent vocabulary is deliberately small and compositional: stable closed
primitive IDs such as move/reach/contact/maintain/release/apply-force/support/
move-object/balance/recover combine with entity/feature IDs, numeric target
poses in declared reference frames, masks and bounds. It is not an enum for
every object, animation, injury and body combination. A model sees only a
derived one-hot/embedding-table encoding of stable IDs, never their strings.
Effector selectors use declared semantic capabilities and canonical metrics/
tie-breaks rather than assuming human left/right limbs.

```text
MotorSkillCommandV1 {
  command_epoch, skill_id, phase_id, style_and_proficiency,
  locomotion_posture_or_object_goal, urgency,
  cancel_mode, minimum_commit_ticks, safe_cancel_phases[], fallback_skill_id,
  source_intent_revision, command_hash
}

ContactPlanV1 {
  source_snapshot_and_query_roots,
  ordered_targets[effector, surface, local point/normal,
                  activation window, desired force range, allow_sliding],
  plan_hash
}

PhysicalActionChunkV1 {
  source intent/command/contact/scene/body revision hashes,
  exact start/end tick and project-bounded 250..1000 ms profile,
  root/center-of-mass and effector/object trajectories,
  selected contact schedule, desired force ranges and support transitions,
  phase/style/proficiency and interruption metadata, chunk_hash
}
```

A future consumer MAY expose one bounded read-only progress projection:

```text
SkillProgressProjection target semantics {
  source intent/command/skill/phase identities and revisions,
  bounded fixed-point progress,
  satisfied and violated predicate IDs,
  cited physics/contact/query/PhysicalOutcome roots,
  Running | SucceededCandidate | FailedCandidate | Blocked | Cancelled,
  stable reason
}
```

The projection is derived only from engine-owned structured facts explicitly
selected for the subject: transforms and velocities, support/contact and grasp
state, balance, target-relative geometry, committed outcomes and permitted
gameplay projections. It consumes no camera image, rendered frame, depth
buffer, visual embedding or presentation state. `SucceededCandidate` and
`FailedCandidate` are evidence-bound observations, not owner mutation: the
private Task Executive and affected domain owners revalidate their revisions,
predicates and capabilities before any task transition or `WorldCommand`.
Missing, stale or contradictory evidence yields `Blocked`/a stable failure;
it is never guessed from presentation.

Simple standing/velocity locomotion MAY bind command features directly to the
low-level policy. Parkour, climbing, two-hand weapons, throwing/catching and
other contact-rich skills SHOULD use an authored, motion-matching or learned
`PhysicalActionChunk`. All three records are proposals/references: only
committed physics proves a contact, grasp, hit, traversal or recovery outcome.

`PhysicalActionChunkV1` is one bounded physical-semantic reference, not an
open-loop actuator sequence. It contains no already accepted `MotorAction`,
joint command list, framework tensor or natural language. A bundle-private
shared latent is permitted only as a declared fixed-width numeric feature
segment and cannot replace the structured root/CoM/effector/contact/force
fields. While one chunk is active, the low-level controller still rebuilds
observation and produces a complete action every motor tick; fixed PD/safety
revalidates every physics substep. The next chunk may be staged in parallel but
becomes visible only at its declared deterministic boundary.

Normal cancel generates a declared transition chunk. Emergency cancel uses
the bounded `stabilize → brace → safe fall → ragdoll → get-up` fallback chain.
The exact target interfaces remain Proposed until a production skill consumes
them.

## Foundation, experts и composition

Одна route обслуживает только bounded morphology/dynamics family and training
envelope. Target family set is humanoid/biped, general legged, serpentine,
aquatic, aerial, bounded modular and musculoskeletal research. A creature that
walks and flies uses separate ground/flight policies plus explicit takeoff/
landing transition experts. One humanoid+snake+fish+bird policy, arbitrary
actuator vocabulary and zero-shot arbitrary topology are unsupported.

The advanced target factorizes one compatible family into a shared physical
backbone plus morphology adapter, skill adapter and explicit locomotion-regime
expert/router. Shared parameters may learn support, contact, balance, force and
momentum; morphology/regime-specific parts decode those meanings for the exact
body. Ground/flight/swim and materially different families remain explicit
routes with validated transitions, not implicit mixtures.

Foundation policy близкой family SHOULD обеспечивать declared subset balance,
locomotion, posture, reach/grip и recovery либо делегировать отсутствующее
действие deterministic procedural controller. Skill route может иметь один из
видов:

| Route kind | Contract |
|---|---|
| `ConditionedMode` | тот же foundation model; skill/proficiency/mode входят в declared observation schema |
| `ResidualSkillAdapter` | bounded residual поверх foundation action для explicit joint mask; overlap с другим residual запрещён в v1 |
| `ExclusiveExpert` | один full-body expert заменяет foundation action producer после safe handoff; foundation/recovery остаётся fallback |

Одновременно может быть активен максимум один engine-level `ExclusiveExpert`.
Arbitrary averaging and composition of outputs from unrelated bundles,
implicit joint ownership and hidden priority are forbidden. One bundle MAY
implement internal MoE/expert routing, hysteresis or soft blend as a single
bounded action producer when all router state that can affect future action is
explicit in SPEC-27 state. Internal routing cannot grant a skill, bypass
`PolicyResolver` or become gameplay authority.

After a family-wide teacher proves quality, a morphology hypernetwork MAY
compile one immutable smaller MLP/GRU student or adapter for an exact
BodySchema/projection. The child is a normal content-addressed policy bundle
with its own compatibility key, golden corpus, runtime cost, fallback and
retention/parity checks. Topology/effective-projection change invalidates it;
runtime never mutates generated weights in place.

`MotorPolicyBundleManifest` MUST содержать:

- PolicyId/version/model SHA-256 and closed required runtime capabilities
  without provider/device/library types; its outer semantics are evaluator-
  neutral, while the exact consumer-backed replacement for the legacy
  ONNX-opset field remains a Proposed `evaluator_format_profile_id` evolution;
- model kind/route kind, morphology/body/topology/actuator compatibility;
- `BodySchema` hash, morphology family/topology bucket and exact
  observation/action/normalization/adaptation schema hashes plus
  control/inference rates;
- exact raw/normalized proficiency and observation `FixedPointDescriptorV1`, `AuthoritativeNumericProfileV1` hash и `NUMERIC-P1` boundary-vector hash;
- supported skill/proficiency/equipment envelope;
- joint mask и residual/action bounds, если применимо;
- exact SPEC-27 state schema width `S`, including explicit adaptation,
  generator and expert-router segments where present, and versioned
  reset/handoff rules;
- training simulator/build/config/dataset provenance и license disclosure;
- golden corpus, runtime/training evaluation manifests, runtime cost и declared fallback.

`PolicyCompatibilityKey` является canonical hash `BodySchema`/compiled body
revision, MorphologyFamilyId, topology bucket/mask, exact SPEC-27 observation/
action/normalization/adaptation/state schemas, actuator profile и runtime/
training correspondence profile. Partial match запрещён. Inference batch order
is exactly `(motor_tick, PersistentId, PolicyId)`; worker order, measured
duration and cache/session identity are excluded.

## Proposed first humanoid and advanced learned profiles

ADR-066 preserves the rejection of the former Mamba-2 foundation target. The first Proposed
learned profile is deliberately small and does not change the procedural R5/v1
requirement:

- one fixed humanoid skeleton with approximately `20..30` controlled DoF;
- feed-forward MLP with an initial target of approximately `1..3M` parameters;
- exact `60 Hz` policy and `240 Hz` physics/actuator profile;
- bounded residual joint-position targets relative to neutral/authored
  reference, optional velocity target, fixed engine-owned PD gains;
- standing, walk/run/backward/strafe/turn/crouch/crawl, slopes/stairs,
  push recovery, ragdoll and get-up;
- authored or motion-matching reference when useful and an independently
  shipping animation/procedural fallback.

Direct torque, adaptive gains and muscle activations require separate profile,
safety and ProductChecks. A learned reference generator is a later upstream
module, not a prerequisite for this route.

Known equipment/stats/damage/fatigue/actuator values enter the observation
explicitly and constrain the same engine-owned actuator envelope. RPG
strength/dexterity/endurance never live in or rewrite skill weights. Proposed
`MotorAdaptationProfileV1` binds the exact recent
observation→action→response history schema, update cadence, latent/state
segments and reset/remap rules. TCN is the first fixed-window comparator and
GRU the first recurrent comparator. Runtime gradients and hidden history are
forbidden.

Mamba MAY be benchmarked under equal parameter count, context length and
latency for long-history adaptation, terrain/perception token streams, motion
generation or temporal planning. It is not the default low-level controller
and portable export alone cannot promote it.

For a later within-family variable-topology profile, the Proposed low-level
shape is cached static node/edge morphology plus dynamic graph state and the
current chunk → one or two local graph-message layers → bounded global
coordination attention → generic temporal core → one shared per-joint/
actuator head. GRU is the first recurrent baseline. A fixed-width
`Linear(hidden → N joints)` is permitted only for the fixed-humanoid comparator
and proves no variable-topology support. The first shared head emits position
and optional velocity targets through fixed engine gains; adaptive gains,
residual torque and muscle actions require independent promotion.

The first Proposed evaluator-format value is
`portable-onnx-standard-ops`; ONNX Runtime is a private reference adapter.
Trainer↔export↔Windows/Linux parity, perturbation/recovery/transition safety,
resource bounds and multi-seed comparison against MLP/TCN/GRU/procedural
baselines are required before any learned profile promotion.

## Proposed candidate rollout and control-quality tiers

A high-fidelity skill route MAY produce exactly `K` candidate
`PhysicalActionChunk` records and score them over one fixed short horizon from
the same immutable canonical physics checkpoint. `K`, horizon, score schema,
candidate IDs/order and logical work budget are project-locked. Each branch has
an isolated RNG stream derived from world/subject/chunk epoch/candidate ordinal
when needed and publishes no command, event, save, external effect or mutable
world state.

All branches complete as bounded synchronous logical work; wall time and worker
completion order cannot select a result. Fixed-point score/evidence vectors use
canonical task progress, balance, contact, collision/damage, energy and safety
facts and have one total order ending in candidate ID. Only the winner is
staged at the ordinary chunk boundary. Replay records candidate-set, scoring,
ordered result, winner and selected-chunk roots; it does not expose a general
branching-replay API. Profile failure retains the already declared base chunk
or procedural route.

The control-quality ladder is selected only from canonical simulation facts
and manifest tokens: abstract outcome → animation/navigation/IK/procedural →
learned closed-loop controller → learned controller plus bounded rollouts.
Measured load, renderer visibility and wall time are not tier inputs. Rollouts
are intended only for a small manifest-bounded set of important nearby actors
and are not an R5/v1 requirement.

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

## Progression, style and physical overlays

Per-character durable state contains body/RPG stats, skill proficiency,
style/school/personality parameters, current damage/fatigue and exact
SPEC-27 policy state. It does not contain a private full neural policy.

Novice/master differences are authored and evaluated through repertoire,
contact timing, safety margin, interruption/recovery choice, energy efficiency
and anticipation. They do not receive magic torque, lower physics frequency or
untrained expert output. Strength/power/flexibility/endurance and equipment
change effective actuator, ROM, energy, mass/inertia and CoM projection through
their owning domains.

Fatigue target has global energy plus bounded local joint/limb fatigue. Damage
may reduce torque/power/velocity/ROM, lock/free an actuator, change latency,
noise/sensory confidence and support capability. Known values are immediate
explicit inputs and hard safety bounds; history adaptation handles only
unknown/effective residual dynamics.

Offline improvement MAY train a small hero/class/school adapter from immutable
trajectories. It always creates a new content-addressed child bundle, passes
old-skill retention, transition, safety and target parity checks and activates
only through a new `ProjectLockV3` and session. Active bundle/save/world bytes
are never edited or hot-swapped.

Target implementation order after the first humanoid is: humanoid variations,
equipment, injuries, weapon/manipulation skills, parkour/contact planning,
general-legged family, then bounded cross-family research. These phases remain
Proposed and add no current schema or completion claim until a production
consumer exists.

## Persistence и replay

Save segment MUST фиксировать creature/physical revisions, SkillProficiency,
ActivePolicyRoute hash, transition state, source/candidate policy hashes,
complete canonical SPEC-27 `PolicyStateRecordV1`, его exact
`authoritative_state_hash` и допустивший запись `PolicyStateCommitV1` для каждой
active policy route. Exact adaptation, motion-generator and internal expert-
router segments are part of that same record, never evaluator-private cache.
Replay manifest дополнительно фиксирует exact model catalog/content hashes,
canonical applied actions/state chain and required periodic physics snapshot
roots; route change является derived DomainEvent и сравнивается как oracle.
Authoritative replay consumes and validates the recorded canonical action/state
chain rather than relying only on evaluator re-execution. Separate evaluator
parity MUST reproduce the same canonical action/state for a `Supported` route.

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

`AgentArchetypeDefinition` содержит behavior traits, routine templates, sensory profile, tactical preferences, memory/relationship priors и initial skill loadout. Stalking, circling, territory guard, wounded retreat и preferred attack изменяют deterministic Utility + bounded GOAP/tactical choice и создают AgentIntent/InvokeAbility, но не MotorAction.

Optional learned tactical policy остаётся за SPEC-06 AgentIntent boundary, имеет deterministic planner fallback и не входит в MotorPolicyBundleManifest. Motor layer не читает biography, quest state, free-form memory или LLM output.

## Authoring and training intent

There is no current public physical-policy CLI or agent-authoring protocol.
Future R5/R6 tooling may expose validate, train, export and route-test
operations, but it must call the same engine-owned validators and cannot create
a status/mutation bypass. MCP, agent context bundles, change-set schemas and
approval ceremonies are not motor architecture.

Training flow:

```text
body/schema + authored standing baseline
→ licensed motion ingestion/retarget
→ reference tracking → command locomotion → terrain
→ robustness/recovery → motion prior/multi-skill
→ dynamics adaptation → equipment/fatigue → damage
→ manipulation/weapons → parkour
→ cross-morphology family work → in-engine transfer
→ portable export/parity → transition/retention/runtime checks
```

Training algorithm является backend detail. PPO, SAC, imitation, motion priors или distillation MAY использоваться, если создают одинаковые normative artifacts и проходят product checks. Isaac Lab — `Proposed`; fallback — engine-owned headless physics lab.

The first Proposed reference toolchain is Isaac Lab/PhysX + ProtoMotions +
RSL-RL + PyTorch, followed by fixed-shape standard-op ONNX export and a private
ONNX Runtime adapter. It is replaceable and never outranks canonical
production `headless`. Every accelerated mirror passes descriptor/axis/
actuator/contact correspondence and every candidate receives final in-engine
evaluation.

Motion data requires explicit intended-use license/provenance. Noncommercial,
no-derivatives, unknown or incompatible training/distribution terms exclude
the affected corpus from a commercial/distributable candidate. Dataset bytes,
checkpoints, optimizer state and generated model output remain outside Git.

[SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md)
is the Proposed common reset/step/trajectory/dataset/run/export data plane.
Production `headless` remains canonical; Isaac Lab or another GPU simulator is
only an accelerated mirror with a correspondence suite and final headless
evaluation. Its trainer/framework types do not enter the manifest above.

На `DeveloperHostTier/macOS-aarch64` workflow ограничен schema/body validation, procedural baseline, generated deterministic 2-DoF smoke, tiny optimization, ONNX export и parity через `TRAIN-MAC-P0`; MPS failure использует declared CPU fallback. Полное foundation/creature/weapon training MAY требовать отдельный accelerator host, но runtime support определяется только exact exported artifact, correspondence и product checks на supported targets. Device/toolchain записываются в RunManifest и не выводятся из наличия model file.

Distributed weights являются licensed package content. Raw datasets, unrestricted checkpoints, protected assets и local training caches не коммитятся в engine repository.

## Failure semantics

- Missing/corrupt/incompatible model → reject before actuation; previous or declared recovery/procedural route.
- Unsupported skill/equipment/topology/proficiency → novice fallback only if explicitly evaluated; otherwise `Unavailable`.
- Cross-family/physical-regime or undeclared topology request → exact
  `MOTOR_POLICY_FAMILY_MISMATCH`; no nearest-family or silent approximation,
  only declared procedural/animation/recovery route.
- Invalid `BodySchema` compilation, overlay revision or stable-ID remap →
  reject before physical/tensor/safety publication and retain prior complete
  projection/topology.
- Hidden adaptation/generator/router history or runtime optimizer state →
  reject the artifact/session before activation; no state is reconstructed from
  arrival order.
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
| BEHAVIOR-01 | habits/tactics with ai-host/adapter present and absent | 0 direct MotorAction/gameplay mutation from habits; all actions pass AgentIntent/WorldCommand; scenario outcomes exact offline | deterministic Utility + bounded GOAP/tactical profile |
| TRAIN-P1 | pinned training backend export/deployment | ADR-066 profile thresholds; license/SBOM complete; export reproducible from pinned config except declared stochastic metrics | engine-owned headless physics lab |
| TRAIN-MAC-P0 | generated deterministic 2-DoF development smoke | 4 fixed seeds; 8 envs ×256 steps; 1 000 observations; PyTorch/ONNX max abs error ≤`1e-5`; 0 NaN/Inf; ≤10 minutes on MPS or declared CPU fallback | CPU smoke; stop this training lane on export/parity failure |
| BODY-SCHEMA-P1 (future) | BodySchema compilation, overlays and topology remap | exact physics/tensor/safety roots; no duplicate owner; every invalid/remap fault publishes nothing | retain prior topology/projection and procedural tier |
| MOTOR-HUMANOID-MVP-P1 (future) | one million steps plus flat/terrain/push/fall/get-up profile | 0 NaN/Inf; flat survival ≥99%; declared terrain success ≥95%; velocity RMSE ≤0.20 m/s; heading error ≤7°; bounded slip and command resume after get-up | procedural/animation/ragdoll route |
| MOTOR-ADAPTATION-P1 (future) | abrupt equipment/damage/friction/latency changes | explicit inputs apply immediately; bounded TCN/GRU history state improves declared metric without weight updates | explicit-only controller and recovery route |
| MOTOR-RETENTION-P1 (future) | new specialist plus old-skill/transition/interruption corpus | new skill passes without declared old-skill or transition regression | retain parent bundle or separate expert |
| MOTOR-REPLAY-P1 (future) | action/state/snapshot save/load/replay plus worker and Windows/Linux permutations | canonical applied action/full state/snapshot chain exact; independent evaluator parity reproduces it | reject learned artifact/profile and use procedural route |
| MOTOR-SKILL-CHUNK-P1 (future) | typed primitive/contact/chunk and interruption corpus | exact roots, no text/token/embedding in physical hot path, closed-loop tracking and safe interrupt | reject chunk and use direct-command/procedural/recovery route |
| MOTOR-MORPHOLOGY-TRANSFER-P1 (future) | fixed-body through graph/shared-head/co-training/fault ablations on held-out bodies | within-family success, safety, latency and body-specific-rule thresholds pass without arbitrary-topology claim | retain fixed-body/family route |
| MOTOR-ROLLOUT-P1 (future) | candidate/worker/completion permutations from one checkpoint | exact ordered scores/winner/chunk root and zero branch side effects | use declared base chunk; do not activate rollout profile |
| MOTOR-DISTILL-P1 (future) | specialist teachers to a family-shared student and optional compiled child | old/new skill retention, exact runtime state/action parity, limits and fallback pass | reject child and retain parent/procedural route |

These checks define runtime and authoring behavior; they are not organizational approval.
The rows marked `future` remain `NOT_RUN(NO_PRODUCTION_CONSUMER)` and do
not create an R5/v1 completion claim in this documentation-only change.
