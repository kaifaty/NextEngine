# NSR3-B4E2D7R dimensionally stable AL commit research

Date: `2026-08-22`

Status: `RESEARCHED / CONTRACT_READY / NO_TRAJECTORY`

## Question

B4E2D7 proves that the full 24-coordinate PHR augmented-Lagrangian problem is
correctly differentiated and that its trust-region inner solves converge. It
nevertheless admits the eighth cold state using
`max(delta_lambda)/beta <= 1e-8`, after which one warm update changes a
multiplier by `3.497549225794927e-7 J`, above the independently frozen
`1e-8 J` state-correspondence limit.

Is this evidence against the AL formulation, or did the stage use the wrong
quantity to decide when pressure state can be committed?

## Dimensional diagnosis

For an active unilateral constraint, the selected PHR update is

```text
lambda_next = lambda + beta*c,
```

where `c = rho/rho0 - 1` is dimensionless, while `lambda` and `beta` have
energy units. Therefore:

```text
delta_lambda / beta = c                 dimensionless feasibility proxy
delta_lambda                             pressure-state change in joules
delta_pressure = delta_lambda*rho0/mass pressure-state change in pascals
```

The failed cold gate bounded the first quantity and the warm gate bounded the
second. They are not interchangeable. At `beta=1226.25 J`, the cold limit
allows an absolute update of `1.22625e-5 J`; the warm limit permits only
`1e-8 J`. For this fixture the latter corresponds to `8e-5 Pa`.

The observed outer sequence is regular rather than stalled: primal violation
contracts by approximately `0.171x` and every inner solve passes without a
rejected trust trial. The evidence therefore rejects immediate beta tuning and
does not trigger the semismooth fallback.

## External-method check

Current ALM literature supports keeping distinct KKT/feasibility tests:

- Andreani et al., *On scaled stopping criteria for a safeguarded augmented
  Lagrangian method with theoretical guarantees*, explicitly scales the KKT
  conditions by a multiplier norm rather than changing the physical units of
  the multiplier state ([DOI 10.1007/s12532-021-00207-9](https://doi.org/10.1007/s12532-021-00207-9)).
- Jia et al., *An augmented Lagrangian method for optimization problems with
  structured geometric constraints*, derives the multiplier update so that
  augmented-gradient stationarity equals ordinary Lagrangian stationarity and
  requires both infeasibility and complementarity information; it warns that
  pure infeasibility is insufficient ([DOI 10.1007/s10107-022-01870-z](https://doi.org/10.1007/s10107-022-01870-z)).
- Eckstein's approximation criterion permits inexact AL subproblem solves from
  primal-iterate and augmented-gradient norms, supporting the separation of
  inner stationarity from outer state admission
  ([Optimization Online RRR 61-2000](https://optimization-online.org/2001/02/272/)).

Those results do not prescribe our `1e-8 J` transaction threshold or a second
confirmation update. Those are an engineering extension required because a
future authoritative pressure field is continuation state and must not be
published while one more deterministic update materially changes it.

## Selected commit protocol

Retain the B4E2D7 fixture, objective, `beta`, derivatives and trust solver.
Replace only the outer admission protocol under a new identity:

1. solve/update privately, reporting primal, stationarity, complementarity,
   absolute multiplier change, equivalent pressure change and RMS position
   change normalized by spacing;
2. mark a provisional state only if all old KKT gates pass and both
   `max|delta_lambda| <= 1e-8 J` and `RMS(delta_x)/dx <= 1e-8` pass;
3. execute one more complete private inner solve and PHR update;
4. require the confirmation state to pass the same gates and be within those
   same state-change limits of the provisional state;
5. commit the confirmed state exactly once. Before confirmation, the public
   state remains bit-exactly unchanged.

The total cap is 14 updates including confirmation. This is derived from the
measured `~0.171x` contraction with margin: the old eighth update is followed
by approximately `3.50e-7`, `5.98e-8`, `1.02e-8` and `1.75e-9 J` updates,
leaving two bounded slots beyond the expected provisional/confirmation pair.
It is a convergence-certification cap, not a performance target.

## Non-regression and decision routes

- The first eight records must reproduce the exact B4E2D7 state root
  `04a9c033...d95e` and exact outer-array root `9bffc61a...82c2`.
- All B4E2D7 derivative, inactive, reset, rollback, mutation and fixed-boundary
  facts remain mandatory.
- A confirmed pair selects `AL_DENSE_STABLE_COMMIT` and authorizes only a tiny
  multi-step transaction contract.
- Exact monotone cap exhaustion with successful inner solves selects the
  already declared semismooth primal-dual research fallback.
- Any prefix, derivative, trust, monotonicity, ownership or rollback mismatch
  is a hard failure.

Nominal Dam/Hydro, runtime pressure schema, persistence, PhysX/GPU and
production remain blocked.
