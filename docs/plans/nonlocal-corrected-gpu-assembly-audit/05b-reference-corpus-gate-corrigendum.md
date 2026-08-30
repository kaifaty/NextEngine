# NCGA5 revision 2 — reference corpus-gate corrigendum

| Field | Value |
| --- | --- |
| Status | `FROZEN / LOGICAL_CORRIGENDUM / NO_NUMERICAL_CHANGE / REPORT_ONLY` |
| Parent checkpoint | commit `a36bf482` |
| Revision-2 contract | SHA-256 `d2a98ff8b1bc51ff0503f37c5b50e06adc6db409841718e799ad2550305ac16b` |
| Pre-corrigendum rerun | byte-identical to revision-1 raw report, SHA-256 `87b7e43431a467afe304141c7152c0a1deb207c971bbde645731e877b34ea2ef` |

The revision-2 prose accidentally attaches a multi-iteration `RESIDUAL` gate
to `compressed_pair`. That is mathematically inconsistent with the frozen
retained counts: four outer trials and eight total Hessian products consist of
four one-iteration inner products plus four model-prediction products. The
published NSR1 evidence requires positive-curvature/residual CG, while NCGA5
requires at least one multi-iteration residual path **across the two-case
corpus**; `combined_tetrahedron` owns that discriminator.

The only correction is:

- compressed pair retains its exact `4/5/8`, raw-gradient, no-reject,
  no-radius-change, no-active-change, no-negative-curvature and `RESIDUAL`
  gates, without an impossible per-case `hvp>1` requirement;
- combined tetrahedron must contain at least one `RESIDUAL` record with more
  than one inner HVP;
- the strict-f32 corpus must still contain at least one multi-HVP residual
  record before either positive or floor classification.

No observed metric, threshold, fixture, arithmetic path, candidate gate,
single reference-arithmetic repair, execution order or claim ceiling changes.
This corrigendum is frozen before the predicate is edited and does not
authorize another apparatus repair.
