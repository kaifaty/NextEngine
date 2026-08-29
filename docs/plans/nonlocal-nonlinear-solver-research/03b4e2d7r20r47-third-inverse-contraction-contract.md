# NSR3-B4E2D7R20R47 third inverse-contraction contract

Status: `FROZEN / REPORT-ONLY THIRD DEFECT AUDIT AUTHORIZED`.

## Parent

- R46 implementation `fb1c8bfb`, semantic
  `e1c904c730afcdad36a24a4c0e430687b65f77ae81bef59bd8a4529e574b1ec5`;
- route `TWO_REFINEMENT_LATER_INVERSE_BOUNDARY`, hook cardinality `3/1/1/1`,
  principal solves/transitions `602/602`;
- final case/step roots `83aacc9a...c3a8c` / `3e94e52d...acb87`;
- third tuple/matrix/inverse/RHS/solution roots `06d5be83...0e62` /
  `79244e37...d477` / `be9cb782...8d26` / `d4e62af1...4397` /
  `fb226f2c...e1d0`.

## Frozen execution

Materialize torsion once and replay the exact R46 prefix with one private hook.
It applies the original depth-eight and second depth-16 certificates once each,
then captures the third 65-row tuple without replacing it. Require exact match,
replacement and capture cardinalities and the frozen R46 final case/step roots.

For the captured candidate evaluate exact-dyadic and R38 Dot2/upward
certificates for all entries of `I-A*X` and `I-X*A`. Require `4225/4225`
containment on each side, no underflow and report both infinity norms, worst
rows and inherited audit. Clear the hook and regress R46 and R45.

Routes in precedence:

1. `THIRD_INVERSE_PARENT_REJECTED`;
2. `THIRD_INVERSE_CAPTURE_REJECTED`;
3. `THIRD_INVERSE_APPARATUS_REJECTED`;
4. `THIRD_INVERSE_UNDERFLOW_REJECTED`;
5. `THIRD_INVERSE_CONTAINMENT_REJECTED`;
6. `THIRD_INVERSE_RIGHT_NONCONTRACTIVE`;
7. `THIRD_INVERSE_LEFT_NONCONTRACTIVE`;
8. `THIRD_INVERSE_CONTRACTIVE_CANDIDATE`.

R47 adds no RHS audit, centered correction, replacement, factorization, inverse
column, NNQP decision, retry, counterflow replay, parameter, state or timing.
