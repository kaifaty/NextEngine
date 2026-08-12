# ADR-066: Contact-centric physical skills and morphology-conditioned motor architecture

| Поле | Значение |
|---|---|
| ID | ADR-066 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-12 |
| Последняя проверка | 2026-08-12 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](../28-skeletal-animation-retargeting-and-ik.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-013](013-self-contained-physical-avatar-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md) |
| Заменяет | полностью [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md); сохраняет его hierarchy, family, exact-action replay and fallback invariants, уточняя no-text boundary, physical action chunks, graph-conditioned control and bounded rollout/training architecture |
| Заменён | narrowly [ADR-068](068-static-morphology-cache-and-action-chunk-field-closure.md) for static cache identity and the closed `PhysicalActionChunk` field set |

## Контекст

ADR-057 правильно отказался от одной universal end-to-end модели и от Mamba
как обязательного low-level foundation. Однако его ещё не реализованный
`MotionReferenceHorizon` оставлял несколько важных вопросов открытыми:

- является ли natural language частью physical/motor runtime;
- что именно переносится между телами: joint motion или физический смысл;
- как отличить static morphology от per-tick dynamic state;
- где проходит граница между contact plan, action chunk и actuator action;
- как variable topology получает shared joint policy без fixed output width;
- можно ли использовать cloned physics rollouts, не создавая второй replay API;
- какие teacher, critic, motion-prior и distillation decisions остаются только
  training concerns;
- как включать дорогие learned paths только там, где они дают продуктовую
  ценность, не делая их обязательными для R5/v1 или тысяч NPC.

Нужна одна архитектурная формулировка, которая принимает переносимый
contact-centric смысл действия, но не принимает speculative wire schemas,
конкретную neural architecture или arbitrary-morphology quality claim до
production consumer и ProductCheck по ADR-046.

## Решение

### 1. Принятая иерархия и граница статуса

Целевая Physical Embodiment hierarchy:

```text
Strategic Agent / gameplay task
  → typed AgentIntent / PhysicalAvatarIntent
  → Skill Orchestrator + Contact/Affordance Planner
  → PhysicalActionChunk
  → morphology-conditioned low-level controller
  → fixed safety + PD/SPD/impedance layer
  → Physics
```

Приняты как архитектурные invariants:

- physical result, contact/support/trajectory intent and actuation are
  separate authority layers;
- переносимый смысл навыка задаётся через contacts, support, root/CoM,
  effector/object trajectories, force envelopes and success/failure evidence,
  а не через joint indices;
- concrete bodies enforce their own kinematics, actuator limits, damage and
  safety through an exact `BodySchema`/instance projection;
- every high-frequency input is typed, bounded and numeric;
- low-level output always passes the same engine-owned safety and fixed
  controller layer before Physics;
- learned execution is optional and always has a family-compatible
  procedural/animation/recovery fallback.

Exact future `PhysicalAvatarIntent`, `MotorSkillCommand`, `ContactPlan`,
`PhysicalActionChunk`, graph-policy, adapter, expert, hypernetwork and rollout
wire profiles remain `Proposed` until a production consumer and its mapped
ProductCheck exist. ADR-058/SPEC-35 current fixed-humanoid substrate and its
procedural R5 route are unchanged.

### 2. Natural language завершается до physical runtime

Natural language MAY exist only at a human/dialogue/optional `ai-host`
boundary that compiles one instruction into a validated typed `AgentIntent`.
After compilation, raw text, token sequence and language embedding are not
inputs to:

- `PhysicalAvatarIntent` execution;
- Skill Orchestrator or Contact/Affordance Planner;
- `MotorSkillCommand`, `ContactPlan` or `PhysicalActionChunk`;
- `MotorObservation`, policy state or motor inference;
- safety, PD/SPD/impedance or Physics.

The compiler resolves stable entity/skill/primitive IDs, reference frames,
numeric targets, masks, constraints, expiry and evidence predicates before the
physical task is admitted. Text labels MAY remain presentation-only metadata
or offline provenance, but they do not enter command identity, policy input,
route selection, replay equivalence or success proof.

