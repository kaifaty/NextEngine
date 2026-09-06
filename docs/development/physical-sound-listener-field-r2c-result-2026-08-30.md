# Physical sound R2C dense complex listener-field result

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Scope | One Green Goblet fixed-impact 600-position listener field; external research only |
| Decision | `REJECT_COMPLEX_LISTENER_FIELD / REDESIGN_OBJECTIVE_BEFORE_ANOTHER_CANDIDATE` |
| Public/runtime authority | None; SPEC-45 remains `Proposed`, no candidate is admitted, and authored clips remain mandatory |

## Question and answer

R2C asked whether one dense coordinate/time/frequency complex-pressure network
could beat the three frozen R2B interpolation controls, and whether a bounded
Helmholtz residual improved the same network under an otherwise identical
protocol.

No. Both candidates train and repeat byte-identically, but both collapse toward
a near-silent prediction. Each passes only the normalized-waveform endpoint
and fails level plus spectral reconstruction by a large margin. The Helmholtz
term falls during training without changing that outcome. The immutable
decision is `RejectComplexListenerField`; no checkpoint is selected.

## Frozen protocol and isolation

The manifest freezes one `341,410`-parameter separable SIREN, `8,000` AdamW
steps, float32 deterministic CUDA execution, context-only normalization and
two candidates:

1. `dense_complex_field_data_only_v1`, Helmholtz weight `0`;
2. `dense_complex_field_helmholtz_v1`, Helmholtz weight `0.0001` over
   `93.75–12,000 Hz` with deterministic midpoint collocation.

Both use the same `420 context / 180 query` split, complex STFT representation,
seed, architecture, optimizer budget, checkpoint policy and inverse cook. Query
audio contributes zero bytes to preprocessing, fitting or checkpoint selection.
Method holdout and admission shadow remain sealed.

The observed environment is Python `3.12.13`, NumPy `2.5.2`, PyTorch
`2.13.0+cu130`, MLflow `3.15.2` and an NVIDIA RTX 3080 with driver `610.43.02`.
The deterministic GPU preflight produced identical weights and loss in two
independent processes before the protocol was frozen.

## Reproducible lineage

| Artifact | SHA-256 or exact value |
| --- | --- |
| Training manifest | `933cefa213f27905bb9852b3883c448e268974b689790c621df986592e75ca1b` |
| Context preflight report A/B | `8ed8d0764003ed21f2f2a75be3dc035f9d313d2e476c9adb6aaac707e1f1b233` |
| Context complex-STFT cache | `842a7c922e08abe20c8498eb8b44849781dbf11a94380737e7daa6ce6d5cc787` |
| Data-only training report A/B | `12f89e3addd26ea0ddb9f611797d0b3d81529920658d6200cfd53d80789163bd` |
| Data-only weights A/B | `21bc5dfa9401bb09b4c3dbb89b5c6c1c8a157bc58d0a101c7919d7ff55fc7c39` |
| Helmholtz training report A/B | `3833c2a3ffeffdc2bb68c4ee660da9fc9d5e2b095d41efbc584b2750543e4a8b` |
| Helmholtz weights A/B | `8a01ca7433b48505dd65ab6f858faf04856d1f2917534059b3fe9c7bc8584c46` |
| Evaluation manifest | `f0276aa1c409f202d12fd31eac00a7099df2bbd1eb7073e4c4ab129a83f93e1a` |
| One-shot evaluation report | `63c2eab6c244918595fd6e37828292be41ab87dff4fc1091b1650a72858ebb1b` |
| Failure diagnostic report A/B | `1263e02f09db167dd52023a2a6c42132e15461020555b68f5535cf93cc503a8f` |

For each candidate, the two non-MLflow output trees compare byte-identically.
Each run emits one final checkpoint and 180 cooked query predictions with zero
cook failures. MLflow stores params, metrics and lineage externally; datasets,
features, weights, generated WAVs and tracking state do not enter Git.

## Frozen evaluation

Lower is better. A candidate must be strictly better than all three controls
on every endpoint; neither candidate is close to that boundary.

| Candidate | Mean level | P95 level | Mean spectrum | P95 spectrum | Mean waveform NRMSE |
| --- | ---: | ---: | ---: | ---: | ---: |
| Best frozen control per endpoint | `1.8584 dB` | `5.2755 dB` | `8.9453 dB` | `11.8003 dB` | `1.9557 dB` |
| Data only | `53.3422 dB` | `68.9146 dB` | `27.3694 dB` | `32.5781 dB` | `0.000074 dB` |
| Helmholtz | `53.3574 dB` | `68.9068 dB` | `27.3827 dB` | `32.7832 dB` | `0.000076 dB` |

Near-zero normalized-waveform NRMSE in this table is not evidence of a good
match. Together with `53 dB` mean level error it identifies an almost-zero
prediction: the error signal has approximately the same energy as the
reference.

## Bounded failure research

After the rejection, a context-only diagnostic ran twice without optimizer,
query audio, method holdout or admission-shadow access. It compared the trained
models against zero/global-mean predictors and an exact linear rank oracle.

| Hypothesis | Evidence | Conclusion |
| --- | --- | --- |
| Helmholtz regularization caused the failure | Data-only and Helmholtz results are nearly identical; Helmholtz residual falls to `0.0200` without restoring signal level | Rejected as primary cause |
| Rank `96` is too small | A context-only rank-96 oracle retains `99.6396%` total energy and has best linear Frobenius NRMSE `0.0600` | Not supported as primary cause |
| Loss sampling and clipped optimization favor silence | Data-only full-context objective is `1.0498×` the zero predictor; all `81/81` logged steps exceed the gradient clip before clipping; query mean level error is `53.34 dB` | Supported |

The zero-predictor objective is `1.203486`; the global-mean-field objective is
`1.177082`; the trained data-only and Helmholtz full-context objectives are
`1.263386` and `1.272429`. The models therefore do not approach even trivial
context controls. This failure occurs before held-listener generalization and
does not justify a width, step, seed, rank or physics-weight grid.

## Decision and roadmap consequence

Retire this joint separable-SIREN plus sampled equal-L1 objective revision. Do
not retry it with a larger model or another Helmholtz coefficient on the opened
query split.

The next revision must remain context-only until it passes a trainability gate:

1. prove the objective and cooker on one-row and small-block micro-overfit
   controls, including absolute energy, spectrum and waveform reconstruction;
2. compare every fit against zero, global-mean and context-only low-rank
   oracles before any query evaluation;
3. preserve energy explicitly in sampling/loss and report clipping saturation;
4. learn spatial coefficients over a frozen context-only low-rank complex
   basis before attempting another joint coordinate/time/frequency field;
5. freeze the successful context protocol and only then authorize one grouped
   query candidate against the unchanged three controls and five endpoints.

This rebaseline creates no learned quality, material identity, exact-object
impact-axis, validator, admission, public schema or runtime authority.
