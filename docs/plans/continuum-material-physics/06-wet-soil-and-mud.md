# 06 — Wet soil and mud

## Outcome

Add the smallest saturation-dependent extension that explains player-visible
soft/wet ground after the dry-terrain lane is stable.

## First model

Add bounded water content/saturation and pore/drainage state to the terrain
material, with explicit permeability, drainage cadence and functions mapping
saturation to cohesion, yield/friction and drag. Preserve total solid mass and
water mass. Parameter curves are immutable profiles and must be monotone where
the model assumes monotonicity.

Water transfer between a free-surface DFSPH region and terrain occurs only
through a closed region-interface flux batch at a fixed substep. Neither solver
mutates the other's state directly. Fluxes sort by stable interface/sample key
and commit atomically with both owner regions.

## Validation ladder

1. infiltration column and drainage/recovery;
2. saturated vs dry shear box;
3. foot/plate sinkage and suction/release;
4. tire slip curve over dry, damp and saturated profiles;
5. repeated wet/dry cycle and save/restart;
6. free-water exchange with mass closure.

## Escalation rule

The simplified model is rejected only when one fixed parameter family cannot
fit the declared drainage and mechanical scenarios without contradictory
errors. Then run a bounded two-phase/mixture-theory MPM spike against the same
corpus. A more complex method is not selected for visual richness alone.

## Exit

Pass requires conservation, stable hysteresis/drainage behavior, repeat and
restart equivalence, and improvement over dry-terrain fallback on the wet
scenario corpus. Otherwise the production fallback remains dry deformable or
rigid terrain.
