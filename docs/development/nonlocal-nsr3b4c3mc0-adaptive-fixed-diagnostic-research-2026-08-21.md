# B4C3MC0 adaptive-versus-fixed diagnostic research

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Why measurement comes first

B4C3MAR establishes complete adaptive composition and B4C3PE1 establishes a
convergent fixed macro reference. Neither establishes an a priori relation
between adaptive global error and the fixed `96/192` temporal difference.
Choosing a multiplier before seeing whether the denominator is resolved would
repeat the scalar-gain mistake rejected by B4C3PE.

B4C3MC0 is therefore threshold-free. It cannot select or reject adaptive
accuracy. It measures the complete aligned trajectories and may authorize only
a separately frozen comparison budget.

## Measurements

For every macro frame and independently for position/velocity, report adaptive
RMS distance to fixed `48`, `96` and `192`, plus:

```text
e = RMS(adaptive, fixed192)
D = RMS(fixed96, fixed192)
resolved ratio = e/D only when D > binary64 floor
physical use   = e/(0.05dx or 0.001c)
```

At fixed-192 also report center, q99 height/front, kinetic difference, terminal
contact equality and onset-time difference. Preserve adaptive and every fixed
trajectory/ledger root separately.

## Decision

Freeze the [B4C3MC0 contract](../plans/nonlocal-nonlinear-solver-research/03b4c3mc0-adaptive-fixed-diagnostic-contract.md).
PASS means only that the measurement is aligned, finite and reproducible. It
may authorize B4C3MC1 accuracy-budget design; no accuracy or production claim
follows from the diagnostic itself.
