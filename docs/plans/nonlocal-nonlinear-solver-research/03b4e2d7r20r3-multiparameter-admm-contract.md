# NSR3-B4E2D7R20R3 multiparameter ADMM development contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Parent

- implementation `bab26969`;
- exact R20 v2 manifest/preflight/problem roots;
- R20R2 semantic `929a6676182dc7702a8950343833e56faad4ac46e4784fda1c67f169f6eff08c`;
- route `GLOBAL_ADMM_ORACLE_UNRESOLVED`;
- supported certified at 16,384; both filled cases unresolved at 65,536;
- candidate iterations, old-oracle reruns and state mutation zero.

The two filled cases are development fixtures from this point forward, not
blind holdouts.

## Frozen probe

Implement the two-penalty binary128 consensus equations and MpSRA update in
the R20R3 research. Start `rho_density=rho_domain=1`. Update at exact multiples
of 64 from the unscaled-dual/prox differences over that same 64-iteration
window. Use factor-ten zero cases, absolute bounds `[2^-20,2^20]`, rescale
scaled duals exactly, and audit every replacement factor before use.

Checkpoints are `0,2^4,2^6,...,2^14`; first full R20 `2^-70` KKT/gap
certificate owns the case. No tolerance-selected stop. Quiet cases remain
cycle-zero, perform no factor/update work and reproduce R20R2 roots.

Dense matrix residual audits execute only immediately after factor creation
and at checkpoints. They must use the same binary128 forward bound as R20R2.
Every report exposes signed gap, gap bound, KKT tuple, both consensus blocks,
both penalty histories, factor roots and structural work.

## Routes and authority

Reject parent, source/problem, penalty, factor, solve, KKT, work and lifecycle
failures in that order. Terminal development routes are
`MULTIPARAMETER_ADMM_DEVELOPMENT_CERTIFIED`,
`MULTIPARAMETER_ADMM_ACTIVE_SET_REFINEMENT_REQUIRED` and
`MULTIPARAMETER_ADMM_UNSTABLE`.

One execution only. No parameter grid, cap extension, candidate solve, new
holdout construction before result, wall timing, nonlinear step, runtime/GPU
integration or production/generalization authority.
