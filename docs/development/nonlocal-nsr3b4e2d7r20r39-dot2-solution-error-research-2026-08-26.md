# NSR3-B4E2D7R20R39 Dot2 solution-error research

Status: `RESEARCH COMPLETE / PASSIVE-SOLUTION DISCRIMINATOR SELECTED`.

## Question

Does the R38 inverse certificate yield a sufficiently narrow rigorous error
for the exact 65-component passive solution that triggered the torsion
verified-inverse audit?

## Derivation

For the represented point system `A x* = b`, candidate `x` and residual
`r=b-Ax`, R38 gives

```text
||A^-1||_inf <= ||X||_inf / (1-rho),   rho >= ||I-AX||_inf,
||x-x*||_inf <= ||A^-1||_inf * ||r||_inf.
```

This is a direct real-arithmetic consequence of the Neumann series and does
not need the old triangular-factor eigenvalue estimate. The only numerical
work still needing proof is `||X||_inf` and `r`. Absolute row sums use upward
arithmetic. Each residual component uses the exact same R38 length-66 Dot2
form, now `[b_i,A_i,:] dot [1,-x]`, and is checked against an exact dyadic
oracle.

## Hypotheses

| ID | hypothesis | discriminator |
|---|---|---|
| S1 | compensated certificate resolves the active-set decision | all exact residuals contained and every passive value interval excludes zero |
| S2 | inverse certificate passes but solution residual/error remains too wide | finite bound, at least one sign interval contains zero |
| S3 | captured RHS/solution does not correspond | parent roots, dimension, old residual/bound or exact oracle mismatch |
| S4 | Dot2 assumptions fail | control, underflow or containment rejection |

## Selected experiment

Extend only the R37 observer payload with the RHS and solution already passed
to the selected verified-inverse audit. Add no solve. Reproduce R38, compute an
upward inverse-norm bound, Dot2/exact residual bounds, final infinity error and
the minimum signed separation `abs(x_i)-error`. Report every root and resolved
positive/negative/unresolved count.

R39 may select a later opt-in verified-inverse candidate only if all signs are
strictly resolved. It cannot modify the NNQP audit, apply the direction,
continue torsion, touch counterflow or claim production readiness.
