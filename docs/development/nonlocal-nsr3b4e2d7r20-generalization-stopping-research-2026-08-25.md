# NSR3-B4E2D7R20 generalization and stopping research

Date: `2026-08-25`

Status: `RESEARCH OPEN / CONTRACT NOT YET FROZEN`.

## Why R65 is not production yet

R65 v11 wins a fair structural-work comparison, but it was derived and tested
on one restored dam state. Production needs evidence that the composed-dual
policy generalizes across topology/contact/active-set regimes and can stop for
a mathematical reason rather than after a fixture-specific 20 outers.

## Research questions

1. Which existing immutable face, corner, supported-column and released-block
   states can be lifted into the same R64 all-row convex TRQP without changing
   their historical physics or boundary ownership?
2. What high-accuracy offline projection reference can independently audit
   primal feasibility, dual value and KKT residual on small and 6,000-row
   cases?
3. Which dimensionless stopping tuple is stable across particle count,
   spacing, timestep and residual scale?
4. Can an adaptive stop reproduce fixed-20 quality without learning a
   threshold from v11?

## Candidate evidence design

The likely corpus has four predeclared regimes:

```text
face contact
corner contact
supported/hydrostatic column
released/dam block
```

At least one regime must be a true holdout whose state did not participate in
R43--R65 decisions. Tiny cases should admit a dense or very-high-accuracy QP
oracle; product-size cases may use an offline monotone Hildreth/Dykstra
reference with a separately certified residual envelope. Fixture selection,
resolution and reference work must be frozen before solver execution.

The stopping study should report a tuple, not one raw epsilon:

```text
scaled primal positive violation
scaled projected-gradient norm
relative composed-dual improvement with numerical resolution bound
stationarity / complementarity certificate
remaining structural budget
```

Scaling must come from the nondimensional R63/R64 formulation and analytic
forward-error bounds. No component may be normalized by the winning v11 final
value.

## Stop conditions for this research

- Do not run outer 21 or another depth grid.
- Do not tune a KKT tolerance on the dam fixture.
- Do not call FISTA an accuracy oracle; it is only the frozen work competitor.
- Do not start GPU/performance timing until corpus correspondence and stopping
  policy are frozen and pass.
- Preserve outer nonlinear filter-SQP as a later globalization owner; R65's
  exact dual applies only inside each fixed convex TRQP.

The next deliverable is a corpus/stopping contract with explicit holdout
provenance, oracle hierarchy, nondimensional tolerances, corruption controls
and finite work budgets. Execution is not authorized by this research note.
