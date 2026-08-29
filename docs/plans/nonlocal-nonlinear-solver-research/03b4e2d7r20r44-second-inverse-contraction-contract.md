# NSR3-B4E2D7R20R44 second inverse-contraction contract

Status: `FROZEN / REPORT-ONLY SECOND-TARGET DEFECT AUDIT AUTHORIZED`.

## Parent

- R43 implementation `5b331950`, semantic
  `7a1e4aaaeb6745361b5bd3f4dcab8a85a5c0c582ceaecf74ef399e2d680f8fa8`;
- case/step roots `787558e0...8910` / `4894d0b1...e494` and final
  `VERIFIED_INVERSE_AUDIT_REJECTED` after 600 transitions;
- second material/matrix/inverse/RHS/solution roots `7e90da89...f683` /
  `f8114cbb...be97` / `a4fa12c9...79cf` / `53cb4684...dd32` /
  `1e51c3ad...523a` at dimension 65;
- one original target replacement and one rejected non-target hook invocation.

## Frozen audit

Replay the same R43 trajectory once. Capture the second hook operands only
after all five object roots and dimension match. Do not return a refinement for
them. Require the final R43 case/step/failure and first certificate root exact.

Compute exact dyadic and R38 Dot2 matrices for both `I-AX` and `I-XA`. Report
exact outward and Dot2 `rho`, worst rows, maximum actual/bounded entry errors,
all roots and the inherited classical audit `rho`/bounds. Require all 8450
entries contained and no underflow.

Routes in precedence:

1. `SECOND_INVERSE_PARENT_REJECTED`;
2. `SECOND_INVERSE_CAPTURE_REJECTED`;
3. `SECOND_INVERSE_APPARATUS_REJECTED`;
4. `SECOND_INVERSE_UNDERFLOW_REJECTED`;
5. `SECOND_INVERSE_CONTAINMENT_REJECTED`;
6. `SECOND_INVERSE_RIGHT_NONCONTRACTIVE`;
7. `SECOND_INVERSE_LEFT_NONCONTRACTIVE`;
8. `SECOND_INVERSE_CONTRACTIVE_CANDIDATE`.

R44 performs one R43 trajectory with the already authorized first replacement,
then exact/report-only arithmetic on the existing second inverse columns. It
adds zero factorization, inverse column, principal solve, second correction,
NNQP transition, trial, counterflow replay, state, parameter change or timing.
It cannot certify the second RHS or continue torsion.
