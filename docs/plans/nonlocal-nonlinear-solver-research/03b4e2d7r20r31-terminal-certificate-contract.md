# NSR3-B4E2D7R20R31 terminal certificate precedence contract

Status: `FROZEN / REPORT-ONLY CERTIFICATE-MERIT AUDIT AUTHORIZED`.

## Parent

- R30 implementation `16527590`, semantic
  `e1f807e9ac25ad9317f07bc9e4a4364af76fa653e664ba8da10eaa2e10581ff5`;
- exact R29 shear root `3f4798c8...e473` and iteration-22 step root;
- 21 same-face, same-ball, strictly Armijo-negative inherited trials;
- exact KKT tolerance `2^-70` remains unchanged.

## Frozen audit

Replay only exact R30 shear. For all 21 existing trials report the full KKT
certificate predicate and every constituent finite/residual/gap condition.
Identify all certified trials and the first in inherited power order.

Recompute the exact R19 Armijo decomposition: nominal dual change, new/old
dual bounds, lower slope term, RHS round bound, dual-increase lower and final
margin. Hash all values and metric roots.

Classify, in priority, parent/reproduction failure, any exact terminally
certified but Armijo-rejected trial as
`TERMINAL_CERTIFICATE_PRECEDES_ARMIJO`, no certified trial with a named first
KKT failure as `NO_TERMINAL_CERTIFIED_TRIAL`, a finite/gap contradiction as
`TERMINAL_CERTIFICATE_CONTRADICTION`, otherwise unresolved. Separately report
whether merit comparison is bound dominated; do not turn that secondary fact
into a tolerance change.

No trial/state application, line/event/recovery, tolerance/cap/Armijo change,
trajectory continuation, timing, runtime/GPU, generalization or production
authority.
