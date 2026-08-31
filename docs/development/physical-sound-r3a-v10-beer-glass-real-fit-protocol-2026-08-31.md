# Physical sound R3A V10 — Beer Glass V9 real-fit protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / FIT_DECODE_AUTHORIZED / ALL_OTHER_PCM_CLOSED` |
| Source manifest | `8a30cef02684ddbc338d03a36e7cfab3dca2d5273e6d1d1fa885410e56048728` |
| Fit object | ObjectFolder Real `60 / Beer_Glass / Glass` |
| Fit contacts | `0,1,2,3,5,6,7,8,10,12,14,15,17,18,19,21,22,24,26,27,28,29` |
| Product effect | None; authored clips remain authoritative |

## Question

Can the single frozen V9 representation reconstruct all 22 real Beer Glass
fit recordings within the unchanged acoustic and cost gates?

This is a representation-fit test. Each contact record may be derived from its
own target; contact interpolation is not evaluated. A pass authorizes only the
five frozen object-60 development contacts. A failure closes V9 before
development and records the counterexample.

## Protected data

- Object `60` development: `4,11,16,23,25`.
- Object `60` exact-object query: `9,13,20`.
- Object `22` representation holdout: `12,13,16,17,26`.
- Object `22` protected unused: every other contact.
- Every method holdout and admission shadow from prior revisions.

The runner must validate the exact source manifest and report zero decoded
sample values for all protected roles.

## Frozen extraction and preprocessing

1. Validate all five external source files against the A0 sizes and SHA-256
   values before opening the audio tar.
2. Stream only the 22 selected `audio/60/<contact>.wav` members into memory.
3. Decode PCM16 mono at native `48,000 Hz`; do not resample or filter.
4. Convert to float64 by division by `32768`.
5. Subtract the median of the first `0.25 s` independently per contact.
6. Define onset as the first absolute sample exceeding both `0.06 * peak` and
   `median(abs(baseline)) + 12 * MAD(abs(baseline))`, where baseline is the
   first `0.25 s`. Failure to find onset rejects the fit run.
7. Shift without wrap so onset becomes sample `512`, zero-padding as needed.
8. Retain exactly `144,000` samples (`3 s`).
9. Apply one fit-only global scale so the largest absolute retained sample over
   all 22 contacts is `0.92`.

Expected decoded source sample count is exactly
`22 * 288,000 = 6,336,000`; retained fit count is
`22 * 144,000 = 3,168,000`. No force samples are decoded because the compact
published revision does not contain that axis.

## Frozen explicit modes

- Estimate `64` object-global frequencies from the RMS-pooled magnitude rFFT
  over all aligned fit targets.
- Eligible band is `120–18,000 Hz`; minimum peak separation is `8 Hz`.
- Select strongest eligible peaks with stable lower-bin tie-break, then store
  frequencies ascending.
- Estimate one object-global damping value per frequency by log-linear
  regression of the pooled `8192`-sample Hann-STFT magnitude envelope, hop
  `1024`, from its peak through `2.5 s` while magnitude remains above `1e-3` of
  peak.
- Clamp damping to `[0.25, 200] s^-1`; use `20 s^-1` when fewer than six valid
  regression points exist.
- For each contact, fit signed sine/cosine gains by deterministic sequential
  least squares in descending pooled peak-strength order.

The selected compact source lacks measured force. The renderer therefore uses
direct onset excitation and may claim only recorded-impact reconstruction.

## Frozen V9 time-varying residual

The modal residual uses one capacity only:

1. Split `120–18,000 Hz` into `96` logarithmically spaced triangular rFFT
   bands with adjacent half-overlap.
2. Measure each residual band's short-time RMS envelope at hop `128`.
3. Encode the log envelope with the first `8` orthonormal DCT-II coefficients;
   store coefficients as float16 with one float32 scale per contact.
4. Regenerate one deterministic loopable carrier per band from seed
   `20,260,831`, loop length `16,384`, and the frozen triangular frequency
   response. Carrier samples or weights are never stored in the contact record.
5. Decode the DCT envelopes, clamp them nonnegative in linear amplitude, apply
   them to tiled carriers and sum all bands in fixed ascending order.
6. Store the remaining correction over the first `2,048` samples as symmetric
   int16 plus one float32 scale.
7. Apply one final float32 least-squares output scale.

The only report-only ablations are modal-only and a stationary residual using
the first temporal coefficient. They cannot be selected as a candidate.

## Cost boundary

The actual serialized record is authoritative. Expected maximum components:

| Component | Bytes |
| --- | ---: |
| Modal sine/cosine gains, float16 | `256` |
| Modal gain scale, float32 | `4` |
| Residual `96 x 8` coefficients, float16 | `1,536` |
| Residual coefficient scale, float32 | `4` |
| Transient `2,048` int16 samples | `4,096` |
| Transient scale, float32 | `4` |
| Final output scale, float32 | `4` |
| Maximum expected record | `5,904` |

Every contact record must be `<= 65,536` bytes. Shared frequencies, damping,
band definitions and any explicitly serialized decoder state must be
`<= 4 MiB`. Analytically regenerable deterministic carriers do not count as
stored state, but their seed/profile hashes are mandatory.

## Frozen fit gates

Use the unchanged V4/V5/V8 endpoints and absolute limits on every one of the
22 contacts:

| Endpoint | Maximum |
| --- | ---: |
| Absolute RMS level error | `0.5 dB` |
| Gain-matched multiresolution log-spectrum RMSE | `4.0 dB` |
| Normalized envelope RMSE | `0.20` |
| Median modal-frequency error | `100 cents` |
| T60 relative error | `0.35` |

Additional hard gates:

- finite source, model, record, reconstruction and metrics;
- exact native source rate/length and aligned length;
- exact decoded-sample accounting;
- all modes and residual bands inside the frozen evaluation band;
- actual shared/contact byte budgets;
- nonzero residual temporal variation beyond the stationary ablation;
- deterministic model, record, reconstruction, manifest and report hashes;
- zero protected-role decoded samples.

The single candidate passes only if every hard and acoustic gate passes on
every fit contact. No pooled mean may hide one failed recording.

## Decision and stop rule

- `READY_FOR_V9_REAL_DEVELOPMENT`: two runs reproduce exact artifacts and all
  fit contacts pass. Only then may a separate committed evaluator decode
  object-60 development IDs `4,11,16,23,25`.
- `REJECT_V9_REAL_REPRESENTATION`: any fit contact or hard gate fails. Do not
  decode development, query or holdout.
- `INVALID_SOURCE_OR_RUNNER`: source, lineage, role, decode accounting or
  deterministic repeat fails. This is no quality evidence.

After a valid rejection, another bank count, DCT rank, mode count, threshold,
postfilter, contact subset or random seed on these opened fit recordings is
forbidden. Reconsideration requires a materially different preregistered
representation and a new source revision.

## Non-claims

A fit pass would not prove held-contact transfer, perceptual glass identity,
force scaling, listener radiation, exact-object field learning, validator
admission, baked-atlas quality or engine readiness. Runtime neural inference,
public contracts and production contact wiring remain unauthorized.
