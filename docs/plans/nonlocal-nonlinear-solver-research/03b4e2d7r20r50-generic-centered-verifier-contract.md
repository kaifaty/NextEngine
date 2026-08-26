# NSR3-B4E2D7R20R50 generic centered-verifier contract

Status: `FROZEN / DEFAULT-OFF ROOT-AGNOSTIC SHADOW POLICY AUTHORIZED`.

## Parent

- R49 implementation `e065274a`, semantic
  `719e0d5067dadc7ab09374e923f3f764ae84e1f509784b0b11339024493df3cc`;
- route `THREE_REFINEMENT_LATER_INVERSE_BOUNDARY`, four distinct calls and
  three replacements, `708` principal solves/transitions;
- first three practical roots `0d3e1914...6849`, `2a271988...472`,
  `dc4449ec...e3a4`;
- fourth tuple roots are frozen only as regression evidence, never as policy
  selectors.

## Frozen policy

Install one private hook for one torsion replay. On every invocation, branch
only on dimension, structural cap and the newly computed certificate. Never
compare matrix/inverse/RHS/solution roots before deciding. For dimension 65 and
fewer than 32 prior replacements, run the existing practical certificate at
fixed depth 16 with target-value checks disabled. Replace if and only if it is
exact, no-underflow, left-contractive, has `65/65` resolved signs and strictly
positive minimum separation. Fail closed otherwise; do not retry.

Record every tuple/certificate root after the decision, bounds, sign counts,
dot counts and input-pair ledger. Require the first three observed tuple and
certificate roots to reproduce R49 only as a post-decision regression. Report
the first generic failure or final trajectory boundary. Clear the hook and
regress R49/R48.

Frozen per-certificate work at dimension 65/depth 16 is 5525 compensated dots
and 360,490 input pairs. Cap 32 implies at most 176,800 dots and 11,535,680
input pairs.

Routes in precedence:

1. `GENERIC_VERIFIER_PARENT_REJECTED`;
2. `GENERIC_VERIFIER_REGRESSION_REJECTED`;
3. `GENERIC_VERIFIER_APPARATUS_REJECTED`;
4. `GENERIC_VERIFIER_WORK_REJECTED`;
5. `GENERIC_VERIFIER_CAP_REJECTED`;
6. `GENERIC_VERIFIER_LATER_INVERSE_BOUNDARY`;
7. `GENERIC_VERIFIER_LATER_BOUNDARY`;
8. `GENERIC_VERIFIER_TORSION_CANDIDATE`.

R50 changes no factorization, inverse-column generation, NNQP formula, ratio,
tolerance, semismooth cap, globalization, trial, counterflow, runtime state or
timing. It grants no production authority and does not add `prodK`.
