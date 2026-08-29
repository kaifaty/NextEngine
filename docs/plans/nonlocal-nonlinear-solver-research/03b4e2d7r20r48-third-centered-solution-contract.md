# NSR3-B4E2D7R20R48 third centered-solution contract

Status: `FROZEN / REPORT-ONLY THIRD CENTER AUDIT AUTHORIZED`.

## Parent

- R47 implementation `6165ebf5`, semantic
  `7ce17f9542e0b024aa98b3102ae052b2da2e86797becbf4fbf841bd1d4091d83`;
- exact/Dot2 right roots `79a5c348...5a5a` / `024cb61d...2b76`,
  `rho=0.00590785`;
- exact/Dot2 left roots `9454068d...e8b9` / `0992eca3...1c1b`,
  `rho=0.241759`;
- third matrix/inverse/RHS/solution roots are frozen by R46/R47.

## Frozen execution

Replay the exact R46 prefix and capture the third 65-row tuple without a third
replacement. Reproduce all four R47 defect roots. Compute the exact third
solution residual, compensated residual samples, exact `Xr`, interval `I-XA`
and `z=Xr`. Starting from zero, generate exactly 32 compensated fixed-point
centers and independently certify depths `1,2,4,8,16,32`.

At every checkpoint report exact-containment counts, radius, signs, minimum
separation, residual and center bounds and roots. Require no underflow, exact
containment and no conflict with every original sign that was already
certified. Regress R47 and R46.

Routes in precedence:

1. `THIRD_CENTER_PARENT_REJECTED`;
2. `THIRD_CENTER_CAPTURE_REJECTED`;
3. `THIRD_CENTER_APPARATUS_REJECTED`;
4. `THIRD_CENTER_UNDERFLOW_REJECTED`;
5. `THIRD_CENTER_CONTAINMENT_REJECTED`;
6. `THIRD_CENTER_SIGN_CONFLICT`;
7. `THIRD_CENTER_SIGN_UNRESOLVED`;
8. `THIRD_CENTER_SOLUTION_CANDIDATE`.

R48 performs no third correction, NNQP decision, factorization, inverse column,
retry, counterflow replay, state, parameter or timing work.
