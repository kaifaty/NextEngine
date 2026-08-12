# Humanoid safety and contact profile V1

| Field | Value |
|---|---|
| Status | Revalidated candidate-local input for `TRAIN-3` after BodySchema revision 2 |
| Candidate | `HumanoidFlatRecoveryCandidateV1` |
| BodySchema | `nextengine.body.humanoid-biomechanics-raja-1700.v2@2` |
| BodySchema hash | `e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d` |
| Physics / motor cadence | `240 Hz / 60 Hz`, exactly four physics substeps per motor tick |
| Scope | Flat locomotion, brace/fall and get-up classification before any ML run |

This profile supplies the exact values intentionally left to `TRAIN-3` by the
requirements baseline and rebuild plan. It is an engine-owned experiment
input, not a public runtime schema, clinical injury model or claim that a
learned policy is safe. A value change creates a new profile hash and
invalidates `TRAIN-3` and every downstream artifact.

## 1. Joint target and fixed-PD safety

For DoF `j`, normalized residual input is signed `Q1.30` in `[-1, 1]`.
Integer multiplication and division use round-to-nearest, ties-to-even:

```text
candidate[j] = reference[j]
             + round_even(residual_q1_30[j] * residual_scale[j] / 2^30)

envelope_min[j] = max(hard_min[j], soft_min[j], skill_min[j])
envelope_max[j] = min(hard_max[j], soft_max[j], skill_max[j])

slew_min[j] = previous_applied[j] + negative_target_delta[j]
slew_max[j] = previous_applied[j] + positive_target_delta[j]

applied[j] = clamp(
  candidate[j],
  max(envelope_min[j], slew_min[j]),
  min(envelope_max[j], slew_max[j])
)
```

`reference` never bypasses the intersection. Invalid width, invalid skill
envelope, empty intersection with the slew interval, non-canonical DoF mapping
or arithmetic overflow rejects the whole action without changing previous
targets, effort history or work counters.

Each 240 Hz PD substep applies the authored per-joint `Kp/Kd`, then intersects:

1. signed effort range;
2. previous effort ± authored effort-rate / `240`;
3. authored instantaneous power;
4. remaining positive-work allowance for the current 60 Hz motor tick.

Positive power is `max(effort * velocity, 0) / 1_000_000` microwatts. Positive
work charged for one physics substep is power / `240` microjoules, rounded
up so sub-microjoule work is not hidden. A state already outside hard ROM or
authored maximum joint velocity returns a stable safety violation before an
effort is published. Reset restores neutral applied targets and clears prior
effort, substep ordinal and accumulated work exactly.

### Procedural standing fallback and neutral scenario

The non-learned fallback runs for exactly `1,800` motor ticks (`30` simulated
seconds) from the compiled neutral reset. It emits reference targets through
the same residual/slew/PD/safety path: both knees use `100,000 µrad`, both
ankle-pitch channels use the following integer ankle strategy and every other
channel uses authored neutral:

```text
pitch_proxy = round_even(root_qx_q1_30 * 2,000,000 / 2^30)
ankle_pitch = -140,000
            + round_even(pitch_proxy / 2)
            + round_even(root_angular_velocity_x / 20)
            + round_even((root_z - reset_root_z) / 10)
            + round_even(root_linear_velocity_z / 50)
```

The scenario passes only by reaching its exact timeout as `Truncated`, with
zero joint/contact/fall terminal event. The controller never writes a body or
joint pose and retains only the reset root-Z target as state.

## 2. Contact measurement

PhysX shape-level contact impulse is consumed in micro-newton-seconds. The
canonical magnitude comparison uses squared Euclidean components, so no
floating-point square root participates in classification.

All contact points for one canonical `(actor, shape)` pair are reduced once
per physics substep: impulse vectors are summed with checked arithmetic and
minimum separation is retained. A manifold therefore consumes one continuity
step, independent of point count or callback order.

| Parameter | Exact value | Meaning |
|---|---:|---|
| Active-contact impulse | `50,000 µN·s` (`0.05 N·s`) | Below this, a positive-separation contact is sensor noise |
| Brush ceiling | `250,000 µN·s` (`0.25 N·s`) | A non-sole contact above this is immediately material |
| Low-impulse grace | `4` consecutive physics substeps | One complete 60 Hz motor frame may be ignored; the fifth active substep is material |
| Continuity break | `1` inactive physics substep | Clears consecutive-contact state for that shape pair |
| Ground actor token | `1` | The fixed flat ground in the compiled descriptor |
| Ground shape token | `0` | Static V1 ground shapes carry no per-shape user token in the PhysX ABI |
| Runtime root-quaternion norm tolerance | `2^40 Q2.60` (about `9.54e-7`) | Larger canonical norm error is malformed state |

