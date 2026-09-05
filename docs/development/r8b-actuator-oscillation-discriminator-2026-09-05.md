# Standing actuator discriminator: shoulder rotation is not passive anatomy

Continuation of the [force-schedule investigation](r8b-force-schedule-profile-2026-09-05.md).
This is a bounded native experiment, not a selected body successor or successful
balance repair. No training, mass redistribution, joint/foot geometry change,
safety relaxation or default environment change is made.

## Prediction and experiment

| Hypothesis | Discriminator | Result |
|---|---|---|
| Shoulder-yaw explicit PD generates the ~24 Hz rotation | Make only those two actuators nearly passive; retain native physics and all other commands | Supported: 20–30 Hz velocity power fraction drops from 0.800/0.746 to 0.003/0.005 |
| Those shoulder oscillations alone cause the standing lean / foot-impact failure | Restore small controlled shoulder motion and repeat the original stance and the existing k=2 hip-feedback case | Falsified as a sufficient explanation: body oscillation remains and hip-feedback still fails contact impact |
| Remaining near-step-frequency motion is a coupled discrete actuator/contact response | Measure local effort-to-velocity response from exactly reconstructed native histories, then compare coupled control response | UNRESOLVED / next test; do not infer a global stability bound from isolated link inertia |

The previous goal turn made progress: `c541b3d5` completed the opt-in native force
schedule boundary and passed checks. Current experiments change the next action
from generic root-feedback tuning to coupled actuator/contact diagnosis.

## Observed causal order

V6 with the new force schedule has shoulder-yaw targets of zero, yet both arms
repeatedly rotate at approximately 24 Hz. Whole-run left/right q ranges are
[-0.323, 0.402] / [-0.390, 0.378] rad, and speeds approach the existing 10 rad/s
limit. This is actual position motion, not only a velocity-report discrepancy.

The first left-shoulder step has q=23 microrad and velocity=0.011989 rad/s.
At the second step, the old gains request -0.171066 N m and velocity becomes
-0.138593 rad/s. With only both shoulder-yaw gains nearly removed, that
second-step effort is zero after integer quantization and velocity is
+0.024971 rad/s. The simultaneous bilateral change is not a measurement of
an isolated diagonal inertia or proof of whole-system linear stability.

The contract requires positive stiffness/damping; a passive actuator is not
represented by the existing schema. The diagnostic therefore uses **1 Q16**
for each gain, explicitly called `shoulder-yaw-near-passive`, through normal
schema validation / compilation / safety. Measured efforts stay below
0.000078 N m. This removes the ~24 Hz mode, but arms drift to their ROM limits.
It is rejected as a usable controlled body.

The restoring-control counterpart divides only these two channels' gains by
16: Kp 140 -> 8.75 N m/rad; Kd 14 -> 0.875 N m s/rad. This reduces the overly
large observed one-step feedback while preserving restoring action. The factor
is an experimental conservative attenuation, not a proved optimum. Existing
effort/rate/power/work, target-slew, ROM and contact limits remain unchanged.
Both candidate IDs and source-provenance hashes are distinct; compilation
hashes the actual changed actuator inputs. No frozen V6 factory is edited.

## Native outcomes

| Case | Duration / terminal | Shoulder-yaw q range L/R, rad | Last-10-s maximum torso tilt |
|---|---|---|---:|
| Unchanged control | 30 s / timeout | [-0.323, 0.402] / [-0.390, 0.378] | 8.595° |
| Near-passive shoulders | 30 s / timeout | approximately [-1.571, 1.571] on both | 10.830° |
| Gains /16, original reference | 30 s / timeout | [-0.006939, 0.005177] / [-0.007512, 0.005312] | 9.905° |
| Gains /16, existing k=2 hip feedback | 9.3 s / contact impact | [-0.004711, 0.003335] / [-0.005412, 0.003323] | not a completed 10-s window |

