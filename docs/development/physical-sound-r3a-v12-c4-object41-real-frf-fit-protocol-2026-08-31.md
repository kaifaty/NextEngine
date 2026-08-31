# Physical Sound R3A V12-C4a — object-41 real FRF fit protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `AMENDED_BEFORE_RUNNER_PREFLIGHT / FIT_PCM_CLOSED` |
| Target | ObjectFolder-Real `41 / Wrench_Large / Steel` |
| Role | `estimator_fit`: contacts `30,13,24,14,19,9,6,15,5,33,2,29,4,16,31,34` |
| Prerequisite | [C3 exact result](physical-sound-r3a-v12-c3-object41-source-role-freeze-result-2026-08-31.md) |
| Research basis | [C4 bounded research](physical-sound-r3a-v12-c4-real-frf-fit-research-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Question and bounded claim

Can a force-certified, regularized one-shot FRF estimator recover a stable set
of shared poles from the sixteen fit contacts, reconstruct their measured
microphone responses from measured force, and fail closed on causal and
coverage controls?

C4a is fit-only. It does not evaluate contact interpolation, unseen response
quality, material identity, automatic admission or runtime behavior. A pass
opens only a separately committed C4b development protocol.

## Exact source and read budget

The runner must bind the C3 manifest, report and exact external `4 GiB` prefix
before signal access. It may decode only the sixteen frozen fit pairs:

- microphone: `16 * 288,000 = 4,608,000` PCM16 values;
- force: `16 * 288,000 = 4,608,000` PCM16 values;
- total decoded signal values: exactly `9,216,000`;
- retained after preprocessing: `16 * 2 * 192,000 = 6,144,000` float64 values.

Every other object-41 contact and role must report zero decoded samples and
zero emitted payload. Network requests are zero. Source WAVs, arrays, models,
reports and audition audio remain external and never enter Git.

## Runner stages and ordering

1. `freeze` binds this protocol, research note, runner, C1 estimator helpers,
   C3 manifest/report and exact source identities without signal access.
2. `preflight` runs twice, reproduces all roles/budgets/hashes and reports zero
   source bytes and samples read.
3. `run` validates C3 lineage, decodes exactly the fit pairs, derives force
   onset and the acquisition certificate, freezes that certificate in memory,
   and only then exposes microphone values to transfer fitting.
4. Run A and Run B use independent empty output roots and must emit
   byte-identical canonical manifest, certificate, model, arrays and report.

Any out-of-order microphone analysis, protected-role read or retry with changed
parameters is `INVALID_C4A_RUN` and grants no estimator evidence.

## Frozen paired preprocessing

For each channel pair:

1. Convert PCM16 to float64 by exact division by `32768`.
2. Independently subtract the median of the first `12,000` samples from force
   and microphone; no per-contact or global amplitude normalization is allowed.
3. On centered force compute peak, first-`12,000` absolute median/MAD and
   threshold `max(0.05 * peak, median_abs + 10 * MAD_abs)`.
4. Select the first force sample at or above threshold. Require exactly one
   primary event under the frozen double-impact discriminator: no later peak
   above `0.5 * primary_peak` may begin after a `240`-sample below-threshold
   gap. Missing/ambiguous impact is `OOD_FORCE_EVENT`.
5. Shift force and microphone together without wrap so onset is sample `512`;
   retain exactly `192,000` samples and zero-pad only the exposed leading edge.
6. Do not resample, denoise, bandpass, phase-align from microphone, apply an
   exponential window, compress, normalize or postfilter.

The last `24,000` microphone samples must have RMS at most `0.10` of samples
`512..24,511`. Otherwise return `DATA_INSUFFICIENT_RESPONSE_DURATION`; do not
window the response or shorten the damping claim.

## Force-only acquisition certificate

Use sample rate `48,000 Hz`, retained length `192,000` and FFT length
`262,144`. For each fit contact, estimate baseline force-noise power from the
first `12,000` original centered samples using Hann windows of `2,048`, hop
`1,024`, the same FFT length and median segment power normalized by window
energy. Scale the estimate to the retained FFT length and floor it by
`192,000 * (1.4826 * baseline_MAD)^2`.

For contact `c` and bin `k`:

```text
Gxx[c,k] = max(|FFT(force_c)|^2 - Nxx[c,k], 0)
input_snr[c,k] = Gxx[c,k] / max(Nxx[c,k], 1e-30)
relative_power[c,k] = Gxx[c,k] / max(max_{f<=12kHz} Gxx[c,f], 1e-30)
```

A contact supports a bin only inside `200…9,500 Hz`, with `input_snr >= 25`
and `relative_power >= 0.005`. A bin is certified when at least four of the
sixteen contacts support it. The certificate must have:

- supported target-band fraction `>= 0.40`;
- cumulative certified width `>= 3,000 Hz`;
- longest continuous certified interval `>= 400 Hz`;
- the same three gates after deleting each frozen quarter of four contacts and
  requiring at least three supporters among the remaining twelve.

The quarters are contiguous blocks in the committed C3 hash order. Response,
coordinates, material labels and discovered modes cannot enter this stage.
Failure is `DATA_INSUFFICIENT_FORCE_COVERAGE`; no microphone fit follows.

## Candidate and controls

For each contact, compute the certificate-masked transfer

```text
H_reg = Y * conj(X) / (|X|^2 + Nxx)
```

only in certified bins, with the existing deterministic certificate taper.
Inverse FFT yields the candidate impulse response. The unchanged C1
Blackman-Harris/Gabor common-pole core discovers shared frequency/damping:

- `1024` window, `32` hop, first `256` frames;
- `200…9,500 Hz`, `-35 dB` region floor, `±1` neighboring bin;
- `24` pencil lags, maximum local order `6`, score margin `20`;
- duplicate radius `1 Hz`, post-fit energy floor `-30 dB`.

Contact-local complex residues are refit by deterministic least squares with
the global frequency/damping fixed. The measured microphone prediction is the
measured force convolved with that explicit modal transfer; no waveform
residual is allowed.

Frozen controls are:

- unregularized direct division/raw one-shot H1 in certified bins;
- shortest-response impulse assumption;
- input-ignorant common-pole fit on aligned microphone outputs;
- a fixed cyclic force permutation by one position in C3 hash order, with
  microphone/contact residues unchanged;
- baseline-only zero-force acquisition, which must return coverage OOD.

The one-shot source has no repeated force profiles, so the exact control
construction is frozen before PCM access as follows. The only aligned
microphone response for a contact is treated as its unit-impulse transfer,
target-band tapered without borrowing the acquisition certificate, and then
convolved with that contact's measured force. The input-ignorant modal control
fits poles/residues directly to the aligned microphone response, treats that
reconstruction as a transfer, and likewise convolves it with measured force;
scoring the reconstruction against itself is forbidden. The zero-force
control places the original centered `12,000`-sample force baseline at the
start of a zero-filled `192,000`-sample record and reruns the unchanged
force-only certificate. It admits zero poles only by returning
`DATA_INSUFFICIENT_FORCE_COVERAGE`.

Source accounting separately reports the full `4 GiB` identity-hash pass and
the compressed archive scan. Force PCM is decoded first. Microphone bytes may
be structurally selected while streaming the interleaved archive, but no
microphone sample is decoded and no response-derived value enters the force
certificate. On a certificate failure the microphone decoded-sample count
remains zero; on a pass it must equal the frozen `4,608,000`-sample budget.

Ordinary coherence is recorded as `NOT_APPLICABLE_SINGLE_RECORD` and cannot
support a pole or pass a gate.

## Stability and fit gates

Refit the complete candidate after omitting each frozen quarter. A full-fit
pole is stable in a refit when a unique pole matches within `2 Hz` and within
`max(2 s^-1, 25% relative)` damping. Retain only poles stable in at least three
of four refits, with certified fraction `>=0.90` in their local neighborhood
`±max(24 Hz, 4*decay/(2*pi))` and non-negligible residue (`>= -35 dB` of that
contact's strongest residue) in at least eight contacts.

All gates are conjunctive:

| Endpoint | Gate |
| --- | ---: |
| Retained stable poles | `6…64` |
| Stable/full-fit candidate fraction | `>= 0.75` |
| Contacts with non-negligible residue per retained pole | `>= 8` |
| Mean / maximum measured-response NRMSE | `<= 0.20 / <= 0.35` |
| Mean gain-matched log-spectrum RMSE | `<= 4.0 dB` |
| Mean normalized envelope RMSE | `<= 0.20` |
| Candidate / impulse mean NRMSE | `<= 0.80` |
| Candidate / raw-H1 mean NRMSE | `<= 1.50` |
| Candidate / input-ignorant modal mean NRMSE | `<= 1.05` |
| Force-permuted / candidate median NRMSE | `>= 1.05` |
| Baseline-only zero-force admitted poles | `0` |

The force-permutation gate proves only that the measured force contains useful
causal information in this fit set. If it fails while all numeric paths are
valid, decide `OOD_FORCE_NOT_DISCRIMINATIVE`, not representation success.

Hard gates additionally require finite values, positive damping, exact counts,
canonical mode order, no duplicate match, bounded arrays, dependency hashes,
sample/read accounting and two-run byte identity. No pooled mean may hide a
contact above the maximum NRMSE gate.

## Decisions and stop rule

- `READY_FOR_C4B_DEVELOPMENT_PROTOCOL`: every source, coverage, stability,
  reconstruction, control and repeat gate passes. Development still remains
  closed until its own committed protocol.
- `DATA_INSUFFICIENT_FORCE_COVERAGE`, `DATA_INSUFFICIENT_RESPONSE_DURATION`,
  `OOD_FORCE_EVENT` or `OOD_FORCE_NOT_DISCRIMINATIVE`: the published fit data
  cannot support the dependent claim. Keep every later role sealed.
- `REJECT_C4A_REAL_COMMON_POLE`: acquisition is admissible but a valid
  stability/reconstruction/control gate fails. Do not tune thresholds, select
  contacts or open development. The only nearby successor is a separately
  preregistered local-rational/polyreference comparison on the same fit role.
- `INVALID_C4A_RUN`: lineage, order, accounting, finiteness, serialization or
  repeatability fails and grants no evidence.

Every outcome keeps authored clips mandatory and authorizes no Physical Sound
Record, ML, validator release, atlas, public contract or runtime model.
