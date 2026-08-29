# NSR3-B4E2D7R20R47 third inverse-contraction research

Status: `RESEARCH COMPLETE / TWO-SIDED DEFECT AUDIT SELECTED`.

## Question

Is R46's third verified-inverse rejection the same cancellation-dominated
certificate defect as the first two boundaries, or has the unchanged torsion
trajectory reached a genuinely noncontractive inverse candidate?

## Bounded discriminator

Replay the exact two-replacement R46 prefix. Apply only the two frozen centered
certificates and capture, but do not replace, the exact third matrix/inverse/
RHS/solution tuple. Evaluate every entry of both

```text
I - A*X
I - X*A
```

with the unchanged exact-dyadic oracle and R38 Dot2/upward certificate. Require
`4225/4225` containment, no underflow and norms below one independently on both
sides. No third RHS residual or centered correction is evaluated in R47.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| T1 | inherited inverse enclosure fails for the same arithmetic reason | exact and Dot2 right/left norms are below one |
| T2 | only one fixed-point orientation remains usable | exactly one orientation is contractive |
| T3 | the candidate is genuinely noncontractive | exact right or left norm is at least one |
| T4 | the frozen trajectory/capture is not reproduced | parent root, cardinality or tuple identity fails closed |

Only T1 may authorize a later report-only third centered-solution audit. It
does not authorize a third trajectory replacement or a case-agnostic policy.
