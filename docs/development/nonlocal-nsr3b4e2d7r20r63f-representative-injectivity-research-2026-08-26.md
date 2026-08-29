# NSR3-B4E2D7R20R63F representative-injectivity research

Status: `FROZEN FOR IMPLEMENTATION`.

## Corrected question

R63E identifies clamp geometry as the source of a near-null dual mode, but all
represented clamp/tangent Gram blocks remain algebraically full-rank 102. That
invalidates an implicit premise of the first remedy sketch: numerical
near-dependence does not establish an exactly nonunique dual representative.

Before adapting extreme-representative methods, determine whether the
rectangular tangent row operator itself has a nontrivial represented kernel.

## Local geometry

For the captured fixed projector face, let

```text
A = T B^T,   A : R^102 -> R^315,
```

where `T` is the clamp-plus-active-ball tangent projector before the positive
uniform scale. A dual perturbation `delta` has zero first-order effect on the
projected primal state only if

```text
A delta = 0.
```

If `A` has full column rank 102, the only represented infinitesimal
projection-equivalent perturbation is zero. The R63E mode is then extremely
weak but not an exact representative freedom. An extreme-point correction
cannot be imported merely because a numerical Gram rank test reports 101.

The 2026 polyhedral result explicitly exploits nonuniqueness of the dual
representation and a projection-equivalent set. Its mechanism remains an
important future option when that premise holds, but R63F must test the premise
for our box-plus-ball face rather than infer it.

Source: [Ding, Feng and Li, Semismooth Newton methods for degenerate polyhedral projection](https://arxiv.org/abs/2607.12551).

## Why test the rectangular operator

The Gram relation is

```text
G_T = B T B^T = A^T A
```

for an exact symmetric idempotent tangent projector. Forming and inverting the
Gram squares the condition number. LAPACK's least-squares guidance therefore
uses orthogonal factorization/SVD on the rectangular operator when rank or
conditioning is in question, rather than treating a normal-equation pivot as
the primary rank fact.

Sources:

- [LAPACK Working Note 149, normal equations and squared conditioning](https://www.netlib.org/lapack/lawnspdf/lawn149.pdf)
- [LAPACK `DGELSY`, complete orthogonal factorization for least squares](https://www.netlib.org/lapack/explore-html/dc/d8b/group__gelsy.html)

For a later scalable implementation, LSQR is attractive because it accesses a
rectangular operator only through `A v` and `A^T u`, and avoids explicitly
forming the normal equations while retaining sparse/matrix-free structure.
That is a later candidate, not an R63F implementation.

Source: [Paige and Saunders, LSQR: An Algorithm for Sparse Linear Equations and Sparse Least Squares](https://stanford.edu/group/SOL/software/lsqr/lsqr-toms82a.pdf).

## Selected discriminator

Reuse the exact R63E projector state and selected source rows. Materialize the
102 tangent row images `t_i=T b_i` as a `102 x 315` dyadic matrix in the same
stable scalar order.

1. Prove or fail to prove its represented row rank with the two already frozen
   prime fields, operating directly on the rectangular matrix.
2. Independently form `t_i^T t_j` and compare it with the R63E
   `b_i^T T b_j` tangent Gram under a fixed binary128 rounding bound.
3. Reuse the inherited near-null row coefficients `alpha` and require
   `sum_i alpha_i t_i` to be finite, nonzero, and equal to the R63E tangent
   witness image within a fixed bound.
4. Publish the direct tangent matrix root, modular pivots and the witness image
   norm. Do not factor or solve a right-hand side.

## Routes

- Direct rectangular rank 102 under either prime:
  `LOCAL_REPRESENTATIVE_NULLSPACE_REFUTED`.
- A literal exact duplicate/dependency with an independently closed witness:
  `LOCAL_REPRESENTATIVE_NULLSPACE_CANDIDATE`.
- Both modular images deficient without an exact witness:
  `RECTANGULAR_MODULAR_RANK_INCONCLUSIVE`.
- Any tangent/Gram/witness mismatch: apparatus failure.

The expected next branch after full rank is not row removal. It is a separately
frozen no-drop rectangular QR/LSQR feasibility study, preserving all 102
columns and certifying the original NNQP residual/sign semantics.

## Ceiling

R63F proves at most the represented local derivative-kernel property of one
face. It does not prove global injectivity, choose a finite rank tolerance,
factor the rectangular matrix, solve an RHS, alter a representative, apply a
center, run a trajectory or authorize runtime/GPU/production work. R64 and R65
remain blocked.
