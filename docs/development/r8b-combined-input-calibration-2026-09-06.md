# BODY-COMBINED-01 — frozen contract revision1

Base f19d28d0, Accepted SPEC-35 and ADR-122; native unloaded diagnostic only.
Consumer: determine whether a non-intersecting combined reference permits the
fixed V11 gains to complete a simultaneous step response. Preserve the failed
BODY-BANDWIDTH-01.r2 census and every gain/safety/body byte. Full calibration
still requires loaded response, heel-rise/recontact and disturbance recovery.

## Fixed discriminator

One input candidate: original reset (knees/elbows0.1rad, other joints0), then
all25 joints receive the same0.05rad step amplitude except both hip-roll signs
are negative (abduction). No amplitude, duration, gain or tolerance search.
Return to reset follows the same joint-space line in reverse. This changes the
combined test input, not anatomy or limits. Individual/sine r2 evidence stays
under its original identity; the successor is not a rerun of r2 case25.

Before native execution check1001 equally spaced f64 configurations on
q(s)=q0+s*delta, s=0..1. Enumerate all distinct-body collider pairs except exact
descriptor collision exclusions; same-body shapes are rigid compound geometry.
Use15-axis OBB SAT, sphere-sphere distance and exact sphere-OBB closest-point
distance. Require every sampled eligible pair separated by more than0.1mm;
report all pair minima, not only feet. Check soft-ROM bounds. Manufactured
overlap/separation/tangent controls and the prior all-positive foot-overlap
counterexample must retain their signs. This finite grid is not a continuous
collision-free certificate or guarantee about actual tracking trajectories.
Any sampled conflict stops native execution; no second input under this ID.

If the preflight passes, explicitly extend only the diagnostic input consumer,
then execute one V11 and one V8 control world with the original4s/960substep
fixture,100m initial height, gravity,60Hz references/240Hz explicit PD and all
existing safety/contact rejection. Step ticks61..180, rest zero. Every channel
is selected. Require horizon completion, each channel plateau2.5–3s and
return3.5–4s RMSE<=0.01rad and final-half-second RMS speed<=0.05rad/s. Retain
all240Hz states, raw contacts, targets, efforts/flags and frequency spectra;
missing tails are censored. Run one exact V11 repeat. No standing/load test
before explicit compatible consumer admission and a successful response.

## Hypotheses, review and stop

H1: geometric infeasibility dominates the old combined failure; the corrected
input clears preflight and native response. H2: transient coupling/clipping or
tracking still causes unsafe/poor response despite separated sampled targets.
H3: sign/frame/oracle disagreement makes the discriminator inconclusive.
Prior native reset/FK correspondence and independently reviewed efforts are
controls, not a global physical certificate. Budget: one1001-pose preflight,
old target/manufactured controls, conditional three native worlds and one
fresh independent executable review with at most one batched repair/re-review.
Seal contract/source/command/raw hashes before review; do not rerun unchanged
ancestor chains. No positive result alone means full calibration or permits
training. A sampled collision or native failure changes the next discriminator,
not the input/criteria in this revision.

## Captured results

Independently checked: sampled geometry preflight PASS, native full response FAIL.
All177 eligible collider pairs at1001 configurations have gap>0.1mm; minimum
30.405mm (torso/forearms at reset), soft ROM passes. Four original positive
target foot pairs retain their overlap signs. This supports only the finite
grid; native motion still needs its own collision checks.

Both native bodies complete960 steps without safety/forbidden-contact failure.
V11 worst plateau RMSE0.000716834rad and return RMSE0.004210065rad pass, but
left elbow final RMS speed0.055037742rad/s exceeds0.05. Right elbow is
0.048514108rad/s; all other channels are below0.05. V8 fails badly despite
reaching the horizon: worst plateau RMSE0.099952746rad, return0.082295133rad,
tail RMS speed7.234228125rad/s. No missing samples are discarded.

Thus removing target intersection is sufficient to avoid the old early contact
in this run, not sufficient for every response criterion. Do not label this
combined calibration passed or retune only to cross a nearly missed threshold.
V11 repeat is byte-exact; the unchanged no-flag27-world V11 output is byte-exact
to r2, including its failed case25. That additional27-world software regression
is required by ADR-123, separate from the three-world successor input budget.

