# NSR3-B4E2D7R19R40 nonlinear normal-step acceptance research

Date: `2026-08-24`

Status: `RESEARCH COMPLETE / CONTRACT FROZEN / IMPLEMENTATION NEXT`.

## Question left by R39

R39 solves the frozen linearized all-inequality normal model extremely well,
but it has never moved the nonlinear particle configuration. Its terminal
objective `9.13328870609491e-27` is therefore a prediction, not evidence that
the real density field or the complete normalized augmented-Lagrangian merit
improves.

The next smallest falsifiable question is:

> If the exact R39 Hager--Zhang endpoint is mapped back to physical particle
> positions, does one freshly rebuilt nonlinear trial agree sufficiently with
> the linear feasibility prediction and also reduce the full normalized
> inertia-plus-PHR merit?

This is a rollback-only acceptance discriminator. It is not another inner
iteration, a state update or a production solver.

## Research basis

Trust-region SQP methods separate restoration of feasibility from objective
optimization at an infeasible iterate. Wright and Tenny describe a bounded
normal step that seeks constraint reduction before the remaining step, in
[A Feasible Trust-Region Sequential Quadratic Programming
Algorithm](https://pages.cs.wisc.edu/~swright/papers/WriT04a.pdf).
Heinkenschloss and Ridzal similarly separate quasi-normal and tangential
components in their
[matrix-free trust-region SQP
method](https://repository.rice.edu/bitstreams/13d38ec8-775a-4c51-94c-52b608fc0d07/download),
and globalize the complete step with an augmented-Lagrangian merit function.

Those algorithms do not prove our squared-hinge particle step correct. They
do establish two obligations that R40 must keep separate:

1. the normal model must predict the nonlinear constraint reduction; and
2. feasibility improvement alone does not authorize a step that increases
   the complete merit.

The inherited trust policy already uses `0.1` as the minimum meaningful
actual/predicted ratio. R40 reuses that pre-existing constant; it does not fit
a new threshold to this candidate.

## Exact physical mapping

R29 defines its dimensionless operator as `A = SPACING * Jc`. Therefore the
only dimensionally consistent mapping of the exact R39 terminal iterate `v`
is

```text
y_trial = y_current + SPACING * v,
SPACING bits = 0x3fa999999999999a.
```

R39 already constrains the global dimensionless L2 norm to `0.25`, so the
physical displacement must be finite and at most `0.25*SPACING = 0.0125`.
No component is clipped, reprojected or collision-corrected in R40; changing
the vector would create a different experiment.

## Nonlinear feasibility agreement

For current and trial workspaces rebuilt from their own particle positions,
define

```text
psi(y) = 1/2 * ||max(0, c(y))||^2.
```

The R39 terminal linear objective is `psi_linear`. R40 computes

```text
predicted_reduction = psi(current) - psi_linear
actual_reduction    = psi(current) - psi(trial)
rho_feasibility     = actual_reduction / predicted_reduction.
```

A nonlinear feasibility pass requires finite positive predicted and actual
reductions and `rho_feasibility >= 0.1`. The trial need not achieve the nearly
zero linear prediction; the ratio measures the local model's validity without
inventing an absolute residual floor.

## Complete normalized merit

The same current/trial workspaces are evaluated using the already closed
normalized objective

```text
M(y) = 1/2 * ||y - y_predicted||^2
     + theta/2 * sum(max(0, u + c(y))^2 - u^2).
```

Actual reduction is evaluated with the certified pairwise-precancelled
divided-difference path and repeated exactly. A direct normalized long-double
audit with binary64-owned membership always resolves or challenges its sign.
Binary128 is allowed only if long double is unresolved or disagrees with the
binary64 candidate; it remains diagnostic and cannot become runtime policy.

The full merit must have a precision-resolved positive reduction. If
nonlinear feasibility passes but the full merit does not, R40 selects
`COMPOSITE_NORMAL_TANGENTIAL_STEP_REQUIRED`. That is a useful result: the
normal direction is doing its job, but a tangential/objective component or a
different merit globalization is required. Penalty or threshold tuning is
explicitly forbidden.

## Topology, contact and ownership

The current workspace must reproduce the exact R30 topology root. R40 also
hashes only the ordered `(fluid, participant)` pair identities and requires
current and trial pair-membership roots to match. This avoids confusing a
neighborhood membership discontinuity with smooth model error. Full trial
topology is still reported.

For the frozen dam fixture, maximum box penetration is evaluated directly at
current and trial positions. The trial may not exceed
`max(source_penetration, 1e-12)`. R40 does not run the contact solver.

Position, predicted position, dual, constraint and the transitive R39
endpoint are root-bound before and after the diagnostic. Neither candidate
nor workspace state can escape the report.

## Dense controls and outcome routes

Before the nominal evaluation, analytic scalar controls prove:

1. a nonlinear constraint case with positive predicted/actual reduction and
   `rho>=0.1` is admitted;
2. the same feasibility improvement is rejected when inertia makes the full
   merit increase; and
3. every hard-failure and scientific-classification route has the frozen
   precedence.

The scientific outcomes are:

- topology or contact rejection;
- nonlinear feasibility-model rejection;
- unresolved merit precision;
- `COMPOSITE_NORMAL_TANGENTIAL_STEP_REQUIRED`; or
- `NONLINEAR_NORMAL_STEP_ACCEPTANCE_CANDIDATE`.

All are rollback-only classifications. A hard parent/source/control/work
failure is not a scientific result.

## Work and continuation

R40 replays R39 once, builds exactly current and trial nonlinear workspaces,
forms one private trial, repeats the divided reduction twice and runs one
long-double audit plus at most one conditional binary128 audit. It adds zero
JVP, VJP, HVP, model solve, outer update, substep, macro or trajectory work.
This is a deterministic correctness experiment, not a timing run.

- nonlinear normal-step candidate: separately research state-safe composite
  step/globalization before any commit;
- feasibility pass but merit reject: research a tangential component and
  merit model, preserving the normal step as a feasibility primitive;
- feasibility model reject: contract/relinearize the normal step without
  changing R39 after the result;
- topology/contact reject: research that first exact boundary rather than
  weakening it.

Frozen contract:
[R40 nonlinear acceptance](../plans/nonlocal-nonlinear-solver-research/03b4e2d7r19r40-nonlinear-normal-acceptance-contract.md).
