# NSR3-B4E2D7R20R63O canonical-defect transport research

Status: `IMPLEMENTED / RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED`.

## Question

R63N proves dense/direct correspondence for every canonical basis vector, but
its local product bounds do not overlap on either large R63I start. Does the
strict linear transport of the already certified per-column representation
defect contain every initial/Krylov product while preserving the unchanged
matrix-free PCG path and original sign certificates?

## Missing term

Let `K = sigma T T^T` denote the exact real matrix represented by the stored
binary128 tangent coefficients and scale, and let stored dense `H` remain the
original captured operator. For canonical column `j`, R63N computes bounded
centers `h_hat_ij` and `k_hat_ij`. Freeze

```text
r_ij = upward_abs_exact_difference(h_hat_ij, k_hat_ij)
E_ij = upward(r_ij + dense_round_bound_ij + direct_round_bound_ij).
```

Then `E_ij >= |H_ij-K_ij|`. For any finite vector `p`, linearity gives

```text
|(H-K)p|_i <= sum_j E_ij |p_j| = model_bound_i(p).
```

All multiplications and sums use existing upward binary128 helpers in fixed
increasing-column order. No scalar multiplier, decimal tolerance or fitted
slack is permitted.

The transported product comparison is therefore

```text
|dense_center_i - direct_center_i|
    <= dense_round_bound_i
     + direct_round_bound_i
     + model_bound_i(p).
```

The direct center still drives PCG. `E` and its transport are verification
only and cannot alter a residual, direction, scalar or iterate.

## Frozen experiment

1. Reconstruct exact R63N, including its canonical root, two raw initial
   failure roots and `DIRECT_RECTANGULAR_PRODUCT_NOT_CONTAINED` route.
2. Recompute all 10,404 canonical pairs and build exactly one immutable
   nonnegative `102 x 102` defect enclosure `E` by the formula above.
3. Require each canonical exact operator difference to be enclosed and bind
   `E` root, maximum entry and row sums.
4. Re-evaluate both original R63I initial products. Publish raw and transported
   slacks; require transported containment before any factor solve.
5. Replay the same two eight-update matrix-free PCG lanes. At all 18 product
   sites transport `E` through the actual current vector and require
   containment. Continue after first sign success exactly as R63M/R63N.
6. Independently certify each iterate against original dense `H,b,X` and all
   102 R60 signs.

The bound is non-vacuous by construction rather than by an adjustable ratio:
every `E_ij` is the one mechanically derived triangle enclosure, with no
inflation factor, and the unmodified matrix-free iterates must still pass the
independent original-system sign certificate.

## Routes

- control, parent, defect construction, exact-difference enclosure, work or
  lifecycle failure: `CANONICAL_DEFECT_TRANSPORT_APPARATUS_REJECTED`;
- a transported initial/Krylov product is not contained:
  `CANONICAL_DEFECT_TRANSPORT_REJECTED`;
- retained-wide matrix-free PCG has no certified iterate through 8:
  `RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED`;
- retained-wide passes but export/wide does not:
  `EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED`;
- all transported products and both lanes pass:
  `EXPORTED_FACTOR_WIDE_TRANSPORTED_MATRIX_FREE_PCG_CANDIDATE`.

## Work and controls

Publish separately:

- the R63N canonical product work;
- 10,404 fixed `E` constructions;
- 10,404 upward defect-transport terms per arbitrary product;
- 18 product transports across two complete PCG lanes;
- unchanged matrix-free candidate work and factor solves;
- dense comparisons, transports and certificates as offline verification.

Controls require a small rectangular operator with a deliberate known dense
column defect, exact arbitrary-vector transport success, failure after setting
the required defect entry to zero, orientation sensitivity, an eight-update
PCG completion control, root mutation and full classifier precedence.

## Interpretation ceiling

A pass proves bounded correspondence and PCG preservation for the complete
captured matrix/RHS profile. It selects fixed-order sparse/zero-elided tangent
application next. It does not make defect transport a runtime requirement;
the independent matrix-free path must stand on its own after validation.

A failure preserves the R63N boundary and selects a different operator
representation or dense-block residual path. No tolerance fit, start removal,
precision change, state replacement, following transition, trajectory, timing,
runtime/GPU or production inference occurs. R64 and R65 remain blocked.

## Result

The immutable defect transport contains both raw R63N failures and all 18
initial/Krylov products. Its maximum entry is `1.36e-35`; no inflation factor
is used. Both matrix-free PCG lanes then complete eight updates, but neither
certifies any iterate against original dense `H`: all retain 66 unresolved
signs and plateau near original-system error `3.746e13`.

The selected route is
`RETAINED_WIDE_TRANSPORTED_MATRIX_FREE_PCG_REJECTED` at semantic
`4b2434fa...2f270`; stdout repeats byte-identically at
`97011922...c131f`. See the
[evidence record](nonlocal-nsr3b4e2d7r20r63o-canonical-defect-transport-evidence-2026-08-26.md).

The next stage must arbitrate stored dense `H` versus direct Gram `K` against
an exact/higher-precision evaluation of `A DPi A^T`. Do not widen the bound or
add a weak-direction patch before that semantic question is answered.
