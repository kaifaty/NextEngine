# NSR3-B4E2D7R19R43 contact-tangent normal-step research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R42

Stable superset ownership and trial relinearization are now valid. The next
independent R40 rejection is contact: R41 found zero new crossings, but 1,268
already-active box faces own inward normal motion. This permits a sharper
repair than collision clipping after the step.

## Selected projection

For every source particle and axis, derive exact active half-spaces:

```text
source x < low  => admissible step dx >= 0
source x > high => admissible step dx <= 0
otherwise       => dx unconstrained by that axis
```

Project the frozen R39 dimensionless normal step componentwise onto the
intersection of these half-spaces. An inadmissible component becomes exact
`+0.0`; every other component retains its original binary64 bits. Then map the
projected dimensionless vector through the unchanged `SPACING` transaction.

This is the Euclidean projection onto the source-active orthant/tangent cone,
not a final-position clamp. It cannot increase any per-particle or global step
norm. Consequently the R42 source-anchored superset coverage certificate also
covers the projected trial, but R43 must recompute actual displacement and
ordered inclusion.

## Required reevaluation

Projection changes the density direction, so none of R40's predicted/actual
reductions or merit signs is inherited. R43 must freshly compute:

- source-linear response and predicted hinge reduction for the projected
  dimensionless step;
- nonlinear projected-trial constraint, actual reduction and `rho` under the
  inherited `rho>=0.1` rule;
- exact all-face contact ledger, requiring no new penetration and no increase
  over every source face;
- stable-superset masks and a fresh projected-trial operator/mapping;
- complete normalized inertia plus PHR merit, two precancelled evaluations,
  unconditional long-double audit and conditional binary128 only if the sign
  remains unresolved or disagrees.

No stationarity/contact tolerance is selected. Source penetration is active
iff it is strictly positive in binary64.

## Scientific routes

Frozen precedence distinguishes projection/contact, coverage/operator,
feasibility, precision and full merit. Passing contact and feasibility with a
negative merit selects `TANGENTIAL_MERIT_STEP_REQUIRED`; a positive resolved
merit selects `CONTACT_TANGENT_NORMAL_STEP_CANDIDATE`. The latter remains a
rollback-only candidate, not authority to commit or resume an outer solve.

## Rejected alternatives

- clamping final positions or only the worst face;
- adding a contact penalty after observing the result;
- treating source penetration as inactive via epsilon;
- preserving R40 predicted reduction or merit after projection;
- changing penalty, trust radius, `H`, skin or the Hager--Zhang recurrence;
- accepting on contact/feasibility alone;
- runtime integration or shared-host timing.

Frozen contract:
[R43 contact-tangent normal step](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r43-contact-tangent-normal-contract.md).
