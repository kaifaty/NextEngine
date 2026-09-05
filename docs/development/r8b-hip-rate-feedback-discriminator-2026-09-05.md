# Hip rate-feedback discriminator — revision 1

Research ID `r8b-hip-rate-feedback-discriminator.v1`. Consumer: retain the
upright correction without reintroducing fast joint motion. This follows the
independently reviewed coupled-damping experiment, not another damping sweep.

The damping-/4 baseline is quiet but leans; existing k=2 hip feedback survives
30 seconds and is more upright, yet selected-channel tail velocity RMS rises
from0.004868615 to0.897786950 rad/s. Its law, sampled at60 Hz, is hip target
`-2 * pitch_proxy - omega_x / 5` in microradians. Joint PD operates at240 Hz.
H1: the direct rate term contributes to re-excitation through this loop;
removing it preserves useful position correction with lower motion. H2:
position feedback, body/control coupling or contact still excites motion or
loses safety. Passing a single ablation supports a bounded remedy, not proof
of a universal mechanism or stability theorem.

Change exactly the hip reference rate coefficient from1/5 to0. Keep k=2
position term, existing ankle reference, newly hashed eight-channel damping-/4
body,16/4 TGS,240 Hz physics/60 Hz target updates, mass/inertia/geometry and
all safety unchanged. New explicit CLI reference `hip-position-feedback`;
no old mode semantics change and no default selection. No learned weights.

Budget one30-second run from the same reset, first original terminal stops.
Positive nominal result requires timeout7200 plus last8-second selected-channel
RMS <=0.02 rad/s, and last10-second maximum root tilt <=2 degrees and recorded
torso tilt <=3 degrees. Use the same eight DOFs, units, normalized quaternion
tilt and sample windows as the prior report; torso is recorded at60 Hz, root
at240 Hz. These are provisional nominal-quality criteria, not global or
disturbance stability. Also report full transients, >80 Hz FFT sum, both foot
tilts and ground loads; an upright terminal frame cannot override failure.

No gain/rate sweep or fallback candidate under v1. If failure persists, retain
the negative result and reconsider the control structure. Require unchanged
old baseline/hip behavior by focused exact-output comparison; freeze source
and raw hashes for one independent review, at most one batched repair/re-review.
Data stays under external body-audit root `hip-rate-feedback-discriminator-01/`.
No architecture/default/training promotion follows this experiment alone.

## Outcome: quiet upright nominal standing

`SUPPORTED_BOUNDED` for all frozen nominal criteria. Independent
`hip_rate_review` inspected the one-term ablation, complete actual target
arrays, unchanged reset/body/native/safety metadata and independently reran
the candidate byte-exactly. This does not establish global or disturbance
stability and is not a learned policy.

The candidate safely times out at7200 physics steps/30 seconds. Selected
last8-second joint velocity RMS is0.003916369864 rad/s (criterion <=0.02),
unnormalized squared FFT sum above80 Hz49.73358680. Last10-second maximum
normalized root tilt1.303655627 degrees (<=2), recorded torso tilt
0.181539031 (<=3). Final root/torso1.298098232/0.174740421 degrees. Whole-run
peaks remain2.468767893/8.192405023 degrees: the initial torso transient is
not removed or excluded from the report. Torso/foot poses are sampled at60 Hz,
root/joints/contacts at240 Hz; do not claim unsampled torso maxima.
The torso peak occurs at0.35 s, root peak at1.85 s; selected instantaneous
joint speed reaches2.645325 rad/s during the full transient. All23-channel
tail RMS is0.026439108 rad/s: the <=0.02 frozen criterion applies only to the
eight explicitly selected channels, not every joint or all motion.

Both soles remain nearly horizontal in recorded frames. Left/right full-run
maximum tilt0.011029089/0.009389823 degrees; last10-second maxima
0.009231882/0.008332584. Their last10-second vertical support impulse ranges
are1.631512–1.648531 and1.430688–1.445913 N s per physics step. Full-run maxima
1.828728/1.851835 N s stay below the unchanged6 N s safety ceiling. This is
loaded flat support for the existing rigid feet, not a forefoot articulation,
walking toe-off or perturbation test.
Both feet have positive load at every recorded physics substep. Minimum
substep-average vertical loads (impulse times240 Hz) are248.3364/252.3204 N;
last10-second means393.587927/345.312775 N. All43452 nonfoot contact records
have zero impulse and separation >=20500 micrometres. These quantities were
independently recomputed; no contact was hidden to obtain timeout.

| Evidence in external `hip-rate-feedback-discriminator-01/` | SHA-256 |
| --- | --- |
| Frozen contract before results | `85e2a1e2b22577aab7bd7809ab35fcb529a2111270e94c40f6adaf7c516c58ad` |
| Native main at measurement | `339ea0b7be32c5b933c3208db24ae9972683ba8b7d9c22137f33278365fc77e5` |
| `position-only.json` | `49cb7fbffe6762f47e6ae00e530f6b7d5e892d5ae5d7470a7eae5aaace4054df` |
| `unchanged-baseline.json` | `e34cf5055e6f435869d170ff4c0ec10f53912a83d32f225354d084a9ec3a2af2` |
| `unchanged-hip.json` | `d183a831df634dc8a19766a5f47ded64498e7b4172417ac5a773df1e3aa8dacb` |

Both predecessor modes retain exact output hashes after the additive reference
change. Run with the pinned SDK from task-state:
`cargo run -p next_motor --features physx-sdk --example probe_biomechanics_body_standing -- 6 0 0 hip-position-feedback per-iteration coupled-damping-4`.
The body/compiled identities are those in the prior damping report; the
reference is additionally identified by the output mode and recorded targets.
No new production reference profile or training manifest exists yet.

Four native example tests, focused native Clippy, format and two invalid CLI
profile/combination rejections pass. Broad ProductChecks NOT_RUN because this
is an additive diagnostic example; previous profiles and bridge are unchanged.
The generated `standing-comparison.png` was visually inspected: it uses all
actual V6 colliders with unchanged geometry, final poses and full tilt curves,
not a skeleton-origin approximation or shortened/cherry-picked time window.

Decision: stop friction/iteration/gain sweeps. The next implementation is an
explicit production body/control successor binding these exact gains and
reference law through the architecture/profile workflow, followed by bounded
disturbance and foot-mechanics checks. Robust recovery, realistic articulated
foot behavior and compatible learned locomotion remain unverified. Do not
silently use this candidate under V4/V6 checkpoint identities.
