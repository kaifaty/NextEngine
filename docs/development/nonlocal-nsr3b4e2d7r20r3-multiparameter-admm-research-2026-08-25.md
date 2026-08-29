# NSR3-B4E2D7R20R3 multiparameter ADMM research

Status: `RESEARCH COMPLETE / DEVELOPMENT PROBE SELECTED`.

## Evidence-led question

R20R2 globally couples density rows and certifies the supported transfer case,
but its filled edge/corner terminal primal-to-dual consensus ratios are about
`6.8` and `211`. A single fixed penalty is spending work unevenly between two
different constraints:

```text
B*s+d = u       (dimensionless density consensus)
s     = v       (contact/trust domain consensus).
```

Classical residual balancing is not adopted blindly. Wohlberg shows that
absolute primal/dual residual balancing is not invariant to problem scaling
and recommends relative residuals. More recent multiconstraint work shows that
one scalar penalty can remain slow when constraint blocks have different
scales, and derives independent penalty parameters as diagonal
preconditioning:

- B. Wohlberg, [ADMM Penalty Parameter Selection by Residual Balancing](https://arxiv.org/abs/1704.06209), 2017;
- L. Lozenski, M. McCann, B. Wohlberg,
  [An Adaptive Multiparameter Penalty Selection Method for Multiconstraint and Multiblock ADMM](https://doi.org/10.1109/OJSP.2026.3664275),
  IEEE OJSP 2026.

## Selected development method

Use independent positive penalties `rho_density` and `rho_domain`:

```text
H = (1+rho_domain) I + rho_density B^T B

H*s = t
      + rho_density B^T(u-d-y)
      + rho_domain (v-z).
```

The orthant/domain proxes and scaled-dual updates are unchanged. At every 64th
iteration, apply the paper's multiparameter spectral-radius approximation to
each block:

```text
rho_j(next) = ||Y_j(k)-Y_j(k-64)||_2
              / ||Z_j(k)-Z_j(k-64)||_2,
```

where `Y_j=rho_j*y_j` is the unscaled dual and `Z_j` is the corresponding prox
block (`u` for density, `v` for domain; the block operator is identity up to
sign). Zero numerator/denominator uses the paper's factor-ten fallback. Finite
positive proposals are bounded to `[2^-20,2^20]`; scaled duals are rescaled by
`rho_old/rho_new`; the binary128 Cholesky factor is rebuilt exactly.

The interval 64 is deliberately longer than the paper's empirical interval 5:
it gives the dominant fixed-point mode time to emerge and amortizes the dense
factorization. It is a frozen engineering choice for this development probe,
not a production default.

## Hypotheses

| ID | Hypothesis | Decisive observation |
|---|---|---|
| M1 | block penalty imbalance is the remaining barrier | both filled development cases certify before `2^14` |
| M2 | adaptive penalties help but active-set polish is still required | coherent reduction in all KKT/consensus residuals without full certificate |
| M3 | spectral updates are unstable for the nonsmooth active set | rejected factor/solve, nonfinite penalty, KKT regression or oscillatory cap route |

## Infrastructure correction

R20R2 audited the dense linear residual every iteration. Since a final KKT
certificate only consumes checkpoint states and every factor reconstruction is
already audited, R20R3 performs the expensive `H*s-rhs` audit only at frozen
checkpoints and immediately after a new factor. Ordinary triangular solves
still count exact work but do not redundantly scan the dense matrix.

R20 v2 filled cases are now development data. Even a successful M1 result
cannot validate generalization or production. It only authorizes construction
of new v3 holdouts and a separately frozen oracle replay.
