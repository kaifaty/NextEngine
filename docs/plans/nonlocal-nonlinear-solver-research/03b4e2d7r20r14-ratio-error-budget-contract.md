# NSR3-B4E2D7R20R14 ratio error-budget contract

Status: `FROZEN / REPORT-ONLY COUNTERFACTUAL AUTHORIZED`.

## Parent

- R13 implementation `b164e4cd`, semantic
  `0ec0fa4a48138ca63abdde07e0c3997051982aa5833721f8b8a8e692a24eb6eb`;
- exact R12 case/final-step roots preserved;
- both original failures `RATIO_ORDER_AMBIGUOUS`.

## Frozen execution

Observation-only instrumentation may retain `current_error`, `candidate_error`,
the gamma term and perturbation term for the first ambiguous pair. Only after
the original ratio decision is frozen as rejected, invoke
`al_r20_verified_inverse` on the same passive factor, residual, bound and
solution. Recompute the ratio diagnostic with unchanged central vectors,
unchanged `current_error` and only the certified refined `candidate_error`.

Require:

1. exact R13 case and final-step roots;
2. original ratio classification and numbers unchanged;
3. a passing inverse audit with its dimension, column count, `rho`, inverse
   norm and old/refined error;
4. no counterfactual state update or solver decision change;
5. zero successful-case solves and no timing.

Classify `CANDIDATE_REFINEMENT_RESOLVES_ALL`, `RESOLVES_SUBSET`,
`RESOLVES_NONE` or `AUDIT_REJECTED`. Success is report-only and authorizes only
the cause-specific next derivation; it grants no solver, runtime or production
authority.

