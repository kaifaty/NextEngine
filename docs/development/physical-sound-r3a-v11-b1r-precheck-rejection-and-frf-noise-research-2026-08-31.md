# Physical sound R3A V11 B1R — precheck rejection and FRF noise research

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Protocol | [B1R joint modal FRF](physical-sound-r3a-v11-b1r-joint-modal-frf-protocol-2026-08-31.md) |
| Status | `INVALID_PROTOCOL_PRECHECK / NO_RUNNER_COMMIT / NO_HOLDOUT / REAL_DATA_CLOSED` |
| Decision | Retire B1R before evidence execution |
| Product effect | None; authored clips remain authoritative |

## Precheck result

The focused implementation precheck falsified B1R's force-observability rule
before a runner or manifest was committed and before an evidence run existed.
The frozen rule normalizes observed force power by the maximum observed power
inside `200…12,000 Hz`:

```text
force_valid = Sxx / max_band(Sxx) >= 1e-5
```

For the weak `hann(1025)` force, nearly all true signal energy lies below the
analysis band. The in-band maximum is therefore sensor-noise power, and every
other noise-dominated bin appears well-conditioned relative to that maximum.
Using the committed B1 V1 generator and the same formula gives:

| Diagnostic | Result |
| --- | ---: |
| Force-valid coverage, `200…12,000 Hz` | `1.0` |
| Force-valid coverage, `4,000…12,000 Hz` | `1.0` |
| High-band normalized conditioning range | `3.2685e-5…0.004251` |
| High-band median response coherence | `0.206130` |
| Frozen weak-force OOD maximum | `0.35` |

The intended weak-force OOD can never pass under this noise-relative
normalization. This is a protocol defect, not a candidate-quality result.
No B1R runner, manifest, model, development result or holdout result is claimed.

## Falsifiable hypotheses after two cycles

### H1 — response coherence alone caused B1 V1

Supported for the clean fixture: force-only coverage is approximately `99.98%`
while contact-local response coherence removes the seventh modal neighborhood.

Falsified as a complete solution: removing coherence from the mask causes a
noise-only force band to look fully observable. A separate force axis is still
necessary, but the proposed normalization is wrong.

### H2 — ordinary H1 remains the correct estimator once masking is repaired

Unproven. H1 assumes negligible input measurement noise. Our synthetic force
channel explicitly contains noise, so H1 can be biased precisely in weak bands.

### H3 — a noise-aware errors-in-variables/local rational estimator is needed

Supported by the literature and the precheck. Source observability must compare
force signal power with an independently estimated force-noise floor; transfer
estimation must account for noise on both force and response. Shared poles
should be fitted over local frequency neighborhoods rather than inferred from
independently thresholded bins.

## Primary-source research

- Allemang et al., [*Frequency response function estimation techniques and the
  corresponding coherence functions: A review and update*](https://doi.org/10.1016/j.ymssp.2021.108100),
  Mechanical Systems and Signal Processing 162, 2022. The review distinguishes
  H1/H2 ordinary least squares assumptions from Hv/SVD total-least-squares
  estimators for noise on both input and output and warns that conditioned
  coherence interpretations are estimator-dependent.
- Schoukens, Godfrey and Schoukens,
  [*Nonparametric Data-Driven Modeling of Linear Systems*](https://doi.org/10.1109/MCS.2018.2830080),
  IEEE Control Systems 38(4), 2018. Their bias analysis shows that input noise
  attenuates the H1 estimate; they recommend choosing an estimator from the
  actual input/output SNR assumptions and retaining the reference signal.
- Coletti, Carter and Schultz,
  [*Local modeling for FRF estimation with noisy input measurements*](https://doi.org/10.1016/j.jsv.2024.118289),
  Journal of Sound and Vibration 576, 2024. They explicitly extend local
  rational FRF fitting to noisy input and output data with an errors-in-
  variables adjustment and automatic model selection.
- Pintelon et al.,
  [*Frequency Response Function Measurements via Local Rational Modeling,
  Revisited*](https://doi.org/10.1109/TIM.2020.3020601), IEEE Transactions on
  Instrumentation and Measurement 70, 2021. Local rational models reduce
  leakage and resolution bias near rapidly varying lightly damped resonances
  and support automatic local order selection.

These sources support estimator selection and uncertainty handling; they do not
prove that one algorithm passes the NextEngine fixture or real impact data.

## Decision

Retire B1R without implementing or executing it. Do not change the committed
protocol in place, reinterpret the failed precheck as quality evidence, or open
B2 data.

The smallest evidence-backed successor is B1R2:

1. generate and retain separate noise-only force/response ensembles before any
   impact trial;
2. define force observability from noise-corrected power and explicit input SNR,
   never from a noise-relative in-band maximum alone;
3. compare H1, H2 and one errors-in-variables/TLS or local-rational candidate on
   fresh known truth;
4. select shared poles by local rational fit with pooled contact support;
5. keep per-contact residue uncertainty, weak-force, low-coherence and missing-
   input OOD as independent gates;
6. use fresh phases and all fresh fit/development/holdout/noise seeds.

Only a separately committed B1R2 protocol and runner may resume numeric work.
