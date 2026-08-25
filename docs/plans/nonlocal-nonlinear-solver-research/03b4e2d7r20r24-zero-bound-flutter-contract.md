# NSR3-B4E2D7R20R24 zero-bound mask-flutter contract

Status: `FROZEN / REPORT-ONLY EXISTING-LADDER AUDIT AUTHORIZED`.

## Parent

- R23 implementation `423d52ad`, semantic
  `48d97d2110b3d3d5c6ebeff24a46c5183bb58063005a7af168b30d383aa11987`;
- seven correct Armijo-positive first crossings;
- one later mask event, isolated to step 32;
- R20's wider bracket contains only the same scalar-189 transition.

## Frozen audit

Expose, without hashing into or changing R23, the exact projector input already
computed by each of its 65 step-32 dual evaluations. Compare scalar 189 with
the independent affine zero-bound polynomial. For every power classify the
projector as current-old, exactly one-scalar predicted-new, or other; retain
ball state.

Report run-length encoding, old/new/other counts, mask and actual-input sign
transition counts, analytic monotonicity/nonnegativity, maximum absolute
actual-minus-affine difference and the first power of the final all-new suffix.
Require exact R23/R22/R21/R20/shear reproduction and zero new ladder points.

Use the classification priority frozen in the research document. No mask
override, fitted suffix/run length, forward-bound policy, solver trial/state
update, cap/tolerance change, timing, runtime/GPU or production authority.

