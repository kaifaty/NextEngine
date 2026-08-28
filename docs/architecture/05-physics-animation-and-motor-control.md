# SPEC-05: Physics, animation и motor control

| Поле | Значение |
|---|---|
| ID | SPEC-05 |
| Статус | Accepted |
| Версия | 3.3 |
| Последняя проверка | 2026-08-28 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md), [ADR-013](adr/013-self-contained-physical-avatar-boundary.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-062](adr/062-r5-physx-humanoid-performance-authority.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md) |
| Дополнительные зависимости V2.9 | [SPEC-36](36-functional-tissue-condition-and-injury.md), [SPEC-37](37-character-embodiment-and-surface-deformation.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |
| Дополнительные зависимости V3.1 | [ADR-090](adr/090-linux-only-v1-and-indefinitely-deferred-windows.md), [ADR-091](adr/091-linux-release-performance-authority.md), [ADR-093](adr/093-deterministic-r5-worker-placement.md), [ADR-094](adr/094-confidence-gated-relative-warnings.md) |
| Дополнительные зависимости V3.2 | [ADR-096](adr/096-active-kernel-linux-performance-cohort.md) |
| Дополнительные зависимости V3.3 | [ADR-098](adr/098-bounded-intact-topology-functional-anatomy-condition-vertical.md) |
| Заменяет | SPEC-05 3.2; admits the bounded revision-bound directional capability clamp before fixed-PD rate limiting without changing the default no-envelope path or physical topology |

## Source of truth и ownership

Для physical LOD `FullArticulation` и `SimplifiedActiveRagdoll` Physical Embodiment physics world владеет engine-owned body pose, velocities, contacts, constraints и canonical snapshot; replaceable PhysicsBackend является private compute adapter, а не source of truth. Для `CapsuleAnimation` validated controller владеет collision transform, animation graph — visual local pose. Physical `Abstract` не совпадает с current World Services `PopulationTierV1`: World Services владеет durable tier/logical region/activity, а RPG Framework — aggregate outcomes. Даже `PopulationTierV1::Active` не является pose или доказательством physical traversal. Renderer всегда читает immutable presentation projection. Ни backend, World Services, animation, AI, gameplay script, importer, renderer, UI/camera, ни LLM не могут напрямую записать active physics authority.

Physical Embodiment владеет semantic `BodySchema` compilation,
backend-neutral descriptors, physics stepping, motor observation/action,
policy safety/resolution/supervision, topology transactions, animation bridge,
LOD coordinator и physical support checks. RPG skill proficiency и Agent
habits остаются за пределами этого ownership. Equipment/stats/damage/fatigue
sources remain with their owning RPG/Mechanics domains; Physical Embodiment
consumes only an immutable revision-bound effective projection.

SPEC-36 specializes that projection for functional muscle groups, tissue
condition, fracture and retained/detached topology. SPEC-37 specializes the
one-way `RenderPose` to skinned/deformed surface path. Neither document makes a
renderer deformer or inferred muscle recruitment a physical or gameplay owner;
the current joint-target plus fixed safety/PD route remains unchanged.

Boundary является self-contained по ADR-013: внешние research документы не задают requirements, phases, public types или support semantics. Frozen annex сохраняется только как ненормативная provenance. ADR-058 выбирает PhysX 5.9.0 как единственный production backend; Jolt/Bullet не являются runtime fallback.

## Public boundary

Нормативная public boundary специализируется [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md) и [SPEC-28](28-skeletal-animation-retargeting-and-ik.md). Она использует только engine-owned body/shape/material/joint/query/contact/snapshot, motor tensor/action/state и skeleton/clip/graph/retarget/IK descriptors. Backend world/device/session handles, native task objects, model-runtime tensors и animation-graph implementation nodes остаются private; public contracts не содержат ECS, OS, vendor, importer либо backend types.

Основные engine-owned data contracts:

| Contract | Обязательные поля/semantics |
|---|---|
| `BodySchema` / `BodyInstanceProjection` | immutable heterogeneous physical-interaction graph plus exact revision-bound effective morphology/equipment/stats/damage/fatigue projection; exact advanced V1 wire shapes remain Proposed |
| `PhysicalBodyDescriptor` | deterministic SPEC-26 projection of BodySchema: PersistentId, parent relation, mass/inertia, collision geometry AssetId, material tags, canonical frame/axis, limits, actuator bounds |
| `PhysicalAvatarIntent` | intent ID, issuer, start/expiry tick, desired locomotion velocity/facing/posture/manipulation target, priority, safety constraints |
| `PhysicalActionChunk` | Proposed bounded root/CoM/effector/object/contact/force/support reference; never an open-loop actuator sequence or natural-language payload |
| `MotorObservation` | schema/model version, normalized root/joint state, target features, contacts/support/terrain features, previous action, masks |
| `MotorAction` | schema/model version, joint targets/torques/controller gains, confidence/validity flags |
| `ContactEvent` | continuity `contact_id`, participants PersistentId/body slot, point/normal, relative velocity, impulse bounds/effective mass, material tags, begin/persist/end, physics tick |
| `PhysicalOutcome` | normalized fall/recover/traverse/hit/grab completion candidate, доказанный physics state |
| `RenderPose` | source physics ticks, hierarchy-local transforms, interpolation policy, topology revision |

Physical archetype, policy manifest, skill/proficiency-aware resolver, ActivePolicyRoute и transition contracts специализируются [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md); они не меняют PhysicsBackend ABI или MotorObservation/MotorAction ownership.

Units/right-handed axes соответствуют SPEC-03. Vendor enumerations и handles не входят в serialized schemas.

## Physics step и data flow

1. Gameplay/tactical layer публикует bounded `PhysicalAvatarIntent`.
2. LOD coordinator выбирает tier только на permitted transition point.
3. Skill Orchestrator resolves skill phase/style/interruption; deterministic
   Contact/Affordance Planner MAY publish a bounded contact plan.
4. Simple locomotion feeds semantic command features directly. Contact-rich
   skills MAY add an authored, motion-matching or learned
   `PhysicalActionChunk`; neither path writes physical state.
5. PolicySupervisor предоставляет committed compatible family route;
   Observation builder reads current physics, exact BodyInstance projection,
   allowed context and compact explicit adaptation state.
6. Low-level motor controller (procedural, learned policy or recovery fallback)
   creates one complete actuator-target `MotorAction` without blocking I/O.
7. Safety/actuator layer checks finite values, schema/model/body/topology/state
   hashes, action age, joint/rate/torque-speed-power/energy/contact/grip limits
   and applies deterministic fixed PD/SPD where selected.
8. PhysicsBackend выполняет CPU substep и выдаёт raw contacts/state; adapter сначала строит `QuantizedPhysicsProjectionV1` по `PhysicsQuantizationProfileV1` [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), затем нормализует scene-query/contact records и сортирует contacts по определённому ниже полному canonical total key. Backend callback, worker completion, native handle, raw float bit pattern и manifold insertion order не участвуют в public ordering.
9. Outcome resolver использует contact continuity для suppress repeated-hit/resting-contact exploits и предлагает `Outcome` WorldCommand для общего stage-9 validator [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md); контакт сам не меняет health/quest, а backend callback не коммитит gameplay.
10. Pose bridge публикует RenderPose, telemetry и replay hash.

ADR-098 adds one current intact-topology specialization to step 7. A validated
`BodyCapabilityEnvelopeV1` reconstructs directional actuator capacity from the
exact BodySchema/profile and committed RPG condition revision. The controller
clamps effort and its previous-effort state into that interval before the rate
limit, so declared zero transmission is immediate. The envelope is not durable
physical state; malformed or foreign bindings reject the complete step.

LLM, tokenizer/text encoder, free-form text, language embedding, `ai-host`,
network и filesystem запрещены на шагах 3–9. Skill/primitive/style/constraint
meaning reaches them only as stable typed IDs, masks and numeric parameters
compiled before Physical Embodiment. The current
articulated standing fallback and the first Proposed learned humanoid profile
use 240 Hz physics/actuator and 60 Hz motor/policy cadence. A separate capsule
locomotion profile MAY retain its declared 120/60 Hz cadence; it cannot stand
in for PHYS-P4. The first Proposed learned humanoid profile uses 60 Hz MLP
inference with residual joint-position targets and fixed engine PD; it is not
a v1 requirement. The PD/safety layer applies the last accepted action on
intermediate substeps. Portable standard-op ONNX with a private ONNX Runtime
adapter is Proposed. Reference performance oracle remains p99 ≤0.5
ms/avatar/inference or ≤2 ms for a batch of 16. Wall-clock duration, worker
completion order and measured CPU load MUST NOT choose authoritative
MotorAction. Only manifest-declared logical motor tick or canonical injected
fault may declare action unavailable; bounded hold/recovery semantics remain
SPEC-27 authority.

## Backend и runtime/training parity

ADR-058 принимает PhysX 5.9.0 reduced-coordinate articulations как
единственный production backend. CPU scene является canonical execution plane;
Isaac Lab GPU/batched simulator — correspondence mirror. Production roots не
имеют reference/Jolt/Bullet fallback, `PhysicsBackendPolicy` или mid-session
switch. Missing SDK/build/profile mismatch is a typed pre-activation failure.
Every 60 Hz Stage 0 motor frame applies four 240 Hz fixed-point PD/safety
substeps through the same PhysX path used by the procedural standing fallback.

Export pipeline MUST записывать model SHA-256, evaluator-format profile,
closed capabilities/operators, input/output/state schema hashes, normalization,
training simulator/build/config and golden observation/action/state corpus. The
first Proposed profile is portable fixed-shape standard-op ONNX; ONNX Runtime
is a private adapter. Runtime rejects mismatch/non-finite/unsupported model and
uses the deterministic controller fallback.

## Deterministic numeric boundary

Используемые здесь reusable numeric schemas определены SPEC-21 как refinement этой owned physical boundary; canonical stage-9 ledger/encoding semantics определены superseding ADR-022.

Raw backend floats остаются private operational state выбранного PhysicsBackend и MAY использоваться только для declared per-field backend/runtime-training correspondence. До любого gameplay comparison, tie-break, scene-query result, contact continuity/classification, PhysicalOutcome, DomainEvent payload либо cross-target hash adapter MUST преобразовать decision-relevant values через `PhysicsQuantizationProfileV1`/`PhysicsQuantizationRuleV1` SPEC-21. Rule фиксирует source IEEE format, canonical unit, destination `FixedPointDescriptorV1`, exact scale numerator/denominator, offset и bounds. Adapter декодирует finite IEEE bits как exact rational, нормализует negative zero и округляет `NearestTiesToEven`; NaN, infinity и undeclared rule fail closed до outcome proposal, а out-of-bounds/intermediate overflow атомарно abort-ит transaction как `NUMERIC_OVERFLOW`. Tolerance не выбирает branch и не превращает exact mismatch в `Pass`.

Adapter сначала упорядочивает участников contact по `(PersistentId canonical bytes, body_slot)` и при перестановке канонически меняет направление participant-relative vector fields. `contact_feature_id` является engine-owned stable shape-feature ID из validated descriptor/asset mapping; raw backend feature index не может попасть в key без exact mapping. Полный ключ contact равен `(physics_tick, substep, participant_low, participant_high, contact_feature_id, quantized point, quantized normal, quantized relative velocity, quantized impulse bounds, quantized effective mass, canonical material tags)`. Все numeric elements ключа являются raw fixed-point integers после SPEC-21 quantization. Exact duplicate normalized records дедуплицируются до continuity classification. `contact_id` и begin/persist/end выводятся только после этой сортировки из canonical participant/feature continuity, а не из backend handle или callback ordinal.

На одинаковых target triple/build/config два successful run MUST иметь exact full authoritative `state_root`, exact contact sequence и exact outcome sequence. Между Windows x86_64 и Linux x86_64 `physics_projection_root` над `CanonicalBinaryV1` bytes `QuantizedPhysicsProjectionV1`, canonical contact/query order и gameplay outcomes MUST совпадать byte-for-byte; raw backend samples сравниваются отдельно только по registered per-field correspondence tolerance и не входят в cross-target exact projection. Для `MotorAction` versioned action schema отвергает non-finite output, декодирует каждый finite raw IEEE value как exact rational, применяет safety clamp по exact bounds и конвертирует его round-to-nearest-ties-to-even через named `FixedPointDescriptorV1` из `AuthoritativeNumericProfileV1` до actuation. MOTOR-P1 tolerance сравнивает raw model evaluators, но не разрешает различие applied canonical action.

## Policy lifecycle specialization

Game-time learning не изменяет model bytes. PolicyResolver и PolicySupervisor
SPEC-14 выбирают immutable family/skill route по exact `BodySchema`/effective
projection, equipment, proficiency, topology mask and intent. Humanoid/biped,
general-legged, serpentine, aquatic, aerial, modular and musculoskeletal
profiles are separate families; cross-regime fallback is never inferred.
Switch выполняется только на motor-tick safe point, никогда не копирует physics
pose и не совмещается с topology/LOD transaction. До atomic commit previous
route остаётся единственным actuator source.

Known physical parameters enter observation explicitly. Optional fast
adaptation reads only the declared recent action-response history and stores
its complete bounded state in SPEC-27 records; runtime gradients, hidden
evaluator history and optimizer state are forbidden. MLP is the first Proposed
low-level humanoid comparator, TCN/GRU the first adaptation comparators. Mamba
is an optional equal-budget experiment for longer history/generation/planning,
not a default foundation.

A later variable-morphology family MAY use cached static BodySchema node/edge
features, per-tick dynamic graph state, bounded local message passing, global
coordination attention, a generic temporal core and one shared per-joint head.
This is independently Proposed: the fixed-humanoid MLP remains the first
learned comparator, GRU the first recurrent baseline, and arbitrary cross-family
topology support is never inferred.

A Supported learned route produces identical canonical applied action and full
policy state across shipping targets. Replay stores and validates that action/
state chain plus required physics snapshots and does not rely only on evaluator
re-execution. Re-evaluation is a separate parity ProductCheck and must reach
the same canonical result.

## Animation bridge

Animation assets задают reference motions, intent features и presentation-only secondary layers через exact SPEC-28 schemas. В physical tiers animation MAY предлагать motor targets, но не final pose. Root motion становится revision-bound validated intent через production command/motor boundary, а не teleport. Physical IK решается внутри authoritative physics/motor ordering; presentation IK читает immutable pose и не меняет contacts/gameplay hashes. Graph, retarget, IK и LOD имеют fixed canonical order. В `CapsuleAnimation` collider motion следует validated locomotion controller, а visual pose — animation graph; расхождение ограничено project-locked leash и диагностируется.

## Physics LOD

| LOD | Simulation | Разрешённое применение |
|---|---|---|
| `FullArticulation` | полный articulated body + motor + contacts | player, near/important NPC, product-check scenarios |
| `SimplifiedActiveRagdoll` | reduced body/actuators, обязательные root/support contacts | nearby background NPC under interaction |
| `CapsuleAnimation` | capsule collision/navigation + animation pose | distant visible NPC без physical interaction |
| `Abstract` | logical region/time/task state | unloaded/non-visible NPC |

Inside `FullArticulation`, controller quality is separately project-locked:
procedural/IK or learned closed-loop, with optional bounded cloned-physics
candidate evaluation only for a small manifest-bounded set of important
subjects. This does not create another physical LOD or world authority. The
rollout tier records one selected chunk and remains Proposed under ADR-066.

Physical LOD request выводится только из current canonical simulation facts: quantized simulation distance, gameplay importance, interaction/contact state, simulation-owned visibility fact, PersistentId, committed `PopulationTierV1` view и versioned integer budget tokens из manifest. Renderer camera/frustum/occlusion, presentation visibility, measured CPU time, wall clock, worker load и completion order MUST NOT влиять на LOD. World Services может предложить residency/physical capability change, но LOD coordinator отдельно валидирует physical transition; unsupported abstract precise outcome детерминированно требует upgrade/defer и не телепортирует pose. Safety constraints имеют приоритет. Downgrade запрещён при external contact impulse, fall/recovery, grab, topology transaction, quest-critical physical interaction или unstable support.

Transition MUST быть explicit state machine `Prepare → Validate → Commit → Stabilize`:

- lower→higher создаёт bodies из canonical descriptor, проецирует pose/velocity, проверяет penetration/support и прогревает 2–8 hidden substeps;
- higher→lower фиксирует root/logical state, сохраняет required physical outcome, проверяет forbidden conditions и только затем удаляет bodies;
- failed validation оставляет исходный LOD без partial mutation;
- `Stabilize` ограничивает artificial energy; position jump ≤5 cm, facing ≤3°, linear velocity discontinuity ≤0.3 m/s для accepted transition.

## Topology changes

Dismemberment/breakable constraints выполняются как physics transaction: validate allowed joint and gameplay command → snapshot → backend mutation → remap body slots/topology revision → contact/render/observation schema update → commit. SPEC-36 additionally distinguishes a stable fracture, an authored unstable break-site retained by a bounded soft-tissue constraint, and complete detachment; runtime arbitrary splitting and presentation-driven topology are forbidden. Policy, не поддерживающая topology mask, немедленно заменяется compatible recovery/passive controller. Удалённые части получают отдельный PersistentId только если становятся durable gameplay objects.

## Failure semantics

- Invalid/stale model или parity mismatch → policy rejected до world activation; heuristic/recovery motor.
- Invalid proficiency/equipment/topology route или unsafe switch → candidate rejected/deferred; previous/novice/recovery route остаётся authoritative согласно SPEC-14.
- NaN/out-of-range action → action rejected, safe controller, structured event.
- Backend fatal/invariant loss → остановка simulation instance и crash capsule; продолжать с недостоверной pose запрещено.
- Missing contact telemetry required текущим gameplay scenario → backend product-check failure, не silent approximation.
- LOD transition failure → остаётся source tier; после 3 failures entity pin-ится в safe tier и emits diagnostic.
- Exhausted manifest-declared integer LOD budget → deterministic downgrade только eligible avatars по canonical order; player/critical physical interaction не теряет required tier.
- Measured CPU overrun → PHYS-P4/PERF-01 failure и diagnostic; он не меняет authoritative LOD, action или outcome текущего run.
- Inference wall-watchdog trip → защитный safe-controller path и `Fail(MOTOR_WALL_DEADLINE_NONCONFORMING)` для exact run; deterministic fallback проверяется только canonical logical fault signal.

## Product checks

Checks используют fixed scenarios/seeds и [RunManifest из SPEC-09](09-tooling-sdk-and-observability.md). Captures и timing reports являются диагностикой продукта и не влияют на authoritative simulation.

The current bounded procedural R5 profile admits exactly one primary solid
capsule and at most one identity-rotation fixed local solid box on the same
kinematic avatar body. The box participates in canonical sweeps, collision
filters, contact continuity and activation penetration checks under its actual
`PhysicsShapeIdV1` and canonical box-face IDs; only the primary capsule may
trigger step-up or establish
ground support. The production reference load uses an opt-in carried-load
layer and a proxy coincident with visible course geometry, so ordinary quest
routes retain their prior collision semantics while authored clearance can
physically stop the complete avatar silhouette. Because both shapes share one
body, Physics remains the sole transform/save owner and the current checkpoint
and Replay V10 formats need no attachment state.

`physical-character` is the bounded `PHYS-P6` procedural closure check. It
repeats the production generation twice, observes a low-riser contact followed
by traversal, proves a carried-shape contact while the capsule keeps positive
clearance, reconstructs that live contact exactly, and verifies that melee
health change remains downstream of committed player/NPC contact. General
grab/drop, multiple loads, mass/effort coupling, ragdoll/get-up, active
articulation and learned control are independent future consumers.

| ID | Сценарий | Ожидаемый результат | Fallback |
|---|---|---|---|
| PHYS-P1 | descriptor parity corpus | 100% bodies/axes/limits; mass/inertia ≤0.1%; torque conversion ≤1% | попробовать следующий backend через тот же contract |
| PHYS-P2 | contact/query/topology suite | 100% required contacts в canonical total order; query/contact output exact under callback, manifold, worker and registration permutations; все key-field vectors проходят `NUMERIC-P1`; 1 000 topology cycles без invalid handle/leak | отвергнуть backend и сохранить последнюю canonical world state |
| PHYS-P3 | same-target replay и cross-target quantized projection | full authoritative `state_root`, applied canonical actions, contact/outcome sequences exact over 100 repeats; Windows/Linux `physics_projection_root`, canonical contact/query order и gameplay outcomes byte-identical | исправить deterministic boundary или выбрать другой backend |
| PHYS-P4 | `r5-physics-16.v3`: 16 independent 23-DoF PhysX humanoids на exact `ref-linux-b550i-3950x-rtx3080-v2` campaign cohort with ADR-093 deterministic physical-core placement | physics 240 Hz, motor 60 Hz, 1/4/8 workers; lockstep 8-worker physics+motor frame p95 ≤4 ms, p99 ≤6 ms; throughput/scaling, restore and resource budgets exactly follow ADR-062/091; roots exact across worker/profiler permutations; no missed critical steps; learned inference budget applies only after evaluator promotion | offline tuning следующего manifest integer LOD budget; safe-tier pin |
| MOTOR-P1 | 10 000 golden observations | ONNX vs training max abs raw action error ≤1e-5; applied safety-clamped/quantized `MotorAction` exact; 0 schema mismatch accepted | reference CPU evaluator или heuristic controller |
| PHYS-P5 | runtime/training golden trajectories | normalized RMSE ≤0.05; contact F1 ≥0.98; outcome pass-rate delta ≤2 percentage points | retrain, mapping fix или backend fallback |
| PHYS-P6 | push, slope, stair, trip, carry, fall/recovery and contact-driven melee | current procedural baseline: `physical-character` plus production capsule-course regressions observe every bounded behavior, exact contact IDs and restart continuation; future broader profiles declare their own thresholds and still require aggregate ≥95% with 0 safety violation | recovery controller; unsupported learned route остаётся отключён |
| PHYS-P7 | every adjacent LOD transition, 1 000 cycles | 0 forbidden transition; jump/velocity limits соблюдены; no lost durable outcome | pin higher safe tier |
| PHYS-P8 | optional local success/failure/comparison capture | fixed camera/profile; overlays command/phase/COM/support/contacts/target; when captured, media reproduces the exact run and contains no authority | сохранить diagnostics; gameplay check остаётся authoritative |

Physical archetype, progressive skill, policy switching и creature authoring проходят дополнительные product checks SPEC-14.
