# SPEC-27: Motor observation, action and deterministic inference

| Поле | Значение |
|---|---|
| ID | SPEC-27 |
| Статус | Accepted |
| Версия | 1.8 |
| Последняя проверка | 2026-08-12 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-057](adr/057-hierarchical-learnable-motor-system-and-policy-family-architecture.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](adr/059-event-sourced-physx-continuation-reconstruction.md), [ADR-064](adr/064-canonical-flat-command-locomotion-environment.md), [ADR-065](adr/065-curriculum-flat-command-locomotion-profile.md) |
| Заменяет | SPEC-27 1.7; admits the V2 curriculum consumer on the unchanged root-local layout/action boundary |

## История принятия

SPEC-27 подготовлен как часть architecture packet 1.8. Он закрепляет
engine-owned observation/action/state schemas, deterministic inference schedule,
safety clamp и procedural fallback для motor layer, принятого ADR-027.
Документ определяет contracts, но не выбирает inference backend и не создаёт
runtime implementation. Поддержка evaluator определяется только его
поведением в product checks ниже.

## Назначение и invariants

SPEC-27 определяет единственную public boundary между authoritative simulation,
motor policy evaluation, safety layer и physics actuation.

- Authoritative motor work происходит только на motor ticks, выведенных из
  `TickRateProfileV1`. Renderer frame, wall clock, worker identity, cache warmth
  и evaluator completion order не являются motor inputs.
- Каждая строка inference имеет canonical batching key
  `(motor_tick, PersistentId, PolicyId)`. Дубликат, неоднозначный active route,
  stale result или неполная batch result не выбираются эвристически.
- Observation, recurrent state input и candidate action имеют exact dtype,
  rank, shape, feature order, unit и normalization/de-normalization rules.
- Model output является untrusted proposal. Только полностью проверенный и
  safety-clamped fixed-point `MotorActionV1` может стать authoritative input
  physics layer.
- Learned action и следующий recurrent state публикуются одной atomic commit.
  Partial action, partial state или per-channel смешивание learned/fallback
  output запрещены.
- Policy/model/schema/safety/fallback inputs immutable, content-addressed и
  входят в exact `ProjectLockV3`.
- Runtime inference не использует RNG. Stochastic op, ambient seed, filesystem,
  network, environment variable или hidden mutable evaluator state в
  authoritative path являются contract violation.
- `game`, `headless` и `capture-worker` используют одни schema hashes, batch
  ordering, state transitions, safety clamp, fallback controller и commit
  rules. Compile-time feature или private backend не может менять semantics.

Для SPEC-27 два запуска equivalent только если на каждом motor tick совпадают
canonical observation-batch root, applied-action root, route/fallback decision,
recurrent-state root и stable diagnostic sequence. Raw evaluator timing,
private session caches и pre-clamp floating-point diagnostics не входят в
authoritative state, но разница, перешедшая через canonical decode/clamp,
является `NONDETERMINISTIC_RESULT`.

## Source of truth и authority split

| State | Единственный owner/source of truth | Allowed projection / forbidden duplicate |
|---|---|---|
| Observation/action/state schema, compatibility and safety policy | Physical Embodiment subsystem | immutable registry/manifest projection; no backend tensor descriptor |
| Motor tick cadence, closed work set, batch commit and replay ordering | Runtime subsystem | schedule trace; no policy-route or physical-state ownership |
| Body, actuator, contact and safety-envelope facts | Physical Embodiment subsystem | immutable revision-bound observation source; no evaluator-owned copy |
| High-level `AgentIntent` | Agent Intelligence subsystem | revisioned untrusted proposal; no direct torque/joint mutation |
| Active policy route, fallback phase and recurrent `PolicyState` | Physical Embodiment subsystem | immutable inspector/save projection; no evaluator-session authority |
| Policy/model/schema/controller bytes and project lock | Asset & Persistence subsystem / Project Composition owner | validated immutable content; no mutable runtime weights |
| Save generation and owner-segment encoding | Asset & Persistence subsystem | staged copy; semantic state remains Physical Embodiment-owned |

Public contracts contain only engine-owned nominal IDs, bounded integers,
canonical hashes, closed enums and versioned records. They contain no ECS
storage/component, raw pointer, OS/window/thread/task object, filesystem path,
database connection, importer structure, model-runtime session, provider,
device handle, physics-backend body, tensor-library object or other
vendor/backend type.

## Canonical scalar, unit and tensor rules

### Closed v1 scalar and unit vocabulary

`MotorTensorDTypeV1` has exactly one supported value:
`Ieee754Binary32LittleEndian`. Public inference tensors are rank two,
row-major, tightly packed and little-endian. No stride, sparse layout, native
endianness, implicit cast or backend-owned buffer appears in the contract.

`MotorUnitV1` is the closed enum:

```text
Unitless
Binary
Metre
MetrePerSecond
MetrePerSecondSquared
Radian
RadianPerSecond
RadianPerSecondSquared
Kilogram
Newton
NewtonMetre
NewtonSecond
NewtonPerMetre
NewtonSecondPerMetre
NewtonMetrePerRadian
NewtonMetreSecondPerRadian
KilogramSquareMetre
Second
OnePerSecond
```

Angles are radians, lengths are metres, time is seconds and torque is
newton-metres. Coordinate handedness and axes come only from the exact
physical/numeric profile. Degrees, centimetres, engine units and implicit
unit conversion are invalid.

Canonical tensor bytes are:

```text
MotorTensorV1 {
  role: MotorTensorRoleV1,
  dtype: Ieee754Binary32LittleEndian,
  rows: u32,
  columns: u32,
  tensor_schema_hash: Hash256,
  data: rows * columns * 4 bytes
}
```

