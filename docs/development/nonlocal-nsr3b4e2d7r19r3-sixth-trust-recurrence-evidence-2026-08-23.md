# NSR3-B4E2D7R19R3 sixth trust-solve recurrence evidence

Date: `2026-08-23`

Status: `PASS / SIXTH_TRUST_FORCING_CONVERGED / REPLAY ONLY / NO STATE`

## Outcome

The exact failed sixth D7R19R2 trust solve is a one-HVP convergence near miss.
Its unpreconditioned Steihaug recurrence remains finite, positive-curvature
and strictly interior through the historical 32-HVP prefix, then satisfies the
unchanged dimensionless forcing test on HVP 33.

This rejects negative curvature, trust-boundary truncation, gross
ill-conditioning and observable loss of CG orthogonality/conjugacy as the
local cause. It does not yet form the sixth trial: the current solver performs
one separate `H(step)` model HVP after the recurrence, so simply setting the
old combined per-step counter to `33` would still stop before trial formation.

No state is selected or confirmed, and no production cap changes.

## Exact parent and target

D7R19R2 remains byte-exact:

```text
stdout SHA-256   3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0
semantic result f806858bba9d3cb5ae781a270ccccbda96c53ce995cca1cc5799e6af36599c95
```

The replay derives its target from accepted trial `4`, never from copied
coordinates:

```text
frame zero       0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7
transaction      027dc6d474c87172a7848f71c33f30e0527e3915248b7b53ac077e0d946f2145
binary64 trace   6e7a30213e967e910ab3245321850d5633f521327de20ce6fd8f699685179343
current state    54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a
predicted state  36112dde1e0b274c5b9216f4818b82977a0c80dc257478c111b0f0a9390d2d7e
trust radius     0x3f8999999999999a = 0.0125
```

## Live-prefix equivalence

The passive sink attached to the historical solve captures exactly 32 HVPs
and the original `STRUCTURAL_BUDGET_HVP_PER_STEP` stop. The independently
rebuilt 128-cap offline lane reproduces all first-32 point, residual,
direction, HVP and scalar projections exactly:

```text
live prefix root     f478832923673956bc98d8067fff9bdeb5c3dab109c0a8e239ad12c6844d60dd
offline prefix root  f478832923673956bc98d8067fff9bdeb5c3dab109c0a8e239ad12c6844d60dd
live trace root      66fbcd92dbd7e9b93cff3194809350c156354033e830b9c76f4fdbce5e7b154a
offline trace root   d2610dc6221ccf520ad8ea1b39976fc6270c2157e8215249cfd30e3715f0aaf0
```

Instrumentation therefore observes the historical recurrence without
changing it, and the continuation starts from the identical Krylov state.

## Convergence mechanism

The frozen forcing value and residuals are:

```text
eta                              2.51880524249163e-05
initial residual norm            3.13003046762642e-10
ratio before HVP 32 (index 31)   4.33038165101276e-05
ratio after HVP 32               3.02723280342496e-05
ratio after HVP 33               2.22605555078985e-05
ratio-after-32 / eta             1.20185266901795
```

Thus HVP 32 misses the forcing threshold by about `20.19%`; HVP 33 crosses it.
The last eight recorded residual ratios decrease monotonically.

The step is nowhere near the trust boundary:

```text
point norm after HVP 32   8.58697215187691e-11
point norm after HVP 33   8.58697219852236e-11
trust radius              1.25e-02
point/radius after HVP 33 6.86957775881789e-09
```

All 33 curvatures are positive. Maximum adjacent residual-orthogonality error
is `1.48324376745445e-14`; maximum adjacent `H`-conjugacy error is
`1.27493999599138e-14`. There is no observable finite-precision recurrence
breakdown.

The 33-dimensional CG/Lanczos tridiagonal gives:

```text
minimum Ritz value       1.01621325396845
maximum Ritz value       36.7130096907994
Ritz condition estimate  36.1272691016676
```

This is moderate conditioning, consistent with an iteration-budget boundary
rather than a need for a preconditioner at this exact state.

## Work and safety

The offline lane consumes exactly `33` HVPs, one workspace build/release and
zero precision audits, trials, acceptances or all-pair calls. Static binding,
workspace lifecycle and rollback all pass. No candidate nominal substep,
macro, trajectory or timing lane runs.

## Reproducibility

Implementation commit:
`c37e613e50fca85c8253e28546ba65b26c71e06f`
(`research: diagnose sixth trust recurrence`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r3-a.Eh6AsG`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r3-b.hqWKOp`.

Both binaries are `5,816,960` bytes, have SHA-256
`6ec97b2f5cd93c6321fd40f54f9f73a98292c87108a446cbd309f9637e469e07`
and GNU build ID `15fdccb82d5e42eadf42ec06f77a2f2a626b5596`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r3-a.02HkNK`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r3-b.BIPTBy`.

Both processes exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  34,108
stdout SHA-256        a1038937496f31ed64008eb8e366763e2da9875e3935194955d68604a7b23771
semantic result       156782481d783abc500c1a1b888b693d158f8cd30503412d41285f4c725df6d8
route                 SIXTH_TRUST_FORCING_CONVERGED
```

## Decision

Do not research a preconditioner for this exact barrier and do not blindly
replace the production cap with 128. Research/freeze the smallest cap-policy
reclosure that distinguishes recurrence HVPs from the final model HVP.

The next discriminator should compare:

1. a bounded near-convergence grace for the 33rd recurrence HVP plus the
   existing direct model HVP;
2. reuse of an `H(step)` image accumulated from already computed Krylov images,
   validated against one direct oracle HVP before eliminating work.

It must remain on the same first substep, retain R2/R3 bytes, and first prove
the sixth trial's model/reduction/acceptance correspondence before any full
transaction or cap-policy selection.
