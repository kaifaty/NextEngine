# NSR3-B4E2D7R20R6 nonnegative Newton representative research

Status: `RESEARCH COMPLETE / FINITE ACTIVE SET SELECTED`.

## Reduction

R20R5 rules out the unconstrained full-face Newton direction in both filled
cases. The correct local problem at `lambda=0` is

```text
minimize q(delta) = 0.5*delta^T H*delta - r^T*delta
subject to delta >= 0,
```

where the R20R4 selected `H` is positive definite. Its KKT system is the
strictly monotone linear complementarity problem

```text
delta >= 0,
w = H*delta-r >= 0,
delta_i*w_i = 0.
```

Equivalently, with `H=L*L^T`, it is a nonnegative least-squares problem. This
connects the required representative selection to the classical finite
Lawson-Hanson active-set family, not to another heuristic penalty update.
Block principal-pivoting methods solve the same NNLS/LCP structure and use
finite-convergence backup rules; see
[Portugal, Júdice and Vicente (1994)](https://doi.org/10.2307/2153286) and the
later active-set/block-pivot comparison by
[Kim and Park](https://faculty.cc.gatech.edu/~hpark/papers/SISC_082117RR_Kim_Park.pdf).

## Algorithm choice

Use deterministic single-variable active-set pivots for the oracle experiment:

1. start with `delta=0`, passive set empty;
2. enter the inactive coordinate with the largest certified positive
   `r-H*delta` (lowest source-row index breaks an exact tie);
3. solve the passive principal system without regularization;
4. if its unconstrained solution has a certified negative component, move to
   the first boundary hit and return hit coordinates to the inactive set;
5. stop only when passive positivity and inactive reduced-gradient signs are
   certified.

Moving one coordinate at a time sacrifices block-pivot speed but avoids
introducing an empirical anti-cycling policy in this first mathematical
discriminator. Every principal submatrix of the R20R4 SPD face matrix is SPD.

## Hypotheses

| hypothesis | discriminator |
|---|---|
| Q1: cone-aware representative selection resolves R20R5 | both filled NNQPs terminate with strict complementarity/KKT bounds and produce a certified exact-dual step |
| Q2: the direction is valid but needs nonlinear relinearization | Q1 holds, but an accepted candidate changes projector masks or remains above the full KKT tolerance |
| Q3: local complementarity is degenerate or numerically unresolved | a passive solve, boundary ratio, passive positivity or inactive reduced-gradient sign cannot be certified under the frozen cap |

This experiment still stops after one accepted nonlinear candidate. A complete
iterative semismooth solver is authorized only after Q1/Q2 evidence.
