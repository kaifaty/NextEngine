# NSR3-B4EP7D evaluation/tape dataflow research -- 2026-08-22

Status: `COMPLETE / PASS / EXACT_FUSION_FEASIBLE`

## Question

Can B4EP7 safely fuse the selected evaluation/base-tape work, and how much
exact duplicate scalar work exists in the B4EP5 transaction before any
arithmetic or ownership change is implemented?

## Current exact dataflow

For a workspace with `N` canonical pairs, `D` directed records attached to
active density centers and `C` fluid centers, the current selected path does:

```text
evaluate_joint density pass       N radius + N weight
evaluate_joint gradient pass      D radius + D weight_gradient
base pressure-tape construction   N radius + C compression
coefficient-tape population       N weight_gradient + N weight_second
evaluation center pass            C compression
```

Across the transaction, `N=85,716,150` and `C=1,356,000` are already frozen.
The aggregate `D` is not serialized by B4EP5. Gprof's combined parent and
transaction call counts cannot separate it safely.

## Candidate fusion boundary

One canonical pair pass can compute radius once, update density in the same
pair order and populate the already selected gradient/second arrays. After
density is complete, the unchanged center/adjacency traversal can read those
binary64 scalars while accumulating gradient in the same order. Its computed
compression can be copied into the pressure tape. Flat CSR is still validated
and transferred only after successful construction.

The projected fused work is:

```text
radius             N       instead of 2N + D
weight_gradient    N       instead of N + D
weight_second      N       unchanged
compression        C       instead of 2C
```

No weight array is needed. Density and gradient accumulation order, vector
operation grouping, pair indices and support reaction writes must not change.

## Why an audit comes first

Add only transaction trace totals derived after each valid tape exists:
pair records, active directed records and fluid centers. Do not increment
counters inside pair loops. A dedicated command reports the formula above and
must preserve exact B4EP5 physics/cache facts and every old command byte.

This audit does not measure speed and does not implement reuse. Its output
freezes `D` and the exact removable-work ratios for a later B4EP7I A/B
contract. A failure preserves B4EP6 without attempting fusion.

## Decision

Freeze B4EP7D as a two-build deterministic dataflow audit only. On PASS it may
authorize B4EP7I fusion design/A-B; runtime, GPU, parallel and solver-policy
work remain blocked.

The audit passes and freezes `D=131,987,230`; see the
[dated evidence](nonlocal-nsr3b4ep7d-evaluation-tape-dataflow-evidence-2026-08-22.md).
