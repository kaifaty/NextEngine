# R8b — lift and return discriminator

Status: `REPORT_ONLY / GEOMETRIC_DISCRIMINATOR_PASS / NOT_LEARNED_WALKING`.
This is the executable next experiment selected by the
[sole/method investigation](r8b-sole-support-and-training-method-research-2026-09-05.md),
not another optimizer run. No body, action, safety or admitted reward changes.

## Question and fixed candidate

Can one geometric signal distinguish V6's grounded rocking from actual foot
release, while rejecting a persistently raised foot and requiring return?
V6's load-transfer term cannot make this distinction. Keep it; the candidate
is an additional negative height-error term, not a replacement load term.

Use V6's 72-tick clock starting at action tick 120. Left sole target rises
from zero at phase 6 to 60 mm at 18, returns to zero at 30; right is offset
36 ticks. Each 12-tick half uses `60000*(10u^3-15u^4+6u^5)` micrometres,
with integer rational evaluation and floor rounding. Elsewhere the target is
zero. At zero command the new cost is disabled, preserving the stopping task.
These constants were selected before measuring this candidate; they are not
an optimized trajectory or a joint-angle teacher.

For each foot, cost is `min(abs(actual_height-target_height),60000)/60000`;
convert each component to Q16 by integer floor and sum. Its proposed reward
coefficient is -1. Actual height is native box minimum world Y, not ankle
origin, heel height or a contact-presence bit. Report geometry rounds the
existing floating measurement ties-even to micrometres. A native implementation
must separately verify its integer geometry against this measurement.

Post-step frame tick `t+1` is compared with target at action tick `t`, as in
V6's load term. This timing is tested. The continuous polynomial has zero
endpoint derivatives, but discretization and a tracking reward do not prove
that the actual plant can follow it or re-contact safely.

## Exact experiment

Script: [walking_lift_return_discriminator.py](../../lab/scripts/walking_lift_return_discriminator.py).
Evidence root: `/home/kaifaty/NextEngine-training/r8b-canonical-walking-v2/evidence/`.
Final `lift-return-discriminator-02.json` SHA-256:
`a6ee91e693b672fe38620088e2d77bb92e5996184c149e9b7d053dd6ac66ef49`.
It closes script, descriptor, manifest, evaluation NPZ and both native traces.
The V6 native replay must exactly match the recorded evaluation before scoring.
The separate control requires three safe 105-tick V5 cases; V5/V6 have the same
body/actions. Full source identities remain in the preceding report.

Synthetic full-cycle costs (lower is better):

| State construction | Mean cost |
| --- | ---: |
| Exact phase lift and return | 0 |
| Grounded / heel-only tilt with minimum corner at zero | 0.333324 |
| Lift wrong foot | 0.666648 |
| Hold left foot at 60 mm | 0.999996 |
| Hold both feet at 60 mm | 1.666668 |

These are reward counterfactuals, NOT simulated successful trajectories.
On all 262 moving V6 frames, measured mean cost is 0.320730 versus 0.321152
for grounded feet under the same phase sequence. The learned rocking earns
only about 0.13% improvement in this component; it is not mistaken for a step.

Native positive controls use zero command, so temporal credit is deliberately
NOT fabricated. At each side's fixed peak phase, score **every** recorded
frame: left release has 41/105 frames better than grounded, right 18/105;
the opposite side and the zero tape have none. Complete-sole clearance exceeds
30 mm on 33/37 frames. These controls establish geometric sensitivity, not
phase alignment or coordinated return. Over-lifting beyond 120 mm saturates
the candidate's error; this explains why clearance counts and improvement
counts need not agree.

## Decision, remaining uncertainty and next action

The discriminator passes its stated purpose. Proceed to a separately versioned
native V7 environment with this additional cost and two observed sole heights.
Preserve the first 86 observations, all existing reward terms, body, action
meaning, safety and original walking acceptance. Geometry alone does not
distinguish every unloaded/airborne state, so retain the existing load term.
Do not claim force-free hovering or synthetic target states prove walking.

Next: implement native integer geometry and reward, compare identical-action
V6/V7 physics and safety, check Python/native target agreement, then freeze a
new recipe with explicit sample budget and final checkpoint selection. No
repeat of V6 and no optimizer launch before this integration is validated.
Observation history and the required training duration remain unresolved;
the current evidence does not justify changing them simultaneously with the
objective. A failed native geometry comparison reopens this candidate before
training. A subsequent failed run requires physical outcome diagnosis, not
weakening travel, support or safety gates.

## Verification

- PASS: 10 focused tests (five candidate and five sole-audit), including phase
  timing, target symmetry/endpoints, wrong side, persistent lift and geometry.
- PASS: Ruff check and formatting; exact V6 replay in the CLI experiment.
- The first CLI attempt exposed a missing keyword argument to the existing
  external-path validator; it failed before writing evidence. Corrected before
  the recorded experiment. Final report -02 adds explicit V6 identity and
  descriptor closure; -01 remains immutable with the same numerical findings.
- NOT RUN: new native environment, optimizer, full walking evaluation or
  runtime admission. This localized diagnostic tool does not require Cargo
  or a repeated broad host-check.
