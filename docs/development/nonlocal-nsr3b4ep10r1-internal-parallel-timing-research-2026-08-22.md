# NSR3-B4EP10R1 internal parallel timing research -- 2026-08-22

Status: `COMPLETE / HIERARCHICAL_ACTIVE_INTERVAL_TIMING_SELECTED`

## Input

B4EP10R cannot route its PC samples because gprofng marks them unreliable and
native sync interception sees no libgomp waits. B4EP10S remains exact and
selects 8 workers. The next method must measure the selected path directly,
keep all durations outside semantic evidence and remain absent from old
commands.

## Selected instrumentation

Add one opt-in command at 8 workers. Its executor retains the same 64 logical
partitions and output ownership. Only when timing is enabled, each OpenMP
region uses this measurement shape:

1. the serial caller records region wall start;
2. after team establishment, every worker records its own active start;
3. `omp for schedule(static,1) nowait` executes unchanged partition kernels;
4. each worker records active end into a unique padded slot;
5. an explicit barrier preserves the original completion boundary;
6. the caller records region wall end and sums/maxes active intervals.

This yields non-overlapping capacity estimates per region:

```text
active        = sum(worker active intervals)
imbalance     = workers * max(active interval) - active
orchestration = workers * (region wall - max(active interval))
```

Their sum is `workers * region wall`, modulo checked timer arithmetic. Active
time includes scheduling and kernel memory stalls but excludes a worker's
post-work barrier wait. No timing write is shared between workers.

## Hierarchical phases

Measure transaction and top-level topology/evaluation/HVP wall. Within them,
measure consecutive subphases:

- topology: setup, active flags, prefix, compaction, metadata, row count,
  offsets, row fill, finalize;
- evaluation: setup, pair coefficients, metadata, density rows, centre terms,
  owner plan/energy fold, directed values, target gather, finalize;
- HVP: setup, compression direction, directed values, target gather, finalize.

Every successful topology/evaluation subphase has 226 calls; every HVP
subphase has 459. Component sums must not exceed their enclosing stage, and
the three stages must not exceed transaction wall.

Run three fresh pinned processes. Require exact B4EP10I correspondence and
one common semantic result excluding durations. Phase/capacity shares must
vary by at most 0.05 across runs.

## Routing

Use median executor capacity shares first:

1. orchestration at least 0.20 selects persistent-team/region amortization;
2. otherwise imbalance at least 0.20 selects partition/load-balance research;
3. otherwise a top-level stage leading the runner-up by at least `1.20x`
   selects that stage's leading subphase for one next design;
4. otherwise use deeper scoped timing and select no optimization.

The instrumented processes do not measure throughput and cannot revise the
B4EP10S worker count.

## Decision

Freeze B4EP10R1 before implementation. Old/default/B4EP10I commands must stay
byte-exact and use the uninstrumented executor branch.
