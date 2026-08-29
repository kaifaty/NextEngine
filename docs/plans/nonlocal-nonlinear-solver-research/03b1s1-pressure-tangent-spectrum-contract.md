# NSR3-B1S1 -- pressure tangent-spectrum diagnostic contract

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / TRAJECTORY_POLICY_BLOCKED`

Parent failure: `LINEAR_ACOUSTIC_POLICY_REJECTED`, B1S semantic SHA-256
`1537a691dfa821c6ccc7ce168c4d4c74868b10db95075a615eaf887f443eabee`.

Objective identity remains `nuv-variational-fcr2`.

## Question

B1S shows amplitude-dependent error that is nearly invariant after the
expected `sqrt(kappa)` scaling. B1S1 asks whether the maximum positive
eigenfrequency of the finite-state pressure Hessian supplies a deterministic,
bounded amplitude factor suitable for a later substep policy.

No trajectory is rerun and no new Courant target is selected in B1S1.

## Operator

At the initial material state `y=x`, freeze the compression-only active set and
define

```text
H_p = d^2/dy^2 [ kappa/2 sum_i max(rho_i(y)/rho0 - 1, 0)^2 ].
```

Use the normalized FCR2 `W/dW/d2W`, canonical unique pairs and the complete
Gauss--Newton plus geometric pressure curvature. Inertia, viscosity and
surface curvature are excluded. The maximum algebraic eigenvalue is
`lambda_max`; define

```text
omega_max = sqrt(max(lambda_max,0) / m)
a_spectral = omega_max / (sqrt(kappa/m) / dx)
           = dx sqrt(max(lambda_max,0) / kappa).
```

Translation modes must remain null to normalized residual `<=1e-12`.

## Deterministic eigensolver

Use at most 48 steps of symmetric Lanczos with full two-pass
reorthogonalization. The start vector at flat component `r` is

```text
sin((r+1) sqrt(2)) + cos((r+1) sqrt(3)),
```

with all three rigid-translation components projected out before
normalization. Build the tridiagonal eigensystem with the existing deterministic
binary64 Jacobi convention. Publish iteration count, Ritz residual
`|beta z_last| / max(|lambda_max|,1)`, orthogonality error and exact-repeat
root. Ritz residual and orthogonality must be `<=1e-8` and `<=1e-10`.

## Independent tiny oracle

Use the B0R normalized active tetrahedron with pressure only. Assemble its
`12x12` dense pressure Hessian by basis HVPs, solve it independently with
Jacobi and compare the maximum eigenvalue to Lanczos at relative error
`<=1e-10`. Dense symmetry and dense/HVP product errors remain `<=2e-12`.

## Six-state matrix

Use the exact B1S initial states:

```text
spacing / dx = {0.99, 0.98}
kappa / 1226.25 = {0.25, 1, 4}.
```

For every state publish active count, pair/neighbor capacity, `lambda_max`,
`omega_max`, `a_spectral`, Lanczos residual and operator-call count. Require:

- finite positive `lambda_max` and valid capacities;
- at fixed amplitude, eigenvalues scale by `4` between adjacent `kappa`
  factors within `1e-10`, frequencies by `2` within `1e-10`, and
  `a_spectral` is invariant within `1e-10`;
- the mean `a_spectral` at `0.98 dx` is strictly greater than at `0.99 dx`;
- two independent executions are bit-exact.

An uncompressed `1.00 dx` lattice is the inactive null control: pressure-active
count is zero and pressure HVP norm/eigenvalue are `<=1e-12` in the operator's
published scale.

## Decision and exit

PASS selects only `SPECTRAL_POLICY_PREMISE_VALID` and authorizes B1S2 policy
design using `dt*omega_max` instead of linear acoustic Courant. Failure stops
spectral policy work and requires a different amplitude model.

Neither result authorizes B1R/B2, boundaries, hydrostatics, CUDA, runtime,
public schemas, save/replay or production use. The cost of estimating the
spectrum must be published and cannot be hidden inside a future policy.