`MotorTensorRoleV1` is exactly `Observation`, `PolicyStateInput`,
`ActionCandidate` or `PolicyStateOutput`. `rows` is `B`; `columns` is fixed by
the referenced schema. A tensor with excess bytes, padding, zero rows,
`B > max_batch_size`, wrong role/hash/shape or a NaN/infinity is invalid.
Negative zero is canonicalized to positive zero at the engine boundary.
Finite subnormal values remain their exact IEEE-754 rational values.

| Role | Exact shape and presence | Scalar meaning |
|---|---|---|
| `Observation` | `[B, O]`, always present, where `O = feature_count` | Feature-local physical unit is converted by its exact normalization rule. |
| `PolicyStateInput` | `[B, S]` iff `S > 0`; absent iff `S = 0` | Dimensionless normalized value derived from canonical fixed-point state. |
| `ActionCandidate` | `[B, A]`, always present, where `A = channel_count` | Dimensionless proposal decoded by the channel's exact physical-unit rule. |
| `PolicyStateOutput` | `[B, S]` iff `S > 0`; absent iff `S = 0` | Dimensionless proposal decoded to canonical fixed-point state. |

Every row position maps to the same batching key in all present tensors.
Dynamic feature/state/action width, singleton-rank elision and broadcasting are
forbidden.

### Exact normalization

Every source value first exists as a signed integer `source_raw` under one
locked SPEC-21 fixed-point descriptor. A feature uses exactly one
`MotorNormalizationRuleV1`:

```text
AffineRationalV1 {
  center_raw: i64,
  scale_numerator: i64,
  scale_denominator: NonZeroU64,
  normalized_min_numerator: i64,
  normalized_min_denominator: NonZeroU64,
  normalized_max_numerator: i64,
  normalized_max_denominator: NonZeroU64,
  range_mode: Reject | Clamp
}

BinaryMaskV1
```

For `AffineRationalV1` the exact mathematical value is
`(source_raw - center_raw) * scale_numerator / scale_denominator`, evaluated
with checked signed 128-bit intermediates. Bounds are exact rationals and MUST
satisfy `normalized_min <= normalized_max` after checked cross-multiplication.
`Reject` rejects the complete subject observation outside the interval;
`Clamp` clamps the rational before conversion. The final binary32 bits are
produced once, using round-to-nearest, ties-to-even. Overflow or invalid
denominator rejects the complete observation.

`BinaryMaskV1` accepts only integer `0` or `1` and emits exact binary32
`+0.0` or `1.0`. An optional physical source MUST have a dedicated validity
mask feature. Missing data is encoded as zero only while that exact mask is
zero; missing required data, a missing mask or a nonzero masked payload rejects
the subject observation. No backend performs normalization.

## Observation, action and inference schemas

### `MotorObservationSchemaV1`

```text
MotorObservationSchemaV1 {
  schema_id: SchemaId,
  schema_version: 1,
  feature_count: u32,
  features: [MotorFeatureDescriptorV1],
  physical_numeric_profile_hash: Hash256,
  body_schema_hash: Hash256,
  actuator_schema_hash: Hash256
}

MotorFeatureDescriptorV1 {
  feature_id: NamespacedId,
  offset: u32,
  width: u16,
  source_semantic_id: NamespacedId,
  source_component: u16,
  unit: MotorUnitV1,
  source_fixed_point_descriptor_id: FixedPointDescriptorId,
  normalization: MotorNormalizationRuleV1,
  validity_mask_feature_id: Option<NamespacedId>
}
```

Features sort by `(offset, feature_id)`. Offsets begin at zero, have no gaps or
overlap and cover exactly `feature_count`. Every
`source_semantic_id/source_component` resolves in the exact registered
physical-observation schema; an unknown semantic requires a new schema version.
The schema MUST explicitly include the policy-required root/joint kinematics,
intent targets, contact/support/terrain facts, previous applied action and
actuator/validity masks. ADR-057 profiles also bind exact BodySchema/effective
projection facts needed by the policy: cached static morphology embedding or
its deterministic source, equipment/load, effective mass/inertia/CoM/ROM,
torque-speed-power/latency, damage/fatigue/sensory confidence, semantic skill/
contact/motion command and compact adaptation latent.

The main observation MUST NOT contain a raw terrain mesh, video frame,
unbounded object list, free-form text, Tactical/Strategic memory or hundreds of
implicit history frames. Terrain/depth/perception uses a separately bounded
encoder/profile; recent action-response history belongs to the explicit
adaptation profile. It MUST NOT read presentation pose, raw physics-backend
state or mutable ECS storage.

`MotorObservationV1` binds one subject and tick to:

```text
MotorObservationV1 {
  motor_tick: u64,
  persistent_id: PersistentId,
  physics_body_id: PhysicsBodyIdV1,
  policy_id: PolicyId,
  route_generation: u64,
  observation_schema_hash: Hash256,
  source_physics_tick: u64,
  source_physics_substep: u16,
  source_physics_snapshot_hash: Hash256,
  body_revision: u64,
  topology_revision: u64,
  intent_revision: u64,
  safety_envelope_hash: Hash256,
  source_projection_hash: Hash256,
  values: MotorTensorV1   // shape [1, O], role Observation
}
```

The tensor is derived only from the immutable, revision-bound source
projection closed at the motor stage. `source_projection_hash` covers the
canonical fixed-point source integers and masks before normalization.

### `MotorActionSchemaV1` and authoritative action

```text
MotorActionSchemaV1 {
  schema_id: SchemaId,
  schema_version: 1,
  channel_count: u32,
  channels: [MotorActionChannelV1],
  actuator_schema_hash: Hash256,
  physical_numeric_profile_hash: Hash256
}

MotorActionChannelV1 {
  action_channel_id: NamespacedId,
  offset: u32,
  actuator_slot_id: NamespacedId,
  semantic: AngularPositionTarget | LinearPositionTarget |
            AngularVelocityTarget | LinearVelocityTarget |
            Torque | Force | AngularStiffness | LinearStiffness |
            AngularDamping | LinearDamping,
  physical_unit: MotorUnitV1,
  output_fixed_point_descriptor_id: FixedPointDescriptorId,
  zero_raw: i64,
  scale_numerator: i64,
  scale_denominator: NonZeroU64,
  schema_min_raw: i64,
  schema_max_raw: i64
}
```

