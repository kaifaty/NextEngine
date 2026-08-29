# NSR3-B4EP10SIRDIREQ1 -- CPU-time residual attribution contract

Status: `COMPLETE / PASS / TOPOLOGY_STRUCTURAL_AUDIT_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10sirdireq1-cpu-time-residual|v1|parent=d22dfdad22a9a3d02dde9ec3301c1e36124c82d3a075410a0828d2d84e8ea9d6:HOST_UNQUALIFIED:7b116eb3c4b7f9c51740c1f2a73daed573a07c7872fea80992e096f2f95f59a3|sirdir=e620072432ce93e98a3f58005b0bc428f8a12b975d5af34143ddde55c415b374:1f66ab3c1bffa2199759c66e6297273e4777902777cedd1634dbad7e0895992e|implementation=b8a1eddffcf37a6281e2f67bc80e9a9ef07b2f04|command=nominal-hydro-directed-scratch-cpu-timing-8|clocks=process:CLOCK_PROCESS_CPUTIME_ID;worker:CLOCK_THREAD_CPUTIME_ID;resolution-ns<=1000;fail-closed|work=transaction1;topology226;evaluation226;hvp459;phases6363;process-phase-intervals7275;process-region-intervals4089;thread-active-intervals32712|categories=topology;target-fold;directed;hvp-compression;other-local;stopped-evaluation-setup;control;sum-exact|measurement=three-fresh-processes;affinity0-7;gnu-time-cross-check;wall-no-credit|gates=exact3of3;cpu-category-range<=0.03;external-over-internal-total-cpu-ratio=1.00..1.05;eligible-leader-share>=0.20;eligible-lead>=1.20|routing=leader:one-structural-audit;no-leader:finer-cpu-discriminator;clock-failure:retain-sirdi|reference=closed|authority=research-only;no-wall-speedup
```

Identity SHA-256:
`6518ed9875e227f25b298eeb6fd5b3eb3708d1e7014357125a700e9c0d112964`.

## Implementation boundary

Add only:

```text
--nominal-hydro-directed-scratch-cpu-timing-8
```

The command is unchanged SIRDI plus existing SIRDIR steady-clock timing and a
parallel CPU-time trace. Use `clock_gettime(CLOCK_PROCESS_CPUTIME_ID)` for
transaction, stage, subphase and complete OpenMP-region intervals. Use
`clock_gettime(CLOCK_THREAD_CPUTIME_ID)` independently in each observed worker
around its active `schedule(static,1)` loop. Check every return value,
`timespec` range, subtraction and accumulation. Any failure rejects the report.

Do not change formulas, vector ownership, loops, partitions, worker count,
OpenMP schedule, arithmetic, convergence, roots or existing command bytes.
CPU durations and resolution remain excluded from all semantic results.

## Exact work and accounting

Require:

- one transaction, 226 topology, 226 evaluation and 459 HVP intervals;
- 6,363 phase intervals and 7,275 combined process phase/total intervals;
- 4,089 process OpenMP-region intervals;
- 32,712 worker-active intervals (`4,089 * 8`);
- process/thread clock resolution from 1 through 1,000 ns;
- positive durations, zero clock/conversion/overflow failures;
- worker active CPU no greater than region process CPU after checked sums;
- exact SIRDIR wall call/capacity/category accounting unchanged.

Define process-CPU categories exactly as the research note. Require nonzero
`other_local` and `control`, a checked seven-category sum equal to transaction
CPU and durations excluded from the semantic result.

## Execution and routing

Build Release with repository `-Werror`. Verify old SIRDI stdout SHA-256
`539f1ec5...e7e7` and the new command's SIRDI/SIRDIR result, correspondence,
work/lifetime counts and five roots.

Run three fresh serialized processes on the existing physical CPU set `0..7`.
Record GNU user/system/RSS only as an external CPU cross-check; wall duration
grants no health, speed or selection credit. Require every CPU-category share
range at most 0.03 and median external total CPU / internal transaction CPU in
`[1.00, 1.05]`.

Exclude `stopped_evaluation_setup` from routing. An eligible category with
median share at least 0.20 and at least 1.20x the second eligible category
authorizes exactly one timing-free structural audit. No leader routes to a
finer CPU-time discriminator. Clock, accounting, exactness or cross-check
failure retains SIRDI without an optimization route.

No result is wall throughput, B4E2, broad corpus, runtime/GPU/schema or
production evidence.

## Closure

The command passes `3/3` with exact semantic result `e5ddff76...14f`, all
category-share ranges at most `0.024734` and median GNU/internal CPU ratio
`1.007535`. Topology has median share `0.301239` and leads target fold by
`1.629380x`, authorizing one timing-free topology structural audit. See the
[dated evidence](../../development/nonlocal-nsr3b4ep10sirdireq1-cpu-time-residual-evidence-2026-08-22.md).
