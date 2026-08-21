# NSR3-B4EP10R -- selected-8 residual profile contract

Status: `FROZEN / EXECUTION_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10r-selected8-profile|v1|parent=f034be427e9ca744391df7848e8c1237d6bda6485126dd1444e4286f6392ea03:839fe1bb6fedadfafd5fe44866723026096124a89bf824ac2fd2c839787f5167:8|implementation=abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd|binary=b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46|profiler=gprofng-2.46;clock=1ms;sync=all,native;periodic=1s;descendants=off;archive=usedldobjects|host=amd-ryzen-9-3950x;affinity=0-7;omp-places=threads;omp-proc-bind=close;dynamic=false|run=one-fresh;watchdog=120s;exact-stdout=c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3;correspondence=917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca|reports=header,overview,metric-list,functions,calltree;experiment-hash=manifest|categories=runtime-sync;topology-lambda1-4+owner-filter;evaluation-lambda1-5+owner-plan;hvp-lambda1-3+owner-hvp;residual|metric=exclusive-total-cpu;minimum-total-cpu=10s|route=normalized-sync>=0.20=>persistent-region-research;else-leader>=1.20=>leader-research;else=>internal-phase-timing|timing=no-speed-claim;reference=closed|credit=one-next-design-only
```

Identity SHA-256:
`6a3b44a4012d23e9ff218e33ebfc896d4e61d566bc34affc42949f3ef68a4358`.

## Inputs and command

Use the exact B4EP10I executable SHA-256
`b5bc2619f58c6eff8d219016b1a3d145add1ca63b4470e28e0b08be4dd325c46`
and GNU gprofng 2.46. Require a clean worktree, the B4EP10S host, CPU affinity
`0..7`, `OMP_PLACES=threads`, `OMP_PROC_BIND=close` and
`OMP_DYNAMIC=false`.

Under a 120-second watchdog run one fresh process equivalent to:

```text
gprofng collect app \
  -p 1 -s all,n -S 1 -F off -a usedldobjects \
  -o <external>/selected8.er \
  nonlocal-formula-reclosure --nominal-hydro-owner-parallel-8
```

The whole collector and target inherit the frozen affinity/environment. Do
not enable hardware counters or alter host policy.

## Admission

Extract the target's final JSON line from collector stdout. Require its SHA-256
to equal
`c47e9393b88208ade76274cff1ee66af00d7da6bef7a461df7b3575b6f4c2ee3`,
its correspondence to equal
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`,
team min/max to equal 8, and all mismatch counters to be zero. Collector stderr
must be empty.

Generate and retain `header`, `overview`, `metric_list`, `functions` and
`calltree` text reports. The header must say `No errors` and contain no
collector reliability warning. Require at least 10 seconds of total sampled
CPU. Hash a sorted manifest of every regular experiment file and every text
report.

## Categories and route

Aggregate non-overlapping exclusive Total CPU Time samples by demangled symbol:

- runtime/sync: libgomp, pthread, futex and collector-reported synchronization;
- topology: owner filter and topology lambdas 1--4;
- evaluation: parallel evaluation builder/lambdas 1--5 and owner-plan builder;
- HVP: owner HVP and HVP lambdas 1--3;
- residual: the remainder.

Normalize summed synchronization wait by `8 * experiment elapsed wall`.
If it is at least 0.20, select persistent OpenMP-region/barrier amortization
research. Otherwise, if one exclusive-CPU category leads the runner-up by at
least `1.20x`, select that category. Otherwise select scoped internal parallel
phase timing.

## Exit

PASS records one exact selected-count attribution and authorizes only the
routed next design research. The profile has instrumentation overhead and is
not a speed result. Failure preserves B4EP10S and starts no optimization.
B4E2, broad corpus, runtime/GPU/schema and production remain blocked.
