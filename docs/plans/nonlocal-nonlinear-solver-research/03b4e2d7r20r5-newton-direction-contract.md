# NSR3-B4E2D7R20R5 Newton direction contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Parent

- R20R4 implementation `23a25a17`, semantic
  `18d3d90b0f3994bb4fac9d4cddeef961a93830c88c82818766de0f47c6209a4d`;
- route `FACE_HESSIAN_NONSINGULAR_CANDIDATE`;
- exact v2 manifest/preflight/problem roots; cases are development data;
- selected joint box-ball generalized derivative and maximum-diagonal
  pivoted-Cholesky rank policy unchanged.

## Frozen one-step diagnostic

At `lambda=0`, select exactly the rows whose binary128 dual gradient exceeds
its outward rounding bound. Build the raw-coordinate matrix
`H=A_I J_D A_I^T`, factor it without permutation or regularization, and solve
`H*delta=r_I`. Require factor reconstruction and solve residual bounds.

Classify every direction component with an outward error bound. Do not clip,
project, prune, regularize or alter a negative component. Require a resolvably
positive `r_I^T delta` before line search.

For a dual-feasible direction, evaluate exactly 21 trials in fixed order
`alpha=2^-k`, `k=0..20`, stopping evaluation after the first certified Armijo
acceptance. The Armijo coefficient is exactly `2^-10`. The projected dual
value, slope and their binary128 error bounds must certify ascent. Record all
box/ball mask changes and frozen KKT metrics at the accepted candidate.

Quiet cases perform zero factorization, solve and line-search trials and must
remain KKT-certified at `lambda=0`. Excited cases perform at most one Newton
solve and one accepted candidate. Analytic SPD/indefinite solve, ascent and
Armijo rejection controls are mandatory.

Routes:

```text
NEWTON_DIRECTION_PARENT_REJECTED
NEWTON_DIRECTION_REJECTED
PROJECTED_REPRESENTATIVE_SELECTION_REQUIRED
GLOBALIZED_NEWTON_RELINEARIZATION_REQUIRED
UNIT_NEWTON_DEVELOPMENT_CANDIDATE
```

No second Newton iteration, active-set pruning, diagonal regularization,
parameter grid, timing, runtime/GPU integration, new holdout or
production/generalization authority.
