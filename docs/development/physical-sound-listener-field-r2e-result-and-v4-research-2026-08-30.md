# Physical sound R2E result and Roadmap V4 research

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Status | `R2E_REJECTED / REPRESENTATION_AND_INTERPOLATION_LIMITED / ROADMAP_V4_REBASELINE` |
| Scope | One fixed REALIMPACT Green Goblet impact, published listener grid, external research only |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Roadmap | [Physical sound synthesis Roadmap V4](../plans/physical-sound-synthesis-roadmap.md) |

## Decision

Close the opened R2 fixed-impact listener-field family. The frozen R2E neural
candidate fits all 420 context rows essentially exactly, repeats byte-for-byte,
and still loses to every classical control on every primary held-listener
endpoint. A post-reject query projection oracle then shows that the frozen
context-only rank-96 representation also cannot cross the complete five-
endpoint rule even when it is given the true query recording.

This is not evidence that neural physical sound is infeasible. It is evidence
that the next neural task must not be another nearby coordinate network over
the same opened 20-degree listener split. Roadmap V4 moves the first product-
useful model to an impact-conditioned, object-specific modal sound field at a
declared canonical listener condition. Environmental spatialization remains in
SPEC-08; detailed object radiation becomes a later optional axis that requires
denser published measurements or an independently validated solver.

No model, quality admission, public record or runtime path is authorized.
Authored clips remain mandatory.

## Frozen R2E protocol

The sole candidate is
`group_conditioned_cosine_coefficient_field_v1`:

- inputs: published azimuth, one of four distance offsets and one of 15
  microphone IDs;
- representation: the context-only rank-96 complex STFT basis passed by R2D
  V2;
- model: seven cosine angular features and one exact distance/microphone-
  conditioned linear head, `80,640` parameters;
- optimizer: the passed half-cosine AdamW schedule, `0.05 -> 0.00001`, `1,600`
  steps;
- context: seven angle planes, `420` rows;
- query: three complete angle planes at 40, 100 and 160 degrees, `180` rows;
- decision: strictly lower than every frozen nearest, linear and complex-STFT
  control on all five primary aggregates.

The freeze manifest is
`0facb72bcbbc215779cb7afd96cc6aa8391335cdd26e9b23de1f7cb526379915`.
Both training runs produce report
`c23a82d0f2d9975636f0984c368a0dcd061f0633aec9150b03d4040bcdd4f790`,
checkpoint
`8b96da41eb4f6232066a1a1ea81d47b1ceeb25807d4796799004ea976160355d`
and the same 208 prediction WAVs. Query audio reads remain zero until the
candidate and repetitions are frozen.

Context fitting passes completely:

| Metric | Result |
| --- | ---: |
| Final objective | `3.3132e-15` |
| Raw coefficient NMSE | `3.5556e-15` |
| Whitened coefficient MSE | `3.0708e-15` |
| Mean absolute log-energy error | `0` |
| Context cook failures | `0` |

The one authorized evaluation report is
`efab8e2cf9d278412445edb2b57695cc7f53ce0ac883b015e21a2435abbb2327`
and returns `RejectLowRankCoefficientField`:

| Primary endpoint | R2E | Best frozen control | Pass |
| --- | ---: | ---: | --- |
| Mean absolute RMS-level error, dB | `4.5182` | `1.8584` | no |
| P95 absolute RMS-level error, dB | `11.2485` | `5.2755` | no |
| Mean gain-matched spectrum RMSE, dB | `9.8880` | `8.9453` | no |
| P95 gain-matched spectrum RMSE, dB | `12.8617` | `11.8003` | no |
| Mean normalized waveform RMSE, dB | `4.8010` | `1.9557` | no |

The largest angle-plane failure is 100 degrees: mean level error is `6.6001
dB` and mean waveform NRMSE is `6.6632 dB`. Failures also vary materially by
distance and microphone, so one isolated bad row does not explain the result.

## Post-reject representation oracle

After the immutable rejection, a separate diagnostic reads the already opened
180 query references. It projects each true query complex STFT into the frozen
context rank-96 subspace, reconstructs through the production Rust metrics and
PCM cooker, and cannot train or select a candidate.

Two complete diagnostic runs produce the same normalized report
`b6dcc5fcea539d1859b05a4f83dc571c22198e8b41018bc3b651b8bf09b47ac2`
and prediction-grid hash
`5cf8279058d22592f455cc29ba486fb9804eaf2f58e921916a99a3eba6cc287c`.

| Diagnostic | Result |
| --- | ---: |
| Query projection Frobenius NRMSE | `0.27605` |
| Query total energy retained | `92.3797%` |
| Mean level error | `0.2460 dB` |
| P95 level error | `0.4132 dB` |
| Mean spectrum RMSE | `9.2714 dB` |
| P95 spectrum RMSE | `10.5629 dB` |
| Mean waveform NRMSE | `-14.5728 dB` |

