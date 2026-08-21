# NSR3-B4C3TAR2 -- combined adaptive recovery replay

Status: `FROZEN / IMPLEMENTATION_AUTHORIZED / B4C3TR_BLOCKED`

Parent B4C3A2 selects `CANONICAL_KKT_SCALE_STAGE_LEDGER_CANDIDATE`; its
JSON-without-final-LF SHA-256 is
`8ebee39be668d1b99758040b42936ceeea3dfc6bc6d09888a4a99c35612ed0d9`
and semantic SHA-256 is
`b92dc30aeabe970b517d98a40ca1aaf2c3c109118b0dbac9d97cb419cbbfca9c`.

## Identity

```text
identity        joint-pressure-canonical-balanced-adaptive-r2-recovery-kkt-ledger
representation  f57d88c222a8334206a962a72a814bdd530696ed2767c94fa88910281265579c
ledger policy   b1136c2c3dc7970cbee0ce67b0d129b849ed8957268bb7281063af9e60200e2f
P1              4128b11190366b45aa6511946cb23b46f254445e5a707ff9cdf7e67a2781aaa6
P2              013a83460fabbded819b7d5d9608747f8bc9548c798d71718603d785c238b3c1
```

## Frozen controller

Run all eight P1 and sixteen P2 macro frames. Preserve B4C3TAR's exact failure
classifier: only `KKT_SOLVE:REJECT_LIMIT` is refinable, a failed level breaks
adjacency, and selection requires two adjacent passing levels plus the
unchanged embedded gate. Count every attempted KKT substep and all nonlinear/
spectral work exactly. Retain maximum four levels, accepted `<=192` per frame
and attempted level `<=768`.

Run every candidate stage under B4C3A2 ledger admission. Raw-published and
strict max-scaled residuals remain mandatory diagnostics. Gate compensated
KKT-scale residual at unchanged `1e-9`, correspondence within its forward
bound, and all closure/energy/non-ledger fields unchanged.

## Atomic roots and rollback

Commit only selected fine canonical frames and entries. Require contiguous
global steps and equal counts for frames, legacy ledger entries and policy
ledger entries. Report:

```text
canonical trajectory root
legacy committed-ledger root
KKT-policy committed-ledger root
```

Discarded/failed entries contribute to none of these roots or physical/energy
totals. After one committed macro frame, inject the same failure after two
private substeps; state, canonical roots, legacy/policy roots, global count and
cumulative totals remain bit-exact.

## Unchanged long-horizon gates

Retain B4C3TA/B4C3TAR without refitting:

- complete binary position/velocity/center/q99/kinetic envelope;
- exact P2 `INACTIVE_EXACT` frames 0--13, `FORECAST_ACTIVE` frame 14 and
  `START_ACTIVE` frame 15;
- exact terminal contacts and bounded onset time;
- precontact pressure/reaction and local canonical free-flight bounds;
- P1 center/density/speed, penetration, support/contact and capacity gates;
- cumulative absolute pressure and mechanical publication deltas each within
  `1%` of the independent energy scale;
- compensated KKT ledger residual `<=1e-9`, finite strict diagnostic and exact
  publication impulse bound.

Report strict diagnostic excursions above `1e-9`; they do not fail physical
admission. No discarded level enters energy utilization.

## Negative and historical controls

Require B4C3TAR recovery policy negatives, B4C3A2 ledger policy negatives and
post-commit rollback. Reproduce B4C3A2, B4C3L, B4C3TAR, B4C3TA, B4C3A1,
B4C3Q, B4C3A and B4C2T historical reports at exact hashes. Two complete r2
reports must be byte-identical.

## Decision boundary

PASS selects `CANONICAL_BALANCED_ADAPTIVE_RECOVERY_KKT_LEDGER_CANDIDATE` and
authorizes only B4C3TR fixed canonical reference design. FAIL preserves B4C3A2
and all negative results. No fixed canonical execution, nominal, CUDA, runtime,
schema or production authority is granted.