No LLM, tokenizer, text encoder, network or process call is allowed in the
physical hot path. Missing or ambiguous compilation produces a typed
rejection/clarification path; it never falls through to free-form motor input.

### 3. `BodySchema` is a heterogeneous Physical Interaction Graph

`BodySchema` remains immutable and engine-owned. Its semantic graph contains:

- body nodes with stable IDs, shape/mass/inertia/rest frame and semantic role;
- joint edges with stable IDs, endpoints, axes/type/limits, passive dynamics,
  actuator envelope, health capability and enabled/topology semantics;
- effectors with support, manipulation, strike, wing, tail or other declared
  physical roles;
- colliders, attachment slots, symmetry/capability relations and the exact
  mappings used by physics and motor schemas.

The graph is heterogeneous: node, joint, actuator, effector and attachment
records have different closed semantics. A skeleton hierarchy or backend
articulation index alone is not a `BodySchema`.

Static morphology is encoded separately from dynamic state. Topology, lengths,
masses, inertia, joint kinds/limits, actuator envelopes and semantic roles MAY
produce a deterministic cached per-node/per-edge morphology representation.
Pose, velocity, joint state, contacts, external impulses, available power,
fatigue, damage and sensor validity are per-tick dynamic inputs and are never
written into the static cache.

Cache identity binds the exact schema, compiled descriptor, effective instance
projection, topology and encoder/profile hashes. It invalidates on creation,
schema/compiled-body change, topology change, material morphology/equipment
change, actuator lock/restore or another declared effective-projection change.
Cache warmth, worker identity and rebuild completion order never affect an
authoritative action; the cache is reconstructible from the bound sources.

### 4. Typed physical vocabulary instead of action-string explosion

The physical task boundary uses a small compositional vocabulary rather than
one enum variant per authored animation or object-specific action. A future
consumer MAY admit approximately 10--30 stable primitive kinds such as:

```text
MoveTo, Reach, EstablishContact, MaintainContact, Release,
ApplyForce, SupportWeight, MoveObject, PreserveBalance,
RecoverBalance, ShiftCenterOfMass
```

Primitive kind is a stable ID/closed enum; target is a stable entity/feature
reference plus numeric position/orientation in a declared reference frame;
constraints and success/failure predicates are typed IDs, masks and fixed-point
bounds. The model receives one-hot/embedding-table values only as a derived
numeric encoding of those IDs, never their display strings.

Effectors are selected by semantic capability (`AnyManipulationEffector`,
`AnySupportEffector`, declared strongest/nearest under one canonical metric, or
`Specific(BodyNodeId)`). Selection validates against the exact effective body
projection and uses a canonical total order. It does not assume human
left/right limbs or guess a body part from an authoring name.

The term **physical skill contract** denotes composition of existing
boundaries, not a fifth public record:

```text
PhysicalAvatarIntent(result + constraints + evidence predicates)
  + MotorSkillCommand(skill/phase/style/interruption)
  + ContactPlan
  + PhysicalActionChunk
```

### 5. `PhysicalActionChunk` is the single short-horizon reference

The unconsumed `MotionReferenceHorizonV1` proposal is withdrawn and renamed
before implementation. Its one successor concept is `PhysicalActionChunk`.
No migration is required because no production wire consumer exists.

A chunk is a bounded reference, normally covering a project/profile-declared
`250..1000 ms`. It binds source intent/skill/contact/scene/body revisions and
contains only the subset required by its profile:

- root and center-of-mass trajectories;
- ordered effector and object-relative trajectories;
- selected contact schedule and support transitions;
- desired force direction/range and allow-sliding semantics;
- motion phase, style/proficiency ID and interruption/cancel metadata;
- exact start/end tick, validity bounds and chunk hash.

It does not contain already accepted joint commands, torques, an open-loop
actuator sequence, natural language or mutable object references. A learned
shared latent MAY exist inside an immutable bundle as a fixed-width declared
numeric feature segment, but an opaque framework `Tensor` is not a public
cross-bundle contract and cannot replace the structured physical fields.

