# Physical sound R3A V12 C1 — acquisition-coverage oracle protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / FRESH_SYNTHETIC_REVISION / IMPLEMENTATION_NOT_RUN` |
| Parent evidence | [B1R3 result](physical-sound-r3a-v11-b1r3-local-modal-support-result-2026-08-31.md) |
| Research basis | [C1/C2 coverage research](physical-sound-r3a-v12-c1-coverage-and-source-research-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Question and bounded claim

Can a force-only acquisition certificate distinguish identifiable modal bands
before response fitting, after which the unchanged noise-aware GTLS/common-pole
path recovers every certified mode, predicts unseen force responses and rejects
unsupported acquisition or unmeasured response corruption?

C1 adds no ML, residual decoder, real data, material claim or runtime path. A
repeat-exact `PASS_KNOWN_TRUTH_FRF` opens only C3 role freezing if C2 separately
finds a source. Any valid failure closes this common-pole revision without
threshold repair.

## Execution and leakage boundary

The protocol is committed before the runner. The runner must expose `freeze`,
`preflight` and `run` stages:

1. `freeze` hash-closes this protocol, the runner, B1/B1R2/B1R3 helpers and the
   common-pole core without generating samples;
2. `preflight` validates exact lineage and emits zero samples;
3. `run` creates measured force plus force-sensor calibration, freezes the
   acquisition certificate, and only then creates transfer truth and response;
4. development and all development controls execute before any holdout sample;
5. holdout is generated only after every development check passes.

Two independent invocations must emit byte-identical manifest, certificate,
model, arrays and report. Network, real payload, prior holdout and optional
model artifacts are forbidden. Generated arrays remain outside Git.

## Force-only acquisition certificate

The force substrate uses `48,000 Hz`, `96,000` samples and FFT length
`262,144`. Five nonnegative profiles, six synthetic sensor instances and four
repeats per profile create `120` force-calibration trials:

```text
half_sine(3)
half_sine(5)
hann(7)
beta(9,2,3)
half_sine(13)
```

Every profile has unit discrete impulse. Force sensor noise is independent
zero-mean Gaussian noise with standard deviation `2e-4 * profile peak`, using
`PCG64` root `2026120101` and `SeedSequence` children in
contact/profile/repeat order.

For profile `p` and frequency bin `k`, summed over its 24 observations:

```text
Gxx[p,k] = max(sum(|F_observed|^2) - sum(|N_force_sensor|^2), 0)
Nxx[p,k] = sum(|N_force_sensor|^2)
input_snr[p,k] = Gxx[p,k] / max(Nxx[p,k], 1e-30)
relative_power[p,k] = Gxx[p,k] / max(max_{f<=12kHz} Gxx[p,f], 1e-30)
```

A profile supports a bin only inside `200…9,500 Hz`, with
`input_snr >= 25` and `relative_power >= 0.005`. A bin is acquisition-certified
when at least three profiles support it. No response, coherence, transfer truth
or modal target may enter this certificate.

For a modal frequency/decay, the local neighborhood is
`±max(24 Hz, 4*decay/(2*pi))`. An ordinary truth mode is eligible only if at
least `90%` of its neighborhood is acquisition-certified. The same `90%`
criterion must hold after omitting each one of the five profiles and requiring
three supporters among the remaining four.

The preregistration force-only discriminator certifies the whole target band;
the frozen truth-neighborhood profile counts are `5,5,5,5,4,4,4`. The runner
must reproduce that fact from fresh generated force observations before
creating `h(t)`.

## Fresh known-truth object

- six contacts and seven shared modes;
- frequencies `719, 1493, 2579, 4013, 6000, 7877, 9251 Hz`;
- amplitude decays `4.75, 7.5, 10.5, 15.0, 21.5, 30.0, 41.0 s^-1`;
- residue magnitude
  `(0.33 + 0.039*((13*contact + 5*mode + 7) mod 19))/(1 + 0.018*mode)`;
- residue phase
  `0.211 + 0.17*contact + 0.137*mode + 0.029*contact*mode` radians;
- one global peak scale `0.25`, never per-contact normalization.

Truth is created only after the certificate passes its hard self-check. It
enters response generation and final scoring, never input coverage, discovery,
order selection, pole fit or residue fit.

The five certificate profiles and force observations become fit inputs. Their
responses use four repeats at all six contacts. Development and holdout each
use one trial per contact/profile:

- development: `half_sine(11)`, `beta(15,3,4)`;
- holdout: `hann(9)`, `beta(17,4,3)`.

Fresh seed roots are:

| Role | Force sensor | Response sensor | Room/disturbance |
| --- | ---: | ---: | ---: |
| certificate + fit | `2026120101` | `2026120102` | `2026120103` |
| development | `2026120201` | `2026120202` | `2026120203` |
| holdout | `2026120301` | `2026120302` | `2026120303` |
| development controls | `2026120401` | `2026120402` | `2026120403` |
| holdout controls | `2026120501` | `2026120502` | `2026120503` |

Response sensor noise remains independent Gaussian at
`2e-4 * clean-response RMS`. The delayed coloured room tail remains
`0.002 * clean-response RMS` and is unmeasured disturbance. Only sensor noise
is available to covariance correction.

## Transfer and modal estimator

C1 retains B1R3 noise-corrected dominant-covariance GTLS and corrected H1/H2,
raw H1, direct division, shortest-response impulse and input-ignorant
raw-output-modal controls. The Hermitian covariance and sensor-only correction
are unchanged.

Preliminary pole discovery is allowed only in acquisition-certified bins.
Broad coverage is reported but has no additional threshold because the
certificate is the coverage authority. The common-pole path remains:

- Blackman-Harris `1024/32`, first `256` frames;
- `200…9,500 Hz`, `-35 dB` region floor and `±1` neighboring bin;
- `24` pencil lags, maximum local order `6`, score margin `20`;
- duplicate radius `1 Hz`, post-fit energy floor `-30 dB`.

A discovered candidate is retained only when its neighborhood is at least
`90%` acquisition-certified, at least three contacts have median corrected
coherence `>=0.90`, and at least three residues have coherence `>=0.85`.
Residue uncertainty is `1 - median local corrected coherence`. The selected
transfer is reconstructed only from retained modes and contains no waveform
residual.

Five leave-one-profile-out fits recompute the force certificate and the complete
estimator after removing each profile. Every one must still certify and recover
all seven truth modes with zero false positives. These fits test ensemble
redundancy; they do not select a profile or checkpoint.

## Acquisition-hole and query-notch separation

The exact `6,000 Hz` mode is the preregistered discriminator.

Three acquisition-hole profiles are made by summing two equal nonnegative
copies, delayed by four samples, of `half_sine(3)`, `half_sine(5)` and
`hann(7)`. The factor `1 + z^-4` gives an exact zero at `6,000 Hz`; all profiles
are normalized once. Four repeats at every contact produce a separate
force-only certificate and fit.

The acquisition-hole fit must:

- certify and recover truth indices `0,1,2,3,5,6`;
- mark only index `4` (`6,000 Hz`) unsupported;
- retain zero false positives;
- return `OOD_ACQUISITION_HOLE` and admit no complete-domain model.

Separately, the full ordinary seven-mode model receives a held query using the
notched `half_sine(3)` pair. It must remain a valid query, predict the clean
response with mean NRMSE `<=0.08`, and show target-mode response RMS at most
`0.02` of the root-sum-square truth modal response. It must not erase the mode
from the object record or emit acquisition OOD.

## Remaining OOD controls

- Weak acquisition uses five `hann` profiles of lengths
  `769, 833, 897, 961, 1025` samples. Failure to certify all truth
  neighborhoods must return `OOD_WEAK_EXCITATION` with zero complete-domain
  admitted modes.
- Low coherence applies a held `half_sine(7)` query to the ordinary model and
  adds independent coloured response interference at `0.75 * clean-response
  RMS`, outside sensor calibration. Median observed-response residual NRMSE
  `>=0.20` must return `OOD_LOW_COHERENCE`.
- Holdout missing-impact uses measured `half_sine(13)` plus one unrecorded
  `half_sine(17)` at gain `0.8` and delays `373, 619, 907, 1231` samples.
  Median residual NRMSE `>=0.15` must return `OOD_MODEL_MISMATCH`.

Every OOD result admits zero complete-domain modes and keeps clip fallback.

## Frozen numeric gates

Development and holdout independently require:

| Endpoint | Gate |
| --- | ---: |
| Acquisition-certified truth neighborhoods | `7/7` |
| Leave-one-profile-out certified/recovered modes | `7/7` for all five omissions |
| Retained truth modes / false positives | `7 / 0` |
| Minimum supporting contacts / confident residues | `>=3 / >=3` |
| Maximum modal-frequency error | `<=1.0 Hz` |
| Maximum absolute / relative damping error | `<=1.5 s^-1 / 0.20` |
| Mean / maximum held modal NRMSE | `<=0.08 / 0.15` |
| Mean gain-matched log-spectrum RMSE | `<=2.0 dB` |
| Modal / impulse NRMSE | `<=0.70` |
| Modal / raw-output-modal NRMSE | `<=0.80` |
| Modal / raw-H1 NRMSE | `<=1.50` |
| GTLS / best corrected H1/H2 NRMSE | `<=1.05` |

Noiseless identity remains `<=1e-11`. All covariance, certificate, mode,
residue, uncertainty and metric values must be finite. The ordinary model has
exactly seven modes and no residual. Control decisions and counts must match
their exact declarations above.

Hard gates require exact component/sample accounting, certificate-before-truth
ordering, sensor/disturbance separation, no holdout after development failure,
canonical serialization, dependency hashes, two-run byte identity and zero
real/network/prior-holdout access.

## Stop rule

- `PASS_KNOWN_TRUTH_FRF`: all development, leave-one-out, controls and holdout
  gates pass twice; C3 may open only if C2 also freezes a viable source.
- `REJECT_COVERAGE_CERTIFIED_COMMON_POLE`: any valid supported-mode,
  reconstruction, redundancy or OOD gate fails. Do not modify this fixture or
  thresholds; close the current common-pole branch and preregister at most one
  local-rational/vector-fitting comparison.
- `DATA_INSUFFICIENT_FORCE_COVERAGE`: the force-only ordinary certificate does
  not reproduce its declared band before truth generation; emit no object
  response and reconsider the acquisition design only.
- `INVALID_C1_RUN`: lineage, ordering, accounting, serialization, finiteness or
  repeatability fails and grants no estimator evidence.
