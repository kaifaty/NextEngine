# NSR3-B4E2D7R20R12 failure-mechanism contract

Status: `FROZEN / REPORT-ONLY REPLAY AUTHORIZED`.

## Parent and inputs

- R11 implementation `8104f1b2`, route
  `V3_GENERALIZATION_SOLVER_REJECTED`, semantic
  `d24dd8da48a1e1d9c15b91bc67241fff22e504fad4c01ef4fe82f56e13399b35`;
- failed problem roots `9911ae71...8c51` and `08c636f4...8b62`;
- exact R11 case roots `cd3ee19c...a0ec4` and `7929dffc...29c5d`;
- R8 implementation semantic `afca1c77...374a`.

## Frozen diagnostic

Materialize all v3 inputs to validate parent identity, but execute the unchanged
`al_r20_semismooth_case(problem,true)` only for failed case ordinals 1 and 2.
Require exact reproduction of both R11 case roots. For each final step report:

- iteration, natural-face size, ambiguity count and face/Hessian roots;
- NNQP exact/terminated/KKT flags and named failure;
- source row, value and bound for a sign failure;
- support, entered/removed/transitions/principal-solves counts;
- direction error, primal/dual/complementarity and slope enclosures;
- verified-inverse audit count and roots.

Classify the first predicate without modifying it. The global route is
`V3_FAILURE_MECHANISM_IDENTIFIED` only when both failures reproduce and have a
named, structurally consistent discriminator. Otherwise use
`V3_FAILURE_PARENT_REJECTED`, `V3_FAILURE_REPRODUCTION_REJECTED` or
`V3_FAILURE_MECHANISM_UNRESOLVED`.

No successful-case solve, parameter change, timing, solver repair,
binary64/runtime/GPU integration or production authority is allowed. Success
authorizes only formulation of the smallest cause-specific research contract.

