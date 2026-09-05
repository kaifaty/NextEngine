# SPEC-35: Deterministic humanoid training substrate

| Поле | Значение |
|---|---|
| ID | SPEC-35 |
| Статус | Accepted |
| Версия | 3.12 |
| Последняя проверка | 2026-09-05 |
| Нормативные зависимости | [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](adr/059-event-sourced-physx-continuation-reconstruction.md), [ADR-062](adr/062-r5-physx-humanoid-performance-authority.md), [ADR-063](adr/063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-064](adr/064-canonical-flat-command-locomotion-environment.md), [ADR-065](adr/065-curriculum-flat-command-locomotion-profile.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-067](adr/067-stage0-profile-identity-and-curriculum-hash-closure.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md), [ADR-090](adr/090-linux-only-v1-and-indefinitely-deferred-windows.md), [ADR-100](adr/100-bounded-standing-reward-profile.md), [ADR-101](adr/101-biomechanics-command-only-standing-environment.md), [ADR-102](adr/102-biomechanics-neutral-self-clearance-successor.md), [ADR-103](adr/103-r8b-rd-only-walking-discriminator.md), [ADR-104](adr/104-r8b-discriminating-walking-objective.md), [ADR-105](adr/105-r8b-dense-tracking-walking-counterfactual.md) |
| Заменяет | SPEC-35 3.11; adds opt-in articulated-foot BodySchema V8 diagnostic |
| Дополнительные зависимости V3.12 | [ADR-118](adr/118-articulated-volumetric-foot-body.md) |
| Дополнительные зависимости V3.9 | [ADR-115](adr/115-full-principal-inertia-body-successor.md) |
| Дополнительные зависимости V3.10 | [ADR-116](adr/116-explicit-per-iteration-force-scheduling.md) |
| Дополнительные зависимости V3.11 | [ADR-117](adr/117-quiet-upright-body-and-standing-reference.md) |
| Дополнительные зависимости V3.8 | [ADR-114](adr/114-anatomical-axes-and-sagittal-body-proxies.md) |
| Дополнительные зависимости V3.7 | [ADR-113](adr/113-explicit-known-walking-candidate-reuse.md) |
| Дополнительные зависимости V3.6 | [ADR-112](adr/112-prospective-validated-walking-training.md) |
| Дополнительные зависимости V3.5 | [ADR-110](adr/110-applied-command-stop-window.md), [ADR-111](adr/111-final-weight-corrected-walking-evaluation.md) |
| Дополнительные зависимости V3.3 | [ADR-109](adr/109-observable-sole-lift-and-return.md) |
| Дополнительные зависимости V3.2 | [ADR-107](adr/107-canonical-cpu-walking-learner.md), [ADR-108](adr/108-observable-periodic-walking-credit.md) |
| Дополнительные зависимости V3.1 | [ADR-106](adr/106-walking-reference-and-leg-clearance-audit.md) |
| Дополнительные зависимости V1.9 | [ADR-069](adr/069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-070](adr/070-biomechanics-reference-tracking-training-environment.md), [ADR-071](adr/071-canonical-physics-material-lineage.md) |

## Назначение и ownership

SPEC-35 задаёт current Stage 0 substrate, на котором можно начинать обучение
humanoid policy, не объявляя policy обученной или shipping-ready.

ADR-066 does not widen this current consumer: Stage 0 remains one fixed
23-DoF humanoid with numeric flat command, fixed PD/safety and no natural-
language, contact-chunk, graph-policy, hypernetwork or candidate-rollout input.
Those are independently Proposed later profiles.

- SPEC-14 владеет semantic `BodySchema`/instance projection и family/skill
  meaning.
- SPEC-26 владеет body/joint/actuator physics descriptors, scene profile,
  contact projection and canonical snapshots.
- SPEC-27 владеет exact observation/action layouts, fixed PD/safety,
  `PolicyStateRecordV1` and applied-action replay.
- SPEC-34 остаётся Proposed general multi-lane training lifecycle. Этот SPEC
  принимает только first-humanoid reset/step/trajectory/mirror consumers,
  включая узкий ADR-070 reference tracker.

Physics остаётся единственным writer pose/contact. Training environment,
reward code, Python mirror and controller never mutate world bypassing the
production descriptor/step path.

