# NSR3-B4E2D7R20R5 globalized Newton-direction research

Status: `RESEARCH COMPLETE / ONE-STEP DISCRIMINATOR SELECTED`.

## Question

R20R4 found a full-rank selected face Hessian at `lambda=0`, but this alone
does not prove that its Newton direction is compatible with the nonnegative
dual cone or with weak box-projector kinks.

For the concave dual

```text
d(lambda) = min_{s in D}
            0.5*||s-t||^2 + lambda^T(c+A*s), lambda>=0,
s(lambda) = P_D(t-A^T*lambda),
r(lambda) = grad d = c+A*s(lambda),
```

the natural complementarity residual is

```text
F(lambda) = lambda - P_+(lambda+r(lambda)).
```

At `lambda=0`, rows with strictly positive `r_i` select the Newton face. On
that face `F=-r` and a selected generalized Newton equation is

```text
(A_I J_D A_I^T) delta_I = r_I.
```

The matrix and derivative were validated in R20R4. Newton/proximal augmented-
Lagrangian QP solvers such as [QPALM](https://arxiv.org/abs/1911.02934) motivate
this direct local solve and dual line globalization. The recent analysis of
[degenerate polyhedral projection](https://arxiv.org/abs/2607.12551) is the
reason not to infer global regularity from one nonsingular representative.

## Hypotheses

| hypothesis | discriminator |
|---|---|
| N1: local model is already in the unit Newton basin | every excited case has a nonnegative direction, accepts `alpha=1`, preserves a valid projector state and closes the frozen KKT tuple |
| N2: Newton geometry is useful but active sets must be relinearized | every excited case has a certified feasible ascent step, but some backtrack, change masks or remain above KKT tolerance |
| N3: the selected face direction is incompatible with the dual cone | an excited direction has a resolvably negative component, nonpositive slope, failed solve, or no certified Armijo step |

## Chosen experiment

Compute exactly one unregularized binary128 Newton direction from `lambda=0`.
Audit factor reconstruction and the linear residual. A direction is dual-cone
feasible only if every component is nonnegative within its outward error bound.

For a feasible positive-slope direction, test the fixed dyadic sequence
`alpha=1, 2^-1, ..., 2^-20`. Evaluate the exact projected dual at every trial
and accept the first step whose lower dual bound exceeds the old upper bound
plus `2^-10 * alpha` times a lower directional-derivative bound. Record box and
ball mask changes and the complete frozen KKT tuple.

Stop after that one accepted candidate. This separates the need for projected
representative selection from ordinary relinearized semismooth Newton without
fitting an iterative solver to the development corpus.
