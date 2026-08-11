# ADR-064: Canonical flat-command locomotion environment

| Field | Value |
|---|---|
| ID | ADR-064 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-10 |
| Last verified | 2026-08-10 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md) |
| Supersedes | The ADR-058/SPEC-35 assumption that the standing smoke profile is the only current first-humanoid training-environment consumer. PhysX authority, fixed 23-DoF body, learned-policy exclusions and all completion gates remain Accepted. |
| Superseded by | [ADR-065](065-curriculum-flat-command-locomotion-profile.md) narrowly supersedes only the assumption that these are the only two current Stage 0 profiles; every V1 semantic and hash remains unchanged. |

## Context

The standing smoke profile proves deterministic actuation, observation and
restore, but its zero/external command and standing-pose reward cannot train a
gait. Training start/stop, forward/backward motion, strafe, diagonals and
controlled yaw needs one exact environment whose commands, observations,
reward and termination are owned by the engine rather than assembled by a
Python trainer.

This is a new consumer of the accepted substrate. It is not evidence that PPO
works, that a policy is useful, that export/runtime inference is ready or that
R5 is complete.

## Decision

### Current environment profiles

`nextengine.motor.env.humanoid-flat-command.v1` is the second current canonical
CPU profile beside the unchanged standing profile. It uses the same immutable
`nextengine.body.humanoid-23dof-v1`, fixed neutral reset, fixed 60/240 Hz
schedule, integer PD/safety path and PhysX-only authority. Its scene adds only
one static flat box with 100 metre X/Z half-extents. Terrain, friction, pose,
mass and morphology randomization are absent.

The profile lasts 1,200 motor ticks. The engine derives a separate
`randomization.command` stream from
`H(run_root, episode_ordinal, vector_slot, purpose_id)`, then derives each
target segment independently with a SHA-256 counter keyed by segment index.
There is no mutable command RNG state.

- ticks 0 through 59 command zero;
- target segments last 120 ticks;
- mode weights are stop 25%, translation 35%, turn 20%, combined 20%;
- local-right velocity is in `[-2,+2] m/s`;
- local-forward velocity is in `[-1.5,+3] m/s`;
- yaw rate is in `[-1.5,+1.5] rad/s`;
- per-tick rate limits are derived exactly from `3 m/s²` linear and
  `1.5 rad/s²` yaw acceleration at 60 Hz.

`O_t` contains `C_t`; the action is applied against `C_t`; post-state reward is
evaluated against `C_t`; the result carries `O_(t+1)` containing `C_(t+1)`.
External commands are rejected for this profile.

### Observation and fixed-point transform

The locomotion observation remains exactly 84 integer source channels:

1. root quaternion in `xyzw` order;
2. root-local linear and angular velocity;
3. 23 joint positions and 23 joint velocities;
4. 23 previous post-clamp applied actions;
5. local-right velocity, local-forward velocity and yaw-rate commands;
6. two declared foot-contact flags.

World-to-root-local rotation is an engine-owned checked Q1.30 matrix transform
with signed `i128` intermediates and round-to-nearest, ties-to-even at every
declared Q30 reduction. Python/Torch mirrors implement the same golden
algorithm. The standing layout retains its existing world-frame semantics and
layout identity.

### Reward and terminal facts

Locomotion reward is the following ordered bounded Q16 `[0,1]` component
vector. Coefficients are themselves Q16 and hash-bound with every declared
normalization denominator:

1. planar command tracking `+1.50`;
2. yaw-rate tracking `+0.50`;
3. yaw-invariant upright `+0.50`;
4. root-height tracking `+0.25`;
5. vertical-velocity cost `-0.05`;
6. roll/pitch-rate cost `-0.05`;
7. normalized post-safety effort cost `-0.02`;
8. post-clamp applied-action-rate cost `-0.05`;
9. contacting declared-foot tangential-slip cost `-0.10`;
10. fall component `-2.00`.

There is no standing-pose component. Actuator, effort, velocity and action
limits remain safety facts rather than reward terms. Pelvis height at or below
`0.45 m` and absolute X/Z position at or beyond `90 m` terminate. Tick 1,200
truncates unless termination has already won on that same committed step.
`terminated` and `truncated` are never conflated.

