# Physical sound R2 phase-aligned listener field and failure research

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Scope | One Green Goblet fixed-impact listener field; external research only |
| Decision | `REJECT_TIME_DOMAIN_LISTENER_FIELD / DENSE_COMPLEX_FIELD_DATA_NEXT` |
| Public/runtime authority | None; SPEC-45 remains `Proposed` and authored clips remain mandatory fallback |

## Question

R2 V1 rejected a direct time-domain listener-latent field. This bounded
successor asked whether the missing variable was coordinate-derived air-path
delay rather than model capacity. The only changed hypothesis was explicit
propagation alignment before the unchanged field and inverse alignment before
PCM cooking. Rows, split, controls, five primary endpoints, optimizer budget,
cooker and conjunctive pass rule remained frozen.

Method holdout and admission shadow were not opened. Training did not read
query audio. Dataset payloads, checkpoints, generated WAVs, MLflow database and
artifacts remain outside Git.

## Frozen protocol and lineage

The phase-aligned protocol uses the published impact and listener coordinates,
Euclidean path length, fixed `343 m/s` propagation speed, and the nearest
context/query path as the relative-delay origin. It applies zero-padded RFFT
phase rotation, trains the same two-layer `tanh` MLP at ranks 4 and 7 for 8,000
fixed-seed Adam steps in CPU float64, restores the query delay and cooks through
the same PCM16 path.

| Artifact | SHA-256 or exact value |
| --- | --- |
| Training manifest | `02a4b0b95713234b7d366bb839b526f13db6fa339f572eb10f10606975c14ffe` |
| Preflight report | `d744ad6658325348ff85a91c63c7ba4accabab350f03fbecbfeb5eeed86a688e` |
| Training report A/B | `e7bc058966ca9c8100f97d0069fd482e8d43a3d2a542cec713cce22d58f4dff1` |
| Evaluation manifest | `f3b6e00e0becb9d57fc261359cf72c8d3943a57c088a76f6c27d976be374a274` |
| Evaluation report A/B | `521c9a826f8d098d8d8902fa0c0e0c068133681b567b9044a3e9f30d704ce612` |
| Phase trainer source | `2e035766f9c82a59b5c6f8852921ace4854524827a3eba5fad8ca575d0f0e57f` |
| Phase evaluator source | `d276176d5150c8056a49ba557ec051cae214b499b2f6449655448645ee57cf96` |
| Diagnostic source | `6290e0df9854c9c697e43c41f06d3b11404de2c078a160747094fa99163dc70e` |
| Diagnostic report A/B | `98dec68ab3b915154c8d54cd8f672d47c558255a9eea1efd33fdbecfe45ec652` |

The preflight measured a maximum relative delay of
`0.002459718918280059 s` (`118.0665` samples), selected a 121-sample guard and
262,144-point FFT, and round-tripped at `-55.3004 dB` waveform NRMSE with
`2.67905e-05` maximum absolute error. Query and sealed-role audio reads were
zero.

Training A/B produced byte-identical checkpoints, predictions and reports. The
rank-4 and rank-7 weights hash to
`bab18478c517ac2a55cf5aa07b1a67dd9796b1b654183a8d9ced346d37c26728`
and `98a32ab6360c7c674c85cde05069c7a7c5a7deb0bf6748fb248776b1a5db69ea`.
Their context waveform NRMSE is respectively `-19.7132 dB` and `-69.1540 dB`.
Evaluation A/B is byte-identical and returns `RejectListenerField` with no
selected candidate.

## Frozen evaluation result

The table gives candidate values; `PASS` means strictly better than both frozen
nearest and linear controls on that endpoint.

| Primary endpoint | Rank 4 | Result | Rank 7 | Result |
| --- | ---: | --- | ---: | --- |
| Mean absolute level error, dB | 2.1981 | PASS | 1.7803 | PASS |
| P95 absolute level error, dB | 3.6878 | PASS | 2.5787 | PASS |
| Mean gain-matched spectrum RMSE, dB | 9.0566 | PASS | 9.4477 | FAIL |
| P95 gain-matched spectrum RMSE, dB | 11.9086 | FAIL | 11.6927 | FAIL |
| Mean waveform NRMSE, dB | 1.5599 | PASS | 1.8800 | PASS |

Rank 4 improves four of five endpoints, but the preregistered rule is
conjunctive. Microphone 07 drives its P95 spectrum failure. Rank 7 improves
three endpoints. Neither near-miss changes the frozen threshold or creates a
quality claim.

## Failure discrimination

After two coherent listener-field failures, the repository persistent-problem
rule required research before another model variant. The diagnostic was run
twice and produced byte-identical final reports and query-informed oracle WAVs.
The oracle is not a deployable model: it reads query truth only to test whether
the representation can express the held listeners.

