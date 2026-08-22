# NSR3-B4EP10SIRDIRE evaluation setup research -- 2026-08-22

Status: `COMPLETE / SETUP_SEGMENT_TIMING_SELECTED`

## Question

B4EP10SIRDIR selects source-local work at 44.72% after directed scratch reuse.
The old directed group no longer has a clear lead, while
`evaluation_setup` is the largest individual source-local subphase at 14.58%
of the transaction. What part of setup is actually expensive?

## Competing hypotheses

The current function performs three architecturally distinct operations before
pair evaluation begins:

1. structural validation scans every flat offset row and all 150,845,996
   directed indices accumulated by the nominal transaction;
2. capacity/control computes checked payload bounds;
3. buffer preparation resizes/value-initializes gradient, density,
   compression, radius, two HVP coefficient and density-contribution arrays.

Validation could be redundant with a stronger topology certificate. Buffer
preparation could instead dominate because four pair-sized `double` arrays
alone cover 85,716,150 accumulated pairs each. Neither change is authorized
without separating their costs. Capacity/control is the negative control.

## Smallest discriminator

Add three nested steady-clock segments inside the already timed
`evaluation_setup` phase:

```text
validation = entry through completed flat source/index validation
capacity = checked byte arithmetic and payload publication
buffer = all setup vector resize/value-initialization
residual = evaluation_setup - validation - capacity - buffer
```

The parent phase timer remains unchanged. Segment timers run only in one new
command, cannot affect branches or roots, and must sum to no more than their
enclosing phase. Run three fresh serialized processes on CPUs `0..7` and
require exact SIRDIR/SIRDI semantics before looking at durations.

Route a structural audit only if the largest median setup share is at least
40%, leads the second segment by `1.20x`, and every segment-share range is at
most `0.05`. Validation selects a certificate/ownership audit; buffer selects
a write-before-read/lifetime/high-water audit. Capacity or no leader selects
no implementation and a narrower investigation.

## Decision

Freeze B4EP10SIRDIRE as timing-only evaluation-setup attribution. It changes
no validation, allocation, ownership or arithmetic and authorizes at most one
subsequent structural audit. B4E2, broad corpus, runtime, GPU, schema and
production remain blocked.
