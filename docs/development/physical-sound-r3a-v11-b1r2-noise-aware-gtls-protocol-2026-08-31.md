# Physical sound R3A V11 B1R2 — noise-aware GTLS modal FRF protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / FRESH_SYNTHETIC_REVISION / IMPLEMENTATION_NOT_RUN` |
| Parent evidence | [B1 result](physical-sound-r3a-v11-b1-force-response-oracle-result-2026-08-31.md) and [B1R precheck/research](physical-sound-r3a-v11-b1r-precheck-rejection-and-frf-noise-research-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Question and claim boundary

Can an independently noise-calibrated errors-in-variables transfer estimator
recover one explicit common-modal system from noisy force and response records,
predict unseen forces and distinguish a truly unexcited force band from sensor
noise?

The selected candidate is a noise-corrected generalized total-least-squares
(`GTLS`) covariance direction followed by the existing input-driven
Gabor/common-pole estimator. H1, H2 and direct division are controls. The model
contains only shared poles/damping and contact residues; it has no waveform
residual and no ML component.

`PASS_KNOWN_TRUTH_FRF` establishes synthetic support only and opens a separate
B2 zero-decode source-feasibility protocol. Any valid failure keeps real data
closed. No material, perceptual, validator, atlas, public-schema or runtime
claim is possible.

## Execution and leakage boundary

The B1R2 runner and this protocol must be committed before numeric evidence.
The runner exposes `freeze`, `preflight` and `run`:

1. `freeze` hash-closes runner, protocol, B1 V1 helper, common-pole core and
   exact constants without generating a fixture;
2. `preflight` validates lineage and reports zero generated samples;
3. `run` generates noise calibration and fit, evaluates development, and only
   after every development gate passes generates holdout.

Two independent runs must emit byte-identical manifest, model, arrays and
report. No network, real payload, B1/B1R holdout or optional model artifact may
be read. Noise arrays, transfer arrays and generated waveforms remain external.

## Fresh known-truth fixture

- `48,000 Hz`, `96,000` samples, `262,144`-sample FFT;
- six contacts and seven shared modes;
- frequencies `557, 983, 1597, 2539, 4051, 6451, 9769 Hz`;
- amplitude decays `4.5, 7.5, 10.5, 15, 22, 32, 47 s^-1`;
- residue magnitude
  `(0.31 + 0.045*((7*contact + 9*mode + 2) mod 13))/(1 + 0.03*mode)`;
- residue phase
  `0.113 + 0.27*contact + 0.21*mode + 0.031*contact*mode` radians;
- one global peak scale `0.25`, never per-contact normalization.

Truth enters only generation and final scoring. Estimators, masks, mode
discovery, order selection and residue fit cannot read it.

## Trials and independent noise calibration

Fit repeats each B1 force profile four times at every contact:
`half_sine(9)`, `hann(19)`, `beta(37,2,4)`, `half_sine(61)`. This produces
`6 * 4 * 4 = 96` fit trials. Development and holdout retain two unseen force
profiles and one trial per contact/profile:

- development: `hann(13)`, `beta(47,3,2)`;
- holdout: `half_sine(25)`, `beta(53,2,5)`.

Every fit trial retains five separate signals:

```text
f_true
n_force
y_clean = f_true * h_contact
n_response = gaussian response noise + bounded room tail
(f_observed, y_observed) = (f_true+n_force, y_clean+n_response)
```

Noise laws stay comparable to B1: force Gaussian standard deviation
`2e-4 * force peak`; response Gaussian `2e-4 * clean-response RMS`; room tail
`0.002 * clean-response RMS`, delayed `1,920` samples with the frozen coloured
recurrence and exponential decay. The noise-only calibration is not an extra
draw: it is the exact separately retained `n_force/n_response` pair associated
with each observed fit trial.

Fresh `PCG64` seed roots are:

| Role | Force noise | Response noise | Room tail |
| --- | ---: | ---: | ---: |
| fit | `2026112201` | `2026112202` | `2026112203` |
| development | `2026112301` | `2026112302` | `2026112303` |
| holdout | `2026112401` | `2026112402` | `2026112403` |
| corruptions | `2026112501` | `2026112502` | `2026112503` |

`SeedSequence` children are spawned in contact/profile/repeat order. Ambient RNG
is forbidden.

## Noise-corrected observability and estimators

For each contact/bin over its sixteen fit trials:

```text
Gxx_obs = sum(abs(F_observed)^2)
Gyy_obs = sum(abs(Y_observed)^2)
Gyx_obs = sum(Y_observed * conj(F_observed))

Nxx = sum(abs(N_force)^2)
Nyy = sum(abs(N_response)^2)
Nyx = sum(N_response * conj(N_force))

Gxx = max(Gxx_obs - Nxx, 0)
Gyy = max(Gyy_obs - Nyy, 0)
Gyx = Gyx_obs - Nyx
```

Source observability is independent of response:

```text
input_snr = Gxx / max(Nxx, 1e-30)
relative_signal_power = Gxx / max(Gxx over 0…12,000 Hz)
force_valid = 200…12,000 Hz
              AND input_snr >= 100
              AND relative_signal_power >= 1e-5
```

The selected GTLS estimator forms the Hermitian corrected covariance
`[[Gxx, conj(Gyx)], [Gyx, Gyy]]`, clips its two eigenvalues at zero for the
reported PSD diagnostic, and takes the dominant eigenvector direction
`H_gtls = v_y/v_x`. A bin with `abs(v_x) <= 1e-12` or failed `force_valid` is
invalid. A four-bin cosine taper is applied only at force-valid edges.

Controls:

- raw H1: `Gyx_obs/Gxx_obs`;
- corrected H1: `Gyx/max(Gxx,1e-30)`;
- corrected H2: `Gyy/conj(Gyx)`;
- equal-weight per-trial direct division;
- shortest-response impulse assumption;
- input-ignorant raw-output modal fitting.

All divisions publish invalid masks. No zero/negative corrected power is
repaired by borrowing truth or interpolation.

Noise-corrected coherence is
`abs(Gyx)^2/max(Gxx*Gyy,1e-30)`, clipped to `[0,1]` only after the ratio is
reported finite. It does not mask the GTLS waveform. It supplies shared-pole
and contact-residue uncertainty.

## Explicit modal transfer

Inverse-transform the six force-valid GTLS fields and run the unchanged
input-driven Gabor/common-pole method: Blackman-Harris `1024/32`, first `256`
frames, `200…12,000 Hz`, `-35 dB` region floor, `±1` neighboring bin, `24`
pencil lags, maximum order `6`, score margin `20`, duplicate radius `1 Hz`,
post-fit energy floor `-30 dB`.

For every discovered mode, the support neighborhood is
`±max(20 Hz, 4*decay/(2*pi))`. A contact supports the shared pole at median
corrected coherence `>= 0.90` and median input SNR `>= 100`. Every retained
mode needs at least three supporting contacts. A contact residue is confident
at corrected coherence `>= 0.85`; every mode needs at least three confident
residues. Uncertainty is `1 - median corrected coherence`.

The selected transfer used for held response is the explicit modal
reconstruction, force-band tapered per contact. The nonmodal GTLS field is
diagnostic only.

## Frozen gates

Development and holdout independently require:

| Endpoint | Gate |
| --- | ---: |
| Minimum force-valid coverage, `200…10,000 Hz` | `>= 0.98` per contact |
| Truth modes / false positives | `7 / 0` |
| Minimum supporting contacts / confident residues per mode | `>= 3 / >= 3` |
| Maximum modal-frequency error | `<= 1.0 Hz` |
| Maximum absolute / relative damping error | `<= 1.5 s^-1 / 0.20` |
| Mean / maximum held modal NRMSE | `<= 0.08 / 0.15` |
| Mean gain-matched log-spectrum RMSE | `<= 2.0 dB` |
| Modal / impulse NRMSE | `<= 0.70` |
| Modal / raw-output-modal NRMSE | `<= 0.80` |
| Modal / raw-H1 NRMSE | `<= 1.50` |
| GTLS diagnostic / best corrected H1/H2 NRMSE | `<= 1.05` |

Noiseless identity is `<= 1e-11`. Corrected covariance, eigenvectors, residues,
uncertainties and every metric must be finite; the selected record must contain
exactly seven modes and no residual.

OOD gates use fresh corruption seeds:

- weak `hann(1025)`: force-valid coverage above `4 kHz <= 0.35` gives
  `OOD_WEAK_EXCITATION`, zero admitted modes;
- independent response interference `0.75 * clean RMS`: median corrected
  coherence `<= 0.80` or coherent coverage `<= 0.50` gives
  `OOD_LOW_COHERENCE`, zero modes;
- unrecorded second `half_sine(19)` at gain `0.8` and delays
  `401/613/887/1151`: median fitted-transfer reconstruction NRMSE `>= 0.15`
  gives `OOD_MODEL_MISMATCH`, zero modes.

Hard gates require exact counts, noise/component accounting, no holdout after
development failure, canonical serialization, dependency hashes, two-run byte
identity and zero real/network/parent-holdout access.

## Stop rule

- `PASS_KNOWN_TRUTH_FRF`: all development and holdout gates pass twice; only B2
  zero-decode source feasibility opens.
- `REJECT_NOISE_AWARE_FRF`: any valid numeric, support, uncertainty or OOD gate
  fails; no threshold/fixture repair or real-data read is allowed.
- `INVALID_B1R2_RUN`: lineage, noise accounting, ordering, serialization,
  finiteness or repeat fails and grants no estimator evidence.
