# NSR3-B4E2D7R13 full private divided transaction evidence

Date: `2026-08-22`

Status: `PASS / FULL_PRIVATE_PRESSURE_STATE_CONFIRMED / PRIVATE_ONLY`

## Reproducibility

Implementation commit:
`bb5d6f5d`.

Two clean Release builds produce byte-identical 5,099,920-byte executables at
SHA `8df2505251b2ce81dea6f21387cc0b777ee32f3ec15ea41ee71b77e8b19336d2`
and Build ID `3ba4b6f376fff525dec3025c3bd2d8fee908bc18`.

Both D7R13 processes exit zero with empty stderr and byte-identical
15,854-byte stdout reports at SHA
`514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de`.
The semantic result is
`37c0828f6f3621f87560aa447aa0ad5f4a7391f9f6134bad46b6f13858b3ecd8`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r13.Tj0lwR`.

D7R12 remains exact at stdout SHA
`3c3893b100ae3d0514394a7c265985eeefb545cdbca4a2235636e48fb186aaca`.
The active in-process repeat, inactive transaction, work/audit ledgers,
formula profile, pair membership and active/inactive rollback all pass.

## Full active transaction

The transaction starts from the original compressed `0.99` state with zero
multipliers. One consistent `eta=1e-10` divided inner is used from outer 0.

```text
active transaction root   4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8
accepted inner trials     19
rejected inner trials      0
HVP calls                 38
candidate-effect accepts   4
confirmation             outer 11 / 12
warm holdout             outer 13, admissible
```

The path is finite, primal-monotone and dual-feasible. The confirmed state is:

```text
positive constraint          1.4290790772975015e-12
stationarity                 2.6249280607908859e-11
complementarity              8.2598128597937164e-13
absolute multiplier change   1.7524082185360612e-9 J
equivalent pressure change   1.401926574828849e-5 Pa
position update              0 dx
```

The same bounds pass again at the outer-13 holdout.

## Candidate-effect audits

Four raw-total rejections become divided acceptances, at outer 1, 4, 9 and
10. Every independent binary128 sign resolves positive, every pair membership
agrees and the maximum relative magnitude error is `6.50867e-6`, or about
`0.000651%`. No runtime binary128 state is selected.

## Inactive control

The inactive full transaction root is
`6df37c6ae522de96d3a300abefcbf035af55e04d9f6d7bf77b6cedbea5f1ab1e`.
Outer 0/1 confirm and outer 2 holds out with zero inner trials, HVPs, particle
movement and multiplier change.

## Performance interpretation

The exact active work count is now a valid structural baseline. It is not a
wall-clock result and not a real-time claim. The full tighter solve uses 19
accepted trials/38 HVPs; optimization should target repeated pair/evaluation/
HVP work while preserving this transaction root and gate outcome.

## Decision

Select `FULL_PRIVATE_PRESSURE_STATE_CONFIRMED`. The consistent full private
solver is mathematically viable on the tiny dense oracle. The next research
stage must map this solver into one nominal Dam-frame shadow transaction and
freeze its state/work/error boundaries before implementation. No multi-step
trajectory, timing, runtime publication or production use is authorized.

