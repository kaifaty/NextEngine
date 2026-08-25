# NSR3-B4E2D7R20R7 iterative semismooth contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Parent

- R20R6 implementation `1854c327`, semantic
  `bfa10141c0f50658548205e44d397a8999f36a0d0c0e00da0cb9d3d7ebc01500`;
- route `NNQP_RELINEARIZATION_REQUIRED`;
- exact v2 problem roots and binary128 arithmetic profile;
- v2 cases are public development data.

## Frozen iteration

Start at `lambda=0`. Before each step, evaluate the exact joint box-ball
projector, complete KKT tuple and exact dual interval. Stop immediately on the
frozen `2^-70` certificate.

Otherwise select the natural-residual face by the outward sign of

```text
lambda_i + r_i / ||a_i||^2.
```

Any sign interval containing zero rejects the ordinary derivative. Build the
selected `H=A*J_D*A^T` and
`b_I=r_I+(H*lambda)_I`. Solve the nonnegative quadratic problem for the next
local representative `p_I` with the unchanged R20R6 single-pivot algorithm;
set `p_Z=0` and `delta=p-lambda`.

Require an outward-positive `r^T delta`. Use the unchanged 21-trial exact-dual
Armijo sequence and accept the first certified step. Every accepted state must
increase the prior dual interval. Rebuild all masks, rows and factors after
acceptance; do not reuse a stale derivative or passive set.

The fixed cap is 32 accepted nonlinear iterations per case. Record per-step
natural-face size, NNQP support, active-set transitions, line-search power,
projector mask changes, dual increase and complete KKT tuple. Quiet cases must
certify at iteration zero.

Controls cover active/inactive natural-residual branches, the exact R20R6 first
step, nonnegative convex-combination updates, nonpositive slope rejection and
monotone dual intervals.

Routes:

```text
SEMISMOOTH_PARENT_REJECTED
SEMISMOOTH_ACTIVE_SET_REJECTED
SEMISMOOTH_DIRECTION_REJECTED
SEMISMOOTH_GLOBALIZATION_REJECTED
SEMISMOOTH_DEVELOPMENT_UNRESOLVED
SEMISMOOTH_DEVELOPMENT_CERTIFIED
```

No warm-started NNQP, regularization, trust region, parameter grid, timing,
runtime/GPU integration, new holdout or production/generalization authority.
