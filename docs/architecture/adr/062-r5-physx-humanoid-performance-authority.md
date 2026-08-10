# ADR-062: R5 PhysX humanoid performance authority

| Field | Value |
|---|---|
| ID | ADR-062 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-10 |
| Last verified | 2026-08-10 |
| Normative dependencies | [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-049](049-performance-evidence-without-allocator-instrumentation.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-059](059-event-sourced-physx-continuation-reconstruction.md), [ADR-061](061-forty-percent-thoth-load-preflight.md) |
| Supersedes | Narrowly supersedes the ADR-036 THOTH NVIDIA driver value `591.86` with installed driver `610.88`; replaces the R5 placeholder and 120 Hz PHYS-P4 workload wording with the method below; advances the ADR-061 methodology identity from v6 to v7. Other ADR-036/049/061 evidence, baseline and preflight rules remain Accepted. |
| Superseded by | none |

## Context

`r5-physics-16` previously returned `NOT_RUN` and PHYS-P4 still described a
120 Hz placeholder. Stage 0 now has a production PhysX articulation consumer,
fixed engine PD/safety, bounded event-sourced restore and exact canonical
roots. A hard gate needs one representative workload, accepted numeric budgets
and resource accounting without treating raw PhysX floats or wall-clock order
as authoritative state.

The current THOTH NVIDIA driver is `610.88`. Keeping `591.86` in the exact
fingerprint would make valid local evidence impossible and would encourage an
undeclared fingerprint override.

## Decision

### Workload and deterministic oracle

`r5-physics-16.v1` creates sixteen independent engine-owned
`nextengine.body.humanoid-23dof-v1` scenes through the production PhysX 5.9.0
adapter. Every slot has one 23-DoF reduced-coordinate articulation, the fixed
standing command/action, 240 Hz physics and 60 Hz motor cadence. A run performs
240 warm-up and 10,000 measured physics substeps per slot.

The same run root, semantic subject IDs, slot assignment and action stream are
executed with 1, 4 and 8 host workers. Workers own disjoint scenes and meet at
each motor-frame lockstep boundary. Completion order and wall time never enter
the canonical root. The run is invalid unless all worker modes and the
profiler-control execution produce the same engine-owned applied-action,
witness and final-snapshot root.

Restore starts a fresh scene for every slot from the canonical reset origin and
bounded post-safety effort prefix. Vendor serialization and hidden solver cache
are excluded. The benchmark reports checkpoint bytes per slot, per-slot restore
samples and replay-prefix overhead separately from live-frame latency.

### Accepted THOTH budgets

All budgets are inclusive maxima. Percentiles use the existing nearest-rank
rule without outlier removal.

| Metric | 1 worker | 4 workers | 8 workers |
|---|---:|---:|---:|
| lockstep motor-frame p95 / p99 | 20,000 / 25,000 us | 6,000 / 8,000 us | 4,000 / 6,000 us |
| aggregate physics cost | 250,000 ns/substep | 83,334 ns/substep | 50,000 ns/substep |
| aggregate motor cost | 1,000,000 ns/frame | 333,334 ns/frame | 200,000 ns/frame |
| minimum scaling efficiency | n/a | 65% | 55% |

The 8-worker 4/6 ms row is PHYS-P4. The 1/4-worker rows and scaling floors
detect scheduler/scene-sharding regressions; they do not redefine the shipping
frame deadline. Direct substeps/s and motor-frames/s remain visible report
fields. The generic lower-is-better baseline stores their reciprocal cost so
that higher throughput cannot be misclassified as a regression.

| Resource/restore metric | p95 maximum | p99 maximum |
|---|---:|---:|
| fresh-scene restore | 3,600,000 us | 4,000,000 us |
| checkpoint bytes per slot | 4 MiB | 4 MiB |
| replay-prefix overhead | 12,000 basis points | 12,000 basis points |
| process peak working set | 320 MiB | 320 MiB |
| logical host bytes per slot | 4 MiB | 4 MiB |

Restore is an explicit checkpoint operation and is not charged to the 4/6 ms
live motor-frame deadline. Replay-prefix overhead compares fresh-scene restore
wall time with the estimated live time of the same prefix on the 8-worker run.

### Evidence and methodology identity

The THOTH exact fingerprint now names NVIDIA driver `610.88`. The global
methodology identity becomes `nextengine-performance-v7`; older V4 run and
baseline files remain decodable but are incompatible with v7 calibration and
hard verdicts. The `r5-physics-16.v1` scenario hash binds slot count, DoF,
cadence, warm-up/measurement sizes, worker modes, controller, restore method,
root parity and logical-accounting profile.

CPU PhysX does not issue Vulkan work. Therefore R5 hard evidence records zero
engine-owned device residency and does not fabricate Vulkan timestamp queries.
It still requires profiler integrity, Windows process counters, canonical
logical charges, exact roots, clean `release`, ready THOTH preflight and a
compatible ten-run baseline.

The report-only calibration of clean commit
`ad333c206610af28e29cb03f916d17aa10e99126` established the budget
scale: 8-worker frame p95/p99 `2,444/2,607 us`, fresh-scene restore
`2,536,057 us` p95/p99, checkpoint `2,000,518 bytes`, replay-prefix overhead
`8,430` basis points, process peak working set `208,486,400 bytes`, logical
host charge `2,130,237 bytes/slot`, and exact root
`0841ca8674fac89f513f1a6f3ccbc2e155265265b055bc7e45e39b2b0ae2858b` for
1/4/8 workers and profiler control. This calibration is provenance for the
budgets, not the required ten-run baseline or hard PASS.

## Consequences

- `r5-physics-16` is a real representative workload rather than a placeholder.
- A report can fail an absolute R5 budget before baseline comparison.
- Exact roots remain the correctness oracle; timing variance cannot repair a
  deterministic mismatch.
- The ready THOTH preflight and one calibration do not close B-12, Stage 0,
  PhysX cutover or R5. Ten clean compatible runs, a published local baseline,
  a hard gate, Linux report-only evidence and the other documented product
  checks remain separate facts.

## Product checks

| Check | Expected |
|---|---|
| `performance --scenario r5-physics-16 --mode report` | Executes the 16-slot production PhysX workload, reports 1/4/8 throughput/latency/scaling, restore/resources and exact roots; without a baseline the timing verdict remains `REPORT_ONLY`. |
| `performance-baseline --runs <ten-run-root> --output <dir>` | Accepts exactly ten clean compatible v7 THOTH reports with ready preflight, profiler integrity, complete budgets/resources and identical authoritative roots. |
| `performance --scenario r5-physics-16 --mode gate --target ref-win-thoth-v1 --baseline <file>` | Produces hard `PASS` only when current absolute budgets, compatible baseline, exact fingerprint/root and all hard evidence pass. |
| `fast`, `persistence-replay` | Budget/schema tests and worker/restore root parity stay exact; replay correctness is not inferred from timing success. |

## Rejected alternatives

- **Gate direct throughput as a lower-is-better metric.** Rejected because the
  common regression comparator would treat an improvement as a failure.
- **Charge restore to every live frame.** Rejected because restore is a bounded
  checkpoint operation, not part of the 240/60 Hz control loop.
- **Use raw PhysX checkpoint floats as the parity root.** Rejected because the
  engine-owned canonical boundary, not vendor-private state, is authoritative.
- **Treat one calibration as a baseline.** Rejected because ADR-036 requires
  exactly ten clean compatible runs and forbids retry-to-green closure.
