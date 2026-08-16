# Package 15T — Closed free-water/terrain flux

## Outcome

Join a passing exact-active water lane and passing saturated-terrain lane
through one bounded atomic interface-flux batch. Neither solver mutates the
other owner's state directly.

## Preconditions

- water W5 exact active persistence is PASS;
- terrain 14T saturation/drainage and 13T persistence are PASS;
- one bounded sealed water/terrain interface consumer is selected;
- a new exact flux profile freezes cadence, interface geometry/keys,
  capacities, permeability rules, particle placement/removal and metrics.

## Interface state and transfer

At a declared composite substep, both solvers consume frozen prior states and
produce proposals keyed by stable interface/sample identity. The engine-owned
candidate batch contains signed fixed-point water mass, momentum and energy
transfer per interface key plus expected owner revisions/roots.

DFSPH V1 uses uniform-mass samples, so sub-particle transfer accumulates in an
exact per-interface mass/momentum reservoir. When the reservoir crosses one
sample mass, a profile-declared canonical operation removes or creates a whole
sample:

- removals select eligible boundary samples by stable interface key and
  `SampleId`;
- creation derives stable identity from interface identity, source composite
  transaction and emission ordinal;
- placement, initial velocity and momentum exchange follow one exact profile;
- the remainder stays in the persisted interface reservoir;
- count/capacity and total free+terrain+reservoir water mass validate before
  publication.

This is a new water-profile/state version and cannot reinterpret the sealed
W0/W5 bytes. It requires its own consumer-backed promotion boundary.

## Atomic commit and failures

Water, interface reservoir, saturated terrain and any rigid reactions publish
as one composite PhysicalStep generation or none publishes. Duplicate or
conflicting interface batch, stale owner, capacity, nonfinite, mass/momentum
closure, sample-placement or downstream solver fault rejects the whole step.
No next-tick compensation, partial absorption or water-only/terrain-only
commit is allowed.

## Evidence

- infiltration from a bounded free-water layer and drainage back to it;
- total water mass and momentum closure including the reservoir;
- deterministic whole-sample threshold crossing and ID/order permutations;
- repeated exchange cycles and exact save/restart;
- capacity, stale/collision, corrupt reservoir and failure atomicity;
- comparison with the no-exchange passing owner baselines.

## Non-goals

Erosion/sediment transport, arbitrary shoreline, cross-region water, adaptive
particles, multiple fluid phases, full two-phase poromechanics and ocean scale.
