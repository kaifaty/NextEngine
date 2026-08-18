# ADR-049: Performance evidence without allocator instrumentation

| Field | Value |
|---|---|
| ID | ADR-049 |
| Status | Accepted |
| Version | 1.1 |
| Decision date | 2026-08-08 |
| Last verified | 2026-08-08 |
| Normative dependencies | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-038](038-versioned-production-worker-handoff-diagnostic.md), [ADR-045](045-low-overhead-hard-performance-evidence.md) |
| Supersedes | Fully supersedes ADR-039 through ADR-043. Supersedes ADR-045 clauses that retain allocator instrumentation, allocator report fields or `allocator-counter-check`. Performance budgets, low-overhead evidence, retained historical results and no-retry policy remain. |
| Superseded by | The methodology identity and Performance V4 wire schemas are superseded by [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md), after intermediate ADR-060/061/062 revisions. [ADR-082](082-linux-first-development-and-deferred-windows-host.md) adds native Linux RSS/I/O report evidence and defers Windows hard execution. The allocator-removal and low-overhead resource-evidence decisions remain Accepted. |

> Current reports use the Performance V5 wire shape and
> `nextengine-performance-v8` under ADR-063. Earlier schemas/methodology strings below are
> retained as decision history.

## Context

The tooling-only global allocator counter required a separate unsafe crate,
target-specific TLS/codegen/symbol admission, a dedicated command and a large
fault-test surface. Its seventh retained candidate still exceeded the declared
overhead limit, and ADR-045 had already removed it from hard performance
admission. Keeping the subsystem no longer protected a current product result.

## Decision

Allocator instrumentation is not performance evidence for Next Engine. The
`process-allocation-counter` crate, global allocator hook, allocator probes,
special unsafe/dependency allowlists, native-gate rules and
`allocator-counter-check` are removed. Runtime and tooling use the ordinary
platform allocator without an engine wrapper.

ADR-049 introduced `PerformanceRunV4`,
`PerformanceResourceCountersV4` and `PerformanceBaselineV4` with methodology
`nextengine-performance-v4`. V2 and V3 readers were removed. Its V4
resource evidence contains:

- canonical logical host/device charges and their hash-bound charge root;
- process peak working set and measured-window process I/O deltas;
- conservative engine-owned GPU allocation ceilings and Vulkan timestamps;
- bounded profiler integrity and exact authoritative roots.

Measured physical counters remain observational. Canonical logical charges are
the budget evidence and do not choose authoritative work, fallback or eviction.

The five implemented scenarios keep their workload semantics: `smoke`,
`long-session-soak`, `interactive-frame-soak`, `production-worker-soak` and
`r2-alpha-render`. Each scenario hash and the common methodology version advance
once to record removal of the allocator window. No scenario is removed or
promoted to a hard gate by this change.

Historical allocator candidates remain facts in Git history, not live tooling
contracts. A clean ten-run THOTH baseline is still required for a representative
hard verdict. B-12 remains open until the existing ADR-036 requirements pass.

## Consequences

- The workspace returns to the FFI-only reviewed unsafe allowlist.
- Performance reports and baselines have one current alpha reader and no legacy
  compatibility branch.
- Logical charges, working set, process I/O, GPU ceilings, timestamps, profiler
  integrity, scenarios, budgets and authoritative roots are unchanged.
- Gameplay, persistence/replay, physics, Luau/Wasm and `CommandLedgerV2`
  semantics are unchanged.

## Product checks

| Check | Expected |
|---|---|
| `fast` | Workspace contains no allocator-counter crate, command, hook, probe or special boundary rule; current Performance V5 validation rejects retired wire versions under ADR-063. |
| `performance --scenario <implemented> --mode report` | All five implemented workloads emit V4 report-only evidence with unchanged workload counts and authoritative roots. |
| `performance-baseline` | Exactly ten compatible clean V4 reports produce one V4 baseline; V2/V3 inputs are incompatible. |
| `play` / `persistence-replay` | Authoritative state and ledger roots remain identical to the R2 baseline. |