## Current public contracts

Current-only alpha contracts в `crates/contracts`:

- `BodySchemaV1`, `BodyInstanceProjectionV1`;
- `PhysicsBodyDescriptorV2`, `PhysicsWorldCatalogV2`,
  `PhysicsJointDescriptorV1`, `PhysicsActuatorDescriptorV1`,
  `PhysicsMaterialDescriptorV2`, `PhysicsMaterialCombineProfileV1`;
- `PhysicsStepInputV3`, `PhysicsStepResultV2`,
  `PhysicsCanonicalSnapshotV3`, `PhysicsWorldCheckpointV2`;
- `MotorObservationLayoutV1`, `MotorActionLayoutV1`,
  `MotorWorldCheckpointV1`, `PolicyStateRecordV1`;
- `MotorTrainingEnvironmentManifestV2`,
  `MotorReferenceTrackingProfileV1`, `MotorTrainingEnvironmentManifestV3`,
  `MotorEpisodeSeedSetV1`,
  `MotorLocomotionCommandProfileV1`, `MotorResetRecordV2`,
  `MotorStepRecordV2`, `MotorTrajectoryManifestV2` and bounded
  `MotorEnvironmentCheckpointEnvelopeV1`;
- aggregate `WorldCheckpointV5`, `SaveManifestV3`, current `ReplayManifestV9`.

Старые alpha versions не мигрируются. Readers проверяют outer discriminator,
bounds and closure before nested decode и возвращают stable `UNSUPPORTED_*`,
не изменяя source bytes. Ни один contract не содержит `Px*`, C++ handle,
vendor enum, raw pointer or raw-float authority.

Every project/save/replay/training closure binds exact body schema, instance
projection, observation/action layout, PhysX build/scene profile, bridge ABI,
quantization, command schedule, reward, termination, RNG derivation and
correspondence hashes. Protocol v2 rejects a V1 environment manifest with a
typed `UNSUPPORTED_*` result before creating mutable environment state.

## Reference humanoid

`nextengine.body.humanoid-stage0.v1`, schema revision `1`, is engine-owned and
immutable. Its authored pelvis root remains exactly `1.050 m`; correcting that
pose, anatomy or ground clearance requires a distinct BodySchema identity and
environment/training generation rather than a revision/hash drift of V1.

The fixed body has:

- free six-axis root; primitive capsule/box/sphere colliders only;
- 23 actuated revolute DoF with stable body/joint/actuator IDs;
- stable feet/hands effectors and declared symmetry pairs;
- explicit SI mass, center of mass, positive inertia, joint limits, maximum
  effort, PD gains, velocity bounds and authored standing state;
- no equipment, damage, attachment or topology mutation in Stage 0.

The compiler canonicalizes source records by semantic ID, validates closure
and derives one `PhysicsWorldCatalogV2`, motor layouts, reset state, safety
table and Isaac mapping. Source order MUST NOT change any derived bytes/hash.

Under ADR-069, the separate biomechanics generation uses `BodySchemaV2` and
the `BODY-SCHEMA-P2`/`PHYS-JOINT-P2` successor corpus. Its full source inertia,
solver projection, carriers, collider/contact roles, hard/soft ROM and complete
actuator safety envelope are canonical inputs. The V2 compiler does not fall
back to any V1 sphere, X-axis, gain, root-height or filter constant. This
addition neither changes the Stage 0 V1 bytes above nor permits a V2 checkpoint
to resume or import a V1 run.

ADR-070 makes that exact V2 generation consumable by one training-only
reference tracker. The profile binds the admitted locomotion corpus, split,
uniform clip/phase selection, reset mixture, 435-channel actor/critic layout,
four-sample reference horizon, 23-channel residual action, reward, terminal
and RNG semantics. `MotorTrainingEnvironmentManifestV3` closes the profile and
input-provenance roots while reusing V2 reset/step/trajectory/checkpoint
records. Recovery clips, command selection, learned runtime execution and
`PhysicalActionChunk` remain outside this current consumer.

ADR-071 adds `CompiledBodySchemaV3` and biomechanics mirror V2 as the only
current biomechanics lineage that claims complete material identity. They
hash-bind three exact material rows, one combine profile, the ground material
ID and native shared-material projection. The V2 compiler/mirror bytes remain
immutable historical inputs and cannot be interpreted as carrying those
facts. Derived USD/Isaac correspondence and any resumed dynamics work require
their own successor identities and gates.

