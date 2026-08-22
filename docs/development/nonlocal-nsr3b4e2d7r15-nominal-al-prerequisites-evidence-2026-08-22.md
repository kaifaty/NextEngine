# NSR3-B4E2D7R15 nominal AL prerequisites evidence

Date: `2026-08-22`

Status: `PASS / NOMINAL_AL_PREREQUISITES_CONFIRMED`

Implementation commit: `4d717a33`.

## Result

D7R15 closes all three prerequisites required before a meaningful nominal Dam
AL substep. The selected route is:

```text
NOMINAL_AL_PREREQUISITES_CONFIRMED
```

The command performs no nominal prediction, nonlinear solve, integration,
publication, trajectory or timing measurement.

## Explicit time step

The sparse candidate path now receives one finite positive binary64 `dt`
through inertia, gradient, HVP, scaled stationarity, direct and divided
reductions and the independent long-double and binary128 audits.

| Fact | Result |
|---|---:|
| legacy `1/240` bits | `0x3f71111111111111` |
| aligned `1/(240*78)` bits | `0x3f0c01c01c01c01c` |
| forced doubled-`dt` bits | `0x3f1c01c01c01c01c` |
| dense/sparse evaluation and gradient | binary64 exact |
| dense/sparse HVP | binary64 exact |
| dense/sparse stationarity | binary64 exact |
| dense/sparse direct reduction | binary64 exact |
| dense/sparse divided reduction | binary64 exact |
| long-double and binary128 `dt` propagation | exact |
| zero/invalid `dt` rejection | exact |

The forced mutation changes every selected time-dependent binary64 control.
At legacy `dt`, the complete sparse active and inactive roots remain:

```text
active   4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8
inactive 6df37c6ae522de96d3a300abefcbf035af55e04d9f6d7bf77b6cedbea5f1ab1e
```

## Sparse binary128 sign oracle

The accepted-sign oracle now evaluates a canonical sorted union of the
current/trial `0.04h` supersets. Compact-support membership is recomputed in
binary128; runtime positions, multipliers and accepted state remain binary64.

All four inherited D7R13 roots are exact:

```text
3f6b74612d212d48ee40ba1bc2fffd27a23a20cc1d2c390cf2e1b59194a4a8fc
a11e56103bc1ea0b3a88b4b82354a0e66240db3e9057f98a1a0f987951cfe0aa
e06edef57dda0cf7cabc75de35365ea57432f19d4706879fc7086afb40865e51
b0db278089eb109d19879de3433fb2d69c6160f755fee572f75fdda86c584312
```

The four inherited audits plus one explicit-substep audit use `10` superset
builds, `4,500` union-candidate records and `18,000` evaluation pair visits.
All-pair candidate calls are zero. This is structural work evidence, not a
wall-time claim.

## Identity-bound static support

Tiny active/inactive topology and AL coefficients are exact between the
standalone and identity-bound builders. A mutated support identity is rejected
before evaluation. The nominal command builds the fixed support index exactly
once and does not recanonicalize the `16,384` support samples per AL workspace.

The decoded nominal workspace reproduces:

| Fact | Value |
|---|---:|
| frame-zero raw-bit root | `0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7` |
| static-index identity | `a2d97ab6f26383d826366eba2a3d4392f5ef9610dda87e89509e93bd9daf61e8` |
| pair root | `fb2b8f8b4c0227cf5d8a7a43ce518ed72b2e5d5cda31fb8e8c17a5727c24ba13` |
| pairs / directed / maximum degree | `342502 / 611520 / 120` |
| active pressure centres | `0` |
| nominal static-index builds | `1` |

All workspaces are released, maximum live workspace count is `2`, rollback is
exact and public commit count remains zero.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r15-final.Qtmn9W`.

Two independent clean GCC 15.2 Release builds produce identical
`5,223,056`-byte executables:

```text
SHA-256  c1d539518acf404d009a5649ceacb7561712bc8bce5ed18fca5fdbd7eb6adc68
Build ID 58f88ee02ee88f79a3f5f5de6bea124c5e9cf210
```

Two fresh processes from each build exit zero with empty stderr. All four emit
the same `2,221`-byte stdout:

```text
stdout SHA-256  dda8399f35556ada9bf38cfb1445c324f6f5b4c0193a59e2d4c9c4ea5f57f5cf
semantic result a207d671c41648ebcc4c89d73dc86acd2569b6ca8b150faa6584b850534c2858
```

Direct checks from the clean build preserve:

```text
D7R14 stdout 88d83b6ec6e659f405595b22e13df363bef2ae4c1f99e4838b54f06a585ad1d3
D7R13 stdout 514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

Both have empty stderr.

## Decision

D7R15 grants one exact nominal AL prerequisite set. D7R16 may research and
freeze one aligned nominal Dam substep shadow with predeclared nonlinear-work,
residual, conservation, watchdog and rollback gates.

No macro frame, trajectory, timing lane, runtime binary128, parallel/GPU path,
public state, physics mutation or production authority is granted.
