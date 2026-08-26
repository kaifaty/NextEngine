# NSR3-B4E2D7R20R36 v4 failure-budget contract

Status: `FROZEN / REPORT-ONLY TWO-CASE AUDIT AUTHORIZED`.

## Parent

- R35 implementation `1f69b8bb`, semantic
  `59c32c19863321eeee98838ef907ee02630c37ca032bb2846d655ae6165037b3`;
- torsion problem/case roots `5584d517...d3ad` / `a543bc06...1d6d`;
- counterflow problem/case roots `62b1dd7d...7a03` / `7e2735e1...c5d3`;
- exact failing step roots `d274e20e...4cd9` / `c7a33746...1590`.

## Frozen audit

Replay only those two exact R35 cases with the unchanged generalized policy.
Require every prefix and final root above.

For torsion final NNQP, expose all existing `inverse_audits` and their complete
column/norm certificates. Classify factor/column failure, nonfinite result,
`rho>=1`, refined-error failure or other exact rejection. Do not execute an
additional inverse or change the audit.

For counterflow, reconstruct `current`, `lambda`, `direction` at iteration 10
and report:

```text
residual_bound_term = sum rb_i * (abs(d_i) + direction_error)
direction_error_term = sum abs(r_i) * direction_error
rounding_term = gamma(16*rows+512) * sum abs(r_i*d_i)
```

Require their binary128 ordered sum to equal the stored step slope bound.
Report the three terms, nominal slope, NNQP-local slope/bound, direction error,
dominance ratios and all roots. Classify residual-bound, direction-error,
rounding, nonpositive-model or unresolved dominance.

No new solve/trial/state, inverse refinement, tolerance/cap/Armijo change,
source edit, retry, timing, runtime/GPU or production authority.
