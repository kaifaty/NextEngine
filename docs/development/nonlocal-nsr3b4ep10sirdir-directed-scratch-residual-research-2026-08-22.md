# NSR3-B4EP10SIRDIR directed scratch residual research -- 2026-08-22

Status: `COMPLETE / CANDIDATE_TIMING_SELECTED`

## Question

B4EP10SIRDI preserves the complete nominal Hydro result and reduces median
wall from 5.514 s to 4.290 s by replacing 454.9M repeated full-buffer
initializations with one transaction-local high-water scratch buffer. The old
B4EP10SIR timing predates that change, so its 58.77% source-local attribution
is no longer evidence about the selected candidate.

Which bounded architecture category owns the remaining candidate cost, and is
executor overhead large enough to justify changing the parallel structure?

## Competing hypotheses

1. Source-local evaluation/HVP work remains the leader because arithmetic over
   374.9M active slots still dominates after initialization is removed.
2. Complete topology or the expanded split-incoming target fold becomes the
   leader once source-local allocation/initialization disappears.
3. Nonlinear control, finalization, executor orchestration or partition
   imbalance becomes material enough that another arithmetic/data-layout
   change would attack the wrong layer.

No hypothesis is selected from the external A/B result alone.

## Smallest discriminator

Reuse the existing B4EP10R1/SIR steady-clock instrumentation unchanged over
the exact B4EP10SIRDI execution. Reconstruct the same four disjoint categories:

```text
topology = topology_total
source_local = evaluation setup + pair + metadata + density + center
             + directed + HVP setup + compression + directed
target_fold = evaluation target + HVP target
control = transaction_total - topology - source_local - target_fold
```

The new command additionally proves the complete SIRDI semantic result and all
685 scratch-reuse calls, including one release and zero live buffers. Timing
values remain outside every physical root, failure class and semantic result.

Run three fresh serialized processes pinned to CPUs `0..7`. Require stable
top-level and category shares before routing. Preserve the existing decision
thresholds: executor orchestration or imbalance must reach 15%; otherwise one
category must own at least 20% and lead the second by `1.20x`. If no condition
holds, select no optimization and narrow the measurement again.

## Decision

Freeze B4EP10SIRDIR as an opt-in candidate-specific timing command. It changes
no formula, solver policy, scratch ownership, worker count or speed claim and
authorizes at most one next mechanical discriminator. B4E2, broad corpus,
runtime, GPU, schema and production remain blocked.