`BodyInstanceProjectionV1` binds the exact schema and zero/default morphology,
equipment, stats, damage, fatigue and attachment revisions. Those fields
remain owned by their source domains; the projection is immutable input, not
a second authority.

## Deterministic control schedule

One motor tick is exactly four physics substeps:

1. accept one bounded 60 Hz command and residual position action;
2. for each 240 Hz substep, build observed joint state;
3. evaluate fixed-point PD and safety using checked saturated integer
   arithmetic with ties-to-even;
4. persist post-safety applied effort, convert only it to bounded `f32`, and
   simulate the locked PhysX scene;
5. export bounded root/joint/contact records, canonicalize, quantize and update
   the next observation.

The procedural standing controller uses the same action layout, PD, safety and
PhysX path. It cannot write joint pose directly.

`PolicyStateRecordV1` contains explicit adaptation, motion-generator and
expert-router segments, even when empty. Hidden evaluator cache, implicit
history, optimizer and mutable weights are invalid. Stage 0 does not execute a
learned evaluator.

## Reset, step, seeds and vector execution

`MotorEpisodeSeedSetV1` derives independent streams from
`H(run_root, episode_ordinal, vector_slot, purpose_id)`. Purpose IDs are
stable semantic values; adding a randomization source cannot consume another
source's stream.

Reset destroys the previous scene and constructs a fresh scene from exact
catalog plus `MotorResetRecordV2`. Step is transactional: on invalid action,
NaN, mismatch or capacity overflow it publishes no partial state and emits a
typed terminal reason.

Each CPU vector slot owns one scene, independently increasing episode ordinal
and RNG set. Partial reset replaces only selected slots; a terminal slot cannot
step until reset. The canonical locomotion Step input is exactly
`(vector_slot, episode_ordinal, action)` and the engine derives its command.
Duplicate/missing slots, stale episode and incomplete batches reject before
mutation. Slots advance lockstep at the
same logical substep. Parallel workers return immutable staged results; the
runner publishes them sorted by `(episode_ordinal, vector_slot)`. Worker count,
completion order and slot storage order do not enter seeds or output bytes.
Slot-indexed input vectors are preallocated; no per-step ordered map is built.

`nextengine.motor.env.humanoid-flat-command.v1` adds the ADR-064 pure
SHA-256-counter command schedule, 60-tick warm-up, 120-tick target segments and
1,200-tick timeout. Its flat static box has 100 metre X/Z half-extents and its
84-channel observation uses root-local linear/angular velocities and
`[local-right, local-forward, yaw-rate]` command order. Standing keeps its
existing world-frame layout and behavior.

`nextengine.motor.env.humanoid-flat-command-curriculum.v2` reuses that exact
body/layout/action/scene/termination closure but selects one of three
engine-owned SHA-256-counter command stages from episode ordinal `0`, `32` or
`96`. ADR-065 owns the narrower ranges, acceleration, reward shaping and
command-conditioned support component. V1 remains byte-for-byte unchanged.

`nextengine.motor.env.humanoid-standing.v2` also reuses the frozen Stage 0 V1
body, world-frame observation, action, controller, zero command, reset, scene,
standing termination and 3,600-tick timeout. ADR-100 changes only its distinct
reward and translator closure: eight commensurate components are individually
bounded to `[0, 65,536]` Q16 and the weighted per-step total is bounded to
`[-148,768, 114,688]` Q16. Standing V1 remains byte-for-byte historical input
and is not an optimizer profile for new R8b runs.

`nextengine.motor.env.humanoid-biomechanics-standing.v2` is the current R8b
optimizer candidate under ADR-102. It uses the exact biomechanics BodySchema
V3 through material-complete `CompiledBodySchemaV3`, an 84-channel world-frame
observation and a 23-channel normalized residual around the deterministic
procedural-standing fallback. It consumes no motion corpus or reference
tracker. Its reward ports the bounded ADR-100 objective to procedural target,
soft-ROM and actuator normalizers derived from the current descriptor; its
reset has exact zero sole clearance and its 3,600-tick termination adds the
current tilt, world, hard-ROM, hard-impact and self-collision safety facts. V3
preserves V2 except for widening the shoulder-root half-width from `170,000`
to `215,000 um`, giving each neutral pelvis/forearm pair `45,405 um` AABB
clearance, and replaces the `0.001 kg / 0.000001 kg*m^2` serial carriers with
an exactly source-conserving `0.25 kg / 0.001 kg*m^2` solver projection. The
former V2/V1 standing identity remains immutable negative evidence and cannot
initialize this successor.

`nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v1` is the sole
ADR-103 `R&D_ONLY` successor experiment. It retains BodySchema V3, procedural-
standing residual control and complete safety/contact termination, but uses
root-local velocities, the fixed ADR-065 foundation command stage and its
bounded eleven-component reward for 1,200 ticks. The admitted standing
checkpoint may initialize weights and observation normalizers only; optimizer,
counters, seed and run root are fresh. Failed `MODEL-MIRROR-P1` still blocks
runtime promotion, broader commands and any second walking budget.

`nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v2` is the sole
ADR-104 successor discriminator. It keeps the V1 body, controller,
observation/action and safety boundaries but replaces the permissive objective
with compact `0.5 m/s` command tracking, posture costs and the existing
one-sole support signal. Its fixed lesson commands rate-limited `0.5 m/s`
forward travel after the 120-tick warm-up, begins ramp-down at tick 991 and
holds exact zero for ticks `1021..1200`. V1 remains immutable negative evidence;
V2 is still `R&D_ONLY` and grants no authority while `MODEL-MIRROR-P1` fails.

V2 is also immutable negative evidence: its final GPU policy survives but
travels only `0.123/7.258 m`, and no canonical CPU checkpoint walks. Under
ADR-105, `nextengine.motor.env.humanoid-biomechanics-forward-start-stop.v3`
changes only compact planar/yaw tracking to the exact dense bounded Q16 kernel
`square(1 / (1 + (error / 0.5)^2))`; schedule, coefficients, body, controller,
PPO and admission gates remain V2-identical.

[ADR-106](adr/106-walking-reference-and-leg-clearance-audit.md) adds walking V4
as a translation-invariant reference diagnostic on BodySchema V3. Walking V5
adds BodySchema V4 leg collision proxies and a bounded `4x` residual multiplier
before the unchanged safety intersection. All old identities remain exact.
V5 preserves source dynamics and anatomy; only bilateral thigh/shank/knee
collision geometry changes. The Proposed V5 training recipe has fresh weights.
Its bilateral 105-tick CPU reachability probe passes, but Isaac joint-safety
and trajectory differences keep paired alternation/correspondence and training
blocked. `WALKING-ACTION-REACHABILITY-P0` is not MODEL-MIRROR-P1 or learned gait
admission. Terminal recording must retain the pre-reset committed facts.

[ADR-107](adr/107-canonical-cpu-walking-learner.md) supersedes the preceding
optimizer sequencing for one direct canonical CPU V5 experiment only. CUDA
computes the policy/optimizer, not physics. A CPU-specific frozen generation
closes the clean commit, native executable, descriptor, profile, dependency
versions, slot partition and sole run path; no USD/corpus is an input. Exact
native adapter control and final-observation timeout bootstrap precede its
1,024,000-sample budget. The final five-episode matrix requires real alternating
support, >=3 m travel and 180 stopped ticks. Isaac correspondence and all
runtime promotion gates remain unpassed; this is not another Isaac recipe.

[ADR-108](adr/108-observable-periodic-walking-credit.md) admits a diagnosed
canonical-only V6 successor: unchanged V5 plant/actions/safety, 86 observations
(84 existing plus two engine-owned periodic clocks), and graded periodic
load-transfer credit in place of binary support. Its single fresh 4,096,000-
sample run retains the original five-episode gait/travel/stop/safety gates.
It adds no reference trajectory or Isaac implementation. Old V5 observations,
rewards and descriptors are frozen; this is a distinct hashed environment.

[ADR-109](adr/109-observable-sole-lift-and-return.md) adds the diagnosed V7
successor: 88 observations (two actual sole heights appended), two bounded
per-foot phase-height costs and the unchanged V6 load credit. A separately
closed 40,960,000-transition run has predeclared report-only milestones and
the original final walking matrix. Body/action/safety and all older environment
identities remain frozen; no Isaac or runtime admission follows from this.

