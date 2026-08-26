# NSR3-B4E2D7R20R63R common-operator inverse research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63Q selects tangent Gram as the better complete finite representation of
`H*`, but no structured solution can yet be certified without a common-
operator inverse bound. Does the original binary128 tangent QR produce an
approximate inverse that is contractive on both sides of exact rational `H*`?

## Candidate verifier

Use the immutable R63G original binary128 factorization of permuted `T^T`:

```text
T^T P = Q R,
K = sigma T T^T,
K^-1 approximately P R^-1 R^-T P^T / sigma.
```

Apply the existing binary128 two-triangular-solve transaction to each canonical
RHS and assemble raw column inverse `Z`. Do not symmetrize or refine it.

Against R63Q's exact common numerator matrix `H_num` and positive denominator
`d`, compute exact dyadic residual numerators for both orientations:

```text
L_num = I d - Z H_num,
R_num = I d - H_num Z.
```

Then

```text
rho_left  = ||L_num||_inf / d,
rho_right = ||R_num||_inf / d.
```

Both comparisons with one are exact integer/exponent comparisons. Binary128
projections are report-only.

## Why both sides

The earlier R63B/R63C lineage demonstrated that one-sided contraction can hide
an orientation failure. A future residual-to-solution certificate needs the
left defect; the right defect additionally validates that the raw column
construction behaves as an inverse of the symmetric common operator rather
than an orientation-specific preconditioner.

## Interpretation

If both defects contract, R63S may finally apply the immutable RHS, use `Z` as
an offline common-operator verifier, and compare structured direct PCG
iterates against exact `H*` rather than stored dense `H`.

If left fails, the original tangent QR is not a usable solution verifier. If
only right fails, research explicit symmetric inverse formation or wider
triangular consumption before an RHS. Neither failure reopens dense `H` as
oracle.

## Scope ceiling

No immutable RHS, solution sign, PCG update, iterative refinement, sparse
realization, timing, state update, runtime/GPU or production authority is
admitted. R64 and R65 remain blocked.

## Result

The raw original-tangent inverse is contractive on both sides of exact `H*`:
`rho_left=9.06e-3` and `rho_right=3.57e-3`. Its exact left amplification bound
is `1.05019e33`; no symmetrization or refinement is used.

The selected route is `COMMON_OPERATOR_TWO_SIDED_CANDIDATE` at semantic
`b8783c53...f382`; stdout repeats byte-identically at `bdf1ae68...d5ca`.
See the
[evidence record](nonlocal-nsr3b4e2d7r20r63r-common-operator-inverse-evidence-2026-08-27.md).

Freeze an immutable-RHS direct-PCG replay with exact common residual and signs
next. Dense `H`, dense `X` and legacy signs cannot certify that stage.
