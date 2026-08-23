# NSR3-B4E2D7R19R12 total-budget atomic completion evidence

Date: `2026-08-23`

Status: `PASS / TOTAL_BUDGET_ATOMIC_ACCEPTANCE_CANDIDATE / SHADOW ONLY`

## Outcome

The solve interrupted at live total HVP `512` can complete as one healthy,
ordinary trial. The exact R11 recurrence needs ten HVPs beyond the captured
14-HVP prefix, then the existing direct model needs one HVP. The resulting
trial is a precision-confirmed acceptance candidate with unchanged trust
radius.

This closes the narrow question asked by R12. It does not authorize raising
the live cap: the shadow does not prove that the accepted position is also a
safe completed inner/outer transaction boundary.

## Parent and target closure

```text
R12 identity SHA-256   17ea852a62651deb0140d0dbdc3b10012d7304a394b620ceb64ae51da1d17894
R11 stdout SHA-256     bd05f7ac3ca76425dbfda1e43882f48adcb4c83addf1fe4ddeaacd37ca8a8efe
R11 semantic           be9cc63d2376f4c560498aa9d28c1d44070a40e1ec412a30cbe735fba89305ef
boundary root          002f3b63423de2e0db8715b599e064043502c3881ba89cb55aaf7394bee850df
live recurrence root   a900c9452fa173f1c60fdf7c670de747653742348340056f05d7336cbaaa974c
first-14 prefix root   b3165d58a442deccd262812fdafb4d385337a7dc81ceddf2200dde890bdc5768
offline trace root     0fb21740e7137234fe17332d451b0767b78ae3fc636a98a829ea175c482458e0
step root              e63fd50c794d9c6679e9a3d63beb5105e05fbe0a56f7bd128465a9ba69993983
```

R10/R9/R8/R7/R6/R5/R2 remain transitively exact through the unchanged R11
parent. Refactoring R11 to expose an internal capture preserves its public
stdout exactly.

## Atomic work accounting

```text
live recurrence prefix          14 HVP
offline recurrence completion   24 HVP
post-boundary recurrence debt   10 HVP
ordinary direct model debt       1 HVP
live total before              512 HVP
projected live total           523 HVP
```

The control physically replays all 24 recurrence HVPs to prove first-14
correspondence. Only the ten-HVP tail is new relative to the captured live
state. The direct model is the ordinary `H(step)` policy; no residual model,
grace, preconditioner or second solve is used.

## Model, reduction and acceptance

```text
step norm                 7.6237421059197321e-10
gradient dot step        -2.2237395646868757e-18
predicted reduction       1.1118697823434404e-18
raw reduction             1.1118831861352297e-18
inherited divided         1.1118831890403455e-18
precancelled divided      1.1118697980219184e-18
rho                       1.0000000141010019
trust radius before/after 0.0125 / 0.0125
```

Raw, inherited-divided and precancelled decisions all accept. Therefore
precancellation has no candidate effect at this boundary. Both inherited and
selected repeat checks are exact. The radius owner remains `NONE`.

The binary64-owned long-double audit is finite and resolves positive at root
`701b0ebe63e44b72a940816c5459819c839ebdf57f5aeaf636ec569273002433`.
There is no topology mismatch. Binary128 is correctly skipped because the
selected reduction does not change the acceptance decision.

## Work, lifecycle and rollback

R12 builds/releases exactly two new sparse workspaces, with maximum live two.
It performs two selected divided evaluations (`681440` total union visits),
one long-double audit, zero binary128 audits and zero all-pair candidate calls.

Current, predicted, gradient, dual, theta, radius, recurrence, step, parent
transaction and budget identities are unchanged after classification. The
trial root is
`58dadefd7426e9f80188b694557a02efcaf005dd67abfff5e337b90f214be5a8`;
it is not committed. No later solve, transaction continuation, substep,
macro, trajectory, timing or public mutation runs.

## Reproducibility

Contract commit:
`ca55cd11` (`research: freeze total budget atomic completion`).

Implementation commit:
`85021498` (`research: classify total budget atomic completion`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r12-a.9pM95R`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r12-b.INO8Gp`.

Both binaries are `6,115,992` bytes, have SHA-256
`77f799f40711d070418471dd049a4bff74587d473759b10738faed6a5b6ee95d`
and GNU build ID `c1015c152e98ebb1f2ab07d86d07ee402fa403a9`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r12-a.t6L9Xu`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r12-b.c2LgIW`.

Both exit `0`, emit empty stderr and reproduce byte-exact:

```text
stdout-with-LF bytes  3,455
stdout SHA-256        e57ba96aca2c3114ea2c9c10fa2728d8c91dd7ac7969010f381a245219175cee
semantic result       3e08a082789053fb5209c4622b391db647328ee9f6582612fd936d8b9ec30f18
route                 TOTAL_BUDGET_ATOMIC_ACCEPTANCE_CANDIDATE
```

## Decision

Retain R12 as evidence that a bounded in-flight completion is numerically
useful and costs eleven HVPs at this boundary. Do not raise the live total cap
yet.

Research D7R19R13 as a rollback-only post-acceptance boundary discriminator.
It must reproduce exact R12, classify stationarity at the accepted trial and
determine whether the existing inner and outer update can finish without a
new trust solve. It may reuse already-formed state and perform non-HVP
evaluation needed for the boundary classification, but may not admit another
trial/solve, continue later transaction work, change the cap, commit state or
run timing.

