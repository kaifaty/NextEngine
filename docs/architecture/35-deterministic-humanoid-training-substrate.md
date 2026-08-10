# SPEC-35: Deterministic humanoid training substrate

| Поле | Значение |
|---|---|
| ID | SPEC-35 |
| Статус | Accepted |
| Версия | 1.0 |
| Последняя проверка | 2026-08-10 |
| Нормативные зависимости | [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-057](adr/057-hierarchical-learnable-motor-system-and-policy-family-architecture.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md) |
| Заменяет | отсутствует; принимает первый production consumer для bounded BodySchema/motor-training contracts |

## Назначение и ownership

SPEC-35 задаёт current Stage 0 substrate, на котором можно начинать обучение
humanoid policy, не объявляя policy обученной или shipping-ready.

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
- `MotorTrainingEnvironmentManifestV1`, `MotorEpisodeSeedSetV1`,
  `MotorResetRecordV1`, `MotorStepRecordV1`,
  `MotorTrajectoryManifestV1`;
- aggregate `WorldCheckpointV5`, `SaveManifestV3`, `ReplayManifestV6`.

Старые alpha versions не мигрируются. Readers проверяют outer discriminator,
bounds and closure before nested decode и возвращают stable `UNSUPPORTED_*`,
не изменяя source bytes. Ни один contract не содержит `Px*`, C++ handle,
vendor enum, raw pointer or raw-float authority.

Every project/save/replay/training closure binds exact body schema, instance
projection, observation/action layout, PhysX build/scene profile, bridge ABI,
quantization and reward-coefficient hashes.

## Reference humanoid

`nextengine.body.humanoid-23dof-v1` is engine-owned and immutable:

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
catalog plus `MotorResetRecordV1`. Step is transactional: on invalid action,
NaN, mismatch or capacity overflow it publishes no partial state and emits a
typed terminal reason.

Each CPU vector slot owns one scene and RNG set. Slots advance lockstep at the
same logical substep. Parallel workers return immutable staged results; the
runner publishes them sorted by `(episode_ordinal, vector_slot)`. Worker count,
completion order and slot storage order do not enter seeds or output bytes.

## Checkpoint and replay

`WorldCheckpointV5` atomically contains runtime/RPG state plus
`PhysicsWorldCheckpointV2` and `MotorWorldCheckpointV1`. A motor checkpoint
contains current command/action, all four substep efforts, complete explicit
policy/motor state, cadence phase and every RNG stream state.

Restore creates a fresh PhysX scene, imports bounded root/joint pose and
velocities plus engine-owned contact-continuity state, and resumes only at a
canonical tick boundary. Stored post-safety efforts drive authoritative
replay. Re-evaluating PD/action is an independent parity assertion and cannot
repair recorded history.

Snapshots and records are canonical encoded, capacity-bounded and hashed.
Vendor serialization/caches are reconstructible and absent. Restore
continuation divergence is terminal `RESTORE_DIVERGENCE`; tolerance must not
turn it into success.

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

## Isaac Lab mirror and standing smoke reward

`lab/` MAY provide a pinned Isaac Lab DirectRLEnv. The translator consumes
canonical schema/catalog and writes derived USD outside repository authority.
Translator version and USD hash are recorded. Python/Torch code uses Rust
golden vectors for joint ordering, seed derivation, fixed-point action/safety,
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

Stage 0 runs `fast`, `host-check`, `play`, `persistence-replay`,
`content-package`, `platform`, `performance`, `BODY-SCHEMA-P1`,
`PHYS-JOINT-P1`, `PHYS-SNAPSHOT-P1`, `MOTOR-SCHEDULE`, `MOTOR-SAFETY`,
`MOTOR-STATE`, `MOTOR-ENV-P1`, `MODEL-DATAPLANE` and `MODEL-MIRROR`.

Exact/stability/correspondence thresholds are normative in ADR-058. Stage 0
is complete only after PhysX-only cutover and Windows/Linux gates. Missing GPU
or target run is `NOT_RUN` and blocks the completion claim. Passing this SPEC
permits policy training; it does not prove PPO quality, learned Motor MVP or
R5 completion.
