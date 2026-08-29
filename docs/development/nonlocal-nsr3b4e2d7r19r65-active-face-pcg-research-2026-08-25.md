# NSR3-B4E2D7R19R65 active-face PCG research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / TWO-PHASE MATRIX-FREE PROBE SELECTED`.

## Evidence-driven change

Projected FISTA is mathematically coherent but spends 770 `A^T` calls and 288
`A` calls to reach maximum raw `1.1823169686944491e-9`. Its scalar Lipschitz
step cannot efficiently resolve the strongly correlated, redundant active
face. More FISTA depth, momentum tuning or another relaxation grid is stopped.

Moré and Toraldo's large-scale bound-QP methods use two phases: gradient
projection identifies a face, then conjugate gradients minimize the quadratic
on that face. That division matches this problem exactly:

```text
dual bound QP       lambda >= 0
identification      direct Hildreth coordinate sweep
face minimization   PCG on rows with lambda > 0
global convex set   exact box intersection ball projection
```

Primary sources:

- [Moré--Toraldo 1989](https://doi.org/10.1007/BF01396045)
- [Moré--Toraldo 1991](https://doi.org/10.1137/0801008)

## Selected operator

At fixed joint correction `p_C`, maintain

```text
q = target - p_C - A^T lambda
r = c + A q = b - A A^T lambda.
```

One ascending `omega=1` Hildreth sweep updates every owned coordinate and `q`
directly. This activates rows whose dual gradient points into the feasible
orthant and releases rows whose coordinate minimizer reaches zero.

Freeze the CG face as the rows with strictly positive post-sweep `lambda`.
Solve the face Newton equation

```text
(A_F A_F^T) d = r_F
```

with matrix-free Jacobi-preconditioned CG. The diagonal is the already
validated R64 row squared norm. Every Hessian product is one bounded-equivalent
`A^T` followed by `A`; no overlap table or Gram is formed.

The accumulated direction is applied with

```text
alpha = min(1, min{-lambda_i / d_i | d_i < 0}).
```

Thus no dual coordinate becomes negative. Rows that hit zero leave the next
CG face; the next Hildreth sweep can reactivate them if the full gradient again
points inward. Reconstruct `q` from a fresh `A^T lambda` before every joint
projection rather than relying on recurrence bits.

## Frozen exploratory schedule

Selected before execution:

```text
outer blocks                    16
identification sweeps            1 per outer / omega=1
PCG iterations                  16 maximum per outer / 256 maximum
checkpoint outers                1,2,4,8,16
tolerance stop                   none; only exact-zero/curvature stop
preconditioner                   exact R64 diagonal / Jacobi
working-set deletion             natural lambda=0 face exclusion only
dense Gram                       none
timing                           none
```

At every checkpoint record ownership/additions, positive/nonzero face counts,
maximum fresh raw, projected-gradient norm, stationarity, complementarity,
joint reprojection, objective/inertia/model reduction, boundary truncations,
curvature stops, `A/A^T` calls, row-entry terms and deterministic roots.

## Predeclared classifications

```text
ACTIVE_FACE_PCG_ACCELERATION_CANDIDATE
ACTIVE_FACE_PCG_FACE_IDENTIFICATION_REQUIRED
ACTIVE_FACE_PCG_CURVATURE_REJECTED
ACTIVE_FACE_PCG_MODEL_REDUCTION_REJECTED
ACTIVE_FACE_PCG_REFERENCE_RETAINED
```

Acceleration requires all primal/KKT/joint-set/model guards, strictly fewer
full sparse-operator terms than the FISTA probe, and strict terminal dominance
of both FISTA maximum raw and projected-gradient norm. It grants only a
bounded exploratory solver candidate. It does not authorize a nonlinear trial,
runtime mutation, timing or production use.
