# NSR3-B4E2D7R16 sparse precision transaction evidence

Date: `2026-08-22`

Status: `PASS / NOMINAL_TRANSACTION_BACKEND_CONFIRMED`

Implementation commit: `1297a1f8`.

## Result

D7R16 removes the remaining dense precision path from the complete sparse AL
transaction and selects:

```text
NOMINAL_TRANSACTION_BACKEND_CONFIRMED
```

The command executes no nominal nonlinear solve, integration, publication,
trajectory or timing measurement.

## Complete transaction equivalence

The identity-bound sparse transaction reproduces the inherited D7R13 roots:

```text
active        4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8
active repeat 4d6b8c58c2db672bea5fcae0a4d5fb077cc8587398540128bcb09b6a75400ef8
inactive      6df37c6ae522de96d3a300abefcbf035af55e04d9f6d7bf77b6cedbea5f1ab1e
```

Every outer and holdout inner root is exact. The active transaction retains
`19` accepted trials and four candidate-effect audits; its repeat is exact.
The inactive transaction remains still.

## Sparse precision path

Every accepted inner trial now evaluates long-double current/trial,
naive/compensated energies over one canonical `0.04h` pair union. All `38`
active-plus-repeat audits match the dense oracle in every field, including
energy components, pair membership, half-horizon/horizon margins, reductions,
ULP ratios and resolved sign classification.

The real candidate-effect path now uses sparse binary128. Both active runs
reproduce the four inherited roots:

```text
3f6b74612d212d48ee40ba1bc2fffd27a23a20cc1d2c390cf2e1b59194a4a8fc
a11e56103bc1ea0b3a88b4b82354a0e66240db3e9057f98a1a0f987951cfe0aa
e06edef57dda0cf7cabc75de35365ea57432f19d4706879fc7086afb40865e51
b0db278089eb109d19879de3433fb2d69c6160f755fee572f75fdda86c584312
```

Structural work for the two active and one inactive transactions is:

| Fact | Value |
|---|---:|
| accepted long-double audits | `38` |
| candidate-effect binary128 audits | `8` |
| current/trial superset builds | `92` |
| union-candidate records | `41,400` |
| long-double evaluation pair visits | `136,800` |
| binary128 evaluation pair visits | `28,800` |
| candidate all-pair calls | `0` |

This replaces the hidden dense accepted-trial path. It is structural evidence,
not a speed or wall-time claim.

## Static support and lifecycle

One identity-bound static index serves every inner, outer and holdout state.
Across all three transactions:

| Fact | Value |
|---|---:|
| static-index builds | `1` |
| support canonicalizations | `1` |
| AL workspace builds / releases | `100 / 100` |
| maximum live workspaces | `2` |

A one-bit identity mutation is rejected before any workspace, superset or
precision evaluation is created.

## Nominal predictor ledger

The read-only aligned Dam predictor confirms:

| Fact | Value |
|---|---:|
| frame-zero root | `0d567ba5512ba237a48e5e0b828a670a398f1bf23a35ac269729cad535f374d7` |
| substep `dt` bits | `0x3f0c01c01c01c01c` |
| lower-`y` clamps | `400` |
| free samples | `5,600` |
| other clamps | `0` |
| predictor contact impulse, `y` | `0.026201923076922928 N*s` |
| gravity impulse, `y` | `-0.39302884615380085 N*s` |

The state remains byte-exact. D7R17 must ledger these impulses separately
from the AL pressure correction.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r16-final.YvWNyU`.

Two independent clean GCC 15.2 Release builds produce identical
`5,275,432`-byte executables:

```text
SHA-256  8ec1a43f6fe792d1a7edb9bc85f8182d6044295167e689468a3865304b6c8e4d
Build ID ef15d6372e27506a9e48fa14bb47e83c4ae53a77
```

Two fresh processes from each build exit zero with empty stderr. All four emit
the same `2,478`-byte stdout:

```text
stdout SHA-256  4c537f706dee3941808f0c44c1b2db30dd79254bcbbd42c9ac0dc92aee3f8cd5
semantic result 47976f826fa5c7c16209f5e3e8a0829e443ec90d3bd225fb61e73c1d506ff156
```

Direct regressions preserve:

```text
D7R15 stdout dda8399f35556ada9bf38cfb1445c324f6f5b4c0193a59e2d4c9c4ea5f57f5cf
D7R13 stdout 514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de
```

## Decision

D7R16 authorizes research/freeze of D7R17: exactly one aligned nominal Dam
substep shadow with bounded nonlinear work, explicit contact/pressure/support
impulse ledger, residual, conservation, penetration, watchdog and rollback
gates.

No macro frame, trajectory, timing lane, public state, runtime-wide precision,
parallel/GPU path, physics mutation or production authority is granted.
