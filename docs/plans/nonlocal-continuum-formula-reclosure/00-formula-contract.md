# FCR0 — formula and algebra contract

Status: `VERIFIED / FCR0_PASS / REPORT_ONLY`

Candidate identity: `nuv-variational-fcr1`

This contract defines a new discrete variational model. It is rooted in the
published Nonlocal energy and its SISPH/SISSM predecessor, but makes every
normalization and pair-counting convention explicit where the paper or the
reference code leaves it implicit.

## State and pair convention

- `x_i`: position at the start of a time step.
- `y_i`: candidate position after the step.
- `delta_ij = (y_i-y_j) - (x_i-x_j)`.
- `r_ij = |x_i-x_j|`, `n_ij=(x_i-x_j)/r_ij` for viscosity.
- `m`: equal particle mass; `rho0`: rest density; `dt`: time step.
- Mathematical energies use a directed sum `sum_i sum_{j != i}`. A symmetric
  neighbor graph therefore contains each undirected pair twice.
- The implementation may visit each undirected pair once or each directed
  edge once, but its coefficient must reproduce the same total energy and
  gradient. FCR1 must prove the mapping explicitly.

## Density kernel

The three-dimensional cubic kernel has support `h`,
`q=2r/h`, and

```text
alpha = 3 / (2*pi*h^3)
W(r) = alpha * (2/3 - q^2 + q^3/2),        0 <= q < 1
     = alpha * (2-q)^3 / 6,                1 <= q <= 2
     = 0,                                  q > 2.
```

`w(r)` always means the derivative with respect to physical distance:

```text
w(r) = dW/dr = (dW/dq) * 2/h.
```

It has dimensions `length^-4`. A function returning only `dW/dq` is not
admissible under this identity. Density is
`rho_i(y)=sum_j m W(|y_i-y_j|)`, including the self contribution.

## Compression-only incompressibility

The operational density ratio is

```text
J_i(y) = max(rho_i(y)/rho0, 1).
```

The declared potential is

```text
Phi(y) = kappa/2 * sum_i (J_i(y)-1)^2.
```

This is the compression-only form implemented by the authors' SISPH and
Nonlocal code paths and avoids negative-pressure attraction at a free
surface. The literal two-sided reading of Eq. 7 remains an FCR1 comparator,
not the selected default. Derivative controls must avoid the non-smooth
threshold `rho=rho0`.

## Viscous incremental potential

Let

```text
P_n = n_ij n_ij^T
P_t = I - P_n
omega(r) = -dW/dr >= 0.
```

The position-based incremental potential is the time-integrated dissipation
from Eq. 10:

```text
Psi_dt(y;x) = m/(rho0*dt) * sum_i sum_j
  [mu/2 * |P_t delta_ij|^2 + lambda/4 * |P_n delta_ij|^2] * omega(r_ij).
```

For one undirected pair its gradient with respect to `y_i` is

```text
m*omega/(rho0*dt) *
  [2*mu*P_t delta_ij + lambda*P_n delta_ij].
```

Consequently a directed-edge implementation that also applies the
equal/opposite endpoint contribution uses the Eq. 13 coefficients
`lambda/2` for the normal part and `mu` for the tangential part. Using `W`
instead of `omega`, or using the same coefficient for both projections, is a
different solver identity.

## Surface potential

The dimensionless force spline uses `q=r/r0`:

```text
c(q) = q^2 - 1,                    0 <= q <= 1
     = 1 - (q-2)^2,                1 < q < 3
     = 0,                          q >= 3.
```

Define a continuous compact physical-distance potential
`C(r)=r0*C_hat(r/r0)` where

```text
C_hat(q) = q^3/3 - q - 2/3,                         0 <= q <= 1
         = q - (q-2)^3/3 - 8/3,                    1 < q < 3
         = 0,                                       q >= 3.
```

Then `dC/dr=c(q)` exactly. The additive constant makes the potential
continuous and zero at the support boundary; it does not change the force.
The declared surface energy is

```text
Upsilon(y) = gamma*m^2 * sum_i sum_{j != i} C(|y_i-y_j|).
```

For one undirected pair, the gradient magnitude is `2*gamma*m^2*c(q)`.
After multiplication by `dt^2/m`, the position-update coefficient contains
`gamma*m*dt^2`. `gamma` and the source code's untyped `strength` are not
assumed to be numerically identical.

## Total objective

```text
E(y;x,y_star) = m/(2*dt^2) * sum_i |y_i-y_star_i|^2
              + Phi(y)
              + Psi_dt(y;x)
              + Upsilon(y).
```

FCR0 checks each non-inertial term separately before any SISSM splitting is
implemented. FCR2 later checks that a solver update reduces the declared
objective or its declared residual under a bounded step policy.

## Mandatory FCR0 controls

1. `W` and `dW/dr` value/derivative goldens in both spline regions and at
   compact support.
2. Compression-only density energy above rest density and exactly zero
   derivative below rest density; two-sided Eq. 7 is reported separately.
3. Normal and tangential viscous derivatives with `omega=-dW/dr`.
4. Surface `dC/dr=c` on both polynomial branches and zero outside support.
5. Central directional derivatives versus analytical gradients at
   relative error `<=1e-7` away from branch boundaries.
6. Translation invariance and equal/opposite pair closure at `<=1e-12`.
7. Two fresh executions produce byte-identical reports.
8. The stopped `--term-self-test` still fails with its frozen report and the
   NPR1-A canonical self-test still passes unchanged.

The strict-f64 implementation passes. See the
[FCR0 evidence](../../development/nonlocal-continuum-fcr0-algebra-evidence-2026-08-20.md).
This selects only `FCR_ALGEBRA_CANDIDATE`.
