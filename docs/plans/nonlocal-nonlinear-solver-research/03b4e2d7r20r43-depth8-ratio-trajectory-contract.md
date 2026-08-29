# NSR3-B4E2D7R20R43 depth-eight ratio trajectory contract

Status: `FROZEN / DEFAULT-OFF DEPTH-EIGHT TRAJECTORY AUTHORIZED`.

## Parent

- R42 implementation `3e021608`, semantic
  `e435daf5e3ba43a6a51d4a014f9ed2b6d10dee4096b6f4ac5e9a942b53cd9a41`;
- practical depth-four root `6c8c6c97...393f` and one exact replacement;
- final case/step roots `6d946714...49ed` / `bb88cf3f...f5ff`;
- ratio rows `61/13`, values `1.51752e-19/3.27822e-19`, bounds
  `1.65656e-19/1.65656e-19` and candidate/current errors
  `0.142140/1.71568e-4`;
- R41 depth-eight root `80cb60a8...b0d`, radius `4.5753008642e-14`,
  signs `24/41/0`, zero conflicts.

## Frozen implementation

Preserve the R42 hook and all default-null behavior. Parameterize only its
bounded center depth; R42 remains exactly depth four and byte-exact. Add a
separate R43 target-bound hook invocation at exactly depth eight. Require the
practical certificate to reproduce R41's exact `rho`, depth-eight radius,
`24/41/0` signs, positive uniform separation, no underflow and all structural
dot counts.

Replay torsion once. Require one target match and one replacement. Report
whether the row-61/row-13 ordering resolves, every resulting ratio budget,
transition count and the first subsequent exact case/step/failure route. Do not
retry a later failure. Clear the hook and regress R42/R41/R40/R39.

Routes in precedence:

1. `DEPTH8_RATIO_PARENT_REJECTED`;
2. `DEPTH8_RATIO_HOOK_REJECTED`;
3. `DEPTH8_RATIO_TARGET_CARDINALITY_REJECTED`;
4. `DEPTH8_RATIO_NOT_CONSUMED`;
5. `DEPTH8_RATIO_STILL_UNRESOLVED` when the frozen row pair remains the final
   boundary;
6. `DEPTH8_RATIO_LATER_BOUNDARY`;
7. `DEPTH8_RATIO_TORSION_CANDIDATE` if torsion certifies.

R43 adds four center iterations and their compensated arithmetic only. It adds
no factorization, inverse column, tolerance, cap, ratio formula, trial,
counterflow replay or production-state change. One local passive replacement
and its unchanged NNQP transitions are the sole trajectory mutation.