The controller remains closed-loop: every motor tick rebuilds current
observation, advances explicit temporal state and produces a complete action;
every physics substep revalidates safety and actuator limits. The next chunk
may be prepared in parallel but publishes only at its fixed deterministic
boundary. Emergency interruption enters the declared brace/safe-fall/recovery
chain rather than blindly finishing the chunk.

### 6. Morphology-conditioned low-level controller

The first learned fixed-humanoid comparator remains a feed-forward MLP with
residual position targets and fixed PD at the accepted 60/240 Hz boundary.
This is the least complex profile and remains ahead of a graph controller in
the promotion ladder.

For within-family variable morphology/topology, the Proposed controller shape
is:

```text
cached static node/edge morphology
  + dynamic node/edge state
  + current PhysicalActionChunk
  → 1--2 bounded local graph-message layers
  → bounded global coordination attention
  → replaceable temporal core
  → one shared per-joint/per-actuator head
  → joint targets
```

Local graph communication follows declared physical edges. Global attention
coordinates distant effectors under an exact mask/token/order/profile.
Temporal state uses the generic SPEC-27 state boundary: frame-stacked MLP/TCN
is the stateless/window comparator, GRU is the first recurrent baseline, and
Mamba/SSM is only an equal-parameter/context/latency experiment. There is no
Mamba-specific public state type or preferred status.

The joint head is shared across compatible edges and consumes the joint,
parent, child, chunk and family context. A fixed `Linear(hidden → N joints)`
is permitted only for the fixed-humanoid comparator and cannot support a
variable-topology claim. The first shared head emits position and optional
velocity targets with fixed engine gains. Adaptive stiffness/damping, residual
torque, direct torque or muscle activation require separate safety profiles
and ProductChecks.

### 7. Capability facts, family factorization and experts

RPG attributes never live in skill weights. RPG/Mechanics own strength,
dexterity, endurance, damage, fatigue and equipment facts. The physical
capability compiler projects them into revision-bound mass/inertia/ROM,
torque-speed-power, latency, precision, fatigue/recovery and sensor limits.
Those facts both constrain actual actuation and appear explicitly in the
allowed observation.

The target factorization is:

```text
shared family physical backbone
  + morphology adapter
  + skill adapter
  + explicit locomotion-regime expert/router
```

Sharing covers compatible physical semantics such as support, contact, balance,
force and momentum. Humanoid, general-legged, aerial, aquatic, serpentine and
other materially different regimes remain separate policy families. A dragon
walking and flying uses explicit ground/flight routes and transition experts;
no result implies one arbitrary human/horse/dragon/spider policy.

An internal soft router is allowed only as one bounded action producer with
all future-action-affecting state explicit in SPEC-27. It cannot grant a skill,
bypass compatibility/safety or mix incompatible family outputs.

### 8. Optional bounded cloned-physics candidate evaluation

A high-fidelity profile MAY generate exactly `K` candidate
`PhysicalActionChunk` values and evaluate them from one immutable canonical
physics checkpoint for a fixed simulation horizon. This is ephemeral
counterfactual evaluation inside the skill planner, not durable branching
replay, save fork or a second gameplay world.

The profile fixes `K`, horizon, candidate order, scoring schema and logical
work budget. Every branch:

- starts from the same validated SPEC-26 checkpoint plus the ADR-059
  continuation closure required for an exact starting witness; a private
  copy-on-write/native clone is only an optimization and never snapshot authority;
- owns an isolated RNG stream derived from the world root, subject, command/
  chunk epoch and candidate ordinal when randomness is required;
- publishes no command, event, save, telemetry side effect, network/LLM call
  or mutable authority;
- returns only a bounded fixed-point score/evidence record.

Scores use canonical task progress, balance, collision/damage, energy, contact
and safety facts. The profile declares component priority and direction; the
result is totally ordered lexicographically by that vector and candidate ID.
All `K` branches complete as bounded synchronous logical work before selection;
worker count, completion order and wall time cannot choose a winner. A wall
overrun is performance evidence, not a different result. Resource/profile
failure leaves the already declared base/procedural chunk authoritative.

