# NSR3-B4E2D7R18R4R1 normalized Krylov forcing research

Date: `2026-08-22`

Status: `RESEARCH_COMPLETE / CONTRACT_FROZEN / IMPLEMENTATION_NEXT`

## Question

Is R4's sole extra Krylov iteration caused by the scale-dependent Steihaug
forcing term, and what decision does an explicitly dimensionless forcing term
make at the exact outer-11/trial-0 boundary?

This stage is a one-iteration replay. It does not accept a trial, replace R4's
policy, continue the inner solve or rerun the complete transaction with a new
forcing term.

## Scale defect

The current normalized and dimensional solvers both use:

```text
eta_raw = min(0.5, sqrt(||r0||_2))
stop when ||r_next||_2 <= eta_raw * ||r0||_2.
```

Let `Ebar=alpha*E`, where the reference normalized objective has
`alpha=dt^2/M=1/7200`. Then `rbar=alpha*r`, so in the small-residual branch:

```text
left  -> alpha * ||r_next||
right -> alpha^(3/2) * sqrt(||r0||) * ||r0||.
```

After cancelling the common `alpha`, normalization makes the relative test
stricter by `1/sqrt(alpha)=sqrt(7200)=84.8528137423857`. The rule is also
dimensionally ill-formed because the square root is taken from a gradient norm
with units.

## Selected dimensionless candidate

Use the normalized stationarity measure already owned by R1/R2/R4:

```text
sigma = max_i ||g_i|| / dx
eta_dimensionless = min(0.5, sqrt(sigma))
stop when ||r_next||_2 / ||r0||_2 <= eta_dimensionless.
```

`sigma` is dimensionless, independent of particle count and exact across the
reference/aligned normalized profiles. It also ties the linear-solve forcing
to the same norm and length scale used by nonlinear admission. This is a
candidate to measure, not yet a selected solver policy.

For diagnosis, also evaluate:

```text
eta_inherited = min(0.5, sqrt(||r0||_2))
eta_mapped_dimensional = min(0.5, sqrt(||r0||_2 / alpha))
eta_fixed = 0.5
q = ||r1||_2 / ||r0||_2
```

The inherited R4 execution requires a second CG iteration, while the mapped
dimensional D7R13 execution stops after one. The replay must verify that `q`
lies on the corresponding sides of those two thresholds; otherwise the
mechanism hypothesis is contradicted.

## Exact target and replay

Recover the R4 reference-active transaction root
`9a3a57e7fcb29700c72c710936ef02ea7459cf2470b7d59ca663605df00c5ee9`
and outer `11`, trial `0`. Require the frozen binary64 anchors:

| Quantity | Bits |
|---|---:|
| stationarity | `0x3de517c90da7b861` |
| step norm | `0x3d8feecfa8d474e2` |
| predicted reduction | `0x3b47d038719e0000` |
| divided reduction | `0x3b47d08b70000000` |
| objective scale `alpha` | `0x3f223456789abcdf` |

Use `u` from the passed outer-10 update, rebuild one normalized sparse
workspace at the target current position, and execute only the first
Steihaug matrix-vector product and residual recurrence: once under the
reference profile and once under the exact aligned normalized profile. The
whole command is then reproduced by one process from each of two clean builds;
the candidate work budget remains exactly two HVPs.

The comparison between `q` and `eta_dimensionless` resolves only if their
absolute difference is at least 4096 binary64 ULPs of their common magnitude.
This prevents a branch selection from a near-equality artifact.

## Routes

1. `KRYLOV_FORCING_MECHANISM_CONTRADICTION`: `q` does not fail the inherited
   threshold and pass the mapped-dimensional threshold as the observed work
   requires.
2. `KRYLOV_FORCING_BOUND_REQUIRED`: the dimensionless comparison is nonfinite
   or lies within the frozen 4096-ULP resolution band.
3. `DIMENSIONLESS_FORCING_RETAINS_SECOND_ITERATION`: resolved
   `q > eta_dimensionless`.
4. `DIMENSIONLESS_FORCING_RESTORES_D7R13_WORK`: resolved
   `q <= eta_dimensionless`.

Both resolved routes are useful evidence. The former would support a new full
contract with a principled normalized 39-HVP baseline; the latter would
support a new full contract testing the dimensionless policy as a way to
restore 38 HVPs. Neither route changes R4 itself.

## Rejected alternatives

- Change R4's expected HVP count to 117: post-observation relaxation.
- Use the mapped dimensional threshold in production: it deliberately
  reintroduces the removed objective scale.
- Select fixed `eta=0.5` only because it likely stops early: invariant but not
  accuracy-adaptive.
- Use `||g||_2/dx` without particle normalization as the forcing state: it
  grows with the number of particles.
- Run the full transaction with any candidate before the replay contract
  resolves the exact first boundary.

## Decision

Freeze R4R1 as a no-acceptance one-HVP diagnostic. A resolved route may
authorize research/freeze of one new complete private transaction with an
explicit dimensionless forcing policy and a work expectation derived from
that policy. D7R19 remains blocked.
