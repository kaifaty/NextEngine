# B4C3MC1 adaptive accuracy budget design

Status: `COMPLETE / PASS / NOMINAL CORPUS DESIGN AUTHORIZED`

Date: `2026-08-21`

## Question

B4C3MC0 measures an adaptive macro trajectory against fixed-192 and the
fixed-96/fixed-192 temporal difference. The measurement cannot itself provide
an acceptance threshold: choosing a ceiling above an observed ratio of
`100.417` would fit the gate to the two fixtures, while requiring a ratio near
one would silently change the goal from bounded physical error to matching the
fine integrator's truncation uncertainty.

## Independent accuracy budget

B4C3MC1 therefore reuses the B4B comparison envelope unchanged. For every
aligned macro frame against fixed-192:

```text
RMS position       <= 0.05 dx
RMS velocity       <= 0.001 c
center component   <= 0.05 dx
q99 height/front   <= 0.10 dx
kinetic difference <= 0.15 relative, or binary-floor overlap near zero
```

Terminal contact sets must match per frame and at completion. First-contact
time must remain within one accepted adaptive substep of the contact-owning
macro frame, plus the existing binary64 allowance. These are pre-existing
requirements, not thresholds inferred from B4C3MC0.

## Temporal-reference classification

For each position and velocity field let `e` be adaptive/fixed-192 RMS error,
`D` the fixed-96/fixed-192 RMS difference and `F` the computed numerical floor.
Classify exactly one branch:

```text
D > F                 RESOLVED_RATIO, report e/D
D <= F and e <= F     FLOOR_COINCIDENT
D <= F and e > F      STABLE_REFERENCE_SEPARATION
```

All three branches are valid measurements. None is an accuracy waiver or an
accuracy gate. In particular, `STABLE_REFERENCE_SEPARATION` exposes a real
adaptive/reference difference where the fixed ladder is level-invariant;
it must never be reported as ratio zero.

## Controls and authority

The implementation must reproduce B4C3MC0 and all adaptive/fixed roots exactly,
then exercise synthetic controls at and one representable value above every
state/aggregate/kinetic/contact boundary. Separate controls must cover the
three temporal branches, non-finite rejection and exact contact-set identity.

A repeatable PASS selects adaptive accuracy only for the tiny P1/P2 research
corpus and may authorize design of a nominal diverse corpus. It does not prove
temporal equivalence, performance, runtime/schema suitability or production.

## Decision

Freeze the
[B4C3MC1 contract](../plans/nonlocal-nonlinear-solver-research/03b4c3mc1-adaptive-accuracy-budget-contract.md)
with no fitted constants and implement it as a distinct parent-gated stage.

Execution passed twice byte-identically. See the
[dated evidence](nonlocal-nsr3b4c3mc1-adaptive-accuracy-budget-evidence-2026-08-21.md).
The unchanged physical gate selects the adaptive controller only for P1/P2;
temporal equivalence remains explicitly unselected. Nominal-corpus design is
authorized next.
