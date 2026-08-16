# Package 20X — Optional lossy sleep/wake conversion

## Outcome

Evaluate a material-specific compact sleeping representation only after that
material's exact active persistence passes. This package is not required by
the first pinned-active water or terrain vertical.

## Conversion contract

Each material receives a separate versioned active↔sleep conversion profile.
One immutable receipt binds source/destination representation IDs and roots,
profile hash, exact conversion boundary and error in:

- solid and/or water mass;
- linear/angular momentum;
- occupied volume and surface/height displacement;
- deformation/plastic/saturation/hysteresis history required by that material;
- material-specific gameplay observables such as float level, sinkage or
  force/slip curve.

Active and sleeping roots are different representations and are never
expected to match. Threshold failure retains the source generation. Wall time,
camera distance and memory pressure may request conversion but cannot choose
its authoritative commit.

## Evidence

- active → sleep → wake versus a declared exact-active reference trajectory;
- no fewer than the profile-declared repeated cycles, measuring accumulated
  rather than single-cycle drift;
- save/restart in sleeping and transition states;
- corrupt/stale/capacity/conversion failure before source eviction;
- `game`/headless equality for transition choice and receipts.

Only a bounded plateau below every frozen cumulative threshold admits the
sleep representation. Otherwise the region stays pinned active or the feature
disallows persistent modification; immutable-base reconstruction is forbidden.

## Non-goals

One universal sleep format, exact root parity with active state, automatic
quality reduction, cross-region transfer and use as evidence for the first
water/terrain persistence gates.
