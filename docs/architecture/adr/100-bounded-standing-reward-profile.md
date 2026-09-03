# ADR-100: Bounded standing reward profile

| Field | Value |
|---|---|
| ID | ADR-100 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-064](064-canonical-flat-command-locomotion-environment.md), [ADR-067](067-stage0-profile-identity-and-curriculum-hash-closure.md) |
| Supersedes | Narrowly supersedes ADR-064/067 wherever the frozen standing V1 reward could be treated as the current optimizer objective or the closed set of Stage 0 standing identities. V1 bytes, hashes, replay meaning and all other environment semantics remain unchanged. |
| Superseded by | none |

## Context

The first R8b standing run completed its exact `4,096,000`-transition budget,
but did not test whether the frozen humanoid can learn to stand. Standing V1
adds unit-scale posture terms to raw micronewton-metre effort and microradian
action-rate penalties. In the completed run, final projected mean effort and
action-rate components were approximately `-241,795,296` and `-1,513,819`,
while upright and pose were `0.480` and `-21.325`. Mean episode length fell
from `101.51` over the first 20 iterations to `62.39` over the last 20, and
value loss reached approximately `5.33e18`.

This is an environment-scale defect, not evidence for changing PPO, adding a
motion corpus or replacing the frozen body. Standing V1 remains immutable
historical/replay input and is retired only from new optimizer runs.

## Decision

### Distinct environment identity

Add `nextengine.motor.env.humanoid-standing.v2`. It reuses the exact Standing
V1 BodySchema, world-frame 84-channel observation, 23-channel residual action,
fixed PD/safety path, zero command, 50 metre scene, reset, `0.25 m` fall
threshold and 3,600-tick timeout. It binds the current
`nextengine.isaac-usda-translator.v3` identity.

Its environment manifest hash is:

```text
b39b4ae2674b0aa8c79990a039015c6c41c57c6f8af23b6c0283ed05a3555f04
```

Standing V1 keeps manifest
`dc64488393a57a07e993fdf062f663c8024fc1475267dbf6e8c35f5458f45396`.
No V1 checkpoint may resume or initialize V2.

### Bounded commensurate reward

Every V2 component is an exact integer Q16 value in `[0, 65,536]`, computed
from post-step production facts:

1. yaw-invariant upright: clipped `1 - 2(x² + z²)` from the engine quaternion;
2. height tracking: `1 - |height - 1.05 m| / 0.60 m`;
3. pose tracking: `1 - sum(|joint position|) / (23 × 1.50 rad)`;
4. root-motion cost: the maximum of normalized three-axis linear and angular
   absolute sums, using `3 × 3 m/s` and `3 × 6 rad/s` denominators;
5. applied-effort cost: absolute post-safety effort over all four substeps,
   divided by the exact summed actuator maximum for the frame;
6. applied-action-rate cost: summed absolute post-clamp action delta divided
   by `23 × 2 rad`;
7. contacting-foot tangential-slip cost: absolute planar speed divided by
   `4 m/s` for each currently contacting declared foot;
8. fall indicator at the unchanged `0.25 m` standing threshold.

The ordered Q16 coefficients are:

```text
[+1.00, +0.50, +0.25, -0.10, -0.02, -0.05, -0.10, -2.00]
```

The exact integer coefficients are
`[65536, 32768, 16384, -6554, -1311, -3277, -6554, -131072]`.
Therefore one motor-step total is bounded by `[-148,768, 114,688]` Q16,
approximately `[-2.27, 1.75]`. Component order, coefficients,
normalizations, body hash and translator identity are hash-bound in the V2
manifest. Rust CPU and Torch/Isaac implementations use the same ties-to-even
integer arithmetic.

### Experiment isolation

The first V2 run retains the standing V1 seed, PPO settings, network shape,
parallelism and `4,096,000`-transition budget. Only the environment/training
profile identities change. This makes reward identity the falsifiable
independent variable.

## Consequences

- New optimizer runs cannot silently consume the broken V1 objective.
- Posture and physical costs have explicit, comparable bounds; discounted
  return targets no longer inherit raw microunit magnitudes.
- CPU PhysX remains final evidence authority. Isaac acceleration requires the
  exact V2 descriptor/golden and a no-training reset/reward probe.
- This decision proves no policy quality, CPU/Isaac trajectory
  correspondence, Stage 0 completion or runtime policy admission.

## Product checks

- `MOTOR-LOCOMOTION-ENV-P1` asserts V1 immutability, V2 manifest identity,
  component bounds, coefficient order and deterministic reset/step records.
- `MODEL-MIRROR-P1` asserts the V2 translator closure and exact Rust/Python
  reward vectors; it remains separately `NOT_RUN` until paired trajectories
  exist.
- Risk-scoped `fast`, `host-check`, `persistence-replay` and `platform` retain
  the applicable SPEC-35 boundary.

## Rollback

Stop and retire the complete V2 optimizer generation. Keep V1 and V2 evidence
immutable. Any change to coefficients, normalizations, fall semantics or
reward facts requires a new environment identity; never rewrite or relabel
the V2 manifest.
