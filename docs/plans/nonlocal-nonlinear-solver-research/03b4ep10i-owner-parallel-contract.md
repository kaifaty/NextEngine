# NSR3-B4EP10I -- owner-parallel implementation contract

Status: `CLOSED / PASS / B4EP10S_CONTRACT_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10i-owner-parallel|v1|parent=db02821e280df90a285fbbebeea8cc2ec87630891cef105e0d2a4b9f6c3bdc88:431ba5d5596cd2742d042294255ea02c6b67e5499e8d9fe81f1bdf527a5acb44:e3223da62d96f54c3ef125dc5511e71fe9fd0103822c117d2f81fe02d14248a2|implementation=66796758224d08aaace4b19e8a2a2b3ad7654807|backend=openmp;dynamic=false;max-active-levels=1;schedule=static,1;logical-partitions=64|workers=1,2,4,8,16;commands=explicit|topology=parallel-flags+row-owner-csr;prefix+metadata=serial;superset-build=serial|evaluation=parallel-pairs+density-rows+directed-values+target-rows;energy-fold=serial;plan-build=serial|hvp=parallel-center+directed+target-rows|ownership=unique-slices;atomics=none;floating-reductions=none|failure=preallocate;partition-status;lowest-ordinal;reject-before-return|physics=b4ep7i-bit-exact;work=b4ep10d-relations|runs=workers-once+16-repeat;common-correspondence|negatives=workers0,17;partition3|timing=none;b4ep10s-separate|regressions=b4ep7i,b4ep9,b4ep10d|reference=closed|credit=b4ep10s-scaling-only
```

Identity SHA-256:
`94a9d6e26959cfb5c72138457fddc9e72e09c7b4f29882654d0fbcb52759f26c`.

## Build and commands

Resolve and link OpenMP CXX for `nonlocal-formula-reclosure` only. Keep the
existing strict Release floating flags. Add explicit commands:

```text
--nominal-hydro-owner-parallel-1
--nominal-hydro-owner-parallel-2
--nominal-hydro-owner-parallel-4
--nominal-hydro-owner-parallel-8
--nominal-hydro-owner-parallel-16
```

Defaults and old commands do not enter OpenMP regions.

## Executor

Use exactly 64 logical partitions for nonempty work. Partition ranges depend
only on item count and logical ordinal, never worker count. OpenMP executes
partition ordinals with `schedule(static, 1)` and explicit `num_threads`.
Require dynamic teams disabled, maximum active levels one, actual team size
equal to the requested count and exact once-only partition coverage.

All outputs are preallocated. A partition writes only its ordinal status and
its unique pair/centre/target slice. No atomics, OpenMP floating reduction,
cross-worker scatter, dynamic/guided scheduling or nested parallel region is
allowed.

Worker kernels must not throw. Invalid counts 0/17 and injected partition 3
must reject before a candidate result is returned; canonical failure selection
uses the lowest logical ordinal.

## Selected parallel dataflow

- topology: parallel active flags and centre-owned row count/fill; serial
  prefix, canonical compaction and metadata;
- fused evaluation: parallel pair-local coefficients, density rows, active
  directed values and target rows; serial owner-plan construction and energy
  fold;
- HVP: parallel compression-direction centre rows, active directed values and
  target result rows.

Return the owner-computes candidate rather than executing the old scatter
oracle internally. B4EP10D is the transformation oracle; external exact roots
and regressions are the implementation correspondence gate.

## Execution gate

Run worker counts `1,2,4,8,16` once each, then repeat 16 in a fresh process.
Every process must exit zero with empty stderr and reproduce:

- B4EP7I physical/publication roots, schedule, ledgers and work chain;
- B4EP10D topology/evaluation/HVP relational counts;
- zero executor/team/coverage/worker/capacity mismatch;
- one common correspondence hash excluding worker count.

The two 16-worker complete reports must be byte-identical. The final binary
must retain B4EP7I exact stdout, B4EP9 exact semantic result and B4EP10D exact
stdout.

These runs admit no timing or speedup claim. PASS authorizes only B4EP10S
balanced serialized scaling. Failure retains the serial B4EP7I/B4EP10D path.
B4E2, GPU/runtime/schema, PhysX and production remain blocked.

## Closure

B4EP10I passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10i-owner-parallel-evidence-2026-08-22.md).
All worker counts share correspondence SHA-256
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`,
all executor mismatch counts are zero and the two 16-worker reports are
byte-identical. This authorizes only a separately frozen B4EP10S scaling
experiment.
