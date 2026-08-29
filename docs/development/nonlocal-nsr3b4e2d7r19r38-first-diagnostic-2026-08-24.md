# NSR3-B4E2D7R19R38 first diagnostic -- 2026-08-24

Status: `STATE-ATTESTATION REPAIRED / CLEAN PROOF PENDING`.

The first strict-f64 replay executes all 15 frozen pair passes and all 12
direction lanes, but exits `MEMORY_STATE_REJECTED`. Parent, identity, dense,
workspace, direction, line, work and rollback facts pass.

The failed check rebuilt the checkpoint root from the fresh source JVP. R37's
checkpoint root is intentionally formed from its maintained linear recurrence;
the separately recomputed response is allowed a relative defect `<=1e-12` and
is not expected to be bit-identical. In the observed state the resulting
objective differs only in terminal binary64 bits. Requiring both root identity
and fresh-bit identity contradicted the frozen bounded-defect rule.

The repair separates the two contract facts:

- rebuild the exact checkpoint root from captured maintained response and
  residual;
- require the fresh JVP response defect `<=1e-12` independently;
- start every no-update lane from the exact captured residual.

No beta, restart, projection, line, work or selection rule changes. Failed
stdout-with-LF SHA-256 is
`22093b1d5bbbbd17a7c5dba95b2607472f87f78711317e452a4a48f506ae6037`;
semantic result SHA-256 is
`511a8d71de4df9661f3ab0ae45b0d253a03804d82f2e0112950cc031980ce7dd`.
The FAIL cannot select a formula.

## Reclosed diagnostic

After separating maintained-root and fresh-defect attestation, the same
strict-f64 build passes every hard gate and selects
`HAGER_ZHANG_DIRECTION_CANDIDATE`. HZ and DY-HS+ are valid, do not restart and
strictly beat steepest at all three states; frozen precedence selects HZ.
PRP+ passes raw descent but fails projected descent at every state and safely
restarts to the exact steepest lane.

All 12 exact lines pass with 15 pair passes, zero accepted updates and exact
rollback. Reclosed stdout-with-LF SHA-256 is
`7f574289496e70fb76886381c2b1460227f15a3f6d07214b0772dabf7247abb1`;
semantic result SHA-256 is
`cc030c6559643952faf70dd3545f7f62a01133527ad5fdf8427159ed994ab7d3`.
Clean proof runs remain pending.
