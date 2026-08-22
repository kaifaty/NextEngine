# NSR3-B4E2D5 pressure-state formulation research

Date: `2026-08-22`

Status: `RESEARCHED / AUGMENTED_LAGRANGIAN_SELECTED / CONTRACT_READY`

## Structural result

For centre compression `c = rho/rho0 - 1`, the selected unilateral penalty is

```text
Phi_penalty(c) = kappa/2 * max(c, 0)^2.
```

Its pressure coefficient is `kappa*max(c,0)`. Therefore a penalty-only model
cannot represent nonzero hydrostatic pressure at zero compression. Increasing
`kappa` reduces the required strain but also increases acoustic curvature and
never removes that constitutive coupling at finite `kappa`.

B4E2D4 makes the scaling concrete. With `kappa=1226.25 J`, particle mass
`0.125 kg` and `rho0=1000 kg/m^3`, the mapped continuum bulk modulus is
`9.81 MPa`. The converged `0.0011739237712489192` strain represents about
`11.516 kPa`, or `1.1739 m` of hydrostatic head. Holding that pressure at the
current `0.001` cap needs at least `kappa=1439.524 J`, a `1.1739x` curvature
increase and `1.0835x` acoustic frequency/substep scale.

This local increase is modest, but the architecture does not scale to a world:
for the same density cap, the penalty coefficient grows linearly with pressure
or water depth, and the acoustic work lower bound grows with its square root.
Relative to the current one-metre anchor, 10 m and 100 m heads imply about
`3.162x` and `10x` acoustic scale before impact pressure.

## Selected mathematical extension

Use the unilateral Powell--Hestenes--Rockafellar augmented term for
`c <= 0`, pressure state `lambda >= 0` and finite augmentation `beta > 0`:

```text
Phi_AL(c; lambda, beta)
  = (max(0, lambda + beta*c)^2 - lambda^2) / (2*beta)

d Phi_AL / dc = max(0, lambda + beta*c)

lambda_next = max(0, lambda + beta*c)
```

At `lambda=0`, this is exactly the current penalty with `beta=kappa`. At
convergence, a positive `lambda` can supply pressure at `c=0`, satisfying
`c<=0`, `lambda>=0` and `lambda*c=0` without taking `beta` to infinity.

This is the smallest architectural extension that reuses the existing smooth
inner objective, analytic HVP, active rows, trust globalization, neighbor
structure and SIRDI execution path. It introduces an explicit per-centre
pressure state and an outer update. A monolithic semismooth primal-dual solve
remains the fallback if multiplier outer iterations stall.

## State and transaction consequences

The pressure state cannot be hidden mutable solver state. A later production
design must define:

- stable ownership by fluid sample ID;
- initialization/warm start and inactive-state decay/reset policy;
- rollback with the macro transaction;
- canonical publication or a separate deterministic pressure ledger;
- split/merge transfer rules for adaptive particles;
- pressure/contact complementarity interaction;
- convergence gates for primal violation, dual change and complementarity.

These are not authorized yet. First close the scalar algebra under a standalone
no-trajectory identity, then build a tiny dense AL oracle before touching the
nominal Dam path.

## Decision

Do not perform a blind `kappa` sweep as the primary repair. Freeze B4E2D5 to
prove the penalty scaling, PHR special case, derivative, zero-strain pressure
equivalence and complementarity algebra. PASS will authorize only a tiny
augmented-Lagrangian oracle contract.
