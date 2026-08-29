# NSR3-B4EP10R1 -- internal parallel phase-timing contract

Status: `CLOSED / PASS / B4EP10P_RESEARCH_AUTHORIZED`

Identity projection (exact bytes, no final LF):

```text
nextengine.nonlocal.nsr3b4ep10r1-internal-parallel-phase-timing|v1|parent=6a3b44a4012d23e9ff218e33ebfc896d4e61d566bc34affc42949f3ef68a4358:1ceaa1eda36280916a2fad0a17a65532c6db29e3ab91ebacbb07e2ba574d89aa:f034be427e9ca744391df7848e8c1237d6bda6485126dd1444e4286f6392ea03:839fe1bb6fedadfafd5fe44866723026096124a89bf824ac2fd2c839787f5167|implementation=abb7a06bdc16c3a906ce3b0e5d85f19a250dd9bd|command=nominal-hydro-owner-parallel-phase-timing-8;workers=8;affinity=0-7|clock=steady;opt-in;transaction-only;runs=3;durations-excluded-result|topology=setup,active,prefix,compact,metadata,row-count,offsets,row-fill,finalize;calls=226|evaluation=setup,pair,metadata,density,center,plan,directed,target,finalize;calls=226|hvp=setup,compression,directed,target,finalize;calls=459|executor=regions3411;logical-partitions218304;region-wall;sum-active;max-active;orchestration;imbalance|semantics=b4ep10i-common-correspondence;old-commands-exact|stability=share-range<=0.05|route=orchestration>=0.20=>persistent-region;imbalance>=0.20=>partition-balance;else-stage-leader>=1.20=>stage-design;else=>deeper-timing|timing=no-speed-claim;reference=closed|credit=one-next-design-only
```

Identity SHA-256:
`ab9f3e0bca369c80e0185d6c73cb8334ff4baa6a9aa8728758b6fe162975d638`.

## Implementation boundary

Add only:

```text
--nominal-hydro-owner-parallel-phase-timing-8
```

It executes the exact B4EP10I 8-worker transaction under a dedicated timing
flag. Old commands do not enter timed executor code and must remain byte-exact.
The command is research-only and has no runtime/public option.

Use `std::chrono::steady_clock`. All duration additions and capacity formulas
must be overflow checked. Each worker writes one cache-line-padded active
duration slot. Use `omp for schedule(static,1) nowait` followed by an explicit
barrier only in the timed branch; output partitions and arithmetic remain
unchanged.

## Exact counters and phases

Require:

- one transaction timer;
- 226 topology totals and each of its nine subphases;
- 226 evaluation totals and each of its nine subphases;
- 459 HVP totals and each of its five subphases;
- 3,411 timed executor regions and 218,304 logical partitions;
- observed team 8 throughout, zero executor/timer/overflow/coverage failure;
- per-region capacity identity
  `active + imbalance + orchestration = 8 * region_wall`.

Subphase sums may not exceed their stage total. Topology + evaluation + HVP
may not exceed transaction total; report the residual. Emit all durations but
exclude them from the semantic result SHA-256.

## Runs and route

From the same final Release binary run three fresh processes pinned to CPUs
`0..7` with `OMP_PLACES=threads`, `OMP_PROC_BIND=close` and dynamic teams off.
Each must exit zero with empty stderr and reproduce B4EP10I common
correspondence SHA-256
`917a04d31bb849a9bee5dd190ad6d15e07c9c90a9c2822130ae1adac6ebcb4ca`.
All three semantic result hashes must match. For executor orchestration,
imbalance, active and top-level stage shares require max-minus-min at most
0.05.

Route by median values:

1. orchestration capacity share at least 0.20: persistent-region/team
   amortization research;
2. else imbalance share at least 0.20: logical partition/load-balance
   research;
3. else a top-level stage at least `1.20x` its runner-up: one design in the
   leading subphase;
4. else: deeper internal timing, no optimization.

## Exit

PASS authorizes only the routed research/design. These instrumented durations
are not throughput or speedup evidence. Failure preserves B4EP10S/B4EP10I.
B4E2, broad corpus, runtime/GPU/schema and production remain blocked.

## Closure

B4EP10R1 passes; see the
[dated evidence](../../development/nonlocal-nsr3b4ep10r1-internal-parallel-timing-evidence-2026-08-22.md).
Executor orchestration and imbalance medians are only 1.11% and 6.55%.
Evaluation instead leads HVP by `1.916851x`, and `evaluation_plan` consumes
42.33% of evaluation. This authorizes only B4EP10P architecture research for
the active owner-plan/energy-fold subphase.