[ADR-110](adr/110-applied-command-stop-window.md) adds V8 as a no-optimizer
schedule repair: ramp-down starts at action index 990, giving exact zero over
all 180 final applied indices 1020..1199. V7 retains the defective historical
vector with only 179 applied zeros. Body, action, observation, reward formula,
phase and safety remain V7-identical. [ADR-111](adr/111-final-weight-corrected-walking-evaluation.md)
separately admits final V7 model 9999 inference on V8 after exact closure and
compatibility checks. Its new matrix requires exclusive classified vertical
load on one foot in all four actual substeps and positive canonical post-step
whole-box height of the other. Retain all original duration, travel, tracking,
stop and safety requirements plus separate old raw-presence results. No new
optimizer, checkpoint selection, mirror or runtime admission follows.

[ADR-112](adr/112-prospective-validated-walking-training.md) separately admits
one fresh V8 canonical run with fixed 1e-5 PPO learning rate and prospective
100-update native validation. Select the first full five-episode pass and stop;
otherwise the budget ends in failed quality. ADR-111 physical gates, all old
results, and the no-robustness/no-runtime boundary remain unchanged.

[ADR-113](adr/113-explicit-known-walking-candidate-reuse.md) separately admits
closed-loop V8 evaluation of the explicitly known-result-selected model 3999.
Keep the same complete physical matrix and exact native replay. Its nominal
artifact claim does not relabel old failed final-only results or establish
held-out statistics, robustness or runtime promotion.

[ADR-114](adr/114-anatomical-axes-and-sagittal-body-proxies.md) adds opt-in
BodySchema V5 with corrected bilateral hip/shoulder/elbow flexion and hip
adduction axes, plus source-guided pelvis/torso sagittal collider centres.
Link frames, mass/COM/inertia, feet and safety remain unchanged. All existing
environments retain their old bodies; V5 is a native geometry/kinematic
diagnostic and does not inherit checkpoints or establish learned quality.

[ADR-115](adr/115-full-principal-inertia-body-successor.md) adds opt-in BodySchema
V6, preserving V5 anatomy and masses but representing all six non-diagonal
inertias with principal moments and mass-frame rotations. Integer and compiled
float32 tensor reconstruction must stay within 1 micro kg m² per component.
Old body/environment bytes remain unchanged. Nominal procedural standing on
V6 is a diagnostic, not checkpoint compatibility or learned-quality admission.

## Checkpoint and replay

[ADR-116](adr/116-explicit-per-iteration-force-scheduling.md) adds the opt-in
`CompiledBodySchemaV4` force-schedule successor for native diagnostics. It keeps
the same body, materials, gains and safety but hash-binds per-iteration external
forces. No existing environment selects it. New consumers must bind the outer
compiled identity and reconstruct that same scene for reset/effort replay;
legacy weights and mirror admission do not transfer automatically.

[ADR-117](adr/117-quiet-upright-body-and-standing-reference.md) adds reusable
opt-in BodySchema V7 and procedural standing reference V2. V7 preserves V6
anatomy/mass/inertia and changes only the declared shoulder-yaw gains and
eight coupled damping coefficients. The reference accepts a complete V7 /
compiled-V4 input, keeps V1 knee/ankle behavior and adds the frozen hip
position correction without direct hip rate feedback. Its state root binds
the compiled identity, subject PersistentId and reset anchoring. Old references/environments remain
unchanged. Full native correspondence to the quiet nominal candidate is
required; this does not admit perturbation robustness, walking, old weights,
training or runtime deployment. Foot mechanics remains separate work.

[ADR-118](adr/118-articulated-volumetric-foot-body.md) adds opt-in BodySchema V8:
two MTP hinges, source calcaneus mass/inertia and finite-volume toe proxies,
26 bodies/25 actuators/21 colliders. Total body mass and non-foot anatomy remain.
Source toe COM/mass are preserved while impossible/planar inertia is replaced
by an explicit projection and new ankle-group moments. Native kinematics and
four-sole material closure do not admit dynamic standing. Existing controller
V2 rejects this body; a successor must preserve6 Ns anatomical-foot aggregate
safety, validate loaded support/re-contact and disturbances, and carry separate
training compatibility. Existing body and environment hashes are unchanged.

