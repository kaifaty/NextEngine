# NSR3-B4E2D7R20R44 second inverse-contraction research

Status: `RESEARCH COMPLETE / TWO-SIDED EXACT DEFECT AUDIT SELECTED`.

## Question

Is R43's second verified-inverse failure another cancellation-dominated
certificate failure, or is its represented inverse genuinely unsuitable for a
centered fixed-point solve?

## Bounded discriminator

Replay the exact R43 target-bound depth-eight trajectory once. The hook applies
only the first frozen certificate and captures, but does not replace, the
second frozen matrix/inverse/RHS/solution tuple.

For the second candidate compute independently:

```text
R = I - A*X,
C = I - X*A.
```

Evaluate all entries as exact dyadic rationals and through the unchanged R38
Dot2/upward arithmetic. Require `4225/4225` containment and no underflow on
each side. Right contraction establishes a valid approximate inverse;
left contraction is the prerequisite for `e=z+C*e` centered refinement.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| S1 | inherited gamma arithmetic fails again | exact and Dot2 right/left norms are below one |
| S2 | only one inverse orientation is usable | exactly one of right/left norms is below one |
| S3 | the represented candidate is genuinely noncontractive | exact right or left norm is at least one |
| S4 | capture/arithmetic is invalid | frozen-root, underflow or exact-containment rejection |

Only S1 authorizes a later report-only centered RHS audit. R44 cannot apply a
second correction or continue torsion.
