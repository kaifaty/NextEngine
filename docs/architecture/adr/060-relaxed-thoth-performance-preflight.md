# ADR-060: Relaxed THOTH performance preflight

| Field | Value |
|---|---|
| ID | ADR-060 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-10 |
| Last verified | 2026-08-10 |
| Normative dependencies | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-049](049-performance-evidence-without-allocator-instrumentation.md) |
| Supersedes | Narrowly supersedes the ADR-036 THOTH hard-run preflight thresholds of CPU/GPU load below 5% and at least 20 GiB free RAM. It also supersedes the ADR-049 current methodology identity while preserving the Performance V4 wire shape. |
| Superseded by | The below-15% CPU/GPU threshold and methodology-v5 identity are superseded by [ADR-061](061-forty-percent-thoth-load-preflight.md). [ADR-082](082-linux-first-development-and-deferred-windows-host.md) supersedes the statement that THOTH is the active development host and defers this preflight until Windows bring-up. The 10 GiB free-RAM threshold remains Accepted for that future hard evidence. |

> Current load admission is below 40% with methodology v6 under ADR-061. The
> below-15% load values below are retained as decision history; the 10 GiB RAM
> threshold remains current.

## Context

THOTH has 32 GiB installed RAM and is also the active development host. The
previous requirement for 20 GiB free RAM excluded otherwise stable measurement
windows because ordinary host services and development tools commonly retained
more than 12 GiB. The below-5% CPU/GPU condition was similarly too narrow for a
repeatable local calibration window and rejected the observed 7% CPU state
before any workload measurement.

Preflight remains an evidence-quality filter. It does not replace release
build, exact-host, clean-commit, representative-workload, compatible-baseline,
profiler-integrity, resource-counter or authoritative-root requirements.

## Decision

THOTH hard-run and baseline preflight now requires:

- CPU load strictly below 15%;
- GPU load strictly below 15%;
- at least 10 GiB free physical RAM;
- CPU clock at least 80% of the reported maximum;
- no reported GPU software or hardware thermal slowdown.

Missing evidence remains a failure. Therefore 14% load is admitted and 15% is
rejected; exactly 10 GiB free RAM is admitted and one byte less is rejected.
The stable diagnostics are
`PERF_CPU_LOAD_AT_OR_ABOVE_FIFTEEN_PERCENT`,
`PERF_GPU_LOAD_AT_OR_ABOVE_FIFTEEN_PERCENT` and
`PERF_FREE_RAM_BELOW_TEN_GIB`.

This decision kept `PerformanceRunV4` and `PerformanceBaselineV4` current-only
wire
shape and file names. The methodology identity advances to
`nextengine-performance-v5`; artifacts carrying the previous methodology are
incompatible with new baselines and hard verdicts. Scenario hashes do not
change because workload bodies and measurement windows are unchanged.

## Consequences

- A normal THOTH development session can reach a ready preflight without
  terminating unrelated services solely to expose 20 GiB free RAM.
- New measurements may contain more background load than ADR-036 admitted, so
  ten-run baselines, retained raw samples and the no-retry policy remain
  mandatory.
- Existing stricter runs remain historical evidence but cannot be mixed into a
  methodology-v5 baseline without a new run.
- This decision does not make smoke a hard workload, close B-12, accept a
  PhysX-only cutover or change Windows/Linux shipping status.

## Product checks

| Check | Expected |
|---|---|
| `fast` | Boundary tests admit 14%/10 GiB, reject 15% and reject less than 10 GiB; report/baseline validation rejects methodology v4. |
| `performance --scenario smoke --mode report` | The observed THOTH preflight uses the new thresholds and emits methodology v5; the nested timing verdict remains `REPORT_ONLY`. |

## Rejected alternatives

- **Keep 5%/20 GiB.** Rejected because the active reference host repeatedly
  failed preflight before measuring a workload under otherwise non-throttled
  conditions.
- **Ignore preflight entirely.** Rejected because uncontrolled contention would
  make baseline and regression evidence less comparable.
- **Reuse methodology v4.** Rejected because it would silently mix runs admitted
  under different environmental thresholds.
