# NSR3-B4EP10SIR split incoming residual research -- 2026-08-22

Status: `COMPLETE / INTERNAL_TIMING_SELECTED`

## Question

B4EP10SII is exact and clears its performance gate, but only by `1.052521x`
and with higher total CPU. Its change simultaneously removes active-plan
construction, adds three topology-time builder regions and scans inactive
incoming entries. Another optimization cannot be selected from the A/B result
alone.

## Available discriminator

The existing B4EP10R1 instrumentation already measures one transaction,
topology/evaluation/HVP stages, 23 non-overlapping subphases and worker active
intervals. Enabling it only for the split-incoming command can reconstruct four
disjoint architecture categories:

1. complete topology, including the currently unattributed incoming builder;
2. source-local floating work: evaluation setup/pair/metadata/density/center/
   directed plus HVP setup/compression/directed;
3. evaluation and HVP target folds;
4. remaining plan/finalization/nonlinear-control work.

The categories sum to the measured transaction duration. Durations remain
diagnostic and are excluded from the semantic result.

## Routing rule

Run three fresh exact processes on the selected eight physical cores. First
reject unstable measurements. Then route persistent-region or partition-
balance research only if median executor orchestration or imbalance reaches
15%. Otherwise select one architecture category only when it owns at least 20%
of transaction time and leads the second category by at least `1.20x`. With no
leader, select no optimization and design a smaller discriminator.

This avoids interpreting B4EP10SII's increased CPU as proof that construction
fusion is necessarily next. The timing must distinguish it from unchanged
source-local HVP/evaluation work and the deliberately expanded target scans.

## Decision

Freeze B4EP10SIR as an opt-in candidate-specific timing command. It changes no
physics, plan ownership, worker policy or A/B claim and authorizes at most one
later mechanical research discriminator. B4E2, broad corpus, runtime, GPU,
schema and production remain blocked.
