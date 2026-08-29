# NSR3-B4E2D7R12 private divided outer continuation evidence

Date: `2026-08-22`

Status: `PASS / PRIVATE_PRESSURE_STATE_CONFIRMED / PRIVATE_ONLY`

## Reproducibility

Implementation commit:
`9f3fd810`.

Two clean Release builds produce byte-identical 5,082,272-byte executables at
SHA `d6bc17704ddecdd366a233664e9b77a249816d1f0324436a4b4bddae0fab4ead`
and Build ID `c04ec12a9447eefc60462a44acdafdd3e66f9a7e`.

Both D7R12 processes exit zero with empty stderr and byte-identical 6,287-byte
stdout reports at SHA
`3c3893b100ae3d0514394a7c265985eeefb545cdbca4a2235636e48fb186aaca`.
The semantic result is
`3f612fbc85c4f39f741bf407584747c075ed468a18a59262d401b883ba7e2d03`;
the private continuation root is
`0a4eb3c04bd0f69fcd9ae930c5dd89dbd6a1c7bcd955828ad40965bd8936b894`.
Raw evidence is under
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r12.fmWiJJ`.

Both builds retain exact parent bytes:

```text
D7R11  3178c5cde71e3859fdf763b448c50da4bba98ce73914321ea40b2cf49c7e06b0
D7R5   7ea5489fa90346f385ce8db08dba445f2e2389e65fdf556a13a17e837ba916d7
```

The post-outer-7 outer/state roots, repeated private solve, work/audit ledger,
formula inputs, pair membership and rollback are exact. No trajectory or
timing ran and no public state was committed.

## Pressure-state result

The continuation completes without rejects:

| Outer | Inner trials / HVP | Positive constraint | Stationarity | Pressure change | Admissible |
|---:|---:|---:|---:|---:|---|
| 8 | `1 / 2` | `2.852232e-10` | `1.065448e-14` | `2.798039e-3 Pa` | no |
| 9 | `1 / 2` | `4.880274e-11` | `4.667335e-15` | `4.787549e-4 Pa` | no |
| 10 | `1 / 2` | `8.350209e-12` | `1.084948e-14` | `8.191555e-5 Pa` | no |
| 11 | `1 / 2` | `1.429967e-12` | `1.045977e-14` | `1.402798e-5 Pa` | provisional |
| 12 | `0 / 0` | `1.429967e-12` | `2.627825e-11` | `1.402798e-5 Pa` | confirmation |
| 13 holdout | `0 / 0` | `1.429967e-12` | `5.254612e-11` | `1.402798e-5 Pa` | yes |

The result is primal-monotone and dual-feasible. Outer 11 and 12 are two
consecutive admissible states, and outer 13 passes the same warm-holdout gate.
The continuation uses four accepted inner trials, zero rejected trials and
eight HVP calls.

## Arithmetic audit

Exactly one trial is accepted by the divided rule but rejected by subtraction
of the rounded absolute totals: outer 11 trial 0. It is the tight D7R10 pair.
The independent binary128 audit resolves its energy decrease positive:

```text
divided reduction       2.8346132192522592e-19
binary128 reduction     2.8347047276408202073e-19
binary128 ULP ratio     3.7679688837640333e17
relative error          3.2281453397500336e-5
pair membership         exact
```

No runtime binary128 state is selected.

## Decision

Select `PRIVATE_PRESSURE_STATE_CONFIRMED`. The evidence rules out the old
inner numerical floor as the remaining cause of the pressure-state failure;
the unchanged gate succeeds once actual reduction is evaluated without
catastrophic cancellation.

This validates only the exact post-outer-7 continuation. D7R13 must start from
the original private state and use one consistent `eta=1e-10` divided inner
for every outer update, with the same audits, confirmation, holdout and
rollback. It must run no physical trajectory and grants no performance or
production authority.