`WorldCheckpointV5` atomically contains runtime/RPG state plus the final
`PhysicsWorldCheckpointV2` and `MotorWorldCheckpointV1` witness. The same
`SaveManifestV3` generation MUST also contain one Motor-owned replay-prefix
segment: canonical reset origin, ordered post-safety effort frames, one exact
physics witness hash per motor tick and the final full witness. Replay V6 binds
the equivalent reset/step/trajectory record chain. Missing either part makes
the closure non-restorable.

The current prefix is bounded to 3,600 motor ticks (60 seconds), exactly four
substeps per tick and the declared actuator count per substep. For the 23-DoF
humanoid, fixed-width efforts consume about 2.65 MiB at the bound; the
standalone motor checkpoint envelope is capped at 4 MiB. CPU vector memory and
restore time are accounted per slot and reported by the performance gate.

Restore creates a fresh PhysX scene from the exact catalog, establishes the
canonical reset origin and applies recorded post-safety efforts directly. It
checks the engine-owned canonical physics witness after every motor frame and
the complete final canonical snapshot before atomic publication. Only then
does it restore explicit controller/motor/RNG state at a canonical tick
boundary. It never evaluates PD or a policy to advance authoritative replay.

Re-evaluating PD/action is an independent parity assertion and cannot repair
recorded history. Vendor serialization/caches remain reconstructible and
absent. Prefix bounds/order/substep/channel mismatch or any continuation
divergence is terminal `RESTORE_DIVERGENCE`; tolerance cannot turn it into
success.

## Headless motor lab

`headless motor-lab` is a long-lived process with a versioned bounded binary
stdin/stdout protocol:

- handshake declares protocol, schema/layout/profile hashes and capacities;
- `Create`, `Reset`, `Step`, `Checkpoint`, `Restore` and `Close` messages carry
  fixed-width canonical records with explicit length and request ID;
- exactly one response is emitted per accepted request; diagnostics go to
  stderr and never corrupt the binary stream;
- malformed version/length/hash/request order fails before mutation;
- per-step process spawning, JSON hot paths and PyO3 are excluded.

The CPU runner is canonical replay/correspondence authority. External clients
cannot submit raw PhysX descriptors or mutate a live body directly.

## Isaac Lab mirror and canonical rewards

`lab/` MAY provide a pinned Isaac Lab DirectRLEnv. The translator consumes
canonical schema/catalog and writes derived USD outside repository authority.
Translator version and USD hash are recorded. Each environment manifest binds
the exact translator semantics it admits: standing/flat-command V1 keep their
historical translator-profile-v2 hash, while bounded standing V2 and
curriculum V2 bind `nextengine.isaac-usda-translator.v3`. Python/Torch code
uses Rust golden vectors for joint ordering, seed derivation, fixed-point
action/safety, termination facts and reward component IDs.

The biomechanics command-only standing descriptor wraps the unchanged
ADR-071 biomechanics mirror V2 and binds its own full descriptor-content hash.
Its derived humanoid and ground USD bytes remain material-complete projections,
not a second source body. GPU execution requires the exact sibling translation
manifest and ground USD before environment creation.

Stage 0 reward is an ordered vector, not one implicit scalar:

1. upright;
2. root-height tracking;
3. standing-pose tracking;
4. linear/angular velocity penalty;
5. effort penalty;
6. action-rate penalty;
7. foot-slip penalty;
8. fall terminal component.

That list is the immutable standing V1 historical vector. ADR-100 defines the
current standing V2 optimizer vector as yaw-invariant upright, normalized
height and pose tracking, normalized root-motion, applied-effort,
post-clamp-action-rate and contacting-foot-slip costs, plus the unchanged
standing fall fact. Every component and the final weighted sum use exact
bounded Q16 arithmetic on both CPU and Isaac.

ADR-101 preserves those coefficients and bounds for the biomechanics standing
profile, but replaces neutral-pose tracking and Stage 0 fixed denominators with
the procedural-standing reference and descriptor-derived biomechanics soft-ROM,
effort and target-rate normalizers.

