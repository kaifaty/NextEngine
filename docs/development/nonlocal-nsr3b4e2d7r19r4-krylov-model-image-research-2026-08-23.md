# NSR3-B4E2D7R19R4 Krylov model-image research

Date: `2026-08-23`

Status: `COMPLETE / THREE-LANE DISCRIMINATOR SELECTED / CONTRACT FROZEN NEXT`

## Question

D7R19R3 proves that the failed sixth Steihaug recurrence satisfies forcing on
HVP 33. The current inner solver then performs a separate HVP on the completed
step to evaluate

```text
predicted_reduction = -g·p - 0.5 p·H(p)
```

Can the already computed Krylov data provide an accurate `H(p)` image, so the
trial model does not need a 34th HVP?

This is a model-image discriminator only. It does not yet grant recurrence
grace, form a trial or alter the R2 watchdog.

## Linear identities

The sparse normalized Hessian is frozen by the current workspace. Its HVP is
linear in the direction: identity plus fixed pairwise Jacobian/curvature
terms, with no topology or active-set change during a trust solve.

Steihaug CG maintains

```text
p_(k+1) = p_k + alpha_k d_k
r_(k+1) = r_k + alpha_k H(d_k)
r_0     = g
```

Therefore, in exact arithmetic, two zero-HVP constructions exist:

```text
accumulated image  z_(k+1) = z_k + alpha_k H(d_k), z_0 = 0
residual image     H(p_k)   = r_k - g
```

Both equal a direct `H(p_k)`. In binary64 they need not be bit-exact because
the direct HVP folds pair terms in another order and the recursive residual
can accumulate a residual gap.

## Candidate lanes

### Direct oracle

Run the exact 33-HVP recurrence once, then execute one direct sparse HVP on
the returned step. This is the reference model and costs 34 HVPs in total. It
does not evaluate candidate energy or form a trial.

### Accumulated Krylov image

Along the same recurrence, accumulate `alpha_k H(d_k)` into a dedicated
vector. This adds vector work each iteration but no HVP. It is also valid for
a later trust-boundary truncation by using the terminal boundary coefficient.

### Residual-derived image

At forcing convergence, subtract the original gradient from the final
recursive residual. This costs one final vector subtraction and no per-
iteration image accumulation. It is the preferred lane if its residual gap is
inside the same oracle bounds.

## Frozen comparisons

For each no-HVP lane, compare against the direct oracle:

- relative L2 image error;
- maximum component error scaled by the direct HVP absolute-term sum and
  observed component magnitudes;
- relative error in `p·H(p)`;
- relative error and sign of predicted reduction;
- exact roots for step, final residual and all three images.

All relative/scaled errors must be at most `1e-10`, and all model reductions
must be finite and positive. This bound is intentionally much smaller than
the trust acceptance thresholds while remaining loose enough to test a
binary64 recursive image rather than demand bit identity.

The residual-derived lane has priority when both pass because it adds only one
O(N) vector operation after convergence. The accumulated lane is selected only
when residual-derived fails but accumulation passes. Otherwise the direct
model HVP remains required.

## Rejected shortcuts

### Raise the combined per-step cap to 34

Rejected as a conclusion before this discriminator. It would make the exact
sixth trial possible but would preserve a potentially redundant HVP on every
completed trust solve.

### Assume `r-g` is exact

Rejected. Recursive CG residuals can drift from the true residual even when
adjacent orthogonality looks good. One direct oracle HVP is necessary to
measure that gap.

### Form the sixth trial now

Rejected. Model-image correspondence must be established before candidate
energy, precision audit, acceptance or radius policy can be interpreted.

### Use timing to choose a lane

Rejected on the shared host. Operation/work ownership and exact numerical
correspondence are sufficient for this stage; timing remains stopped.

## Smallest falsifiable experiment

Freeze D7R19R4 to:

1. reproduce exact D7R19R3 and D7R19R2 parent bytes;
2. reuse R3's exact target, first-32 prefix and 33-HVP forcing convergence;
3. expose the converged step, final recursive residual and accumulated image
   from the passive offline recurrence without changing R3 output bytes;
4. rebuild one exact sparse workspace and execute one direct oracle `H(step)`;
5. compare both zero-HVP images and their model reductions under the frozen
   bounds;
6. select residual-derived, accumulated or direct-required under frozen
   precedence;
7. retain one new HVP, one workspace build/release, zero precision audits,
   zero trials/acceptances, zero all-pair calls and exact rollback;
8. run no candidate nominal substep, macro, trajectory or timing lane.

## Follow-up boundary

A zero-HVP candidate route authorizes research/freeze of a sixth-trial-only
reclosure using one bounded recurrence grace HVP and the selected model image.
That later stage must compare direct/reused model bits, divided reduction,
precision audit, acceptance and radius update before any full transaction.

`DIRECT_MODEL_HVP_REQUIRED` instead authorizes a sixth-trial-only reclosure
with two bounded grace HVPs (33rd recurrence plus direct model HVP). It does
not justify an unqualified global cap increase.

## Authority boundary

D7R19R4 can select a model-image candidate for one exact replay state. It
cannot form or accept a trial, change a watchdog, run another nominal substep,
claim performance or create runtime/production authority.
