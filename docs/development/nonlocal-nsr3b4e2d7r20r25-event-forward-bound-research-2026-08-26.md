# NSR3-B4E2D7R20R25 event-input forward-bound research

Status: `RESEARCH COMPLETE / COMPONENTWISE GAMMA BOUND SELECTED`.

## Question

Can an operation-count-derived componentwise forward bound dominate the actual
versus affine projector-input discrepancy for every frozen ladder point?

## Selected bound

For the predicted particle component, enumerate the exact transpose entries
that contribute to it. Bound the convex multiplier construction, coefficient
products, sequential transpose additions, target subtraction and the
independent endpoint/direction affine arrangement by

```text
B_z(alpha) = gamma(32*m + 256)
             * (|target_i|
                + sum_j |A_ji| (|lambda_j| + |representative_j|)),
```

where `m` is the exact contributing-entry count. This intentionally
over-approximates both arrangements with one alpha-independent local bound; it
does not use the observed `7.84e-36` discrepancy or suffix power.

## Hypotheses and discriminator

| ID | hypothesis | prediction |
|---|---|---|
| G1 | ordinary binary128 accumulation error owns the flutter | all 455 frozen ladder discrepancies are no larger than their independently derived `B_z`, including step 32/power 6 |
| G2 | the selected bound is structurally incomplete | at least one actual/affine difference exceeds `B_z` despite exact topology/entry ownership |
| G3 | the bound is valid but uselessly wide | containment passes but no ladder sample has affine distance from zero strictly above `B_z` |

Replay R24/R23 without new alpha points. Report per event entry count, scale,
bound, maximum discrepancy, containment margin and first ladder power whose
analytic zero-bound input is strictly greater than `B_z`. Require that sample
and all later samples select the predicted new face and remain Armijo-positive.

This validates a possible stable-side certificate only. It cannot freeze the
formula into the solver, choose a trial, fit gamma, force a mask or claim
binary64/runtime sufficiency.

