# ADR-094: Confidence-gated relative performance warnings

| Field | Value |
|---|---|
| ID | ADR-094 |
| Status | Accepted |
| Version | 1.0 |
| Decision date | 2026-08-22 |
| Last verified | 2026-08-22 |
| Normative dependencies | [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-063](063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-091](091-linux-release-performance-authority.md), [ADR-092](092-dimensional-relative-performance-comparison.md), [ADR-093](093-deterministic-r5-worker-placement.md) |
| Supersedes | Narrowly supersedes the warning clause of the ADR-063/091/092 relative policy: a warning now additionally requires the lower bootstrap bound to reach the warning threshold. Absolute budgets, FAIL semantics, ten-run calibration, fixed three-run batches, preflight and no-retry rules remain Accepted |

## Context

Three independent evidence events show the accepted point-estimate warning
rule misclassifying order-statistic noise as regressions:

1. `9dc6919` R5 gate: `worker-8.physics-motor-frame` warned at `+589bp` with
   CI `[-1952, +884]`;
2. `f3cb715` first R5 gate attempt: one fresh-process member returned verdict
   `FAIL`, but its report was unparsable by the parent because failed
   commands append a diagnostic JSON to stdout, so the batch never assembled;
3. `f3cb715` second R5 gate attempt: a fully assembled immutable batch warned
   solely on `worker-4.physics-motor-frame` at `+205bp` with CI
   `[-293, +782]`, while its substep cost was flat (`+12bp`) and every
   absolute budget passed.

The current policy is asymmetric: `FAIL` requires both the change and the
lower 95% confidence bound to reach `500bp`, but `WARNING` fires on the point
estimate alone at `200bp`. With observed between-run spreads of `±10–15%` on
lockstep frame rows even under ADR-093 placement, any three-member gate has a
substantial probability of tripping that line in either direction, and each
trip publishes an immutable artifact that blocks release closure without
evidence of a real regression.

## Decision

The relative verdict policy becomes symmetric in its use of the already
computed deterministic run-level bootstrap interval:

- `FAIL`: unchanged — change `>=500bp` and CI95 lower bound `>=500bp`;
- `WARNING`: change `>=200bp` **and** CI95 lower bound `>=200bp`.

A point estimate at or above the warning line whose confidence interval still
contains material improvement is recorded as relative evidence but does not
warn. Real regressions narrow the interval; noise cannot clear the lower
bound. Absolute budgets, exact roots, environment boundaries and all other
ADR-063 controls are unchanged.

### Workload and tooling identity

The methodology identity advances to `nextengine-performance-v11`; v10 and
older baselines are incompatible with v11 gates through the existing
methodology admission. Additionally, failed commands now emit their
diagnostic JSON on stderr only, keeping stdout a single machine-readable
report channel so failing gate members can always be aggregated; their real
metrics become preservable negative evidence instead of parse errors.

## Consequences

- Warning artifacts now carry decision-grade uncertainty information; noise
  no longer produces immutable blocking evidence.
- Genuine regressions keep both detection paths: large point estimates with
  tight intervals warn or fail exactly as before.
- All four representative workloads must be recollected under v11 on one
  exact clean commit for R7c closure; earlier v10 sets remain immutable
  historical evidence.
- The stdout pollution defect can no longer discard a failing member's
  measured metrics.

## Product checks

| Check | Expected |
|---|---|
| Focused verdict tests | Point estimates above `200bp` with CI low bounds below `200bp` do not warn; boundary values warn inclusively; FAIL cases unchanged |
| Baseline compatibility | A v10 baseline is rejected by a v11 gate rather than relabelled |
| Failing-member aggregation | A member with verdict `FAIL` produces parseable single-document stdout so the batch aggregates and publishes its verdict |
| Fresh four-scenario evidence | Ten-run baselines plus isolated fixed three-run gates for R2–R5 on one exact clean commit pass all absolute and CI-gated relative requirements |

## Rejected alternatives

- **Raise the warning threshold above `200bp`.** Rejected: weakens
  sensitivity to genuine small regressions; ADR-092 already rejected
  widening thresholds.
- **Keep collecting until quiet conditions produce PASS.** Rejected:
  statistically coin-flipping immutable artifacts is retry-to-green in
  effect.
- **Make the w4 rows absolute-only.** Rejected: removes regression
  sensitivity for direct costs without principle; ADR-092 confined that
  treatment to already-normalized ratios.
