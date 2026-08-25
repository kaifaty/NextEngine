# NSR3-B4E2D7R20 scale-free KKT stopping research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / CONTRACT SELECTED / NOT EXECUTED`.

## Fixed convex problem

Each R20 case audits the same dimensionless convex TRQP as R65:

```text
minimize    F(s) = 0.5 ||s - t||^2
subject to  c + A s <= 0
            s in D = contact box intersect global trust ball.
```

For `lambda >= 0`, let

```text
z(lambda) = t - A^T lambda
s(lambda) = projection_D(z(lambda)).
```

Completing the square gives an exactly evaluable concave dual:

```text
d(lambda) = 0.5 ||s-z||^2
          + lambda^T c
          + (A^T lambda)^T t
          - 0.5 ||A^T lambda||^2.
```

Its gradient is `r = c + A s`. This is why the R65 composed-dual merit is the
correct inner merit for the fixed projection problem. It does not grant the
same merit ownership to the outer nonlinear Nonlocal solve.

## Row-scale invariant residual

Let `d_i = ||a_i||^2`, trust radius `Delta`, and `b_i` be the predeclared
binary64 forward-error upper bound for row `i`. Define

```text
sigma_i = max(|c_i|, Delta*sqrt(d_i), b_i)
q_primal = max_i [r_i]_+ / sigma_i

lambda_i^+ = max(0, lambda_i + r_i/d_i)
G_i = d_i * (lambda_i - lambda_i^+)
q_dual = max_i |G_i| / sigma_i.
```

If one inequality is multiplied by any positive scalar `alpha`, then `c_i`,
`a_i`, `r_i`, `G_i` and `sigma_i` all scale by `alpha`, while its multiplier
scales by `1/alpha`. Both quotients are unchanged. This is the required
nondimensional property missing from the raw v11 norms.

The remaining tuple is:

```text
q_stationarity = ||s - projection_D(t-A^T lambda)||_inf / s_scale
q_complementarity = max_i |lambda_i*r_i| / E_scale
q_dual_change = (d_k-d_{k-1}) / E_scale
remaining structural budget.
```

`q_stationarity` must lie inside an analytic projection/reduction roundoff
bound, not an observed epsilon. `E_scale` is the maximum of the inertial trust
scale `0.5*Delta^2`, current primal/dual magnitudes, multiplier row scales and
their forward-error floor.

Dual change is diagnostic only. Dykstra sequences can stall while still far
from the projection, so a small iterate or objective change may trigger an
offline/fallback route but may never declare convergence.

## Selected stopping gate

The candidate may stop only at a complete outer transaction when all hold:

```text
q_primal          <= 2^-20
q_dual            <= 2^-20
q_complementarity <= 2^-20
q_stationarity    <= analytic binary64 bound
dual is finite and monotone within its analytic bound
all contact/projection/rollback controls pass.
```

`2^-20` selects approximately six binary digits in the scaled KKT system. It
is a binary rational derived from the normalized problem, not the v11 final
value. R20 does not retroactively use this gate to strengthen the one-fixture
R65 claim.

The hard cap is 32 complete outers with no result-dependent extension. Failure
to satisfy the tuple is a bounded failure, never implicit convergence.

## Independent offline oracle

For these `24--75` row cases, use an offline `__float128` primal Dykstra lane
over every density halfspace, the contact box and trust ball. It must use a
different update order and state representation from the composed-dual
candidate, run fixed power-of-two checkpoints through at most `2^18` cycles,
and require both binary128 KKT and primal-dual gap certificates.

The oracle may return `ORACLE_UNRESOLVED`; it may not infer infeasibility from
failure to converge. Candidate accuracy is compared to the first certified
oracle checkpoint. FISTA remains a work competitor, not ground truth.

## Why this design

OSQP and the ADMM convergence literature stop on separately scaled primal and
dual residuals rather than iterate change alone. Bregman, Censor and Reich
identify Dykstra as a primal-dual/dual-coordinate method, supporting an
independent primal projection lane. Perkins also notes that ordinary Dykstra
iterates need not be feasible and derives separate error bounds; this is why
R20 requires a certified oracle checkpoint rather than a fixed cycle count.

Primary references:

- [OSQP solver residual definitions and absolute/relative scaling](https://osqp.org/docs/solver/)
- [Boyd et al., ADMM optimality conditions and stopping criteria](https://web.stanford.edu/~boyd/papers/pdf/admm_distr_stats.pdf)
- [Bregman, Censor and Reich, Dykstra as primal-dual optimization](https://math.haifa.ac.il/yair/Dykstra.jca99.pdf)
- [Perkins, convergence/error analysis for polyhedral Dykstra](https://doi.org/10.1137/S0036142900367557)

## Scope

This research selects the R20 contract. It does not execute the solver, fit a
tolerance, run outer 21 on the dam fixture, time CPU/GPU code, mutate runtime
state or promote Nonlocal to production.
