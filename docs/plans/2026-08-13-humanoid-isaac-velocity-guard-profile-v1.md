# Humanoid Isaac velocity guard profile V1

| Field | Value |
|---|---|
| Status | Candidate-local `TRAIN-3` correspondence input |
| Candidate | `HumanoidFlatRecoveryCandidateV1` |
| BodySchema | `nextengine.body.humanoid-biomechanics-raja-1700.v2@2` |
| BodySchema hash | `e2460e7dc4af93538ae4b0b68a9e1bf74b2b7990161e08e441d58687e953c43d` |
| Isaac Lab / Isaac Sim | `2.3.2 / 5.1` |
| Physics / motor cadence | `240 Hz / 60 Hz`, exactly four ordered physics substeps per motor tick |
| Outer velocity limit | Exact per-DoF BodySchema `maximum_velocity_microradians_per_second` |
| PhysX inner velocity limit | `9,000` basis points (`90%`) of the corresponding outer limit |

This profile adds solver headroom without relaxing the BodySchema safety
contract. The BodySchema maximum remains the observation normalization, the
hard terminal comparison and the externally reported limit. The lower PhysX
value is an implementation guard against constraint/contact impulses and
solver canonicalization crossing that outer limit between two 240 Hz safety
checks.

## 1. Normative mapping

For every descriptor joint `j`, the Isaac articulation must receive:

```text
velocity_limit_sim[j]
  = maximum_velocity_microradians_per_second[j]
    * 9,000 / 10,000 / 1,000,000 rad/s
```

All frozen descriptor limits are integer multiples that make this mapping
exact in decimal (`3.6`, `7.2`, `9.0` or `10.8 rad/s`). Missing, non-integer,
out-of-range or unrecognized guard values reject environment construction.
The guard is profile-owned: a command-line override, learned output, reward or
checkpoint cannot change it.

The four 240 Hz substeps still check the observed canonical velocity against
the full BodySchema outer limit before publishing effort. Any outer-limit
crossing remains `terminal.joint-safety`; no tolerance is added. Effort,
effort-rate, power, work, target slew and ROM rules are unchanged.

## 2. Basis for the 90% value

The optimizer-free V3 phase audit used the exact outer limit as the PhysX
limit and covered `4,112` episodes over all `169` admissible start phases of
`cmu104-start-right`. Reset pose, target and reference velocities were valid,
and reset-window ROM/contact failures were zero, but `1,187` episodes crossed
the outer velocity limit on their first motor tick.

The largest measured numerical/constraint overshoot was `246,954 µrad/s` on
the `4,000,000 µrad/s` torso-roll outer limit (`6.17385%`). A 90% inner limit
reserves `400,000 µrad/s` on that smallest outer limit, exceeding the measured
overshoot by `153,046 µrad/s`. A directed reproduction at start frame `46`
then changed only the PhysX inner limit from 100% to 90%: it changed the
first-tick result from a joint-velocity failure to no terminal, while all
observed velocities remained below their unchanged outer limits. These values
are engineering correspondence evidence, not learned-quality targets or a
medical safety claim.

The measurement only motivates the frozen candidate value. Acceptance still
requires a fresh full-coverage audit; the directed reproduction alone cannot
authorize training.

## 3. Acceptance and invalidation

`TRAIN-3/5` may advance only when all of the following are hash-bound to this
document through the immutable tracker profile:

- environment construction proves the exact per-joint inner-limit mapping;
- input audit reports the guard identity and exact `9,000` basis points;
- at least `4,096` randomized episodes cover all `169` admissible start
  phases with zero hard-safety event in the declared reset window;
- directed limit-saturation evidence has zero outer-limit crossing;
- hard-impact, self-collision, forbidden-contact, world-bound and fall
  terminal probes still pass under the guarded articulation.

A change to the basis points, physics cadence, Isaac/PhysX version, compiled
USD, BodySchema, contact profile, tracker input or corpus invalidates this
evidence. Until a new audit and gate close, optimizer execution remains
forbidden.
