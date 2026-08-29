# NSR3-B4E2D7R19R30 linearized range-projection research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

## Question

At the exact R28 successor, how much of the currently violated compression
residual lies outside the image of the verified dimensionless linearized
constraint operator?

R29 closes

```text
A = SPACING * J_c
```

over all 6000 constraint rows and 18000 fluid coordinate variables. R30
restricts only the rows with exact binary64 `c_i > 0`. Let `R` be that row
restriction, `B = R A`, and `b = -R c`. The diagnostic problem is

```text
minimize ||B v - b||_2
```

with zero initial iterate. Its converged residual `q = b - Bv` is the
component of `b` orthogonal to `Range(B)` to numerical accuracy.

This is an equality-range diagnostic. It is deliberately stronger than the
linearized unilateral condition `Bv <= b`, but it ignores activation of rows
outside the frozen violated set. Therefore neither a small nor a large
projected residual alone proves a nonlinear feasibility or discretization
floor.

## Method selection

Use zero-damping LSMR with matrix-free products. Fong and Saunders derive LSMR
from Golub--Kahan bidiagonalization and show that it is algebraically
equivalent to MINRES on the normal equations while monotonically reducing the
normal residual `||B^T q||`; this makes early least-squares termination safer
than a method that monitors only `||q||`:

- [Fong and Saunders, LSMR (SIAM J. Sci. Comput. 2011)](https://web.stanford.edu/group/SOL/software/lsmr/LSMR-SISC-2011.pdf)
- [Stanford SOL LSMR implementation notes](https://web.stanford.edu/group/SOL/software/lsmr/)

LSQR is the retained comparison basis. It also uses only forward/adjoint
products and is numerically preferable to explicitly forming normal
equations, but only its primal residual norm is monotone:

- [Paige and Saunders, LSQR (ACM TOMS 1982)](https://web.stanford.edu/group/SOL/software/lsqr/lsqr-toms82a.pdf)

CGLS or explicit `B^T B` is rejected because it unnecessarily squares the
conditioning in the arithmetic representation. A dense SVD/QR is rejected
because the production-relevant operator has 18000 columns and R29 exists
specifically to preserve matrix-free pair actions.

## Scaling and frozen solver policy

Compute exact scalar-coordinate column 2-norms of `B` from the owned-pair row
gradients. Define diagonal `D` by

```text
D_jj = 1 / ||B[:,j]||_2   when the norm is nonzero
D_jj = 0                  otherwise
C = B D
```

`Range(C) = Range(B)` because zero columns contribute no range and every
nonzero column is scaled by a finite nonzero factor. This follows the primary
LSMR recommendation to normalize available column norms. Row scaling is
forbidden because it would change the Euclidean projection being measured.

Frozen LSMR policy:

```text
lambda          0
initial y       0
maximum iter    512
ATOL            1e-10
BTOL            1e-10
CONLIM          1e12
reorthogonalize none
```

No parameter is selected after observing the nominal result. Breakdown is
accepted only if an independently recomputed residual satisfies a frozen
stopping rule. A condition-limit or iteration-limit stop without that direct
certificate is `RANGE_PROJECTION_NOT_CONVERGED`, not evidence of a physical
floor.

## Independent acceptance

After LSMR stops, recompute with fresh pair actions:

```text
p = C y
q = b - p
g = C^T q
```

Accept convergence if all values are finite and either the compatible rule

```text
||q|| <= BTOL ||b|| + ATOL ||C||_F ||y||
```

or the least-squares rule

```text
||g|| <= ATOL ||C||_F ||q||
```

holds. Also require energy-normalized projection orthogonality

```text
|p^T q| / max(||b||^2, tiny) <= 1e-10
```

and Pythagorean defect at most `1e-10`. The angle cosine
`|p^Tq|/(||p||||q||)` is report-only because it is ill-conditioned when a
compatible solve drives `||q||` to roundoff. These are verification controls,
not a threshold on the physical size of `q`.

The implementation itself must first pass four small dense controls:

1. square identity;
2. compatible underdetermined system;
3. inconsistent overdetermined system with analytic projection;
4. rank-deficient inconsistent system with analytic projection.

Each uses the same LSMR implementation and frozen `1e-12` dense residual,
normal-residual and solution/projection checks.

## Required observations

Report without classifying them:

- iteration/stop reason and exact work count;
- RHS, scaled-column, iterate, projection, residual and normal roots;
- `||q||/||b||`, `||C^Tq||/(||C||_F||q||)` and projection orthogonality;
- compatible-rule value, estimated condition and zero/nonzero column counts;
- boundary/interior decomposition of `b`, `p` and `q` within the violated
  rows;
- full-row linear response `A v`, predicted newly positive rows outside the
  frozen violated set, and RMS/maximum dimensionless preimage magnitude.

The last items reveal active-set leakage and impractically large linear
preimages, but have no pass threshold in R30. Applying `v`, evaluating a moved
nonlinear state or refining topology belongs to a later separately frozen
experiment.

## Work and authority

One exact R29/R28 parent replay, one diagnostic workspace, one column-norm
assembly pass, one initial adjoint pass, two pair passes per LSMR iteration and
two independent final passes. At 512 iterations the hard maximum is 1028 pair
passes. No Nonlocal HVP, objective/model evaluation, trust trial, precision
audit, AL outer, state update, substep, macro, trajectory or timing is allowed.

R30 can select only `LINEARIZED_RANGE_PROJECTION_CANDIDATE`. It cannot select
a correction, nonlinear floor, solver policy, runtime route or production
authority.

## Continuation

- converged projection: inspect the frozen residual and active-set leakage,
  then research a separate nonlinear validation/refinement discriminator;
- nonconvergence: preserve exact output and research conditioning,
  reorthogonalization or a stronger range method without changing R30;
- dense-control/operator/source failure: repair that first exact boundary and
  do not interpret nominal projection values.
