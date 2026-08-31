# NCGP12 — correctness-first Nonlocal pressure-state discriminator

Status: `FROZEN_REVISION_2 / CPU_ONLY / IMPLEMENTATION_NEXT`

Date: `2026-08-31`

## Decision under test

NCGP10 showed that the corrected mixed-precision CPU and CUDA implementations
agree closely, yet both fragment in the hydrostatic corpus. NCGP11 measured
that failed penalty-only implementation but made no water-quality claim.

This stage asks the smallest causal question needed before another trajectory
or performance run:

> Can the existing corrected Nonlocal density operator and basin support
> generate a nonnegative pressure state that removes the predicted density
> violation caused by gravity, without relying on finite penalty compression?

This is not a switch to DFSPH and does not use the Rust water solver. The
candidate keeps the Nonlocal cubic kernel, corrected kernel scale, particle
mass, horizon, dynamic/ghost geometry and density Jacobian. It changes only
the treatment of incompressibility from a constitutive penalty coefficient to
an explicit unilateral pressure multiplier.

## Frozen hypotheses

| ID | Hypothesis | Distinguishing prediction |
| --- | --- | --- |
| H12A | The penalty constitutive law is the first cause | A nonnegative multiplier solve closes the linearized density/KKT gates while zero pressure does not |
| H12B | Surface tension is the first cause | The pressure-only isolated solve closes, but a later surface-enabled trajectory reintroduces the failure |
| H12C | The corrected density/boundary discretization cannot represent hydrostatic support | Even the converged pressure solve leaves a material KKT, density or force residual |
| H12D | One linearized projection is insufficient but the pressure state is viable | Linearized KKT closes while the exact nonlinear post-update density gate fails, authorizing a bounded nonlinear projection successor |

H12A is not selected merely because the solve returns finite values. The
positive and negative controls and all frozen residual gates must pass.

## Frozen fixture

- profile source: `nonlocal_water_corrected_profile()`;
- `dt=1/240 s`, `spacing=0.05 m`, `horizon=0.15 m`, `mass=0.125 kg`,
  `rho0=1000 kg/m^3`, corrected `kernel_scale=7.985668078772472`;
- isolate incompressibility by setting `lambda=mu=gamma=0` only in this
  discriminator; gravity remains `(0,0,-9.81) m/s^2`;
- dynamic state: exact canonical binary32 `8 x 8 x 8` lattice from
  `make_lattice_state`, zero velocity, stable sample IDs;
- boundary state: exact canonical three-layer basin ghosts from
  `make_basin_ghosts`;
- inclusive current support `r <= horizon`, owner rows in ascending sample-ID
  order and dynamic degrees of freedom in that same order;
- all diagnostic arithmetic is `long double`; no CUDA, timing, surface,
  viscosity, warm start or previous pressure state participates.

## Frozen constrained step

For each dynamic centre, define the corrected Nonlocal density constraint

```text
c_i(x) = rho_i(x) / rho0 - 1 <= 0
```

and its exact sparse row `J_i = dc_i/dx` over dynamic positions. Ghosts
contribute to density and to the owner derivative but have no inertia or
dynamic degree of freedom.

With the gravity-only predictor `x_hat = x + dt^2 g`, linearize at `x`:

```text
b = c(x) + J (dt^2 g)
A = (dt^2 / mass) J J^T

minimize    0.5 lambda^T A lambda - b^T lambda
subject to  lambda >= 0

s = dt^2 g - (dt^2 / mass) J^T lambda
x_trial = x + s
```

The implementation must assemble `rho`, `J` and dense `A` independently from
the current CUDA/full-step evaluator. The symmetric matrix is permitted only
for this 512-centre oracle. It is not the scalable GPU design.

Solve the convex quadratic by deterministic cyclic projected coordinate
descent in ascending row order:

```text
gradient = A lambda - b
lambda_i <- max(0, lambda_i - gradient_i / A_ii)
```

- initialize `lambda=0`;
- update the complete gradient after every coordinate change;
- maximum `4096` full sweeps;
- stop only after a completed sweep when both frozen KKT measures pass;
- reject a nonfinite value, `A_ii <= 0`, symmetry error or sweep exhaustion;
- no tolerance change, retry, relaxation factor or fitted active-set epsilon
  is allowed after observing the result.