| Hypothesis | Evidence for | Evidence against | Conclusion |
| --- | --- | --- | --- |
| Insufficient optimizer or MLP capacity | Rank 4 leaves context error | Rank 7 fits context at `-69.1540 dB` yet fails two spectrum endpoints | Rejected as the primary cause |
| Missing air-path phase | Phase alignment moves rank 4 from one to four passing endpoints | It still fails P95 spectrum under the unchanged rule | Partially supported, insufficient |
| Sparse-line spatial aliasing is the sole cause | Context spacing `0.26 m` implies a `659.62 Hz` spatial Nyquist; 24.53% mean and 44.23% mic-07 energy lie above it | Phase-vs-linear spectrum error is not selectively worse above Nyquist: it is slightly worse below, slightly better from 1–4x, and nearly equal above 4x | Not supported as the sole cause |
| The context time-domain latent span is structurally inadequate | Context ranks are 7; all-listener centered rank is 14. Even a query-informed rank-4/7 subspace oracle passes only waveform NRMSE and badly fails frozen level/spectrum aggregates | The oracle deliberately has access unavailable to a real predictor, so failure is a strong representation test | Supported |
| More data plus a complex/time-frequency field is the smallest credible successor | Published prior art succeeds with dense spatial sampling and complex or STFT fields; REALIMPACT exposes a 600-position fixed-impact semicylinder | It is not yet measured under the Next Engine split and frozen controls | Next falsifiable path |

The query-informed rank-4 oracle records mean/P95 level error
`6.8668/13.4966 dB` and mean/P95 spectrum error `9.5887/14.4947 dB`.
Rank 7 records `4.5790/9.2120 dB` and `10.6479/16.9998 dB`. Both pass only
waveform NRMSE. This rejects another nearby rank/width/epoch/threshold sweep:
the opened 15-row time-domain family does not preserve the held-query
level/spectral distribution even under an unrealistically informed projection.

## External research

- [RealImpact](https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf)
  publishes 600 microphone positions per struck vertex on a semicylinder, five
  vertices per object, coordinates, impact force, material and mesh metadata.
  The current 15-row vertical line therefore leaves substantial published
  fixed-impact spatial evidence unused.
- [Learning Neural Acoustic Fields](https://arxiv.org/html/2204.00628v2)
  reports that naive direct waveform prediction performs poorly and instead
  models STFT log magnitude plus unwrapped instantaneous-frequency phase over
  listener/emitter position, time and frequency with dense spatial data.
- [Compact Acoustics-Informed Neural Network](https://arxiv.org/html/2402.08904v1)
  predicts real and imaginary complex pressure per frequency and adds a
  Helmholtz-equation loss at measured and interior points, supporting a field
  representation with explicit physical regularization rather than only a
  sample-space loss.
- [Deep Sound Field Reconstruction](https://arxiv.org/html/2102.06455v1)
  shows that sparse microphone reconstruction becomes difficult as frequency
  rises and that denser measurement grids materially improve the task.
- [Spatial sampling and aliasing analysis](https://www.wirelesslab.ca/File/pdf_files/journals/J030.pdf)
  supplies the sampling-theory context. It does not override the local band
  diagnostic, which failed to identify aliasing as the sole current cause.

These papers support the next experiment design; they do not prove that a
specific model will pass Next Engine's frozen endpoints.

## Decision and next experiment

Retire direct and phase-aligned time-domain listener-latent fields for this
opened slice. Do not tune MLP width, rank, epochs, seed, phase speed or frozen
thresholds. Preserve both failures as negative knowledge.

The next commit boundary is `R2B_DENSE_COMPLEX_FIELD_DATA_PREFLIGHT`:

1. Acquire and project one Green Goblet fixed-impact block over the full
   published 600-position semicylinder using the existing internet-only,
   hash-closed source boundary.
2. Split by complete gantry columns or another preregistered spatial group,
   never by interleaved individual microphones, so nearby samples cannot leak
   across context and query.
3. Freeze a complex time-frequency target before optimization: either complex
   STFT pressure or log magnitude plus continuous phase/instantaneous
   frequency. Record window, hop, frequency band, scaling and inverse-cook
   error.
4. Export nearest/linear and simple complex-field controls on the grouped
   split. Quantify coordinate coverage, spatial sampling limits and query
   isolation before training.
5. Preregister an exterior-air physics regularizer, such as Helmholtz residual
   at bounded frequencies, plus a no-physics ablation. Geometry and source
   coordinates must come from published data; missing axes remain missing.
6. Keep method holdout and admission shadow sealed. Do not start N0.4,
   validator admission, runtime integration or public schema work until this
   R2 representation beats both frozen controls on every primary aggregate.

This is a data/representation preflight, not approval for a larger optimizer.
Its first useful failure result is `DATA_INSUFFICIENT` or
`REJECT_COMPLEX_FIELD_REPRESENTATION`, not another threshold revision.
