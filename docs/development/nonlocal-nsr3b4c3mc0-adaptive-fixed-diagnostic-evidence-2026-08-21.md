# NSR3-B4C3MC0 adaptive-versus-fixed diagnostic evidence

Status: `PASS / MEASUREMENT_ONLY / ACCURACY_BUDGET_DESIGN_AUTHORIZED`

Date: `2026-08-21`

## Reproducible result

Full command:

```text
nonlocal-formula-reclosure --adaptive-fixed-diagnostic-self-test
```

Two parent-gated reports executed concurrently and are byte-identical:

```text
status                 PASS / MEASUREMENT_ONLY
raw JSON + LF          cc05d841f187ea3028aef80482af3ba7b4c66b21c031dd6f8b8b02057e608937
raw JSON without LF    923c09a8e86a473df8b5903ce170a10154c959a6ca8532136618bd05cf2d57c8
semantic result        063a7e4b9955ef5c35f5fef743707728aa0373c2bd5c8747748aeff5d28532a6
wall time              92.77 s / 91.56 s
CPU utilization        303% / 302%
maximum RSS            16,488 KiB / 14,892 KiB
parent B4C3MAR exact    true
```

The isolated diagnostic passes in `21.11 s` at raw-with-LF
`7de053eb710a5c1e82b067c6ac88b1885df5fb8ab2cc713c26ec0f94af5fc8ae`,
raw-without-LF
`e269b587822093f124b49c7c431b13b309b1022240fabf01bc6e867961d9fbb0`
and the same semantic result.

## State and reference measurements

The error is adaptive versus fixed-192. Temporal ratio is that error divided
by the independent fixed-96/fixed-192 difference when the latter exceeds its
computed binary64 floor. Physical utilization uses the pre-existing B4B scales
`0.05dx` for RMS position and `0.001c` for RMS velocity.

| Case / field | Resolved frames | Temporal ratio min / median / max | Maximum physical utilization |
|---|---:|---:|---:|
| P1 position | `8 / 8` | `1.863 / 3.495 / 3.936` | `0.00647` |
| P1 velocity | `8 / 8` | `2.779 / 3.512 / 3.819` | `0.07028` |
| P2 position | `16 / 16` | `60.716 / 86.653 / 100.417` | `0.18631` |
| P2 velocity | `1 / 16` | `94.035 / 94.035 / 94.035` | `0.79151` |

P2 velocity is exactly level-invariant through free flight. Fourteen frames
have zero adaptive error as well; frame 14 has a nonzero adaptive/fine
separation while fixed-96/fixed-192 remains at the numerical floor. Frame 15
is the one temporally resolved velocity comparison. An unresolved denominator
must therefore not be serialized as evidence of temporal closeness.

## Aggregate and event measurements

| Case | Center max (`dx`) | q99 height/front max (`dx`) | Kinetic max | Onset error |
|---|---:|---:|---:|---:|
| P1 | `1.8375e-4` | `3.20e-4 / 0` | `0.02052` relative | `7.7505e-5 s` |
| P2 | `8.4267e-3` | `1.142e-2 / 7.86e-3` | `0.02984` relative | `3.6892e-4 s` |

Every aligned frame has the exact fixed-192 terminal contact set, and the
final terminal sets are exact. All values are finite. Adaptive and fixed
trajectory, legacy-ledger and policy-ledger roots are emitted independently;
the adaptive roots reproduce the selected B4C3MAR P1/P2 roots exactly.

## Interpretation

The fixed reference is sufficiently stable to measure the adaptive error, but
the two scenarios occupy different regimes. P1 is close to the fine trajectory
both absolutely and relative to its temporal difference. P2 remains inside the
existing physical comparison scales, yet is deliberately much coarser than the
fixed temporal ladder; its position error is `60--100x` the fine temporal
difference and its contact velocity reaches `0.792` of the physical scale.

This does not invalidate the adaptive controller: fixed-192 uncertainty and
the independently chosen product-scale accuracy envelope answer different
questions. It does prevent a claim that the P2 adaptive trajectory is inside
the fine discretization uncertainty. The next gate must preserve both facts
instead of collapsing them into one fitted multiplier.

## Decision

B4C3MC0 passes only as a deterministic measurement stage. Authorize B4C3MC1
design with two independent outputs:

1. an accuracy decision against the unchanged B4B physical, aggregate, kinetic
   and event budgets;
2. a temporal-reference classification that distinguishes resolved ratio,
   floor-level coincidence and stable-reference separation.

Do not fit a ratio threshold to these two fixtures. Adaptive accuracy,
nominal-corpus execution, B4C4/B4D, CUDA, runtime/schema and production remain
blocked until the separately frozen gate runs.
