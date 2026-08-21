# B4C3PE macro publication stability diagnostic

Status: `COMPLETE / MEASUREMENT_CONTRACT_FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Purpose

B4C3P restored fixed-level convergence but missed one inherited velocity tube.
Changing the coefficient from that observation would fit the answer. B4C3PE
therefore introduces no acceptance threshold. It measures how a balanced macro
publication perturbation enters and propagates through the physical macro map.

## Error decomposition

For fixed level `S` and macro frame `k`, record:

```text
B_k       independent binary64 fixed-lane boundary state
C_k^-     private interval result from prior durable C_(k-1)
C_k       decoded balanced macro publication of C_k^-

start error       ||C_(k-1) - B_(k-1)||_RMS
propagated error  ||C_k^-     - B_k||_RMS
direct error      ||C_k       - C_k^-||_RMS
published error   ||C_k       - B_k||_RMS
```

Position and velocity are kept separate. The triangle closure
`published <= propagated + direct + fp_bound` is mandatory. A propagation gain
is reported only when the start error is resolved above its computed binary64
floor; unresolved frames are counted rather than divided by noise.

## Fine-reference contamination

At every macro boundary compare the fixed-192 published error with the
independent binary `96->192` temporal difference:

```text
r = ||C_192 - B_192|| / ||B_96 - B_192||.
```

Report `r` only when the binary temporal difference is above its computed
binary64 floor. Otherwise mark the field unresolved and report absolute error
plus utilization of the already existing physical comparison scale
`0.05dx / 0.001c`. No threshold is selected in this stage.

Also report direct-error share, maximum resolved macro-map gain, worst frames,
event/contact identity and the complete candidate roots. The existing B4C3P
FAIL hash is the exact parent.

## Decision

The measurements may authorize design of one of three separately frozen
budgets:

1. local macro-map gain propagation bound;
2. resolved temporal-contamination budget plus unresolved absolute floor;
3. rejection of microunit durable continuation if neither separates harmless
   representation error from changed physical events.

B4C3P is not reclassified by this diagnostic. No adaptive, runtime or
production authority is granted.
