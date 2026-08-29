# NSR3-B4E2D7R19R5 guarded sixth-trial research

Date: `2026-08-23`

Status: `COMPLETE / SIXTH-TRIAL SHADOW SELECTED / CONTRACT FROZEN NEXT`

## Question

D7R19R3 proves that the sixth recurrence needs exactly one HVP beyond the
historical 32-HVP stop. D7R19R4 proves that `r_final-g` reproduces the direct
model scalar exactly and eliminates the separate candidate `H(step)` HVP.

Does one narrowly guarded recurrence-grace HVP produce an admissible sixth
trial under the existing divided-reduction, precision, acceptance and radius
policies?

The experiment must stop after classifying that trial. It cannot commit the
accepted position, continue the inner solve or change the production budget.

## Guard design

An unconditional cap of 33 is rejected because R3 certifies only one exact
near-converged state. The grace is admitted only at the 32-HVP boundary when
all of the following are already known from the live-equivalent recurrence:

```text
all first 32 HVP/images/scalars finite
all first 32 curvatures positive
all first 32 candidates strictly inside the trust radius
residual_ratio_after_32 > eta
residual_ratio_after_32 <= 1.25 * eta
last eight next-residual ratios strictly decrease
```

If eligible, exactly one additional recurrence HVP is allowed. The absolute
recurrence cap is 33, and HVP 33 must satisfy forcing. There is no second grace
and no candidate model HVP.

For the frozen target, R3 pre-derives:

```text
ratio after 32  3.02723280342496e-05
eta             2.51880524249163e-05
ratio / eta     1.20185266901795
```

The guard therefore admits the known target without reducing the forcing
tolerance.

## Model and trial

Use the R4-selected residual-derived image:

```text
model_image = r_final - g
predicted   = -g·step - 0.5 step·model_image
trial       = current + step
```

Retain one direct `H(step)` only as an oracle and require the residual-derived
predicted reduction to equal its frozen binary64 bits
`0x3bc27dd9b2871ea7`.

Build exact current and trial sparse workspaces and compute the existing
pairwise pre-cancelled divided reduction twice. Candidate acceptance remains:

```text
trial finite
predicted reduction > 0
divided reduction > 0
divided / predicted >= 0.1
```

If the trial would be accepted, run the existing binary64-owned long-double
audit and require finite resolved-positive sign. Membership mismatch remains
diagnostic under the already closed R2 policy.

Apply the existing trust-radius decision only to a shadow record. Do not
replace current position/workspace, update outer dual state or append the trial
to a transaction.

## Why trial-level evidence comes before a full transaction

The new step changes three coupled facts at once: structural work ownership,
the model image and the previously absent sixth trial. A full transaction
would immediately continue from any accepted trial and obscure which change
caused later behavior.

The sixth-trial shadow isolates:

- grace eligibility and exact work;
- direct/residual model correspondence;
- candidate position and workspace validity;
- divided reduction and repeatability;
- precision sign;
- acceptance and radius result.

Only after all six are bound should the policy enter a full private
transaction.

## Rejected alternatives

### Unconditional combined cap 34

Rejected. It pays the direct model HVP and grants two extra HVPs to every trust
solve without a convergence-state predicate.

### Unconditional recurrence cap 33

Rejected as production policy. The shadow uses an explicit one-shot guard and
does not generalize from one target.

### Reduce forcing eta

Rejected. R3 shows the existing forcing condition is satisfied naturally on
the next iteration; changing tolerance would alter solver accuracy.

### Commit and continue the accepted trial

Rejected for this stage. That is a separate full-transaction candidate with
its own work and convergence contract.

## Smallest falsifiable experiment

Freeze D7R19R5 to:

1. reproduce exact D7R19R4/R3/R2 bytes;
2. bind exact target/step roots and the live-equivalent first-32 prefix;
3. evaluate the frozen grace predicate and permit at most HVP 33;
4. require forcing convergence on HVP 33 and no candidate model HVP;
5. reconstruct `H(step)` from residual, retain one direct oracle and require
   exact predicted-reduction bits;
6. build current/trial workspaces, compute pre-cancelled divided reduction
   twice and bind the trial root;
7. if accepted, run one binary64-owned long-double precision audit;
8. classify precision contradiction, insufficient grace, accepted candidate
   or rejected candidate under frozen precedence;
9. calculate but do not apply the existing radius result;
10. retain exact work/lifecycle/static binding/rollback and run no candidate
    nominal substep, continuation, macro, trajectory or timing lane.

## Authority boundary

D7R19R5 can classify one sixth shadow trial. It cannot mutate a transaction,
commit state, select a production cap, run another nominal substep, claim
performance or create runtime/production authority.
