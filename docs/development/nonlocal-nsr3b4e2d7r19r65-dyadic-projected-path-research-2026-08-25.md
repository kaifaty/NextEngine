# NSR3-B4E2D7R19R65 dyadic projected-path research

Date: `2026-08-25`

Status: `RESEARCH COMPLETE / PROJECTED-PATH SEARCH PROBE SELECTED`.

## Mathematical correction

For nonnegative variables, projected Newton uses

```text
lambda(alpha) = max(0, lambda + alpha*d),
```

not the chord `lambda+alpha*(max(0,lambda+d)-lambda)`. The former can cross
many coordinate bounds before `alpha=1`. This distinction is explicit in
Bertsekas' projected-Newton form and aligns with GPCG face changes:

- [Bertsekas, Projected Newton methods](https://doi.org/10.1137/0320018)
- [Moré--Toraldo GPCG](https://doi.org/10.1137/0801008)

## Selected deterministic search

After one Hildreth identification sweep, use 15 Jacobi-PCG Hessian products to
obtain face correction `d`. Test the projected path in descending fixed order:

```text
alpha_k = 2^-k,  k=0..15
trial_k = max(0, lambda + alpha_k*d)
delta_k = trial_k - lambda
```

For fixed joint correction the exact sparse quadratic change is

```text
Delta phi_k = -raw^T delta_k + 0.5*||A^T delta_k||^2.
```

It needs one R64 transpose and no action. Accept the first finite candidate
with `Delta phi_k<0`; this is the largest predeclared dyadic projected step
that strictly decreases the modeled dual objective. If no candidate passes,
reject the line. Freshly reconstruct before the joint projection.

## Frozen exploratory schedule

```text
outer blocks                         16
Hildreth identification sweeps        1 per outer / omega=1
face PCG Hessian products             15 per outer / 240 maximum
projected-path candidates              1..16 per outer
candidate order                        1,1/2,...,2^-15
acceptance                             first strict finite model decrease
tolerance                              none
dense Gram                             none
timing                                 none
```

The worst-case candidate budget uses 528 total `A^T` and 272 total `A` calls;
with frozen topology and maximal coordinate work it remains below the FISTA
structural reference before execution.

Record candidate count, selected dyadic exponent/alpha, model change,
predicted/applied batch zeros, face-PCG residual, recurrence/fresh gap and all
existing primal/KKT/joint/model/work roots.

## Predeclared classifications

```text
DYADIC_PROJECTED_PATH_ACCELERATION_CANDIDATE
DYADIC_PROJECTED_PATH_LINE_REJECTED
DYADIC_PROJECTED_PATH_CURVATURE_REJECTED
DYADIC_PROJECTED_PATH_RECURRENCE_REJECTED
DYADIC_PROJECTED_PATH_MODEL_REDUCTION_REJECTED
DYADIC_PROJECTED_PATH_REFERENCE_RETAINED
```

Acceleration retains the same strict FISTA dominance and lower-work gates.
