# NSR3-B4EP10R selected-8 residual profile research -- 2026-08-22

Status: `COMPLETE / GPROFNG_CLOCK_AND_SYNC_SELECTED`

## Question

B4EP10S proves a host-specific 8-worker speedup, but the 1-worker owner path
adds about 4.10 s and 16 workers improve the 8-worker knee by only 1.70%.
B4EP10R must distinguish useful kernel work from topology/evaluation/HVP
residuals, serial owner-plan work and OpenMP runtime/synchronization overhead
before another code change.

## Tool audit

- Linux `perf` remains blocked by host `perf_event_paranoid=4`; do not alter
  kernel policy.
- classic GCC `-pg`/gprof was adequate for the serial lineage but is a poor
  ownership boundary for worker-thread attribution.
- Valgrind/Callgrind and uftrace are unavailable.
- GNU gprofng 2.46 is installed. Its local primary manual documents clock PC
  sampling, native synchronization-wait tracing, periodic resource sampling
  and profiling of any ELF executable without requiring debug information.
- A separate `/bin/true` collector probe and a dry run of the exact command
  both succeed. The dry run resolves 1 ms clock sampling, all native sync
  tracing, one-second resources, no descendants and used-load-object archive.

The Release binary retains distinct symbols for the four topology, five
evaluation and three HVP OpenMP outlined functions. This makes an unmodified
profile more informative and lower risk than first adding timers to every
parallel region.

## Selected profile

Run one fresh exact 8-worker process under gprofng 2.46:

- clock sampling at 1 ms;
- native synchronization tracing at all waits;
- periodic resource sampling at 1 s;
- descendant following off;
- archive only used load objects;
- CPU affinity `0..7`, `OMP_PLACES=threads`, `OMP_PROC_BIND=close`;
- 120-second watchdog.

The collector adds one informational stdout line. Extract the target's final
JSON line and require the frozen B4EP10I worker-8 stdout/correspondence,
requested team and zero mismatches. Profiler overhead is not a timing result.

Archive header, overview, metric list, function view and call tree. Build a
content manifest over every regular experiment file rather than hashing
directory metadata.

## Frozen attribution

Use non-overlapping exclusive Total CPU Time samples:

- `runtime-sync`: libgomp/pthread/futex runtime and synchronization;
- `topology`: `b4ep10d_owner_filter_superset` and its lambdas 1--4;
- `evaluation`: `build_joint_evaluation_tape_owner_parallel_from_flat`, its
  lambdas 1--5, and `build_joint_owner_gather_plan`;
- `hvp`: `apply_joint_pressure_tape_owner_gather` and its lambdas 1--3;
- `residual`: all remaining application work.

Require at least 10 sampled CPU seconds and a collector header with no error
or reliability warning. Compute normalized synchronization wait as summed
sync wait divided by 8 times experiment elapsed wall.

Routing is frozen:

1. normalized sync wait at least 0.20 selects persistent-region/barrier
   amortization research;
2. otherwise, a CPU category leading the runner-up by at least `1.20x`
   selects only that category for next design;
3. otherwise select scoped internal parallel phase timing.

## Decision

Freeze one exact gprofng profile. B4EP10R cannot claim speed or authorize
B4E2/runtime/GPU/production work; it may select only one next design target.
