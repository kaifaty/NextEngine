# NSR3-B4E2D7R20R40 directional Krawczyk evidence

Status: `PASS / DIRECTIONAL_KRAWCZYK_SIGN_UNRESOLVED / 59 OF 65 RESOLVED`.

Implementation `c03b5116` emits reproducible semantic:

```text
a999b65a7f3d4817d0aa58b1195942e63df23a2197f564d785701c6fb3dd6f7e
```

R39 and every captured parent root reproduce exactly. The exact and Dot2 left
defect roots are `5f3c0e06...73f6` and `61bf229b...aa50`; all `4225/4225`
entries are contained without underflow.

| field | value |
|---|---:|
| exact outward `rho(I-XA)` | `0.1317676975770132537272002207351788` |
| Dot2 left-defect bound | `0.1317676975770132537272002207352104` |
| worst left-defect row | `27` |
| exact `||Xr||inf` | `2.4991073128988463e14` at row `64` |
| certified directional error | `2.8783855494947104e14` |
| R39/directional improvement | `35.9239577823260x` |
| signs | `24 positive / 35 negative / 6 unresolved` |
| directional root | `9d02237b...e99` |

All 65 directional products contain the independent exact dyadic correction.
The largest actual directional-product error is `6.264e-19`, only
`1.605e-5` of its issued bound. Residual-input uncertainty (`3.966e-14`
maximum) dominates the compensated Dot2 term (`4.813e-20` maximum), but both
are negligible beside the physical correction magnitude.

The left defect is safely contractive, so failure is not caused by a bad
right-to-left certificate transfer. Retaining residual direction improves the
R39 enclosure about 36-fold, but the actual residual still excites a correction
of order `1e14`; the same six signs remain unresolved. Repeating R40 is
bit-exact and R39 remains at semantic `83e1f738...1bac`.

R40 applies no correction and executes no additional factorization, inverse
column, solve, NNQP decision, trial or state update. A centered fixed-point
enclosure is required next: shadow-build a refined center, then independently
certify its remaining fixed-point residual. Retrying another scalar norm around
the original represented solution would repeat the disproved approach.
