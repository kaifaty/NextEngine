# NSR3-B4E2D7R20R5 Newton direction evidence

Status: `PASS / N3 DUAL-CONE INCOMPATIBILITY SUPPORTED`.

## Frozen result

Implementation `e24d039e` ran twice with stable semantic:

```text
5ae59256f4b4836d9ceccf5e2ad6b1a071e6fce7097463b4b8b4ff1e921745b7
```

Route:

```text
PROJECTED_REPRESENTATIVE_SELECTION_REQUIRED
```

All parent, factor, solve, sign-enclosure, dual-value and lifecycle controls
pass. No component was clipped, no active row was removed, and no diagonal was
added.

## What succeeded

The supported transfer case has a strictly nonnegative full-face direction.
It accepts `alpha=1` without a projector-mask change and reaches the complete
strict KKT certificate in one step:

```text
primal             1.33e-33
dual mapping       1.07e-35
complementarity    7.95e-37
stationarity       0
scaled gap         2.11e-30
```

Its certified dual increase lower bound is `1.395e-4`. This independently
confirms that the selected projector derivative, Hessian sign and Newton
equation are correct in a regular regime.

## What failed, and why

Both filled development cases solve their unregularized full-face linear
systems accurately and have positive ascent slopes, but the solutions leave
the nonnegative multiplier cone:

| case | face rows | negative direction components | minimum component | direction error bound |
|---|---:|---:|---:|---:|
| filled edge | 38 | 14 | `-6.759` | `1.14e-15` |
| filled corner | 42 | 12 | `-25.958` | `2.49e-14` |

The signs are separated from their outward bounds by roughly 15 orders of
magnitude. This is not floating-point ambiguity and not Hessian singularity.
It is the expected constrained-Newton geometry: the set of initially positive
dual gradients is larger than the multiplier support of the local quadratic
model.

Because `lambda=0`, any negative direction component violates dual feasibility
for every positive step length. The frozen contract therefore performs zero
line-search trials for these cases. Backtracking cannot fix this obstruction.

## Decision

Replace the unconstrained face equation with the strictly convex nonnegative
quadratic Newton model

```text
minimize  0.5*delta^T H*delta - r^T*delta
subject to delta >= 0.
```

Solve its complementarity system with a deterministic finite active-set
method, certify the selected support, then reuse the exact-dual line search.
Do not tune a diagonal or return to ADMM penalties.

This remains development evidence only. No generalization, runtime, GPU or
production authority exists.
