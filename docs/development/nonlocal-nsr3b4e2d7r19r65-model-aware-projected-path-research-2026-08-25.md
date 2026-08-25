# NSR3-B4E2D7R19R65 model-aware projected-path research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / MODEL-AWARE PATH PROBE SELECTED`.

## Problem isolated by v4

The projected-Newton path is the correct nonnegative-dual geometry and
successfully changes many active coordinates. Its v4 line search, however,
only sees the fixed-correction dual quadratic

```text
Delta phi(delta) = -raw^T delta + 0.5*||A^T delta||^2.
```

The actual outer method then sends `target-A^T lambda` through the exact
contact-box/trust-ball projection. That second block changes the final
inertia model. Outer 2 proves that `Delta phi<0` does not imply positive
reduction after this composition.

This is consistent with projected-Newton globalization and block-projection
methods: descent must be tested against the objective owned by the composed
step, not only against an intermediate block model.

- [Bertsekas, Projected Newton methods](https://doi.org/10.1137/0320018)
- [Pang, SHQP acceleration for Dykstra's algorithm](https://doi.org/10.1137/16M106090X)

## Selected acceptance rule

Retain the v4 direction and largest-first dyadic sequence. For every
candidate `delta_k`:

1. compute `A^T delta_k` and require finite strict `Delta phi_k<0`;
2. reconstruct `A^T lambda_k` from the current exact recurrence;
3. project `target-A^T lambda_k` through the frozen exact box-ball operator;
4. form the physical endpoint from R43 and compute the frozen inertia;
5. require finite strict `normal_inertia-candidate_inertia>0`;
6. accept the first candidate satisfying both conditions.

The candidate projection is a model oracle only. After selection, freshly
reconstruct `A^T lambda`, repeat the committed joint projection, audit all
rows and retain the existing checkpoint gates. No candidate may publish
state directly.

## Frozen exploratory schedule

```text
outer blocks                         16
Hildreth identification sweeps        1 per outer / omega=1
face PCG Hessian products             15 per outer / 240 maximum
projected-path candidates              1..16 per outer
candidate order                        1,1/2,...,2^-15
dual acceptance                        strict finite decrease
joint-model acceptance                 strict finite positive reduction
candidate joint projections            1 per dual-decreasing candidate
tolerance                               none
dense Gram                              none
timing                                  none
```

Record separately dual rejections, model rejections, candidate projection
calls, selected candidate model reduction and the already frozen
face/zero/recurrence/KKT/joint/work roots. Candidate box-ball projections do
not add sparse operator terms, but they remain explicit work counts.

## Predeclared classifications

```text
MODEL_AWARE_PROJECTED_PATH_ACCELERATION_CANDIDATE
MODEL_AWARE_PROJECTED_PATH_LINE_REJECTED
MODEL_AWARE_PROJECTED_PATH_MODEL_REJECTED
MODEL_AWARE_PROJECTED_PATH_CURVATURE_REJECTED
MODEL_AWARE_PROJECTED_PATH_RECURRENCE_REJECTED
MODEL_AWARE_PROJECTED_PATH_CHECKPOINT_MODEL_REJECTED
MODEL_AWARE_PROJECTED_PATH_REFERENCE_RETAINED
```

Acceleration still requires strict terminal dominance over FISTA at lower
counted sparse work. Do not increase PCG depth, fit tolerances, weaken the
checkpoint model gate, run timing or admit the result into production.
