# NSR3-B4E2D7R20R53 counterflow direct-center contract

Status: `FROZEN / REPORT-ONLY FINAL DIRECT-SOLVE AUDIT AUTHORIZED`.

## Parent

- R51 semantic `48df3b26...adca`, counterflow case
  `7e2735e1...c5d3`, final step `c7a33746...1590`;
- R52 exact FAIL semantic `86d2dcd4...17a6`, zero verified-inverse calls,
  zero final inverse-audit records, support 66/66 and 86 affine-shadow updates;
- R36 current/direction/slope roots `e6221363...26e` /
  `f09a8af6...821e` / `57b1b93b...f02f`.

## Frozen apparatus

Add one default-null direct-solution observer at the existing
`negative.empty()` commit point in `al_r20_nnqp_solve`. It may copy the factor,
principal RHS, direct solution, solve residual/bound and cheap error only. With
the observer enabled, replay counterflow once and select the last direct solve
by execution order, never by an object root.

Require exact R51/R36 reproduction, final natural-face/support dimension 66,
captured solution equal the final NNQP direction componentwise, captured cheap
error equal the final direction error, and no verified-inverse call during the
parent replay.

Run `al_r20_verified_inverse` exactly once on the captured factor only to build
66 inverse columns. It may not refactor the matrix or alter the parent work.
Then run one root-agnostic depth-16 practical centered certificate. Frozen
certificate ledger for dimension 66 is 5,676 compensated dots and 376,002 dot
input pairs. Require exact/no-underflow, left contraction, 66 positive, zero
negative/unresolved and strictly positive minimum separation.

Use the certificate's refined solution, not the old direct center, to construct
one shadow global direction. Recompute the R36 slope and ordered bound with the
unchanged residual, natural face and arithmetic ordering. Do not mutate NNQP or
evaluate any trial.

## Routes in precedence

1. `COUNTERFLOW_DIRECT_PARENT_REJECTED`;
2. `COUNTERFLOW_DIRECT_CAPTURE_REJECTED`;
3. `COUNTERFLOW_DIRECT_INVERSE_REJECTED`;
4. `COUNTERFLOW_DIRECT_CERTIFICATE_REJECTED`;
5. `COUNTERFLOW_DIRECT_UNDERFLOW_REJECTED`;
6. `COUNTERFLOW_DIRECT_LEFT_NONCONTRACTIVE`;
7. `COUNTERFLOW_DIRECT_FACE_CHANGED`;
8. `COUNTERFLOW_DIRECT_SLOPE_STILL_UNRESOLVED`;
9. `COUNTERFLOW_DIRECT_CENTERED_SLOPE_CANDIDATE`.

Regress R52, R51 and R50. R53 adds one report-only 66-column inverse build and
the frozen compensated-dot ledger. It adds no factorization, applied
correction, direction decision, trial, tolerance, cap, state, timing, runtime
or production change.
