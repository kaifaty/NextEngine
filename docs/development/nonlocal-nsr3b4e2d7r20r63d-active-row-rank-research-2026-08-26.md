# NSR3-B4E2D7R20R63D active-row rank research

Status: `FROZEN FOR IMPLEMENTATION`.

## Question

R63C ruled out another ordinary binary64 inverse-factor construction: the
required lower left defect remains noncontractive even when the inverse rows
are formed directly from transposed triangular solves. Before considering a
rank-revealing solver or extended precision, what actually makes the selected
102-dimensional passive principal system difficult?

The alternatives are materially different:

1. the 102 original constraint rows are exactly dependent;
2. they are exactly independent but already numerically near-dependent;
3. the source rows are numerically healthy and the active projector metric
   creates the near-null directions;
4. both represented systems retain full numerical rank at the standard
   binary64 threshold, while explicit inverse formation still crosses the
   conditioning boundary.

R63D is a report-only discriminator among those explanations. It cannot drop
a row, choose a physical rank, solve an NNQP right-hand side or alter the
solver.

## Why a Gram rank alone is insufficient

The captured factor matrix is a passive principal block of

```text
H = B J B^T,
```

where `B` contains the selected original constraint rows and `J` is the
current projector derivative. A small pivot in `H` therefore does not say
whether the dependency was already present in `B` or introduced/amplified by
the metric `J`. R63D must capture the exact 102 source row IDs from the same
NNQP audit and reconstruct `B` from the immutable sparse row operator.

Normal equations also square the condition number. LAPACK Working Note 149
uses this fact when contrasting normal-equation and orthogonal-factorization
approaches. Consequently, a failed inverse of `H` is not evidence that the
underlying rows are exactly redundant.

Source: [LAPACK Working Note 149, Accurate solution of least squares problems in floating point arithmetic](https://www.netlib.org/lapack/lawnspdf/lawn149.pdf).

## Two rank meanings

R63D deliberately keeps exact algebraic rank separate from numerical rank.

### Exact represented rank

Every finite binary64/binary128 coefficient is a dyadic rational. Reduction
modulo an odd prime is therefore well-defined because powers of two are
invertible. Deterministic Gaussian elimination is applied to:

- the rectangular source matrix `B` (`102 x 315` for the frozen case);
- the captured projected Gram block `H` (`102 x 102`).

If either prime image has row rank 102, some 102-minor is nonzero modulo that
prime and hence nonzero over the rationals. This is a one-way proof of exact
full row rank for the represented coefficients. A deficient modular image is
not, by itself, proof of exact deficiency: a nonzero minor may happen to be
divisible by the selected prime. Such a result must be reported as
inconclusive unless a literal duplicate row is independently found.

The two fixed Mersenne primes are `2^61-1` and `2^31-1`. They are not a
probabilistic sample selected after observing the matrix.

### Numerical rank

For diagnosis only, form the diagonally normalized source and projected Gram
matrices and run deterministic complete-pivoted Cholesky in binary64. The
stopping threshold is fixed to the LAPACK `DPSTRF` default form

```text
n * u64 * max(diagonal),  u64 = 2^-53.
```

LAPACK documents `DPSTRF` as a complete-pivoted Cholesky factorization for a
positive semidefinite matrix and defines its default rank threshold in this
form. It also warns that the routine does not establish positive
semidefiniteness for arbitrary input. R63D therefore calls the result a
*numerical rank diagnostic*, not a physical or exact rank.

Sources:

- [LAPACK `DPSTRF` source and default threshold](https://www.netlib.org/lapack/explore-html/dd/dad/dpstrf_8f_source.html)
- [LAPACK Working Note 161, complete pivoted Cholesky](https://www.netlib.org/lapack/lawnspdf/lawn161.pdf)

The existing binary128 pivoted-rank diagnostic is also evaluated on both
normalized matrices. It is a precision-profile comparison only; it cannot
override the modular proof or authorize row removal.

## Selected discriminator

One frozen parent replay captures the unique R63 matrix root and the matching
source row IDs. The harness then publishes:

- source row ID root and dense source coefficient root;
- exact modular ranks of `B` under both fixed primes;
- exact modular ranks of represented `H` under both primes;
- binary64 and binary128 normalized pivot profiles for `B B^T` and `H`;
- literal duplicate-row count;
- immutable construction and rank work ledgers.

Classification precedence is:

1. apparatus/capture/correspondence failure;
2. literal exact source duplicate;
3. source modular proof inconclusive;
4. projected modular proof inconclusive;
5. exact-full source but binary64 source numerical rank below 102:
   `SOURCE_ROW_NUMERICAL_NEAR_DEPENDENCE`;
6. binary64 source numerical rank 102 but projected rank below 102:
   `PROJECTOR_METRIC_NUMERICAL_RANK_LOSS`;
7. both binary64 numerical ranks 102:
   `FULL_RANK_INVERSE_CONDITIONING_BOUNDARY`.

The last route does not claim that the matrix is well-conditioned. It says
only that the standard fixed pivot threshold does not select a smaller rank,
so an explicit-inverse failure must not be repaired by silently deleting a
constraint.

## Branch after the result

- Source near-dependence: research scaled RRQR/SVD semantics and the physical
  meaning of redundant constraints before any row policy.
- Projector-metric loss: derive a nullspace/range-space formulation that
  respects clamped and ball-tangent projector geometry.
- Both exact-full and numerically full: research a rare verified solve in
  wider precision or a factor-based residual certificate that avoids explicit
  inverse formation.
- Modular result inconclusive: add an exact rational/minor or independently
  certified rank witness; do not infer deficiency.

Rank-revealing QR can be an implementation candidate later, but its threshold
and column/row semantics must be frozen separately. The classic RRQR literature
explains why pivoting quality matters; it does not supply the physical policy
for discarding our constraints.

Source: [Chan and Hansen, Some applications of the rank revealing QR factorization](https://epubs.siam.org/doi/abs/10.1137/0913043).

## Ceiling

R63D is one immutable case and one passive block. It proves no universal rank
property, chooses no production tolerance, executes no RHS, applies no center,
changes no trajectory and provides no runtime, GPU, timing or production
authority. R64 and R65 remain blocked.
