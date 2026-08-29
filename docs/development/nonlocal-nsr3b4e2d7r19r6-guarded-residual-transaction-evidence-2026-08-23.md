# NSR3-B4E2D7R19R6 guarded-residual transaction evidence

Date: `2026-08-23`

Status: `PASS / NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED / PRIVATE FIRST SUBSTEP ONLY`

## Outcome

The guarded-residual completion policy integrates correctly into one complete
private first-substep transaction. The legacy direct-model policy remains
byte-exact, R2 trials `0..4` remain exact, and the R5 sixth trial is reproduced
at every frozen state, model, divided, precision and radius anchor.

The accepted sixth trial is no longer the terminal boundary. The candidate
continues to nine accepted trials and a second outer update. A later trust
solve reaches the base recurrence cap, fails the frozen guard and stops at
`STRUCTURAL_BUDGET_GUARDED_HVP_DENIED`. The selected report route is therefore
`NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED`.

This is a successful bounded classification, not a confirmed solver state or
production admission.

## Parent correspondence

The complete parent chain remains exact:

```text
R5 stdout SHA-256  fe0a75877bf539e37e11955f25cebb05821dcbacf7cb50eb120913367f664294
R5 semantic        ae751940bf195fdf70aab9f1a5df6ce986decfc9c4f3c5fbeb26bf95d43e8880
R4 retained        true
R3 retained        true
R2 stdout SHA-256  3dad88903f5f619d540587e805b35d63e2ef8c848e53e1ab87786c9e90587ba0
legacy policy      direct model HVP, byte-exact
```

The guarded-residual policy is explicit and used only by the separate R6
candidate transaction with binary64-owned precision membership.

## First-six anchors

R2 trials `0..4` compare exact over their complete legacy trial records,
including positions, steps, binary64 scalars, acceptance/radius decisions and
precision evidence.

Trial `5` reproduces the R5 certificate:

```text
current root        54bafbf48d0798438fd9baad9fb91e67c7b5cf6b12e37c4bb1d49694384ddf8a
step root           74a9b58726d5d0279699498c41a70fb0e199f45e531d11befd2bc2dfe692d2bd
trial root          932ce178a6238025c8de6ea907d9966638fa0f5abef7780a15fb9c8171f67ea2
predicted bits      0x3bc27dd9b2871ea7
divided bits        0x3bc27dd8dc16d400
ratio bits          0x3feffffe8ce92225
precision root      a58caa2c1cb83c23dbcc15b8d2daf243de7749e6e692a92369141bbfd741f8db
radius before/after 0x3f8999999999999a
recurrence HVPs     33
model HVPs          0
residual model      used
accepted            true
```

## Transaction result

```text
transaction root    bf4e9dad4b6b8690905d07b3609dc6f0483205276b199ebcc3a354ff5a09cc5d
outer updates       2
accepted trials     9
rejected trials     0
recorded trials     9
confirmed           false
failure             INNER:STRUCTURAL_BUDGET_GUARDED_HVP_DENIED
route               NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED
```

All nine accepted trials have finite, repeat-exact pre-cancelled divided
reductions and finite long-double audits. No sign contradiction, unresolved
precision result or binary128 candidate-effect audit occurs.

## Guard and model ownership

```text
guarded-policy trust solves  10
recurrence HVPs             219
direct model HVPs             8
residual model uses            1
guard attempts                 2
guard eligible                 1
guard HVPs                     1
guard converged                1
guard denied                   1
total HVPs                   227
```

Every ordinary completed solve uses one direct model HVP and no residual
model. The one guarded completed solve uses HVP 33, converges there and uses
`r_final-g` with zero model HVP. The later denied solve receives no HVP 33 and
forms no trial. Recurrence plus direct-model work equals the structural total
exactly.

## Work and safety

```text
outer cap / used          16 / 2
trials/update cap         16
absolute HVP/step cap     33
total HVP cap / used      512 / 227
workspace cap / used      288 / 12
precision cap / used      64 / 9
workspace releases        12
maximum live workspaces   2
all-pair candidate calls  0
mass                      750 kg exact
```

Static binding, finite state, divided repeat, precision ledger, lifecycle,
mass, rollback and route precedence all pass. Since the solver is not
confirmed, boundary and impulse admission are not reached. No public state,
second substep, macro, trajectory or timing lane is executed.

## Reproducibility

Implementation commit:
`805793c4` (`research: integrate guarded residual transaction`).

Two clean Release builds:

- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r6-a.4egfvm`;
- `/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r19r6-b.PyyVPd`.

Both binaries are `5,901,792` bytes, have SHA-256
`7dcd7f723d71b3d3e56eb6a8bf03b25112127797a1af68fa717ac2f3047be6c1`
and GNU build ID `58b91873502a8dc77a24e306ebd0c39186d5a4e0`.

Fresh process outputs:

- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r6-a.mDBVZ0`;
- `/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r19r6-b.hywHSA`.

Both exit `0`, emit empty stderr and reproduce:

```text
stdout-with-LF bytes  3,091
stdout SHA-256        67dfb6781a5840e228b63f3524bf4999ca6d1d9b650f8c461117575a8172611c
semantic result       a2687bac0ba18b58ca4903047de6645c389174512452dad82c5e0d46320f5811
route                 NORMALIZED_NOMINAL_STRUCTURAL_WATCHDOG_EXHAUSTED
```

## Decision

Retain the narrow guarded-residual integration as proven private research,
but do not promote it or loosen the guard. The first barrier is solved; the
new exact boundary is the first later solve that reaches 32 recurrence HVPs
and fails the frozen guard after nine accepted trials.

Research the denied recurrence separately before changing any policy. The
next experiment should capture that solve passively, reproduce its live
prefix, continue only an offline replay under a bounded diagnostic cap and
identify which guard clause failed and whether the recurrence subsequently
converges. It must not form another trial, continue the transaction, change a
cap or run performance work.
