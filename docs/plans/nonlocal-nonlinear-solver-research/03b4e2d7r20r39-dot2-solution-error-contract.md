# NSR3-B4E2D7R20R39 Dot2 solution-error contract

Status: `FROZEN / REPORT-ONLY PASSIVE-SOLUTION AUDIT AUTHORIZED`.

## Parent

- R38 implementation `3260382a`, semantic
  `6898dcbe1b610a9a4cf38276d5af4e5f79ee63fe095315ddafb86b6deaa706ae`;
- inverse certificate root `cd14942d...3181`, compensated residual root
  `d388080b...30f8` and `rho_bound=4.443635206380081e-3`;
- selected inverse audit dimension/columns `65 / 65`, root
  `5fe4d71e...5e7c`, cheap error `7.109566050753423e38`.

## Frozen audit

Replay R38 once. Extend the selected observer capture with the exact RHS,
represented solution, stored solve residual/bound and cheap error already used
by the parent verified-inverse call. Require both existing capture call roots,
all R37 payload roots and the complete R38 certificate root. Add no solve.

Compute `||X||_inf` with one-ULP-upward absolute row sums and

```text
inverse_norm_bound = up(||X||_inf / (1-rho_bound)).
```

For each of 65 rows evaluate `[b_i,A_i,:] dot [1,-x]` with unchanged R38 Dot2
and bound. Independently compute the exact dyadic residual, require entrywise
containment and form its exact root. Let `residual_inf_bound` be the maximum
`up(abs(dot2)+bound)` and

```text
solution_error_bound = up(inverse_norm_bound * residual_inf_bound).
```

Classify each represented solution component as negative when
`x_i + error < 0`, positive when `x_i - error > 0`, otherwise unresolved.
Report counts, minimum absolute component/separation, old and new errors,
inverse/residual/error roots and exact-to-bound ratios.

Routes in precedence:

1. `DOT2_SOLUTION_PARENT_REJECTED`;
2. `DOT2_SOLUTION_CAPTURE_REJECTED`;
3. `DOT2_SOLUTION_APPARATUS_REJECTED`;
4. `DOT2_SOLUTION_UNDERFLOW_REJECTED`;
5. `DOT2_SOLUTION_CONTAINMENT_REJECTED`;
6. `DOT2_SOLUTION_SIGN_UNRESOLVED`;
7. `DOT2_SOLUTION_ERROR_CANDIDATE` when all 65 signs resolve.

R39 executes one parent torsion replay and its existing 128 inverse columns,
65 compensated residual dots and one exact residual oracle. It adds zero
factorization, inverse column, solve, NNQP transition/decision, trial, state,
counterflow work, tolerance/cap/Armijo change or timing. It cannot install the
candidate or continue the trajectory.
