# SPEC-26: Physics world, collision, constraints, queries and canonical snapshots

| Поле | Значение |
|---|---|
| ID | SPEC-26 |
| Статус | Accepted |
| Версия | 2.8 |
| Последняя проверка | 2026-09-03 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md), [ADR-013](adr/013-self-contained-physical-avatar-boundary.md), [ADR-018](adr/018-authoritative-project-composition-and-configuration.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-025](adr/025-schema-content-and-migration-authority.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](adr/059-event-sourced-physx-continuation-reconstruction.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-068](adr/068-static-morphology-cache-and-action-chunk-field-closure.md) |
| Заменяет | SPEC-26 2.7; 2.8 records the Proposed ADR-105 buoyancy batch: `PhysicsWorldCheckpointV1` schema version 4 with the batch profile as field 6, `PhysicsStepInputV2` schema version 3 with `external_impulses` (field 10) applied once at the first substep, and the exact buoyancy/drag law. 2.7 recorded the WR1 increment (plan `continuum-water/10`): `mass_microkilograms` as field 9 of `PhysicsBodyDescriptorV1` and exact vertical free-body dynamics for up to `16` dynamic boxes in the bounded capsule profile. 2.6 recorded the Proposed ADR-103 water flow network as field 5 of `PhysicsWorldCheckpointV1` schema version 3, its exact per-tick step and its flux and volume queries |
| Дополнительные зависимости V2.5 | [ADR-100](adr/100-authoritative-water-volume-and-presentation-only-gpu-water.md) |
| Дополнительные зависимости V2.6 | [ADR-103](adr/103-authoritative-water-flow-network.md) |
| Дополнительные зависимости V2.8 | [ADR-105](adr/105-exact-level-buoyancy-reaction-batch.md) |
| Дополнительная зависимость V2.0 | [ADR-071](adr/071-canonical-physics-material-lineage.md) |
| Дополнительные зависимости V2.2 | [SPEC-36](36-functional-tissue-condition-and-injury.md), [ADR-075](adr/075-product-grounded-functional-anatomy-and-character-embodiment.md) |

## История принятия

SPEC-26 принят как часть consolidated architecture packet 1.8. Он
специализирует upstream authority ADR-027 для physics-world API, collision,
constraints, scene queries, contact stream and canonical snapshot/restore.

ADR-032 сохраняет эти semantics, но резервирует active implementation version
`PhysicsCanonicalSnapshotV2`: ранний private version-1 prototype не содержал
complete body/contact/continuation closure и не изменяется same-version.

Документ определяет engine-owned contracts, но не создаёт runtime
implementation и не выбирает physics backend. Поддержка конкретного backend
определяется только его поведением в product checks ниже.

## Назначение и invariants

SPEC-26 задаёт единственную переносимую границу между simulation и
replaceable physics implementation.

- Physical Embodiment subsystem владеет active physics world, body pose/velocity,
  constraints, collision/contact state, query semantics and canonical physical
  snapshot.
- Public schemas являются engine-owned, versioned and registered through
  SPEC-22. Vendor/backend types, native handles and raw layouts никогда не
  пересекают boundary.
- Public and durable physical identity uses nominal engine IDs,
  `PersistentId`, `AssetId` and exact revisions/hashes. `RuntimeEntityId`
  является ephemeral adapter lookup only.
- Canonical units are metres, kilograms, seconds and radians in one
  right-handed coordinate system. Every numeric field has a finite declared
  range; non-finite, overflow and undeclared unit conversion fail closed.
- Descriptor bytes, IDs, topology, filter result, query/contact order,
  quantized values, snapshot bytes/root and gameplay-facing classifications
  are exact. Raw backend correspondence tolerance is diagnostic only.
- Physics query/contact callback order, manifold insertion, worker count,
  renderer presence, wall time and backend handle allocation never determine
  a public result.
- Body/joint/topology publication and snapshot restore are atomic. Failure
  exposes the complete prior world or no world, never a partial graph.
- `game`, deterministic `headless` and displayless `capture-worker` share the
  same descriptors, validator, stage order, quantization, query/contact
  normalization and snapshot format.
- Gameplay consequences of physical state are only proposals until the common
  validated Outcome `WorldCommand` transaction commits them.

## Authority and public boundary

| State | Единственный owner/source of truth | Allowed input / projection |
|---|---|---|
| Registered physics schemas and migrations | Asset & Persistence subsystem through SPEC-22/ADR-025 | Exact schema refs, compatibility and copy-on-write migration |
| Immutable collision assets and their feature mapping | Asset & Persistence subsystem through SPEC-24 | Exact `AssetId`, revision, record hash and canonical feature table |
| Active world/body/joint/contact/numeric state | Physical Embodiment subsystem | Validated descriptors, physical step batches and canonical snapshots |
| Stage/tick assignment and deterministic result merge | Runtime subsystem | SPEC-21 schedule/clock and staged immutable results |
| Motor route/action proposal | Motor Runtime | Read-only observation; accepted action enters one bounded step batch |
| RPG/mechanics result | Owning domain through common command transaction | Immutable physical outcome proposal |
| Animation and render pose | Animation/Presentation according to ADR-027 | Read-only committed physical projection |

`PhysicsBackendV1` is an engine-owned facade implemented behind a private
adapter. Public operations are world stage/validate/activate/destroy,
descriptor graph stage/validate/commit, fixed substep, bounded query batch,
normalized contact read and canonical snapshot/restore. An adapter MAY retain
opaque runtime handles internally, but they are not IDs, are never serialized
and cannot influence canonical ordering.

Scripts, packages, AI and tools MAY author or propose public descriptors and
commands according to capability. They cannot call an adapter or mutate an
active world directly. First-party content and controllers use the same
validator and command/proposal path.

## Canonical coordinate, unit and scalar contracts

### `PhysicsCoordinateProfileV1`

V1 has one canonical physical frame:

| Property | V1 value |
|---|---|
| Handedness | Right-handed; `+X × +Y = +Z` |
| Axes | `+X` right, `+Y` up, `+Z` forward |
| Positive rotation | Right-hand rule about the positive axis |
| Length / position | metre (`m`) |
| Mass | kilogram (`kg`) |
| Duration | second (`s`) |
| Angle / angular velocity | radian (`rad`), radian per second (`rad/s`) |
| Derived dynamics | `m/s`, `m/s²`, newton (`kg·m/s²`), newton-second, newton-metre, kilogram-square-metre |
| Quaternion | component order `(x, y, z, w)`; canonical Q1.30 sign and norm rules from SPEC-24 |

Canonical position storage is signed `i64` micrometres in the inclusive range
`-8_388_608_000_000..=8_388_608_000_000`. Quaternion sign is `w > 0`, or,
when `w = 0`, the first nonzero `(x, y, z)` component is positive. Its squared
Q1.30 norm is within the exact SPEC-24 bounds. A pose contains no runtime
scale: authored scale MUST be positive, validated and baked into exact
collision geometry before descriptor publication.

Every public physical scalar is:

```text
PhysicsScalarV1 {
  unit_id: NamespacedId,
  fixed_point_descriptor_id: NamespacedId,
  raw: i64
}

PhysicsVec3V1 {
  unit_id: NamespacedId,
  fixed_point_descriptor_id: NamespacedId,
  x_raw: i64,
  y_raw: i64,
  z_raw: i64
}

PhysicsPoseV1 {
  translation_micrometres: [i64; 3],
  rotation_q1_30: [i32; 4]
}
```

