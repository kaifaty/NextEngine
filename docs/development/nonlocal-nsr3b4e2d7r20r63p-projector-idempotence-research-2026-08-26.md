# NSR3-B4E2D7R20R63P finite-projector idempotence research

Status: `FROZEN / IMPLEMENTATION_NEXT`.

## Question

R63O proves that the stored dense operator `H` and the direct tangent Gram
operator `K=sigma T T^T` are close entrywise but have materially different
inverse semantics. Is this difference caused by treating the finite active-ball
projector as idempotent when its stored norm denominator is not the exact real
norm of its stored vector?

## Derived structural hypothesis

On free coordinates the frozen projector derivative is

```text
P_d = I - y y^T / d,
d   = stored projector.free_norm_squared,
s   = exact-real y^T y over the stored binary128 vector,
sigma = stored 1 / (1 + eta).
```

`P_d` is symmetric, but it is idempotent only if `s=d`. Exact algebra gives

```text
P_d^2 - P_d = ((s-d) / d^2) y y^T.
```

For the frozen source-row operator `A`, the intended source/JVP/source model is

```text
H* = sigma A P_d A^T,
```

whereas an exact tangent Gram represents

```text
K* = sigma A P_d^2 A^T
   = H* + sigma ((s-d)/d^2) (A y)(A y)^T.
```

Thus the suspected defect is not an empirical patch: it is an analytically
forced rank-one term produced by applying the same finite projector twice.

## Smallest decisive experiment

Use the immutable R63E weak witness `alpha` and reconstruct its source vector
`z=A^T alpha` with exact dyadic additions and products. Evaluate, without
floating division, the exact rational numerators of

```text
h* = sigma (z_f^T z_f - (y^T z_f)^2 / d),
k* = h* + sigma (s-d) (y^T z_f)^2 / d^2.
```

Independently evaluate exact dyadic quadratic forms of:

1. the stored dense matrix `alpha^T H alpha`;
2. the stored tangent coefficients `sigma ||T^T alpha||^2`.

All semantic comparisons use integer cross-products, never a decimal
tolerance. The discriminator asks whether stored dense `H` is strictly closer
to `h*`, stored tangent Gram is strictly closer to `k*`, `s-d` is exactly
nonzero, and the ideal difference is exactly the derived rank-one term.

This witness-only experiment is intentionally before another RHS solve. It can
identify the operator mismatch causally at much lower complexity than an
arbitrary-precision `102 x 102` inverse.

## Candidate interpretation

If the hypothesis passes, the next matrix-free candidate is not
`sigma T(T^T p)`. It is one application of the actual derivative:

```text
u = A^T p
v = DPi(u) = sigma P_d u
q = A v
```

R63Q must then establish exhaustive canonical correspondence and replay the
frozen PCG lanes. A rank-one formula may remain an audit identity, but it is
not authorized as a runtime correction in R63P.

If the hypothesis fails, preserve R63O and proceed to a broader independent
operator oracle; do not inflate the transport bound or add PCG iterations.

## Scope ceiling

No RHS solve, PCG, sparse realization, precision change, state update, timing,
runtime/GPU or production authority is admitted. R64 and R65 remain blocked.

