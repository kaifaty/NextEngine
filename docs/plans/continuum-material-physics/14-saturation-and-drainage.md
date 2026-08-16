# Package 14T — Saturation and drainage

## Outcome

Add the smallest exact saturation/drainage extension to the passing dry-sand
owner state, without yet coupling free-surface DFSPH water.

## State and profile

Extend each material sample with exact bounded water mass/saturation and only
the pore/drainage history required by one calibrated profile. The immutable
profile freezes porosity, permeability, drainage cadence/boundaries,
capillary/hysteresis policy and monotone mappings from saturation to friction,
cohesion/yield and drag. Invalid/non-monotone or mass-inconsistent curves reject
before patch activation.

Solid mass and water mass are tracked separately. Drainage/infiltration uses a
closed exact boundary-flux input with stable interface keys; no solver writes
another owner's state. All future-affecting saturation/history fields enter
the exact active checkpoint from Package 13T.

## Validation ladder

1. prescribed infiltration column and wetting front;
2. drainage and recovery under fixed boundary conditions;
3. dry/damp/saturated direct shear at the frozen loads;
4. plate sinkage and suction/release;
5. prescribed single-wheel force/slip/sinkage curves;
6. repeated wet/dry cycles and exact active save/restart.

Metrics declare separate solid/water mass closure, external flux, mechanical
work and hysteresis. A monotone visual darkening or deeper rut is not evidence.

## Escalation and exit

The simplified model passes only if one fixed parameter family fits both the
drainage and mechanical curve families under predeclared thresholds. If those
families require contradictory parameters, record the failure cluster and run
one bounded two-phase/mixture-theory MPM comparator against the same corpus.
Complexity is not selected for visual quality alone.

Package 15T remains blocked until saturation, repeated cycles and exact
save/restart pass. Fallback stays the passing exact dry-sand profile or
accepted rigid terrain, selected before activation.

## Non-goals

Free-surface water transfer, erosion, particle emission/removal, cross-region
flow, full poromechanics, sleep/wake and full vehicle.
