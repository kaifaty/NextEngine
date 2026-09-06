# Physical sound R3A V11 B1R3 — local modal-support oracle protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / FRESH_SYNTHETIC_REVISION / IMPLEMENTATION_NOT_RUN` |
| Parent evidence | [B1R2 result](physical-sound-r3a-v11-b1r2-noise-aware-gtls-result-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Question and bounded claim

Can a noise-aware GTLS/common-pole estimator recover every **locally
observable** mode, predict unseen force responses and reject unsupported or
incoherent modal neighborhoods without requiring almost complete broad-band
force coverage?

B1R3 changes only the evidence policy around modal support and the accounting
of response interference. It retains explicit shared poles/damping, per-contact
residues, noise-corrected GTLS, corrected H1/H2 controls and an explicit modal
renderer. It adds no residual model and no ML component.

`PASS_KNOWN_TRUTH_FRF` establishes synthetic estimator support only and opens a
separate B2 zero-decode source-feasibility protocol. Any valid failure keeps all
real data closed. No material, perceptual, validator, atlas, public-schema or
runtime claim is possible.

## Execution and leakage boundary

The protocol is committed before the runner. The later runner must expose
`freeze`, `preflight` and `run`:

1. `freeze` hash-closes this protocol, runner, B1 V1 helper, B1R2 helper and
   common-pole core without generating samples;
2. `preflight` validates lineage and emits zero generated samples;
3. `run` generates calibration/fit, evaluates development and corruption
   controls, and generates holdout only after every development gate passes.

Two independent runs must emit byte-identical manifest, model, arrays and
report. Network, real payload, parent holdout and optional model artifacts are
forbidden. All generated arrays and waveforms remain external to Git.

## Fresh known-truth fixture

- sample rate `48,000 Hz`, response length `96,000`, FFT length `262,144`;
- six contacts and seven shared modes;
- frequencies `613, 1097, 1777, 2819, 4433, 6643,
  8000.06103515625 Hz`;
- amplitude decays `5.25, 8.25, 11.75, 16.5, 24.0, 34.5, 49.0 s^-1`;
- residue magnitude
  `(0.29 + 0.041*((11*contact + 7*mode + 5) mod 17))/(1 + 0.025*mode)`;
- residue phase
  `0.173 + 0.23*contact + 0.19*mode + 0.037*contact*mode` radians;
- one global peak scale `0.25`, never per-contact normalization.

Truth enters generation and final scoring only. Estimation, masks, discovery,
order selection and residue fitting cannot read it.

Fit repeats four force profiles four times at every contact:
`half_sine(11)`, `hann(23)`, `beta(41,3,4)`, `half_sine(67)`. Development and
holdout each use one trial per contact/profile:

- development: `hann(15)`, `beta(49,4,3)`;
- holdout: `half_sine(29)`, `beta(59,3,5)`.

Fresh `PCG64` seed roots are:

| Role | Force sensor | Response sensor | Room/disturbance |
| --- | ---: | ---: | ---: |
| fit | `2026113101` | `2026113102` | `2026113103` |
| development | `2026113201` | `2026113202` | `2026113203` |
| holdout | `2026113301` | `2026113302` | `2026113303` |
| corruptions | `2026113401` | `2026113402` | `2026113403` |

`SeedSequence` children are spawned in contact/profile/repeat order. Ambient RNG
is forbidden.

## Calibration noise versus unmeasured disturbance

Every ordinary trial retains these distinct signals:

```text
f_true
n_force_sensor
y_clean = f_true * h_contact
n_response_sensor
d_room
f_observed = f_true + n_force_sensor
y_observed = y_clean + n_response_sensor + d_room
```

The two sensor channels retain independent zero-mean Gaussian noise at the B1R2
levels: force standard deviation `2e-4 * force peak` and response standard
deviation `2e-4 * clean-response RMS`. They are the only noise components
available to covariance correction. In a real import they correspond to a
separately measured stationary/pre-impact noise floor.

The bounded coloured room tail remains `0.002 * clean-response RMS`, delayed
`1,920` samples with the parent recurrence and exponential decay, but is
recorded as unmeasured disturbance and is **not** subtracted. The low-coherence
control adds independent coloured interference at `0.75 * clean-response RMS`
to the same unmeasured field. No estimator input may relabel either disturbance
as calibrated response noise.

## Noise correction and preliminary discovery field

For each contact/bin over its sixteen fit trials, B1R3 retains the B1R2
covariance correction but subtracts only sensor calibration:

```text
Gxx = max(sum(|F_observed|^2) - sum(|N_force_sensor|^2), 0)
Gyy = max(sum(|Y_observed|^2) - sum(|N_response_sensor|^2), 0)
Gyx = sum(Y_observed*conj(F_observed))
      - sum(N_response_sensor*conj(N_force_sensor))
input_snr = Gxx / max(Nxx, 1e-30)
```

The Hermitian corrected covariance and dominant-direction GTLS estimate are
unchanged. Corrected H1, corrected H2, raw H1, direct division,
shortest-response impulse and input-ignorant raw-output modal fits remain
controls.

Broad-band coverage is diagnostic only. A bin may enter preliminary pole
discovery when it is inside `200…10,000 Hz`, has `input_snr >= 3` and corrected
force power at least `1e-8` of that contact's maximum corrected power below
`12,000 Hz`. These are discovery floors, not admission floors. They prevent
noise-only bins and zero corrected power from driving the common-pole core while
allowing a candidate neighborhood to be assessed by the stricter local policy.
A four-bin cosine taper applies only at preliminary-mask edges.

## Local modal support and retained model

The unchanged input-driven Gabor/common-pole method runs on all six preliminary
GTLS fields: Blackman-Harris `1024/32`, first `256` frames, `200…10,000 Hz`,
`-35 dB` region floor, `±1` neighboring bin, `24` pencil lags, maximum order
`6`, score margin `20`, duplicate radius `1 Hz` and post-fit energy floor
`-30 dB`.

For every discovered candidate, the local neighborhood is
`±max(20 Hz, 4*decay/(2*pi))`. A contact supports the pole only when:

- median local input SNR is at least `10` (linear, `10 dB`);
- at least half of the local bins pass the preliminary discovery mask;
- median local corrected coherence is at least `0.90`.

A contact residue is confident at the same local input-SNR/mask rules and
corrected coherence `>= 0.85`. A candidate is retained only with at least three
supporting contacts and three confident residues. Residue uncertainty remains
`1 - median local corrected coherence` and is never used to invent a value.

The selected held-response transfer is the explicit reconstruction from only
retained modes. It is not multiplied by a broad-band coverage mask. Raw GTLS
and corrected H1/H2 fields remain diagnostics. A locally unsupported candidate
is reported with its frequency interval and excluded from the model; it is not
interpolated or repaired.

## Positive, partial-support and OOD controls

Development evaluates:

1. the ordinary fresh fixture, which must retain all seven truth modes;
2. a measured positive comb-notch force made from five nonnegative copies of a
   three-sample half-sine at delays `0,3,6,9,12` with weights `1,4,6,4,1`,
   normalized once. Its fourth-order spectral zero targets the exact FFT-bin truth mode at
   `8000.06103515625 Hz`;
3. weak `hann(1025)` excitation;
4. low-coherence interference outside calibration.

The comb-notch control must retain the six supported truth modes, identify the
single unsupported truth neighborhood and return
`FallbackOutOfDomain / OOD_UNSUPPORTED_MODAL_BAND` for the complete seven-mode
domain. It may expose the six supported estimates diagnostically, but admits no
complete-domain model. The weak control must likewise return local unsupported
bands rather than optimistic modes.

Low coherence is detected from the frozen local coherence evidence or a median
per-trial observed-response reconstruction NRMSE `>= 0.20`. The extra
interference is not part of `Nyy`. The required decision is
`OOD_LOW_COHERENCE` with zero complete-domain admitted modes.

Holdout repeats the ordinary and comb-notch checks with fresh seeds and adds the
existing missing-second-impact control: an unrecorded `half_sine(23)` at gain
`0.8` and delays `389/617/941/1207`. Median observed-response reconstruction
NRMSE `>= 0.15` must yield `OOD_MODEL_MISMATCH`.

## Frozen gates

Development and holdout independently require:

| Endpoint | Gate |
| --- | ---: |
| Supported truth modes / unsupported truth modes | `7 / 0` |
| False-positive retained modes | `0` |
| Minimum supporting contacts / confident residues per retained mode | `>= 3 / >= 3` |
| Maximum modal-frequency error | `<= 1.0 Hz` |
| Maximum absolute / relative damping error | `<= 1.5 s^-1 / 0.20` |
| Mean / maximum held modal NRMSE | `<= 0.08 / 0.15` |
| Mean gain-matched log-spectrum RMSE | `<= 2.0 dB` |
| Modal / impulse NRMSE | `<= 0.70` |
| Modal / raw-output-modal NRMSE | `<= 0.80` |
| Modal / raw-H1 NRMSE | `<= 1.50` |
| GTLS diagnostic / best corrected H1/H2 NRMSE | `<= 1.05` |

Noiseless identity remains `<= 1e-11`. Corrected covariance, poles, residues,
support, uncertainty and metrics must be finite. The selected record contains
exactly seven modes and no waveform residual. The comb-notch control requires
exactly `6` supported and `1` unsupported truth mode, zero false positives and
the declared OOD decision. Broad `200…10,000 Hz` coverage is published only as
a diagnostic and has no pass threshold.

Hard gates require exact component/sample accounting, calibration/disturbance
separation, no holdout after development failure, canonical serialization,
dependency hashes, two-run byte identity and zero real/network/parent-holdout
access.

## Stop rule

- `PASS_KNOWN_TRUTH_FRF`: every development and holdout gate passes twice; only
  B2 zero-decode source feasibility opens.
- `REJECT_LOCAL_MODAL_SUPPORT`: any valid numeric, local-support, residue,
  partial-support or OOD gate fails; do not change this fixture or thresholds
  and do not read real data.
- `INVALID_B1R3_RUN`: lineage, calibration accounting, ordering, serialization,
  finiteness or repeat fails and grants no estimator evidence.