`fixed_point_descriptor_id` resolves in the exact
`AuthoritativeNumericProfileV1`; unit IDs and backend conversion rules resolve
in the exact `PhysicsQuantizationProfileV1`. Public descriptor/state schemas
contain no `f32` or `f64`. A private adapter source may be IEEE binary32 or
binary64 only when an exact SPEC-21 rule decodes it as a rational, normalizes
negative zero, rejects NaN/infinity and applies one
`NearestTiesToEven` conversion with checked `i128` intermediates.

An importer, cooker or adapter starting in another handedness, axis or unit
MUST convert before constructing a public descriptor and bind the conversion
recipe hash to provenance. No implicit centimetre/degree/vendor-unit default
survives validation.

### `PhysicsLimitsProfileV1`

The exact limits profile and hash are part of `ProjectLockV3`,
`PhysicsWorldDescriptorV1`, save/replay compatibility and every canonical
snapshot. A project MAY lower a V1 ceiling but MUST NOT raise it without a new
Accepted architecture decision.

| Limit | V1 hard bound |
|---|---:|
| Physics worlds in one project instance | 64 |
| Bodies in one world | 1,048,576 |
| Shapes on one body | 64 |
| Total shapes in one world | 4,194,304 |
| Materials in one world catalog | 65,536 |
| Joints in one world | 2,097,152 |
| Collision layers | 64 |
| Step input operations per substep | 1,048,576 |
| Query requests per substep | 65,536 |
| Excluded body/shape IDs per query | 4,096 |
| Candidate hits examined per query | 4,194,304 |
| Published hits per query | 4,096 |
| Normalized contact records per substep | 4,194,304 |
| Canonical snapshot decoded bytes | 2,147,483,648 |
| Physics frequency | integer `1..=4,096 Hz`, and an integer multiple of gameplay frequency |
| Positive shape dimension | `0.000001..=8,388,608 m` |
| Dynamic mass | `0.000001..=1,000,000,000 kg` |
| Principal inertia component | `0.000000001..=1,000,000,000,000,000 kg·m²` |
| Linear speed magnitude | `0..=16,384 m/s` |
| Angular speed magnitude | `0..=65,536 rad/s` |
| Acceleration / gravity magnitude | `0..=1,000,000 m/s²` |
| Force or torque magnitude | `0..=1,000,000,000,000,000 N` or `N·m` |
| Impulse magnitude | `0..=1,000,000,000,000 N·s` |
| Query distance | `0..=33,554,432 m` |
| Friction coefficient | `0..=16` |
| Restitution | `0..=1` |
| Linear/angular damping | `0..=1,024 s⁻¹` |

Vector magnitude is checked from exact components with checked integer
arithmetic; checking components alone is insufficient. Count, decoded length,
offset, product, squared magnitude or accumulation overflow rejects the entire
staged descriptor/operation/query/snapshot before allocation or publication.
Runtime-private resource budgets MAY impose a lower capacity result and never
raise these hard bounds.

## Canonical descriptor envelope and identities

All descriptors are `CanonicalBinaryV1` records with
`owner_id = "nextengine.physics"`, an exact registered schema ID and
`segment_id = "v1"`. Unknown fields, unknown enum variants, missing declared
fields, duplicate IDs, noncanonical collection order and trailing bytes are
invalid.

```text
physics_descriptor_sha256 = SHA256(
  "nextengine.physics-descriptor.v1\0"
  || u64_le(descriptor_bytes.len)
  || descriptor_bytes
)
```

The hash is an external envelope/reference value and is never embedded in its
own preimage. Catalogs store `(ID, revision, descriptor_sha256)` alongside the
exact descriptor bytes.

Nominal IDs are:

```text
PhysicsWorldIdV1  = opaque Id128 bound to WorldIdentityManifestV1
PhysicsBodyIdV1   = (subject_id: PersistentId, body_slot: u32)
PhysicsShapeIdV1  = (body_id: PhysicsBodyIdV1, shape_slot: u32)
PhysicsJointIdV1  = (owner_id: PersistentId, joint_slot: u32)
PhysicsMaterialIdV1 = NamespacedId
PhysicsQueryIdV1  = (physics_tick: u64, query_slot: u32, issuer_stream_id: CommandStreamId)
PhysicsFeatureIdV1 = engine-owned canonical feature key from an exact shape revision
```

Slots are zero-based and stable only inside their exact owner descriptor
revision. Durable/cross-record reference uses the complete nominal ID plus
revision/hash where required. Backend body/shape/joint indexes, pointer values,
callback ordinals and `RuntimeEntityId` never substitute for these IDs.

Catalogs sort by full canonical encoded ID bytes. Descriptor revision is a
positive monotonic `u32`; reuse of one `(ID, revision)` with different bytes is
`PHYS_DESCRIPTOR_ID_COLLISION` and fails closed.

## `BodySchema` compiler boundary

ADR-066 `BodySchema` is the immutable heterogeneous Physical Interaction Graph
for an articulated creature. SPEC-26 remains the sole owner of exact runtime physics descriptors;
the relation is a deterministic one-way compiler, not shared mutable storage:

```text
BodySchema revision + BodyInstanceProjection revision
  → ordered PhysicsBodyDescriptorV2[]
  → ordered PhysicsJointDescriptorV1[]
  → attachment/pair-exclusion/actuator profile
  → descriptor catalog roots and topology revision
```

Each schema body/joint/actuator/effector/attachment identity maps through an
explicit canonical table to exact SPEC-26 IDs/slots. Array position, authoring
name, backend index and nearest-role matching are forbidden. The mapping hash,
source schema hash and effective projection hash are part of the compiled
catalog closure used by motor compatibility, snapshot, save and replay.

Compiler input separates immutable static node/edge morphology from revision-bound
equipment/stats/damage/fatigue/attachment overlays. Effective mass/inertia,
CoM, ROM, damping/friction, torque-speed-power, target rate, latency and break/
support limits validate before adapter construction. Backend-computed defaults
or policy-inferred known parameters cannot replace declared values.

On a validated topology transaction, surviving elements retain their stable
BodySchema identities and receive an explicit old→new physics-slot map.
Removed nodes/actuators are absent, new prosthesis/attachment elements require
a declared schema/overlay identity, and every dangling/colliding/ambiguous map
rejects the whole transaction. SPEC-27 then resets or remaps only as explicitly
allowed; physics never interprets evaluator state.

A deterministic morphology cache MAY store derived per-node/per-edge encoder
inputs or embeddings. The compiler emits the canonical SPEC-14
`StaticMorphologyProjection` and `static_morphology_hash` from source schema,
compiled catalog, topology and the profile-declared static subset of instance
overlays. The cache key is `(static_morphology_hash, encoder_profile_hash)`; it
does not bind the full effective-instance revision/hash.

Material morphology/equipment/attachment or persistent actuator lock/restore
rebuilds the cache only when it changes the static projection. Ordinary pose,
contact, velocity, fatigue, available power, transient damage/health and sensor
ticks remain dynamic and cannot change the key. Every profile classifies each
input field as static or dynamic before compilation. The cache is
reconstructible; cache warmth, worker and completion order are not physics or
motor authority.

The semantic compiler boundary is Accepted. ADR-058/SPEC-35 are the first
production physical-character consumer and accept `BodySchemaV1`,
`BodyInstanceProjectionV1`, `PhysicsBodyDescriptorV2`,
`PhysicsJointDescriptorV1`, `PhysicsActuatorDescriptorV1`,
`PhysicsWorldCatalogV2`, `PhysicsStepInputV3`, `PhysicsStepResultV2`,
`PhysicsCanonicalSnapshotV3` and `PhysicsWorldCheckpointV2`. Earlier alpha
versions are unsupported inputs and are not migrated.

