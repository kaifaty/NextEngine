# NSR3-B4E2D6 augmented-Lagrangian path-oracle research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / NO_TRAJECTORY`

## Purpose

B4E2D5 proves the scalar PHR algebra, but not that multiplier updates converge
when `c` is a real nonlocal density function. Moving directly to the complete
vector Newton--Krylov solver would combine density derivatives, inner
globalization, multiplier policy, pressure-state ownership and nominal-scale
work in one experiment.

The next smallest falsifiable step is a manufactured one-dimensional path
through the already verified NSR3-B2 `2x2x2` corner fixture:

```text
y_i(q) = center + (1-q) * (x_i-center)
```

It retains eight true particle density constraints and 176 fixed support
samples. Only the primal motion is restricted to symmetric scalar `q`, making
the inner optimum independently bracketable and removing full-vector solver
behavior from the AL question.

## Manufactured problems

The base objective is `0.5*(q-q_star)^2`.

- active load: `q_star=0.01`, whose unconstrained state has all eight centres
  compressed by about `0.0023624`;
- inactive load: `q_star=-0.01`, whose optimum expands the fixture and must
  retain exactly zero multipliers.

For each centre, compute density, `dc/dq` and `d2c/dq2` analytically from the
radial kernel and fixed boundary geometry. The AL inner derivative is

```text
(q-q_star) + sum_i max(0, lambda_i + beta*c_i(q)) * dc_i/dq.
```

Use deterministic safeguarded bisection on `[-0.02,0.02]`; it is an oracle,
not the eventual inner optimizer. Then apply the PHR multiplier update.

## Required controls

1. Density path derivatives correspond to central differences away from
   topology/branch changes.
2. Cold active solve reaches primal, dual-change, stationarity and
   complementarity gates with nonnegative multipliers.
3. Warm start reproduces the solution in at most two outer iterations.
4. Inactive load stays expanded with zero multipliers.
5. Resetting the active solution to zero multipliers returns a positive-
   compression inner solution, proving pressure state is material.
6. A forced failure after a private multiplier update leaves public `q` and
   multipliers bit-exact.
7. A private one-multiplier mutation changes the deterministic result.

If every inner solve is exact and primal violation decreases monotonically but
the eight-outer cap is exhausted, the diagnostic may select the predeclared
semismooth primal-dual fallback. Any other numerical or ownership failure is a
hard FAIL.

## Authority boundary

This path oracle does not prove a complete AL fluid solve: it excludes
non-symmetric modes, contact, time integration, free surfaces and multiplier
transport. PASS can authorize only a separately frozen dense vector oracle.
