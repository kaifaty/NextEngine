# NP1-P4 — certified Verlet neighbor reuse

Status: `COMPLETE / CORRECTNESS_STOP / P2_ROLLBACK / EXACT_RESULT_REUSE / REPORT_ONLY`

## Hypothesis

`verlet-skin-p4` can amortize directed-CSR construction across sequential
substeps without changing the active logical neighborhood or solver result.
On the 32-step advected water trace it should improve total p95 by at least
`5%` after paying for a maximum-displacement certificate and exact per-slot
active filtering. Coupled controls may not regress by more than `2%`.

The adjacent denominator and rollback identity is retained P1+P2. P3 is not an
input:

```text
nuv-gather-directed-r0
+ pointer-swap-o1
+ nuv-terms-specialized-o2
+ stable-sample-v0
+ fused-owner-terms-p1
+ compact-csr-u16-p2
```

P4 adds a persistent, report-only neighbor cache. It does not change sample
count, nonlinear iterations, coefficients, accepted iterate or arithmetic
association over the active logical CSR.

## Frozen skin and certificate

- skin `s = 0.04 h`, where `h` is the frozen interaction horizon;
- rebuild anchor `a_i` is the canonical reference position copied when a
  superset is built;
- before reuse, compute `d²_max = max_i ||x_i - a_i||²` in double precision;
- reuse is certified only when
  `4 d²_max <= s² (1 - 2e-12)`;
- cache invalidation/reset always forces a rebuild; no warm state crosses a
  fixture reset or trace epoch;
- non-finite displacement, grid/capacity error or failed certificate rebuilds
  or rejects before solver use.

The factor four is the conservative pair-relative triangle-inequality bound:
if both endpoints move at most `s/2`, a pair currently within `h` was within
`h+s` at the anchor. The small safety contraction prevents a floating boundary
from authorizing an under-covered list.

## Canonical superset construction

The grid cell size and packed keys remain based on `h`, not `h+s`. A rebuild
searches `ceil((h+s)/h) = 2` cells per axis and admits anchor pairs within
`h+s`. Because the retained `h` neighborhood can occupy only the central
`[-1,+1]` cell range, filtering a superset row preserves the exact retained
cell/sample-ID order of every active pair.

P4 reuses the existing compact neighbor capacity:

- fixed coherent/permuted lattice: the `0.04h` skin does not cross the next
  lattice shell, so the frozen 123-neighbor capacity remains sufficient;
- advected profiles retain their frozen 192-neighbor capacity;
- count/scan/fill still clamp and signal overflow before a solve;
- no second active CSR, per-edge flag array or shadow 32-bit list is allocated.

The only new persistent storage is one `float3` anchor per sample and one
64-bit maximum-displacement reduction value: `12N + 8` bytes.

## Exact active filtering

On both rebuilt and reused executions, density, fused owner terms, final
density and energy diagnostics visit the superset row but execute an entry
only when the current canonical reference distance passes the exact retained
`h² * (1 + 2e-6)` membership predicate. Slot order among active entries is
unchanged. Filtering is based on the substep reference state, not nonlinear
iterate positions.

Host capture emits two distinct receipts:

- physical superset CSR/hash and candidate-pair count;
- filtered active logical CSR/hash and active-pair count.

Only the active CSR participates in correspondence, topology, momentum and
output identity. Filter cost is inside every pair kernel. Host diagnostic
filtering/hashing remains outside timing.

## Timing and state

Neighbor-stage timing includes displacement reduction, device-to-host rebuild
decision synchronization, and the complete sort/count/scan/fill when rebuilt.
Reports record maximum displacement, rebuild reason, rebuild/reuse count,
active/candidate pair ratio and amortized neighbor cost.

The tournament uses 256 conditioning executions, 32 formal warm-ups and 96
alternating measured rounds. Dynamic traces reset both canonical state and
cache every 32 steps, so each epoch contains a forced cold rebuild. Fixed-input
controls also invalidate at each explicit reset; unchanged consecutive states
may then reuse by certificate.

## Correctness and retention

Before timing P4 must pass:

1. retained P2 checks;
2. tiny CPU `f64`, stiff surface i2 and target cold-rebuild correspondence;
3. at least three forced-cold/reused transitions with exact output and active
   CSR;
4. every 32-step advected output, active CSR and handoff exactly matching P2;
5. certificate inequality and rebuild reason on every reuse;
6. symmetric active topology, finite state, momentum and capacity bounds;
7. exact `12N + 8` added memory and unchanged v0 hashes.

Retain P4 only if all gates pass, advected water total p95 improves by at least
`5%`, neither coherent nor advected 50k regresses by more than `2%`, and both
coupled 16k controls remain within `2%`. Otherwise retain P2 as the NP1
finalist. A capacity overflow or active-CSR mismatch stops the family without
skin tuning after the fact.

P4 stopped at the reproduced step-1 active-order mismatch. See the
[dated evidence](../../development/nonlocal-continuum-np1-p4-evidence-2026-08-20.md).
