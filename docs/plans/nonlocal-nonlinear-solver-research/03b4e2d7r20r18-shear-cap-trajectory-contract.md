# NSR3-B4E2D7R20R18 shear cap-trajectory contract

Status: `FROZEN / REPORT-ONLY REPLAY AUTHORIZED`.

## Parent

- R17 implementation `9546e852`, semantic
  `d224bc8c0f451e1a07aa4cbdf6cc194c85f6341ca3be5a1d74e2a7d97cfe6ab8`;
- shear problem root `08c636f4...8b62` and candidate case root
  `fdd57c9e...fbb9`;
- 32 accepted steps, one dual refinement, no rejection, terminal
  primal/dual mapping `1.745069791762...e-7`.

## Frozen audit

Materialize all v3 roots for parent identity, solve only shear ordinal 2 with
`verified_inverse=true`, `dual_ratio_refinement=true`, and unchanged cap 32.
Require exact R17 case root and report per accepted step:

- natural face, NNQP support, principal solves and transitions;
- line-trial count, selected power, alpha and dual-increase lower bound;
- projector mask and ball-activity changes;
- primal, dual mapping, complementarity, stationarity and scaled gap;
- consecutive primal and dual-mapping central ratios;
- dual-refinement and inverse-column counts.

Classify the late window (steps 25–32) from observed contraction, face/mask and
line-power facts using these pre-observation rules:

- `useful_contraction`: every finite consecutive primal and dual-mapping ratio
  is at most `0.9`;
- `plateau`: every such ratio is at least `0.99`;
- `active_set_chatter`: any selected projector mask change or natural-face-root
  change occurs in the window;
- `globalization_throttled`: at least six of eight selected powers are positive.

Choose route priority chatter, globalization, plateau, useful contraction,
then mixed/unresolved. These are diagnostic labels, not convergence theorems.
No cap extension, retry-driven tuning, timing, solver change, runtime/GPU or
production authority.
