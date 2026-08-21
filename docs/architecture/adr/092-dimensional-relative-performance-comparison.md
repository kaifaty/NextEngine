# ADR-092: Dimensional relative performance comparison

| Field | Value |
|---|---|
| ID | ADR-092 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-21 |
| Last verified | 2026-08-21 |
| Normative dependencies | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-062](062-r5-physx-humanoid-performance-authority.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-091](091-linux-release-performance-authority.md) |
| Supersedes | Narrowly supersedes the generic relative-comparison requirement of ADR-036/063/091 for normalized R5 ratio metrics; all absolute budgets, direct-cost relative checks, exact roots, ten-run calibration, fixed three-run batch, preflight and no-retry rules remain Accepted |

## Context

The first clean Linux R5 baseline/gate on `feae4f0` passed every absolute row
and exact-root invariant but returned `WARNING`. After fresh-process gate-member
isolation, the sole gate on `3bbc19e` again passed every absolute row and exact
root but returned `FAIL` only for 4-worker scaling inefficiency:
`1108 -> 1535 basis points`.

That result did not represent a slower four-worker workload. Its direct
physics-substep cost changed by `-0.25%`; one-worker cost improved by about 5%
and eight-worker cost improved by about 7.5%. Scaling inefficiency is already a
dimensionless ratio against the same run's one-worker result. Applying the
generic percentage comparator to that ratio divided by a changing denominator
again and misclassified a faster reference denominator as a regression.
Replay-prefix overhead has the same shape: it is already a ratio of restore
wall time to an estimated live prefix.

## Decision

Performance V6 remains the wire format. The methodology identity advances to
`nextengine-performance-v10`; v9 and older baselines are incompatible with v10
gates.

The canonical metric policy now owns two comparison classes:

- direct duration, reciprocal cost and byte metrics keep both their unchanged
  absolute budgets and ADR-063 whole-run relative bootstrap;
- `r5-physics-16.worker-4.scaling-inefficiency`,
  `r5-physics-16.worker-8.scaling-inefficiency` and
  `r5-physics-16.replay-prefix-overhead` are absolute-only normalized ratios.

Absolute-only does not mean diagnostic-only. All raw samples and run boundaries
remain in reports/baselines, worst-run p95/p99 must pass their existing
ADR-062 ceilings, and a budget overrun remains `FAIL`. These metrics simply do
not receive a second percent-over-percent comparison and serialize
`relative: null` in the gate report.

Metrics without a canonical absolute budget remain report-only as before.
Fresh-process gate members, exact authoritative roots, environment boundaries,
profiler integrity and resource counters remain mandatory independently of the
comparison class.

## Consequences

- A faster one-worker denominator cannot manufacture a relative scaling
  regression while the multi-worker path itself is unchanged or faster.
- Direct frame latency, reciprocal throughput costs, restore duration and
  physical/logical byte metrics retain the 2% warning and significant 5%
  failure policy.
- Existing absolute product ceilings are not widened and failed/warning v9
  evidence remains immutable historical evidence.

## Product checks

| Check | Expected |
|---|---|
| Focused metric-policy tests | The three normalized R5 ratios retain exact budgets but are marked absolute-only; direct R5 frame cost remains relative-comparable |
| Baseline compatibility | A v9 baseline is rejected by a v10 gate rather than relabelled |
| Fresh R5 baseline/gate | Ten clean reports plus one isolated fixed-three gate retain all ratio samples, emit `relative: null` only for the three declared ratios and pass every unchanged absolute/direct-relative/root/environment requirement |

## Rejected alternatives

- **Raise the global 2%/5% thresholds.** Rejected because direct costs would
  lose regression sensitivity.
- **Widen R5 absolute ratio budgets.** Rejected because the recorded runs
  already pass the product ceilings.
- **Remove the ratio metrics.** Rejected because absolute scaling and restore
  overhead are still release requirements.
- **Retry either recorded gate.** Rejected by the no-retry policy; v10 requires
  a new clean commit and evidence set.
