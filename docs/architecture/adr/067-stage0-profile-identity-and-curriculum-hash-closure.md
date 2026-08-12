# ADR-067: Stage 0 profile identity and curriculum hash closure

| Field | Value |
|---|---|
| ID | ADR-067 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-12 |
| Last verified | 2026-08-12 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-064](064-canonical-flat-command-locomotion-environment.md), [ADR-065](065-curriculum-flat-command-locomotion-profile.md) |
| Supersedes | Narrowly supersedes ADR-064/065 wherever their fixed-body or hash wording could permit V1 BodySchema drift, an unbound curriculum translator, or an implicit moving/stop deadband. All other environment, PhysX authority, reward and claim boundaries remain Accepted. |
| Superseded by | none |

## Context

Standing and flat-command V1 runs already bind exact BodySchema and environment
manifest hashes. Raising the authored pelvis from `1.050 m` to `1.095 m` while
incrementing the same schema revision changed those identities and made old
checkpoints unrestorable. Separately, the shared mirror descriptor named USD
translator v3 while every environment manifest still hashed translator v2.

Curriculum support shaping also used an undocumented magnitude threshold over
linear velocity and yaw-rate values with different units. Early rate-limited
nonzero commands could therefore be classified as stop, and that threshold was
absent from the reward hash.

## Decision

### Immutable V1 body and manifests

The current V1 body is `nextengine.body.humanoid-stage0.v1`, schema revision
`1`, with authored pelvis root `1.050 m` and canonical body hash
`13f01daf349cddd84f6c3a068cf9da9ff1307a727949ffa9232ec2e02cbc9bd5`.
Standing and flat-command V1 manifest hashes remain respectively:

```text
dc64488393a57a07e993fdf062f663c8024fc1475267dbf6e8c35f5458f45396
dd5e392d482c43f94fadb1eb1e7b484c9bd73410d092fd3586b35ff0feb074d8
```

No anatomy, root height, collider, mass, joint or actuator correction may alter
that closure. A corrected biomechanics body receives a distinct BodySchema
identity and a new environment/training generation before use. It does not
resume a V1 run.

### Per-profile translator closure

Standing and flat-command V1 retain their historical
`nextengine.isaac-translator.v2` profile hash. Curriculum V2 binds the exact
`nextengine.isaac-usda-translator.v3` identity used by the Rust descriptor and
Python translator. The descriptor publishes each profile's
`translator_version_hash`; Rust golden generation and Python validation
recompute the same domain-separated hash from the exact body hash.

A future translator semantic change creates a new affected environment
manifest closure. It cannot rewrite frozen V1 manifests.

### Exact moving/stop semantics

For `reward.command-conditioned-support`, stop is exactly integer command
`[0, 0, 0]`; every other tuple is moving, including `[0, 16_666, 0]`. There is
no magnitude threshold and no addition across linear and angular units. A typed
`ExactZero` mode tag and its shaping ID are included in the curriculum reward
profile hash.

## Consequences

- Old standing/flat-command runs and checkpoints recover their exact identity.
- Curriculum V2 receives a new manifest/golden hash for the corrected
  translator and reward closure; it cannot resume an older incompatible V2
  candidate.
- The `1.095 m` tangent-ground edit is not lost as a design input, but belongs
  to the new biomechanics generation planned by TRAIN-1/2.
- This decision proves no policy quality, Stage 0 completion or R5 completion.

## Product checks

- `BODY-SCHEMA-P1` asserts V1 revision/root/body hash and source-order parity.
- `MOTOR-LOCOMOTION-ENV-P1` asserts both frozen V1 manifest hashes, exact-zero
  boundary vectors and byte-exact checkpoint continuation.
- `MODEL-MIRROR-P1` asserts per-profile translator hashes, Rust-generated
  descriptor/golden bytes and Python reward/translator correspondence.
- `fast`, `persistence-replay` and `platform` retain their ADR-064/065 scope.

## Rollback

Revert curriculum V2 as one complete manifest/golden generation while leaving
V1 bytes untouched. Never restore compatibility by relabeling an incompatible
BodySchema, translator or reward profile with an old hash.
