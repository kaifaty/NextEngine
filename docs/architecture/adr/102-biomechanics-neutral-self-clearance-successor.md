# ADR-102: Biomechanics neutral self-clearance successor

| Field | Value |
|---|---|
| ID | ADR-102 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-09-04 |
| Last verified | 2026-09-04 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md), [ADR-071](071-canonical-physics-material-lineage.md), [ADR-101](101-biomechanics-command-only-standing-environment.md) |
| Supersedes | ADR-101 only for the current R8b body and standing optimizer identity. The V2 body, V1 standing environment, descriptors, runs and evaluations remain immutable historical evidence. |
| Superseded by | none |

## Context

Pair-complete self-contact enforcement invalidated both biomechanics-standing
V1 checkpoints. The fresh contact-correct 1,024,000-transition run improved
mean rollout length from `3.19` to `143.61` ticks, but its predeclared final
checkpoint terminated at GPU tick `139` and CPU tick `135` on self-collision.

A smaller discriminating control then separated policy quality from body
viability. The deterministic zero-residual fallback survives the complete
3,600-tick canonical CPU episode, while the same reset terminates on GPU tick
`3`. The V2 neutral pose leaves only `405 um` between each forearm box and the
pelvis box. Reducing Isaac contact offset below that gap changes the reported
pair from positive-separation predictive contact to `279..333 um` of actual
penetration but does not prevent termination. The body therefore lacks enough
neutral self-clearance to tolerate an otherwise small CPU/GPU trajectory
difference; another optimizer run on that identity would train around a
geometry defect.

## Decision

Add `nextengine.body.humanoid-biomechanics-raja-1700.v3`, revision `3`, as a
strict successor to V2. It preserves topology, 23 anatomical axes and ROM,
source-segment mass/CoM/inertia, colliders, materials, effectors, actuators and
all collision exclusions. The numeric morphology change moves the bilateral
shoulder roots from `+/-170,000 um` to `+/-215,000 um` on engine X. This raises
the exact neutral pelvis-to-forearm AABB clearance from `405 um` to `45,405 um`,
above the pinned PhysX default pair contact distance of `40,000 um`, without a
GPU-only contact-offset override or a pelvis/forearm pass-through exclusion.

V3 also replaces each `0.001 kg / 0.000001 kg*m^2` non-colliding serial-axis
carrier with `0.25 kg / 0.001 kg*m^2` and subtracts the exact deltas from the
co-located physical link. Source mass, first moment and inertia therefore
remain exact while the worst local solver ratios are reduced by orders of
magnitude. This follows the PhysX stability requirement to avoid extreme
link-mass/inertia ratios; it is not backend auto-mass or an unbound armature.

The successor uses distinct identities:

- `nextengine.motor.env.humanoid-biomechanics-standing.v2`;
- `nextengine.motor.observation.humanoid-biomechanics-standing.v2`;
- `nextengine.motor.action.humanoid-biomechanics-standing-residual.v2`;
- `nextengine.isaac.humanoid-biomechanics-standing.v2`;
- `nextengine.isaac-rsl-rl.rtx3080-biomechanics-standing.v2`.

Standing reward, procedural fallback, action representation, safety limits,
termination thresholds, `240/60 Hz` timing and the `128 x 32 x 250` smoke-scale
optimizer budget remain unchanged. V2/V1 artifacts cannot resume or initialize
the successor. CPU PhysX remains authority; Isaac remains a hash-bound mirror.
Observed-state validation applies the same explicit backend-conversion guard in
both lanes: `10 microrad` beyond hard ROM and `1,000 microrad/s` beyond the
declared maximum velocity. The latter is `0.001 rad/s` (`0.01%` of the affected
`10 rad/s` shoulder limit) and admits float-to-integer solver quantization only;
target, effort, power, work and authored velocity limits are unchanged.

Before optimizer work, canonical CPU and GPU reset controls must show that the
successor no longer terminates on the immediate neutral pelvis/forearm contact.
The procedural fallback is an initialization aid, not a standing-quality gate:
requiring it to survive the full episode would move the optimizer's job into a
hand-written controller. The final fresh checkpoint still requires five
complete CPU standing episodes and paired `MODEL-MIRROR-P1` evidence before
walking may start.

## Consequences

- The V2 body remains available to the frozen reference-tracking lineage and
  historical evidence; no old hash or identifier is relabeled.
- The wider shoulder placement is an explicit morphology revision, not a
  trainer-side workaround.
- Any checkpoint trained against the V1 standing environment remains
  incompatible with V2.
- Immediate neutral self-contact in either reset control rejects this successor
  before another expensive PPO run; a later physical fall is valid optimizer
  input and not itself a morphology failure.

## Product checks

- `BODY-SCHEMA-P2` asserts the exact bilateral shoulder bind, `45,405 um`
  neutral pelvis/forearm clearance and source-group mass/first-moment/inertia
  conservation while preserving V2 bytes.
- Native biomechanics standing tests cover two-sole contact, safety, reward and
  zero-policy survival on the V2 environment, including exact observed-state
  guard boundaries.
- Isaac reset/reward preflight confirms bounded reward and a useful safety
  horizon without immediate neutral self-contact before training.
- `MODEL-MIRROR-P1` and the five fixed CPU episodes remain admission gates.

## Rollback

Retire the complete V3/V2 generation and preserve it as negative evidence.
Do not mutate V2/V1, relax self-collision thresholds, add backend-only collider
settings or resume an incompatible checkpoint.
