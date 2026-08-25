# NSR3-B4E2D7R19R65 active-face FISTA/SHQP research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / MATRIX-FREE DUAL BLOCK PROBE SELECTED`.

## Why another cyclic variant is rejected

The dynamic probe owns all necessary rows before checkpoint 8, preserves the
Dykstra stationarity identity at about `1e-20` and has complementarity near
`1e-16`, yet cyclic primal raw remains about `1e-11` after 2048 sweeps.
Fresh-violation ordering is worse and `omega=1.5` only improves the late raw by
about 3.3x. More depth/order/relaxation search is stopped.

## Dual block at fixed joint-set correction

With joint `box∩ball` correction `p_C` fixed, define

```text
z = target - p_C
b = c + A z
```

The density-halfspace Dykstra block is the nonnegative dual QP

```text
minimize over lambda >= 0
    phi(lambda) = 0.5 * ||A^T lambda||^2 - b^T lambda.
```

Its gradient is `A(A^T lambda)-b`. Both operators are the validated R64 sparse
owner; no Gram is formed. After a dual block,

```text
q   = target - A^T lambda
s   = project_box_ball(q)
p_C = q - s
```

and a fresh R61 audit grows the monotone working set. This is structured dual
alternating minimization/SHQP, not a new physical model. Pang's primary paper
describes Dykstra in the dual alternating-minimization view and accelerated
SHQP variants: [DOI 10.1137/16M106090X](https://doi.org/10.1137/16M106090X).

## Selected exploratory algorithm

Use projected FISTA on the owned dual face:

```text
g      = A(A^T y) - b
lambda = max(0, y - g/L)
```

`L` is not fitted. Start from the maximum owned diagonal and use deterministic
power-of-two backtracking until the exact smooth upper-model inequality holds.
The next iteration may try half the accepted `L`; failure doubles it again.
Use the standard FISTA momentum sequence and deterministic gradient restart.
Newly owned rows start at zero; previous multipliers warm-start unchanged.

Frozen exploratory work, selected before execution:

```text
outer blocks          16
FISTA iterations      16 per block / 256 total
checkpoint blocks     1,2,4,8,16
tolerance stop        none
working-set deletion  none
dense Gram            none
timing                 none
```

At every outer checkpoint record full raw/candidate feasibility, outside rows,
dual objective, projected-gradient norm, stationarity, complementarity, joint
reprojection, inertia/model reduction, operator/backtracking work and roots.

Predeclared outcomes:

```text
ACTIVE_FACE_FISTA_ACCELERATION_CANDIDATE
ACTIVE_FACE_FISTA_DEPTH_REQUIRED
ACTIVE_FACE_FISTA_BACKTRACK_REJECTED
ACTIVE_FACE_FISTA_MODEL_REDUCTION_REJECTED
ACTIVE_FACE_FISTA_REFERENCE_RETAINED
```

This probe has no scientific, runtime, timing or production authority. A later
R65 contract may be frozen only if it strongly reduces equal operator work and
all primal/KKT/joint-set audits remain coherent.
