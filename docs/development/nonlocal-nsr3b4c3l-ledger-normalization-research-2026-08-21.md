# B4C3L compensated-ledger normalization research

Status: `COMPLETE / DISCRIMINATOR FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Finding

B4C3TAR reveals two different normalized residuals for the same compensated
physical ledger vector. Let

```text
d = |delta physical momentum|
e = |gravity| + |support reaction| + |contact reaction|
L = delta momentum - gravity + support reaction + contact reaction
```

The KKT state admits:

```text
r_kkt = |L| / max(d + e, 1e-30)
```

The B4C3A1 publication ledger currently admits:

```text
r_max = |L| / max(d, e, 1e-30)
```

For positive non-floor scales:

```text
1 <= r_max / r_kkt = (d + e) / max(d, e) <= 2.
```

Thus the publication gate can reject a transition that the source KKT gate
accepted even when compensation reconstructs the source ledger exactly. That
is what occurs at P1 frame seven: `r_kkt=9.9175e-10`,
`r_max=1.1268e-9`, and compensated-vector closure is zero.

## Architectural interpretation

Three quantities have different jobs:

```text
raw published residual     exposes canonical representation impulse
compensated KKT residual   proves the physical transition still satisfies KKT
strict max residual        measures cancellation sensitivity / conditioning
```

Only the second is an admission predicate for the physical transition. The
first and third remain mandatory evidence. Treating the stricter diagnostic as
a second physical solver changes the accepted model after the solve and makes
time refinement non-monotonic for a purely semantic reason.

The correction is not to copy a boolean from the source solve. Each ledger
entry must record both normalizing scales and recompute both residuals from the
compensated vector. It must also prove closure to the source KKT vector within
the existing forward-error allowance.

## Required discriminator

The new B4C3L gate must establish:

1. exact algebraic bounds and a balanced-magnitude case approaching the factor
   two separation;
2. a threshold-separating case where `r_kkt<=1e-9` and `r_max>1e-9`;
3. exact/forward-bounded equivalence between recomputed compensated KKT
   residual and source KKT residual on one-frame P1/P2;
4. the real frame-seven legacy pattern `PASS/FAIL/PASS/FAIL` at
   `16/32/64/128`, with all four passing the KKT-scale ledger gate;
5. rejection of corrupted closure, KKT-scale overflow, nonfinite input and
   invalid scale.

Canonical frames, balanced quantization, physical coefficients, KKT tolerance,
energy accounting and adaptive selection do not change in this discriminator.

## Decision

Freeze B4C3L before changing ledger admission. PASS may authorize a new B4C3A2
selected-policy stage-ledger replay; it may not directly resume the complete
controller. This keeps the B4C3A1 and B4C3TAR evidence executable and prevents
the normalization correction from being hidden inside recovery logic.
