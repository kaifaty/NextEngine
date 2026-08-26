# NSR3-B4E2D7R20R52 counterflow centered-slope contract

Status: `FROZEN / REPORT-ONLY SUCCESSFUL-AUDIT REFINEMENT AUTHORIZED`.

## Parent

- R51 implementation `21cc6988`, semantic
  `48df3b2650abb5ff9338c6926c60754494418cfd7037ef4c43851b8c7946adca`;
- counterflow problem/case roots `62b1dd7d...7a03` /
  `7e2735e1...c5d3`, route `DIRECTION_REJECTED`, 701 transitions;
- R36 natural face size 66, nominal slope `1.8493164062e-19`, inherited
  direction error `8.8529495396e-3` and bound `2.0123909084e-14`.

## Frozen audit

Run the original counterflow case once with `al_r20_verified_inverse_capture`
and no solution-refinement hook. Require its exact R51 case/final-step and R36
natural-face/current/direction roots. Require capture's last audit root equal
the final NNQP inverse audit and that it passed classically.

Evaluate one depth-16 practical centered certificate with target-value checks
disabled. Require exact/no-underflow, left contraction, all captured solution
signs resolved and positive separation. Recompose the R36 slope bound with the
new certificate error only; preserve all other terms/order and report old/new
ratios and roots. Do not apply the refined solution.

Routes in precedence:

1. `COUNTERFLOW_CENTER_PARENT_REJECTED`;
2. `COUNTERFLOW_CENTER_CAPTURE_REJECTED`;
3. `COUNTERFLOW_CENTER_APPARATUS_REJECTED`;
4. `COUNTERFLOW_CENTER_UNDERFLOW_REJECTED`;
5. `COUNTERFLOW_CENTER_LEFT_NONCONTRACTIVE`;
6. `COUNTERFLOW_CENTER_SIGN_UNRESOLVED`;
7. `COUNTERFLOW_CENTER_SLOPE_STILL_UNRESOLVED`;
8. `COUNTERFLOW_CENTERED_SLOPE_CANDIDATE`.

Regress R51/R50. R52 adds no inverse column, factorization, NNQP decision,
direction, trial, tolerance, cap, state, timing, runtime or production change.
