# Physical sound R3A V10 A1R — object-51 force-onset fit protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / INVENTORY_PASS / ONLY_FIT_PCM_AUTHORIZED` |
| Source manifest | `3041c19d07f78fc9bdfd092ced72e07b5e795b235a286a23ede68108556b6ed5` |
| Fit contacts | Object `51`: `27,15,4,3` |
| Protected contacts | development `9`; sealed `18` |
| Product effect | None; authored clips remain authoritative |

## Question

Can the unchanged V9 explicit-mode/time-varying-residual representation fit
four fresh real Glass impacts when alignment is derived from the source's
synchronized measured force rather than inferred from microphone noise?

This tests representation fit only. It does not test contact interpolation,
project-disjoint transfer, validator quality or runtime behavior.

## Source and read boundary

1. Validate the exact A1R inventory manifest, raw prefix, official page and all
   five compact-source hashes before reading a WAV payload.
2. Stream only `mic.wav` and `Force.wav` for fit contacts `27/15/4/3` from the
   raw prefix; validate their inventory commitments and native headers.
3. Decode exactly `4 * 2 * 288,000 = 2,304,000` PCM16 values: `1,152,000`
   microphone and `1,152,000` force values.
4. Development `9`, sealed `18`, all other contacts and every unrelated raw
   payload must report zero decoded sample values.
5. No source waveform, target, prediction or model payload enters Git.

## Frozen force-derived preprocessing

For each fit pair:

1. Convert PCM16 to float64 by division by `32768`; do not resample or filter.
2. Subtract the median of the first `12,000` samples independently from the
   microphone and force channels.
3. On the centered force channel compute absolute values, its whole-recording
   peak and the first-`12,000` absolute median/MAD.
4. Set the force threshold to
   `max(0.05 * force_peak, baseline_abs_median + 10 * baseline_abs_MAD)` and
   select the first sample `>=` that threshold. Missing onset rejects the
   revision before representation fitting.
5. Shift microphone and force together without wrap so force onset is sample
   `512`; zero-pad the exposed edge and retain exactly `144,000` samples.
6. Apply one fit-only microphone scale so the maximum absolute retained sample
   across all four contacts is `0.92`.

The force waveform supplies synchronization evidence only. It is neither a
target nor a modal convolution input, and it is not mislabeled as an acoustic
transfer response. This preserves the A1 V9 representation while changing the
failed onset source.

## Frozen representation and budget

Every representation, decoder and record rule is inherited unchanged from the
[A1 Beer Glass protocol](physical-sound-r3a-v10-beer-glass-real-fit-protocol-2026-08-31.md),
whose current SHA-256 is
`0883675f656f29a6881b7701e8eb6dc83aca68989a830c799e39ac8b260f83e1`:

- `64` shared modes in `120–18,000 Hz`, minimum separation `8 Hz`;
- pooled `8192`-sample Hann-STFT damping, hop `1024`, clamp
  `[0.25,200] s^-1`;
- per-contact signed direct-onset sine/cosine gains;
- `96` deterministic triangular noise bands, loop `16,384`, seed
  `20,260,831`;
- `2048`-sample Hann-STFT residual envelope, hop `128`, first `8`
  orthonormal DCT-II coefficients;
- first `2,048` int16 transient correction and one final float32 LS scale;
- exactly `5,904` expected bytes per contact, maximum `65,536`;
- maximum `4 MiB` shared decoder state.

Modal-only and stationary-first-coefficient reconstructions remain report-only
ablations and cannot be selected.

## Frozen fit gates

Every fit contact must independently pass:

| Endpoint | Maximum |
| --- | ---: |
| Absolute RMS level error | `0.5 dB` |
| Gain-matched multiresolution log-spectrum RMSE | `4.0 dB` |
| Normalized envelope RMSE | `0.20` |
| Median modal-frequency error | `100 cents` |
| T60 relative error | `0.35` |

Hard gates additionally require exact source/sample accounting, all finite
values, exact mode/band counts and ranges, nonzero temporal residual variation,
shared/contact budgets, deterministic carrier/model/record/prediction/report
hashes and zero protected reads. A pooled mean cannot hide a failed contact.

## Decision and stop rule

- `READY_FOR_A1R_DEVELOPMENT`: all four fit contacts and hard gates pass in two
  byte-identical runs. Only then may a separate committed evaluator decode
  development contact `9`.
- `REJECT_A1R_V9_REAL_REPRESENTATION`: any valid fit/hard gate fails. Do not
  decode development or sealed data.
- `INVALID_A1R_SOURCE_OR_RUNNER`: lineage, identity, decode accounting or exact
  repeat fails; this grants no quality evidence.

After a valid reject, do not alter force threshold, contact subset, bank/DCT/
mode count, seed, metrics, thresholds or postfilter on these opened contacts.
Reconsideration requires a materially different preregistered representation
and fresh object/source revision.

Even a fit pass authorizes no project-disjoint claim, exact-object field,
validator, atlas, public contract or runtime neural inference.
