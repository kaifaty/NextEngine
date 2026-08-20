# ADR-045: Low-overhead hard performance evidence

| Field | Value |
|---|---|
| ID | ADR-045 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-06 |
| Last verified | 2026-08-06 |
| Normative dependencies | [SPEC-00](../00-product-contract.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-039](039-tooling-only-process-wide-system-global-allocator-measurement.md), [ADR-040](040-fixed-tls-sharded-global-allocator-measurement.md), [ADR-041](041-owner-thread-quiescent-global-allocator-measurement.md), [ADR-042](042-unobserved-deallocation-system-pass-through.md), [ADR-043](043-codegen-proven-non-reentrant-count-bearing-allocator-callbacks.md) |
| Supersedes | Narrowly supersedes the ADR-036 and ADR-039–ADR-043 clauses that make an exact active `ProcessAllocationCounterV1` mandatory for a hard run, baseline publication or B-12. Their allocator boundary, exact counter semantics, retained failed candidates and no-retry evidence policy remain Accepted. |
| Superseded by | Allocator-retention clauses and Performance V2/V3 tooling schemas are superseded by [ADR-049](049-performance-evidence-without-allocator-instrumentation.md). [ADR-090](090-linux-only-v1-and-indefinitely-deferred-windows.md) removes Windows hard execution from current v1/R7 scope. Low-overhead evidence and no-retry policy remain reusable for a future accepted Linux profile; THOTH budgets remain historical. |

> Current tooling uses Performance V5 without allocator fields under ADR-063. The V2/V3/V4 and
> allocator sections below are retained only as decision history; ADR-049 is
> the current contract.

## Context

The admitted Windows allocator counter is exact, but the retained candidate-7
measured `+4.30%` overhead and therefore failed its own `3%` integrity budget.
Continuing to require that counter inside every hard timing window creates a
contradiction: the evidence source invalidates the measurement it is required
to authorize.

SPEC-23 already defines canonical logical resource charges as the decision
input. Peak working set, process I/O and conservative device allocation are
independent low-overhead observations. Together with bounded profiler evidence
and exact authoritative roots they cover the v1 performance gate without
promoting allocator layout to authority.

## Decision

### Current tooling schemas

Current commands emit `PerformanceRunV3`, `PerformanceResourceCountersV3` and
`PerformanceBaselineV3` with methodology `nextengine-performance-v3`.
`PerformanceRunV2`, `PerformanceResourceCountersV2` and
`PerformanceBaselineV2` remain strict decode-only historical evidence. A V2
artifact cannot enter a V3 baseline or produce a current hard verdict.

`PerformanceResourceCountersV3` contains:

- Windows process peak working set;
- process read/write I/O deltas over the measured window;
- the conservative ceiling of engine-owned device allocations;
- bounded Vulkan timestamp query evidence;
- canonical logical host/device charges bound to an accounting-profile hash
  and a canonical charge root;
- optional exact `ProcessAllocationCounterV1` diagnostics.

Report mode may leave scenario-specific logical charges or device/GPU values
unavailable and remains `REPORT_ONLY`. A hard run and every run admitted to a
baseline require all low-overhead counters above, profiler integrity, the
applicable absolute budgets and unchanged authoritative roots. Missing or
invalid required evidence is `NOT_RUN`, never `PASS`.

### Allocator diagnostics

Normal V3 timing windows do not activate allocator measurement. The wrapper may
remain linked but inactive, and `allocator-counter-check` remains the dedicated
diagnostic path. Absence or an explicit allocator-only unavailable diagnostic
does not invalidate a V3 hard run.

If allocator evidence is present, all top-level totals, the exact V1 payload,
scenario/hash binding and contradiction checks remain mandatory. Partial or
malformed allocator evidence invalidates the run. The counter remains gross
successful `alloc`/`alloc_zeroed`/`realloc` traffic, not live heap or peak RSS.

### Evidence immutability

Candidates 1–7 and their failures remain retained. An unchanged failed run may
not be repeated to obtain a green result. A new hard run requires a material
implementation or workload change and a new immutable artifact. ADR acceptance,
schema availability, smoke results and fallback-only runs do not close B-12.

## Consequences

- Hard timing no longer waits for a future zero-overhead exact allocator
  implementation.
- Resource budgets use canonical logical charges; physical counters remain
  observational safety ceilings and diagnostics.
- The Windows portion of B-12 still requires ten valid release runs for every
  representative R2–R5 workload on the exact clean commit and THOTH profile.
- This decision makes no Linux admission or cross-target claim. Linux evidence
  remains deferred and R1/R7 cannot close from Windows-only results.

## Rejected alternatives

- **Treat candidate-7 as close enough.** Rejected: `+4.30%` exceeds the Accepted
  integrity limit.
- **Approximate allocator counters.** Rejected: they would duplicate RSS and
  logical charges while losing the exact diagnostic semantics.
- **Remove the allocator tooling.** Rejected: it remains useful for focused
  regressions and preserves the retained evidence line.
- **Use OS counters as authoritative admission inputs.** Rejected by SPEC-23;
  host allocator layout and cache state must not choose simulation outcomes.
