# NSR3-B4E2D7R20R16 current-provenance contract

Status: `FROZEN / REPORT-ONLY DUAL INVERSE AUDIT AUTHORIZED`.

## Parent

- R15 implementation `ed832851`, semantic
  `d9c42d3b26bf6240dc7f6db15759f211ce0f1df446e12b8c666a45cdcdda1240`;
- exact R13 case/step roots and R14 candidate-inverse roots;
- affine shadow equals old current error at both collisions.

## Frozen observer

Retain, but never consume, the last full-passive solution provenance:

- factor and ordered passive source rows;
- solution, residual, solve bound and cheap error;
- support root and transition ordinal.

Set provenance direct on `negative.empty()` and invalidate it before every
boundary interpolation. At the preserved ratio rejection, require current
direction/support correspondence. If direct, invoke the existing verified
inverse on the retained current factor and recompute ratio ordering with both
current and candidate refined errors.

Require old case/step/candidate-inverse roots unchanged. Report current audit
dimension, columns, `rho`, inverse norm, cheap/refined error, provenance
support, and the dual-refined ratio result. Classify
`DUAL_REFINEMENT_RESOLVES_ALL`, `RESOLVES_SUBSET`, `RESOLVES_NONE`,
`CURRENT_PROVENANCE_ABSENT` or `CURRENT_AUDIT_REJECTED`.

No solver decision/state update, successful-case solve, timing, runtime/GPU or
production authority. A positive result authorizes only a separately frozen
candidate-solver replay.

