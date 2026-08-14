# ADR-071: Canonical physics-material lineage

| Field | Value |
|---|---|
| ID | ADR-071 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-14 |
| Last verified | 2026-08-14 |
| Normative dependencies | [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-070](070-biomechanics-reference-tracking-training-environment.md) |
| Supersedes | Completes the current biomechanics material lineage without changing the bytes or meaning of the incomplete implemented `PhysicsMaterialDescriptorV1`, `CompiledBodySchemaV2` or biomechanics mirror V1. |
| Superseded by | none |

## Context

The accepted material model in SPEC-26 includes static, dynamic, rolling and
spinning friction, restitution, surface velocity and one explicit combine
profile. The implemented `PhysicsMaterialDescriptorV1` contains only static
and dynamic friction plus restitution. The biomechanics compiler resolves
collider material IDs but does not compile their coefficient rows or a combine
profile. Its native PhysX bridge consequently constructs one ambient material
from C++ literals, while the derived USD lineage has no material prims or
bindings.

TRAIN-4 R109 therefore stopped before any dynamics solve. R110 v2 selected a
bounded repair: three semantic materials with exact Q16 coefficients
`52429/45875/0`, zero rolling friction, spinning friction and surface velocity,
and arithmetic-mean combine rules. The three current coefficient rows are
identical, so the current native generation may use one shared `PxMaterial`
only after exact closure and equality checks.

Changing the implemented V1 record or the V2 compiled descriptor in place
would make historical artifacts appear to have a model lineage they never
contained.

## Decision

### Current contracts and version boundary

Add current-only alpha `PhysicsMaterialDescriptorV2` as a successor which
retains the implemented V1 base and adds the omitted rolling friction,
spinning friction and surface velocity fields. Add
`PhysicsMaterialCombineProfileV1` with one closed rule for every coefficient
and canonical participant ordering for surface velocity.

The biomechanics compiler emits `CompiledBodySchemaV3`. It wraps the immutable
V2 result and hash-binds:

- the complete, canonically ordered material table;
- the exact combine profile;
- the ground material ID;
- resolution of every collider material ID;
- the native shared-material projection.

`BodySchemaV2` and its hash do not change. `CompiledBodySchemaV2`, the
biomechanics mirror V1 and their tracked golden bytes remain unchanged and
unsupported as evidence of the repaired material model. There is no alpha
migration from those records: a consumer selects the current successor or
fails before adapter construction.

### Current biomechanics material profile

The current table contains exactly these IDs:

- `physics-material.humanoid-body.v1`;
- `physics-material.humanoid-ground.v1`;
- `physics-material.humanoid-sole.v1`.

Every row has descriptor revision `1`, static friction Q16 `52429`, dynamic
friction Q16 `45875`, restitution Q16 `0`, rolling friction Q16 `0`, spinning
friction Q16 `0`, surface velocity `[0, 0, 0]` micrometres per second and no
tags. The combine profile
`nextengine.physics-material-combine.humanoid-motor.v1@1` uses
`ArithmeticMeanTiesToEven` for all five scalar coefficients and
`CanonicalParticipantOrder` for surface velocity.

The compiler rejects an unresolved material ID. The current PhysX adapter also
rejects unequal coefficient rows, any nonzero extended field or any combine
rule outside this exact profile. Per-shape unequal material selection and
nonzero rolling, spinning or surface velocity require a later bridge ABI and a
new compiled lineage; silent collapse is forbidden.

### Native boundary

Bridge ABI 4 removes ambient material construction from world creation. Scene
construction requires an explicit fixed-layout material input first. The
current biomechanics route converts the exact Q16 integers to `PxMaterial`,
sets the supported average friction/restitution modes and explicitly retains
zero torsional patch radii. Zero surface velocity needs no contact-modification
callback.

Legacy Stage 0 routes explicitly request their frozen legacy material profile;
it is no longer a backend default. This preserves their prior bytes and
behavior while making that profile unreachable from the V3 biomechanics
lineage.

The Q16 values differ slightly from the old C++ `f32` literals. No old-runtime
equivalence claim is made; later correspondence evidence must use the new
compiled and translation identities.

### Translation boundary

The successor biomechanics mirror is V2 and includes the full material table,
combine profile and ground assignment. Derived USD/Isaac material prims and
bindings are a separate implementation step. Until that successor translation
exists and a clean identity preflight passes, PhysX evaluation, optimization,
candidate construction and training remain blocked.

## Product checks

- `fast` covers contract round trips, canonical hash sensitivity, compiler
  closure, legacy-golden immutability, adapter equality/zero-extension checks
  and ABI layout/version checks.
- `MODEL-MIRROR-P2` later covers exact V2 mirror-to-derived-USD material and
  ground correspondence.
- No R111 check executes a PhysX scene. A later bounded correspondence step
  owns native behavior evidence for the new exact coefficients.

## Rollback

Remove the V2 material/combine contracts, V3 compiled/mirror route and ABI 4
material input as one unit. Retain R109 `STOP_INVALID_MODEL_LINEAGE`, the
immutable V1/V2 artifacts and the old training prohibition. Never relabel an
old compiled descriptor, mirror, USD or checkpoint with the repaired lineage.
