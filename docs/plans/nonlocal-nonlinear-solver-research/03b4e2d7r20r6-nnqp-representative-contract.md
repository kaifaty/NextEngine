# NSR3-B4E2D7R20R6 NNQP representative contract

Status: `FROZEN / EXECUTION AUTHORIZED / REPORT ONLY`.

## Parent

- R20R5 implementation `e24d039e`, semantic
  `5ae59256f4b4836d9ceccf5e2ad6b1a071e6fce7097463b4b8b4ff1e921745b7`;
- route `PROJECTED_REPRESENTATIVE_SELECTION_REQUIRED`;
- unchanged exact v2 problem roots, selected projector derivative, raw face
  matrices and binary128 arithmetic profile;
- v2 cases are public development data.

## Frozen local solve

For each excited case at `lambda=0`, solve exactly

```text
min 0.5*delta^T H*delta-r^T*delta, delta>=0
```

using deterministic single-variable Lawson-Hanson-style active-set pivots.
The entering coordinate has maximum certified positive `r-H*delta`; exact ties
use the lowest original density-row index. Passive principal systems use
unregularized binary128 Cholesky with reconstruction and residual audits.

If a passive solution has a certified negative component, take the minimum
nonnegative ratio `x_i/(x_i-z_i)` to the boundary; exact ratio ties again use
the lowest original row. Return every certified boundary hit to the inactive
set in stable row order. Any unresolved sign or ratio selects the rejected
route rather than an epsilon tolerance. The hard transition cap is
`8*n*n+8`, fixed before execution.

At termination certify, with outward bounds:

```text
delta >= 0
w=H*delta-r >= 0
delta_i*w_i = 0
```

and a strictly positive `r^T delta`. Reuse the R20R5 exact-dual Armijo sequence
`alpha=2^-k`, `k=0..20`, coefficient `2^-10`. Stop after the first accepted
candidate and report mask changes plus the full frozen KKT tuple.

Controls include an interior positive solution, a boundary solution whose
unconstrained direction is negative, an SPD principal solve, an indefinite
rejection, exact ratio ties and the Armijo accept/reject pair.

Routes:

```text
NNQP_PARENT_REJECTED
NNQP_ACTIVE_SET_REJECTED
NNQP_DIRECTION_REJECTED
NNQP_GLOBALIZATION_REJECTED
NNQP_RELINEARIZATION_REQUIRED
NNQP_UNIT_DEVELOPMENT_CANDIDATE
```

No second nonlinear/Newton iteration, block pivots, diagonal regularization,
epsilon sign tolerance, parameter grid, timing, runtime/GPU integration, new
holdout or production/generalization authority.
