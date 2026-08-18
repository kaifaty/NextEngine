# ADR-061: Forty-percent THOTH load preflight

| Field | Value |
|---|---|
| ID | ADR-061 |
| Status | Accepted |
| Version | 1.1 |
| Decision date | 2026-08-10 |
| Last verified | 2026-08-10 |
| Normative dependencies | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-049](049-performance-evidence-without-allocator-instrumentation.md), [ADR-060](060-relaxed-thoth-performance-preflight.md) |
| Supersedes | Narrowly supersedes the ADR-060 CPU/GPU load threshold of below 15% and its methodology-v5 identity. The ADR-060 minimum of 10 GiB free RAM remains Accepted. |
| Superseded by | [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md) supersedes the methodology identity and adds per-run environment boundaries after ADR-062. [ADR-082](082-linux-first-development-and-deferred-windows-host.md) defers THOTH execution until Windows bring-up; the below-40% start-load and 10 GiB free-RAM admission remain Accepted for that future hard evidence. |

## Context

The first report after adopting ADR-060 observed 37% CPU and 34% GPU load.
Those values were below the explicitly requested operational ceiling of 40%
but still failed the interim below-15% rule. The reference host is used for
active local development, so the preflight must distinguish heavy contention
from a normal occupied workstation without pretending that smoke evidence is a
hard performance result.

## Decision

THOTH hard-run and baseline preflight now requires:

- CPU load strictly below 40%;
- GPU load strictly below 40%;
- at least 10 GiB free physical RAM;
- CPU clock at least 80% of the reported maximum;
- no reported GPU software or hardware thermal slowdown.

Missing evidence remains a failure. Therefore 39% load is admitted and 40% is
rejected. The load diagnostics become
`PERF_CPU_LOAD_AT_OR_ABOVE_FORTY_PERCENT` and
`PERF_GPU_LOAD_AT_OR_ABOVE_FORTY_PERCENT`. The existing
`PERF_FREE_RAM_BELOW_TEN_GIB` diagnostic remains current.

This decision retained the current-only `PerformanceRunV4` and
`PerformanceBaselineV4` wire
shape and file names. The methodology identity advances to
`nextengine-performance-v6`; methodology-v5 and older artifacts are
incompatible with new baselines and hard verdicts. Scenario hashes remain
unchanged because workload bodies and measurement windows did not change.

## Consequences

- The previously observed 37% CPU / 34% GPU state passes the load portion of
  preflight when all other evidence is valid.
- Measurements may contain substantially more background activity than under
  ADR-036 or ADR-060. Ten clean runs, retained raw samples, compatible exact
  fingerprints and the no-retry policy remain mandatory.
- This change does not provide a representative workload, baseline, release
  run or hard timing verdict. B-12 and the PhysX cutover performance gate remain
  open.

## Product checks

| Check | Expected |
|---|---|
| `fast` | Boundary tests admit 39%/10 GiB, reject 40% and reject less than 10 GiB; current validation rejects methodology v5 and older artifacts. |
| `performance --scenario smoke --mode report` | Host probe emits methodology v6 and applies the below-40% load threshold; timing remains `REPORT_ONLY`. |

## Rejected alternatives

- **Treat 40% as inclusive.** Rejected to preserve the existing exclusive
  ceiling convention and an unambiguous integer boundary.
- **Keep methodology v5.** Rejected because it would mix baseline inputs
  admitted under materially different load ceilings.
- **Let a ready smoke report close the performance gate.** Rejected because
  smoke has no hard budget and is not a representative R2-R5 workload.