Channels sort by `(offset, action_channel_id)`, start at zero, have no gaps or
overlap and cover exactly `channel_count`. Model output is
`ActionCandidate[B, A]` binary32. For one channel its exact finite IEEE rational
`x` decodes to physical raw candidate
`zero_raw + x * scale_numerator / scale_denominator`, with checked 128-bit
intermediates and one round-to-nearest, ties-to-even conversion to the declared
fixed-point integer. Invalid scale, nonfinite value or overflow rejects the
whole row and its state output. `schema_min_raw <= schema_max_raw` is required;
channel/actuator IDs and offsets are unique.

Semantic/unit pairs are exact: angular/linear position use
`Radian`/`Metre`; angular/linear velocity use
`RadianPerSecond`/`MetrePerSecond`; `Torque`/`Force` use
`NewtonMetre`/`Newton`; angular/linear stiffness use
`NewtonMetrePerRadian`/`NewtonPerMetre`; angular/linear damping use
`NewtonMetreSecondPerRadian`/`NewtonSecondPerMetre`. Any other pair is a schema
error. The actuator slot's SPEC-26 axis kind and `PhysicsActuatorBoundsV1`
must admit the same semantic and unit.

Under ADR-058/SPEC-35, the current Stage 0 humanoid action schema emits bounded
residual joint-position targets relative to the exact neutral/authored
reference, with optional velocity channels. Engine-owned fixed stiffness/
damping and torque-speed-power/velocity/rate limits remain in the safety
profile; the model cannot rewrite them. A separately bounded small residual
torque MAY be evaluated only after the position-target baseline. Learned
stiffness/damping, direct torque and muscle activation require separate
quality/safety profiles and gates; they are not the default route.

The current records are `MotorObservationLayoutV1`, `MotorActionLayoutV1`,
`MotorWorldCheckpointV1` and `PolicyStateRecordV1`; Stage 0 populates empty but
explicit adaptation/generator/router state and never hides evaluator history.

ADR-064 adds one exact training-only observation layout without changing the
standing layout. `nextengine.motor.env.humanoid-flat-command.v1` has width 84
and order `root quaternion xyzw`, root-local linear velocity, root-local angular
velocity, 23 joint positions, 23 joint velocities, 23 previous **applied**
actions, local-right/local-forward/yaw-rate command and two declared-foot
contact flags. World-to-root-local conversion uses checked `i128` Q1.30 matrix
math and round-to-nearest, ties-to-even; mirrors use the same golden algorithm.
`O_t` binds engine command `C_t`, action/reward consume `C_t`, and the committed
result carries `O_(t+1)` with `C_(t+1)`. The standing layout remains
world-frame and is not silently reinterpreted.

ADR-065's curriculum profile consumes this same 84-channel layout and exact
action schema. Only its engine-owned episode-ordinal command schedule and
reward-profile hashes differ; there is no second tensor layout, trainer-owned
command or action reinterpretation.

Only the following fixed-point record is authoritative:

```text
MotorActionV1 {
  motor_tick: u64,
  persistent_id: PersistentId,
  physics_body_id: PhysicsBodyIdV1,
  policy_route_id: NamespacedId,
  policy_id: PolicyId,
  route_generation: u64,
  causal_command_id: CommandId,
  action_slot: u32,
  target_physics_tick: u64,
  target_physics_substep: u16,
  action_schema_hash: Hash256,
  safety_profile_hash: Hash256,
  safety_envelope_hash: Hash256,
  source: Learned | LastSafeHold | ProceduralRecovery,
  channel_values_raw: [i64; A],
  clamp_mask: BitSet<A>,
  action_hash: Hash256
}
```

`MotorActionV1` contains no float and no backend handle. Physics receives it
only at the ADR-027 motor-to-physics commit boundary. `clamp_mask` is exactly
`ceil(A / 8)` bytes, with channel offset `i` stored in bit `i mod 8` of byte
`floor(i / 8)`, least-significant bit first; unused high bits are zero.
`action_hash` is the domain-separated SHA-256 of every preceding canonical
field and does not hash itself. `policy_route_id`, `causal_command_id` and
`action_slot` come from the validated active route/intent expansion, never
arrival or worker order, and match the SPEC-26 accepted-action ordering key.

### Immutable `MotorInferenceProfileV1`

```text
MotorInferenceProfileV1 {
  profile_id: NamespacedId,
  profile_revision: u32,
  policy_id: PolicyId,
  policy_route_id: NamespacedId,
  policy_bundle_hash: Hash256,
  model_asset_hash: Hash256,
  observation_schema_hash: Hash256,
  action_schema_hash: Hash256,
  policy_state_schema_hash: Hash256,
  compatibility_key_hash: Hash256,
  physical_numeric_profile_hash: Hash256,
  safety_profile_hash: Hash256,
  fallback_input_schema_hash: Hash256,
  fallback_controller_hash: Hash256,
  motor_period_physics_substeps: NonZeroU16,
  max_batch_size: NonZeroU16,
  required_evaluator_capabilities: [MotorEvaluatorCapabilityV1]
}
```

Capabilities are closed engine-owned operation/numeric limits, never provider,
device or library names. Every referenced byte sequence and schema is immutable
and content-addressed in the exact `ProjectLockV3`. Runtime never
discovers a model, schema, normalization table or fallback controller by path,
environment, “latest” tag or mutable registry. Any hash/compatibility/tick-rate
mismatch makes the learned route unavailable before observation batching.
Runtime gameplay never trains or mutates weights.