The bounded R5d reference consumer additionally compiles the exact
project-activated `BodySchemaAssetV1` twice, once for each stable player/NPC
subject. The compiler validates the complete neutral
`BodyInstanceProjectionV1` against the schema before constructing any adapter
input and returns the descriptor catalog plus `BodyProjectionRootsV1`
atomically. The physics-descriptor root covers every emitted body, shape,
joint and actuator field in canonical identity order; observation/action and
actuator-safety roots bind the other derived consumers. A mismatch publishes
no candidate projection.

This production projection is currently a compatibility/admission witness for
the shared physical-animation archetype. It is not inserted into the active
reference gameplay scene: the existing upright kinematic avatar body remains
the sole transform owner. R5j adds one ordinary fixed-local box shape to that
same body for a bounded carried-load consumer; it does not activate the
BodySchema articulation graph or create another mutable projection. Active
articulation cutover, topology transactions and non-default overlays require
later bounded consumers and their own snapshot/replay evidence.

### Current bounded compound capsule profile

The current reference adapter accepts one solid capsule at identity local
translation and at most one solid identity-rotation box with a fixed local
translation on the same kinematic body. Every other avatar shape family,
duplicate capsule, second box, rotated shape or non-solid participant rejects
before activation. The box:

- uses its catalog `PhysicsShapeIdV1`, canonical moving/obstacle primitive-face
  pair, material/reporting and symmetric layer/mask filter in sweeps and
  contact continuity;
- is checked against static and current dynamic boxes on activation;
- may block horizontal/vertical displacement but cannot establish grounded
  support or invoke capsule step-up;
- reconstructs entirely from the unchanged catalog plus
  `PhysicsCanonicalSnapshotV2` body/contact state.

