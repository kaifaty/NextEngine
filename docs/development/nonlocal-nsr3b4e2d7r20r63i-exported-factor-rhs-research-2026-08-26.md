# NSR3-B4E2D7R20R63I exported-factor RHS research

Status: `FROZEN FOR IMPLEMENTATION`.

## Question

R63H proves that a wide-built factor may be exported to binary64 without
losing the immutable weak operator direction. Can that exported triangular
factor consume the one captured NNQP right-hand side while preserving the
original Gram residual and all 102 already certified solution signs?

This is the first RHS in the R63D--R63H lineage. It remains shadow-only.

## Factor solve identity

Let `C` be the tangent transpose and

```text
C P = Q R,
s = 1 / (1 + eta),
H = s C^T C = s P R^T R P^T.
```

For the captured system `H lambda=b`, define `z=P^T lambda`. Solve:

```text
R^T y = P^T b / s
R z   = y
lambda = P z.
```

The first solve is forward substitution and the second is backward
substitution. All 102 columns are retained.

## Three fixed consumption lanes

All lanes use the same R63H permutation, stored-binary64 operator and
wide-built factor:

1. `wide/wide`: retained binary128 `R`, binary128 RHS and fixed-order
   binary128 substitutions;
2. `export/wide`: exported binary64 `R` promoted exactly, binary128 RHS and
   the same binary128 substitutions;
3. `export/binary64`: exported binary64 `R`, RHS/scale rounded once to
   binary64 and strict fixed-order binary64 substitutions.

No lane uses `Q`; the Gram solve needs only `R`. No iterative refinement or
retry occurs.

Higham proves componentwise backward stability properties for triangular
substitution but also documents why triangular form alone does not guarantee
normwise forward accuracy. LAPACK's robust triangular solve adds scaling for
overflow safety. R63I freezes finite intermediates and an independent original
system certificate rather than inferring success from completion.

Sources:

- [Higham, The Accuracy of Solutions to Triangular Systems](https://nhigham.com/wp-content/uploads/2023/08/high89t.pdf)
- [LAPACK `xLATRS`](https://www.netlib.org/lapack/explore-html/de/d23/group__latrs.html)

## Original-system residual certificate

The candidate factor represents the stored rectangular operator, but the
acceptance system is the original captured R60/R63 Gram and RHS:

```text
H root   aa401d0191ad53b7caa7837c827913fa711e1fdc433bc200c020719d297fb09d
b root   64be49510b51f8e9898ed2d021fb2d41d98690cecebe5d95e0a108406cf192b1
```

For each lane independently recompute `r=b-H lambda` with the existing Dot2
enclosure. Reuse the already exact R60 approximate inverse `X`, whose left
defect satisfies

```text
rho = ||I-XH||inf < 1.
```

Then

```text
lambda* - lambda = X r + (I-XH)(lambda* - lambda)
```

and the a-posteriori infinity error is bounded by

```text
error <= ||X r||inf / (1-rho).
```

Every residual and `Xr` dot uses outward Dot2 bounds. The denominator is
rounded downward and the final radius upward. This is the standard verified
left-inverse residual principle.

Sources:

- [Rump, Verification methods: rigorous results using floating-point arithmetic](https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf)
- [Oishi and Rump, Fast verification of solutions of matrix equations](https://www.tuhh.de/ti3/paper/rump/OiRu02.pdf)

## Sign semantics

Recompute the immutable R60 depth-4 certificate. It must repeat:

```text
24 positive / 78 negative / 0 unresolved
rho  < 1
minimum separation about 32.4234
```

For each R63I candidate classify component `i` only when

```text
lambda_i - error > 0
```

or

```text
lambda_i + error < 0.
```

Require zero unresolved components and exact componentwise agreement with the
R60 certified sign vector, not merely the same positive/negative totals.

## Routes

- parent/control/factor/RHS/certificate/work failure:
  `EXPORTED_FACTOR_RHS_APPARATUS_REJECTED`;
- retained-wide lane fails original residual/sign certification:
  `WIDE_RHS_SIGN_CERTIFICATE_REJECTED`;
- retained-wide passes but export/wide fails:
  `EXPORTED_FACTOR_WIDE_CONSUMPTION_REJECTED`;
- both wide-consumption lanes pass but strict binary64 fails:
  `EXPORTED_FACTOR_BINARY64_CONSUMPTION_REJECTED`;
- all three pass:
  `EXPORTED_FACTOR_BINARY64_RHS_CANDIDATE`.

## Continuation and ceiling

If strict binary64 fails while export/wide passes, the factor may remain
binary64 but triangular dot/divide arithmetic needs a bounded wider or
compensated implementation. If all lanes pass, a later stage may replace the
single parent principal solution in a private replay and compare the following
NNQP decision; R63I itself cannot do so.

If retained-wide fails, stored-operator perturbation rather than consumption
arithmetic is the remaining boundary; research one original-residual
refinement before changing representation.

R63I performs exactly three two-triangular RHS solves. It performs no inverse,
iterative refinement, LSQR/LSMR/SVD, rank action, row drop, regularization,
center beyond the read-only R60 certificate, replacement, state update,
following transition, trajectory or timing. R64 and R65 remain blocked.
