# NSR3-B4E2D7R19R11 total-budget offline replay research

Date: `2026-08-23`

Status: `COMPLETE / OFFLINE RECURRENCE REPLAY SELECTED`

## Question

R10 proves that the total-HVP limit interrupts a finite,
positive-curvature, interior and monotonically improving recurrence after
HVP 14. Its final residual is nevertheless `37.9603 eta`, so neither a
one-shot grace nor a numerical estimate from the observed contraction is a
sufficient basis for changing the live global budget.

The next falsifiable question is:

> Under the exact same workspace, gradient, radius and dimensionless forcing,
> does this recurrence reach forcing convergence before the existing offline
> diagnostic cap of 128 HVPs, and after how much exact work?

## Candidate approaches

### A. Increase the live total budget and observe the transaction

Rejected. This combines three unresolved decisions: recurrence completion,
model/trial formation and a global budget policy. A failure after HVP 512
would also spend new live work before the interrupted solve is understood.

### B. Extrapolate from the last eight contraction ratios

Rejected. Conjugate-gradient residuals need not follow a fixed geometric
rate. The extrapolation is useful intuition but cannot own a structural cap.

### C. Resume serialized Krylov vectors at iteration 14

Rejected for R11. The capture stores enough diagnostic recurrence state but
does not expose the complete mutable CG continuation state as an independent
checkpoint contract. Adding checkpoint/resume would create a second
implementation before the ordinary deterministic replay is tested.

### D. Rebuild and replay the exact solve from iteration zero

Selected. Rebuild one binary64-owned sparse workspace from the captured
current/predicted/dual/theta state, reproduce all first-14 iteration
projections exactly, and allow the same Steihaug recurrence to continue under
the already established offline cap of 128 HVPs.

This is diagnostic work. It does not form `H(step)`, a trial position,
divided reduction, precision audit or transaction state.

## Required correspondence

The R10 parent report and all transitive parents remain exact. The offline
replay binds:

```text
boundary root    002f3b63423de2e0db8715b599e064043502c3881ba89cb55aaf7394bee850df
recurrence root  a900c9452fa173f1c60fdf7c670de747653742348340056f05d7336cbaaa974c
outer/trial      5 / 1
solve index      20
live prefix      14 HVPs
```

It must reproduce the exact captured gradient and first-14 prefix root before
later iterations count. Radius, forcing eta, initial residual, static-support
identity and every input root must remain exact.

## Terminal classifier

Use the established offline precedence:

1. nonfinite HVP or recurrence arithmetic;
2. nonpositive curvature;
3. trust-boundary return;
4. forcing convergence;
5. exact 128-HVP diagnostic-cap exhaustion.

Expose the full recurrence root, final/minimum residual ratio, maximum
residual orthogonality, maximum adjacent A-conjugacy and CG-derived Ritz
spectrum/condition estimate. These diagnose the solve; they do not authorize
a preconditioner or a live cap by themselves.

## Work and authority

R11 owns one offline sparse workspace and at most 128 diagnostic recurrence
HVPs. It owns zero model HVP, trial, acceptance, precision audit, outer update
or candidate substep. It may not continue the R9 transaction, run another
substep, macro, trajectory or timing lane.

## Decision

Freeze R11 around option D. If the recurrence safely converges, use the exact
completion count only as input to a later global-budget-policy discriminator.
If it does not, preserve the terminal mechanism and do not increase the live
budget.