ADR-057 accepts the BodySchema/adaptation/reference compatibility semantics but
does not silently extend this current wire record. The Proposed
consumer-backed schema evolution is:

```text
MotorInferenceProfile target extension {
  body_schema_hash: Hash256,
  body_instance_projection_schema_hash: Hash256,
  adaptation_profile_hash: Option<Hash256>,
  motion_reference_schema_hash: Option<Hash256>
}
```

For profiles without adaptation or motion reference, the corresponding option
is canonical `None`; hidden defaults are forbidden. The exact version/name and
registry entry remain Proposed until their first production consumer under
ADR-046.

## Canonical batching and result validation

At a motor stage Runtime closes the request set before evaluator work begins.
Each request is:

```text
MotorInferenceRequestV1 {
  key: (motor_tick, PersistentId, PolicyId),
  route_generation: u64,
  active_policy_route_hash: Hash256,
  profile_hash: Hash256,
  observation_hash: Hash256,
  policy_state_identity: PolicyStateIdentityV1,
  policy_state_tick: u64,
  prior_authoritative_state_hash: Hash256,
  safety_envelope_hash: Hash256
}
```

The global directory sorts lexicographically by the exact canonical key
`(motor_tick, PersistentId, PolicyId)`. One `PersistentId` has exactly one
active `PolicyId` per motor tick. Equal keys with equal bytes are one duplicate
submission error; equal keys with unequal bytes are an identity collision.
Both reject learned evaluation for that subject. Runtime MUST NOT use insertion
order or “last writer wins”.

The sorted directory is partitioned by exact inference-profile hash. Within a
profile, row order is the projection of the global canonical order. Groups
larger than `max_batch_size` split into consecutive fixed-size chunks; only the
last chunk may be shorter. Padding, if privately required by an adapter, is
non-authoritative and cannot add a public row. `MotorInferenceBatchV1` hashes
the ordered keys, profile, observation tensor `[B, O]`, optional state tensor
`[B, S]`, ordered full `prior_authoritative_state_hash` values and all other
source hashes.

```text
MotorInferenceResultV1 {
  batch_hash: Hash256,
  ordered_keys: [(motor_tick, PersistentId, PolicyId); B],
  profile_hash: Hash256,
  active_policy_route_hashes: [Hash256; B],
  route_generations: [u64; B],
  prior_authoritative_state_hashes: [Hash256; B],
  action_candidate: MotorTensorV1,              // [B, A]
  policy_state_output: Option<MotorTensorV1>    // [B, S] iff S > 0
}
```

An accepted result MUST echo the batch hash, exact ordered keys, profile hash,
active-route hashes, route generations, every exact
`prior_authoritative_state_hash` and output shapes `[B, A]` and, when stateful,
`[B, S]`. A result whose prior authoritative-state hash differs from the
request/current authoritative record is stale and cannot be rebound to newer
state. Before any subject commits, the complete batch envelope and row set
validate and every row reaches one terminal decision: `ValidatedLearned` or
`RejectedToDeclaredFallback`. Those decisions commit in canonical key order.
A missing, extra, duplicate, stale, reordered or partially decoded row rejects
the complete affected row; no neighboring subject is substituted and no
partial row commits.

Every `prior_authoritative_state_hash` request/result field is an exact binding
to `PolicyStateRecordV1.authoritative_state_hash`. There is no second
recurrent-vector-only state hash.

Worker count, task stealing, SIMD width and completion order may change private
execution only. The batch/result roots and applied action/state commits MUST be
identical for worker counts `1, 2, 4, 8` and every product-check-declared request and
completion permutation.

## Recurrent `PolicyState` identity, reset and persistence

Every policy has one content-addressed `PolicyStateSchemaV1`, including a
stateless policy. It defines exact state width `S`; `S = 0` means both state
tensors are absent while the identity/record still carries last-action and
fallback continuity. For `S > 0`:

```text
PolicyStateSchemaV1 {
  schema_id: SchemaId,
  schema_version: 1,
  state_width: u32,
  elements: [PolicyStateElementV1]
}

PolicyStateElementV1 {
  state_element_id: NamespacedId,
  offset: u32,
  width: u16,
  storage_fixed_point_descriptor_id: FixedPointDescriptorId,
  model_input_normalization: AffineRationalV1,
  model_output_zero_raw: i64,
  model_output_scale_numerator: i64,
  model_output_scale_denominator: NonZeroU64,
  storage_min_raw: i32,
  storage_max_raw: i32,
  initial_value_raw: i32
}
```

### Explicit adaptation, generator and expert-router segments

ADR-057 requires every future-action-affecting history value to live inside
the generic schema above. A stateful profile declares a canonical fixed
segment table:

```text
MotorPolicyStateSegmentV1 {
  segment_id: NamespacedId,
  kind: LowLevelPolicy | DynamicsAdaptation |
        MotionGenerator | ExpertRouter,
  offset: u32,
  width: NonZeroU32,
  element_schema_range_hash: Hash256,
  update_period_motor_ticks: NonZeroU16,
  reset_profile_hash: Hash256,
}
```

Segments sort by `(offset, segment_id)`, are contiguous/non-overlapping and
cover exactly the profile-declared `S`. Values live only in
`PolicyStateRecordV1.values_raw`; segment metadata creates no second state
store. Explicit known equipment/stats/damage/fatigue/actuator parameters are
observation inputs, not guessed adaptation state.

`MotorAdaptationProfileV1` additionally binds the bounded recent
observation→applied-action→response source schema/window, update cadence,
teacher-only privileged schema hash, latent segment and reset/remap behavior.
The runtime profile exposes no privileged value and performs no gradient or
optimizer update. TCN/GRU are the first Proposed comparators. A Mamba/SSM cache
may use the same generic segment only in an equal-budget Proposed experiment;
no Mamba-specific public state type exists.

