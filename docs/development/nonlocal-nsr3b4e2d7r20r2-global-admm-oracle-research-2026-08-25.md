# NSR3-B4E2D7R20R2 globally coupled ADMM oracle research

Status: `RESEARCH COMPLETE / ORACLE SELECTED`.

## Corrected diagnosis

All R20 TRQPs are feasible because `s=0` satisfies density, contact and trust
constraints. Cyclic primal Dykstra nevertheless leaves both filled blind cases
far from KKT after `2^18` cycles. Its sequence of local halfspace projections
does not account globally for the highly correlated filled-box density rows;
adding more cycles is neither authorized nor a credible oracle strategy.

The replacement must be mathematically equivalent, numerically independent of
the R65 candidate and globally couple all density rows in each iteration.

## Scaled split formulation

Use the frozen positive row scales `sigma_i` and define

```text
B_i = A_i / sigma_i
d_i = c_i / sigma_i.
```

Positive row scaling preserves the feasible set. Split the projection QP as

```text
min 0.5*||s-t||^2 + I_-(u) + I_D(v)
s.t. B*s+d = u
     s     = v,
```

where `I_-` is the indicator of the nonpositive orthant and `I_D` is the
contact-box/trust-ball indicator. Scaled ADMM with fixed `rho=1` has the global
primal step

```text
(2I + B^T*B) s = t + B^T(u-d-y) + (v-z),
```

followed by

```text
u = min(0, B*s+d+y)
v = projection_D(s+z)
y = y + B*s+d-u
z = z + s-v.
```

The dense matrix is small for the R20 corpus (at most 192 scalar variables),
constant per case and symmetric positive definite. A binary128 Cholesky
factorization couples all rows and removes the local-cycling mechanism of the
rejected oracle. This is offline reference work; the dense factorization is
not a runtime architecture proposal.

At a fixed point, normalized density multiplier `y` maps back to the original
QP multiplier as `lambda_i=y_i/sigma_i`. The existing independently derived
binary128 primal, projected-dual, complementarity, stationarity and gap audit
then certifies the same TRQP.

## Hypotheses

| ID | Hypothesis | Decisive result |
|---|---|---|
| A1 | local cyclic ordering caused the blind oracle failure | both blind cases certify under globally coupled ADMM |
| A2 | the strict `2^-70` audit or formulation has a deeper obstruction | at least one case remains unresolved with coherent ADMM residual decay |
| A3 | dense/scaled formulation is inconsistent with frozen sparse TRQP | Cholesky, row-scale, weak-duality, KKT or dense-control rejection |

## Independence and limits

The oracle uses a primal consensus split, dense binary128 normal-equation solve
and fixed row normalization. The candidate uses binary64 sparse composed-dual
transactions and projected face directions. They share only immutable problem
data and the final mathematical KKT definition.

Use zero consensus initialization, fixed `rho=1`, checkpoints
`2^4,2^6,...,2^16`, first certified checkpoint selection and no adaptive rho,
residual stop, timing or post-observation parameter grid. Quiet cases retain
the exact cycle-zero KKT precheck. If `2^16` is unresolved, stop and study the
reported ADMM/KKT components; do not extend either oracle depth.