## Frozen observables and gates

All norms use deterministic ascending-index accumulation.

1. Kernel/density/Jacobian admission:
   - finite values and no duplicate IDs;
   - use the stable-ID direction
     `v_i=((id mod 17)-8,(id mod 13)-6,(id mod 11)-5)` and normalize the
     complete vector to unit L2 norm;
   - relative L2 `Jv` error against a centred finite difference with
     `epsilon=2^-20 * spacing`, normalized by the larger analytic/finite-
     difference norm: `<= 2e-7`;
   - relative symmetry error of `A`: `<= 2e-12`.
2. Pressure solve:
   - every multiplier is finite and `>=0`;
   - normalized primal violation
     `||max(b-A lambda,0)||_2 / max(||max(b,0)||_2,1e-30) <= 1e-8`;
   - define the dimensionless projected KKT row
     `r_i=min(A_ii*lambda_i,(A lambda-b)_i)` and require
     `||r||_2 / max(||b||_2,1e-30) <= 1e-8`;
   - complementarity
     `max_i |lambda_i (A lambda-b)_i| <= 1e-10 J`;
   - at least one positive multiplier and bottom-layer median pressure larger
     than the top-layer median pressure.
3. Candidate update:
   - linearized post-update constraint agrees with `b-A lambda` to relative
     L2 `<=2e-12`;
   - exact nonlinear density after `x_trial` has maximum positive strain
     `<=1e-3` and RMS positive strain `<=2.5e-4`;
   - normalized stationarity residual
     `||(mass/dt^2)(s-dt^2*g)+J^T lambda||_2 /
      max(||mass*g||_2,1e-30) <=1e-8`;
   - no particle crosses the analytic inset and no particle/ID/mass is lost.

Mandatory controls:

- zero multiplier must leave normalized positive predicted-density violation
  `>=0.99` when the source violation is nonzero;
- negating the pressure correction must fail the exact nonlinear density gate;
- omitting ghost derivatives must fail either pressure/KKT or nonlinear
  density/boundary admission;
- stable-ID permutation must reproduce all scalar metrics within `2e-12`
  relative and the canonical multiplier/update roots exactly;
- one multiplier mutation by `nextafter` must change the result root.

## Routes and authority

- `PRESSURE_STATE_LINEARIZED_SUPPORTED` — every gate and control passes;
  freeze a successor with repeated nonlinear projection and a 240-step tiny
  hydrostatic trajectory before 4k.
- `NONLINEAR_PROJECTION_REQUIRED` — derivative, convex solve, KKT,
  stationarity and controls pass, but only the exact post-update density gate
  fails. Freeze a bounded repeated-projection successor.
- `NONLOCAL_SUPPORT_REDESIGN_REQUIRED` — a converged/apparatus-valid solve
  cannot close pressure/KKT/stationarity or boundary support.
- `APPARATUS_INCONCLUSIVE` — identity, derivative, solver convergence,
  permutation, work or control failure.

This stage cannot claim correct water, a complete Nonlocal solver, 4k/50k
correctness, GPU correspondence, performance, runtime integration or product
readiness. CPU DFSPH remains the fallback and SPEC-38/ADR-076 remain Proposed.

## Prior-art boundary

The experiment is motivated by, but does not inherit correctness from:

- Liu et al., *A Nonlocal Unified Variational Framework for Free Surface
  Flows*, DOI `10.1145/3799902.3811196`;
- Bender and Koschier, *Divergence-Free SPH for Incompressible and Viscous
  Fluids*, DOI `10.1109/TVCG.2016.2578335`;
- the already hash-closed B4E2D5/B4E2D6 PHR pressure-state algebra and path
  oracle in this repository.

The external papers support the distinction between penalty pressure and an
implicit incompressibility solve. They do not prove this fixture, operator,
boundary treatment or implementation.

## Revision 2 correction

Revision 1 was frozen before implementation and then rejected by the required
independent dimensional audit. Its expression
`min(lambda,A lambda-b)` mixed an energy-valued multiplier with a
dimensionless dual gradient, and its stationarity normalization was
underspecified. Revision 2 replaces that KKT map with the dimensionless
`min(A_ii*lambda_i,(A lambda-b)_i)`, defines force normalization, and fixes the
single finite-difference direction. No executable result existed and no
observed number informed this correction.
