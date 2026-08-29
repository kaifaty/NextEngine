# NSR3-B4E2D7R20R63H bounded-wide-factor research

Status: `FROZEN FOR IMPLEMENTATION`.

## Question

R63G proves that binary64 storage preserves the immutable weak direction but
strict-binary64 Householder arithmetic loses it. Is it sufficient to promote
only the stored active rectangular block during factor construction, and can
the completed factor then be exported back to binary64 without losing the
same signal?

This stage locates the minimum precision boundary. It does not solve an NNQP
right-hand side.

## Selected mixed boundary

Keep the canonical tangent operator in binary64 exactly as in R63G. At the
private active-block factor boundary only:

```text
binary64 C storage
        |
        | exact promotion of stored values
        v
binary128 Householder workspace -> Q128, R128
        |
        +---- retain wide factor lane
        |
        +---- one-time cast -> Q64_export, R64_export
```

The wide factor is built from the binary64 coefficients, not from the original
binary128 oracle. Therefore it cannot silently recover information already
lost at storage. R63G has separately shown that this storage perturbation lies
inside the weak-signal ball.

This follows the mixed-precision principle that storage, factor, residual and
solution precisions are distinct design choices. Extra-precise refinement
literature similarly retains ordinary input/output formats while using wider
arithmetic at selected factor/residual boundaries.

Sources:

- [LAPACK Working Note 149, extended and mixed precision](https://www.netlib.org/lapack/lawnspdf/lawn149.pdf)
- [LAPACK Working Note 188, extra-precise refinement for overdetermined least squares](https://www.netlib.org/lapack/lawnspdf/lawn188.pdf)
- [Carson and Daužickaitė, mixed-precision least-squares refinement comparison](https://epubs.siam.org/doi/10.1137/24M1664927)

The last source emphasizes that refinement variants have different
conditioning sensitivities. R63H therefore does not infer that a future RHS
will be certifiable merely because the factor preserves one direction.

## Why factor-first, not LSQR-first

The frozen block is only `315 x 102`, its fixed permutation already exists,
and the weakest observed factor diagonal is order `1e-16`. A direct promoted
factor gives a deterministic oracle with bounded storage and work. It can
later precondition a rectangular iteration if necessary.

LSQR/LSMR remain the scalable matrix-free alternative, but extreme
conditioning can cause slow convergence and stagnation even though these
methods avoid explicitly forming normal equations. Reorthogonalization and
stopping policy would introduce additional choices before the simplest
precision boundary is known.

Sources:

- [Paige and Saunders, LSQR](https://stanford.edu/group/SOL/software/lsqr/lsqr-toms82a.pdf)
- [Fong and Saunders, LSMR](https://web.stanford.edu/group/SOL/software/lsmr/LSMR-SISC-2011.pdf)

## Frozen experiment

Replay R63G exactly, including its binary128 oracle factor, strict-binary64
factor, fixed permutation and all four signal lanes. Require their roots and
route to repeat.

Then:

1. Take the already permuted binary64 factor input `C64 P` and promote every
   stored value exactly to binary128.
2. Form an independent binary128 Gram of the unpermuted stored operator.
3. Run the unchanged R63G binary128 Householder construction on `C64 P`.
4. Audit `Q^TQ`, `QR-C64P` and
   `R^TR-P^T(C64^TC64)P` against the stored-operator identities.
5. Apply the wide factor to the inherited exact `alpha` and compare both with
   the stored-operator image and the original R63F image.
6. Cast completed `Q128,R128` once to binary64, evaluate the exported factor
   exactly in binary128 audit arithmetic, and repeat both signal comparisons.

All 102 columns remain present. No new pivot, effective-rank decision or
regularization exists.

## Two signal obligations

Let `u` be the original R63F image and `u64=C64 alpha`. The wide factor must
satisfy both strict gates:

```text
||u_wide-u64||^2 < ||u64||^2
||u_wide-u||^2   < ||u||^2
```

The first checks factor fidelity to the actually stored operator. The second
checks that storage plus factor error still preserves the original physical
direction.

The export lane is classified by the same two open-ball gates. No decimal
tolerance is fitted.

## Routes

- parent/control/correspondence/work failure:
  `BOUNDED_WIDE_FACTOR_APPARATUS_REJECTED`;
- promoted factor fails either wide signal obligation:
  `STORED_OPERATOR_WIDE_FACTOR_REJECTED`;
- wide factor passes but binary64 export fails either obligation:
  `WIDE_FACTOR_STATE_REQUIRED`;
- wide factor and binary64 export both pass:
  `WIDE_BUILD_BINARY64_EXPORT_CANDIDATE`.

## Continuation and ceiling

`WIDE_FACTOR_STATE_REQUIRED` permits a frozen RHS experiment which keeps the
factor and two triangular solves wide, then casts only the candidate solution
and certifies the original system independently. An export candidate permits
the same RHS experiment with exported factors as the control.

A wide-factor rejection returns to reorthogonalized Golub--Kahan or
high-relative-accuracy SVD research. No route selects default binary128
runtime arithmetic.

R63H performs no RHS, triangular solve, iterative refinement, LSQR/LSMR/SVD,
rank action, row drop, regularization, center, replacement, state update,
trajectory or timing. R64 and R65 remain blocked.
