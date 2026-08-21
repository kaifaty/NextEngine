# NSR3-B4DR1C2 failure-observability research -- 2026-08-21

Status: `COMPLETE / OBSERVABILITY_RECLOSURE_SELECTED / NO_RERUN_YET`

## Question

What is the narrowest experiment that can explain the first Hydro
`PRESSURE_NOT_CONVERGED` result without silently changing the frozen external
DFSPH profile?

## Finding

The patched upstream object already records pressure/divergence iteration
counts, last errors and convergence booleans. The adapter reads them, but
`project_and_measure` throws before returning its diagnostic record, and the
outer failure report serializes only the reason and two lifecycle booleans.
The missing evidence is therefore an adapter observability bug, not a need for
another upstream equation patch.

The result cannot currently tell us:

- which of the 24 steps failed;
- whether pressure reached the exact 100-iteration cap;
- the raw pressure residual relative to the frozen `0.01%` tolerance;
- whether divergence had already converged;
- whether the time-step bits still matched before rejection.

## Selected discriminator

Add a diagnostic-only failure context populated immediately after the
upstream step and before any convergence/contact rejection. Serialize step,
phase, pressure/divergence iterations, raw binary64 error bits, convergence
booleans and observed time-step bits. Preserve:

- exact upstream commit, source patch and static-library closure;
- every solver parameter, sample, boundary and contact formula;
- operation ordering and the existing successful payload layout;
- stop-on-first-failure and no-next-scenario behavior.

Run exactly one Hydro diagnostic process. It grants no R1C credit even if it
unexpectedly passes. The result selects a later remediation question; it must
not itself tune iteration caps, tolerances, warm starts, mass/volume, boundary
support or time step.

## Rejected alternatives

- Blindly raise the pressure cap: this hides whether the residual is slowly
  converging, stalled or invalid.
- Loosen `0.01%`: this changes the comparator before measuring the actual
  miss.
- Attach a debugger and infer fields from process memory: that is less
  reproducible than a canonical report from the already frozen getters.
- Run Dam/Orifice: the parent cost-aware gate forbids advancing past the
  first Hydro failure.

