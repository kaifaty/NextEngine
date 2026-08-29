# NSR3-B4E2D7R20R40 directional Krawczyk research

Status: `RESEARCH COMPLETE / DIRECTIONAL DEFECT AUDIT SELECTED`.

## Question

Does the actual torsion solve residual excite the enormous worst-case inverse
direction measured by R39, or can a directional Krawczyk/Neumann enclosure
resolve the six remaining signs?

## Derivation

Let `X` be the represented approximate inverse and

```text
r = b - A*x,
C = I - X*A,
z = X*r.
```

Then the exact solution error satisfies

```text
e = A^-1*r = (I-C)^-1*z,
||e||inf <= ||z||inf / (1-||C||inf),   when ||C||inf < 1.
```

Unlike R39's `||X||inf*||r||inf`, this bound retains the residual direction.
It remains a sufficient real-arithmetic certificate. The underlying
Krawczyk/Rump fixed-point form is reviewed in Theorem 10.6 of
https://www.tuhh.de/ti3/rump/intlab/ActaNumerica2010.pdf.

## Bounded evaluation

1. Recompute the left defect `I-XA` with the unchanged R38 Dot2 bound and an
   exact dyadic oracle for all 4225 entries.
2. Recompute the 65 R39 residual representatives/bounds.
3. Evaluate each component of `X*r_nominal` with Dot2. Add the explicit input
   uncertainty `sum_j abs(X_ij)*r_bound_j` upward.
4. Require exact dyadic `X*r` inside every combined bound, then form the
   directional infinity error and test all original solution signs.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| K1 | global inverse norm was the only blocker | left defect contractive and directional error resolves all 65 signs |
| K2 | residual genuinely excites the ill-conditioned mode | rigorous directional error leaves signs unresolved |
| K3 | right certificate does not transfer to a left defect | exact/Dot2 left `rho >= 1` |
| K4 | composed interval arithmetic is invalid | underflow, exact containment or parent-root rejection |

K1 may authorize a later opt-in inverse-audit candidate, not its installation.
K2 selects componentwise interval iteration or higher-precision/refined solve
research. R40 applies no correction or NNQP decision.
