# Buoyancy from exact levels — first coupling consumer

| Field | Value |
| --- | --- |
| Research ID | `WB1` |
| Status | `FROZEN / NOT_RUN (prerequisite WR1, plan 10, delivered 2026-09-03)` |
| Parent | ADR-105 (Proposed); ADR-104 product check `CONTINUUM-WATER-BUOYANCY-P1`; ADR-076/081 one-pass step and exchange tuple |
| Purpose | the first rigid coupling of water: buoyancy and drag from exact levels through an exact impulse batch in the physics step input |

## Frozen scope

- Contracts: `WaterBuoyancyBatchV1` (records sorted by body id, at most
  `64`, each with the ADR-081 exchange tuple, displaced volume in cubic
  millimetres, linear impulse in micronewton-seconds, application point
  in micrometres); `PhysicsStepInputV3` with `external_impulses`;
  profile `WaterBuoyancyProfileV1` with `rho_water = 1000 kg/m^3`,
  `g = 9.81 m/s^2`, `k_damp = 2000` permille per second, bounds rule
  `canonical-aabb`.
- Physics owner: batch computed before the rigid step from the committed
  previous-tick levels and poses; the reference world and the PhysX
  backend apply each impulse once at the first substep.
- Reference scene: one dynamic cube, `0.5 m` edge, `50 kg`, resting on
  the basin floor at `x 6.5 m, z 2.0 m`; basin level `0.5 m`.
- Verification `xtask water-buoyancy` (`CONTINUUM-WATER-BUOYANCY-P1`):
  `600` ticks to settle, `SetLevel 1.5 m` at tick `600`, `600` more
  ticks, save at tick `900`, restore and continue, repeat generation.

## Prerequisite found before running (2026-09-03)

The canonical world (`GroundedCapsuleWorld`, used by both the reference and
the PhysX backend, which supplies only sweep queries) integrates only the
player capsule. Dynamic boxes have no mass, no gravity and no free
integration: they move only when pushed (R5b/R5j). A floating crate
therefore needs a new rigid increment first: an exact vertical free-body
model for dynamic boxes (mass in the body descriptor, gravity, floor
contact, velocity integration) or, narrower, buoyancy applied as a
kinematic level-following rule. The plan stays frozen; its gates are
unchanged; which prerequisite to take is a product decision recorded in
the water task-state. Resolved 2026-09-03 by plan `continuum-water/10`
(WR1): dynamic boxes carry `mass_microkilograms` and integrate exact
vertical free-body motion with floor support; the batch can now move a
crate through a velocity change of `impulse / mass`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 equilibrium | cube immersion at tick `600` (level minus bottom face) | `0.20 +- 0.05 m` |
| G2 follows the level | at tick `1200` the cube's immersion is again `0.20 +- 0.05 m` against the raised level and its bottom face is above the old level | pass |
| G3 determinism | identical state roots and step inputs on `game` and `headless`, live and restored, repeated generation | pass |
| G4 no coupling outside water | a second dynamic body outside every volume produces no record and its trajectory equals the run without the batch | pass |
| G5 no presentation read | reviewed: the batch code imports no presentation type | pass |
| G6 cost | batch computation for `64` bodies over `64` volumes on the reference host, release build | `<= 20 us` per tick |
| G7 closure | the step input with the batch round-trips byte-exactly and rejects an unknown body, an overflowed impulse and a duplicate body | pass |

Torricelli-free scene: the analytic equilibrium for a `50 kg`, `0.5 m`
cube is a displaced `0.05 m^3`, an immersion of `0.20 m`; the `k_damp`
drag makes the settling overdamped enough to reach `+- 0.05 m` within
`600` ticks by the frozen constants. Do not tune `k_damp`, the bounds
rule or the settling time after seeing the results.