The oracle beats every control on four endpoints, but its mean spectrum error
is worse than linear interpolation (`9.2714` versus `8.9453 dB`). Therefore
rank 96 is useful but is not an admissible query representation under the
frozen conjunctive rule. The exact reconstruction residual is concentrated in
`0–3 kHz` for this object (`0.8008` band NRMSE); the measured result does not
support blaming only high-frequency spatial aliasing.

The large improvement from the rejected candidate to the query projection
oracle still proves a separate interpolation failure: level, P95 level,
spectrum, P95 spectrum and waveform aggregates improve by `4.2721`, `10.8352`,
`0.6166`, `2.2987` and `19.3738 dB`, respectively. The bounded conclusion is
`RepresentationAndInterpolationBothLimited`.

## Primary-source research

The result agrees with known data and representation boundaries, but the
sources do not prove a replacement Next Engine model by themselves:

- [REALIMPACT](https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf)
  records 600 microphone positions per impact, but its authors explicitly
  describe 20-degree azimuth as a measurement-time compromise and show that a
  high-frequency radiation pattern exceeds the spatial Nyquist limit. They
  also report that the 12 cm vertical spacing is coarse for high-frequency
  detail. Thus the published grid cannot justify arbitrary continuous
  listener-field claims.
- [Learning Neural Acoustic Fields](https://arxiv.org/abs/2204.00628) learns a
  continuous emitter/listener impulse-response field with learned local
  geometric context. R2E contains neither scene/object geometry nor a learned
  local field; matching its coordinate-function label would not match its
  evidence boundary.
- [Measurement of Sound Fields Using Moving Microphones](https://arxiv.org/abs/1609.09390)
  states the general sampling constraint directly: acoustic fields obey
  spatial as well as temporal sampling limits, and volume reconstruction can
  require a very large number of known-position measurements.
- [Objects as Audio-Visual Modal Sound Fields](https://arxiv.org/html/2608.05145v2)
  is closer to the product problem. It factorizes impact sound into object-
  global modal frequencies/damping, contact-dependent modal gains and a
  residual; it conditions gains on geometry-aware visual features and reports
  novel-contact improvement on ObjectFolder Real and REALIMPACT. Its
  REALIMPACT experiment deliberately uses one nearest microphone and varies
  impact location, not listener radiation. The paper also reports collapse
  without modal initialization/warm-up and failure where sampling lacks
  representative local geometry.
- [Real Acoustic Fields](https://openaccess.thecvf.com/content/CVPR2024/papers/Chen_Real_Acoustic_Fields_An_Audio-Visual_Room_Acoustics_Dataset_and_Benchmark_CVPR_2024_paper.pdf)
  evaluates neural acoustic fields on densely measured real rooms and reports
  gains from geometry/visual cues and simulated pretraining followed by sparse
  real fine-tuning. This supports treating dense coverage and structural priors
  as first-class experimental axes, not replacing missing observations with a
  larger MLP.

## Competing hypotheses

| Hypothesis | Evidence | Conclusion |
| --- | --- | --- |
| The optimizer or objective still collapses | Context objective is `3.31e-15`, energy error is zero, repetitions are exact | Rejected at the context-fit boundary |
| Seven cosine features are a poor spatial inductive bias | Perfect context interpolation becomes worse than all controls on query; oracle is much better | Supported |
| Context rank 96 is sufficient for hidden listeners | Query projection NRMSE is `0.276`; oracle misses the mean-spectrum gate | Rejected under the frozen rule |
| A nearby larger network on the same split is the smallest next step | The query is open, the representation itself misses a gate, and published sampling is coarse | Rejected; would be query-informed architecture search |
| Listener radiation must precede useful object sound | SPEC-08 already owns environmental propagation; AV-MSF demonstrates contact-conditioned canonical-listener modeling | Rejected as an ordering requirement |
| A geometry-aware modal contact field is the next falsifiable neural task | It matches the product input, separates global modes from local gains and has real-dataset prior art | Selected for Roadmap V4, pending a new hash-closed corpus preflight |

## Roadmap consequence

1. Retire the R2 Green Goblet query as immutable negative/research evidence.
   It is never a method holdout for a new model family.
2. Audit and freeze a new internet-only multi-object/multi-impact corpus. The
   first task uses one declared canonical listener condition and force/peak
   alignment; no arbitrary radiation claim is made.
3. Before training a neural field, compare modal, codec and classical
   representations with query-seeing development oracles. A representation
   that cannot beat the frozen target baseline does not authorize a model.
4. Train an exact-object few-shot geometry-aware modal contact field, with
   global frequency/damping, contact-conditioned gains and an explicit
   residual. Require synthetic controls, micro-overfit, exact cooker and an
   unopened contact-position holdout.
5. Only after exact-object success, test cross-object pretraining and few-shot
   adaptation. Zero-shot material generalization remains optional.
6. Keep listener directivity as a separate later branch requiring denser
   published observations or validated BEM/FEM transfer, while the engine-native
   spatializer remains the fallback.
7. Freeze the independent validator, one-shot admission, formula registry and
   runtime consumer only after a generator claim exists.

The smallest next action is the Roadmap V4 data-source and split preflight, not
another model training run.
