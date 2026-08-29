# NSR3-B4E2D7 augmented-Lagrangian dense-vector research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / NO_TRAJECTORY`

## Question

B4E2D6 proves fast multiplier convergence only on a symmetric scalar path.
The unresolved risk is whether the augmented objective remains correctly
differentiated and practically solvable when all 24 fluid coordinates can move
independently and the pressure Hessian contains both coupled Gauss--Newton and
radial-curvature terms.

## Dense oracle

Retain the B2 `2x2x2` fixture: eight fluid centres, 176 immutable support
samples. Use the real incremental inertia objective with `mass=0.125 kg` and
`dt=1/240 s`.

- active prediction: isotropic factor `0.99`;
- inactive prediction: isotropic factor `1.01`;
- primal variable: all eight binary64 position vectors;
- dual state: one nonnegative multiplier per density centre;
- augmentation: `beta=1226.25 J`.

For active coefficient

```text
a_i = max(0, lambda_i + beta*c_i)
```

the AL terms are

```text
gradient_i = a_i J_i
HVP_i      = beta J_i^T J_i v + a_i H_i v.
```

This is the current penalty gradient/HVP with `beta*c_i` replaced by `a_i`;
the neighbor graph and radial derivatives do not change.

## Inner and outer algorithms

Use a standalone dense-oracle variant of the already validated
Steihaug--Toint trust solver. It must not modify the selected penalty runner.
Every accepted inner step needs positive predicted and actual reduction. The
outer loop performs the frozen PHR update only after a successful private inner
solve.

Close analytic gradient/HVP against finite differences and explicit dense
products before accepting any multiplier result. Then run cold, warm,
inactive, reset, rollback and mutation controls analogous to B4E2D6.

## Decision boundary

- Converged inner/outer controls select `AL_DENSE_VIABLE` and authorize a tiny
  vector transaction contract.
- If all inner solves remain exact and primal violation decreases monotonically
  but eight multiplier updates are insufficient, select the predeclared
  semismooth primal-dual fallback.
- Derivative, trust, ownership, monotonicity or rollback failure is a hard
  failure, not permission to tune `beta`.

This stage still excludes time integration across frames, contact, free-surface
state transport, canonical pressure publication and nominal-scale work.
