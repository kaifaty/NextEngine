# ADR-069: Biomechanics BodySchema V2 and explicit solver projection

| Field | Value |
|---|---|
| ID | ADR-069 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-12 |
| Last verified | 2026-08-12 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-067](067-stage0-profile-identity-and-curriculum-hash-closure.md) |
| Supersedes | Introduces a distinct current-only `BodySchemaV2` for the biomechanics training generation. It does not mutate or reinterpret `nextengine.body.humanoid-stage0.v1` or any V1 environment/checkpoint identity. |
| Superseded by | none |

## Context

The fixed Stage 0 `BodySchemaV1` was deliberately small. It can encode one
positive mass and three axis-aligned inertia values per articulation link, one
hard interval per hinge and a symmetric effort/rate bound. Its compiler also
assumes a centred single collider and one principal X hinge axis. Those facts
are sufficient for its frozen experiment but cannot express the accepted
TRAIN-1 biomechanics consumer without parallel Python authority.

The new consumer needs all of the following to be hash-bound before any ML:

- physical source bodies separated from serial-axis solver carriers;
- source full inertia tensors and an explicit PhysX-representable projection;
- multiple collider poses, collision filtering and contact roles;
- anatomical axes/frames, hard and inner soft ROM and intentional mirror signs;
- fixed PD, effort, effort-rate, velocity, power, positive-work, residual and
  target-slew bounds.

Changing the V1 record would invalidate the immutable hashes closed by
ADR-067. Keeping the extra facts in a training script would violate the single
BodySchema authority required by SPEC-14, SPEC-27 and ADR-058.

## Decision

### Separate current-only record

Add `BodySchemaV2` as a new discriminated record. V1 remains readable and
byte-identical for its exact historical consumers; there is no V1-to-V2
checkpoint migration and no fallback that fills missing V2 values from PhysX,
Python or shared constants.

The V2 closure contains strictly ordered body, joint, actuator, collider,
effector, symmetry, capability and declared self-collision-exclusion records.
Every record has a stable schema-scoped ID. Its canonical hash includes the
coordinate profile, source/provenance manifest hash and solver-projection
profile hash in addition to the complete numeric tables.

### Body and solver-mass records

Every V2 body row declares:

- parent, local neutral bind pose, semantic role and source-mapping group;
- exact positive mass and local CoM;
- authoritative symmetric local inertia tensor ordered
  `(xx, xy, xz, yy, yz, zz)` in signed fixed-point SI units;
- a positive solver mass, local solver CoM pose, three positive principal
  inertia values and principal-frame orientation;
- the exact component-wise tolerance under which reconstructing the solver
  tensor must match the authoritative tensor;
- one or more colliders, or an explicit `NonCollidingCarrier` role.

An authored source-mass split may introduce positive serial-axis carriers only
through a named projection group. The group fixes carrier mass/inertia/CoM and
the compensating physical-link values. Validation sums the group back to its
source mass, first moment and inertia tensor within its declared integer
quantization bounds. Unexplained epsilon mass, backend auto-mass and a
zero-mass value silently replaced by the adapter are invalid.

For the first biomechanics candidate, every serial carrier is `0.001 kg` with
`0.000001 kg*m^2` isotropic inertia and shares the source segment CoM at the
neutral pose. The same values are subtracted from its physical link. This is a
solver-stability projection, not an anatomical claim. Changing it creates a
new BodySchema and invalidates downstream training.

### Quantized rotations and axes

V2 pose quaternions and unit axes retain signed Q1.30 storage but validate by
an exact integer squared-norm band rather than the V1 equality that admits only
degenerate power-of-two tuples. The permitted deviation is part of the numeric
profile and canonical hash; sign canonicalization chooses the first non-zero
quaternion component positive. Float conversion occurs only after validation
and uses the locked SPEC-21 conversion rule. A vector outside the band is
rejected, never normalized implicitly by a backend.

### Colliders and contact semantics

Each collider row owns geometry, local pose, material, collision layer/mask,
participation, reporting mode and one closed contact role. Endpoint and
non-adjacent self-collision exclusions are explicit sorted body pairs. A
`NonCollidingCarrier` has no support/contact role and compiles to no simulation
shape; it may not receive a hidden sphere or default filter.

Sole support features are stable effectors on the exact foot body. Head, hand,
forearm, knee, pelvis and torso contacts remain distinguishable even when a
source body was rigidly merged.

### Joint and actuator safety closure

Every one-DoF V2 joint owns semantic axis, parent/child frames, hard minimum
and maximum, inner soft minimum and maximum, neutral value, maximum velocity
and an explicit left/right mirror rule. Validation requires

```text
hard_min <= soft_min <= neutral <= soft_max <= hard_max
```

unless a separately tagged non-neutral joint supplies its own exact ordering.
Knee and elbow profiles in this generation prohibit anatomical
hyperextension with non-negative flexion ranges.

Every actuator owns fixed `Kp/Kd`, negative/positive effort bounds, effort-rate,
power and positive-work-per-motor-tick bounds, residual scale and negative/
positive applied-target delta per 60 Hz tick. These limits apply to reference
targets and policy residuals alike. Unsupported or intentionally unused energy
semantics are tagged explicitly; zero does not mean unlimited.

## Consequences

- TRAIN-1 can freeze one complete engine-owned physical/safety table before
  compiler work, while the Stage 0 V1 hashes remain unchanged.
- TRAIN-2 must compile V2 full mass, collider and axis/frame facts and prove
  the declared solver projection; it cannot reuse the V1 centred-sphere/X-axis
  assumptions.
- TRAIN-3 reads soft ROM, contact roles and action bounds from the same V2 hash.
- Any V2 numeric-table change creates a new schema hash and invalidates every
  dependent corpus, run, checkpoint and evaluation.
- This decision accepts a contract needed by a concrete training consumer. It
  does not accept a learned runtime route, promote SPEC-34 or prove candidate
  quality.

## Product checks

- `BODY-SCHEMA-P2` covers canonical ordering, source-group conservation,
  inertia positivity/projection, rotations, hard/soft ranges, mirror closure,
  collider/filter roles and all negative cases.
- `PHYS-JOINT-P2` covers every joint axis/frame/limit and solver projection in
  the production PhysX adapter.
- `MODEL-MIRROR-P2` requires the Isaac descriptor to be generated from the V2
  canonical descriptor with no duplicate physical constants.
- Existing P1 vectors continue asserting the frozen V1 bytes and hashes.

## Rollback

Before any V2 candidate is published, remove the entire V2 generation and its
consumer together; keep V1 untouched. After publication, never relabel V2
bytes or solver projection with a previous schema ID/revision/hash. A failed
biomechanics candidate falls back at the product routing layer, not by loading
a V1 checkpoint into V2.
