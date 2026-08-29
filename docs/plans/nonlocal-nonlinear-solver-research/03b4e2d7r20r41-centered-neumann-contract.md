# NSR3-B4E2D7R20R41 centered Neumann contract

Status: `FROZEN / REPORT-ONLY CENTERED ENCLOSURE AUTHORIZED`.

## Parent

- R40 implementation `c03b5116`, semantic
  `a999b65a7f3d4817d0aa58b1195942e63df23a2197f564d785701c6fb3dd6f7e`;
- left exact/Dot2 roots `5f3c0e06...73f6` / `61bf229b...aa50`;
- residual/correction/directional roots `0774694c...69df` /
  `5318c6a3...f2d9` / `9d02237b...e99`;
- R40 directional error `2.8783855494947104e14`, signs
  `24/35/6` positive/negative/unresolved.

## Frozen audit

Replay the exact R40 parent once. Materialize every represented left-defect
entry `C_hat_ij` and Dot2 bound `dC_ij`; require all 4225 entries to contain
the unchanged exact dyadic `C=I-XA`, with no underflow and the unchanged
contractive aggregate root.

Reuse the R40 residual representatives to materialize `z_hat` and `dz`, again
requiring all 65 exact `z=X*(b-A*x)` components inside their intervals.

Starting from an all-zero represented vector, execute exactly 32 shadow
recurrences

```text
y_(k+1) = Dot2([z_hat_i, C_hat_i,:], [1, y_k]).
```

Do not stop early. At depths `1,2,4,8,16,32`, independently evaluate

```text
g_hat_i = Dot2([z_hat_i, C_hat_i,:, -y_i], [1, y, 1]),
dg_i = dot2_bound_i + dz_i + up(sum_j dC_ij*abs(y_j)).
```

Require exact dyadic `g=z+C*y-y` inside every interval. Form

```text
g_inf = max_i up(abs(g_hat_i)+dg_i),
radius = up(g_inf / down(1-rho_left_bound)).
```

Use an error-free TwoSum/Dot2 center for `x+y`, require exact dyadic
containment, and test all 65 strict signs with its addition bound plus
`radius`. Report every checkpoint root, radius, sign counts, first fully
resolved depth and sign conflicts against the 59 signs already resolved by
R39. Candidate status requires the depth-32 interval to resolve all signs with
zero conflict.

Routes in precedence:

1. `CENTERED_NEUMANN_PARENT_REJECTED`;
2. `CENTERED_NEUMANN_APPARATUS_REJECTED`;
3. `CENTERED_NEUMANN_UNDERFLOW_REJECTED`;
4. `CENTERED_NEUMANN_CONTAINMENT_REJECTED`;
5. `CENTERED_NEUMANN_LEFT_NONCONTRACTIVE`;
6. `CENTERED_NEUMANN_SIGN_CONFLICT`;
7. `CENTERED_NEUMANN_SIGN_UNRESOLVED`;
8. `CENTERED_NEUMANN_SOLUTION_CANDIDATE`.

R41 executes one parent torsion replay and the existing 128 inverse columns.
It adds only bounded Dot2/exact-oracle shadow arithmetic: 4225 left-defect
entries, 65 residual entries, 65 `z` entries, 32x65 center-generation dots and
six fixed 65-row certificate/center audits. It adds zero factorization,
inverse column, principal solve, correction apply, NNQP transition/decision,
trial, state, counterflow work, tolerance/cap/Armijo change or timing. It cannot
install the refined solution or continue torsion.
