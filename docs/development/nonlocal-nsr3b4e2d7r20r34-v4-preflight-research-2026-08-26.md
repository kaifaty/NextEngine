# NSR3-B4E2D7R20R34 v4 input preflight research

Status: `RESEARCH COMPLETE / INPUT-ONLY OPERATOR PREFLIGHT SELECTED`.

## Question

Are all five immutable R33 sources valid, distinct and nontrivially excited
inputs for a future solver generalization test?

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| P1 | every source provides a valid excited problem | exact operator ownership and at least one rigorously positive projected row per case |
| P2 | one source is quiet after joint projection | its certified-positive row count is zero |
| P3 | source geometry does not survive operator construction | structural, lifecycle, scale or source-positive gate fails |
| P4 | different source constructions collapse to one problem | duplicate complete problem roots occur |

## Selected discriminator

Materialize each committed source once. Audit sparse slot/entry/incidence
ownership, source feasibility, joint box-ball projected target, rigorous row
bounds and the same low/high scaling controls used by R10. Record all complete
problem roots and require pairwise distinction.

Excitation is `projected_raw[row] > row_bound[row]` for at least one row. This
is an admission predicate only; no solver, inverse, KKT trajectory, tuning or
timing is allowed. Stop after preflight regardless of outcome.

## Ceiling

A pass authorizes exactly one unchanged-policy v4 solver execution under a new
contract. It does not predict convergence or provide generalization,
performance, runtime/GPU or production evidence.