Coefficient order/value/units are hash-bound in the environment manifest.
Rewards and done facts are training observations, never gameplay authority.
GPU execution is evaluated by correspondence and statistics, not byte-exact
replay.

The flat-command profile instead uses the ten bounded Q16 components and
coefficients accepted by ADR-064: planar/yaw tracking, yaw-invariant upright,
height, vertical/roll-pitch velocity costs, normalized applied effort,
post-clamp action rate, declared-foot slip and fall. It contains no
standing-pose reward. Pelvis height `<=0.45 m` or X/Z bounds `>=90 m` terminate;
tick 1,200 truncates. `terminated` and `truncated` remain distinct facts.

The curriculum V2 profile uses the eleven ADR-065 components: squared planar
and yaw tracking, the same production posture/effort/action facts, stricter
normalizations, command-conditioned biped support and a `-10` fall coefficient.
All values remain Q16/hash-bound; the support component reads declared contact
facts and cannot create or override contact.

The biomechanics forward/start-stop V2 profile uses a distinct eleven-field
vector: compact planar/yaw tracking, root-tilt and height-error costs, the same
bounded velocity/effort/applied-target/slip facts, command-conditioned support
and fall. The steady `0.5 m/s` stationary control receives only the yaw term,
exactly 10% of the ideal upright one-sole moving total.

The V3 vector retains the same order and coefficients but gives its first two
components distinct `*-dense` identities. At `0.5 m/s` commanded error its
planar component is `16384/65536`, is strictly larger at `0.25 m/s` error and
reaches `65536/65536` only at zero error. This preserves a local learning
signal while keeping direct distance and stop gates authoritative.

## Failure semantics

Fail closed before partial publication on unsupported shape/joint/profile,
duplicate/missing semantic ID, invalid mass/inertia/limit/gain, ABI/SDK/build
mismatch, non-finite conversion, capacity overflow, seed collision, malformed
protocol, stale request or restore divergence. An episode terminal record
contains the stable reason and last committed roots.

Missing PhysX cannot select a reference solver. Before world activation it is
a typed configuration failure. During a live session any native failure aborts
the uncommitted step/session and retains only the last committed checkpoint.

## Product checks and completion

`r5-physics-16.v3` is the current performance consumer for this substrate. It
runs 16 independent reference humanoids, 240 warm-up plus 10,000 measured
physics substeps per slot and the same standing action stream under 1/4/8
workers. Under [ADR-093](adr/093-deterministic-r5-worker-placement.md) every
worker pins itself to a deterministic physical core drawn from a round-robin
cache-domain interleave before warm-up, through the one reviewed
`next_cpu_affinity` boundary; placement failures fail closed before timing and
authoritative roots remain byte-exact across worker counts. Exact canonical
root parity across those modes and the profiler control is mandatory. It reports direct physics/motor throughput, reciprocal
cost, lockstep p95/p99, scaling, checkpoint size, fresh-scene restore,
replay-prefix overhead and host/logical memory. ADR-062 owns the exact THOTH
budgets and separates restore latency from the live 4/6 ms PHYS-P4 deadline.
ADR-063 makes one hard gate a fixed batch of three complete R5 workload runs;
absolute budgets use the worst per-run tail and relative evidence bootstraps
whole run-p95 observations against ten calibration runs.

Stage 0 runs `fast`, `host-check`, `play`, `persistence-replay`,
`content-package`, `platform`, `performance`, `BODY-SCHEMA-P1`,
`PHYS-JOINT-P1`, `PHYS-SNAPSHOT-P1`, `MOTOR-SCHEDULE`, `MOTOR-SAFETY`,
`MOTOR-STATE`, `MOTOR-ENV-P1`, `MOTOR-LOCOMOTION-ENV-P1`, `MODEL-DATAPLANE`
and `MODEL-MIRROR`.

Exact/stability/correspondence thresholds are normative in ADR-058. Current
development uses Linux focused/platform/report-only evidence. Stage 0 learned
default-readiness remains optional and outside the procedural v1 baseline; a
future Linux promotion must complete its applicable PhysX, hard-performance
and correspondence matrix. Windows/cross-target evidence is outside current
scope under ADR-090 and can return only through a separate future product
decision. Passing this SPEC permits policy training; it does not prove PPO
quality, learned Motor MVP, R5 completion or v1 release readiness.