Dynamic unbounded history, evaluator-session cache, provider-owned recurrent
state, implicit warm state and router state reconstructed from arrival order
are forbidden. The portable evaluator remains one fixed-shape pure step
`(Observation[B,O], State[B,S]) → (Action[B,A], NextState[B,S])` and must pass
trainer/export/Windows/Linux canonical action/state parity.

Elements have the same contiguous ordering rules as observation features.
Authoritative state is `[i32; S]`, not raw evaluator float. Input normalization
uses the exact observation conversion. Each finite state output is the exact
rational `x`; its storage candidate is
`model_output_zero_raw + x * model_output_scale_numerator /
model_output_scale_denominator`, converted with checked 128-bit arithmetic and
round-to-nearest, ties-to-even. It MUST fall in the declared storage interval,
and `initial_value_raw` MUST also be inside that interval. State output is never
silently clamped. Any invalid element rejects the whole learned action/state
pair.

```text
PolicyStateIdentityV1 {
  persistent_id: PersistentId,
  policy_id: PolicyId,
  policy_bundle_hash: Hash256,
  policy_state_schema_hash: Hash256,
  route_generation: u64,
  state_generation: u64
}

PolicyStateRecordV1 {
  schema_version: 1,
  identity: PolicyStateIdentityV1,
  active_policy_route_hash: Hash256,
  action_schema_hash: Hash256,
  state_tick: u64,
  values_raw: [i32; S],
  last_applied_action_tick: u64,
  last_applied_action_hash: Hash256,
  last_applied_channel_values_raw: [i64; A],
  consecutive_learned_unavailable_ticks: u16,
  fallback_phase: Learned | LastSafeHold | ProceduralRecovery,
  authoritative_state_hash: Hash256
}
```

`authoritative_state_hash` is the sole normative hash of the complete
authoritative record, not merely the recurrent vector:

```text
policy_state_record_payload_bytes =
  CanonicalBinaryV1(
    owner_id = "nextengine.motor",
    schema_id = "nextengine.policy-state-record",
    segment_id = "v1",
    value = PolicyStateRecordV1 with authoritative_state_hash omitted
  )

authoritative_state_hash = SHA256(
  "nextengine.policy-state-record.v1\0"
  || u64_le(policy_state_record_payload_bytes.len)
  || policy_state_record_payload_bytes
)
```

The payload therefore covers identity and both generations, active route,
action schema, recurrent state tick/vector, previous applied action tick/hash/
raw values, unavailable counter and fallback phase. Every authoritative field
that can affect a future observation, clamp, action, route, reset or fallback
MUST be inside this payload. An extension cannot add an unhashed authoritative
field; it requires a new registered record schema/hash domain.
`authoritative_state_hash` itself is omitted only to avoid self-reference.

Before inference request construction, result validation, save publication,
load, replay restore or replay comparison, the consumer MUST canonicalize the
complete record once, recompute `authoritative_state_hash` and compare it
exactly. Field use before this comparison is forbidden.
State identity and tick MUST match the request exactly. A result for an older
tick or route generation is stale even when tensor bytes match.

After learned or fallback validation, Runtime constructs the complete next
record including the newly applied action, counter and phase, computes its full
`authoritative_state_hash`, then publishes:

```text
PolicyStateCommitV1 {
  key: (motor_tick, PersistentId, PolicyId),
  prior_authoritative_state_hash: Hash256,
  next_authoritative_state_hash: Hash256,
  applied_action_hash: Hash256,
  route_generation: u64,
  state_generation: u64
}
```

`prior_authoritative_state_hash` MUST equal the request/result echo and current
authoritative record. `next_authoritative_state_hash` MUST recompute from the
exact record published beside the applied action. `MotorActionV1`,
`PolicyStateRecordV1` and
`PolicyStateCommitV1` become visible atomically; no learned, hold or recovery
path can publish an action while retaining a hash for different counter,
phase, previous action or recurrent state.

`PolicyStateResetReasonV1` is the closed enum `Spawn`,
`PolicyRouteCommitted`, `BodyTopologyChanged`, `PhysicalLodChanged`,
`RecoveryEntered` and `SaveMigrationCommitted`. Reset fills every element with
its declared `initial_value_raw`, increments `state_generation` on every reset,
increments `route_generation` only when active route or compatibility identity
changed, and records the reason in replay. Generation increment is checked;
overflow fails closed. Same-policy save/load restores exact state without
reset.

ADR-027 transition exclusion is mandatory: a policy-route commit cannot share
one body boundary with a topology or physical-LOD commit. The topology/LOD
transaction commits or aborts first; its exact result becomes input to the
earliest following eligible motor tick, where route validation and any reset
occur atomically.

Cross-policy state carry and transform are forbidden in v1. A committed route
change creates the target `PolicyStateIdentityV1`, increments both generations
as applicable and fills the complete target vector from its locked
`initial_value_raw` values. A bundle that declares carry, partial element copy
or an external handoff transform is incompatible with SPEC-27 v1 and route
activation fails closed. Supporting cross-policy migration requires a future
versioned schema and synchronized architecture decision; it cannot be inferred
from equal tensor widths.

Same-policy save migration has exactly two v1 dispositions:
`RestoreExactIdentity` when every identity/hash is unchanged, or
`ResetToTargetInitial` when the target locked route explicitly declares
`SaveMigrationCommitted`. The latter runs on a complete copy, validates the
target bundle/schema/action/safety closure, increments `state_generation`,
records the reset and publishes one new save generation atomically. Numeric
element transform, truncation, extension by implicit zero or in-place source
edit is forbidden. Missing disposition rejects load and preserves the source
save.

