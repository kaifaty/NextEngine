# NSR3-B4E2D7R20R38 Dot2 inverse-certificate evidence

Status: `PASS / DOT2_INVERSE_CERTIFICATE_CANDIDATE`.

Implementation `3260382a` emits reproducible semantic:

```text
6898dcbe1b610a9a4cf38276d5af4e5f79ee63fe095315ddafb86b6deaa706ae
```

All R37 case, matrix, factor, inverse and exact-oracle roots reproduce. The
ORO binary128 compensated audit reports:

| field | value |
|---|---:|
| residual dots / contained | `4225 / 4225` |
| underflow events | `0` |
| exact worst row / Dot2 worst row | `25 / 25` |
| exact outward `rho` | `4.443635206380081269321826155479622931e-3` |
| Dot2 `rho_bound` | `4.443635206380081269321826155507460894e-3` |
| maximum entry bound | `1.312742758834111e-33` |
| maximum exact Dot2 error | `2.638019047622844e-37` |
| maximum exact-error / issued-bound | `3.216350150067308e-4` |
| compensated residual root | `d388080b...30f8` |
| certificate root | `cd14942d...3181` |

The compensated bound is contractive and only slightly wider than the exact
dyadic oracle. `Dot2` therefore replaces the exact big-integer computation as
a plausible same-binary128 residual-verification mechanism for this captured
inverse candidate. It is not yet installed in `al_r20_verified_inverse`.

Two R38 repeats are exact and R37 remains `e9e61c01...598ff`. R38 executes no
new factorization/inverse column, no original-solution refinement and no solver
decision or state update. The result authorizes only a separate passive-
solution residual/error audit.
