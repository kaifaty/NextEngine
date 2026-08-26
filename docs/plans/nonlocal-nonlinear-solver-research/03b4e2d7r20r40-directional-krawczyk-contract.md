# NSR3-B4E2D7R20R40 directional Krawczyk contract

Status: `FROZEN / REPORT-ONLY DIRECTIONAL ENCLOSURE AUTHORIZED`.

## Parent

- R39 implementation `9ade7a70`, semantic
  `83e1f7387a1ff2675d3899af904666734fd6b71f3e3079b34617d46f63d21bac`;
- RHS/solution/exact-residual roots `464263ae...42f8` /
  `dbf5c26c...b26e` / `8a0e41dd...79c4`;
- R39 solution root `19a90db0...4713`, error `1.03403e16`, signs
  `24/35/6` positive/negative/unresolved.

## Frozen audit

Replay R39 once with all R37/R38 roots exact. Compute `C=I-XA` entrywise using
the unchanged length-66 Dot2/upward bound and an independent exact dyadic
oracle. Require 4225/4225 containment, no underflow and report exact/Dot2 left
`rho`, roots and worst rows.

Recompute all R39 residual Dot2 representatives `r_hat_j` and bounds `dr_j`.
For each row evaluate `z_hat_i=Dot2(X_i,:,r_hat)` and

```text
dz_i = dot2_bound_i + up(sum_j abs(X_ij)*dr_j).
```

Require the independently exact dyadic `z=X*(b-Ax)` inside every
`z_hat_i +/- dz_i`. Let

```text
z_inf_bound = max_i up(abs(z_hat_i)+dz_i),
directional_error = up(z_inf_bound / down(1-rho_left_bound)).
```

Apply the unchanged strict sign predicates to the represented R39 solution.
Report positive/negative/unresolved counts, minimum separation, every root,
input-versus-Dot2 bound split and exact-to-bound ratios.

Routes in precedence:

1. `DIRECTIONAL_KRAWCZYK_PARENT_REJECTED`;
2. `DIRECTIONAL_KRAWCZYK_APPARATUS_REJECTED`;
3. `DIRECTIONAL_KRAWCZYK_UNDERFLOW_REJECTED`;
4. `DIRECTIONAL_KRAWCZYK_CONTAINMENT_REJECTED`;
5. `DIRECTIONAL_KRAWCZYK_LEFT_NONCONTRACTIVE`;
6. `DIRECTIONAL_KRAWCZYK_SIGN_UNRESOLVED`;
7. `DIRECTIONAL_KRAWCZYK_ERROR_CANDIDATE` when all signs resolve.

R40 executes one parent torsion replay, its existing 128 inverse columns,
4225 left-defect dots, 65 residual dots and 65 directional dots plus exact
oracles. It adds zero factorization, inverse column, solve, correction apply,
NNQP decision/transition, trial, state, counterflow work, tolerance/cap/Armijo
change or timing. It cannot install the method or continue torsion.