Only the selected chunk is staged at the normal commit boundary. Replay stores
the candidate-set root, scoring-profile hash, ordered score/evidence roots,
winner ID and selected chunk hash. Ordinary authoritative replay consumes the
recorded selected chunk without exposing `branch/step_with`; a separate parity
check MAY recompute candidates and selection. Supporting this profile therefore
does not promote general branching replay infrastructure.

The profile is intended only for player, boss or a small manifest-bounded set
of important nearby subjects. It is not a default for all NPCs and is not an
R5/v1 blocker.

### 9. Training-only architecture

The Proposed training order is:

1. independent specialist teachers for locomotion, flight, climbing,
   manipulation/weapon, recovery and other bounded regimes;
2. supervised distillation into the family/shared-latent student;
3. joint PPO fine-tuning on continuous joint targets with GAE/normalized
   advantages and hard engine safety outside reward;
4. retention, held-out morphology, fault, transfer, export and runtime parity
   evaluation before immutable child-bundle publication.

The runtime actor is maximally shared only inside its declared family. The
training critic is more strongly conditioned on morphology, capabilities,
skill and regime, or uses declared family heads, so advantage estimates do not
average physically unequal bodies. Critic inputs may contain declared
teacher-only facts, but the critic is never exported, never enters runtime
decision/replay state and cannot leak privileged data into actor inputs.

A motion prior MAY be trained as morphology-conditioned structured inpainting:
masked keypoints/poses, contacts, object trajectories, styles and physical
goals are reconstructed under physics. Natural-language captions may be
offline authoring/provenance labels compiled into the same typed IDs before
training batches; runtime text conditioning is not required or implied.

Fault distribution explicitly covers disabled/locked/removed actuator or node,
reduced torque/ROM, changed mass/load, delayed/missing sensor and topology
change. Current health/power/enabled masks are observations. Unobserved fault
identification may use the explicit temporal state; no hidden evaluator cache
is allowed.

After a family-wide teacher proves quality, a morphology hypernetwork MAY
compile an immutable small MLP/GRU student or adapter for one exact
`BodySchema`/projection. The generated child has its own hash, complete
compatibility key, golden corpus, runtime cost, fallback and retention/parity
checks. Hypernetwork generation is load/build-time artifact production, never
runtime weight mutation. A topology change invalidates the child; the
family-wide compatible route or recovery fallback covers the transition until
another validated child is available.

### 10. Deterministic control LOD and cost discipline

Control quality tiers are explicit and selected only from canonical simulation
facts and manifest tokens:

```text
Abstract outcome
  → animation + navigation/IK/procedural controller
  → learned closed-loop motor controller
  → learned controller + bounded candidate rollouts
```

Simple `MoveTo`, `LookAt`, ordinary reach and predictable authored interaction
SHOULD use navigation, motion matching, IK or procedural control when they meet
the required envelope. Learned control is reserved for contact-rich,
disturbed, damaged or otherwise unsupported cases. Measured CPU/GPU load,
renderer visibility, wall time and worker completion cannot select a tier.

The fixed-humanoid numeric flat-command substrate remains the first current
step. Graph attention, action chunks, cross-embodiment latent transfer,
hypernetwork students and rollout search are independently promotable future
profiles, not a mandatory stack that must land together.

## Рассмотренные варианты

### Natural language or a VLA embedding in every motor frame

Отклонено: expensive, nondeterministic and unnecessary after typed intent
compilation; it weakens validation, replay and testability.

### One enormous action enum

Отклонено: object/body/damage/style combinations recreate an animation state
explosion. Small physical primitives plus numeric targets remain compositional.

### Language or BodyGraph directly to torques

Отклонено: it conflates task semantics, contact planning, morphology decoding,
safety and actuation and provides no stable fallback or owner boundary.

### One monolithic universal policy across all creatures

Отклонено: cross-regime dynamics and effectors differ too much for an
unsupported quality claim. Sharing is bounded by family plus explicit adapters
and experts.

