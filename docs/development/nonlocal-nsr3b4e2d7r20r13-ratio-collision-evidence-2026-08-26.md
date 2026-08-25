# NSR3-B4E2D7R20R13 ratio-collision evidence

Status: `PASS / V3_RATIO_SUBPREDICATE_IDENTIFIED`.

Implementation `b164e4cd` preserves both R12 case and final-step roots exactly.
Semantic result:

```text
0ec0fa4a48138ca63abdde07e0c3997051982aa5833721f8b8a8e692a24eb6eb
```

Both failures classify as `RATIO_ORDER_AMBIGUOUS`:

| case | best row | best ratio ± bound | competitor row | competitor ratio ± bound |
|---|---:|---:|---:|---:|
| corner | 33 | `0.313449 ± 0.027795` | 41 | `0.314525 ± 0.027884` |
| shear | 45 | `0.365718 ± 0.106316` | 37 | `0.411097 ± 0.002266` |

Every central denominator is strictly positive and every central ratio lies in
`[0,1]`. Q1 and Q2 are falsified; Q3 is supported on both counterexamples.
The exact original decisions and roots are unchanged, falsifying the observer
effect Q4.

The corner central-ratio gap is only about `1.08e-3`, versus about `5.57e-2`
combined half-width. In shear the best row's small denominator (`3.117e-2`)
amplifies its bound to `1.063e-1`, while the competitor denominator is
`1.511` and its bound only `2.266e-3`.

## Prior-art boundary

The canonical Lawson-Hanson NNLS implementation in
[Netlib](https://www.netlib.org/lawson-hanson/all) computes the minimum central
ratio, moves that selected coefficient out of the passive set, then moves any
remaining nonpositive coefficients out as round-off cleanup. That source was
inspected directly on 2026-08-26. It establishes the classical active-set
mechanism, but supplies no rigorous interval-order certificate; copying its
central-value choice would weaken this campaign's finite-precision claim.

No solver decision changed and no repair is authorized. The next discriminator
must separate candidate-solve uncertainty from accumulated current-direction
uncertainty.