The Physical Embodiment save segment stores the complete canonical
`PolicyStateRecordV1`, its exact `authoritative_state_hash`, the active route
and the atomic commit link that accepted it. The owner-segment/save-generation
root covers those exact bytes and hash; a loader recomputes
`authoritative_state_hash` before reading any previous action, counter, phase
or recurrent value. Evaluator sessions, allocator/task state, compiled graphs,
device state, caches, padding and raw float outputs are reconstructible and
MUST NOT be serialized. Hash mismatch rejects the complete staged owner
segment and preserves the immutable source save plus prior active generation.
Save/load/save and replay restore MUST reproduce exact canonical record bytes,
`authoritative_state_hash`, commit link and state/action roots.

## Deterministic safety clamp

`MotorSafetyProfileV1` and the per-tick `MotorSafetyEnvelopeV1` are
engine-owned, fixed-point and content/revision bound:

```text
MotorSafetyChannelRuleV1 {
  action_channel_id: NamespacedId,
  actuator_slot_id: NamespacedId,
  actuator_min_raw: i64,
  actuator_max_raw: i64,
  max_negative_delta_raw_per_motor_tick: u64,
  max_positive_delta_raw_per_motor_tick: u64,
  neutral_raw: i64
}

MotorSafetyEnvelopeChannelV1 {
  action_channel_id: NamespacedId,
  envelope_min_raw: i64,
  envelope_max_raw: i64,
  enabled: bool
}
```

The envelope binds `motor_tick`, `PersistentId`, body/topology/contact
revisions, the exact source `PhysicsCanonicalSnapshotV1` hash,
action/safety profile hashes and one entry for every action channel. Disabled
means the interval is the singleton `neutral_raw`. Missing, duplicate, stale
or inverted bounds invalidate the learned row.

For each channel in schema order, the clamp computes:

1. `schema_interval = [schema_min_raw, schema_max_raw]`;
2. `actuator_interval = [actuator_min_raw, actuator_max_raw]`;
3. the exact envelope interval, or singleton neutral when disabled;
4. the rate interval around the previous applied raw value using the declared
   negative/positive per-motor-tick deltas;
5. `allowed = intersection` of all four closed integer intervals;
6. `applied_raw = min(max(candidate_raw, allowed.min), allowed.max)`.

Checked integer arithmetic is mandatory. An empty interval, overflow, identity
or revision mismatch rejects the whole candidate. Otherwise numeric excess is
clamped deterministically and its bit in `clamp_mask` is set. Channels process
in canonical order, but no channel is published until every channel and the
next recurrent state pass. There is no epsilon, native float comparison,
energy “best effort” or backend-specific clamp.

The safety layer resolves coupled/contact/energy limits into the immutable
per-channel envelope before inference commit. Its own deterministic
construction is part of `MOTOR-SAFETY-P1`; an evaluator cannot weaken,
reinterpret or mutate it.

## Procedural fallback and route semantics

Every `MotorInferenceProfileV1` names one immutable
`ProceduralMotorControllerV1` revision through
`fallback_controller_hash` and one exact engine-owned fixed-point input schema
through `fallback_input_schema_hash`. It consumes the same closed source
projection used to build the model observation, plus exact intent, prior
applied action and safety envelope; it never consumes the possibly invalid
model tensor or reads extra mutable state. It performs checked fixed-point
arithmetic only and emits a complete candidate passed through the same action
schema and safety clamp. The v1 controller is stateless; its only continuity
inputs are the persisted prior action, unavailable counter and fallback phase
in `PolicyStateRecordV1`. If the source projection or fallback input schema is
itself invalid, no procedural action is fabricated and the conformant commit
stops. The controller has no model runtime, RNG, wall clock, network,
filesystem or hidden device state.

Its output is `ProceduralActionCandidateV1 { action_schema_hash,
channel_values_raw: [i64; A] }` in the exact per-channel fixed-point units.
It bypasses only model-tensor decode; it bypasses no schema, actuator,
envelope, rate or atomic-publication check.

A fallback decision is a deterministic route fact. It may be selected only by:

- preflight incompatibility or unavailable supported evaluator capability;
- a canonical evaluator error result;
- a product-check/scenario-injected logical `EvaluatorUnavailable` fault;
- schema/hash/nonfinite/overflow/state/result rejection;
- an already committed supervisor transition or recovery condition.

For one committed learned failure, the unavailable counter increments. Ticks
one and two use `LastSafeHold` when the previous action remains valid after the
current safety clamp. Tick three and later use `ProceduralRecovery`. If the
held action is unavailable or no longer safe, procedural recovery starts
immediately. One complete compatible learned result resets the counter and
returns to `Learned` only through the declared supervisor transition.
The counter saturates exactly at `u16::MAX`. Last-safe hold preserves the
learned recurrent vector and its `state_tick`; entry to procedural recovery
performs the exact `RecoveryEntered` reset before a future learned evaluation.
Fallback action, fallback phase and recurrent-state disposition commit
atomically. A fallback controller failure preserves the last committed
authoritative state, emits `MOTOR_FALLBACK_UNAVAILABLE` and stops the
motor commit; it cannot fabricate an unsafe action.

Measured elapsed time or a wall-clock timeout MUST NOT create any of these
logical route facts. A wall deadline miss fails the conditional `performance`
ProductCheck with `MOTOR_WALL_DEADLINE_NONCONFORMING`. A
production watchdog may pause the authoritative tick and engage an
out-of-band emergency safety stop, but it MUST NOT record a different
authoritative motor action or continue the run as passed. Retry-to-green,
adaptive batch size, worker-count route choice and “use whichever result
arrives first” are forbidden.

## Replay, diagnostics and divergence

Replay records the closed request directory, batch hashes, exact key lists,
profile/schema/content hashes, logical evaluator result/fault, safety-envelope
hash, route/fallback phase, applied action hash, canonical
`PolicyStateRecordV1` bytes or their content-addressed reference, full prior and
next `authoritative_state_hash` values and the atomic `PolicyStateCommitV1`
boundary, plus the periodic canonical physics snapshots required by the locked
profile. Authoritative replay consumes and validates the recorded canonical
applied action/full-state chain; it does not require model re-execution to
advance the reproduced physical world. Replay rehydration recomputes each
`authoritative_state_hash` before
use and verifies that every `prior_authoritative_state_hash` equals the
preceding `next_authoritative_state_hash`. Model bytes need not be duplicated
when their content hash is resolvable from the locked project closure.

