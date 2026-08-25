# NSR3-B4E2D7R19R65 equal-work composed-dual completion contract

Date: `2026-08-25`

Status: `FROZEN / PRIVATE 20-OUTER IMPLEMENTATION AUTHORIZED`.

Parent: v10 `PASS / COMPOSED_DUAL_PATH_REFERENCE_RETAINED`, semantic
`f497ab0a61e846d95ec5d86cc4de0328a01b99b8d622fadf17a431aa4af7d32b`.

## Work proof

At most one row residual and one row update scan occur per owned row in an
outer. With 33 fixed transposes, 17 fixed actions and one committed audit, a
conservative extra-outer bound is

```text
33*535,588 + 18*605,144 + 2*535,588 = 29,638,172 terms.
```

Starting from v10's 467,826,594 terms:

```text
four extra <= 586,379,282 < 596,971,680
five extra <= 616,017,454 > 596,971,680.
```

Exactly four additional outers are authorized. The analytic bound is a work
gate, not an estimate of wall time.

## Hard gates

1. Exact v10 parent semantic and exact prefix records/state/work through outer
   16.
2. Exactly 20 Hildreth/PCG/projected-dual outer blocks; no residual stop.
3. Identical 15-HVP PCG and 16-value dyadic completed-dual selection policy.
4. Checkpoints 1/2/4/8/16/20 with fresh direct dual, physical model,
   recurrence, projection, stationarity, complementarity and all-row audits.
5. Every line retains strict fixed-density descent, positive normal-reference
   reduction and positive composed-dual ascent; candidate/commit predictions
   satisfy inherited bounds.
6. Actual sparse structural terms remain strictly below 596,971,680.
   Completed-dual local vector terms and projections remain separate.
7. Acceleration requires final maximum raw `<1.1823169686944491e-9` and
   projected-gradient norm `<9.5382120867736572e-9` simultaneously.
8. Parent/source rollback, dense controls and route precedence remain exact.
9. No outer 21, tolerance fitting, timing, nonlinear trial, runtime mutation or
   production authority.

## Routes

```text
all gates and strict dual/KKT/work dominance
    -> EQUAL_WORK_COMPOSED_DUAL_ACCELERATION_CANDIDATE

all correctness/work gates, no strict dominance
    -> EQUAL_WORK_DEPTH_EXHAUSTED

prefix, transaction, bound or correspondence failure
    -> EQUAL_WORK_COMPOSED_DUAL_REJECTED
```

`EQUAL_WORK_DEPTH_EXHAUSTED` closes further depth on this policy. A PASS remains
one-fixture exploratory evidence, not production approval.

Rationale:
[equal-work research](../../development/nonlocal-nsr3b4e2d7r19r65-equal-work-dual-completion-research-2026-08-25.md).
