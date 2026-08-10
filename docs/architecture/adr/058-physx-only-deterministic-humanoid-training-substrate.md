# ADR-058: PhysX-only deterministic humanoid training substrate

| Поле | Значение |
|---|---|
| ID | ADR-058 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-10 |
| Последняя проверка | 2026-08-10 |
| Нормативные зависимости | [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) |
| Заменяет | полностью [ADR-033](033-physx-grounded-capsule-parity-ffi-boundary.md); частично ADR-027 в выборе physics backend и reference fallback |
| Заменён | частично [ADR-059](059-event-sourced-physx-continuation-reconstruction.md) для fresh-scene reconstruction скрытого solver continuation state |

## Контекст

Grounded-capsule reference solver проверил engine-owned physics boundary, но
не способен быть production- и training-plane для reduced-coordinate humanoid.
Сосуществование двух production solvers оставляет неоднозначность в
контактах, reset/restore и replay и требует parity с реализацией, которую
продукт не собирается поставлять.

Первый обучаемый humanoid требует одной canonical CPU среды, настоящих
articulations, fixed actuation и массового GPU mirror. PhysX гарантирует
детерминизм лишь при фиксированных build/platform/API-call условиях, а Isaac
Lab GPU execution зависит от hardware scheduling. Поэтому exact contract
должен находиться на engine-owned canonical boundary, а не в raw vendor float
или скрытом solver cache.

## Решение

### Production backend и bootstrap

- PhysX `5.9.0` становится единственным production physics backend для
  `game`, `headless` и runtime-bearing tools на Windows x86_64 MSVC и Linux
  x86_64 GNU после атомарного cutover.
- `PhysicsBackendPolicy`, reference world/query implementation и production
  backend feature selection удаляются в cutover. Silent activation fallback
  и mid-session backend switch запрещены.
- SDK source и build artifacts не входят в repository. Только явный
  `xtask physx setup` MAY скачать официальный archive, проверить pinned
  SHA-256/license, собрать static CPU libraries и поместить их в OS cache.
  `cargo build`, production startup и replay не обращаются к сети.
- `NEXTENGINE_PHYSX_SDK_DIR` остаётся explicit override. Отсутствующий,
  неподходящий или hash-mismatched SDK завершает composition типизированной
  ошибкой до активации мира.
- Runtime не загружает SDK динамически. Build profile связывает target,
  compiler, PhysX source revision, configuration, bridge ABI и fixed scene
  settings. macOS FFI mock допустим только для wrapper unit/Miri tests и не
  может быть собран production composition root.

### Authority, execution planes и determinism classes

CPU PhysX в Rust runtime является `CanonicalEnvironmentReplay` plane.
Движок владеет canonical IDs, construction order, fixed-point actuation,
contact sorting/bounds, quantization, snapshots, hashes и replay facts.
PhysX владеет только private live solver objects.

Три класса проверки не взаимозаменяемы:

1. `CanonicalEnvironmentReplay` — byte-exact canonical actions,
   observations, snapshots, roots и continuation для одного locked build
   profile, включая worker/slot permutations. Cross-target Windows/Linux
   equality проверяется на canonical engine representation.
2. `EvaluatorCorrespondence` — bounded CPU/GPU trajectory and done/contact
   agreement. Isaac Lab GPU mirror не является replay authority.
3. `StatisticalTrainingOutcome` — multi-seed quality, stability и throughput.
   Он не доказывает exact replay.

Raw PhysX floats, vendor serialization, broadphase/manifold order and hidden
solver caches не сериализуются и не хешируются как authority. Перед branch,
publication или hash raw state/contact records проверяются, сортируются по
semantic IDs, ограничиваются и квантуются engine-owned кодом. Non-finite,
overflow, capacity exhaustion или unsupported feature aborts uncommitted
episode/session.

### Fixed production scene

CPU profile использует static PhysX, TGS solver, enhanced determinism, exact
declared tolerance scale, solver iterations, broadphase/friction settings и
одного solver worker на scene. Параллелизм выполняется только между
независимыми scenes. Bodies, joints, shapes, articulations and constraints
создаются в canonical stable semantic-ID order.

Reset и restore всегда создают fresh scene из exact catalog/checkpoint. Actor
insertion/removal history не является входом. Build/profile/scene/
quantization hashes входят в project/save/replay closure; mismatch отвергается
до nested decode или state mutation.

### Stage 0 humanoid substrate

Первый production-consumed `BodySchemaV1` описывает engine-owned humanoid с
free root, primitive colliders и ровно `23` actuated DoF. Он содержит stable
schema-scoped body/joint/actuator/effector IDs, authored standing pose,
explicit mass/inertia, limits, gains и safety bounds.

Из schema детерминированно выводятся SPEC-26 descriptors, SPEC-27 layouts,
canonical reset, contact effectors, safety limits и Isaac mirror mapping.
Каждый 60 Hz motor frame исполняет четыре 240 Hz substep:

```text
command -> residual joint targets -> fixed-point PD/safety
        -> canonical applied efforts -> PhysX
        -> bounded canonical state/contact projection -> observation
```

