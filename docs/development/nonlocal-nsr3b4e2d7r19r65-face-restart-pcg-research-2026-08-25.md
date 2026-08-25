# NSR3-B4E2D7R19R65 in-budget face-restart PCG research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / PROJECTED PCG RESTART PROBE SELECTED`.

## Problem isolated by the first PCG probe

Jacobi-PCG has positive curvature and reduces each fixed-face linear residual,
but an accumulated 16-step Newton direction is globally truncated by one tiny
positive multiplier. Every outer loses nearly all of the computed direction.
The remedy is active-set restart, not more Krylov depth.

This follows the two-phase face-exploration logic in the primary GPCG work:

- [Moré--Toraldo 1989](https://doi.org/10.1007/BF01396045)
- [Moré--Toraldo 1991](https://doi.org/10.1137/0801008)

## Selected recurrence

After one ascending `omega=1` Hildreth identification sweep, initialize the
strict face `F={i | lambda_i>0}` and preconditioned residual
`r_F=c+Aq`, where `q=target-p_C-A^T lambda`.

For each of at most 16 Hessian products:

```text
h        = A_F A_F^T p
alpha_cg = (r^T M^-1 r) / (p^T h)
alpha_b  = min{-lambda_i / p_i | p_i < 0}
alpha     = min(alpha_cg, alpha_b)

lambda   += alpha p
q        -= alpha A^T p
```

If `alpha_b < alpha_cg`, choose the first stable-row-order minimizer, set that
coordinate exactly to zero, update `q` by its row entries, refresh `c+Aq` with
one exact R64 action and restart PCG on the reduced strict-positive face.
Otherwise perform the normal preconditioned-CG recurrence. The terminal state
is reconstructed from a fresh `A^T lambda` before the joint projection.

This uses the same 16 Hessian-product budget. A boundary refresh costs one
additional `A`, never `A^T`; even the frozen worst case of 256 refreshes keeps
the calculated full sparse work below the FISTA reference before execution.

## Frozen exploratory schedule

```text
outer blocks                       16
Hildreth identification sweeps      1 per outer / omega=1
PCG Hessian products               16 maximum per outer / 256 maximum
face restart                        immediately on first bound hit
boundary residual refresh           one exact A per hit
checkpoint outers                   1,2,4,8,16
tolerance stop                      none; exact zero/curvature only
dense Gram                          none
timing                              none
```

Record all previous primal/KKT/joint/model fields plus boundary hits,
face restarts, residual refresh calls, stable hit-row root, recurrence-to-fresh
terminal gap, exact operator terms and roots.

## Predeclared classifications

```text
FACE_RESTART_PCG_ACCELERATION_CANDIDATE
FACE_RESTART_PCG_DEPTH_REQUIRED
FACE_RESTART_PCG_CURVATURE_REJECTED
FACE_RESTART_PCG_RECURRENCE_REJECTED
FACE_RESTART_PCG_MODEL_REDUCTION_REJECTED
FACE_RESTART_PCG_REFERENCE_RETAINED
```

Acceleration again requires all primal/KKT/reprojection/model guards, lower
structural work than FISTA and strict terminal dominance of both FISTA maximum
raw and projected-gradient norm. No nonlinear trial, runtime mutation, timing
or production authority is admitted.