### Mamba as mandatory temporal core

Отклонено: GRU/TCN/MLP comparators are cheaper and better established for the
initial bounded history. Mamba must earn promotion under the same budget.

### Open-loop joint action chunks

Отклонено: contact and damage invalidate predicted actuator sequences.
`PhysicalActionChunk` is a reference; low-level control and safety remain
closed-loop.

### General branching replay as prerequisite for candidate rollouts

Отклонено: a bounded ephemeral fork can use one canonical checkpoint and
record one selection receipt without exposing durable branch operations or
paying their storage/API cost.

## Последствия

- ADR-057 becomes historical; its accepted invariants continue through this
  ADR while its `MotionReferenceHorizon` name is retired before a consumer.
- High-frequency physical runtime has no natural-language dependency and stays
  offline, typed, bounded and replayable.
- BodySchema cache and graph-policy shapes can support variable limb counts
  without making arbitrary topology a current claim.
- The current fixed-humanoid MLP/procedural substrate remains the least-complex
  baseline; no roadmap stage becomes complete from this decision alone.
- Expensive inference, compiled students and candidate rollouts are explicit
  quality/performance tiers with deterministic selection and fallback.
- Training may use richer critics, teachers and priors, but shipped actors,
  state and action contracts remain engine-owned and inference-only.
- Exact new schemas and model/toolchain choices wait for production consumers
  under ADR-046.

## Product checks

These future checks remain `NOT_RUN(NO_PRODUCTION_CONSUMER)` until their
profiles are promoted together with implementation:

| Check | Expected | Fallback |
|---|---|---|
| `MOTOR-SKILL-CHUNK-P1` | Typed primitive/target/contact/chunk identities and roots are exact; no text/token/embedding reaches the physical hot path; overlapping chunks remain closed-loop and interrupt safely | reject chunk and use declared direct-command/procedural/recovery route |
| `MOTOR-MORPHOLOGY-TRANSFER-P1` | Fixed body → randomization → graph/shared-head → co-training/semantic-subgoal/fault ablations report held-out within-family transfer, latency, safety and body-specific rules without arbitrary-topology claim | retain the fixed-body/family route |
| `MOTOR-ROLLOUT-P1` | Candidate/worker/completion permutations yield the same ordered scores, winner and selected chunk root; branches leak no command/event/save/RNG effect | use the declared base chunk; do not activate rollout profile |
| `MOTOR-DISTILL-P1` | Specialist teachers → family-shared student → optional compiled child retain required old/new skills, exact runtime actions/state, limits and fallback on shipping targets | reject child and retain validated parent/procedural route |

Existing `BODY-SCHEMA-P1`, `MOTOR-HUMANOID-MVP-P1`, `MOTOR-ADAPTATION-P1`,
`MOTOR-RETENTION-P1`, `MOTOR-REPLAY-P1`, `ANIM-HYBRID-P1`, applicable
`MODEL-*` and conditional `performance` checks remain independently required.

## Research provenance

This decision is informed by
[MetaMorph](https://arxiv.org/abs/2203.11931),
[Contact-Anchored Policies](https://arxiv.org/abs/2602.09017),
[Latent Action Diffusion](https://arxiv.org/abs/2506.14608),
[Action Chunking with Transformers](https://arxiv.org/abs/2304.13705),
[GCNT](https://arxiv.org/abs/2505.15211),
[Shared Modular Recurrence](https://arxiv.org/abs/2506.08630),
[HyperDistill](https://arxiv.org/abs/2402.06570),
[MorFiC](https://arxiv.org/abs/2603.14554),
[MaskedMimic](https://research.nvidia.com/labs/par/maskedmimic/) and
[Random Joint Masking](https://arxiv.org/abs/2403.00398).
They motivate hypotheses and evaluation ladders only; they do not create
shipped support, public vendor types or license rights for datasets/models.

## Supersession

ADR-066 полностью заменяет ADR-057. ADR-057 сохраняется как historical record
со статусом `Superseded` и backlink. Active specifications, routing, glossary,
roadmap and traceability use ADR-066.
