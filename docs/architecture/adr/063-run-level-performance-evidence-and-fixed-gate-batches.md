# ADR-063: Run-level performance evidence and fixed gate batches

| Field | Value |
|---|---|
| ID | ADR-063 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-10 |
| Last verified | 2026-08-10 |
| Normative dependencies | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-045](045-low-overhead-hard-performance-evidence.md), [ADR-049](049-performance-evidence-without-allocator-instrumentation.md), [ADR-061](061-forty-percent-thoth-load-preflight.md), [ADR-062](062-r5-physx-humanoid-performance-authority.md) |
| Supersedes | The ADR-036 raw-sample relative-bootstrap rule and the ADR-062 single-candidate-run gate semantics. Absolute budgets, the 5% relative threshold, exact roots, ten-run calibration, THOTH fingerprint, preflight thresholds and no-retry policy remain Accepted. |
| Superseded by | [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md) removes Windows/THOTH from current v1/R7 authority. [ADR-091](091-linux-release-performance-authority.md) advances current Linux evidence to Performance V6/methodology v9 while retaining the ten-run baseline, fixed three-run batch, whole-run bootstrap and no-retry semantics. |

## Context

The first R5 hard gate used one candidate workload run and compared its frame
samples with all frame samples concatenated from ten calibration runs. Frames
inside one process run share scheduling, temperature, clock and background-load
conditions, so they are not independent experimental units. Bootstrapping them
as independent observations produced an unjustifiably narrow interval and let
one uniformly slower host window dominate the relative verdict.

The failed artifact remains valid historical evidence under methodology v7. It
must not be retried until it passes, but it also must not define the corrected
methodology.

## Decision

### Independent evidence unit

One complete workload invocation is the independent evidence unit. Performance
V5 stores every raw sample plus explicit `sample_run_lengths`; consumers can
reconstruct run boundaries without inferring them from metric names or sample
counts. `PerformanceRunV5` also records `evidence_runs` and the ordered host
environment samples for those runs.

A baseline contains exactly ten independent clean release reports from one
commit. Every calibration report contributes exactly one run per metric and a
full start/postflight environment pair. Concatenating raw samples remains
useful for inspection, but relative statistics never treat those samples as
independent runs.

### Fixed hard-gate batch

One hard-gate command is one immutable batch of exactly three independent
workload runs. The command publishes one V5 report only after all three runs
complete. A failed or invalid member invalidates the batch; members cannot be
dropped, selected or rerun inside the batch. A later command is new evidence,
not a continuation or repair of the previous batch.

For every metric:

- raw samples from all three candidate runs are retained with their boundaries;
- the absolute p95/p99 verdict uses the worst per-run p95/p99, so aggregation
  cannot hide one budget overrun;
- the relative point estimate compares the median candidate-run p95 with the
  median of the ten baseline-run p95 values;
- the deterministic 95% interval bootstraps whole run-p95 observations and
  recomputes the median; it never resamples individual frames across a run
  boundary;
- `<2%`, `2..5%` and `>=5%` with a lower interval bound of at least 5% retain
  the existing noise, warning and failure meanings.

The methodology identity is `nextengine-performance-v8`. V4/v7 run and
baseline files are current-only historical artifacts and are incompatible with
V5/v8 calibration or gates.

### Environment integrity

The full THOTH readiness check runs immediately before every independent run:
CPU/GPU load are strictly below 40%, free RAM is at least 10 GiB, CPU clock is
at least 80% of maximum and GPU thermal slowdown is clear. Immediately after
the measured workload, a second sample checks free RAM, clock and thermal
integrity. Postflight records CPU/GPU utilization but does not apply the idle
threshold to utilization created by the benchmark itself.

Heavy background polling inside a timed workload is forbidden because the
observer would perturb the measurement. Missing samples, changed fingerprint,
capacity overflow, incompatible roots or any failed required boundary sample
produce `NOT_RUN`.

## Consequences

- One noisy host window can still affect one run, but cannot masquerade as
  thousands of independent observations.
- Hard gates take three times the workload duration and retain more raw data.
- The failed v7 R5 gate remains a non-retried historical fact; it neither passes
  nor fails v8.
- Under ADR-082 B-12 remains open as a deferred R7/release gate until a fresh
  ten-run v8 baseline and one fixed three-run hard-gate batch pass on the
  required future Windows commit and host; it is no longer a current R5
  development handoff condition.

## Product checks

| Check | Expected |
|---|---|
| `performance-baseline --runs <ten-run-root> --output <dir>` | Accepts exactly ten compatible V5 reports, each with one metric run and two valid environment boundary samples. |
| `performance --scenario r5-physics-16 --mode gate --target ref-win-thoth-v1 --baseline <v5-file>` | Executes exactly three independent runs, retains run boundaries, applies worst-run absolute budgets, compares run-level medians and publishes one verdict. |
| focused `xtask` tests | Reject malformed run boundaries and incomplete batches; prove absolute worst-run behavior, deterministic run-level bootstrap and aggregation of profiler/resource evidence. |
| `fast`, conditional `performance` | Tooling remains buildable and a fresh v8 calibration/gate is reported separately; implementation alone does not close B-12. |

## Rejected alternatives

- **Raise the 5% regression threshold.** Rejected because it hides real
  regressions without fixing the independence error.
- **Pool all frames and use a clustered label only in documentation.** Rejected
  because the serialized evidence would still permit the invalid computation.
- **Retry a single candidate until it resembles the baseline.** Rejected by the
  no-retry policy and because it selects favorable host noise.
- **Continuously poll WMI/NVML inside the timed window.** Rejected because the
  probe becomes part of the workload and changes the quantity being measured.
