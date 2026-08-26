# NSR3-B4E2D7R20R63N matrix-free correspondence research

Status: `FROZEN FOR IMPLEMENTATION`.

## Question

R63M selects PCG for the captured RHS, but its algorithm applies the already
assembled dense `102 x 102` Gram matrix `H`. Can the same represented operator
be applied directly through the immutable `102 x 315` tangent rows while
preserving finite-arithmetic containment and both eight-update PCG sign
certificates?

This is an implementation-correspondence stage, not a throughput claim.

## Model identity

Let `a_i` be captured active source row `i`. On the frozen projector face,
let `P_T` be the symmetric tangent projector after clamped coordinates and the
active-ball radial direction are removed, and let

```text
T_i = P_T a_i
sigma = 1 / (1 + eta).
```

The projector JVP is `D Pi = sigma P_T`. Because `P_T` is symmetric and
idempotent on this face,

```text
H_ij = a_i^T D Pi a_j
     = sigma (P_T a_i)^T (P_T a_j)
     = sigma T_i^T T_j.
```

Therefore the direct product is

```text
u = T^T p
q = sigma T u.
```

R63E already binds the captured dense matrix to the direct projector JVP;
R63F binds the tangent-row Gram identity and exact represented rank. R63N must
still prove the executed two-pass rounding correspondence. The real-arithmetic
identity alone cannot select a finite-profile route.

## Frozen correspondence ladder

1. Reconstruct exact R63M, including both parent lane roots.
2. Reconstruct `T` and require its immutable value root
   `114a73ea...73f1`, dimensions `102 x 315` and 17,748 nonzero stored values.
3. Apply dense and direct rectangular products to all 102 canonical basis
   vectors. Require interval overlap for every one of the 10,404 output
   components and bind the first/worst separation.
4. Replay both R63M starts and factors for exactly eight PCG updates, replacing
   only initial-residual and `H p` products with the direct rectangular
   product.
5. At all 18 algorithmic product sites compare the direct product with the
   dense product under independent Dot2 bounds. The dense result cannot drive
   the recurrence.
6. Independently certify every matrix-free PCG iterate against original dense
   `H,b,X` and all 102 R60 signs.

The matrix-free center path uses binary128 tangent values reconstructed from
the original binary64 source operator and projector state. It must not use the
factor's once-exported binary64 `R` input as the physical operator: that Gram
is the preconditioner `B`, not original `H`.

## Error transport

For each product, use fixed increasing-index binary128 Dot2 reductions:

```text
(u_hat[k], du[k]) = Dot2(T[:,k], p)
(w_hat[i], dw0[i]) = Dot2(T[i,:], u_hat)
dw[i] = dw0[i] + sum_k |T[i,k]| du[k]
q_hat[i] = sigma * w_hat[i]
```

Round every propagated sum upward with the existing binary128 bound helpers.
Include the final scale-product rounding. A component corresponds only when
the direct interval overlaps the independent dense-Dot2 interval. No decimal
tolerance, norm-only substitute or a-posteriori bound fitting is permitted.

## Work accounting

One full rectangular product has 315 inner plus 102 outer Dot2 reductions and
64,260 scalar terms before zero elision. Per PCG lane the frozen nine products
therefore have 3,753 Dot2 reductions and 578,340 terms. Publish separately:

- canonical-basis correspondence work;
- 18 algorithmic rectangular products across two lanes;
- dense comparison and certificate work, excluded from the candidate path;
- the 17,748-nonzero structural lower work available to a later sparse stage.

This dense rectangular reference has more arithmetic than applying the
captured dense `H` for dimension 102. A pass authorizes only sparse/direct
operator research; it is not a performance win.

## Routes

- control, parent, identity, product-bound, work or lifecycle failure:
  `MATRIX_FREE_CORRESPONDENCE_APPARATUS_REJECTED`;
- any canonical or PCG-site interval does not overlap:
  `DIRECT_RECTANGULAR_PRODUCT_NOT_CONTAINED`;
- retained-wide matrix-free PCG has no certified iterate through 8:
  `RETAINED_WIDE_MATRIX_FREE_PCG_REJECTED`;
- retained-wide passes but export/wide does not:
  `EXPORTED_FACTOR_WIDE_MATRIX_FREE_PCG_REJECTED`;
- all product correspondence and both PCG lanes pass:
  `EXPORTED_FACTOR_WIDE_MATRIX_FREE_PCG_CANDIDATE`.

## Controls and falsifiers

- a small full-row-rank, non-axis-aligned rectangular `T` with known
  `sigma T T^T` passes all canonical products and a known PCG solution;
- a deliberate order-one coefficient mutation fails canonical containment;
- transpose reversal is observable on the non-axis-aligned control;
- recursive/direct residual drift, vector/product/certificate roots and first
  passing iteration are mutation-sensitive;
- classifier precedence covers all five routes;
- R63B--R63M public stdout remains byte-exact.

## Interpretation ceiling

A pass proves bounded finite-profile correspondence for the complete captured
matrix and preserves the R63M result under direct rectangular application. It
selects sparse tangent/operator application as the next research candidate.

It does not prove that materializing dense `T` is faster or smaller, that
binary64/compensated arithmetic is sufficient, that iteration 2 is a runtime
stop rule, or that the solver may replace state. No following NNQP transition,
trajectory, timing, runtime/GPU or production inference occurs. R64 and R65
remain blocked.