For the current PhysX Stage 0 profile, the restorable owner-segment closure also
contains the episode-origin reset and every post-safety effort for exactly four
240 Hz substeps per recorded 60 Hz frame, bounded to 3,600 motor frames. These
efforts, not an action-to-PD recomputation, advance fresh-scene rehydration.
Each frame carries an exact physics witness hash so divergence stops at the
first differing motor tick; the final full snapshot must also match. The
current standalone encoded prefix is limited to 4 MiB.

Evaluator re-execution is a separate parity/correspondence mode. For a
`Supported` route it MUST decode/clamp to the same canonical action and full
next state on Windows x86_64 and Linux x86_64. Raw evaluator tensor tolerance
may be diagnostic before canonical decode only; a different applied action or
state is `NONDETERMINISTIC_RESULT`.

First divergence reports motor tick, `PersistentId`, `PolicyId`, route
generation, state generation, batch/profile/schema hashes and the first
differing authoritative record field, observation feature, action channel,
clamp bit or state element. A record/chain mismatch stops replay at that
compare-point before action or state publication. Raw wall timing and private
evaluator telemetry are diagnostic attachments only.

| Code | Required result |
|---|---|
| `MOTOR_POLICY_INPUT_INVALID` | Reject the learned route before batching; preserve previous committed route/state and enter only the declared logical fallback. |
| `MOTOR_OBSERVATION_INVALID` | Reject the complete subject observation; no evaluator call or partial action/state publication. |
| `MOTOR_BATCH_KEY_COLLISION` | Reject conflicting/duplicate learned work for the subject; never choose first/last writer. |
| `MOTOR_RESULT_STALE` | Discard the stale payload; a current-tick rejection follows the exact declared fallback decision, while a late payload after commit changes no state. |
| `MOTOR_RESULT_INVALID` | Reject wrong batch/profile/key/shape/dtype/hash or nonfinite/overflow output as one learned action/state pair. |
| `MOTOR_STATE_IDENTITY_MISMATCH` | Reject restore/result/migration before publication; apply only the declared reset or route failure. |
| `MOTOR_STATE_HASH_MISMATCH` | Recomputed `authoritative_state_hash` differs from the stored/bound value or hash chain; reject the complete inference request/result, save owner segment or replay record before any covered field is used, preserve the prior committed root and report the first differing field/byte when available. |
| `MOTOR_SAFETY_ENVELOPE_INVALID` | Reject the candidate and preserve the previous committed state; never infer missing limits. |
| `MOTOR_FALLBACK_UNAVAILABLE` | Stop the motor commit and preserve the last committed authoritative state. |
| `MOTOR_WALL_DEADLINE_NONCONFORMING` | Fail the active conditional `performance` ProductCheck; wall time does not select another authoritative action. |
| `NONDETERMINISTIC_RESULT` | Fail at the first canonical batch/action/route/state divergence; retry cannot turn the run green. |

## Deterministic positive and negative corpora

Every corpus is immutable, content-addressed and bound to its product check.
“Reject” means the named stable diagnostic, zero partial action/state/route
publication and preservation of the exact prior authoritative root.

| Corpus | Required positive set | Required negative set | Exact oracle |
|---|---|---|---|
| `motor-schema-golden-v1` | At least 4,096 rows covering every supported unit, normalization mode, optional-mask state, tensor role and exact rational/ties-to-even boundary | Wrong dtype/rank/shape/role/length/endianness, gap/overlap/duplicate feature, zero denominator, unit/schema/hash mismatch, missing mask, masked nonzero payload, overflow, NaN and infinities | exact source-projection, tensor-byte and schema roots; 100% invalid cases reject |
| `motor-batch-permutation-v1` | 10,000 motor ticks for 1 and 16 subjects across worker counts `1,2,4,8`, every declared request/completion permutation and each fixed batch split | Duplicate/conflicting key, two active policies for one subject/tick, reordered/missing/extra/stale row, wrong route generation or `prior_authoritative_state_hash`, partial batch and result replay after reset | exact ordered key directory, batch/result/action/`authoritative_state_hash` roots for every permutation; every fault maps to one stable diagnostic |
| `motor-safety-boundary-v1` | For every channel: `min`, `min+1`, interior, `max-1`, `max`, rate-boundary and disabled-neutral cases, including exact `+0`, negative-zero canonicalization and finite subnormal inputs | `min-1`, `max+1`, empty/inverted/missing/stale envelope, nonfinite values, largest finite overflow, invalid scale, actuator/schema mismatch and procedural-controller fault | valid finite excess clamps to the exact integer boundary; structural/numeric invalidity rejects the whole pair; no partial channel commit |
| `motor-state-lifecycle-v1` | 10,000 evaluate/save/load/replay cycles for stateless and recurrent profiles, every reset reason, same-policy restore, every cross-policy reset and known-answer `authoritative_state_hash` values for `Learned`, `LastSafeHold` and `ProceduralRecovery` | One-field/one-bit tamper of schema version, identity/generations, route/action-schema hash, state tick/vector, previous action tick/hash/raw values, unavailable counter, fallback phase or stored `authoritative_state_hash`; request/result prior-hash mismatch; broken replay `prior_authoritative_state_hash`→`next_authoritative_state_hash` link; unknown/trailing field, out-of-range element, forbidden carry and truncated save | exact canonical record bytes/`authoritative_state_hash`, action root, commit link and reset/save-migration event after every cycle; every tamper yields `MOTOR_STATE_HASH_MISMATCH` with zero inference/restore/replay or partial publication |
| `motor-route-fault-v1` | Learned success, capability-preflight fallback, canonical evaluator error, two valid last-safe holds, immediate unsafe-hold recovery, third-tick recovery and learned return transition | Tampered/missing policy/controller bytes, unsupported capability, undeclared route, fallback failure, wall-deadline miss and retry-to-green attempt | exact route/fallback/action/state roots; wall miss yields `Fail` and never a different authoritative action |

