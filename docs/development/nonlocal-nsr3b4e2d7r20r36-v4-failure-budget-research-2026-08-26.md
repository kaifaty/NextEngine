# NSR3-B4E2D7R20R36 v4 failure-budget research

Status: `RESEARCH COMPLETE / TWO-CASE REPORT-ONLY AUDIT SELECTED`.

## Question

Which certified error budget first blocks torsion and counterflow in R35, and
are the failures instances of one mechanism or two independent boundaries?

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| F1 | torsion inverse columns/factor fail numerically | rejected audit has failed solves, nonfinite values or large column residual |
| F2 | torsion inverse-norm enclosure alone fails | all columns solve but the residual norm `rho` is not below one |
| F3 | counterflow's current residual bounds dominate slope | residual-bound contribution is the largest explicit term |
| F4 | counterflow's accumulated direction enclosure dominates | `sum(abs(residual)*direction_error)` is the largest term and exceeds nominal slope |
| F5 | rounding or a nonpositive nominal model causes rejection | rounding dominates or nominal slope is nonpositive/nonfinite |

## Selected discriminator

Replay only exact R35 torsion/counterflow problem and case roots. For torsion,
report every existing verified-inverse audit: dimension, column solves,
cheap/refined errors, maximum column residual/bound, inverse residual `rho`,
inverse norm and roots. Add no inverse solve.

For counterflow, reconstruct the exact accepted-prefix current dual and final
line direction. Recompose the step slope bound into current-residual-bound,
direction-error and arithmetic-rounding terms in the same binary128 order;
report parity with the stored bound, NNQP-local slope/bound and the final
global slope/bound. Add no trial, factorization or state.

## Ceiling

R36 may select the next bounded mathematical audit. It cannot refine an
inverse, replace a norm, change direction/Armijo/tolerance/cap, rerun v4 as a
candidate, time the solver or claim runtime/production readiness.
