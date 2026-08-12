# ADR-065: Curriculum flat-command locomotion profile

| Field | Value |
|---|---|
| ID | ADR-065 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-12 |
| Last verified | 2026-08-12 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md), [ADR-064](064-canonical-flat-command-locomotion-environment.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Supersedes | Narrowly supersedes ADR-064's assumption that standing and `humanoid-flat-command.v1` are the only current Stage 0 environment profiles. It does not change either existing profile or their hashes. |
| Superseded by | none |

## Context

The first pure-PPO run over `humanoid-flat-command.v1` completed 4,096,000
samples, but its held-out policies survived only about 135--137 motor ticks and
all 768 evaluated episodes ended in a fall. The environment asked a policy
starting from no locomotion skill to cover backward motion, strafe, combined
turning and up to 3 m/s immediately. Its broad linear tracking component still
gave a mostly stationary body substantial reward, while a fall cost only two
reward units.

The old profile remains useful as a fixed stress baseline and is already bound
into completed run manifests. Mutating it would invalidate reproducibility.
The next experiment therefore needs a new engine-owned identity rather than
trainer-side command or reward overrides.

## Decision

### New current environment profile

`nextengine.motor.env.humanoid-flat-command-curriculum.v2` is a third current
Stage 0 profile. It reuses the exact fixed humanoid, neutral reset, flat scene,
84-channel root-local observation, 23 residual-position actions, fixed
60/240 Hz PD/safety path, `0.45 m` fall threshold, `90 m` world bound and
1,200-tick timeout of ADR-064. V1 standing and flat-command bytes and behavior
remain unchanged.

The engine selects one immutable command stage from `episode_ordinal`; a
trainer cannot select or rewrite the stage. Every stage still uses the pure
SHA-256 counter schedule and the `randomization.command` purpose seed:

| First episode ordinal | Warm-up / segment | stop / translation / turn / combined | right m/s | forward m/s | yaw rad/s | linear / yaw acceleration |
|---:|---:|---:|---:|---:|---:|---:|
| 0 | 120 / 240 ticks | 40% / 60% / 0% / 0% | 0 | 0..0.75 | 0 | 1.0 m/s² / 0.5 rad/s² |
| 32 | 90 / 180 ticks | 25% / 55% / 15% / 5% | -0.35..0.35 | 0..1.25 | -0.6..0.6 | 1.5 m/s² / 0.75 rad/s² |
| 96 | 60 / 120 ticks | 15% / 45% / 15% / 25% | -1..1 | -0.5..2 | -1..1 | 2.0 m/s² / 1.0 rad/s² |

The command-profile hash covers the ordered stage thresholds and the exact
`MotorLocomotionCommandProfileV1` hash of every stage. Reset, partial reset,
checkpoint and restore derive the same stage from the recorded episode
ordinal. Bad policies cannot make an unrecorded adaptive curriculum decision;
advancing an episode is the only progression input.

### V2 reward vector

V2 uses eleven bounded Q16 components. Planar L1 tracking uses
`max(0, 1 - error / 2.5 m/s)^2`; yaw tracking uses
`max(0, 1 - error / 1.5 rad/s)^2`. Height, vertical velocity, roll/pitch rate
and contacting-foot slip normalize at `0.4 m`, `2 m/s`, `4 rad/s` and
`2 m/s` per contacting foot. Upright, normalized effort and applied-action
rate retain their ADR-064 facts.

`reward.command-conditioned-support` is one when a moving command has exactly
one declared foot in contact, or a stop command has both declared feet in
contact; otherwise it is zero. It is only shaping evidence and never creates a
contact or replaces physics authority.

The ordered coefficients are:

```text
planar +2.00, yaw +0.50, upright +1.00, height +0.50,
vertical -0.10, roll/pitch -0.10, effort -0.01,
action-rate -0.02, slip -0.20, command-conditioned support +0.25,
fall -10.00
```

Every formula, denominator, component ID and coefficient is included in the
reward-profile hash. The V1 ten-component vector is unchanged.

### Reference pure-PPO experiment

The Proposed RTX 3080 experiment profile uses 512 environments, 24 steps per
environment, 3,000 iterations (36,864,000 samples), a `[512,256,128]` MLP,
initial action noise `0.6`, entropy coefficient `0.005`, adaptive learning rate
`3e-4`, KL target `0.01`, five epochs and four mini-batches. Checkpoints are
saved every 100 iterations.

Held-out deterministic evaluation uses seeds 1001--1005 and explicitly starts
at episode ordinal 96 so it exercises the full command stage rather than only
the introductory curriculum. The ordinal start is recorded in the evaluation
manifest. These are experiment settings, not public gameplay or PPO authority.

## Consequences and claim boundary

- CPU motor-lab and Isaac mirror implement the same stage selection, integer
  command schedule, reward order and Q16 formulas.
- `humanoid-flat-command.v1` remains the comparison baseline for every prior
  run and checkpoint; a V1 checkpoint cannot resume under the V2 config hash.
- The new profile improves the pure-PPO baseline but does not demonstrate a
  human-like gait, learned Motor support, export parity or R5 completion.
- Licensed motion tracking, randomized motion-phase starts and imitation plus
  task RL remain the next learned-policy phase under SPEC-34/ADR-066; they are
  not approximated by hidden trainer logic in this profile.

## Product checks

`MOTOR-LOCOMOTION-ENV-P1` additionally covers all three stage boundaries,
schedule bounds/rate limits, reward ordering/support facts, partial-reset
episode independence and byte-exact checkpoint continuation. `MODEL-MIRROR-P1`
binds the third descriptor profile and Rust-generated golden schedules. The
existing `fast`, `host-check`, `persistence-replay`, `platform` and conditional
performance selection remains unchanged.

## Rejected alternatives

- **Mutate V1 reward or command ranges.** Rejected because completed runs bind
  its exact hashes.
- **Trainer-controlled adaptive commands or reward coefficients.** Rejected
  because Python would become a second environment authority.
- **Evaluate only ordinal-zero episodes.** Rejected because it measures the
  introductory stage instead of the final command distribution.
- **Add pose imitation without an admitted motion corpus.** Rejected for this
  profile because source license, retargeting, contact labels and randomized
  reference starts require their own immutable data and correspondence closure.