Corpus generation MUST enumerate every closed enum value and every manifest
declared feature/channel/state element. A schema extension updates the corpus
hash and product-check input; sampling an unrecorded subset cannot satisfy the check.
For `motor-state-lifecycle-v1`, the generator mutates every canonical byte
position in the full record and independently substitutes each semantically
valid future-action-affecting field while retaining the old
`authoritative_state_hash`. It also recomputes a tampered
`authoritative_state_hash` and verifies that request/result/save/replay
bindings reject it against the prior authoritative hash chain. Both classes
MUST fail before a covered field influences an action.

## Product checks

| ID | Сценарий | Ожидаемый результат | Fallback |
|---|---|---|---|
| `MOTOR-SCHEMA-P1` | `motor-schema-golden-v1`, all boundaries | one exact schema/tensor/source root for every positive encoding; all invalid encodings reject before evaluator/action/state publication; no implicit conversion | reject learned route; use locked procedural controller only with valid source/envelope |
| `MOTOR-SCHEDULE-P1` | 10 000 ticks, worker/request/completion permutations on Windows/Linux | key/batch/action/state roots exact across targets and permutations; no duplicate/stale/partial commit; reference CPU p99 ≤`0.5 ms/avatar` and ≤`2.0 ms/batch-of-16` | stop the product check; wall time never selects an authoritative fallback action |
| `MOTOR-SAFETY-P1` | `motor-safety-boundary-v1`, all channels | every valid boundary produces exact fixed-point clamp/action root; all structural/non-finite/overflow cases reject the learned pair atomically | run locked procedural candidate through the same clamp |
| `MOTOR-STATE-P1` | 10 000 evaluate/save/load/replay cycles | exact full record, `authoritative_state_hash`, commit links, last action, counter and phase; every tamper/reset/migration fault rejects before use | retain immutable source generation and prior full record; reset only when declared |
| `MOTOR-ROUTE-P1` | 100 repeats of every success/fault route | learned/hold/recovery transitions and action/state roots exact; at most two holds; wall misses never create another authoritative action | locked fixed-point procedural recovery; stop motor commit if unavailable |
| `MOTOR-REPLAY-P1` (future) | recorded action/full-state/snapshot replay plus independent evaluator re-execution across workers and Windows/Linux | authoritative replay reaches exact roots without model execution; parity mode independently reproduces every canonical action/state | reject learned artifact/profile and use procedural route |

`MOTOR-REPLAY-P1` remains `NOT_RUN(NO_PRODUCTION_CONSUMER)` until a learned
route and recorded-action replay consumer exist.

## Requirements

| ID | Technical requirement | Product checks |
|---|---|---|
| REQ-132 | Motor observation, action, safety and recurrent-state schemas MUST declare exact engine-owned dtype, rank, shape, order, units, fixed-point conversions and immutable hashes, with no vendor/backend/ECS/OS/importer type. | MOTOR-SCHEMA-P1 |
| REQ-133 | Inference MUST close and commit by `(motor_tick, PersistentId, PolicyId)`, remain invariant under worker/request/completion permutations and never let wall time select another authoritative action. | MOTOR-SCHEDULE-P1 |
| REQ-134 | Every learned and procedural candidate MUST pass the same exact fixed-point safety clamp, with atomic action/state publication, bounded two-tick hold and deterministic recovery route. | MOTOR-SAFETY-P1, MOTOR-ROUTE-P1 |
| REQ-135 | `PolicyState` MUST have exact identity/reset/migration rules, forbid cross-policy carry and bind every future-action-affecting field to one canonical hash across inference, save and replay. | MOTOR-STATE-P1 |

## Failure paths

| ID | Trigger | Required behavior | Product checks |
|---|---|---|---|
| FAIL-054 | Policy/model/schema/content mismatch, malformed tensor, invalid unit/normalization, non-finite/overflow value, stale envelope or unsafe/partial candidate | Reject the complete learned action/state pair before physics mutation. Use only the locked procedural path through the same clamp; stop if no safe action exists. | MOTOR-SCHEMA-P1, MOTOR-SAFETY-P1, MOTOR-ROUTE-P1 |
| FAIL-055 | Duplicate/conflicting key, missing/extra/stale completion, state-hash/chain/reset/migration/save fault, evaluator failure, wall miss or fallback failure | Validate the full record before use; preserve prior root on failure. Arrival order, retry and wall time are never authority. | MOTOR-SCHEDULE-P1, MOTOR-STATE-P1, MOTOR-ROUTE-P1 |

## Technology neutrality and claim boundary

This contract chooses no inference runtime, tensor library, physics backend,
job system, allocator, device or operating-system API. The outer policy bundle
is evaluator-format-neutral under ADR-057. Proposed ADR-053 records the first
replaceable profile: fixed-shape standard-op ONNX with explicit state and a
private ONNX Runtime adapter. SPEC-27 does not promote that profile merely by
documenting or exporting it. Any evaluator must pass the same schemas,
canonical action/state parity, fallback and ProductChecks before use.

Mamba has no public type or preferred low-level status. It may be evaluated
only through the same generic state/format boundary against the declared MLP,
TCN, GRU and procedural comparators.

The deterministic reference/fixed-point procedural path is the mandatory
offline fallback. Passing these checks proves exact runtime behavior for the
declared profile. Policy quality and `Prototype`/`Supported` status are defined
by SPEC-14.
