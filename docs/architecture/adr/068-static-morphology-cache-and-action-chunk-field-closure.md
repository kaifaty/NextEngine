# ADR-068: Static morphology cache and action-chunk field closure

| Field | Value |
|---|---|
| ID | ADR-068 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-12 |
| Last verified | 2026-08-12 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](../28-skeletal-animation-retargeting-and-ik.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Supersedes | Narrowly supersedes ADR-066 cache identity over the full effective-instance projection and closes the single Proposed `PhysicalActionChunk` field set. No current wire schema or learned route is promoted. |
| Superseded by | none |

## Context

ADR-066 separated static morphology from per-tick pose/contact/fatigue state,
but its cache key still named the complete effective instance. Because
`BodyInstanceProjectionV1` carries fatigue and other dynamic owner revisions,
an implementation would either rebuild every tick or use a key that no longer
matched its declared source.

SPEC-28 also listed keypoint/pose trajectories inside
`PhysicalActionChunkV1`, while ADR-066 and SPEC-14 closed the chunk over
root/CoM, effector/object, contact, force and support fields. That created two
incompatible future wire/hash interpretations.

## Decision

### Dedicated static morphology projection

Before a graph encoder may cache morphology, its exact profile defines one
canonical `StaticMorphologyProjection` containing:

- BodySchema and compiled descriptor/catalog hashes;
- topology revision/hash;
- ordered static overlay entries
  `(owner_id, field_id, source_revision, value_hash)`;
- the profile hash that classifies every input field as static or dynamic.

Static entries include morphology parameters, material mass/inertia/geometry
equipment or attachments, and persistent joint/actuator configuration such as
a committed lock/restore. Pose, velocity, contacts, ordinary fatigue,
transient health/damage, available power and sensor validity are dynamic and
excluded. A field is static or dynamic in one exact profile, never both.

```text
static_morphology_hash = SHA256(
  "nextengine.static-morphology-projection.v1\0"
  || u64_le(static_projection_bytes.len)
  || static_projection_bytes
)

cache_key = (static_morphology_hash, encoder_profile_hash)
```

Only a change to this key rebuilds the reconstructible cache. The full
`BodyInstanceProjectionV1` revision/hash is not a cache-key component.

### Single action-chunk field set

The Proposed `PhysicalActionChunk` contains only the profile-selected subset
of:

- root and center-of-mass trajectories;
- ordered effector and object-relative trajectories;
- selected contact schedule and support transitions;
- desired force direction/range and allow-sliding semantics;
- phase/style/proficiency, interruption/cancel metadata, exact validity ticks
  and chunk hash.

Pose/keypoint values may remain offline motion-inpainting conditions and loss
targets. They compile into the structured fields above before runtime
publication and do not add pose/keypoint chunk wire fields.

## Consequences

- Fatigue/contact ticks cannot spuriously invalidate a static morphology cache.
- Structural/material changes deterministically create a new cache key.
- SPEC-14/26/27 use the same cache identity; SPEC-28/34 use the same chunk
  field set as ADR-066.
- Exact projection/chunk schemas remain Proposed until a production consumer
  exists under ADR-046; current Stage 0 fixed tensors do not change.

## Product checks

Future variable-morphology promotion must add cache hit/invalidation vectors
for dynamic ticks versus structural changes to `MOTOR-MORPHOLOGY-TRANSFER-P1`.
Future chunk promotion must add one canonical encode/hash corpus shared by
authored, procedural and learned producers to `MOTOR-SKILL-CHUNK-P1`.

## Rollback

Before a production consumer exists, revert the Proposed profile as one unit.
After promotion, never reuse a projection/chunk version with a different field
classification or field set.
