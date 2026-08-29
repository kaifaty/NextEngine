# NSR3-B4EP10SIRD source-local discriminator research -- 2026-08-22

Status: `COMPLETE / TIMING_REDUCTION_SELECTED`

## Question

B4EP10SIR rejects executor orchestration, partition imbalance, topology and
target-fold work as the immediate target. Its 58.77% source-local category is
still too broad: it contains allocation/setup, density work, HVP compression
and both evaluation/HVP directed kernels. Selecting an optimization from the
aggregate would be premature.

## Existing measurement boundary

No new instrumentation is required. The exact B4EP10SIR report already
contains positive, non-overlapping timers for all source-local phases. Reduce
them into four disjoint groups:

```text
directed = evaluation_directed + hvp_directed
setup = evaluation_setup + hvp_setup
compression = hvp_compression
local_scalar = evaluation_pair + evaluation_metadata
             + evaluation_density + evaluation_center
```

Their sum must equal `source_local_ns` exactly. The first B4EP10SIR snapshot
suggests that directed work may dominate, but one process is not evidence of
stable leadership.

Code inspection also shows why a later structural audit may be useful:

- `evaluation_directed` allocates/value-initializes a full directed `Vec3`
  array and overwrites only active-source rows before arithmetic;
- every HVP separately allocates/value-initializes compression, directed and
  target arrays, while split target folds read directed values only for active
  sources;
- HVP compression and HVP directed arithmetic traverse the same active source
  rows in canonical order, but combining them could alter data lifetime and
  must not be attempted from timing alone.

These are hypotheses, not authorization to reuse scratch storage, skip
initialization or fuse loops.

## Decision

Freeze B4EP10SIRD as an external reduction of three fresh exact B4EP10SIR
processes. Require stable shares, then select at most one structural audit
only if one group owns at least 20% of transaction time and leads the second
by `1.20x`. No code, physics, roots or timing instrumentation changes.
