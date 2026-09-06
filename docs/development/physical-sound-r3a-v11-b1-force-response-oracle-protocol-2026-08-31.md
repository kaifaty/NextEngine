# Physical sound R3A V11 B1 — force/response oracle protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / SYNTHETIC_ONLY / IMPLEMENTATION_NOT_RUN` |
| Roadmap | [V11 B1](../plans/physical-sound-synthesis-roadmap-v11.md#b1--synthetic-forceresponse-oracle) |
| Predecessor | [A1R result and V11 research](physical-sound-r3a-v10-a1r-force-onset-fit-result-and-v11-research-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Question and permitted claim

Can one frozen force-normalized estimator recover a known shared modal transfer
from multiple noisy force/response trials, predict responses to unseen force
profiles and fail closed when force conditioning or the single-input model is
invalid?

`PASS_KNOWN_TRUTH_FRF` establishes only synthetic estimator support and opens a
separate zero-decode internet-source freeze. It does not establish real-object
quality, material identity, contact interpolation, a validator release, an
atlas, a public schema or runtime use. `REJECT_ESTIMATOR` keeps every fresh real
waveform closed.

## Freeze and role boundary

The committed runner and this protocol must exist before the numeric evidence
run. The runner has three stages:

1. `freeze` writes one external canonical manifest containing the exact runner,
   protocol and common-pole dependency hashes; it generates no fixture sample;
2. `preflight` validates the manifest and records zero generated or evaluated
   fit/development/holdout samples;
3. `run` generates `fit`, evaluates `development`, and only if every development
   gate passes generates `holdout`. Candidate choice and thresholds are code and
   manifest constants; no result can change them inside the run.

Two independent complete `run` outputs must be byte-identical. Arrays, models
and reports remain under the external physical-sound experiment store and do
not enter Git.

| Role | Contents | Read rule |
| --- | --- | --- |
| `fit` | six contacts × four clean force profiles | estimator input only |
| `development` | six contacts × two unseen clean profiles; weak-excitation and low-coherence controls | opens after fit construction; selects only the already frozen candidate |
| `holdout` | six contacts × two further unseen clean profiles; weak-excitation and missing-second-impact controls | generated once only after all development gates pass |

All pseudorandom streams use NumPy `PCG64` with independent role/purpose seed
roots: fit force noise `2026110101`, fit response noise `2026110102`, fit room
tail `2026110103`; development `2026110201…203`; holdout
`2026110301…303`; corruptions `2026110401…403`. A `SeedSequence` child is
spawned in canonical contact/profile/repeat order. No ambient RNG is allowed.

## Exact known-truth modal fixture

- sample rate: `48,000 Hz`;
- transfer length: `96,000` samples;
- analysis FFT: `262,144` samples, real one-sided form;
- six contact outputs with shared poles and contact-specific complex residues;
- modal frequencies: `523, 911, 1487, 2381, 3769, 6029, 9137 Hz`;
- amplitude-decay rates: `4, 6.5, 9, 13, 19, 28, 42 s^-1`;
- contact/mode magnitude:
  `0.38 + 0.035*((5*contact + 7*mode) mod 13)`, divided by
  `1 + 0.04*mode`;
- contact/mode phase:
  `0.19*contact + 0.23*mode + 0.017*contact*mode` radians;
- causal transfer sample:
  `sum(gain * exp(-decay*t) * cos(2*pi*frequency*t + phase))`;
- one global scale makes the maximum absolute sample over all six exact
  transfers `0.25`; no per-contact normalization is allowed.

Truth frequencies, damping and residues are available only to fixture
generation and final scoring. Conditioning, transfer estimation, mode
discovery, order selection, residue fit and OOD decisions cannot read them.

## Exact forces and observation corruptions

Every clean force is non-negative, begins at sample `0`, and is normalized to
unit discrete sum. `half_sine(n)` is
`sin(pi*(i+0.5)/n)`; `hann(n)` is the interior of `hann(n+2)`; `beta(n,a,b)`
is `(x**a)*((1-x)**b)` at `x=(i+0.5)/n`.

| Role | Force profiles |
| --- | --- |
| fit | `half_sine(9)`, `hann(19)`, `beta(37,2,4)`, `half_sine(61)` |
| development | `hann(13)`, `beta(47,3,2)` |
| holdout | `half_sine(25)`, `beta(53,2,5)` |

For a clean trial, `y_clean = fftconvolve(f_true, h_contact)[:96000]`.
Observed force is `f_true` plus independent zero-mean Gaussian noise with
standard deviation `2e-4 * peak(f_true)`. Observed response adds independent
Gaussian noise with standard deviation `2e-4 * rms(y_clean)` and an independent
room tail. The room tail starts at sample `1,920`, is white noise passed through
the exact recurrence `z[n] = 0.82*z[n-1] + e[n]`, is multiplied by
`exp(-18*t)`, and is globally scaled to `0.002 * rms(y_clean)`. Division by a
zero measured RMS is invalid.

Three controls are separate from clean roles:

- `weak_excitation`: four `hann(1025)` repeats; response remains generated from
  the measured true force, so only spectral conditioning should fail;
- `low_coherence`: four `half_sine(19)` repeats whose response additionally
  receives independent coloured interference at `0.75 * rms(y_clean)`;
- `missing_second_impact`: four repeats whose recorded force contains one
  `half_sine(19)` but whose response contains an unrecorded second copy with
  gain `0.8` and delays `401, 613, 887, 1151` samples.

Development sees the first two controls. Holdout sees independently seeded
weak excitation and the missing-second-impact control. Corruptions never enter
the transfer fit.

## Frozen estimators

For every contact and positive-frequency bin, over the four fit profiles:

```text
Sxx = sum(X * conj(X))
Syx = sum(Y * conj(X))
Syy = sum(Y * conj(Y))
coherence = abs(Syx)^2 / (Sxx * Syy)
H1 = Syx / (Sxx + 1e-8 * max_band(Sxx))
```

The analyzed band is `200…12,000 Hz`. A bin is valid only when normalized
`Sxx >= 1e-5` and coherence `>= 0.98`. The candidate may predict only through
its regularized `H1`; invalid bins are zeroed with a four-bin cosine transition
at each contiguous mask edge. Valid coverage and every mask interval are
published.

The diagnostic direct-division control averages `Y/X` with equal trial weight
only where that trial has normalized input power `>= 1e-5`; a bin without an
eligible trial is invalid. The impulse-assumption control treats the shortest
fit response, divided by the exact unit force sum, as the transfer. The
raw-output modal control discovers poles and fits residues from the mean clean
fit response while ignoring every force sample. None may borrow candidate
conditioning or truth labels.

Mode discovery uses the existing input-driven Gabor/common-pole method over the
estimated six-contact transfer: Blackman-Harris `1024`-sample windows, hop `32`,
first `256` frames, local-energy discovery over `200…12,000 Hz` at `-35 dB`,
one neighbor bin on each side, `24` pencil lags, maximum order `6`, minimum
order-score margin `20`, single-link duplicate radius `1 Hz`, and post-fit mode
energy floor `-30 dB`. Estimated residues are ordinary all-contact least squares
on the discovered damped cosine/sine basis. Only the final scorer performs a
one-to-one nearest matching within `12 Hz` to known truth.

The noiseless identity uses an exact one-sample unit impulse to recover one
contact transfer with zero regularization, then predicts `beta(43,2,3)`. It is
independent of the noisy candidate selection.

## Frozen metrics and gates

Development and holdout independently require:

| Endpoint | Gate |
| --- | ---: |
| Candidate valid coverage in `200…10,000 Hz` | `>= 0.90` per contact |
| Truth modes matched / retained false positives | `7 / 0` |
| Maximum modal-frequency error | `<= 1.0 Hz` |
| Maximum absolute damping error | `<= 1.5 s^-1` |
| Maximum relative damping error | `<= 0.20` |
| Mean held-response NRMSE | `<= 0.06` |
| Maximum held-response NRMSE | `<= 0.10` |
| Mean gain-matched log-spectrum RMSE, `200…12,000 Hz` | `<= 1.5 dB` |
| Candidate mean NRMSE / impulse-assumption mean NRMSE | `<= 0.65` |
| Candidate mean NRMSE / raw-output-modal mean NRMSE | `<= 0.75` |
| Candidate mean NRMSE / direct-division mean NRMSE | `<= 1.00` |

The noiseless identity NRMSE must be `<= 1e-11`. Every value must be finite.
The common modal set must be recovered from input-driven discovery, and every
contact must have a nonzero fitted residue for every retained mode.

OOD gates are fail-closed:

| Control | Required observation and decision |
| --- | --- |
| weak excitation | valid coverage above `4 kHz <= 0.35`; `OOD_WEAK_EXCITATION`; zero admitted modes |
| low coherence | median band coherence `<= 0.80` or valid coverage `<= 0.50`; `OOD_LOW_COHERENCE`; zero admitted modes |
| missing second impact | median per-repeat fitted-transfer reconstruction NRMSE `>= 0.15`; `OOD_MODEL_MISMATCH`; zero admitted modes |

Hard gates additionally require exact role/counter order, exact force/contact/
mode counts, no holdout generation after a development failure, all external
outputs outside Git, canonical JSON, deterministic `.npy` arrays, zero network
and real-data access, exact implementation/protocol/dependency hashes and
byte-identical manifest, model, arrays and report across two complete runs.

## Decision and stop rule

- `PASS_KNOWN_TRUTH_FRF`: preflight passes; all development and holdout numeric,
  conditioning, OOD, count, lineage and repeat gates pass. Only B2 zero-decode
  source feasibility is authorized.
- `REJECT_ESTIMATOR`: any valid numeric or OOD gate fails. Do not relax a
  threshold or alter fixture/noise/roles on this revision; formulate a new
  estimator hypothesis before another real-data step.
- `INVALID_ORACLE_RUN`: lineage, finiteness, access ordering, serialization or
  exact-repeat failure. It gives no method evidence.

Regardless of the decision, no B2 waveform, ObjectFolder protected contact,
runtime contract, neural model, cooked atlas or admission shadow may be opened
by this experiment.
