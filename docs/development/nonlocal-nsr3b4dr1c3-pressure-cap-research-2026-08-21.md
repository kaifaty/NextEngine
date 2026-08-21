# NSR3-B4DR1C3 pressure-cap research -- 2026-08-21

Status: `COMPLETE / CAP_SWEEP_SELECTED / NO_SWEEP_EXECUTED`

## Question

Does the step-1 pressure solve merely need a bounded increase above 100
iterations, or is its residual stagnating under the frozen mass, volume,
boundary and timestep profile?

## Analysis

At cap 100 the pressure residual is finite `0.82744058228594985` against
threshold `0.1`; divergence and timestep are exact. One endpoint cannot reveal
the local convergence rate. Raising the production/reference cap immediately
would therefore conflate diagnosis with remediation.

The cheapest discriminator is to rebuild no upstream code and run the same
first Hydro step from a fresh process at fixed maximum-iteration caps. A
monotonically decreasing residual with a bounded crossing supports a later
profile reclosure. A flat/increasing residual or no crossing by 300 redirects
research to the volume/mass and boundary calibration before any 24-step retry.

## Selected sweep

Use caps `25, 50, 75, 100, 125, 150, 200, 300` in ascending order. Each run:

- performs the exact R1C1 manifest preflight and full Hydro initialization;
- changes only DFSPH `MAX_ITERATIONS`;
- executes one upstream step;
- publishes no payload and runs no next scenario;
- reports exact iteration count, residual bits, convergence booleans and
  timestep bits under a distinct diagnostic identity.

Stop after the first converged cap. The existing R1C2 cap-100 point remains
evidence, but the sweep retains its own cap-100 run so that all points share
one implementation and report schema.

## Rejected alternatives

- Full 24-step cap experiments before locating the step-1 boundary.
- Tolerance sweep: it answers a different physical-accuracy question.
- Warm start: it cannot improve the very first pressure solve and changes
  later-step semantics.
- Simultaneous mass/volume change: it would prevent attributing the result to
  iteration budget alone.

