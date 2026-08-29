# NSR3-B4E2D7R20R4 projector semismooth-Newton research

Status: `RESEARCH COMPLETE / DERIVATIVE DIAGNOSTIC SELECTED`.

## Why leave ADMM penalty tuning

R20R2 proves that global coupling helps; R20R3 proves that unconstrained
spectral penalty adaptation can oscillate badly when the box/ball and density
active sets change. More penalty grids would fit observed development cases
without addressing the final KKT system.

The exact projection dual is already available:

```text
d(lambda) = min_{s in D}
            0.5*||s-t||^2 + lambda^T(c+A*s),  lambda>=0

s(lambda) = P_D(t-A^T*lambda)
grad d     = c+A*s(lambda).
```

`P_D` is strongly semismooth away from unresolved active-set degeneracy. A
generalized dual Hessian is

```text
H(lambda) = A * J_D(t-A^T*lambda) * A^T,
```

where `J_D` is a generalized Jacobian of the contact-box/trust-ball projector.
A regularized Newton solve on the projected dual residual can therefore finish
the KKT system directly, with a monotone exact-dual line search as globalization.

This direction matches Newton-type proximal augmented-Lagrangian QP solvers
such as [QPALM](https://arxiv.org/abs/1911.02934), which use semismooth Newton,
exact line search and active factor updates. A July 2026 study on
[degenerate polyhedral projection](https://arxiv.org/abs/2607.12551) warns
that dual generalized Jacobians may remain singular near degenerate solutions;
we must measure rank/degeneracy rather than hide it with an arbitrary diagonal.
For the ball component, the Newton derivative has the familiar tangent
projector form; recent globalized fixed-point work states it explicitly for
closed balls ([Alphonse et al., 2025](https://doi.org/10.1007/s10589-025-00722-8)).

## Box-ball generalized derivative

For `y=P_D(z)`, let `F` be components strictly inside their contact bounds.
If the trust ball is inactive,

```text
J_D h = h on F, 0 on clamped components.
```

If the ball is active, clamped coordinates are fixed and the free coordinates
lie on the remaining-radius sphere. With projection multiplier `eta` and free
projected vector `y_F`, select

```text
J_D h |_F = (1/(1+eta))
             * (I - y_F*y_F^T/||y_F||^2) h_F,
J_D h |not-F = 0.
```

At exact kinks this is one admissible generalized derivative, but Newton
regularity is not assumed. The implementation must expose ball activity,
clamped/free sets, `eta`, remaining radius and all margins.

## First experiment: derivative/rank only

Before any Newton iteration, evaluate the projector and derivative at
`lambda=0` for every R20 case and deterministic tangent-safe probes. Require:

- direct projection KKT and exact reconstruction of `z_F=(1+eta)y_F`;
- central finite-difference agreement for probes whose active set is unchanged;
- symmetry, positive semidefiniteness and norm nonexpansiveness of `J_D`;
- symmetry/PSD of the excited-row dense matrix `A J_D A^T`;
- pivoted rank, nullity, diagonal range and dependency witnesses for the
  positive-density row set;
- dense analytic inactive-ball, active-ball, lower/upper-clamp and degenerate
  controls.

No Newton direction, multiplier update or line search is allowed in R20R4.
The observation selects one of:

1. nonsingular face Hessian -> ordinary semismooth Newton research;
2. rank-deficient but consistent -> representative-selection/lifted active-set
   research;
3. projector derivative failure -> correct the calculus before any solver.

The diagnostic uses the now-public development corpus. New v3 holdouts remain
blocked until the complete oracle policy is fixed.
