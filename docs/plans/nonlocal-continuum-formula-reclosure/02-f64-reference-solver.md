# FCR2 — corrected f64 variational reference solver

Status: `FROZEN FOR IMPLEMENTATION / REPORT_ONLY`

Predecessor: FCR1 pair/pressure semantics `PASS`.

## Purpose

FCR2 creates a correctness authority for the declared total objective before
optimizing it with SISSM. The reference is intentionally simple and slow: a
strict binary64 preconditioned gradient method with deterministic Armijo
backtracking. It is not a production solver candidate.

This ordering follows from the primary-source audit: SISSM requires a
positive/negative split, refreshed nonlinear coefficients and an overshoot
treatment, while the Nonlocal paper provides no unconditional convergence
guarantee without a global line search. Validating the split against another
copy of itself would not independently validate the objective.

## State and graph rules

- immutable step state: `x_i`, `v_i`, material/configuration and `SampleId`;
- prediction: `y_star_i=x_i+dt*(v_i+dt*g)`;
- initial iterate: `y=y_star`;
- density and pressure pairs are rebuilt from each candidate `y` within `h`;
- surface pairs are rebuilt from `y` within `3*r0`;
- viscosity pairs and normals are frozen from `x` within `h`, as in Eq. 10;
- every graph uses the FCR1 lexicographic unique-pair rule and writes both
  endpoints;
- self density is included; self forces are not.

The exact objective is the FCR0 total energy. The full analytical gradient is
evaluated independently of the optimizer update.

## Optimizer

At each iteration:

```text
p = -(dt^2/m) * gradient(E)
alpha = 1
while E(y + alpha*p) > E(y) + 1e-4*alpha*dot(gradient,p):
    alpha *= 1/2
```

The attempt fails after `40` backtracks, on any nonfinite value, or if the
objective increases. Maximum iterations are case-bounded. The report records
initial/final objective, gradient norm, iteration count, total backtracks and
minimum accepted `alpha`. Zero-gradient prediction is a successful no-op.

This line search is a reference safeguard, not a claim that the later SISSM
path must use the same algorithm.

## Tiny corpus

1. `isolated_free_fall`: exact ballistic position and velocity with no
   internal energy.
2. `compressed_pair`: compression-only pressure separates an over-dense pair,
   lowers density error and preserves center of mass.
3. `normal_viscosity_pair`: relative normal velocity magnitude decreases;
   center-of-mass velocity is unchanged.
4. `shear_viscosity_pair`: relative tangential velocity magnitude decreases;
   rigid translation remains null.
5. `surface_repulsive_pair`: a pair at `0.8*r0` moves apart.
6. `surface_attractive_pair`: a pair at `1.7*r0` moves together.
7. `combined_tetrahedron`: all three terms are active, every value is finite,
   objective and gradient norm decrease and internal momentum closes.

Control parameters are local fixtures chosen to make each term observable;
they are not the FCR3 product profile and must not be promoted.

## Exit gate

- analytical full-objective directional derivative agrees with central
  difference at `<=1e-7` on the combined fixture;
- every case is finite and never increases the objective (relative allowance
  `1e-12`);
- every active nontrivial case lowers objective and gradient norm;
- free-fall error, center-of-mass drift and normalized internal momentum
  residual are each `<=1e-12` where applicable;
- pair effects move/damp in the declared direction by more than `1e-9`;
- two reports are byte-identical;
- FCR0/FCR1 and frozen old-line controls remain unchanged.

Passing selects a corrected CPU variational reference for FCR3 only. It does
not prove basin physics, coefficient calibration, SISSM correctness, CUDA or
runtime authority.