PD/safety используют checked saturated arithmetic и ties-to-even. Только
после clamp bounded efforts преобразуются в `f32` на FFI boundary. Procedural
standing controller остаётся обязательным deterministic motor fallback, но
исполняется только через PhysX.

Replay хранит 60 Hz command/action, post-safety effort каждого substep,
complete motor/RNG state и canonical physics snapshots. Recomputed PD является
отдельным parity check. Seeds выводятся content-addressed из
run/episode/vector-slot/purpose; purposes никогда не разделяют mutable stream.

### FFI и persistence

`crates/physics-physx-ffi` остаётся единственной reviewed unsafe boundary. Ее
handle-based C ABI содержит только fixed-width POD descriptors, typed status,
opaque handles и caller-owned bounded buffers. C++ exceptions, STL, callbacks,
allocator pointers и `Px*` types границу не пересекают. Safe Rust adapter
явно владеет lifetime и переводит все failure paths в typed errors.

Authoritative checkpoint содержит root/joint positions and velocities,
articulation state, bounded contact-continuity projection, motor state и RNG,
но не native serialization. По ADR-059 restore реконструирует hidden TGS
continuation через bounded canonical reset + post-safety effort prefix в fresh
scene и проверяет каждый declared compare point до atomic publication.

### Training planes

`headless motor-lab` предоставляет long-lived bounded binary reset/step
protocol через stdin/stdout. CPU vector environments содержат одну scene на
slot, шагают lockstep и публикуют результаты в `(episode_ordinal,
vector_slot)` order независимо от worker completion.

Isaac Lab DirectRLEnv является optional GPU mirror. Derived USD, translator
version/output hash, GPU/driver/Isaac profile и correspondence result входят в
training manifest, но не в gameplay authority. Rust генерирует golden ordering,
seed, fixed-point safety, termination и reward-component vectors. Generated
USD, runs, trajectories, datasets and checkpoints остаются во внешнем
configured store/cache.

## Stage 0 scope и cutover

Stage 0 включает SDK bootstrap, real capsule/static-box regressions,
articulation, accepted schema/layout/checkpoint contracts, procedural standing,
canonical vector environments and Isaac mirror. Он не включает PPO, learned
policy, ONNX inference, animation/retargeting, ragdoll/get-up,
equipment/damage или topology changes.

Cutover выполняется одним commit после Windows/Linux platform, exact replay,
restore, stability and performance gates. До него промежуточная ветка не
объявляет PhysX default; после него reference implementation отсутствует.
Stage 0 разрешает начать policy training, но не закрывает learned Motor MVP,
R5 или текущий WIP=1 R4a.

## Проверка

- SDK/FFI: clean-host setup/doctor, ABI assertions, invalid handles/capacity,
  repeated lifecycle, Linux ASan/UBSan and safe-wrapper mock under Miri.
- `BODY-SCHEMA-P1`: source record permutation yields identical catalog/layout
  hashes; duplicate IDs, cycles, invalid inertia/limits/gains and unsupported
  topology fail closed.
- `PHYS-JOINT-P1` and `PHYS-SNAPSHOT-P1`: capsule, collision, articulation
  effort, limits, contacts, fresh-scene restore and exact continuation use the
  production PhysX path.
- `MOTOR-SCHEDULE`, `MOTOR-SAFETY`, `MOTOR-STATE`, `MOTOR-ENV-P1`: exact
  four-substep cadence, checked PD/clamp, complete state, purpose-isolated
  seeds and lockstep ordered publication.
- 128 seeds × 10,000 CPU substeps are byte-exact on repeat, 1/4/8 workers,
  slot permutations and Windows/Linux canonical representation. Random
  checkpoint restore continues exactly for 10,000 steps.
- At least 1,000,000 steps contain no NaN/overflow; procedural standing
  survives at least 99% of 1,000 flat 60-second episodes.
- `MODEL-MIRROR`: 256 matched ten-second episodes meet joint RMSE ≤0.02 rad,
  root position RMSE ≤0.03 m, velocity RMSE ≤0.05 m/s, contact occupancy ≥98%
  and done-tick agreement ≥95%. 4,096 GPU environments execute at least ten
  million aggregate steps without NaN, leakage or seed collision.
- Required product checks are `fast`, `host-check`, `play`,
  `persistence-replay`, `content-package`, `platform`, `performance` and the
  named substrate checks. Unavailable shipping host/GPU is `NOT_RUN` and
  blocks Stage 0/cutover completion rather than weakening a threshold.

## Последствия и rollback

Production packaging becomes dependent on a prepared, exact PhysX SDK.
Canonicalization and fresh-scene reconstruction add cost but remove a second
solver authority. GPU training remains replaceable because only the derived
mirror consumes vendor USD/Isaac types.

Rollback before cutover retains the last production revision. Rollback after
cutover reverts the whole cutover commit; it does not reintroduce a runtime
fallback policy or mix checkpoint versions.

## Supersession

ADR-033 is Superseded. ADR-027 remains Accepted for ownership/layering, but its
backend-neutral candidate list and reference-fallback wording are superseded
for production physics by this ADR. ADR-057 remains Accepted; its learned
profiles stay Proposed beyond the deterministic Stage 0 substrate. ADR-059
частично supersedes только direct continuation-import assumption этого ADR.
