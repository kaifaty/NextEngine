# Physical sound R3A V9 — time-varying residual synthetic preflight protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-08-31` |
| Status | `PREREGISTERED / NO_REAL_AUDIO / IMPLEMENTATION_NEXT` |
| Parent evidence | [V8 fit rejection and V9 research](physical-sound-r3a-v8-object91-fit-result-and-v9-residual-research-2026-08-31.md) |
| Allowed claim | Synthetic implementation viability only |

## Question

Can a deterministic, compact time-varying noise-band residual preserve known
spectro-temporal structure and can a small neural field interpolate its latent
contact control better than nearest-contact lookup, while explicit modes remain
a separate renderer?

This is a successful-control prerequisite, not evidence of realistic glass.
Every real waveform counter remains zero. ObjectFolder object `91`, its
development contact and every sealed source are forbidden inputs.

## Frozen representation

The synthetic renderer is:

`waveform(contact) = explicit_modal(contact) + noise_band_residual(contact)`.

The residual uses:

- native `48,000 Hz`, `48,000` samples;
- `96` deterministic loopable noise bands over `120–18,000 Hz`;
- a fixed PRNG seed and canonical float64 construction;
- `8` nonnegative spectro-temporal atoms with band weights and time envelopes;
- one `8`-value contact latent that mixes those atoms;
- linear interpolation of atom amplitudes to audio rate;
- a coordinate field trained only on context contacts to predict the latent of
  held query contacts.

The explicit path uses `16` global frequencies and damping values with
contact-dependent sine/cosine gains. Noise-band phase is never asked to carry
stable modal identity.

The synthetic object uses a deterministic `7 × 7` surface grid. Context points
are selected by the existing deterministic farthest-point rule; all remaining
points are query. Neither query latent nor query audio contributes to feature
normalization, optimizer state, stopping, capacity or thresholds.

## Frozen model and budgets

- positional encoding: normalized 3D position plus fixed Fourier bands;
- field: two hidden SiLU layers, width `64`, output `8` latent values;
- optimizer: AdamW, fixed seed, full-context updates, one CPU thread and
  deterministic Torch algorithms;
- shared model budget: `≤ 4 MiB` serialized float32 parameters plus fixed
  renderer descriptors;
- per-contact record budget: `≤ 64 KiB`, counting coordinate, modal gains,
  optional damping and latent values in the canonical packed record;
- runtime inference: forbidden; the model is an external cooker experiment and
  a successful successor still bakes ordinary clips.

## Controls

The runner must report:

1. exact noise-band construction repeat, including a basis SHA-256;
2. exact renderer repeat from the same contact record;
3. quantized record reconstruction versus unquantized synthetic truth;
4. neural query latent RMSE;
5. global-mean and nearest-context latent baselines;
6. held-query waveform endpoints for neural, nearest and modal-only outputs;
7. exact repeat of training, parameters and query predictions;
8. model and maximum contact-record byte counts;
9. zero real/development/sealed/method/shadow access counters.

## Frozen gates

V9-SYNTH passes only if all conditions hold:

- all values are finite;
- basis, record decode and two complete model runs repeat byte-identically;
- quantized known-record waveform normalized RMSE is `≤ 0.01`;
- neural query latent RMSE is strictly below global mean and at most `0.60 ×`
  nearest-context RMSE;
- neural query waveform normalized RMSE is at most `0.65 ×` nearest-context
  waveform RMSE and strictly below modal-only;
- every neural query has normalized envelope RMSE `≤ 0.20`;
- median neural-query gain-matched multiresolution log-spectrum RMSE is
  `≤ 4.0 dB`;
- shared and per-contact byte budgets pass;
- all real and protected counters are zero.

The exact full-query spectrum distribution is retained in the report; the
median gate prevents one synthetic boundary point from selecting the method
while still requiring every query to pass envelope structure.

## Decisions

| Outcome | Consequence |
| --- | --- |
| all gates pass twice | `READY_TO_FREEZE_SOURCE_DISJOINT_V9_REAL_PROTOCOL`; no real quality or R3B credit |
| renderer/quantization fails | `REJECT_V9_RESIDUAL_SUBSTRATE`; fix without real data |
| neural field loses controls | `REJECT_V9_CONTACT_FIELD`; do not open real data |
| budget fails | `REJECT_V9_CAPACITY`; do not weaken `64 KiB / 4 MiB` budgets |

No post-result threshold change, third capacity or object-91 repair is allowed.
Any successor must receive a new revision and new source-disjoint roles.