### Contracts, runner and replay

`MotorTrainingEnvironmentManifestV2` is current for both profiles. In addition
to the V1 closure it binds command-schedule, reward, termination, RNG
derivation and correspondence profile hashes. Protocol v2 rejects V1 with a
typed `UNSUPPORTED_MOTOR_ENVIRONMENT_MANIFEST_VERSION` before mutation.

The consumer-backed records are `MotorLocomotionCommandProfileV1`,
`MotorResetRecordV2`, `MotorStepRecordV2`, `MotorTrajectoryManifestV2` and the
bounded `MotorEnvironmentCheckpointEnvelopeV1`. A step record chains prior and
next observation roots, applied command/action, ordered reward vector and Q16
total, separate terminal flags, physics/motor roots and prior/next step roots.

Each vector slot owns its episode ordinal, fresh scene, terminal lifecycle and
precomputed command schedule. Partial reset increments only selected slots.
Step accepts `(slot, episode_ordinal, action)` for every slot in a closed batch.
Duplicate/missing slots, stale episode, malformed action and post-terminal step
reject before mutation. Input staging is preallocated and indexed by slot;
publication remains sorted by `(episode_ordinal, vector_slot)`.

Checkpoint payload is at most 4 MiB and binds profile, manifest, run, episode,
slot, tick, terminal disposition, observation/step roots and opaque canonical
Motor runtime bytes. Restore validates all identity and hashes, creates a
fresh scene, replays the post-safety effort prefix and atomically replaces the
slot only after exact witness and command-schedule continuation match.

### Protocol and accelerated mirror

`headless motor-lab` protocol v2 owns monotonic request IDs and the exact
`Create`, partial `Reset`, `Step`, `Checkpoint`, `Restore`, `Ping` and `Close`
operations. `Create` accepts only an engine-known profile ID, slot capacity and
run root. It exposes hashes and capacities; it accepts no reward or physics
override.

The Python client is an integer-authority client with an optional normalized
action adapter. Trajectory NPZ v2 artifacts are written only to an explicit
external training store. Isaac Lab selects an engine descriptor, precomputes
the exact CPU command schedule at reset, preserves quaternion/frame semantics,
uses the declared integer formulas and supports partial per-environment reset.
It is never replay authority.

Correspondence v2 adds byte-exact commands, profile hashes and component order
plus reward-total MAE at most `0.05`. ADR-058 joint/root/velocity/contact/done
thresholds remain unchanged.

## Consequences and claim boundary

- Canonical CPU trajectories may be collected and used for local policy
  experiments after the relevant CPU checks pass.
- Standing behavior and world-frame observation semantics do not change.
- PPO, motion imitation, terrain, pushes, recovery/get-up, export and runtime
  learned evaluation remain separate future consumers.
- Missing GPU or Linux evidence is `NOT_RUN`: it does not block local CPU
  experiments, but it blocks full readiness, mirror promotion and R5 claims.

## Product checks

`MOTOR-LOCOMOTION-ENV-P1` covers golden schedules, bounds/modes/rate limits,
input and reset permutations, independent episode ordinals, terminal lifecycle,
byte-exact checkpoint continuation, reward invariants and at least 1,000,000
aggregate integer steps without overflow. Existing BODY/PHYS/MOTOR checks,
`MODEL-DATAPLANE-P1`, `MODEL-MIRROR-P1`, `fast`, `host-check`, `play`,
`persistence-replay`, `platform` and conditional `performance` remain required
according to the routing table.

## Rejected alternatives

- **Trainer-provided commands or reward weights.** They would make the Python
  process a second authority and make replay/profile identity ambiguous.
- **Reuse the standing-pose reward.** It rewards suppressing the joint motion
  needed to form a gait.
- **World-frame command tracking.** It changes command meaning with heading and
  does not express strafe/forward intent consistently.
- **Mutable per-slot RNG continuation.** It makes partial reset and restore
  depend on hidden consumption order instead of episode seed and tick.
