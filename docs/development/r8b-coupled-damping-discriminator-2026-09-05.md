# Coupled damping discriminator — revision 1

Research ID `r8b-coupled-damping-discriminator.v1`, baseline `afb1971b` with
original native bridge restored. Consumer: reduce actual closed-loop standing
oscillation without hiding it through body mass, geometry or solver changes.
Accepted semantics/defaults unchanged; example-only newly hashed BodySchema.

## Observation and competing mechanisms

Existing `actuator-discriminator-01/gain-16-baseline.json` (SHA
`200615a8be4e71b7545cdc6b78917679c275afd7637358ce3fb0fa7404ce5a25`)
has final-eight-second joint velocity spectral peaks near 119.875–120 Hz in
hip pitch/yaw, knees and torso pitch/yaw. Recorded explicit efforts frequently
differ from unclamped PD in these channels. This is not proof of instability
or of which coupled channel originates it. The previous shoulder-yaw-only
repair does not remove this remaining mode.

H1: excessive sampled damping in these coupled channels is necessary for this
near-Nyquist motion; a fourfold damping decrease reduces it while stiffness
and static reference remain unchanged. H2: contact response or another actuator
continues to force the mode; reduction fails the frozen motion criterion or
causes earlier safety failure. Coupling means a successful group ablation does
not isolate a single joint. No inverse-mass fit from the inconsistent contact
map is permitted or used.

## Finite experiment and criteria

Starting from V6 shoulder-yaw-gain-/16, divide damping Q16 by4 for precisely
eight channels: bilateral hip-pitch, hip-yaw and knee, plus torso-pitch and
torso-yaw. Keep all stiffnesses, remaining damping, geometry, inertia, forces,
limits, safety, timestep240 Hz and original16/4 TGS iterations unchanged.
Derive a distinct schema/source hash using the normal compiler; assert the
complete input delta in a unit test. There is no native override.

Run one30-second baseline reference. Stop at the first existing terminal.
If it safely times out at7200 substeps, run the existing k=2 hip feedback once.
No further gain values or target changes under v1. Compare final8-second
velocity RMS and absolute spectral energy above80 Hz, using all23 ordered
joint channels, the existing uniform240 Hz trace and identical FFT window.
For H1 support require baseline safe timeout and at least50% reduction in both
selected-channel aggregate RMS and >80 Hz energy. Aggregate RMS is square
root of the mean squared velocities across1920 steps and8 channels; energy is
the sum of squared demeaned rectangular-window rFFT magnitudes above80 Hz.
Also report unselected channels and full root/torso tilt. This numerical
criterion diagnoses the sampled mode, not human-like standing acceptance.

Hip reference is judged separately: any original joint/contact/fall terminal
is failure even when the torso is upright. Preserve failures. Review one
frozen source/raw snapshot independently, at most one batched repair/re-review.
No training or body-default promotion. Old diagnostic output must remain
byte-exact after the additive example refactor. External artifacts belong in
`/home/kaifaty/NextEngine-training/r8b-human-body-mass-2026-09-05/coupled-damping-discriminator-01/`.

Stop with INCONCLUSIVE on correspondence/control failure. Reconsider a failed
scale only with new causal evidence, not another arbitrary damping sweep.

## Outcome: sampled mode suppressed, hip follow-up still noisy

`SUPPORTED_BOUNDED` for attenuation in the declared nominal baseline, not
necessity of damping as a cause, asymptotic stability or training admission.
Independent `damping_review` verified the exact8-channel source delta, all
raw hashes, unchanged native path and a byte-exact baseline rerun. Selected
DOFs are0,2,3,6,8,9,12,14. Both50% attenuation criteria pass:

| Last8-second aggregate | Original | Damping-/4 |
| --- | ---: | ---: |
| Selected RMS, rad/s | 0.345459039 | 0.004868615 |
| Selected squared FFT sum above80 Hz | 1446465.804572 | 307.059626 |
| Unselected RMS, rad/s | 0.102715530 | 0.041400024 |
| Unselected squared FFT sum above80 Hz | 162165.146512 | 23135.126877 |

RMS falls98.59%, spectral sum99.979%. The FFT sum is not mechanical energy
in joules. Calculation is independently reproducible from the final1920
`substep_samples`: decode all velocities /1e6, select the declared DOFs,
`sqrt(mean(v**2))`; subtract each channel mean, rectangular `rfft(axis=0)`,
sum squared magnitudes at `rfftfreq(1920,1/240)>80`. Do not infer an inverse
mass or a smooth contact Jacobian from this result.

Both baseline and k=2 hip follow-up safely time out at7200 substeps. Baseline
root peak/final tilt6.827133/3.438924 degrees; recorded60 Hz torso
13.353577/2.897551. Last10-second maxima3.449529/2.921099. Hip follow-up root
peak/final3.849404/0.660284, torso9.125046/1.423154; last10-second maxima
0.938468/1.519184. Full-run peak foot impulses are1.696453/1.676289 N s for
baseline,4.763868/3.702696 for hip feedback, below the unchanged6 N s ceiling.

However hip selected-channel RMS is0.897786950 rad/s and >80 Hz squared FFT
sum189952.349142. Upright appearance is not quiet standing. All23 DOFs and
7200 steps are recorded sequentially; safety runs every physics step and
contact evaluation precedes timeout. Realized ankle targets vary with the
trajectory even though the reference law is unchanged. Next isolate the hip
rate term through the [distinct discriminator](r8b-hip-rate-feedback-discriminator-2026-09-05.md).

| Evidence in external `coupled-damping-discriminator-01/` | SHA-256 |
| --- | --- |
| Frozen contract before results | `ac9a76693680644e9f1a44fc8e93804b0aabc3449821a7aaeb1912f0844c12d1` |
| Native main at measurement | `4e845e16382e1bdf91a8b2a22f08fa0c24d25bd2df690ccdf4840cee488bbef4` |
| `baseline.json` | `e34cf5055e6f435869d170ff4c0ec10f53912a83d32f225354d084a9ec3a2af2` |
| `hip-feedback.json` | `d183a831df634dc8a19766a5f47ded64498e7b4172417ac5a773df1e3aa8dacb` |
| `unchanged-control.json` | `899de64b39cc4fbb2e5fea897100970f30e04742b72ba933f2b9e1c1a81ceef2` |
| `frozen-head-control.json` | `899de64b39cc4fbb2e5fea897100970f30e04742b72ba933f2b9e1c1a81ceef2` |

New body hash `c63ec6b8ff0761045dc6b30c5a80a21ef22b9d114fd757a195e3f3f65bd8d86c`,
compiled hash `719a6661ad98f645d4d8570a1e927d0e33607e7cfb4e9fab1430be43494cc040`.
The historical raw control predates `response_probe:null`, explaining its
different byte hash. Every historical field is exact. Strict non-regression
against the explicitly frozen `afb1971b` implementation also passes: its
exact main source (SHAb48e138b…) was built as a temporary second example with
unchanged helpers/bridge and generated `frozen-head-control.json`, byte-equal
to the new unchanged mode. That temporary duplicate source is removed.

Four native example tests, focused native Clippy and formatting passed. No
library/default/training change; broad ProductChecks not rerun for this
additive example-only discriminator. Robust balance and physical foot
successor remain open. Source IDs and raw metadata distinguish this candidate
from old learned weights; these results do not make them interchangeable.
