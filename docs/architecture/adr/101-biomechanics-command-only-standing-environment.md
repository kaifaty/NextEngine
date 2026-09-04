# ADR-101: Biomechanics command-only standing environment

| Field | Value |
|---|---|
| ID | ADR-101 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-explicit-solver-projection.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-100](100-bounded-standing-reward-profile.md) |
| Supersedes | Narrowly supersedes ADR-100 wherever its frozen Stage 0 V1 body could be treated as the current R8b optimizer candidate, and ADR-070 wherever the corpus-bound reference tracker could be treated as the only current training consumer of biomechanics V2. Historical identities and evidence remain unchanged. |
| Superseded by | none |

## Context

The bounded standing V2 run fixed reward scale and materially increased rollout
survival, but deterministic evaluation still fell at ticks `100` and `97`.
Frame-by-frame inspection and the exact descriptor showed a more fundamental
problem: the frozen Stage 0 V1 body uses 24 sphere colliders, 23 collinear
X-axis revolute joints, uniform symmetric ROM and isotropic inertia. It remains
useful as a deterministic regression fixture, but is not a credible body for a
learned humanoid claim.

The existing `nextengine.body.humanoid-biomechanics-raja-1700.v2` already has
anatomical pitch/yaw/roll axes, asymmetric hard and soft ROM, full inertia,
box/capsule support geometry, stable contact roles and a complete fixed-PD
safety envelope. Its authored neutral pose places both sole boxes exactly on
the ground. The apparent neutral-contact blocker came from a mock ABI test
whose contact exporter deliberately returns zero contacts; the same invariant
passes against the pinned native PhysX SDK with two distinct sole shape tokens.

R8b must remain command-only. It cannot consume the stopped motion-reference
corpus, reference tracker or R123–R141 artifacts merely to make biomechanics V2
trainable.

## Decision

### Distinct current-body identity

Add `nextengine.motor.env.humanoid-biomechanics-standing.v1` over
`CompiledBodySchemaV3` for the exact biomechanics V2 body and ADR-071 material
lineage. Its manifest is
`1b60550d003e4c04045a14ed40c65ddb2b12b00940053276b469ebd2e8c473bb`;
its compiled descriptor is
`6751853a812f549866f1db9d3662d8115b18db9b6d73beabd7221bb9f972f027`.
No Stage 0 V1 or reference-tracker checkpoint can resume or initialize it.

The reset is the authored neutral articulation at pelvis height `0.9435 m`,
with exact zero sole clearance. One motor tick remains four native PhysX
substeps. Self-collision exclusions, contact roles, materials, hard/soft ROM,
effort, velocity, target-rate, power and positive-work limits all come from the
compiled descriptor.

### Command-only residual control

The 23-channel policy action is a normalized Q1.30 residual around the existing
deterministic procedural-standing controller. That controller supplies the
zero-action fallback and adjusts the two knees and ankle pitches from canonical
root pose/velocity facts. The ordinary safety owner then applies residual
scale, skill soft ROM, target slew, fixed PD and the complete actuator envelope.
The trainer cannot write joint transforms, efforts or alternative targets.

The observation has 84 channels: root quaternion and world velocities,
23 ordered joint positions, 23 joint velocities, 23 previous post-safety
targets, exact zero command and two declared sole-contact bits. The action and
observation identities are distinct from all Stage 0 V1 and reference-tracker
layouts.

### Bounded objective and termination

The environment retains ADR-100's eight Q16 component coefficients and
`[-148768,114688]` total bound. Pose tracking is measured against the current
procedural-standing reference and normalized by the exact sum of biomechanics
soft-ROM spans. Effort and target-rate normalizers are derived from the current
compiled actuator envelopes; height, root-motion, sole-slip and fall remain
explicit bounded facts.

Pelvis height `<=0.45 m`, root tilt `>=60°`, planar world bound `>=90 m`, hard
ROM/joint safety, hard impact or self-collision terminate. Tick `3,600`
truncates. CPU PhysX remains final evidence authority.

### Minimal first optimizer run

The first Proposed RTX 3080 profile is isolated at `128 × 32 × 250 = 1,024,000`
transitions, seed `42`, with checkpoints every 25 iterations. This is a smoke-
scale discriminator, not a standing-quality budget. It may start only after a
native CPU reset/step/contact/safety/reward probe and exact descriptor-to-USD
generation preflight pass.

## Consequences

- New learned-humanoid optimization uses the anatomically meaningful current
  body instead of the bead-like Stage 0 regression body.
- R8b stays independent of Kimodo, motion-corpus and stopped TRAIN-4 lineage.
- The procedural standing controller remains a deterministic in-process
  fallback and makes zero policy output a viable controlled baseline.
- Isaac may accelerate the isolated experiment, but its checkpoint has no
  runtime or quality authority until paired correspondence and fixed held-out
  CPU gates pass.
- Frozen V1 environments, descriptors, checkpoints and evidence are not
  rewritten or relabelled.

## Product checks

- Native PhysX proves two distinct sole contacts and a 32-tick current-material
  reset/step/safety/contact/termination/reward preflight.
- Focused Rust/Python checks assert manifest hashes, ordered layouts, bounded
  rewards, exact procedural targets, actuator safety and immutable USD/material
  translation.
- `MODEL-MIRROR-P1` remains `NOT_RUN` until paired CPU/Isaac trajectories are
  captured; a GPU reset/reward smoke is only execution preflight.

## Rollback

Stop and retire the complete biomechanics-standing generation and any child
runs. Preserve its exact artifacts for diagnosis. Resume no Stage 0 V1 or
reference-tracker checkpoint. Any body, reset, action, reward, termination or
translator semantic change requires another environment and generation
identity.
