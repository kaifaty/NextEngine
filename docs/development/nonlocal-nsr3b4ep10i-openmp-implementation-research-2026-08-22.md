# NSR3-B4EP10I OpenMP implementation research -- 2026-08-22

Status: `COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_AUTHORIZED`

## Input

B4EP10D proves the complete owner-computes topology/evaluation/HVP dataflow
bit-for-bit before threading. B4EP10I therefore does not need atomics,
floating reductions or per-worker output arrays. It only schedules operations
whose destination slice already has one logical owner.

## Backend boundary

Use OpenMP only inside the C++17 research executable. The existing repository
reference adapter already resolves OpenMP under the same GCC toolchain, so the
dependency is locally available. This does not select OpenMP for the Rust
runtime or a production solver.

Use 64 logical partitions independent of worker count. Execute their ordinals
with `schedule(static, 1)` and explicit `num_threads`. Disable dynamic teams
and nested active levels. The candidate worker counts are `1,2,4,8,16`; 32
SMT threads are deferred until physical-core scaling is understood.

## Parallel and serial portions

Parallel:

- topology active flags and centre-owned CSR rows;
- pair-local radius/density-scalar/gradient/second coefficients;
- centre-owned density rows;
- active directed evaluation vectors and target-owned gradient rows;
- HVP compression-direction rows, directed values and target-owned result
  rows.

Serial by design in this stage:

- topology-cache certificate and the one initial superset build;
- exclusive prefix, pair/count metadata and energy fold;
- immutable owner transpose-plan construction;
- nonlinear controller, publication and evidence.

This prioritizes exactness and the B4EP9 dominant phases. B4EP10S will show
whether the remaining serial setup is material after implementation.

## Failure model

All destination vectors and capacity checks complete before a parallel region.
Worker kernels allocate nothing and do not throw across OpenMP. Each logical
partition owns one status slot; after the barrier, the lowest failing ordinal
is selected. A failure prevents the candidate result from returning to the
solver.

The contract includes invalid worker counts 0/17 and injected logical
partition 3. It also records actual team size and partition coverage for every
parallel region.

## Evidence separation

B4EP10I is correspondence-only. Run one process at every worker count and one
extra 16-worker repeat. Require common physics/work correspondence and exact
per-count repeat semantics, but do not use those runs for speedup because
their ordering and instrumentation are not a balanced timing design.

Only PASS may authorize B4EP10S serialized scaling. B4EP10S will compare the
unchanged serial B4EP7I command against admitted worker counts in a balanced
order, with system load controlled.

## Decision

Freeze the OpenMP implementation/A-B boundary. No B4E2, GPU, runtime or
production work is authorized.
