# NSR3-B4E2D7R20R2 global binary128 ADMM oracle contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Immutable parent

- R20 oracle implementation `5be5b059` and semantic
  `885dc7bdb628e4d435b3cd6966c87b6866dd9ca33fec3d8b9718b8477328c60f`;
- exact v2 manifest, preflight and eight problem roots;
- old route `ORACLE_UNRESOLVED`, candidate iterations zero;
- explicit feasibility witness `s=0` for every case;
- R20R1 phase-I withdrawn and never executed.

## Algorithm

For `B_i=A_i/sigma_i`, `d_i=c_i/sigma_i`, use binary128 consensus ADMM with
`rho=1`, zero `s,y,z`, `u=d`, `v=0` and the fixed update equations in the R20R2
research. Construct `H=2I+B^T*B` once in stable row/component order and solve
each primal step with a binary128 dense Cholesky factor.

Audit factor positivity, finite arithmetic, a gamma-bounded reconstruction of
`H`, and every solved-system residual. Use the exact binary128 box-ball
projector. Checkpoints are exactly `0, 2^4, 2^6, ..., 2^16`; select the first
full KKT/gap certificate at `2^-70`. Quiet cases may certify at zero and must
perform no factorization or ADMM iteration.

At checkpoints map `lambda_i=y_i/sigma_i` and reuse the direct/completed dual,
primal, projected-dual, complementarity, stationarity and gap definitions of
R20. Also report normalized ADMM consensus residuals and exact dense/sparse
work. A certificate requires both ADMM finite/linear-solve gates and the full
R20 oracle KKT/gap tuple.

## Controls and routes

Controls cover analytic dense feasible projections, row-scale invariance,
nonpositive-orthant and box-ball prox, Cholesky diagonal corruption,
transpose/action consistency, multiplier conversion, weak duality, source and
problem roots, cycle-zero ownership, cap, lifecycle and rollback.

Routes, in precedence order:

```text
GLOBAL_ORACLE_PARENT_REJECTED
GLOBAL_ORACLE_MATRIX_REJECTED
GLOBAL_ORACLE_FACTOR_REJECTED
GLOBAL_ORACLE_ITERATION_REJECTED
GLOBAL_ORACLE_KKT_REJECTED
GLOBAL_ADMM_ORACLE_CERTIFIED
GLOBAL_ADMM_ORACLE_UNRESOLVED
```

## Prohibitions

One frozen execution only. No adaptive rho, depth/tolerance grid, old Dykstra
rerun, candidate solve, wall timing, nonlinear step, GPU/runtime integration,
state mutation or production authority.
