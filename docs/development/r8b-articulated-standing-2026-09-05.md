# R8b articulated-foot contact and standing diagnostic

Status: `CONSUMERS_IMPLEMENTED / NOMINAL_TIMEOUT / DISTURBANCE_OPEN`.
Authority: [ADR-119](../architecture/adr/119-articulated-foot-standing-diagnostics.md).
Body: unchanged [V8 implementation](r8b-articulated-foot-v8-2026-09-05.md).

## Implementation

The old 6 Ns contact ceiling applies per body pair. Simply using the old
classifier after splitting the foot would allow rear/toe contacts individually
below the ceiling even when their combined impulse exceeds it. The new exact-V8
classifier adds a ground-oriented anatomical-foot vector sum before activity
filtering. It preserves old pair limits (including under vector cancellation),
non-foot classifications and validated endpoint projection. Checked arithmetic
rejects excessive sums without mutating continuity. One pre-existing unchecked
square in the common magnitude helper now fails with NumericOverflow instead
of a debug panic; normal old-profile results are unchanged.

Anatomical continuity survives rear-to-toe transfer without merging the physical
pair records. New contact roots bind full compiled body, subject, profile,
raw foot totals and foot counters as appropriate. The terminal successor
requires that profile; the standing V3 reference validates the complete V8
compilation and applies the existing upright equations with neutral MTP targets.
No geometry, masses, gains, ROM, effort or impact limit was retuned.

The existing standing example gains one strict diagnostic command; all old
commands remain. Variable-size effort arrays are necessary for 25 DOFs; the
old 23-DOF response-probe path retains its exact array contract.

## Bounded native test

Question: does replacing the rigid foot with V8 under the retained upright law
complete nominal standing, or introduce an observed safety/contact failure?
One 30-second episode, no disturbance, no parameter search. Keep every substep;
stop at existing safety/terminal failure or 1,800 motor ticks. A timeout does
not certify quiet joint velocities, loaded heel rise, re-contact or robustness.

Command, with pinned native SDK `f259d3da...` from task-state:

```sh
cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 8 0 0 articulated-v3 per-iteration unchanged
```

Observed: `terminal.timeout`, all 7,200 physics substeps and 1,801 pose samples.
Both anatomical feet remain active throughout; neither crosses 6 Ns. Descriptive
orientation values use native 60 Hz samples and normalized diagnostic quaternions:

| Observable | Result |
| --- | --- |
| Whole-episode maximum pelvis / torso tilt | 2.4953° / 8.1712° |
| Last-10-second maximum pelvis / torso tilt | 1.3251° / 0.14994° |
| Last-10-second maximum rear-foot tilt, L/R | 0.01131° / 0.01021° |
| Last-10-second maximum forefoot tilt, L/R | 0.001987° / 0.006750° |
| Maximum anatomical foot impulse, L/R | 1.909552 / 1.866850 Ns |
| Last-10-second mean absolute vertical load, left rear/toe | 384.820 / 12.469 N |
| Same, right rear/toe | 326.783 / 14.829 N |

The toe bodies bear nonzero mean load, not just decorative shapes. Contact-role
continuity is not a sole-normal test; recorded pose geometry supplies the
separate nominal flatness observation. These descriptive measurements are not
an independent mathematical stability proof or disturbance criterion.

The existing actuator summary also exposes a limitation: final-eight-second
MTP velocity RMS is 0.02471 / 0.11115 rad/s, spectral peaks116.375 /120 Hz, despite
small recorded joint positions (L[-0.000249,-0.000098], R[-0.000227,0.002769] rad
over the complete trace). Do not describe every joint as quiet or tune damping
from a final image. A loaded transfer trial and actual position/velocity/effort
comparison are needed to distinguish contact/solver rate behavior from motion.

## Evidence identities

External root:
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/articulated-standing-01`.

- `standing.json` SHA256 `d1e8aab3117dded83287ef49726050e4fb47a63f0542e87bdb2f1e86b8130e29`.
- Body hash `0424903f748c0922bafaf77c2312c2ba8b820fbe9b3aad0bf276e2f37f995533`.
- Compiled V4 hash `4b8de3a8275c8dfb28ff44e15a8736edd02463d8632a6e9bc113f1e6f541f60d`.
- Contact profile `5bba92cda2e5a16cec5cb9a4e7be3162caa5d619ea235a1989c9498c17a9e4d8`.
- Standing reset root `5b91b2145041d84261d030c1ed24038a7a5b18fd9874a66a6921bfff32405785`.
- `v7-control.json` is byte-exact to the prior `body-v7-01/standing-subject-bound.json`,
  SHA256 `6eef44ec589dbb1b28069165fc1784e55bd5c6c81fbdf06c3c3ae5fac465c720`.
- `inspection.json` and `inspect_standing.py` contain descriptive measurements;
  the script uses existing `native_body_geometry.py` and `next_lab.motion_math`.
  `standing-preview.png` shows actual colliders at0/3/30 seconds, not a skin mesh.
- `actuator-summary.json` is produced by the unchanged
  `lab.scripts.audit_standing_actuator_response` with the V8 inspection descriptor.

## Verification and next action

PASS:149 native motor tests,5 native example tests, all-target native Clippy,
the unchanged V7 full-output comparison, boundary-scan and content-package.
New manufactured tests cover inclusive/split limits, inactive contribution,
per-segment cancellation protection, reversal/order/manifold invariance,
transfer/gap/reset, malformed/overflow atomicity, terminal mismatch/impact,
and exact body/subject/reset/reference vectors. Formatting, diff whitespace and
local documentation links pass. `persistence-replay` and `play` pass. The first
`play` process ended with SIGTERM (exit143), without a result or diagnosis;
its retained `play.log` is not a success. After that confirmed terminal result,
the fresh invocation completed PASS in `play-r2.log`. No criterion/code change
was made between them. Broad host-check and performance were not run for this
package-local opt-in diagnostic; no runtime/performance admission is claimed.

Next: loaded heel-rise/re-contact with explicit bounded reference inputs and
actual ground-contact geometry. Then bounded disturbances and recovery, before
any learning/environment selection. Keep V7 as rollback/control, keep the full
body/mass/leaning goal open, and do not modify toe gains solely from its velocity
spectrum. No optimizer, checkpoint reuse, mirror or game default was selected.
