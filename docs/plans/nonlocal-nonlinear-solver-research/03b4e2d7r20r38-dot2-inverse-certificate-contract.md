# NSR3-B4E2D7R20R38 Dot2 inverse-certificate contract

Status: `FROZEN / REPORT-ONLY COMPENSATED CERTIFICATE AUTHORIZED`.

## Parent

- R37 implementation `9d01cf30`, semantic
  `e9e61c01d6ae9ebe07d7ef707dfb5121f352b1e2df85f8e2a5160d9015a598ff`;
- matrix/factor/inverse roots `45e2c92b...9bef` / `1c159edb...bbf0` /
  `8ec902ee...5803`;
- exact residual/audit roots `d6bd19f5...dcfa` / `eb9d2fb3...fefe`;
- exact outward `rho=4.443635206380081e-3`, worst row 25.

## Frozen audit

Replay the exact R37 torsion capture once. Preserve both existing call roots,
all selected payload roots and the exact dyadic residual root.

Implement error-free `TwoSum` and FMA `TwoProduct`, then ORO `Dot2` for each
length-66 residual dot. Require error-free identities on predeclared normal
controls and reject any nonzero subnormal input, product, remainder or
accumulator intermediate. Use exact `u=2^-112`; construct `gamma_66` and the
Proposition-5.5 rearranged bound

```text
error <= (u*abs(dot2) + gamma_66^2*sum_abs_products) / (1-u)
```

with one-ULP-upward nonnegative arithmetic. For every entry require the exact
R37 dyadic residual inside `dot2 +/- error`. Sum `abs(dot2)+error` upward per
row; report all roots, worst row, maximum entry error, maximum exact-to-bound
ratio and `rho_bound`.

Routes in precedence:

1. `DOT2_PARENT_REJECTED`;
2. `DOT2_APPARATUS_REJECTED`;
3. `DOT2_UNDERFLOW_REJECTED`;
4. `DOT2_CONTAINMENT_REJECTED`;
5. `DOT2_RESIDUAL_BOUND_NONCONTRACTIVE` for `rho_bound >= 1`;
6. `DOT2_INVERSE_CERTIFICATE_CANDIDATE` for `rho_bound < 1`.

R38 executes one parent torsion replay with only its already existing
factorizations/128 inverse columns. It adds 4225 compensated residual dots and
the exact oracle repeat, but zero new factorization, inverse column, original
solution refinement, NNQP decision, trial, state, tolerance/cap/Armijo change,
counterflow work or timing. It cannot replace the current verifier, continue
torsion, enter runtime/GPU code or claim production readiness.