Penetration (`separation < 0`) is active even below the impulse threshold.
Speculative PhysX pairs with positive separation and zero impulse are not
contacts. Continuity keys use the canonical ordered `(actor, shape)` pair and
therefore do not depend on callback order.

## 3. Hard per-role impact budgets

The budget applies to a single 240 Hz contact sample and is checked before
skill-specific allowance. Exceeding it is always a safety violation.

| `BodyContactRoleV2` | Maximum impulse |
|---|---:|
| `FootWithSoleFeature` | `6.00 N·s` |
| `ThighGround`, `ShankGround`, `KneeGround`, `PelvisGround`, `TorsoGround` | `4.00 N·s` |
| `AnkleGround`, `UpperArmGround`, `ForearmGround`, `HandGround` | `3.00 N·s` |
| `HeadGround` | `1.00 N·s` |

The sole value corresponds to about `1.95 BW` averaged over one 240 Hz
substep for the fixed `75.337 kg` body. It deliberately exceeds reported
walking peaks of roughly `1.0–1.5 BW` while remaining below the `2.0–2.9 BW`
range reported for running, which is outside this candidate. Source:
Nilsson and Thorstensson, *Ground reaction forces at different speeds of human
walking and running*, DOI `10.1111/j.1748-1716.1989.tb08655.x`.

The non-sole values are conservative project engineering bounds, not values
claimed by that paper and not medical harm thresholds.

## 4. Skill/contact classification

| Role | Locomotion | Brace/fall | Get-up |
|---|---|---|---|
| sole | `SoleSupport` | `SoleSupport` | `SoleSupport` |
| hand, forearm | material → `ForbiddenLocomotion`; sub-material → `TransientAllowed` | `BraceSupport` | `GetUpSupport` |
| knee | material → `ForbiddenLocomotion`; sub-material → `TransientAllowed` | `TransientAllowed` | `GetUpSupport` |
| pelvis, torso, head, thigh, shank, ankle, upper arm | material → `ForbiddenLocomotion`; sub-material → `TransientAllowed` | `TransientAllowed` below hard impact budget | `TransientAllowed` below hard impact budget |
| non-excluded articulation self contact | `SelfCollisionViolation` when material | same | same |
| joint/controller safety event | `JointSafetyViolation` | same | same |

“Material” means either impulse strictly above the brush ceiling in the
current substep or active continuity longer than the four-substep grace.
Hard-impact violation has no grace.

## 5. Locomotion terminal priority

At a 60 Hz commit boundary, exactly one reason is chosen in this order:

1. `terminal.non-finite-state`;
2. `terminal.joint-safety`;
3. `terminal.contact-impact`;
4. `terminal.self-collision`;
5. `terminal.forbidden-locomotion-contact`;
6. `terminal.world-bounds` at `|X|` or `|Z| >= 90 m`;
7. `terminal.fall` at root height `<= 0.45 m` or root up-axis tilt
   `>= 60 degrees` from world up;
8. `terminal.timeout` at the profile episode limit.

Reasons 1–7 are `Terminated`; timeout alone is `Truncated`. A required
recovery profile uses the same non-finite, joint, impact, self-collision and
world-bound failures but does not reinterpret permitted brace/get-up support
as forbidden locomotion. Terminal evaluation consumes all four ordered contact
classification frames from the motor tick, so a violation in an earlier
substep cannot disappear before commit. A terminal decision is latched until
reset; reset clears contact continuity and terminal state.

## 6. Freeze checklist

- all numeric comparisons are integer/exact after PhysX canonicalization;
- root tilt is compared from the canonical quaternion's rotated up-axis,
  without Euler-angle decomposition;
- shape token → body contact role comes only from the compiled BodySchema;
- no reward component can waive a clamp, violation or terminal reason;
- CPU safety/contact roots and the generated mirror carry this profile hash;
- `TRAIN-3` begins with no policy, optimizer, checkpoint or motion corpus.
