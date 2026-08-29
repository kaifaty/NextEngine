# NSR3-B4E2D7R19R65 batch projected-Newton research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / BATCH PROJECTED-NEWTON PROBE SELECTED`.

## Why batch projection is the next discriminator

The restart probe spends every one of 256 Hessian products removing one dual
coordinate. Bertsekas' projected-Newton method is specifically designed for
simple nonnegative constraints and permits many constraints to enter or leave
the binding set in one projected iteration. Moré--Toraldo GPCG and Dostál's
proportioning work likewise separate face minimization from a projected face
change:

- [Bertsekas, Projected Newton methods](https://doi.org/10.1137/0320018)
- [Moré--Toraldo GPCG](https://doi.org/10.1137/0801008)
- [Dostál, proportioning and projections](https://doi.org/10.1137/S1052623494266250)

## Selected quadratic step

At fixed joint correction, run one ascending `omega=1` Hildreth sweep and use
15 Jacobi-PCG Hessian products to approximate the unconstrained Newton
direction `d` on `F={lambda>0}`. Form the batch-feasible direction

```text
d_P = max(0, lambda + d) - lambda.
```

This can zero every predicted-negative multiplier simultaneously. Evaluate one
final matrix-free Hessian product `H d_P=A(A^T d_P)`. With dual gradient
`g=-r`, the exact quadratic line minimizer on the feasible segment is

```text
alpha = min(1, max(0, -g^T d_P / (d_P^T H d_P))).
```

Require `g^T d_P<0` unless `d_P` is exactly zero. Apply
`lambda <- lambda + alpha d_P`, reconstruct through fresh `A^T`, evaluate the
fixed-correction dual objective directly, then execute the unchanged joint
projection and full audit.

## Frozen exploratory schedule

```text
outer blocks                         16
Hildreth identification sweeps        1 per outer / omega=1
face PCG Hessian products             15 per outer / 240 maximum
batch projected-direction products     1 per outer / 16
total Hessian products                16 per outer / 256 maximum
backtracking                           0
checkpoint outers                      1,2,4,8,16
tolerance stop                         none; exact zero/curvature only
dense Gram                             none
timing                                 none
```

Record predicted and applied batch-zero counts, slope, curvature, exact line
alpha, objective decrease, projected-gradient norm, recurrence/fresh gap,
primal/KKT/joint/model fields, sparse work and roots.

## Predeclared classifications

```text
BATCH_PROJECTED_NEWTON_ACCELERATION_CANDIDATE
BATCH_PROJECTED_NEWTON_LINE_REJECTED
BATCH_PROJECTED_NEWTON_CURVATURE_REJECTED
BATCH_PROJECTED_NEWTON_RECURRENCE_REJECTED
BATCH_PROJECTED_NEWTON_MODEL_REDUCTION_REJECTED
BATCH_PROJECTED_NEWTON_REFERENCE_RETAINED
```

Acceleration requires all guards, lower structural work than FISTA and strict
terminal dominance of FISTA maximum raw and projected-gradient norm. This
remains a bounded linearized projection experiment only.
