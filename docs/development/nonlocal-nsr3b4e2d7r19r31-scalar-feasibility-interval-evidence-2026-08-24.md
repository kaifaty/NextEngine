# NSR3-B4E2D7R19R31 scalar-feasibility interval evidence

Date: `2026-08-24`

Status: `PASS / INEQUALITY_ACTIVE_SET_REFORMULATION_REQUIRED / REPORT ONLY`

## Outcome

R31 proves that no scalar fraction of the exact R30 direction can satisfy all
frozen R29 linearized inequalities simultaneously.

```text
required lower alpha                1.0000019514939671
safe upper alpha                    6.7350725332513859e-14
interval gap                       -1.0000019514938998
selected nondecreasing rows         0
zero-margin inactive rows           0
binary64 lower repairs              0
binary64 upper repairs              0
```

The lower bound is already greater than the allowed full step. The first
inactive crossing occurs about thirteen orders of magnitude earlier. Both
bounds agree directly with separately evaluated binary64 predicates after
binary128 ratio ordering.

At the required lower bound all 1,420 originally selected rows are repaired,
but 450 inactive rows become positive and their violation norm is
`369470.6151915277`. At the safe upper bound no inactive row crosses, but all
1,420 selected rows remain positive and the selected violation reduction is
only `6.7390537594747002e-14`.

Therefore line search or scalar damping along the R30 preimage is not a
globalization remedy for this state. A later method must change the direction
while accounting for all inequalities and a step bound inside the subproblem.
R31 does not select which active-set, interior-point or trust-region method to
use.

## Controls and authority

All four analytic controls pass: nonempty feasible interval, inactive blocker,
selected nondecreasing impossibility and canonical `+0.0` zero-margin bound.
All nine route cases, exact R30 parent bytes, source/partition roots and
rollback pass. R31 adds zero pair passes, HVPs, model evaluations, trials,
precision audits or outer updates.

The first dense-control signed-zero failure and its formula-preserving repair
are retained in the
[first-diagnostic record](nonlocal-nsr3b4e2d7r19r31-first-diagnostic-2026-08-24.md).

No correction or nonlinear moved-state evaluation occurs. No floor, timing,
runtime or production authority is claimed.

## Reproducibility

Research/contract commit: `6c3fd702`.

Implementation commit: `3b479fbb`.

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r31-a.BEA0MR`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r31-b.EcDHuk`.

Both binaries are `7,037,152` bytes, have SHA-256
`47dd737f1c5e2bf001815513afe264fe9ed1f7549a866cef5542f5c04e32859f`
and GNU build ID `d1d4170bed38d5b594a2b667ce6cffeb390982e4`.

Fresh one-process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r31-a.p4TUzO`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r31-b.avHjNK`.

Both exit `0`, have empty stderr and reproduce `3,336` stdout bytes exactly:

```text
stdout SHA-256  67adbee532f615b9014e93cb44fccd053bc887c54f5d2ffcd76a19f2c771f3cb
semantic        d57f2712573d8e8dda52bdf44a11f40e2a6cbca03b1ac30e07c13b20af8aaa98
route           INEQUALITY_ACTIVE_SET_REFORMULATION_REQUIRED
```

These are correctness/reproducibility runs, not timing or performance
measurements.

## Next action

Research a separately frozen R32 matrix-free inequality feasibility normal
step with an explicit trust bound and all-row active-set closure. Do not apply
R30, execute another outer, choose a QP implementation from name alone,
evaluate a moved nonlinear state or claim production readiness.
