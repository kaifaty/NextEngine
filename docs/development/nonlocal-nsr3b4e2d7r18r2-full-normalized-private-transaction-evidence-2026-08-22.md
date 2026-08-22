# NSR3-B4E2D7R18R2 full normalized private transaction evidence

Date: `2026-08-22`

Status: `PASS / NORMALIZED_INNER_POLICY_STILL_INSUFFICIENT / D7R19_BLOCKED`

Implementation commit: `71f652f427d9f2d3aad705f08e1413b7dc6bc17a`.

## Result

The complete normalized private discriminator executes reproducibly and
selects:

```text
NORMALIZED_INNER_POLICY_STILL_INSUFFICIENT
```

This is a successful fail-closed classification, not a confirmed normalized
solver. D7R19, nominal execution and production claims remain blocked.

## Reproducibility

Raw evidence:
`/home/kaifaty/.cache/nextengine/run-nonlocal-b4e2d7r18r2.kqdYl3`.

Two independent clean GCC 15.2 Release builds produce byte-identical
5,516,496-byte executables:

```text
SHA-256  35b773ff4d15f8910672ffb2543462063466a7ae1b1f7ba41c8567c412595c5a
Build ID 10bace2ac87ff8d824fb03ea9aeaded43a6d3286
```

Build directories:

```text
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r2-a.lB4UBp
/home/kaifaty/.cache/nextengine/build-nonlocal-b4e2d7r18r2-b.SQ8Ofy
```

One fresh R2 process from each build exits zero with empty stderr and emits
the same 10,996-byte stdout:

```text
stdout SHA-256 3659eac888c22eae5bcf7ae8c5f8a426bcbc12bd24bd3320c23be0c176815d77
semantic result 69223a86a9a889a5c85fd11b824a4bd178efe505907d6f3877b5a42b8fd0f392
```

Each clean binary directly reproduces D7R13's 15,854-byte stdout at
`514ea1925a85d398a948a2dcbc319689116114a02335a599e51d6703202c18de`.
The R2 process also reproduces complete D7R18R1 parent bytes internally.

## What passes

Both independently derived physical profiles use exact normalized
`theta=0x3fc5cccccccccccd`. Reference repeat and reference/aligned complete
transaction roots are byte-identical:

```text
active   c76bea9f1bff57c16e27a08c4fc51dad7029af1730c11ac80e568ebe112601a5
inactive be761a3c07bc4a7f558486c5e9bc5d3b80e1cc0ba2982e1698e7f5a3d24c2585
```

The inactive profile confirms exactly at outer `0/1` and holds out at outer
`2`, with zero accepted/rejected trials, zero HVPs, no movement and unchanged
`u`. All four invalid profile/dual/binding/budget cases reject before
workspace, pair, precision or HVP work. Rollback is exact.

Across five transactions:

| Fact | Value |
|---|---:|
| outer updates | `12` |
| inner trials | `42` |
| HVP calls | `84` |
| workspace builds/releases | `63/63` |
| maximum live workspaces | `2` |
| long-double accepted audits | `21` |
| binary128 candidate-effect audits | `0` |
| pair-union candidates | `18,900` |
| extended pair visits | `75,600` |
| candidate all-pair calls | `0` |

All accepted audits are finite with exact pair membership. Six of seven
active accepted signs resolve positive in long double; one is unresolved and
none resolves negative. No candidate-effect acceptance occurs before the
failure.

## First failing boundary

The active solve completes outer `0`, then fails at outer `1` with
`MINIMUM_TRUST_RADIUS`. It has seven accepted and seven rejected trials,
28 HVPs and final normalized stationarity
`2.7191714554716553e-10`, above the frozen `1e-10` limit.

The first causal divergence from D7R13 is outer `1`, trial `2`:

| Quantity | Normalized R2 |
|---|---:|
| stationarity before | `4.3508730711255175e-10` |
| step norm | `1.0282158542755083e-11` |
| predicted reduction | `3.163331278531801e-22` |
| raw subtraction | `-3.0715532249809066e-19` |
| current normalized divided reduction | `-3.0714726551614456e-19` |
| ratio | `-970.9614279118468` |

D7R13 accepts the corresponding trial with dimensional divided reduction
`2.277579377453211e-18`; dividing by the exact objective scale
`M/dt^2=7200` gives `3.1633046909072375e-22`. Its independent binary128
oracle gives `3.1633252799233112e-22` after the same normalization. R2's
predicted reduction is within `1.90e-6` relative of that oracle, while the
current normalized divided reduction has the opposite sign and about
`971.96x` relative error.

The formula therefore loses the signal while subtracting already rounded
`current_active` and `trial_active` values. R1's single larger trial did not
exercise this cancellation boundary. This is not evidence against the
normalized objective or cross-scale identity; it is evidence that the D7R10
precancellation construction was not fully carried into normalized divided
differencing.

## Decision

Preserve R2 and do not rerun it with the same normalized divided formula,
looser stationarity, a smaller minimum radius or changed acceptance gates.
Research/freeze one replay-only D7R18R3 discriminator at the exact outer-1
trial-2 state. It must propagate normalized density and active-square deltas
through the canonical current/trial pair union and compare against direct
normalized long-double/binary128 oracles without accepting a trial.

Only an independently frozen precancellation candidate that restores the
positive sign and bounded magnitude may return to a full normalized private
transaction. D7R19 remains blocked.
