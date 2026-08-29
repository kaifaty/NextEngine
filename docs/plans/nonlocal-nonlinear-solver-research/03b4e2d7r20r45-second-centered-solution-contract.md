# NSR3-B4E2D7R20R45 second centered-solution contract

Status: `FROZEN / REPORT-ONLY SECOND SOLUTION AUDIT AUTHORIZED`.

## Parent

- R44 implementation `84c09a12`, semantic
  `cdb334f4981ad13c0bcd5a337bd39054e21a6e76b832543141785c02c53ebc08`;
- exact/Dot2 right roots `9ac1d0e0...d214` / `283e803a...c691`,
  `rho=0.00502173`;
- exact/Dot2 left roots `c743e1ff...2d39` / `f2a7457c...56b9`,
  `rho=0.177559`;
- second matrix/inverse/RHS/solution roots remain the frozen R43 tuple.

## Frozen audit

Replay/capture the exact R44 second tuple with only the inherited first depth-
eight replacement. Reproduce both contraction certificates. Compute the exact
dyadic second RHS residual and all practical Dot2 residual intervals. Compute
exact/practical `z=Xr` and require all entries contained without underflow.

Execute exactly 32 represented center recurrences. At fixed depths
`1,2,4,8,16,32`, use the R41 centered checkpoint construction with independent
exact dyadic `g=z+C*y-y` and compensated `x+y` containment. Report checkpoint
roots, radii, positive/negative/unresolved counts, first resolving depth,
minimum separation and arithmetic-floor behavior.

Routes in precedence:

1. `SECOND_CENTER_PARENT_REJECTED`;
2. `SECOND_CENTER_CAPTURE_REJECTED`;
3. `SECOND_CENTER_APPARATUS_REJECTED`;
4. `SECOND_CENTER_UNDERFLOW_REJECTED`;
5. `SECOND_CENTER_CONTAINMENT_REJECTED`;
6. `SECOND_CENTER_LEFT_NONCONTRACTIVE`;
7. `SECOND_CENTER_SIGN_UNRESOLVED`;
8. `SECOND_CENTER_SOLUTION_CANDIDATE`.

R45 adds only exact/Dot2 offline arithmetic: the existing defect dots, 65
residual dots, 65 `z` dots, `32x65` center dots and six 65-row residual/center
audits. It adds zero factorization, inverse column, principal solve, second
correction, NNQP decision/transition, trial, counterflow replay, state,
parameter change or timing.
