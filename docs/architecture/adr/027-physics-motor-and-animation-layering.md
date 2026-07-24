# ADR-027: Physics, motor and animation authority layering

| Поле | Значение |
|---|---|
| ID | ADR-027 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Physical Embodiment Team |
| Требуемые согласующие | Repository Owner, Architecture Working Group, Runtime Team, RPG Framework Team, Physical Embodiment Team, Rendering Team, Verification & Evidence Team |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-013](013-self-contained-physical-avatar-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR-027 принят как upstream authority decision consolidated architecture packet
1.8. Downstream physics-world, motor-inference and skeletal-animation
specifications MUST depend on this ADR; ADR-027 не зависит от них и не
переопределяет их versioned schemas или gate descriptors.

Принятие ADR фиксирует только архитектурное распределение authority. Оно не
создаёт runtime implementation, не выбирает physics или inference backend, не
создаёт verification evidence и не объявляет ни один PHYS, MOTOR, ANIM,
`vertical-v1` или certification gate пройденным.

## Контекст

Physics, motor control и skeletal animation обрабатывают близкие числовые
данные, но не могут совместно владеть одной mutable pose. Если animation root
motion, motor output и physics solver способны независимо записывать body
transform, то contact, traversal, hit и replay result зависят от порядка
callbacks. Если presentation IK может возвращаться в simulation, camera,
renderer frame rate и GPU execution становятся скрытыми gameplay inputs.

Предобученная policy также не является источником физического факта. Она
предлагает ограниченное actuation, а не доказывает контакт или успех
действия. Напротив, physics contact сам по себе не изменяет health, quest или
inventory: такой gameplay outcome проходит общий validated `WorldCommand`
commit path.

Нужен один upstream decision, по которому дальнейшие public schemas разделяют
numeric authority, proposal authority и presentation authority без vendor
types или скрытого first-party пути.

## Решение

### Единственный owner каждого слоя

| State / fact | Единственный authority | Разрешённый вход | Запрещённый второй owner |
|---|---|---|---|
| RPG, quest, inventory, damage, stamina и durable gameplay outcome | Owning RPG/Mechanics context через validated `WorldCommand` transaction | Engine-validated command/proposal | Physics callback, motor model, animation event или renderer |
| Active physical pose, linear/angular velocity, contacts, constraints, topology и physics activation state | Physical Embodiment physics world | Validated descriptor/topology transaction, bounded force/impulse/kinematic request и accepted motor actuation | Animation graph, motor evaluator, renderer, AI, script, package или backend-native public handle |
| Active policy route, bounded recurrent state и candidate `MotorAction` | Motor Runtime / `PolicySupervisor` | `PhysicalAvatarIntent`, canonical `MotorObservation`, exact route/profile hashes | Physics pose store, animation graph, Agent plan или mutable RPG state |
| Reference motion, animation state-machine cursor, presentation-only secondary motion и visual local pose | Animation Runtime | Immutable authored animation content, intent and committed physical snapshots | Gameplay state, active physical pose, contact or collision transform |
| `RenderPose`, interpolation, visual IK, camera-facing and GPU deformation | Presentation / Rendering | Immutable committed physics or animation projection according to physical LOD | Any authoritative simulation field |

Physics authority означает authority над physical numerics, а не над foreign
domain semantics. Только validated outcome transaction может превратить
physical observation в durable gameplay fact.

Backend-side body, island, manifold, solver, animation-job or GPU copies
являются reconstructible private caches. Они не становятся вторым source of
truth и не входят в public contracts, saves, replays, scripts, WIT или AI IPC.

### Canonical tick flow

Каждый composition root, исполняющий simulation, использует один logical flow:

1. common runtime закрывает Ingress batch и коммитит validated intent/config
   changes;
2. physics world публикует immutable canonical observation текущей declared
   physics boundary;
3. observation builder формирует versioned `MotorObservation` только из
   разрешённого physics snapshot и immutable gameplay context;
4. active deterministic policy route создаёт bounded `MotorAction` proposal;
5. safety/actuation validator проверяет schema/profile/body/topology hashes,
   logical age, units, finite source values, quantized bounds, joint mask,
   rate, torque/force, energy and contact guards;
6. physics world принимает только validated canonical actuation на следующем
   declared substep и остаётся единственным writer physical state;
7. adapter публикует canonical physics snapshot, contact/query records and
   physical outcome proposals;
8. common Outcome batch validator решает gameplay mutation; physics callback
   или animation marker не коммитит её напрямую;
9. animation bridge читает committed intent/physics state, обновляет
   reference/presentation state and publishes immutable `RenderPose`;
10. renderer интерполирует presentation snapshots без обратной записи.

Stage order, tick rates and profile hashes являются authoritative inputs.
Worker completion, wall time, measured load, renderer frame, GPU completion,
backend callback order and animation task order не выбирают action, pose,
contact, outcome или transition.

Mutable physics/ECS access через `await` или между fixed boundaries запрещён.
Async model/content preparation возвращается immutable staged result и
становится доступной только на declared deterministic commit point.

### Physics-authoritative numerics

- Active physical body transform, velocity, support/contact and constraint
  result доказываются только committed physics state.
- Raw backend float является private operational sample. До branch, ordering,
  query/contact publication, outcome proposal or hash он проходит exact
  conversion через `PhysicsQuantizationProfileV1` and checked numeric contract
  SPEC-21/ADR-022.
- IDs, topology, enum classes, tick/substep, order, quantized integer values,
  contact/query classes, snapshot bytes/root and gameplay outcomes are exact.
  Tolerance допустима только для отдельно declared non-authoritative raw
  correspondence metric и никогда не выбирает outcome или `PASS` exact
  mismatch.
- Non-finite input, undeclared unit, overflow, missing quantization rule or
  projection mismatch aborts uncommitted physical transaction. Retry не может
  превратить `NONDETERMINISTIC_RESULT` в успешный run.
- CPU runtime physics остаётся authority для v1 active simulation. GPU training
  or presentation cannot become runtime physical authority.

### Motor является proposal layer

`MotorAction` MAY содержать joint targets, bounded torques/forces, controller
gains or other engine-owned actuator parameters declared body profile. Он MUST
NOT содержать backend handle, direct world/body transform, collision-filter
mutation, topology mutation, gameplay effect, animation cursor or arbitrary
callback.

Model evaluator, procedural controller and heuristic fallback use the same
action schema and safety validator. First-party policy не получает hidden
actuation API. Exact `MotorAction` owner до validation — Motor Runtime; после
validation physics consumes one immutable actuation record, but never transfers
pose authority to the producer.

Rejected, stale, non-finite, incompatible or unsafe action changes no body
state. The declared deterministic safe-hold/recovery route remains available.
Wall watchdog MAY protect the process only by marking the run
`NonConforming`; it cannot be an authoritative policy-selection clock or gate
evidence.

### Animation и presentation не являются gameplay authority

For `FullArticulation` and `SimplifiedActiveRagdoll`:

- animation clips and graphs MAY produce reference trajectories, phase
  features and bounded motor targets;
- physics produces final body pose and contacts;
- visual IK, cloth, jiggle, facial, camera-facing and other secondary layers
  modify only `RenderPose`;
- authored root motion becomes `PhysicalAvatarIntent`, never a teleport or
  direct transform write.

For `CapsuleAnimation`, the validated capsule locomotion controller owns the
collision transform while Animation Runtime owns only visual local pose. The
declared visual leash is diagnostic/presentation behavior; visual pose cannot
move the capsule or prove a contact.

For `Abstract`, no hidden body/contact exists. World Services/RPG owners
produce only their declared abstract outcomes. Animation and renderer absence
changes no mandatory result.

Animation events MAY emit presentation cues or future command proposals, but
do not mutate RPG, physics or topology state. A marker name, frame index,
rendered bone transform or GPU skinning buffer is not authoritative identity.

### LOD and topology transitions

Physical LOD and topology transitions preserve ADR-013/SPEC-05 ownership:

- request is a proposal derived only from canonical simulation facts;
- one validated state machine owns `Prepare → Validate → Commit → Stabilize`;
- source authority remains active until atomic commit;
- motor policy switch, topology mutation and LOD transition do not commit on
  the same body boundary unless a future Accepted contract defines their
  composition;
- animation bridge consumes the committed result and never selects transition
  success;
- failure retains the last safe tier/topology/route without partial pose copy.

`WorldResidencyTier`, physical LOD and animation detail are separate state
machines. Camera visibility, occlusion or presentation importance cannot
collapse them into one mutable level.

### Public boundary and backend neutrality

Engine-owned contracts live in `crates/contracts` and use nominal IDs,
`PersistentId`, `AssetId`, versioned descriptors, canonical fixed-point values
and immutable snapshots. Public schemas MUST NOT expose:

- ECS storage/components or `RuntimeEntityId` in durable data;
- raw pointers, OS/window objects, tasks/futures or database connections;
- backend world/body/shape/joint/manifold/solver handles or enum values;
- model-runtime tensor/session handles;
- animation middleware graph/node handles or renderer/GPU buffer objects;
- importer/source-format types.

PhysX, Jolt and Bullet remain `Proposed` physics candidates in
EVIDENCE-001. ONNX Runtime remains a `Proposed` motor inference candidate.
This ADR selects none of them. Each implementation maps privately to the same
engine-owned contract and may be rejected without changing layer authority.

## Рассмотренные варианты

### Animation-authoritative root motion

Rejected. Collision and contact become a correction after an animation write,
and gameplay depends on frame sampling and graph order.

### Motor policy writes backend bodies directly

Rejected. Untrusted model output bypasses unit, topology, energy and safety
validation and leaks a vendor actuation API.

### One shared mutable pose for physics, animation and renderer

Rejected. It has multiple writers, cannot identify the causal owner of a
change and breaks headless/replay parity.

### Accept raw-float tolerance as authoritative equivalence

Rejected. Tolerance can hide branch, event and ordering divergence. Only
quantized canonical values may drive authority; raw tolerance is diagnostic.

### Select a physics or animation vendor in the layering ADR

Rejected. Authority and public semantics must survive backend replacement.
Technology selection requires separate evidence and, when status changes, a
separate ADR.

## Последствия

- Physics-world, motor-inference and skeletal-animation specifications refine
  one-way schemas under this ADR and MUST NOT create a second pose/contact or
  gameplay authority.
- Tests and tools use production descriptors, observations, actions and
  immutable snapshots; test-only transform mutation is forbidden.
- A backend may need an adapter or deterministic reference fallback to meet the
  canonical boundary. Inability to do so rejects that candidate; it does not
  weaken the contract.
- Observable physical/motor/animation changes still require automatic gates
  and hash-bound human evidence. This ADR creates neither evidence nor human
  approval.

## Supersession

ADR-027 complements ADR-009, ADR-013 and ADR-022 and does not supersede them.
Changing physics numeric authority, allowing motor/animation/presentation to
write active physical pose, or allowing raw tolerance to decide gameplay
requires a new Accepted ADR with explicit supersession and synchronized
specification, traceability, gate and evidence updates.
