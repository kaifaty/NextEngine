# BODY-COMBINED-01.r2 and conditional V11 standing

## Frozen contract

Base a80d5663, Accepted SPEC-35 and ADR-119/122/123. Full calibration remains
the four requirements in BODY-GAIN-NATIVE-01, not this bounded test alone.
Previous r1 is a measured one-second-return settling FAIL; preserve it.

## R2: equal observation time, fixed body and input

One exact V11 vector, original paired-abduction step input. No new gain,
amplitude, safety threshold, smoothing or collider change. Extend return
observation to two seconds, matching the two seconds following the initial
step: hold0–1s, step1–3s, return3–5s. Raise the free initial root200m instead
of100m so full-gravity free fall remains clear of ground over5s (~123m fall).
All other reset joint coordinates, full240Hz PD/60Hz targets,16 position
iterations and contact/safety rejection are unchanged. Use explicit new
`abduction-tail` input identity/case28, not overwrite r1 or its case27.

Reuse the independently checked177-pair/1001-configuration joint-path geometry
only after its descriptor/helper/raw seals match; global translation does not
change this f64 input geometry. Do not infer native translation invariance:
compare all first960 joint observations, targets/efforts/flags against r1 and
report differences. A difference limits causal comparison, but does not erase
the actual new200m-fixture result. Keep all1200 physics samples and raw contacts.

R2 response criterion: all1200 substeps finish; every joint plateau RMSE over
(2.5,3] and return RMSE over(4.5,5] <=0.01rad; every joint final RMS speed over
(4.5,5] <=0.05rad/s. Preserve the original(3.5,4] metrics too. Report all-channel
half-second RMS and correctly normalized240Hz spectra, not just left elbow.
This is explicitly a two-second-settling requirement, not a pass of r1's
one-second criterion and not a walking bandwidth or global stability proof.

H1: finite decaying transient explains r1; second return second becomes quiet.
H2: persistent oscillation/coupling remains; final channel criteria still fail.
H3: fixture/operation mismatch limits comparison. Native prefix equality and
independent effort mapping check H3; actual late samples discriminate H1/H2.
Budget: V11+V8 control, exact V11 repeat, old case27 byte regression; one fresh
independent executable review, at most one batched apparatus repair/re-review.
No new gain vector or different time window under r2 if it fails.

## Conditional loaded standing, separate empirical boundary

After r2 passes its independent review, explicitly admit only exact canonical
V11 to diagnostic articulated contact/terminal/standing consumers. Their
equations/limits stay identical to V8/V10; body/subject/reset/profile binding
gets a new identity. This is a test of an existing nominal law under softened
gains, not implicit training admission or a selected calibrated controller.

One fresh authored ground reset, gravity/free pelvis, full safety/contact rules,
1800ticks/7200substeps. Same gate as BODY-GAIN-NATIVE-01: timeout without another
terminal/safety failure; last10s root tilt<=2deg, torso<=1deg, both feet loaded,
neither MTP tail RMS speed above the exact V8 control. V11 repeat and unchanged
V8 control required. Record earliest failure/all efforts/raw contacts. Stop on
failure instead of rerunning the same failed prefix for transfer/disturbances.
On pass, loaded transfer/heel-rise/recontact and all-channel loaded responses
plus disturbance recovery remain required, not silently credited here.

H4: unloaded bandwidth correction also supports the original stance. H5:
softer gains need load compensation/another controller, even with good unloaded
response. This native test distinguishes sufficiency, not unique causality.
Budget: conditional three standing worlds, one independent review of the new
consumer/standing boundary, at most one batched repair/re-review. Do not rerun
unchanged ancestor research. Shared exact-base controls remain labeled as such.
No optimizer, native safety relaxation, geometry or mass adjustment.

## Results: response passes, loaded standing fails

External evidence: `/home/kaifaty/NextEngine-training/body-settling-nZsPFi`.
Response manifest SHA256
`dba948380df3836ca86ca31bfb6e3dbfd0a273027588326f6b42fa2f5250b181`;
standing manifest SHA256
`8c8b47a92fb1083b3481f54b0c7e884565e1670b4dc81dc73640e7268c9cc57b`.
The original pre-result contract is preserved as external `contract.md`.
Source snapshots, patches, binaries, commands, descriptors and raw traces are
sealed separately for the two executable boundaries.

V11 completes all1200 unloaded substeps. Worst plateau RMSE is0.000716849rad,
return RMSE0.000857849rad, final RMS speed0.00835923rad/s. All joint criteria
pass the new two-second-settling test. The old return window still fails:
left-elbow RMS speed0.05503818rad/s. H1 is supported within the new fixture;
do not credit r1 or walking bandwidth. Repeat is byte-exact; old case27 output
also repeats exactly. V8 fails its velocity guard at159, before the step input;
its missing late windows are not zeros or passes.

