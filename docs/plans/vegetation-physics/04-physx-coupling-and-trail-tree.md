# V4 — PhysX coupling and trail-tree vertical

## Outcome

Integrate one active structural tree with the production PhysX world and the
player command path. Prove contact-load exchange, atomic detached-body handoff
and the trail-blocking product loop without a second rigid writer.

## Coupling profile

1. Freeze prior structural state, collision proxies and rigid projection.
2. PhysX applies player/tool inputs, integrates dynamic bodies once and emits
   canonical loads against the frozen tree proxies.
3. Validate one batch keyed by world generation, tick/substep and structure
   identity; records bind complete bodies/proxies, revisions, contact point and
   fixed-point impulse/moment.
4. The living solver consumes that batch plus wind and advances its graph once.
5. Validate both candidates and publish one composite `PhysicalStep`, or none.
6. Build the next proxy projection only from the accepted graph.

The one-substep stagger is exact profile identity. No wall-time-selected
coupling loop, delayed unrecorded force, contact callback mutation or tree write
to a PhysX transform is permitted.

## Detached handoff

When V3 selects the trunk split, stage one bounded compound body from the
detached graph component. Validate geometry/collision capacity, stable new body
identity, material mapping, mass, CoM, inertia and V0 momentum thresholds.
Graph topology, owner mapping, PhysX body creation and handoff receipt publish
atomically. After commit the living solver cannot advance the detached mass.

V1 falling-tree flexibility, branch re-fracture and tree-to-tree damage are
absent; the rigid compound may collide with terrain, player and existing rigid
bodies through normal PhysX paths.

## Product fixture

- Activate the exact V0 trail/tree/project closure.
- Player approaches, performs the recorded notch/back-cut interaction trace and
  leaves the result to wind/gravity/section physics.
- Headless uses the same production commands and contact/query path, never a
  test-only mutation.
- Verify no cut, insufficient cut and successful trail-blocking branches.
- Debug presentation extracts graph, cells, forces, proxies and handoff state
  only after commit; renderer cadence/device absence cannot change roots.

## Failure and exit

Stale/missing body, batch collision, nonfinite/overflow, capacity excess,
PhysX rejection, handoff mismatch or structural failure rejects both owners.
No static/scripted/decorative fallback occurs after activation.

`VEGETATION-COUPLING-P1 = PASS` requires the V0 impulse, penetration,
mass/CoM/momentum and trail outcome thresholds; exact command/registration/
worker permutations; and one complete owner handoff with PhysX as sole rigid
writer.
