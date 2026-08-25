# NSR3-B4E2D7R20R8 verified inverse contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Parent

- R20R7 implementation `96af8d88`, semantic
  `8ca318ee6328d44e3a4a4f6959849d12b6e6a5da79fe271f8a00192b1f2f1c58`;
- exact rejection `PASSIVE_SIGN_UNRESOLVED`, source row 53, value
  `9.227e-2`, cheap bound `2.133e-1`;
- all earlier algorithms, problem roots and development-data status unchanged.

## Frozen refinement

Run the unchanged R7 solver. Only when a passive sign is unresolved by the
cheap bound, solve all unit right-hand sides with the already built binary128
Cholesky factor. Audit every inverse-column residual under the existing gamma
policy. Build `Q`, then outward-bound `||I-HQ||_inf` and `||Q||_inf`.

Require `rho<1`. Set

```text
inverse_bound = ||Q||_inf/(1-rho)
refined_error = inverse_bound*(solve_residual+solve_bound)
                + binary128 rounding allowance.
```

Reclassify every passive component with `refined_error`. If all signs resolve,
continue the exact same active-set state; otherwise preserve the rejection.
Do not alter the solution vector, passive set, pivot order, boundary ratios,
natural face, Armijo sequence or 32-step nonlinear cap.

The refinement may run at most once per ambiguous principal solve. Report
inverse dimension, column solves, `rho`, inverse norm bound, cheap/refined
errors and the affected source row. Controls cover exact diagonal inverse,
correlated SPD improvement, `rho>=1` rejection and unchanged solutions.

Routes:

```text
VERIFIED_INVERSE_PARENT_REJECTED
VERIFIED_INVERSE_AUDIT_REJECTED
VERIFIED_INVERSE_SIGN_UNRESOLVED
VERIFIED_INVERSE_DEVELOPMENT_UNRESOLVED
VERIFIED_INVERSE_DEVELOPMENT_CERTIFIED
```

No higher precision, iterative refinement of values, regularization, pivot
change, warm start, timing, runtime/GPU integration, new holdout or
production/generalization authority.
