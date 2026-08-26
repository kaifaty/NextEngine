# NSR3-B4E2D7R20R42 centered-refinement trajectory evidence

Status: `PASS / CENTERED_REFINEMENT_LATER_BOUNDARY / RATIO BUDGET EXPOSED`.

Implementation `3e021608` emits reproducible semantic:

```text
e435daf5e3ba43a6a51d4a014f9ed2b6d10dee4096b6f4ac5e9a942b53cd9a41
```

The default-null hook matches the exact frozen target once and consumes one
practical certificate. Its root is `6c8c6c97...393f`; `rho`, radius and signs
reproduce R41 at `0.131768`, `0.142140` and `24/41/0`. Converting the
componentwise center-addition bounds to the existing uniform-error API yields
the slightly wider `0.1421399732012798586`, still with minimum separation
`1054.2439`.

The first apparatus run incorrectly required this uniform minimum separation
to equal R41's componentwise value bit for bit. It was rejected before any
replacement and has no scientific credit. The corrected predicate preserves
exact `rho`, radius and sign equality and requires the conservative uniform
separation to remain strictly positive.

With the one replacement, the original `VERIFIED_INVERSE_AUDIT_REJECTED`
disappears. The unchanged NNQP consumes the 41 negative components and reaches
the next boundary `BOUNDARY_RATIO_UNRESOLVED` / `RATIO_ORDER_AMBIGUOUS` in the
same eighth outer step:

| field | best row 61 | competitor row 13 |
|---|---:|---:|
| nominal ratio | `1.5175242316e-19` | `3.2782163987e-19` |
| issued bound | `1.6565648546e-19` | `1.6565648546e-19` |
| rounding term | `9.8607613153e-32` | `9.8607613153e-32` |
| input term | `1.6565648546e-19` | `1.6565648546e-19` |

The nominal gap is about `1.761e-19`, while the two bounds sum to about
`3.313e-19`. Rounding is negligible; vector uncertainty causes the overlap.
The candidate error `0.14214` dominates the current error `1.71568e-4` by
roughly 829x. The already predeclared R41 depth-eight checkpoint reaches the
arithmetic floor (`4.5753e-14` radius), so substituting that certificate should
leave current error dominant and reduce each ratio input bound to order
`2e-22`, well below the observed gap.

Two corrected R42 runs are exact. R41/R40/R39 reproduce with the hook null.
R42 adds no trial, factorization or inverse column and does not touch
counterflow or production state. It proves depth four fixes sign selection but
is intentionally insufficient for the following ratio ordering.
