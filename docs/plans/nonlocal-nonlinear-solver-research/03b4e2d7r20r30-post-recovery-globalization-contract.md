# NSR3-B4E2D7R20R30 post-recovery globalization contract

Status: `FROZEN / REPORT-ONLY TERMINAL AUDIT AUTHORIZED`.

## Parent

- R29 implementation `685e940c`, semantic
  `3c4f5d5149d1510880f2d65b4c122e51fa441a6102d43f492a5c4e25930bba7b`;
- candidate shear root `3f4798c8...e473` with exact one-shot R28 recovery;
- 21 accepted iterations followed by iteration-22 globalization rejection;
- frozen certificate tolerance remains exactly `2^-70`.

## Frozen audit

Replay only exact R29 shear. Reproduce all 22 step roots and the iteration-21
KKT metric root. Report every certification predicate, exact tolerance ratio,
gap and gap bound. For iteration 22, report face/support/NNQP roots, slope and
bound, and all inherited trial alphas, exact R19 margins, changed scalar lists,
mask/ball roots, dual increase and KKT tuples.

Classify, in priority, parent/reproduction failure, any stable strict-negative
margin as `POST_RECOVERY_SAME_FACE_REJECTION`, all trials crossing as
`POST_RECOVERY_LINE_ENVELOPE_EXHAUSTED`, multiple non-nested event regimes as
`POST_RECOVERY_MIXED_EVENT_REGIMES`, any unsigned margin as
`POST_RECOVERY_PRECISION_BOUNDARY`, otherwise unresolved.

No new line point, event derivation, recovery, tolerance/cap change, state
update, trajectory continuation, timing, runtime/GPU, generalization or
production authority.