Elevation is not native-bit-invariant. Against r1's first960 V11 substeps,
953 joint records and813 effort records differ, first at6/7 respectively;
maximum position/velocity/effort differences are2urad,234urad/s,474microNm.
All applied targets and flags match. V8 amplifies this difference and fails
early; its exact mechanism is not isolated. Geometry reuse is limited to the
sealed177-pair/1001-pose joint-space calculation, not native trajectory equality.

Fresh independent response review:
`/home/kaifaty/NextEngine-training/body-settling-review-C1z0nH/review.md`, SHA256
`b8ed62bcbb20781a5d587e9fad37a7e328139af8f070c41d374e3f52b5ae9219`.
It reconstructs all30,000 V11 and3,975 V8 applied effort channels, verifies
metrics/prefix differences/geometry seals and accepts bounded r2 PASS without
an apparatus repair. The example's opening comment still names r1; this is
non-load-bearing, since explicit CLI/header/case28 identify r2.

Only after that response review, ADR-124's exact V11 consumers were used with
`11 0 0 bandwidth-v6 per-iteration unchanged`. Standing stops at substep346
(1.441667s), motor tick87's second substep, on bilateral ankle-pitch hard ROM:
525995/526003urad versus upper523599urad plus10urad observation tolerance.
No earlier joint/contact/root failure was found. Final foot impulses are
3.383471/4.067221Ns, below6Ns; the safety-before-classification final frame's
raw contacts are retained. Repeat is byte-exact. V8 completes7200 substeps and
is byte-identical to the prior nominal-standing control. H4 is refuted: this
unchanged standing law is insufficient with V11. H5 remains a controller/load
hypothesis, not uniquely identified gravity compensation or defective anatomy.

Fresh standing review:
`/home/kaifaty/NextEngine-training/body-standing-review-aexoSN/review.md`, SHA256
`5572072b42bac42dcad111b1fc976b4ed5ea1d1f73ba443cd03397d8b8d09c14`.
All55 manifest and17 source-snapshot hashes match. Independently reconstructed
8,650 V11 and180,000 V8 efforts, all references/targets, identity envelopes
and final raw contact classification match. A focused hash-matched binary
rerun is byte-exact. No load-bearing apparatus defect or repair was found.
Candidate tail posture/load metrics are NOT_TESTED because that horizon is
absent, not fabricated failing tail scores. V11 raw SHA256
`04971a42a6353fc04e0b64656256ca53634b76f74795672a6095150db017ef3f`.

## Exploratory next-boundary triage (not independently reviewed)

`load_prefix_triage.py` SHA256
`4135a842bfdc255b1b1a85ecbac04788aa805c0b1d00587f40d61fa2a8eb6a78`;
`load-prefix-triage.json` SHA256
`cd1186bbed0b501131292f21056c477392305fad2eda827ffd88d72052ac992f`.
These post-seal files do not modify either sealed experiment.
Read-only f64 reconstruction uses all26 body masses/local COMs and normalized
observed xyzw orientations at60Hz, with nonzero ground-contact impulse points.
At the first sampled sagittal COM excursion behind that active-point span,
substep200 (0.833333s), V11 COM Z=-0.1319065m versus rear point-0.130045m;
ankle angle is still-0.171045rad, target-0.211519rad, effort-0.98105Nm.
At failure, excursion is0.364567m and left ankle effort-14.487889Nm. V8 has no
sampled rear-span excursion over30s. Thus the exploratory causal order places
balance drift before ROM, not a ROM limit as the initiating event. This is
neither a dynamic viability proof nor a calibrated controller result.

Next discriminate load-compensation/control authority from feedback-law error
using unchanged safety and explicit target/effort boundaries; inspect existing
inverse-dynamics contracts before adding a feed-forward path. NVIDIA's
[articulation stability guide](https://nvidia-omniverse.github.io/PhysX/ovphysx/latest/guides/articulation_stability.html)
(read2026-09-06, page updated2026-08-21) recommends feed-forward effort for
gravity/acceleration instead of excessive stiffness. This motivates the test,
not proof about our pinned native5.9 implementation. Do not restart an
unloaded-only gain sweep, widen ROM or rerun the failed standing prefix as
transfer/disturbance tests. Full loaded response, heel-rise/recontact,
disturbance recovery and final profile selection remain open.

## Verification

Passed:157 native motor library tests,12 example tests, native all-target
Clippy with warnings denied, formatting, boundary-scan6, content-package,
play and persistence-replay; five invalid V11 CLI combinations reject before
native output. Candidate/legacy response and standing repeat checks pass.
Broad host/performance and training/mirror: NOT_RUN, outside this localized
diagnostic change. Software checks do not turn physical standing FAIL into PASS.
