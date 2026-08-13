# Humanoid safety and contact profile V2

| Field | Value |
|---|---|
| Status | Candidate-local `TRAIN-3` correspondence input |
| Candidate | `HumanoidFlatRecoveryCandidateV1` |
| BodySchema | `nextengine.body.humanoid-biomechanics-raja-1700.v2@2` |
| BodySchema hash | `e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d` |
| Physics / motor cadence | `240 Hz / 60 Hz`, exactly four ordered physics substeps per motor tick |
| Predecessor | [Humanoid safety and contact profile V1](2026-08-12-humanoid-safety-contact-profile-v1.md), SHA-256 `ad20d7a4abd5cc8b59069ecdb59161499ce7754953cbff2477f2850395adb42c` |
| ML authorization scope | Flat locomotion only; brace/fall and get-up remain quarantined |

This profile is a fail-closed correspondence revision of V1. It exists because
the PhysX CPU contact callback exposes shape pairs while the Isaac GPU tensor
API exposes rigid-body pairs. Training must not start with two classifiers that
can assign different identities, continuity, or impact limits to the same
physical contact.

Except for the contact identity and role projection defined below, every
numeric value, fixed-PD rule, hard-ROM tolerance, terminal priority, reset
rule, integer comparison and freeze condition from the predecessor is inherited
unchanged. This document plus the identified predecessor is the complete V2
profile. A byte change to either document creates a new input lineage and
invalidates `TRAIN-3` and every downstream artifact.

## 1. Normative rigid-body projection

Every collider endpoint is first validated against the compiled BodySchema
shape-token to actor-token mapping. Unknown actors, unknown shapes, mismatched
actor/shape endpoints, ground/ground pairs, nonzero ground shape tokens, and
two shapes belonging to the same dynamic actor are rejected before classifier
state changes.

For each dynamic actor that owns one or more colliders, derive exactly one
representative collider by taking the lexicographic minimum of:

```text
(hard_impact_limit(role), role_ordinal, shape_token)
```

Here `hard_impact_limit` and `role_ordinal` are the exact V1 values and the
`BodyContactRoleV2` wire ordinal. This makes a shared rigid body inherit its
strictest impact budget. Equal-budget roles resolve deterministically without
depending on descriptor traversal order.

For the frozen humanoid BodySchema the multi-collider projections are:

| Rigid body | Authored collider roles | Representative role | Consequence |
|---|---|---|---|
| `torso-yaw` | `TorsoGround`, `HeadGround` | `HeadGround` | the complete body uses the strict `1.00 N·s` limit |
| `left-knee`, `right-knee` | `ShankGround`, `KneeGround` | `ShankGround` | equal `4.00 N·s` limit; get-up knee support is not claimed |
| `left-elbow`, `right-elbow` | `ForearmGround`, `HandGround` | `ForearmGround` | equal `3.00 N·s` limit; brace support remains conservative and equivalent |

Every other contact-bearing rigid body has one collider role and therefore
projects to itself. The fixed ground remains actor token `1`, shape token `0`.

## 2. Canonical contact reduction

After endpoint validation, replace each dynamic endpoint shape token and role
with that actor's representative. Canonically order the resulting body
endpoints. During one 240 Hz substep, reduce all PhysX contact points and all
authored shapes for that canonical actor pair to one record:

```text
body_pair_impulse = checked vector sum of oriented point impulses
body_pair_minimum_separation = minimum separation of every point
```

The existing `ContactPairKeyV1` wire fields remain unchanged, but their shape
fields contain representative shape tokens. Classification and continuity
roots keep their V1 domain separators and include this V2 profile hash.
Continuity is therefore owned by the rigid-body pair, not by a transient
choice of collider or contact point.

Active contact, material contact, grace and continuity break remain exactly:

| Parameter | Exact value |
|---|---:|
| Active impulse | `50,000 µN·s` (`0.05 N·s`) or any negative separation |
| Brush ceiling | material when impulse is strictly above `250,000 µN·s` |
| Low-impulse grace | `4` consecutive physics substeps; the fifth is material |
| Continuity break | one inactive physics substep |

Impulse magnitude uses squared integer Euclidean components. A hard-impact
budget is evaluated on the fully aggregated body-pair impulse before any skill
allowance and has no grace.

## 3. Classification and authorization

Locomotion semantics are unchanged after projection: foot bodies are sole
support; a material ground contact by any other representative role is
`ForbiddenLocomotion`; a material dynamic-body pair is
`SelfCollisionViolation`; every impact over the strictest endpoint budget is
`HardImpactViolation`.

Brace/fall and get-up enum values remain available for deterministic offline
reports, but V2 does not authorize a recovery optimizer, checkpoint, or gate.
In particular, the shared shank/knee body cannot prove that the knee collider
was the contacting shape through the Isaac GPU boundary, so V2 never claims
`GetUpSupport` for it. A later recovery profile requires either a body/schema
split or a separately proven exact contact boundary.

At the 60 Hz commit boundary, terminal priority is inherited exactly from V1:
non-finite state, joint safety, contact impact, self-collision, forbidden
locomotion contact, world bounds, fall, then timeout. All four ordered contact
frames participate and reset atomically clears continuity and terminal state.

## 4. Correspondence acceptance

`TRAIN-3` may close only when all of the following carry this document hash:

- the native Rust classifier and independently implemented Python mirror;
- regenerated contact/terminal golden scenarios;
- the Isaac GPU body-pair tensor classifier at every 240 Hz substep;
- reset, continuity, terminal-priority and strict-limit tests;
- an optimizer-free Isaac phase audit bound to one immutable tracker profile.

Until that evidence exists, training authorization is false. Learned-quality
metrics remain report-only and cannot waive a safety termination.
