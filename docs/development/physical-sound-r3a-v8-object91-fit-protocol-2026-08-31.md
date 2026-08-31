# Physical sound R3A V8 — ObjectFolder Real object-91 fit protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / FIT_PCM_CLOSED / DEVELOPMENT_PCM_CLOSED` |
| Source evidence | [Zero-decode inventory pass](physical-sound-r3a-v8-objectfolder-real-inventory-result-2026-08-31.md) |
| Fit contacts | `18`, `12`, `4` |
| Development / sealed | `20` / `27`, numerically unread |
| Product effect | None; authored clips remain authoritative |

## Hypothesis

V4 may have failed not because explicit modal structure is useless, but because
its residual capacity was orders of magnitude too small and it ignored the
measured excitation. Test one bounded representation with:

1. object-global modal frequencies;
2. measured `Force.wav` convolved with damped modal impulse responses;
3. per-contact signed modal gains;
4. a phase-preserving sparse complex residual selected globally from fit only;
5. a bounded block envelope and short transient correction;
6. either global damping or one preregistered per-contact damping ablation.

This is an autoencoding representation-fit test. Deriving a contact record
from its own target is permitted; contact interpolation is not tested and R3B
remains closed.

## Frozen extraction and preprocessing

- Validate the exact inventory manifest `e1651b64…147b` and source prefix
  `5ef9789a…7313` before extraction.
- Stream only through the three fit contacts and extract only their `mic.wav`,
  `Force.wav`, `metadata.yaml` and `striking_force.yaml` members in memory.
- Decode PCM16 mono at native `48,000 Hz`; no resampling or filtering before
  onset alignment.
- Remove each channel's median over the first `0.25 s`.
- Force onset is the first absolute centered force sample above
  `max(0.05 * peak, median(abs(force)) + 10 * MAD(abs(force)))`.
- Shift audio and force together so onset is sample `512`, zero-padding rather
  than wrapping, then retain exactly `144,000` samples (`3 s`).
- Normalize each force channel to unit peak. Apply one fit-only audio scale so
  the largest absolute sample over all three aligned fit contacts is `0.92`.
- Parse fit YAML only after role/source validation. YAML values are provenance
  metadata, not model-selection inputs.

Decoded-sample accounting must report exactly `3 * 2 * 288000 = 1,728,000`
source sample values (audio plus force), `864,000` fit audio/force values used
after alignment, and zero development/sealed values.

## Frozen modal initializer

- Estimate `64` object-global frequencies from the RMS-pooled magnitude spectra
  of the three aligned fit microphone signals.
- Eligible band: `120–18,000 Hz`; minimum peak separation `6 Hz`; strongest
  peaks first, then store frequencies sorted ascending.
- Estimate global damping by log-linear regression of the pooled `8192`-sample
  Hann-STFT magnitude envelope at each selected frequency, hop `1024`, using
  the peak-to-`2.5 s` region above `1e-3` of its maximum.
- Clamp damping to `[0.25, 200] s^-1`; use `20 s^-1` only when fewer than six
  regression points exist.
- For each contact and mode, create sine and cosine impulse responses and
  convolve them with that contact's normalized measured force. Fit the two
  signed gains by deterministic sequential least squares in descending pooled
  peak-strength order.

The `global-damping` capacity uses these pooled values. The only successor,
`per-contact-damping`, re-estimates damping at the same frozen frequencies from
each contact's own fit-only STFT and stores the 64 values in its contact record.
Frequencies never vary per contact.

## Frozen residual record

For each capacity:

1. subtract the force-convolved modal reconstruction from every fit target;
2. choose one object-global set of the `14,500` highest RMS residual rFFT bins
   in `120–18,000 Hz`, using all three fit residuals and stable index tie-breaks;
3. store each contact's complex coefficients at those bins as symmetric int16
   real/imaginary values plus one float32 scale;
4. inverse-rFFT the sparse residual at exactly `144,000` samples;
5. fit `512` nonnegative block gains by target/candidate least squares, clamp
   to `[0, 4]`, linearly interpolate them and store as float16;
6. store the remaining correction over the first `2,048` samples as int16 plus
   one float32 scale;
7. apply one final float32 least-squares output scale.

This residual preserves selected complex phase. It is not random noise and it
does not claim a compact runtime formula; its purpose is to discriminate
whether an explicit modal field plus a bounded contact record can carry real
impact structure. The later atlas still bakes ordinary clips offline.

## Cost boundary

Per-contact encoded bytes are computed from actual stored arrays:

| Component | Global damping | Per-contact damping |
| --- | ---: | ---: |
| Complex residual int16 | `58000 + 4` | `58000 + 4` |
| Transient int16 | `4096 + 4` | `4096 + 4` |
| Envelope float16 | `1024` | `1024` |
| Modal gains float16 + scale | `256 + 4` | `256 + 4` |
| Damping float16 | `0` | `128` |
| Final scale float32 | `4` | `4` |
| Total | `63392` | `63520` |

Both must remain below `65,536` bytes. Shared poles and bin indices must remain
below `4 MiB`. No third capacity, larger bin set or post-fit repair is allowed.

## Fit gates

Each capacity is evaluated on all three fit contacts with the unchanged V4/V5
endpoints and absolute limits:

| Endpoint | Maximum |
| --- | ---: |
| Absolute RMS level error | `0.5 dB` |
| Gain-matched multiresolution log-spectrum RMSE | `4.0 dB` |
| Normalized envelope RMSE | `0.20` |
| Median modal-frequency error | `100 cents` |
| T60 relative error | `0.35` |

Additional hard gates: finite arrays/metrics, exact native length/rate, actual
record/shared byte budgets, every selected-bin index in band, and deterministic
reconstruction hashes.

Selection order is frozen:

1. choose `global-damping` if it passes every hard and acoustic gate;
2. otherwise choose `per-contact-damping` only if it passes every gate;
3. otherwise return `REJECT_V8_REAL_FIT_REPRESENTATION` and do not decode
   development contact `20`.

Two independent fit runs must reproduce manifest/report/model/record bytes.
Only `READY_FOR_V8_REAL_DEVELOPMENT_EVALUATION` may authorize a separate,
committed development evaluator. No threshold, bin count, transient length,
mode count or capacity may change after this fit result.

## Non-claims

A fit pass would show only that the frozen record can reconstruct the three
opened fit recordings within cost. It would not show held-contact transfer,
contact-coordinate learning, perceptual glass identity, source-disjoint
generalization, validator admission or engine readiness.
