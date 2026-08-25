# NSR3-B4E2D7R20R22 event-side trial contract

Status: `FROZEN / REPORT-ONLY NEXT-REPRESENTABLE AUDIT AUTHORIZED`.

## Parent

- R21 implementation `0c261dd4`, semantic
  `3ff49117f8ad70f55c345e4a9f9459bd11bf994125267baf04891c89ec857bca`;
- R20 semantic `4ca89c1f...1de5` and exact seven transition cells;
- seven unique zero-lower-bound event roots with correct observed scalar.

## Frozen audit

At each R21 event evaluate exactly:

```text
alpha_root
alpha_after = nextafterq(alpha_root, +HUGE_VALQ)
```

using the unchanged affine multiplier path, projector, dual objective, R19
Armijo formula and KKT metrics. Require nonnegative multipliers, finite/exact
evaluation and unchanged R21/R20/shear roots. The `alpha_after` projector must
differ from the current projector at exactly the predicted scalar and retain
ball activity; any additional change rejects the representable-side claim.

Report root/after alpha, ULP separation, masks, Armijo margins, dual increase
lower and KKT tuple. Classify:

1. any wrong side/additional mask/ball change:
   `NEXT_REPRESENTABLE_FACE_REJECTED`;
2. correct new face but any negative Armijo:
   `EVENT_SIDE_ARMIJO_REJECTED`;
3. exactly five of seven accepted and they equal R20's offset brackets:
   `EVENT_SIDE_OFFSET_SUBSET`;
4. all seven accepted: `EVENT_SIDE_TRIAL_CANDIDATE`;
5. otherwise: `EVENT_SIDE_UNRESOLVED`.

The two shadow evaluations per bracket are diagnostic work only. No trial is
inserted into the solver, no state/fallback/offset is selected, and cap,
tolerances, timing, runtime/GPU and production authority remain unchanged.