Dynamic boxes are exact free bodies along the vertical axis (WR1, plan
`continuum-water/10`). The profile accepts up to `16` solid identity-rotation
dynamic boxes, one shape each, with a positive mass. Each physics substep,
before the capsule integrates and in shape-id order, every dynamic box adds
the solver-profile gravity to its vertical velocity (`g / physics_hz`, the
capsule's delta), sweeps `v_y / physics_hz` along `y` against the static
solids, the other dynamic boxes at their current staged poses, the capsule's
axis-aligned bounds (`centre ± [r, half_segment + r, r]`) and the attached
carried boxes (all filtered by layer/mask), moves by the applied delta and
zeroes its vertical velocity when the sweep was cut. Horizontal motion is
push-only: the capsule push sweeps the box against the static solids and the
other dynamic boxes (no chain push) and the box's horizontal velocity is the
applied push times `physics_hz` for that substep. Body revisions bump
exactly when pose or velocity change; boxes never sleep; box-to-support
contacts are not reported through contact continuity. Activation rejects a
dynamic box that penetrates a static solid or another dynamic box it collides
with, and a world without an avatar integrates its boxes the same way.

The production carried-load layer is opt-in: a visible wall owns a coincident
filtered clearance proxy while the ordinary world-layer proxy still serves
the capsule and dynamic push box. The carried collider's `200×300×200` mm
half-extents match the reused visible push-box mesh bounds. This authoring
choice prevents a permanent load from silently changing unrelated quest-route collisions. General
attachment commands, detach/drop state, multiple loads, independent load pose,
mass transfer and articulated constraints are not implied.

## Proposed ephemeral candidate-physics forks

ADR-066 optional high-fidelity skill planning MAY evaluate exactly `K`
candidate `PhysicalActionChunk` values from one validated immutable canonical
checkpoint for one fixed short horizon. This is a private bounded physics-fork
operation, not a public save/world clone, durable replay branch or second
authoritative scene.

Each fork receives the same descriptor/canonical-snapshot plus ADR-059
continuation closure, an isolated content-derived RNG stream when required and
one candidate ID. Rehydration validates the exact starting witness. A private
copy-on-write/native clone is allowed only as a reconstructible optimization
behind that witness and is never snapshot authority. The fork may produce
only a bounded fixed-point score/evidence record; commands, events, saves,
contacts/outcomes for publication, external effects and mutable caches cannot
escape the fork. All `K` forks complete as synchronous logical work and merge
in candidate-ID order. Worker count, completion order and elapsed wall time do
not change score or winner; a wall overrun is conditional performance evidence.

The runtime records candidate-set, checkpoint, scoring-profile, ordered result,
winner and selected-chunk roots. Authoritative replay consumes the recorded
winner/chunk and does not expose `branch` or `step_with`; an independent parity
mode may rerun the bounded fork. General branching replay, arbitrary nested
branches and persisted branch lifecycle remain out of scope. Exact fork/result
schemas remain Proposed until `MOTOR-ROLLOUT-P1` has a production consumer.

## World, material, shape and body descriptors

### `PhysicsWorldDescriptorV1`

```text
PhysicsWorldDescriptorV1 {
  schema_ref,
  world_id,
  descriptor_revision,
  world_identity_manifest_sha256,
  coordinate_profile_sha256,
  tick_rate_profile_sha256,
  authoritative_numeric_profile_sha256,
  physics_quantization_profile_sha256,
  physics_limits_profile_sha256,
  collision_layer_registry_sha256,
  material_combine_profile_sha256,
  solver_semantics_profile_sha256,
  gravity: PhysicsVec3V1,
  allowed_query_stage,
  material_catalog_sha256,
  body_catalog_sha256,
  joint_catalog_sha256
}
```

`solver_semantics_profile_sha256` binds only engine-visible behavior such as
fixed substeps, sleep/activation thresholds, continuous-collision class,
constraint stabilization class and contact reporting obligations. It contains
no iteration enum or native solver setting. A private backend configuration is
supported only when the product check proves the same public behavior.

World activation is:

```text
Absent → Staged → Validated → AdapterConstructed → Active
```

Validation resolves every schema/profile/content/reference hash, checks all
limits, constructs canonical catalog roots and reserves resources before
adapter construction. `Active` is published only at a declared physics commit
boundary after adapter round-trip reports the same descriptor graph. Failure
destroys staging and leaves the prior active world unchanged.

### `PhysicsMaterialDescriptorV1`

```text
PhysicsMaterialDescriptorV1 {
  schema_ref,
  material_id,
  descriptor_revision,
  static_friction,
  dynamic_friction,
  restitution,
  rolling_friction,
  spinning_friction,
  surface_velocity: PhysicsVec3V1,
  canonical_material_tags[]
}
```

All coefficients are exact fixed-point and within the active limits profile.
`dynamic_friction <= static_friction`. Tags are NFC namespaced IDs sorted by
canonical bytes. Physics material is independent of renderer
`NeutralMaterialV1`.

`PhysicsMaterialCombineProfileV1` chooses one closed rule per coefficient:
`Minimum`, `Maximum`, `ArithmeticMeanTiesToEven` or
`ProductTiesToEvenClamped`. The world has one exact combine profile; a backend
cannot substitute its default. Surface velocity is applied in canonical
participant order. Pairwise effective values are converted to exact
quantized integers before contact classification or publication.

ADR-071 records that the implemented current-only alpha V1 omitted rolling
friction, spinning friction and surface velocity from its canonical record.
Those bytes are immutable. `PhysicsMaterialDescriptorV2` therefore wraps that
implemented V1 base and adds the three omitted fields; it is the current
complete material contract for the biomechanics consumer. The exact
`PhysicsMaterialCombineProfileV1` is a separate canonical, hash-bound record.
Old alpha material/compiled records are not migrated or reinterpreted.

### `PhysicsShapeDescriptorV1`

```text
PhysicsShapeDescriptorV1 {
  schema_ref,
  shape_id,
  descriptor_revision,
  local_pose: PhysicsPoseV1,
  geometry: PhysicsGeometryV1,
  material_id,
  collision_layer: u8,
  collision_mask: u64,
  participation: Solid | Sensor | QueryOnly,
  contact_reporting: Disabled | BeginEnd | BeginPersistEnd,
  semantic_tags[]
}
```

`PhysicsGeometryV1` is a closed union:

- `Box { half_extents }`;
- `Sphere { radius }`;
- `Capsule { radius, half_segment }`, whose segment axis is local `+Y`;
- `ConvexHull { collision_asset_revision, shape_index, feature_table_sha256 }`;
- `TriangleMesh { collision_asset_revision, shape_index, feature_table_sha256 }`;
- `HeightField { collision_asset_revision, shape_index, feature_table_sha256 }`.

Every asset reference resolves to exact `NeutralCollisionV1` bytes and a
portable `DomainRelevant` content hash. Convex/mesh/heightfield feature IDs
come from the validated engine-owned canonical feature table, never a backend
face index. Primitive feature IDs use the closed engine mapping for face,
edge, vertex and analytic surface classes.

All dimensions are strictly positive and finite in the declared fixed-point
range. Degenerate hull, invalid triangle/heightfield index, reflection,
runtime nonuniform scale or missing feature mapping rejects the shape.
`TriangleMesh` and `HeightField` are allowed only on `Static` bodies in V1.
`Sensor` emits normalized overlap/contact events but no impulse.
`QueryOnly` participates only in queries and never in simulation contact.

### `PhysicsBodyDescriptorV1`

```text
PhysicsBodyDescriptorV1 {
  schema_ref,
  body_id,
  descriptor_revision,
  motion_kind: Static | Kinematic | Dynamic,
  initial_pose: PhysicsPoseV1,
  initial_linear_velocity: PhysicsVec3V1,
  initial_angular_velocity: PhysicsVec3V1,
  mass_microkilograms,            // current field 9: 1..=10^15 for Dynamic, 0 otherwise
  mass_properties: Option<PhysicsMassPropertiesV1>,
  gravity_scale,
  linear_damping,
  angular_damping,
  maximum_linear_speed,
  maximum_angular_speed,
  activation_policy: AlwaysAwake | MaySleep,
  shape_descriptors[],
  semantic_tags[]
}
```

`PhysicsMassPropertiesV1` contains positive mass, local center-of-mass pose,
three positive principal inertia values and their canonical local orientation.
It is required exactly for `Dynamic`, absent for `Static` and `Kinematic`.
The current `PhysicsBodyDescriptorV1` carries the mass alone as
`mass_microkilograms` (canonical field 9, always encoded): `1..=10^15`
for `Dynamic` bodies, exactly `0` for `Static` and `Kinematic` bodies;
a descriptor outside that rule rejects. The mass is the impulse divisor of
reaction batches (ADR-105) and does not change free fall.
Backend-computed mass/inertia is not authoritative. Shape IDs are unique and
strictly sorted. Initial velocities of `Static` bodies are exact zero.

A kinematic target is a validated future-step request, not an unbounded
teleport or a mutable field in the descriptor. Dynamic transform is written
only by the active physics step. Descriptor mutation creates a new revision
and uses the atomic topology transaction below.

## Collision filtering, pairs and normalized response

`collision_layer` is `0..=63`. A pair is simulation-eligible exactly when:

```text
((shape_a.collision_mask >> shape_b.collision_layer) & 1) == 1
AND
((shape_b.collision_mask >> shape_a.collision_layer) & 1) == 1
AND
neither participant is QueryOnly
AND
the exact joint/pair-exclusion profile does not disable the pair
```

Participant order is the lexical SPEC-05 body key
`(PersistentId bytes, body_slot)`, with `shape_slot` used only as a strict
tie-break for equal body keys. Filter evaluation, material combination and
sensor/solid class use this order. Registration order, backend broadphase pair
order and native filter bits have no authority.

The collision adapter MUST:

1. map each raw candidate to exact body/shape/feature identity;
2. reject an unmappable or duplicate conflicting identity;
3. canonicalize participant order and flip participant-relative vectors when
   order changes;
4. quantize every decision-relevant numeric field through SPEC-21;
5. sort by the complete canonical collision/contact key;
6. deduplicate byte-identical normalized records;
7. apply declared sensor/solid and material response semantics;
8. derive contact continuity and publish only bounded engine-owned records.

Missing required contact telemetry is a backend failure. It cannot be replaced
by animation events, bounding-box guesses or a renderer depth query.

## Joint and constraint descriptors

### `PhysicsJointDescriptorV1`

```text
PhysicsJointDescriptorV1 {
  schema_ref,
  joint_id,
  descriptor_revision,
  endpoint_a: PhysicsBodyIdV1 | WorldAnchor,
  endpoint_b: PhysicsBodyIdV1 | WorldAnchor,
  local_frame_a: PhysicsPoseV1,
  local_frame_b: PhysicsPoseV1,
  kind: Fixed | Hinge | Slider | Ball | SixDof,
  linear_axes: [PhysicsDofDescriptorV1; 3],
  angular_axes: [PhysicsDofDescriptorV1; 3],
  coupled_angular_limit: Option<PhysicsBallLimitV1>,
  actuator_bounds: PhysicsActuatorBoundsV1,
  break_force: Option<PhysicsScalarV1>,
  break_torque: Option<PhysicsScalarV1>,
  collision_between_endpoints: bool,
  semantic_tags[]
}
```

At most one endpoint is `WorldAnchor`; two body endpoints must exist and be
different. Local frames are expressed in each endpoint's canonical body frame.
`PhysicsDofDescriptorV1` is `Locked`, `Free` or
`Limited { minimum, maximum, restitution, damping }`. `minimum <= maximum`;
linear axes use metres and angular axes radians.

Joint `kind` is a validation shorthand for an exact six-axis pattern:

- `Fixed`: all six locked;
- `Hinge`: one declared angular axis free/limited, other five locked;
- `Slider`: one declared linear axis free/limited, other five locked;
- `Ball`: all linear locked, angular axes enabled as declared and exactly one
  `PhysicsBallLimitV1` supplies twist minimum/maximum plus the two positive
  swing-cone half-angles;
- `SixDof`: each axis explicitly declared and `coupled_angular_limit` absent.

`coupled_angular_limit` is absent for `Fixed`, `Hinge` and `Slider`. All
angular bounds are radians in the active fixed-point profile.

`PhysicsActuatorBoundsV1` declares permitted axis mask, maximum target rate,
force/torque, velocity, power, impulse and energy per substep, plus the exact
engine-owned torque-speed/power limit profile where the compiled BodySchema
actuator requires it. It authorizes no action by itself. A motor or gameplay
proposal still passes the ADR-027/ADR-066 safety validator. Fixed PD/SPD gains
are selected by the exact actuator profile; a learned action cannot rewrite
them unless a separately accepted adaptive-gain profile exists.
Target values and current accumulated impulses are state, not immutable joint
descriptor fields.

Break comparison uses only quantized-exact force/torque/impulse fields and
declared exact thresholds. A broken constraint emits a physical outcome
proposal; durable topology changes only through the common validated command
and `PhysicsTopologyTransactionV1`.

```text
PhysicsTopologyTransactionV1 {
  transaction_id,
  expected_world_revision,
  expected_material_catalog_sha256,
  expected_body_catalog_sha256,
  expected_joint_catalog_sha256,
  ordered_operations[],
  resulting_material_catalog_sha256,
  resulting_body_catalog_sha256,
  resulting_joint_catalog_sha256
}
```

Operations sort by `(operation_class, canonical target ID, operation_slot)`.
The complete graph, limits, shape/content references, joint endpoints,
capabilities and resulting roots validate before adapter mutation. Adapter
failure restores the pre-transaction canonical snapshot and publishes no
partial body/joint graph.

SPEC-36 specializes this transaction for authored `BreakSite` variants. A
stable fracture changes no topology. An unstable retained fracture may replace
one declared node with preauthored proximal/distal nodes plus one bounded
retention constraint after exact mass/inertia, shape/filter, ID-remap and
fallback validation. Detachment is a separate transaction. Raw contact,
renderer geometry and backend break callbacks can only propose these outcomes;
they cannot invent a split or publish it directly.

## Fixed-step input and commit

At one physics boundary Runtime provides one immutable
`PhysicsStepInputV1`:

```text
PhysicsStepInputV1 {
  world_id,
  expected_world_revision,
  physics_tick,
  substep,
  topology_transactions[],
  kinematic_targets[],
  external_force_requests[],
  impulse_requests[],
  accepted_motor_actions[]
}
```

`physics_step_input_sha256` is an external domain-separated SHA-256 of the
complete canonical `PhysicsStepInputV1` bytes; it is not a self field.

Each request has a stable causal ID, exact target body/joint, target
tick/substep, profile hash and bounded fixed-point payload. Exact duplicate
causal request deduplicates; same ID with different bytes rejects the complete
step batch. The first four rows below order step collections; the last row
orders the separate closed query batch:

| Collection | Canonical total order |
|---|---|
| Topology transactions | `(transaction_id bytes, resulting catalog roots)` |
| Kinematic targets | `(body_id bytes, causal command_id, operation_slot)` |
| Force/impulse requests | `(body_id bytes, application point bytes, request kind, causal command_id, operation_slot)` |
| Accepted motor actions | `(body_id bytes, policy route ID, motor tick, action slot, action hash)` |
| Query requests | `(physics_tick, query_slot, issuer stream ID bytes, request hash)` |

Step flow is atomic at the public boundary:

1. validate world revision, exact profile/catalog hashes and all request
   preconditions;
2. apply any validated topology transaction on staging;
3. map exact requests to private adapter objects;
4. execute the declared fixed CPU substep without gameplay callback;
5. read raw state/contact candidates;
6. quantize and validate the complete required physical projection;
7. normalize constraints and contacts in canonical order;
8. build the new canonical snapshot/root and contact-derived physical outcome
   proposals;
9. publish one committed physics revision;
10. stage Outcome proposals for the common later validator.

No mutable world access crosses `await`. A private parallel solver MAY run, but
worker completion order is erased before publication. Fatal backend invariant
loss stops the simulation instance and preserves the last valid checkpoint;
continuing from an untrusted partial pose is forbidden.

Authoritative queries are a separate closed `PhysicsQueryBatchV1` executed
only against an already committed snapshot selector. They never observe a
half-stepped or half-published world. Results and any derived proposals are
staged for a future common command boundary; same-tick query re-entry is
forbidden.

## Scene-query contract

### `PhysicsQueryRequestV1`

```text
PhysicsQueryRequestV1 {
  schema_ref,
  query_id,
  world_id,
  snapshot_selector: (physics_tick, substep, physics_snapshot_sha256),
  geometry: PhysicsQueryGeometryV1,
  filter: PhysicsQueryFilterV1,
  cardinality: Any | Closest | All,
  maximum_published_hits
}
```

`physics_query_request_sha256` is an external domain-separated SHA-256 of the
complete canonical request bytes and is used only after schema validation.

`PhysicsQueryGeometryV1` is a closed union:

- `RayCast { origin, unit_direction, maximum_distance }`;
- `ShapeCast { query_shape, origin_pose, unit_direction, maximum_distance }`;
- `Overlap { query_shape, pose }`;
- `ClosestPoint { point, maximum_distance }`.

`query_shape` is a validated primitive or exact shape descriptor revision.
Direction is canonical Q1.30 and must be a canonical unit vector under the
SPEC-24 exact norm bounds. Every distance is nonnegative and within profile.
Irrelevant fields do not exist in a union variant and therefore cannot carry a
hidden default.

`PhysicsQueryFilterV1` declares collision layer/mask, inclusion of
`Solid`/`Sensor`/`QueryOnly`, canonical sorted exclusions and optional
material/tag predicates from a closed engine-owned expression grammar. It
contains no callback, closure, backend query flags or mutable object.

Every result is bound to the exact selected immutable snapshot. Querying
“latest” without tick/substep/root is invalid for authoritative use.
Presentation MAY use a separately classified non-authoritative query, whose
result cannot enter gameplay, contact or state root.

### Query normalization and order

`PhysicsQueryHitV1` contains exact participant/shape/feature identity and
quantized-exact fraction, distance, point and outward normal. A raw backend hit
without stable feature mapping rejects the authoritative query.

For `RayCast` and `ShapeCast`, the complete hit key is:

```text
(
  quantized_fraction,
  quantized_distance,
  target PersistentId bytes,
  body_slot,
  shape_slot,
  feature_id bytes,
  quantized_point,
  quantized_normal
)
```

For `ClosestPoint`, `quantized_distance` is first. For `Overlap`, the key is:

```text
(
  target PersistentId bytes,
  body_slot,
  shape_slot,
  feature_id bytes
)
```

Exact duplicate hits are removed after normalization. The adapter examines
the complete bounded candidate set or fails `PHYS_QUERY_CAPACITY_EXCEEDED`;
backend early-out order cannot choose a hit.

- `Any` publishes only the exact boolean `eligible_hit_count > 0`.
- `Closest` publishes the first canonical hit after complete normalization.
- `All` publishes the first `maximum_published_hits` canonical hits plus exact
  `eligible_hit_count` and `truncated`.

`Overlap` permits only `Any` or `All`; other query kinds permit all three
cardinalities. `maximum_published_hits` is exactly `1` for `Closest`,
`1..=4,096` for `All` and exactly `0` for `Any`. Query output, count,
truncation flag and order are exact across callback, registration, broadphase,
worker and backend allocation permutations.

## Contact event contract

`ContactEventV1` refines the glossary `ContactEvent`:

```text
ContactEventV1 {
  schema_ref,
  contact_id,
  physics_tick,
  substep,
  phase: Begin | Persist | End,
  kind: Solid | Sensor,
  participant_low,
  participant_high,
  contact_feature_id,
  point,
  normal_low_to_high,
  relative_velocity_low_to_high,
  impulse_lower_bound,
  impulse_upper_bound,
  effective_mass,
  canonical_material_tags[],
  source_snapshot_sha256
}
```

Participants are ordered first by the SPEC-05 key
`(PersistentId canonical bytes, body_slot)`; `shape_slot` is a strict
tie-break only when the body keys are equal. Swapping raw participants negates
participant-relative normal/velocity fields before quantization.
`contact_feature_id` includes both exact canonical shape slots/feature mappings
and never derives from a native manifold/contact index.

The normalized contact-record key is exactly the SPEC-05 refinement:

```text
(
  physics_tick,
  substep,
  participant_low,
  participant_high,
  contact_feature_id,
  quantized_point,
  quantized_normal,
  quantized_relative_velocity,
  quantized_impulse_lower_bound,
  quantized_impulse_upper_bound,
  quantized_effective_mass,
  canonical_material_tags
)
```

Byte-identical normalized records deduplicate before continuity
classification. `contact_id` and `Begin`/`Persist`/`End` derive from the
canonical participant/feature continuity state after sorting, not callback
ordinal or raw point lifetime. Published event order is the record key followed
by phase ordinal `Begin < Persist < End` and `contact_id` bytes.
An `End` event carries the last committed quantized record of that continuity
key plus the current tick/substep and `End` phase; it never takes missing or
implementation-defined numeric fields from a backend end callback.

Contact continuity suppresses duplicate repeated-hit/resting-contact
proposals, but does not itself change health, stamina, quest or inventory.
Such a result remains an immutable proposal for the common Outcome batch.

## Exact, quantized-exact and tolerance-only classification

Every physics field and assertion has exactly one class:

| Class | Included values | Normative comparison and allowed use |
|---|---|---|
| `ExactCanonical` | schema/version/profile/content hashes; descriptor bytes/hashes; IDs and revisions; enum classes; body/shape/joint topology; layer/mask/filter decisions; ticks/substeps; counts; canonical order; query cardinality/boolean/truncation; contact phase/ID; snapshot bytes/root; command/event/gameplay result | Byte/integer exact. May drive branch, ordering, state, save/replay and product-check result. Any mismatch is failure. |
| `QuantizedExact` | backend-derived pose, quaternion, velocity, acceleration, force/torque/impulse, mass/inertia observation, joint state, contact point/normal/relative velocity/effective mass, query fraction/distance/point/normal and solver-continuation values after the exact SPEC-21 rule | Resulting fixed-point integers and canonical bytes are exact. May drive branch/order/outcome/hash only after complete successful conversion. |
| `ToleranceDiagnosticOnly` | private pre-quantization backend samples, solver residuals and separately captured runtime/training or candidate-backend correspondence metrics | Never public/durable/authoritative; never an ID/order/branch/outcome/root input; cannot waive an `ExactCanonical` or `QuantizedExact` mismatch. |

A tolerance assertion is valid only when its scenario names the exact field,
source format, unit, sample population, absolute/relative metric, finite
threshold, aggregation and diagnostic output. Implicit epsilon,
platform-default tolerance, unordered float reduction, NaN/infinity and
“close enough” snapshot/contact/query equality are forbidden. A missing
classification is `PHYS_NUMERIC_CLASS_UNDECLARED`.

## Canonical ordering summary

| Subject | Canonical key |
|---|---|
| Materials | `PhysicsMaterialIdV1` canonical bytes, revision |
| Bodies | `PersistentId` bytes, body slot, revision |
| Shapes | owning body key, shape slot, revision |
| Joints | owner `PersistentId` bytes, joint slot, revision |
| Collision pair | lower participant key, higher participant key |
| Query request | physics tick, query slot, issuer stream ID bytes, request hash |
| Query hit | query-kind key defined above |
| Contact record | complete normalized contact key defined above |
| Snapshot tables | material, body, shape, joint, contact-continuity and solver-continuation keys in that order |

An unordered map/set/native container MUST be projected into these orders
before hashing or publication. Duplicate canonical key with different bytes is
a collision and rejects the complete owning transaction/result.

## `PhysicsCanonicalSnapshotV1`

### Snapshot schema

Snapshot is an engine-owned complete physical checkpoint at a closed physics
commit boundary:

```text
PhysicsCanonicalSnapshotV1 {
  schema_ref,
  world_id,
  world_revision,
  physics_tick,
  completed_substep,
  world_descriptor_sha256,
  body_catalog_sha256,
  joint_catalog_sha256,
  material_catalog_sha256,
  topology_revision,
  coordinate_profile_sha256,
  tick_rate_profile_sha256,
  authoritative_numeric_profile_sha256,
  physics_quantization_profile_sha256,
  physics_limits_profile_sha256,
  sorted_body_states[],
  sorted_joint_states[],
  sorted_contact_continuity_states[],
  sorted_solver_continuation_states[]
}
```

`PhysicsBodyStateV1` contains body ID/revision, canonical pose, linear/angular
velocity, activation class, sleep counter and bounded accumulated
force/impulse state required at the boundary. `PhysicsJointStateV1` contains
joint ID/revision, quantized six-axis position/velocity, break state and
accepted target state.

`PhysicsContactContinuityStateV1` contains contact identity, participant/feature
key, last-seen tick/substep and phase state. Under ADR-059, a PhysX TGS profile
MUST NOT claim that hidden per-contact/per-joint accumulated solver impulses
are directly portable when the vendor API cannot export and import them.
`sorted_solver_continuation_states` is empty for that profile; exact hidden
continuation is reconstructed from the Motor-owned bounded reset + post-safety
effort prefix. Opaque solver/island/manifold bytes, native padding, pointer
identity and backend-specific extension fields remain forbidden.

Snapshot may be taken only after all state-mutating input/result batches for
that boundary are closed. A mutable callback, half-applied topology
transaction or unstaged async state result cannot be captured as authority.
A pending query batch is external read-only work: it is neither embedded in
nor allowed to mutate the selected snapshot.

```text
physics_snapshot_bytes =
  CanonicalBinaryV1(
    owner_id = "nextengine.physics",
    schema_id = "nextengine.physics-canonical-snapshot",
    segment_id = "v1",
    value = PhysicsCanonicalSnapshotV1
  )

physics_snapshot_sha256 = SHA256(
  "nextengine.physics-canonical-snapshot.v1\0"
  || u64_le(physics_snapshot_bytes.len)
  || physics_snapshot_bytes
)
```

The snapshot root, descriptor/catalog/profile hashes and associated
`QuantizedPhysicsProjectionV1` root are exact on Windows x86_64 and Linux
x86_64 for the same project/build/config/input. A private native fast
checkpoint MAY exist only as a reconstructible cache validated against the
canonical root. It is not accepted in save, replay, package, script, WIT,
capture job or public API.

### Atomic restore and continuation

Restore performs:

1. decode with exact length/depth/count limits and no trailing bytes;
2. validate schema compatibility, world identity and every project/profile/
   content/descriptor/catalog hash;
3. validate unique IDs, topology, ranges, continuity references and the exact
   bounded replay-prefix closure required by the active backend profile;
4. reserve bounded resources before adapter construction;
5. construct a private world in canonical material/body/shape/joint order;
6. establish the canonical episode-origin reset state;
7. replay each stored post-safety substep effort without PD/policy evaluation;
8. require the exact declared witness at every motor compare point and the
   complete final snapshot/root;
9. atomically replace the target world at a declared commit boundary.

Any failure destroys staging and retains the complete prior world/save
generation. Restore never edits a live world in place. A backend unable to
reconstruct the exact continuation from its declared bounded replay closure
fails `PHYS_SNAPSHOT_RESTORE_FAILED`; the contract is not weakened to accept a
native blob or tolerance.

After restore, the next declared continuation window MUST produce exact
quantized projection, constraint state, contact/query order, physical outcome
and snapshot roots. Raw sample correspondence MAY be reported separately
under declared tolerances but cannot change this result.

### Authoritative water volumes (Proposed ADR-100)

`PhysicsWorldCheckpointV1` schema version `4` (introduced at schema `2` in
R8c, bumped by ADR-103 and ADR-105) carries `WaterVolumeSetV1` as field 4 next to the catalog and the canonical snapshot: at most `64`
disjoint sealed regions, each an exact integer micrometre extent, an
initial level, a swimming depth, an optional authored linear level ramp
and a profile revision, plus one mutable record per volume (record
revision, committed level, ramp suspension). The table is outside the
canonical snapshot and the rigid step never reads it; backends carry it
unchanged and it changes only through the validated
`nextengine.command.water-volume` transaction (event
`nextengine.event.water-volume-changed`) and, for volumes that are cells of
the ADR-103 network, through the exact per-tick flow step, which rewrites
the committed level without a command, an event or a record-revision
change. A cell cannot carry an authored ramp; a checkpoint that binds a
ramped volume as a cell rejects at validation, before activation or
restore.

`WaterVolumeSetV1::submersion_at(point, tick)` is an exact query on the
committed table: the containing volume, `level - y` as depth, and the
`Dry`/`Wading`/`Swimming` class from the authored swimming depth. It is
evaluated only against a committed checkpoint, never against presentation
water, and has no backend, tolerance or float input. The physics checkpoint
hash domain is `nextengine.physics-world-checkpoint.v4`; earlier checkpoint
bytes are current-only alpha artifacts and reject before mutation.

### Authoritative water flow network (Proposed ADR-103)

`PhysicsWorldCheckpointV1` carries `WaterFlowNetworkV1`
as field 5: at most `256` edges over the water-volume cells (open sill,
pipe, gate, pump, source, sink; every coefficient a permille profile
value), one mutable state per edge (record revision, gate opening, pump
switch, rate, last flux), the integration tick rate (`1..=1000` per
second, `0` only for an empty network; it MUST equal the world
`TickRateProfileV1` gameplay rate and the physics owner rejects a mismatch
before activation) and, per cell, one exact stored volume in cubic
millimetres plus the water-volume record revision it was last
synchronised with (an authored `SetLevel` rewrites the committed level and
the volume is recomputed from it at the next step). Cells are the water volumes themselves; the level a
submersion query sees is `floor + volume / area` in integer arithmetic,
saturated at the cell ceiling. The network changes through the validated
`nextengine.command.water-flow` transaction (`SetGate`, `SetPump`,
`SetSource`, event `nextengine.event.water-flow-changed`) and through one
exact Jacobi step per tick inside the physics owner after the rigid step:
integer square roots of the Torricelli and weir laws, fluxes limited by
the water above the sill and half the equalising volume, per-cell
largest-remainder scaling so no cell goes negative, total volume changed
only by sources and sinks. `cell_volume(id)` and `edge_flux(id)` are the
two exact queries. The rigid step never reads the network; backends carry
it unchanged. Flux limits apply per kind: `Open`, `Pipe` and `Gate` fluxes
are bounded by the water above the sill on the source side and by half the
equalising volume; a `Pump` by its source cell's volume; a `Sink` by its
cell's volume; a `Source` is unbounded. Edges evaluate in ascending edge-id
order. The command rejects with six stable codes:
`WATER_FLOW_EDGE_UNKNOWN`, `WATER_FLOW_EDGE_KIND_MISMATCH`,
`WATER_FLOW_REVISION_STALE`, `WATER_FLOW_OPENING_OUT_OF_RANGE` (opening
`> 1000`), `WATER_FLOW_RATE_OUT_OF_RANGE` (rate outside `0..=1 m^3/s`)
and `WATER_FLOW_REVISION_EXHAUSTED`; `SetSource` sets the rate of a
`Source` or a `Sink`. Record bounds: `64` cells, `256` edges, one network
per world, every cell incident to at least one edge, areas `<= 1000 m^2`,
widths `<= 1000 m`, rates `<= 1 m^3/s`, sills and inverts within the
canonical position range. An arithmetic or validation failure inside the
flow step is an invariant fault that stops the physical run with the prior
checkpoint retained. Water motion alone does not advance the world
revision (the rigid snapshot hash is unchanged); it changes the physics
checkpoint hash and therefore every state root.


### Exact-level buoyancy batch (Proposed ADR-105)

`PhysicsWorldCheckpointV1` schema version `4` carries the optional
`WaterBuoyancyProfileV1` as field 6 (`rho_water` in kilograms per cubic
metre, the buoyant gravity magnitude in micrometres per second squared,
`k_damp` in permille per second, the `CanonicalAabb` bounds rule); a world
without the profile computes no batch. Before the rigid step of every
gameplay tick the physics owner computes `WaterBuoyancyBatchV1` from the
water table as staged for that step (the previous tick's flow result plus
any level command committed earlier in the same tick) and the committed
canonical body poses of the previous tick: for every active `Dynamic`
body whose axis-aligned bounds (the union of its box shapes) intersect a
water volume horizontally and lie below its effective level, the displaced
volume is the exact integer volume of the bounds clipped by the cell and by
the level plane (cubic millimetres, the largest cell on a straddle), the
buoyancy impulse is `rho g V / hz` upward at the clipped centroid and the
drag impulse is `-k rho V v / (1000 hz)` per axis with `v` the committed
linear velocity; all in micronewton-seconds and `i128` intermediate
arithmetic, truncating toward zero. Records sort by body id, one per body,
at most `64`; bodies outside every volume receive none.

The batch rides `PhysicsStepInputV2` schema version `3` as
`external_impulses` (field 10; hash domain
`nextengine.physics-step-input.v3`): each `ExternalImpulseV1` binds the
body id, the impulse, the application point and the ADR-081
`WaterExchangeTupleV1` (namespace `nextengine.exchange.water-buoyancy`,
owners water and physics, world id, the committed water table hash and
highest water record revision as the source root/revision, the committed
world revision and snapshot hash as the destination root/revision, tick,
substep `0`, edge profile `nextengine.water-buoyancy.v1`, operation slot
`0`). Validation rejects an unsorted or duplicate body, more than `64`
records, a component beyond `10^15` micronewton-seconds or a tuple that
binds other roots than the input's; the world rejects an impulse on a
body it does not integrate. Both backends apply every impulse exactly once
at the first substep: the dynamic box's vertical velocity changes by
`J_y / m` (micrometres per second, truncated) before that substep's
gravity; horizontal components have no effect in the push-only profile.
The batch is replayed and hashed with the step input, reads no
presentation state and never feeds back into the water of the tick it
was computed from.
## Persistence, replay and schema evolution

Persistence stores the canonical snapshot/root, exact descriptor/catalog/
profile/content hashes and, where ADR-059 applies, a content-addressed bounded
reset/effort-prefix owner segment in one atomic save closure. Immutable
collision assets remain in content storage; backend caches, query scratch,
worker state and native snapshot bytes are not saved.

Replay binds each `PhysicsStepInputV1`, topology transaction, canonical query
batch, contact event batch, physical outcome proposal and resulting snapshot/
projection root. First divergence identifies physics tick, substep, subject/
body/shape/joint/query/contact ID, field class and first differing canonical
value. It does not retry with a looser tolerance.

Schema changes follow SPEC-22/ADR-025:

- the schema registry admits only monotonic version and stable field identity;
- incompatible descriptor or snapshot change requires an explicit
  deterministic copy-on-write migration;
- migration operates on the complete snapshot/catalog closure and publishes
  one new generation only after exact revalidation;
- source save/snapshot remains immutable;
- unknown field/variant/profile/content hash fails before world mutation.

`ProjectLockV3` and launch profile bind the exact coordinate,
numeric, limits, solver-semantics, descriptor and content closure. Runtime
does not resolve a floating backend or physics profile from ambient machine
state.

## Stable diagnostics and failure semantics

| Code | Required result |
|---|---|
| `PHYS_DESCRIPTOR_INVALID` | Reject the complete descriptor/catalog before adapter construction; preserve prior world. |
| `PHYS_DESCRIPTOR_ID_COLLISION` | Fail closed and quarantine conflicting revision; never rename, salt or choose by arrival order. |
| `PHYS_UNIT_AXIS_MISMATCH` | Reject before conversion/publication; require an exact declared conversion/provenance recipe. |
| `PHYS_LIMIT_EXCEEDED` | Reject before allocation or step mutation; checked overflow is not clamped unless the schema explicitly defines a pre-operation clamp. |
| `PHYS_NUMERIC_CLASS_UNDECLARED` | Reject schema/scenario; an unclassified field cannot enter authority or tolerance comparison. |
| `PHYS_REFERENCE_INVALID` | Reject the complete world/topology transaction; no dangling material/body/shape/joint/content reference. |
| `PHYS_COLLISION_FILTER_INVALID` | Reject shape/world descriptor or contact batch; retain prior filter/catalog. |
| `PHYS_FEATURE_MAPPING_INVALID` | Reject authoritative collision/query/contact result; backend feature ID cannot leak or be guessed. |
| `PHYS_JOINT_INVALID` | Abort the complete topology transaction and restore prior canonical snapshot. |
| `PHYS_QUERY_INVALID` | Reject the complete query request/result; no partial hit list enters gameplay. |
| `PHYS_QUERY_CAPACITY_EXCEEDED` | Fail the query before authoritative result; never accept backend early-out order. |
| `PHYS_CONTACT_CAPACITY_EXCEEDED` | Abort the uncommitted substep/result publication; preserve last valid checkpoint. |
| `PHYS_NONFINITE_VALUE` | Reject adapter result/action before projection or actuation; no tolerance or saturation. |
| `PHYS_SNAPSHOT_INCOMPATIBLE` | Reject before adapter construction; retain source snapshot and prior active generation. |
| `PHYS_SNAPSHOT_RESTORE_FAILED` | Destroy staging, retain prior active world/save and reject that backend/profile. |
| `NONDETERMINISTIC_RESULT` | Stop at first exact or quantized-exact divergence; no retry-to-green or tolerance waiver. |

Backend fatal/invariant loss stops the simulation instance with a bounded
diagnostic/crash capsule. Optional alternative backend may run the same full
contract later, but no automatic mid-step backend switch, approximate contact
or partial snapshot continuation is allowed.

## Product checks

| ID | Сценарий | Ожидаемый результат | Fallback |
|---|---|---|---|
| `PHYS-API-P1` | `physics-api-contract-v1` on Windows/Linux, 10 000 permutations and N−1/N/N+1 limits | canonical descriptor bytes/hashes/roots are exact; every invalid schema, ID, revision, unit, bound, reference or forbidden public type rejects before mutation | reject descriptor/backend and retain prior exact world/project lock |
| `PHYS-COLLISION-P1` | `physics-collision-contact-v1`, 100 000 substeps and 10 000 order permutations | filter/material decisions, contact bytes/IDs/phases/order and roots are exact on both targets; every mapping, capacity and non-finite fault rejects atomically | abort uncommitted substep and retain last canonical checkpoint |
| `PHYS-JOINT-P1` | all joint kinds/axis modes, BodySchema compiler mappings and 10 000 topology/snapshot cycles | descriptor/topology/joint/projection roots exact; stale endpoint, invalid stable-ID map, frame/limit/mask/bound and unsafe action produce no partial graph | restore pre-transaction snapshot and retain prior topology/route |
| `PHYS-QUERY-P1` | every query kind/cardinality, N−1/N/N+1 capacities and 10 000 order permutations | boolean/count/truncation/hit bytes/order/root exact on Windows/Linux; invalid or over-capacity query publishes no partial result | reject the complete query and use only a separately declared deterministic fallback |
| `PHYS-SNAPSHOT-P1` | 1 000 checkpoint restores with 100-substep continuation across `game`, `headless`, `capture-worker`, worker counts and Windows/Linux | canonical snapshot/projection roots, joint state, query/contact order and outcomes byte-identical; faults expose only complete prior or restored world | retain prior valid checkpoint/save and reject incompatible backend/profile |
| `MOTOR-ROLLOUT-P1` (future) | `K` candidate chunks from one checkpoint under candidate/worker/completion permutations | exact ordered score/evidence roots, winner and selected chunk; zero command/event/save/RNG/cache side effect escapes a fork | reject rollout profile and retain declared base chunk/procedural route |
| `CONTINUUM-WATER-VOLUME-P1` (Proposed ADR-100) | `xtask water-volume`: reference basin probes, level command, rejections, checkpoint round trip, restore/continue, repeated generation | exact probe table and roots; rejections leave the table unchanged; restored and live runs converge | reject the region before activation and keep the dry scene |
| `CONTINUUM-WATER-FLOW-P1` (Proposed ADR-103) | `xtask water-flow`: two reference vessels joined by a gated pipe with a source and a sink, stepped for a bounded run with a gate closure, a mid-run save, restore, continuation and rejections | exact volume conservation at every tick, drain within twice the analytic Torricelli time, gate response within one tick, identical live and restored roots, stable rejections | keep the network absent and the authored levels (R8c behaviour) |

These checks cover the lower-level descriptor, collision, joint, query and
snapshot contracts consumed by `PHYS-P1`…`PHYS-P8` and `NUMERIC-P1`.

## Requirements

| ID | Technical requirement | Product checks |
|---|---|---|
| REQ-128 | Every physics world, material, shape and body MUST use bounded versioned engine-owned descriptors, canonical units/right-handed axes and exact IDs/hashes, with no ECS, OS, importer or vendor/backend public type. | PHYS-API-P1 |
| REQ-129 | Collision filtering, material combination, shape-feature mapping and `ContactEventV1` continuity MUST be backend-independent, bounded and published in complete canonical participant/feature/value order. | PHYS-COLLISION-P1 |
| REQ-130 | Joint graphs and authoritative scene queries MUST use closed bounded descriptors, exact revisions/snapshot selectors, atomic mutation and complete canonical ordering; callers MUST NOT access backend handles or partial results. | PHYS-JOINT-P1, PHYS-QUERY-P1 |
| REQ-131 | Every authoritative field MUST be `ExactCanonical`, `QuantizedExact` or `ToleranceDiagnosticOnly`; active save/replay MUST use portable `PhysicsCanonicalSnapshotV2` inside `PhysicsWorldCheckpointV1` (schema version 3, carrying the water table and the flow network) with exact continuation across composition roots and shipping targets. | PHYS-SNAPSHOT-P1, CONTINUUM-WATER-VOLUME-P1, CONTINUUM-WATER-FLOW-P1 |

## Failure paths

| ID | Trigger | Required behavior | Product checks |
|---|---|---|---|
| FAIL-052 | Unknown/stale/duplicate descriptor; invalid schema/reference/unit/axis/range/count/filter/feature/frame/limit; forbidden public backend type | Reject the complete descriptor/catalog/topology/query before adapter mutation, emit stable first-cause diagnostic and preserve prior world/root. | PHYS-API-P1, PHYS-COLLISION-P1, PHYS-JOINT-P1, PHYS-QUERY-P1 |
| FAIL-053 | Non-finite backend value, missing mapping/contact, order divergence, exact mismatch, corrupt snapshot or continuation-root mismatch | Abort the uncommitted step/query/restore, preserve last valid checkpoint and report `NONDETERMINISTIC_RESULT`; retry, tolerance and native snapshots cannot waive it. | PHYS-COLLISION-P1, PHYS-JOINT-P1, PHYS-QUERY-P1, PHYS-SNAPSHOT-P1 |

## Production technology profile

ADR-058 selects static CPU PhysX 5.9.0 as the sole production implementation
of these engine-owned contracts. Vendor types and native feature IDs remain
private, and canonical state/contact/snapshot construction remains Rust-owned.
Jolt, Bullet and the old reference solver are not production fallbacks.

The locked scene uses TGS, enhanced determinism, declared iterations,
tolerances, broadphase/friction settings and one solver worker. Independent
environment scenes MAY run in parallel. `PHYS-COLLISION-P1`, `PHYS-JOINT-P1`
and `PHYS-SNAPSHOT-P1` exercise only the PhysX path; GPU PhysX belongs to the
separate SPEC-35 correspondence mirror.
