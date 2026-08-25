# NSR3-B4E2D7R19R65 model-aware projected-path probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / GLOBALIZATION VALID / REFERENCE RETAINED`.

Implementation commit: `8f4cfcd6`.

The v5 probe returns `PASS` with route
`MODEL_AWARE_PROJECTED_PATH_REFERENCE_RETAINED`. Exact parent, curvature,
recurrence, KKT, joint reprojection, model agreement, work and rollback gates
pass. The result has no nonlinear, timing, runtime or production authority.

## Result

| outer | face | trials / model projections | model rejects | alpha | applied zeros | maximum raw | projected gradient | committed model reduction |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1,074 | 1 / 1 | 0 | `1` | 240 | `3.1108357693353815e-7` | `2.6650423810233326e-6` | `1.7759100325346879e-13` |
| 2 | 2,111 | 2 / 2 | 1 | `1/2` | 358 | `1.8287926454178031e-7` | `2.2982557705926202e-6` | `3.3975357754505358e-13` |
| 4 | 3,368 | 2 / 1 | 0 | `1/2` | 651 | `8.7500654195495389e-8` | `1.7747431964890482e-6` | `1.4479903028234913e-14` |
| 8 | 2,407 | 3 / 1 | 0 | `1/4` | 207 | `1.6711649021339177e-8` | `2.5683561103484792e-7` | `1.4399770971010729e-14` |
| 16 | 2,257 | 1 / 1 | 0 | `1` | 38 | `1.8999700250730942e-9` | `2.7056703324874309e-8` | `1.8106755191894386e-14` |

Outer 2 is the discriminating case: v4 accepted `alpha=1` and obtained
negative model reduction. v5 rejects that candidate, accepts `alpha=1/2` and
obtains positive reduction `3.3975357754505358e-13`. Candidate and freshly
committed reductions are bit-exact at every checkpoint; the agreement gaps
are zero against bounds `3.05e-22..4.16e-22`.

Stationarity reaches `6.34e-24`; recurrence gaps stay
`9.81e-21..2.25e-20` below `6.92e-16..2.07e-15` bounds. Semantic result
SHA-256 is
`d64dd74af2c314e22eca11f04ab532e5fa45a5e3d5e14955c79977fcd0ee8b69`.

## Work and comparison

```text
face PCG products                    240
dyadic candidate transposes           37
candidate joint projections           17
dual / model rejections            20 / 1
accepted lines                         16
predicted / applied zeros     5,591 / 4,324
A^T / A calls                   309 / 272
structural terms              350,496,935
FISTA structural terms        596,971,680
dense Gram storage                      0
```

v5 uses `41.29%` less sparse structural work than FISTA and proves the model
safeguard without adding sparse operator calls beyond candidate search. It
does not dominate FISTA: final maximum raw is `1.61x` higher and projected
gradient is `2.84x` higher. It is also `1.29x/1.24x` worse than v4's terminal
raw/projected-gradient pair despite applying 259 more zero events.

## Classification

Model-aware acceptance is retained as a correctness mechanism. The remaining
loss is no longer a false positive model step. The largest-first policy stops
at the first safe candidate and provides no evidence that this candidate is
best for the composed objective. Research a fixed exhaustive dyadic search
with a no-PCG composed baseline before changing PCG depth or tolerances.
