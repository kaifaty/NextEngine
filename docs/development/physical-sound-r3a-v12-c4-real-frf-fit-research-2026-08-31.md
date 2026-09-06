# Physical Sound R3A V12-C4 — one-shot real FRF fit research

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `BOUNDED_RESEARCH_COMPLETE / PROTOCOL_INPUT` |
| Target | ObjectFolder-Real `41 / Wrench_Large / Steel` |
| Parent evidence | [C3 exact result](physical-sound-r3a-v12-c3-object41-source-role-freeze-result-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Falsifiable question

Can sixteen paired one-shot force/microphone recordings identify one stable
common-pole object model without pretending that single-record coherence is
evidence, and without opening development or protected contacts?

Three explanations must remain distinguishable:

1. the published force ensemble does not excite a useful shared band;
2. the acquisition is usable, but the current regularized FRF/common-pole
   representation is unstable or cannot reconstruct the microphone response;
3. the representation is stable on fit contacts and may proceed to a separately
   preregistered development stage.

The first outcome is `DATA_INSUFFICIENT_FORCE_COVERAGE`, the second is a valid
representation reject, and only the third may open a development protocol.

## Source facts and limits

The official [ObjectFolder-Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
states that each object has 30–50 six-second impact recordings, a strike
coordinate and a ground-truth contact-force profile. The
[ObjectFolder-Real paper](https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf)
describes a force-transducer-equipped impact hammer synchronized with a
free-field microphone. C3 additionally proved the exact object-41 raw pairing,
headers, coordinates and geometry before signal decode.

The source publishes one paired recording per object/contact key, not repeated
hits at the same contact. It does not publish SI channel calibration, numeric
listener pose or an exact per-object support revision. C4 can therefore claim
only normalized force-to-microphone transfer for one exact object in the
canonical recorded setup. It cannot claim absolute pascal/newton response,
free-field radiation, material-family transfer or perceptual quality.

## Primary-source modal-testing findings

The Siemens [FRF guide](https://community.sw.siemens.com/articles/en_US/Knowledge/what-is-a-frequency-response-function-frf)
requires the input-force spectrum to excite the frequencies of interest and
states that coherence from one measurement is identically one; at least two
repeats are needed for meaningful coherence. C4 therefore uses no ordinary
coherence gate.

The Siemens [impact-testing workflow](https://community.sw.siemens.com/articles/en_US/Knowledge/simcenter-testlab-impact-testing)
checks force bandwidth before accepting an FRF. It also warns that an
exponential response window adds artificial damping and prefers a longer
record when the response has not decayed. C4 keeps a fixed four-second crop,
applies no exponential window and rejects an unsettled tail rather than
manufacturing damping.

The Siemens [stabilization-diagram guide](https://community.sw.siemens.com/articles/en_US/Knowledge/Modal-Stabilization-Diagram-Tips)
describes stable frequency/damping solutions across model orders and notes
that multiple exciter locations can reveal modes missed by one excitation.
Peeters et al.,
[Consistent multi-input modal parameter estimators in the frequency domain](https://doi.org/10.1016/j.ymssp.2012.05.008),
describe global estimators that combine multiple FRFs under a common
denominator, while warning that a global estimate can hide inconsistent
measurements. C4 therefore combines contact FRFs for common poles but requires
four leave-quarter-out refits and keeps per-contact residuals visible.

Pintelon et al.,
[Frequency Response Function Measurements via Local Rational Modeling,
Revisited](https://doi.org/10.1109/TIM.2020.3020601), show that finite-record
transients/leakage can bias non-parametric FRFs near rapidly varying
resonances. C4 keeps local-rational/vector fitting as a separately frozen
successor comparison if the current candidate fails on a force-certified
band; it is not added after observing object-41 values.

## Consequences for the frozen experiment

- Estimate input-noise power only from the pre-impact force baseline and build
  a force-only ensemble certificate before microphone analysis.
- Do not use response peaks, coordinates, object labels or candidate modes to
  choose certified bands.
- Use a noise-regularized one-shot H1 transfer per contact, then a shared-pole
  fit with contact-local residues.
- Treat frequency/damping stability under contact-group deletion as the real
  repeatability witness; ordinary coherence is report-disabled.
- Require the measured-force candidate to beat a fixed force-permutation
  control. If the published force shapes are not discriminative enough for
  that test, return OOD instead of granting causal transfer credit.
- Keep raw/direct H1, an input-ignorant output-modal fit and an impulse
  assumption as honest controls. In-sample reconstruction alone cannot pass.
- Decode only the sixteen `estimator_fit` pairs. Development, representation
  holdout, both validator roles and admission shadow remain sealed.

## Decision

Preregister C4a as a fit-only experiment. A pass establishes only that one
bounded real common-pole representation is numerically stable enough to test
on five fresh contacts. It does not create a Physical Sound Record, train ML,
release the independent validator, bake an atlas or change the runtime.
