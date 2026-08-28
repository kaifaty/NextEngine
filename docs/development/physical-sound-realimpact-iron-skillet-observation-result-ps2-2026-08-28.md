# REALIMPACT Iron Skillet observation result — PS-2 — 2026-08-28

## Outcome

The preregistered independent-object method transfer is rejected. One exact
four-request acquisition emits report
`6a99d291be499e06c7ba9f633794a4150d04303b2ddc7932eda2b864a900dfeb`;
the single decode emits
`47acdc35449390f639cda34990520c5f74cb8627d0932079345ca35735ffc099`;
and two offline analyses emit byte-identical report
`a9c4ae36ae04bf9a9772cb8a1a6c55e2862ed918eaff3ae6d9a2043acb07206d`
with decision `IndependentObservationMethodTransferRejected`.

Six of eight frozen gates pass. Onset-to-tail persistence recall is
`0.35294 < 0.50`, and valid adaptive-fit fraction is `0.50 < 0.75`. This
rejects the relative fractional-dominance plus fixed `900 ms` persistence
pipeline on the unopened Iron Skillet object. No threshold or selector was
changed after access.

This result does not reject the recording, establish a material property or
identify a mechanical cause. It grants no quality, admission, runtime, `Pass`
or physics credit. Planter remains sealed.

## Acquisition and decode

The source identities and row contract pass exactly:

- metadata ranges: 707, 392 and 261 bytes;
- audio prefix: exactly 536870912 bytes, SHA-256 `c13ffe30…5ec0`;
- decoded block: exactly 553317600 bytes, SHA-256 `e26d1df1…7eeb`;
- NPY shape: `(3000, 230549)`, dtype `<f4`;
- impact-zero rows: 600 at vertex ID `19308`;
- conditions: 10 angles, four distances and 40 pairs;
- each condition orders microphones `0..14`; and
- reference row 7 is `0° / 0 mm / microphone 7`.

No retry, prefix growth or additional request occurred. The two analyses use
the immutable decoded block and make zero network requests.

## Frozen gate result

| Gate | Observed | Required | Result |
| --- | ---: | ---: | --- |
| Selected persistent modes | 6 | 6..16 | pass |
| Onset-to-tail persistence recall | 0.352941 | ≥0.50 | **fail** |
| Median frequency error | 0.130985 cents | ≤40 | pass |
| Valid adaptive fits | 0.50 | ≥0.75 | **fail** |
| Decaying fraction | 0.50 | ≥0.50 | pass |
| Median valid-fit `R²` | 0.993144 | ≥0.95 | pass |
| Scale-bin invariance | exact at `0.125/1/8` | true | pass |

The selector sees 4217 local onset peaks and reduces them to 17 salient onset
candidates. Nineteen salient candidates exist at the tail, but only six match
injectively within 40 cents. Their frequencies are approximately
`251, 293, 473, 752, 1704, 4437 Hz`. The three lowest have only
`5.23..11.79 dB` adaptive dynamic range and fail the frozen `20 dB` criterion;
the other three have clean fits (`R² 0.9825..0.9936`).

The diagnostic top-16 comparator is not an admission gate, but it is important
counterevidence: it selects 16 frequencies from `293..8159 Hz`, with valid and
decaying adaptive fractions both `0.9375`. Therefore the failure is not simply
"this recording has no decaying modes". The source-derived salience/persistence
selection is the component that fails to transfer.

## Interpretation and competing hypotheses

- `H-dominance`: ±`10%` stronger-bin suppression is too sparse for the dense
  real skillet spectrum and preferentially retains low-frequency bands.
- `H-tail`: fixed `900 ms` persistence discards valid faster-decaying metal
  modes even when the adaptive estimator can fit them earlier.
- `H-boundary`: the pinned helper was applied to a differently normalized
  single response; combining spatial power, a relative floor and its dominance
  heuristic changes the intended estimator boundary.
- `H-recording`: acquisition/deconvolution artifacts dominate both windows.
  Exact row identity and the strong diagnostic comparator weigh against the
  broad version of this hypothesis, but do not eliminate localized artifacts.

The result cannot discriminate dominance from tail timing because they are
coupled in the current pipeline. Do not tune either on Iron Skillet.

## Decision and next action

Status is
`IRON_SKILLET_METHOD_TRANSFER_REJECTED / SALIENCE_SELECTOR_REAL_TRANSFER_REJECTED /
SYNTHETIC_CONTROL_RETAINED / CURRENT_TOP16_DIAGNOSTIC_STRONGER /
BOUNDED_METHOD_RESEARCH_REQUIRED / MECHANICS_BLOCKED / PLANTER_SEALED /
AUTHORED_CLIP_FALLBACK`.

Before another implementation attempt, run a bounded research cycle using the
already opened block only. Freeze diagnostics that independently vary the
dominance stage and tail-time observation without changing admission gates or
publishing a replacement result. Search primary modal-estimation sources for a
multichannel transient method with explicit known-truth controls. Do not open a
new object, retry acquisition, tune Iron Skillet, run physics or access Planter.
