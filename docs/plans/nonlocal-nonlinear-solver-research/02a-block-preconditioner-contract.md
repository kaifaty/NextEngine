# NSR2-A -- block-preconditioner scaling discriminator

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / REPORT_ONLY`

Identity: `nuv-newton-krylov-r0`

## Question

Can the already derived local SPD block-Gauss--Newton curvature reduce full-HVP
Krylov work when used only as a preconditioner, without replacing the exact
objective, gradient, Hessian or trust-ratio acceptance?

FCR3-A used local blocks as the outer optimization direction and failed. NSR2-A
tests a materially different role: the full coupled Hessian remains the Krylov
operator and the block changes only coordinates inside its trust subproblem.

## Candidate metric

For each particle, construct the same local block from:

```text
B_i = inertia_i
    + pressure local J_i^T J_i contributions
    + exact positive viscosity curvature
    + positive-clamped local surface radial/tangential curvature.
```

The inertial term makes every block strictly positive definite. Normalize
`M_i = B_i / (mass/dt^2)`, so the trust norm
`||p||_M = sqrt(sum_i p_i^T M_i p_i)` retains units of metres. In inner PCG:

```text
z = M^-1 r
d_0 = -z
alpha = (r^T z)/(d^T H d)
beta = (r_next^T z_next)/(r^T z).
```

Boundary intersections use the exact `M` norm. Residual stopping replaces
Euclidean residuals with `sqrt(r^T M^-1 r)` and otherwise retains NSR1's
forcing rule. Negative curvature still tests the exact `d^T H d`. Outer
actual/predicted reduction and acceptance remain exactly NSR1.

## Frozen fixtures

Use centered cubic lattices with side counts `2`, `3`, and `4` (`8`, `27`,
`64` particles), placement pitch `0.04 m`, unchanged material spacing/surface
radius `0.05 m`, horizon `0.15 m`, default mass/cadence and:

```text
kappa = 200
lambda = 20
mu = 10
gamma = 100
v(x,y,z) = (-0.35*x + 0.08*y,
            -0.25*y - 0.06*z,
            -0.30*z + 0.05*x)
```

After computing `y_star`, set the single rest density to
`max_i rho_i(y_star) / 1.05`. This creates a deterministic pressure-active
interior/free-surface conditioning fixture; it is not a physical-water claim.

For each fixture run:

1. unchanged NSR1 Euclidean/unpreconditioned trust-region Newton-CG;
2. the block-preconditioned `M`-norm candidate.

Both retain the NSR1 `0.05 m` initial radius, radius policy, `64` outer trials,
`1e-10` gradient stop, exact HVP and objective acceptance. All-pairs loops are
allowed in NSR2-A; NSR2-B owns neighborhood complexity.

## Gates

For every fixture, both paths must succeed and the candidate must:

- finish with objective no greater than baseline plus
  `1e-10 * max(|E_baseline|,1)`;
- finish with gradient norm `<= max(2*||g_baseline||,1e-8)`;
- preserve internal momentum residual `<=1e-12`;
- use no more objective/gradient evaluations or HVP calls than baseline;
- emit no nonfinite, non-positive model, minimum-radius or outer-limit failure.

Across the `27` and `64` fixtures together, candidate HVP calls must be at most
`75%` of baseline HVP calls. The `8` fixture prevents small-case regression but
does not enter the aggregate reduction gate. Two executions must be
byte-identical, and NSR0/NSR1/FCR hashes must remain unchanged.

## Selection

- PASS selects `block-gn-metric-v1` for NSR2-B.
- Failure rejects this preconditioner only. NSR2-B proceeds with the
  unpreconditioned solver, preserving the exact negative result; no blend,
  switch iteration, diagonal shift or clamp tuning is allowed in NSR2-A.
- Penalty/KKT reformulation remains blocked because NSR1 showed excellent model
  agreement; a preconditioner miss is not evidence against the objective.