Post-result localization retains all half-second elbow windows. Left-elbow RMS
after the first step is0.326745,0.058953,0.008072,0.004444rad/s in successive
half-seconds; after return it is0.325561,0.055038rad/s over the available second.
The final window's demeaned spectrum peaks in the2Hz bin (2Hz resolution),
with about0.60% power above60Hz. This supports a decaying low-frequency
transient, not a demonstrated sustained near-Nyquist limit cycle. It does not
measure the unrecorded second return second or prove future decay.

Smallest next discriminator: unchanged V11 gains and same paired-abduction
input, but separately freeze a time-symmetric two-second return observation
to distinguish finite settling time from persistent oscillation. Preserve this
r1 failure; do not retroactively shift its window or threshold. A longer raised
trial must explicitly preserve gravity and avoid the100m floor impact near4.5s;
do not silently extend this fixture beyond its safe unloaded horizon. Loaded
support/recontact/disturbances remain required and NOT_RUN; no training.

## Evidence and checks

External root `/home/kaifaty/NextEngine-training/body-combined-CHcslR`.
Entry `manifest.json` SHA256
`9ef3b6b0de588648080123f1c58411b44b71ba84c900a0c53d7277ee8cba3318` seals the
original contract, complete candidate diff, geometry/control/raw native data,
binary and commands. Its earlier pre-content-completion seal is preserved;
only the completed content log hash changed before review entry, not any
native/source/geometry evidence. Initial geometry contract hash resolves to
`contract-r1.md`; the current report appends outcomes after that freeze.

Geometry result SHA256
`455107b41d9d25f694661ff9a35abe90923c0a65b166ea2920e3fccbd05f8e37`;
post-seal author analysis SHA256
`fb5294914439d6b0934757799e8b66ec75f80a4d4deda2c69db194f68b270278`;
post-result elbow localization SHA256
`507b1c10d448e4e8e8acd8d4c921ce12a0657f8be617f1d2a5fd64c13324ada2`.
Scripts/results include input and script hashes; Python3.12/NumPy2.5.2.

PASS: two native example tests, focused native example Clippy-Dwarnings,
format/diff, six boundary checks, content-package, candidate repeat and old
no-flag native byte regression. First content invocation mistakenly used the
unknown `product-check` wrapper; the actual `xtask content-package` passed.
The failed CLI log is retained, not a product failure hidden by retrying.
Native library156 tests remain exact-base evidence, not rerun for this
example-only edit. Broad host/play/replay/platform/performance and training
NOT_RUN (unchanged library/standing/game consumers). All760 local documentation
links pass. Independent executable review completed in one pass without repair.

## Independent review closure

`/home/kaifaty/NextEngine-training/body-combined-review-S9tNSp/independent-review.md`
SHA256 `b6b61a166ba816a656ed961a2b5d2d3461c410776ddbeaa82819083422765b11`.
All nine review artifacts in its SHA256SUMS verify. Independent homogeneous
transforms/projected vertices and sphere-plane distances reproduce177177 pair
checks (maximum author gap difference4.45e-16m). The separately authored,
previously reviewed controller arithmetic is reused with declared lineage;
all48000 applied effort channels/flags and both failure boundaries agree.
Every position criterion passes for V11; only left-elbow settling fails.
No load-bearing apparatus defect, new native rerun or ancestor execution.

All3827 V8 and5478 V11 raw contact records have zero canonical impulse and none
is active; do not call these absent constraints. The independent full spectra
retain all481 bins/all25 channels and satisfy Parseval correspondence. V11
maximum above60Hz mean-square speed is0.000209262(rad/s)^2 versus V8's10.522613;
this descriptive result does not introduce a new acceptance criterion.

The post-result elbow RMS windows also match independently. Non-load-bearing
note: author tail.py omits one-sided interior-bin doubling in its descriptive
power fraction. Corrected final left-elbow fraction is0.005952187 rather than
0.006031727; both are about0.60%, and neither affects RMS/the failed criterion.
The original artifact and this correction are retained, without another review
loop. Later return settling remains unobserved; full calibration remains open.
