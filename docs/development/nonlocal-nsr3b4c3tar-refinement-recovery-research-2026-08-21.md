# B4C3TAR refinement-recovery design research

Status: `COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`

Date: `2026-08-21`

## Research question

Does B4C3TA fail because canonical perturbations invalidate the Nonlocal/KKT
formulation, or because the adaptive transaction mistakes a coarse nonlinear
failure for an unrecoverable frame failure?

## Result

The exact frame-four P1 state rejects at 16 substeps and passes at 32, 64 and
128. The unchanged 32/64 and 64/128 embedded gates both pass. This distinguishes
three concepts that the r0 controller conflated:

```text
candidate solver validity   one level can fail at its chosen time step
refinement validity         a finer time step can solve the same state
commit validity             two adjacent passing levels must agree
```

The correct control flow is therefore a small state machine:

```text
PASS(l-1) + PASS(l) + gate  -> commit l
REJECT_LIMIT(l)             -> discard l, continue
PASS(l-1) + failure(l)      -> adjacency broken
other failure(l)            -> abort frame
levels exhausted            -> rollback frame
```

This is not an optimizer-tolerance change. Every individual solve retains the
same trust-region, KKT, stationarity, reaction and numerical-floor predicates.
The repair only lets the time-discretization controller ask the already-planned
finer question.

## Work semantics

Failed levels must not disappear from cost evidence. For a KKT failure before
publication, exact attempted substeps are the number of staged successful
substeps plus the failing substep. Outer-trial and HVP counters include the
failing solve. Planned substeps remain separately reported so an early failure
cannot be mislabeled as full-level work.

Only accepted fine work enters physical/publication ledgers. Attempted private
work enters controller cost and diagnostics, never conservation or energy
budgets. This maintains the distinction between computational cost and
physical state authority.

## P2 harness finding

The isolated P2 lane has zero precontact pressure violations, zero support
reaction, zero position error and `1.9246e-7 m/s` velocity error. B4C3TA prose
already required an analytical local `<1` canonical-unit allowance, but its
harness used equality to zero. The repair restores the pre-run `1e-6` bound;
the observed value does not set or tune it.

## Decision

Implement the frozen
[B4C3TAR contract](../plans/nonlocal-nonlinear-solver-research/03b4c3tar-adaptive-refinement-recovery-contract.md)
as a distinct r1 controller. Keep the r0 command and report byte-exact so the
negative evidence remains directly executable. B4C3TR and later stages remain
blocked until two identical complete r1 reports pass.
