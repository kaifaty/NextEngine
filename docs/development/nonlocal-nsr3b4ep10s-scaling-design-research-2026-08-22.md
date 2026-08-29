# NSR3-B4EP10S serialized scaling design research -- 2026-08-22

Status: `COMPLETE / OWNER_PARALLEL_8_SELECTED / B4EP10R_NEXT`

## Question

B4EP10I proves exact deterministic execution but deliberately records no
durations. B4EP10S asks the narrower host-specific question: does the selected
owner-computes backend reduce one nominal Hydro macro wall time on the current
CPU, what physical-core count is the useful knee, and does measured CPU time
show real resource utilization?

## Host boundary

The admitted host is one AMD Ryzen 9 3950X socket with 16 physical cores and
two SMT threads per core. Logical CPUs `0..15` are distinct physical cores;
`16..31` are their siblings. The observed preflight is:

- 32 online logical CPUs, one NUMA node;
- `amd-pstate-epp`, governor and EPP `performance`, boost enabled;
- process affinity initially `0-31`;
- at least 8 GiB available memory and initial one-minute load at most 2.0.

Do not alter governor, boost, kernel policy or host configuration. Pin the
serial condition to CPU 0 and worker count N to logical CPUs `0..N-1`. Set
`OMP_PLACES=threads` and `OMP_PROC_BIND=close`; the executable still enforces
dynamic teams off, maximum active levels one and the exact requested team.
SMT counts above 16 are outside this stage.

## Alternatives considered

1. **Reuse B4EP10I process durations.** Rejected: those runs were ordered for
   correspondence, not timing, and did not bind physical cores.
2. **Run conditions concurrently.** Rejected: it reduces harness wall time
   but measures contention rather than one-solver scaling.
3. **Exhaustive 6x6 counterbalancing.** Statistically stronger but needlessly
   expensive before knowing whether the backend clears a 10% gate.
4. **Three serialized balanced rounds.** Selected: every condition receives
   three fresh processes across early/middle/late positions, while keeping the
   experiment short enough to avoid another multi-hour screen.

## Selected experiment

Use the unchanged B4EP10I Release executable. Run one unmeasured serial
warmup and one unmeasured 16-worker warmup. Then execute these fresh processes
serially:

```text
round 1: S, 1, 2, 4, 8, 16
round 2: 4, 8, 16, S, 1, 2
round 3: 16, S, 1, 2, 4, 8
```

`S` is the exact B4EP7I fused serial command. Parallel conditions are the
five exact B4EP10I commands. Record monotonic wall nanoseconds plus GNU Time
user seconds, system seconds and maximum RSS outside the executable. Every
process has a 120-second watchdog. Require exact expected stdout, empty
program stderr and the common B4EP10I correspondence hash before admitting
its timing.

For every condition compute median wall, range ratio `max/min`, median RSS and
median effective cores `(user+system)/wall`. Candidate workers are
`2,4,8,16`; worker 1 is a dataflow/parallel-runtime scaling anchor.

## Selection and routing

Find the fastest candidate median, then select the smallest worker count no
more than 3% slower than that median. It passes only when:

- all three selected runs beat the serial run in the same round;
- median serial/selected speedup is at least `1.10`;
- median worker-1/selected speedup is at least `1.10`;
- median effective cores are at least `1.50`;
- every condition's wall range ratio is at most `1.10`;
- all exactness and affinity gates pass.

PASS retains the selected host-specific worker count and authorizes only an
exact B4EP10R residual profile. FAIL retains the serial B4EP7I path and routes
parallel overhead for diagnosis. Neither route authorizes B4E2, runtime/GPU,
public schema or production claims.

## Decision

The [dated evidence](nonlocal-nsr3b4ep10s-owner-parallel-scaling-evidence-2026-08-22.md)
passes all gates and selects 8 workers. Retain that host-specific count and
research B4EP10R residual attribution before another optimization. Do not
generalize the result to runtime or production.
