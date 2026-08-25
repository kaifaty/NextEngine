# NSR3-B4E2D7R19R65 batch projected-Newton probe evidence

Date: `2026-08-25`

Status: `EXPLORATORY PASS / CHORD LINE REJECTED / PROJECTED PATH REQUIRED`.

Implementation commit: `7d2f5f1c`.

The v3 probe returns `PASS` with route
`BATCH_PROJECTED_NEWTON_REFERENCE_RETAINED`. Parent, sparse owner, KKT,
reprojection, positive model reduction and rollback all pass; no runtime or
timing authority is admitted.

## Result

| outer | face | predicted zeros | applied zeros | line alpha | maximum raw | projected gradient |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 1,074 | 240 | 0 | `0.83148866014173262` | `3.0018814415250228e-7` | `3.6829957664900634e-6` |
| 2 | 2,131 | 492 | 0 | `0.66905328613492943` | `2.0347295783569434e-7` | `3.3259495615598730e-6` |
| 4 | 3,584 | 1,020 | 0 | `0.35265573743004247` | `1.0522813443084010e-7` | `2.5982400009431430e-6` |
| 8 | 2,536 | 446 | 0 | `0.15000308879273586` | `2.2413842695953101e-8` | `3.5746419858682925e-7` |
| 16 | 2,194 | 35 | 0 | `0.73033010304137869` | `3.3373916141243756e-9` | `3.8967925169231744e-8` |

All 240 face-PCG and 16 batch-direction products have positive curvature and
descent. Stationarity reaches `6.03e-24`; recurrence gap `2.27e-20` remains
far below its `2.04e-15` bound. Semantic result SHA-256:
`0b36aacfdcd7e6c5a7b52a2960997dc1b1dbeb572603f05d942605619a23177d`.

## Work and comparison

```text
face PCG products                    240
batch direction products              16
A^T / A calls                    288 / 288
structural terms              349,366,230
FISTA structural terms        596,971,680
dense Gram storage                       0
```

v3 uses `41.48%` less structural work than FISTA. It improves the restart-v2
terminal raw by `4.83x` and projected gradient by `3.33x`, proving that the
batch Newton direction carries useful curvature information. It still loses
to FISTA by `2.82x` raw and `4.09x` projected gradient.

## Why no predicted zero is applied

The implementation first forms endpoint `P(lambda+d)` and then searches the
straight chord from `lambda` to that endpoint. Every exact quadratic minimizer
has `alpha<1`, so a coordinate intended to be zero at the endpoint remains
positive on the chord. The algorithm predicts 6,883 zero events across the
run and applies none.

Bertsekas' method instead follows the projected path
`P(lambda+alpha*d)`, where a coordinate reaches zero at its own breakpoint.
That path, not more PCG depth or another chord formula, is the next research
target.
