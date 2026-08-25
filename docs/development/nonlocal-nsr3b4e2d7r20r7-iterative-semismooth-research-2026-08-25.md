# NSR3-B4E2D7R20R7 iterative semismooth research

Status: `RESEARCH COMPLETE / COLD-START ITERATION SELECTED`.

## Natural residual away from lambda zero

For

```text
F(lambda)=lambda-P_+(lambda+D^-1*r(lambda)),
```

use `D_i=||a_i||^2` only to select the scale-invariant projection branch.
Rows whose projection argument is positive form `I`; all other next
multipliers are fixed to zero by the selected natural-residual derivative.

Let `p=lambda+delta` be the next local multiplier representative. On `I`, the
Newton equation and cone constraint reduce to

```text
minimize 0.5*p_I^T H_II*p_I - b_I^T*p_I,
p_I >= 0,
b_I = r_I + (H*lambda)_I.
```

Rows outside `I` have `p=0`. Thus the R20R6 NNQP is not a special startup
trick; it is the bound-constrained semismooth Newton subproblem at every
iteration. The exact dual remains the globalization merit function, consistent
with semismooth proximal-QP practice such as
[QPALM](https://arxiv.org/abs/1911.02934).

## Risks measured explicitly

- projector box/ball masks can change after every accepted step;
- the selected natural face can become rank deficient even though the startup
  face was full rank;
- a zero projection argument creates a generalized-derivative ambiguity;
- a local NNQP can be exact while its nonlinear direction has nonpositive dual
  slope;
- monotone dual ascent can still converge too slowly under repeated face
  changes.

These are classified separately. A failed factor or ambiguous sign is not
reported as ordinary non-convergence.

## Hypotheses

| hypothesis | discriminator |
|---|---|
| S1: cold-start semismooth relinearization closes the development corpus | every case reaches the frozen complete KKT certificate within 32 accepted nonlinear iterations |
| S2: globalization is correct but face churn stalls | every accepted step has certified dual ascent, but a filled case remains uncertified at the fixed cap |
| S3: a structural degeneracy blocks ordinary semismooth Newton | a natural-face sign, principal factor, NNQP KKT, slope or Armijo condition becomes unresolved/rejected |

The experiment is about convergence architecture, not speed. No timing is
admitted on the shared host.
