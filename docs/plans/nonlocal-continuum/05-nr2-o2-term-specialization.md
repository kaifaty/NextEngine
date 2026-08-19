# NR2-O2 — Compile-time viscosity term specialization

Status: `SPECIFIED / IMPLEMENTATION_NOT_STARTED / REPORT_ONLY`

Prerequisite:
[NR2-O1 evidence](../../development/nonlocal-continuum-nr2-o1-evidence-2026-08-20.md)
exits `O1_RETAINED_POINTER_SWAP` and admits O2 from the exact gather/swap
identity.

## Outcome

Measure the remaining device-side term-dispatch cost without changing pair
passes, formulas, term order, local-system shape or fixed work. The O1 baseline
is:

```text
accumulation = nuv-gather-directed-r0
handoff       = pointer-swap-o1
term kernels  = nuv-terms-runtime-v0
```

The only O2 candidate is:

```text
accumulation = nuv-gather-directed-r0
handoff       = pointer-swap-o1
term kernels  = nuv-terms-specialized-o2
```

`--term-kernels` selects the third identity explicitly in every CUDA command
and JSON report. Commands without it preserve `nuv-terms-runtime-v0`.
`nuv-terms-specialized-o2` is rejected at construction unless the retained
gather and pointer-swap identities are also selected.

## Audited specialization boundary

The current physical accumulation is already separated into density,
incompressibility, viscosity and surface launches. Host-side term checks omit
inactive viscosity and surface launches, and the density/surface kernels have
no runtime term mask. O2 therefore targets only the remaining viscosity mask:

```text
bulk only    -> viscosity<bulk=true,  shear=false>
shear only   -> viscosity<bulk=false, shear=true>
bulk + shear -> viscosity<bulk=true,  shear=true>
none         -> no viscosity launch
```

Each candidate instantiation owns the same particle, traverses the same
directed CSR slots in the same order and emits the same two endpoint
contributions. The bulk+shear instantiation retains the original expression
order. Single-term instantiations omit construction and arithmetic belonging
only to the disabled term. Template flags are not kernel arguments and no
device branch or select may depend on them.

O2 intentionally does not fuse incompressibility, viscosity or surface
passes. Fusion, clear removal and launch reduction belong to O5. It also does
not specialize the post-timing energy diagnostic: keeping that independent
runtime-mask path makes it a stable comparator and cannot improve the measured
device timeline.

## Frozen dimensions

O2 retains:

- every fixture/profile hash, active term set, coefficient, tolerance and
  fixed iteration count;
- CUDA `f32`, compiler flags, O1 pointer ownership and all allocations;
- density, incompressibility, surface, update and velocity kernels;
- the full nine-entry matrix and unregularized `3x3` local inverse;
- explicit source/matrix/error clears and per-term launch order;
- neighbor construction, exact CSR membership/order, pair evaluation count,
  capture order and diagnostics;
- default atomic/copy behavior and every NR1/RC1/O1 identity.

O2 adds no device buffer or temporary storage. Executable size and the three
template instantiations are reported as engineering cost, not device capacity.

## Correctness gates

Run in order and stop before timing on any failure:

| Gate | Required result |
|---|---|
| `O2-BUILD` | external CMake/Ninja build PASS under the unchanged CUDA flags |
| `O2-LEGACY` | default atomic/copy/runtime tiny 11/11 PASS; O1 gather/swap/runtime tiny 11/11 PASS; invalid atomic or copy + specialized combinations reject |
| `O2-TINY-MASKS` | specialized tiny passes CPU correspondence 11/11; bulk-only, shear-only and bulk+shear cases execute; three reused executions are exact; runtime/specialized field correspondence passes; CSR and memory are exact |
| `O2-SURFACE-I2/I20` | ten cold specialized repeats at both counts are exact and match runtime output/CSR exactly because no viscosity path is active |
| `O2-FULL` | water-16k/48k i5 and viscous-16k i20 pass finite/topology/capacity/momentum gates; two reused candidate executions are exact; runtime/candidate full-field correspondence, CSR and memory pass |

Runtime/specialized ordered output digests are reported. Exact equality is
recorded when observed but is not the mathematical gate: removing arithmetic
on an identically zero disabled term may change signed-zero or `f32`
association while still passing the pre-existing absolute-or-relative field
contract. Candidate repeats themselves MUST remain exact.

## Adjacent timing and retention

After correctness, run runtime immediately followed by specialized on
water-16k, water-48k and viscous-16k with five warm-ups and 50 measured runs.
Use one final binary/toolchain/device state and report complete stage/total
statistics, executable bytes and device memory.

Retain O2 only if:

- every correctness gate passes;
- device memory does not increase;
- viscosity p95 decreases on all three timed profiles;
- total p95 improves on both HN-3 denominator profiles, water-48k and
  viscous-16k.

No minimum percentage is required for one isolated candidate. A failure exits
`O2_REJECTED_NO_TOTAL_BENEFIT`; `nuv-terms-runtime-v0` remains the retained NR2
baseline and the specialized path stays selectable only to reproduce evidence.
O1 and O2 percentages are never added across non-adjacent runs.

## Required post-O2 profiler capture

After the retain/reject decision, capture the resulting retained path on
water-48k and viscous-16k with Nsight Systems and one filtered Nsight Compute
viscosity launch per profile. Raw `.nsys-rep`/`.ncu-rep` files remain outside
Git. The evidence records tool versions, exact commands and report hashes, and
summarizes where available:

- kernel/device time attribution and launch counts;
- registers per thread and achieved occupancy;
- SM and DRAM throughput;
- whether specialization moved the dominant bottleneck.

Profiler replay is diagnostic and cannot replace the adjacent timing gate.
Unavailable hardware counters are recorded rather than inferred.

## Exit states

### `O2_RETAINED_TERM_SPECIALIZATION`

All correctness and retention rules pass. The specialized identity becomes
the input to O3, subject to the post-O2 profiler record.

### `O2_NUMERIC_MISMATCH`

Candidate repeatability or runtime/candidate correspondence fails. Preserve
both identities and stop without timing.

### `O2_REJECTED_NO_TOTAL_BENEFIT`

Correctness passes but a timing rule fails. Retain the runtime O1 path, record
the negative result and continue to O3 without counting O2 as speedup.

### `O2_PROFILER_INCOMPLETE`

The timing decision is frozen, but the required post-O2 attribution cannot be
completed. Preserve the result and keep O3 blocked until the missing capture
or explicit unavailable-counter record is checked in.

## Non-goals

Pass fusion, clear or launch removal, accumulation/layout changes, cell
sorting, precision changes, reduced local systems, adaptive exit, warm start,
CUDA Graphs, runtime integration, public contracts, Windows execution and NR4
selection.
