# SPEC-35: Deterministic humanoid training substrate

| Поле | Значение |
|---|---|
| ID | SPEC-35 |
| Статус | Accepted |
| Версия | 1.7 |
| Последняя проверка | 2026-08-12 |
| Нормативные зависимости | [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](adr/059-event-sourced-physx-continuation-reconstruction.md), [ADR-062](adr/062-r5-physx-humanoid-performance-authority.md), [ADR-063](adr/063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-064](adr/064-canonical-flat-command-locomotion-environment.md), [ADR-065](adr/065-curriculum-flat-command-locomotion-profile.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-067](adr/067-stage0-profile-identity-and-curriculum-hash-closure.md) |
| Заменяет | SPEC-35 1.6; restores exact V1 BodySchema identity and closes curriculum translator/reward semantics |

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
  принимает только first-humanoid reset/step/trajectory/mirror consumer.

Physics остаётся единственным writer pose/contact. Training environment,
reward code, Python mirror and controller never mutate world bypassing the
production descriptor/step path.

## Current public contracts

Current-only alpha contracts в `crates/contracts`:

- `BodySchemaV1`, `BodyInstanceProjectionV1`;
- `PhysicsBodyDescriptorV2`, `PhysicsWorldCatalogV2`,
  `PhysicsJointDescriptorV1`, `PhysicsActuatorDescriptorV1`;
- `PhysicsStepInputV3`, `PhysicsStepResultV2`,
  `PhysicsCanonicalSnapshotV3`, `PhysicsWorldCheckpointV2`;
- `MotorObservationLayoutV1`, `MotorActionLayoutV1`,
  `MotorWorldCheckpointV1`, `PolicyStateRecordV1`;
- `MotorTrainingEnvironmentManifestV2`, `MotorEpisodeSeedSetV1`,
  `MotorLocomotionCommandProfileV1`, `MotorResetRecordV2`,
  `MotorStepRecordV2`, `MotorTrajectoryManifestV2` and bounded
  `MotorEnvironmentCheckpointEnvelopeV1`;
- aggregate `WorldCheckpointV5`, `SaveManifestV3`, `ReplayManifestV6`.

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

## Checkpoint and replay

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
historical translator-profile-v2 hash, while curriculum V2 binds
`nextengine.isaac-usda-translator.v3`. Python/Torch code uses Rust golden
vectors for joint ordering, seed derivation, fixed-point action/safety,
termination facts and reward component IDs.

Stage 0 reward is an ordered vector, not one implicit scalar:

1. upright;
2. root-height tracking;
3. standing-pose tracking;
4. linear/angular velocity penalty;
5. effort penalty;
6. action-rate penalty;
7. foot-slip penalty;
8. fall terminal component.

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

`r5-physics-16.v1` is the current performance consumer for this substrate. It
runs 16 independent reference humanoids, 240 warm-up plus 10,000 measured
physics substeps per slot and the same standing action stream under 1/4/8
workers. Exact canonical root parity across those modes and the profiler
control is mandatory. It reports direct physics/motor throughput, reciprocal
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

Exact/stability/correspondence thresholds are normative in ADR-058. Stage 0
is complete only after PhysX-only cutover and Windows/Linux gates. Missing GPU
or target run is `NOT_RUN` and blocks the completion claim. Passing this SPEC
permits policy training; it does not prove PPO quality, learned Motor MVP or
R5 completion.
