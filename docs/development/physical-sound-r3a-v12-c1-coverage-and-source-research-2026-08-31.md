# Physical sound R3A V12 C1/C2 — excitation coverage and source research

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `BOUNDED_RESEARCH_COMPLETE / C1_PROTOCOL_JUSTIFIED / C2_ZERO_DECODE_ONLY` |
| Trigger | B1R3 repeat-exact loss of the `6,643 Hz` truth mode despite accurate retained modes |
| Product effect | None; authored clips remain authoritative |

## Falsifiable question

The B1R3 failure admits three competing explanations:

1. the common-pole estimator cannot recover seven lightly damped modes;
2. the local-support policy is still wrong even when the input identifies a
   mode;
3. the frozen impact bank does not excite one truth neighborhood, so the
   estimator correctly refuses to identify information that is not present.

Exact B1R3 evidence supports the third explanation. Six retained modes have
maximum frequency error `0.119071 Hz`, maximum relative damping error
`0.011412` and `0.007180` mean held-response NRMSE. The missing `6,643 Hz`
neighborhood has median input SNR only `1.207…1.341`, discovery coverage
`0.008…0.042` and zero supporting contacts. The `8,000.061 Hz` mode remains
accurate because its local input SNR is approximately `37.7…41.3`.

## Primary-source findings for C1

The official Siemens [modal-testing guide](https://community.sw.siemens.com/articles/en_US/Knowledge/Modal-Testing-A-Guide)
shows that hammer-tip stiffness and contact time control excitation bandwidth.
A metal tip keeps the input-force spectrum usable at higher frequencies; a
soft tip rolls off and produces a noisy FRF. The official
[impact-test workflow](https://community.sw.siemens.com/articles/en_US/Knowledge/simcenter-testlab-impact-testing)
therefore evaluates input bandwidth before accepting the measurement.

The Siemens [multiple-hammer guide](https://community.sw.siemens.com/articles/en_US/Knowledge/Multiple-Hammers-in-Simcenter-Testlab-Neo-Impact-Acquisition)
merges low- and high-frequency FRF evidence from exciters with different useful
bands. This supports a per-profile force certificate and an ensemble union,
not an assumption that every smooth pulse is broadband.

The Siemens [FRF guide](https://community.sw.siemens.com/articles/en_US/Knowledge/what-is-a-frequency-response-function-frf)
also states that a single measurement makes ordinary coherence equal to one by
construction and that low coherence at an antiresonance can be normal. Thus
coherence is useful only with repeats and cannot replace independent input-SNR,
residual and mismatch checks.

Pintelon et al.,
[Frequency Response Function Measurements via Local Rational Modeling, Revisited](https://doi.org/10.1109/TIM.2020.3020601),
show that finite measurement duration and leakage can bias non-parametric FRF
estimates, especially near rapidly varying resonances. Local rational modeling
is therefore the frozen successor comparison only if the final common-pole
oracle fails on a force-certified mode; it is not added speculatively to C1.

### Conclusion for C1

Acquisition support must be computed from the measured force ensemble before
object response or modal truth exists. Object-mode admission then requires a
locally force-certified neighborhood and repeated response evidence. A held
query force uses the already identified transfer model; a spectral zero in that
query means negligible contribution, not retroactive loss of object identity.

## Force-only prefreeze discriminator

Five nonnegative, unit-impulse profiles were evaluated without creating an
object transfer or response:

```text
half_sine(3), half_sine(5), hann(7), beta(9,2,3), half_sine(13)
```

For every profile, six synthetic sensor instances and four repeats use fresh
`PCG64` force-noise root `2026120101`. Per-profile corrected force power is
supported at a bin when input SNR is at least `25`, corrected power is at least
`0.005` of that profile's maximum below `12 kHz`, and the bin is inside
`200…9,500 Hz`. The ensemble certificate requires three profiles.

This force-only calculation certifies `100%` of the target band; every bin has
at least three supporting profiles. The seven frozen truth neighborhoods have
support counts `5,5,5,5,4,4,4`. After deleting any one profile, at least three
remaining profiles certify at least `90%` of every truth neighborhood. No
response, pole, residue or quality metric participated in this selection.

## Primary-source findings for C2

The [RealImpact paper](https://openaccess.thecvf.com/content/CVPR2023/html/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.html)
documents synchronized `48 kHz` hammer-force and microphone acquisition,
impact coordinates, listener positions and RGBD geometry. However, the
[official repository](https://github.com/samuel-clarke/RealImpact) still says
that the raw dataset is being packaged. Its current public per-object archives
contain geometry, coordinates and force-deconvolved responses, but no raw
hammer or microphone arrays. RealImpact is therefore eligible for relative
transfer/radiation controls, not C2 paired raw-force fitting.

The official [ObjectFolder-Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
publishes stable mesh and acoustic archives for 100 objects and states that
each of 30–50 impact points includes a six-second sound, mesh strike coordinate
and ground-truth contact-force profile. It is the strongest current C2
candidate for paired force/audio/contact/geometry. Exact listener calibration,
per-object support revision and source-disjoint validator roles remain
unproven; the first claim must therefore be one exact object at the recorded
canonical listener, with missing axes explicit.

Other searched impact/audio sources either provide force/structural vibration
without object sound fields or sound/video without synchronized contact force.
They may become validator negatives, but they do not improve the first paired
transfer claim.

## Decision

- Preregister one fresh C1 oracle with a force-only ensemble certificate,
  leave-one-profile-out recovery and separate acquisition-hole/query-notch
  controls.
- Do not modify B1R3 or weaken its gates.
- Keep real PCM closed until C1 passes.
- C2 may freeze ObjectFolder-Real availability and missing-axis facts without
  reading waveform samples. RealImpact remains a transfer-only secondary
  source until its raw release surface changes.
- If C1 loses a certificate-supported mode, close the current Gabor/common-pole
  branch and compare it once against a preregistered local-rational/vector-fit
  control before any real decode.
