# NSR3-B4EP10S -- owner-parallel serialized scaling contract

Status: `CLOSED / PASS / B4EP10R_PROFILE_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10s-owner-parallel-scaling|v1|parent=94a9d6e26959cfb5c72138457fddc9e72e09c7b4f29882654d0fbcb52759f26c:f8299c06b49dbdab866a98067db2809da58d41d21955a54df8e42634ab100cd2:917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca|implementation=abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd|binary=b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46|host=amd-ryzen-9-3950x;physical-cores=16;smt=2;cpus=0-31;governor=performance;boost=1|affinity=serial:0;worker-N:0..N-1;omp-places=threads;omp-proc-bind=close;dynamic=false;max-active-levels=1|warmup=serial,16|rounds=3;order=S,1,2,4,8,16;4,8,16,S,1,2;16,S,1,2,4,8|processes=fresh;serialized;watchdog=120s;clock=monotonic-ns;resources=gnu-user,sys,rss|exact=serial-b4ep7i-stdout;parallel-b4ep10i-stdout;common-correspondence;empty-stderr|selection=workers2,4,8,16;fastest-median;choose-min-within1.03|gate=3of3-vs-serial;median-speedup>=1.10;median-vs-worker1>=1.10;effective-cores>=1.50;range-ratio<=1.10|route=pass-b4ep10r;fail-retain-serial|timing=host-specific;reference=closed|credit=b4ep10r-profile-only
```

Identity SHA-256:
`f034be427e9ca744391df7848e8c1237d6bda6485126dd1444e4286f6392ea03`.

## Immutable inputs and preflight

Use implementation `abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd` and the
exact B4EP10I Release executable at SHA-256
`b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46`.
The worktree must be clean before timing.

Require the host facts frozen by the identity: one Ryzen 9 3950X, 16 physical
cores/32 logical CPUs, logical CPUs `0..15` on distinct cores, performance
governor/EPP, boost enabled, initial affinity `0-31`, at least 8 GiB available
memory and initial one-minute load at most 2.0. Do not mutate host policy.

## Commands, affinity and environment

`S` is:

```text
--nominal-hydro-fused-evaluation-tape-ablation
```

Worker N is:

```text
--nominal-hydro-owner-parallel-N
```

Pin S and worker 1 to CPU 0. Pin worker N to `0..N-1`. For every parallel
process set `OMP_PLACES=threads`, `OMP_PROC_BIND=close` and
`OMP_DYNAMIC=false`. Run fresh processes one at a time; no overlapping solver,
profile or build process is allowed.

Run one unmeasured S warmup and one unmeasured worker-16 warmup. Then run:

```text
S,1,2,4,8,16
4,8,16,S,1,2
16,S,1,2,4,8
```

Apply a 120-second watchdog to each process. Record monotonic wall nanoseconds
around the process and GNU Time user seconds, system seconds and maximum RSS
to separate external files.

## Admission and computation

Require every process to exit zero with empty program stderr. S must retain
B4EP7I stdout SHA-256
`8d3c8115861feb80e322a594be8f338dcab9622ac7b2839ec17a0f779fac2095`.
Each parallel stdout must match its B4EP10I receipt and expose correspondence
SHA-256
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`,
the requested/observed team and zero executor mismatch.

For S/1/2/4/8/16 compute median wall, `max/min` wall range ratio, median RSS
and median effective cores `(user+system)/wall`. Pair each condition with the
same-round S value. Do not round values before selection.

Among workers `2,4,8,16`, find the fastest median and choose the smallest
worker count with median wall at most `1.03 * fastest`. Retain it only if:

- selected/S wins are 3/3;
- median of the three same-round `S/selected` ratios is at least `1.10`;
- selected median is at least `1.10x` faster than worker-1 median;
- selected median effective cores are at least `1.50`;
- every condition has wall range ratio at most `1.10`;
- exactness, affinity and resource parsing have no failure.

## Exit

PASS selects one host-specific `OWNER_PARALLEL_<N>_SCALING_CANDIDATE` and
authorizes only B4EP10R exact residual profiling at that count. A speed,
stability or utilization failure retains serial B4EP7I and routes to a
parallel-overhead diagnostic; an exactness failure also rejects the B4EP10I
implementation result.

This experiment cannot claim broad-corpus throughput, 50k scaling, real-time,
runtime/GPU readiness or production suitability. B4E2 remains blocked.

## Closure

B4EP10S passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10s-owner-parallel-scaling-evidence-2026-08-22.md).
The frozen knee rule selects 8 workers at median `1.2370200585216442x`
same-round speedup and 6.421 median effective cores. Worker 16 is only 1.70%
faster while consuming nearly twice the CPU. This authorizes only B4EP10R
selected-count residual profile research.
