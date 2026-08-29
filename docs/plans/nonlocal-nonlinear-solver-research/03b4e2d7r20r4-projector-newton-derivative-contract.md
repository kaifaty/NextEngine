# NSR3-B4E2D7R20R4 projector/Newton derivative contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Parent

- R20R3 implementation `58132477`, semantic
  `cd094b7fe5cb7ca5a713f60185493250fb9d66ed3221038eb1c8a0f0a25db486`;
- scientific route `M3_ADAPTIVE_PENALTY_INSTABILITY_SUPPORTED`;
- emitted route defect preserved; no rerun or relabelled semantic;
- exact v2 manifest/preflight/problem roots; cases are development data;
- candidate/Newton iterations zero.

## Frozen diagnostic

At `lambda=0`, compute `z=t`, the exact binary128 box-ball projection `y`, its
selected generalized derivative and the positive-density row set. Use stable
scalar component order and expose the projector multiplier, ball activity,
free/clamped masks and margins.

Run exactly six deterministic tangent-safe derivative probes per nonzero case.
Use central binary128 differences at steps `2^-20` and `2^-24`; require the
active mask to remain unchanged and the two error levels to contract. Check
bilinear symmetry, PSD and operator norm of the analytic derivative.

Build the dense excited-face matrix `A_I J_D A_I^T`. Use deterministic
maximum-diagonal pivoted Cholesky with a binary128 gamma-bound pivot test;
report rank, nullity, positive pivots, rejected pivot bounds, reconstruction
and a dependency-witness root. Do not regularize a rejected pivot.

Analytic controls cover inactive/active ball, lower/upper clamps, a tangent
direction, duplicate rows and a forced negative pivot. Source, problem,
operator, mask, finite, work, lifecycle and rollback gates are exact.

Routes:

```text
PROJECTOR_DERIVATIVE_PARENT_REJECTED
PROJECTOR_DERIVATIVE_REJECTED
FACE_HESSIAN_REJECTED
FACE_HESSIAN_NONSINGULAR_CANDIDATE
FACE_HESSIAN_REPRESENTATIVE_SELECTION_REQUIRED
```

No Newton solve, multiplier change, line search, ADMM rerun, parameter grid,
timing, candidate, nonlinear step, runtime/GPU integration, new holdout or
production/generalization authority.
