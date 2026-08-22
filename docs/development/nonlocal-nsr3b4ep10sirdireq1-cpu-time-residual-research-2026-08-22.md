# NSR3-B4EP10SIRDIREQ1 CPU-time residual research -- 2026-08-22

Status: `COMPLETE / PASS / TOPOLOGY_STRUCTURAL_AUDIT_SELECTED`

## Question

Can residual solver work still be attributed on the shared desktop host after
SIRDIREQ rejects short-margin wall timing?

Linux `CLOCK_PROCESS_CPUTIME_ID` measures CPU consumed by all threads in one
process, while `CLOCK_THREAD_CPUTIME_ID` measures only the calling thread.
Unlike `steady_clock` wall intervals, neither advances while the benchmark is
descheduled by another process. The local Linux 7.0/glibc 2.43 host reports
1 ns resolution for both clocks and exposes the POSIX CPU-time capability.
The relevant semantics are also documented by the
[Linux man-pages project](https://man7.org/linux/man-pages/man2/clock_gettime.2.html)
and [POSIX](https://pubs.opengroup.org/onlinepubs/9799919799/functions/clock_gettime.html).

CPU time is not throughput. It can rank algorithmic work on this host, but it
does not measure latency, contention with the rest of the engine, frequency/
thermal effects or the final gameplay frame budget.

## Selected measurement

Add one opt-in command over unchanged SIRDI and its existing SIRDIR disjoint
phase hierarchy:

- process CPU intervals for the transaction, topology/evaluation/HVP totals,
  all 6,363 phase invocations and every OpenMP region;
- per-worker thread CPU intervals around active static-partition work;
- fail-closed clock reads, checked nanosecond conversion and exact call/sum
  accounting;
- the existing wall timing remains report-only parent evidence and cannot
  grant speed credit.

At the frozen nominal trace this is exactly 7,275 process phase/total
intervals, 4,089 process region intervals and 32,712 worker-active intervals.
Clock reads are bounded at 14,550, 8,178 and 65,424 respectively. Resolution
must be at most 1,000 ns.

## Disjoint routing categories

The prior `source_local` aggregate is no longer a useful implementation target
because its leading setup-buffer branch is stopped. Split process CPU into:

```text
topology
target_fold
directed = evaluation_directed + hvp_directed
hvp_compression
other_local = source_local - evaluation_setup - directed - hvp_compression
stopped_evaluation_setup
control = transaction - topology - source_local - target_fold
```

All seven categories must sum exactly to transaction CPU time. The stopped
setup category remains visible but cannot win routing. Across three fresh
processes, every category share must have range at most 0.03. An eligible
category must own at least 20% and lead the second eligible category by 1.20x;
otherwise select no optimization and narrow CPU timing again.

GNU user+system time provides an external total-CPU cross-check. Its median
ratio over the internal transaction CPU must remain in `[1.00, 1.05]`; the
small positive remainder covers command/report work outside the transaction.

## Decision

Freeze B4EP10SIRDIREQ1 as CPU attribution only. It may select one timing-free
structural audit, never an implementation or a wall-speed claim. SIRDI remains
selected; B4E2, broad corpus, runtime/GPU/schema and production stay blocked.

The frozen execution passes `3/3`. Topology owns median process-CPU share
`0.301239`, leads the next eligible category by `1.629380x`, and therefore
selects exactly one topology structural audit; see the
[dated evidence](nonlocal-nsr3b4ep10sirdireq1-cpu-time-residual-evidence-2026-08-22.md).