Original-reference shoulder velocity RMS in the last eight seconds falls from
5.692 / 5.980 to 0.07673 / 0.08226 rad/s. Residual small shoulder motion has
near-120-Hz content; do not claim all actuator oscillation is eliminated.
The k=2 run's left-foot summed normal impulse reaches **6.952268 N s** at
substep **2229**, above the unchanged 6 N s limit. Terminal evaluation ends
motor tick 558 / substep 2232. A final torso tilt of 0.860° is not success.
Neither the more upright terminal frame nor lower shoulder noise justifies
adopting this as the complete balance fix.

The new unchanged control reproduces all 1801 motor samples, all 7200 physics
samples and both body/compiled hashes of `profiled-tgs.json` exactly. The new
probe metadata/schema differs, so whole-file equality is not claimed.

## Reproduction and evidence

External directory:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/actuator-discriminator-01`.

| Trace | SHA-256 |
|---|---|
| `unchanged-control.json` | `c9f19a092a0e21e68b75866d6d4aaa9cebdd07354966bcc0bcf86717f7336706` |
| `near-passive-baseline.json` | `f95a289d0113c66d5e74891203a0c61521ce44d1c58e5011e9a7a312212035ff` |
| `gain-16-baseline.json` | `200615a8be4e71b7545cdc6b78917679c275afd7637358ce3fb0fa7404ce5a25` |
| `gain-16-hip-feedback.json` | `aa05124bbb48349e5dcc89955ad148b7c800c2896b6f1a73eed52493c97ea1f6` |

Run the existing standing example with arguments `6 0 0 baseline per-iteration`
and append `shoulder-yaw-near-passive` or `shoulder-yaw-gain-16` for the two
counterfactuals. Replace baseline with hip-feedback only to reproduce that
rejected reference. Use the explicit SDK environment from the task state.
The earliest near-passive trace has schema 9 without the later additive
`actuators` map; its `ordered_actuator_ids` and native joint ordinals are present.

The tracked [audit script](../../lab/scripts/audit_standing_actuator_response.py)
takes a V6 descriptor path and one trace path, writes JSON to stdout, and binds
descriptor, trace and script SHA-256. Descriptor use is restricted to label/DOF
mapping, not policy compatibility. All four `*-audit.json` outputs are external.
Spectrum window: final 8 s, DC removed, rectangular window, 240 Hz samples.
Shorter traces use their actual available window. This is descriptive evidence,
not a certificate inferred from frequency or average effort alone.

[Isaac Lab actuator documentation](https://isaac-sim.github.io/IsaacLab/develop/source/concepts/actuators.html)
was opened on 2026-09-05: it distinguishes explicit effort computation from
solver-integrated drives and cautions that contacts/limits can still destabilize
implicit drives. This supports examining the actuator boundary, not replacing
the pinned backend or claiming an implicit controller guarantees balance.
The author-hosted Stable PD PDF fetch timed out this turn; the earlier paper
read and its limited applicability remain recorded in the preceding report.

## Decision and checks

Keep the identified gain-/16 candidate as evidence for the shoulder mechanism;
do not select it as the new training body or keep searching shoulder gains to
fix the remaining trunk/leg/contact mode. Do not repeat the finished mass audit.
Next: a small, symmetric effort-response experiment at the same reconstructed
standing state, with two perturbation magnitudes to expose nonlinear/contact
sensitivity, before changing coupled torso/leg control. Baseline reconstruction
must match exactly; freeze geometry/mass and preserve original safety checks.

PASS: one native Rust discriminator test (exact permitted schema delta and bad
mode rejection); five Python spectrum/input-validation tests; example clippy
with warnings denied; Ruff format/lint; exact full unchanged trace comparison.
Full workspace/product checks are not rerun for this example-and-analysis-only
change; their prior PASS is at `c541b3d5`, not a new whole-product validation.
NOT_RUN: perturbation robustness, a new learning environment, foot articulation
and training. Overall human-body/balance goal remains open.
