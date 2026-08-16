# Structural vegetation physics — standalone implementation roadmap

Status: `PLANNED / NOT_ACTIVE`; post-v1 isolated program. Governing candidate
architecture: [SPEC-37](../../architecture/37-layered-physical-world.md),
[SPEC-38](../../architecture/38-structural-vegetation-physics.md) and
[ADR-073](../../architecture/adr/073-layered-physical-world-and-living-structures-track.md).
Package V0 is open and no `VEGETATION-*` ProductCheck has run.

This directory is the resume and execution surface for a dedicated vegetation
worktree. The main [Next Engine roadmap](../../roadmap.md) keeps the track
inactive until `VEGETATION-BEAM-REF-P1 = PASS`. Work here cannot change the
current PhysX, content, save/replay or public-contract baseline by implication.

## Selected product result

One pinned-active procedural tree stands beside a trail. It has a structural
trunk and major-branch graph, gravity sag and deterministic analytical wind.
The player uses the production interaction/command path to cut a directional
notch and back cut. Remaining section geometry forms a hinge, fails under
computed load, and the detached component transfers atomically to one bounded
PhysX compound body that may block the trail. Debug rods, section cells,
stress/contact arrows and collision proxies are sufficient presentation.

If the capability is unavailable before activation, the project uses an
authored static rigid-tree variant. An active structural tree never silently
becomes static, scripted, decorative or GPU-authoritative.

## Stage graph

```text
V0 Product/profile/evidence closure             OPEN / BLOCKS CODE
 └─ V1 Serial structural formulation oracle     NOT_STARTED
     ├─ V2 Rooted tree graph + analytical wind  NOT_STARTED
     │   └─ V3 Section damage + graph fracture  NOT_STARTED
     │       └─ V4 PhysX coupling + trail tree  NOT_STARTED
     │           └─ V5 Exact active persistence NOT_STARTED
     │               └─ V6 Forest LOD + budget  NOT_STARTED
     │                   └─ V7 Production promotion NOT_STARTED
     └─ VG Optional GPU correspondence mirror   NOT_STARTED / NON_BLOCKING

V5/V7 ── VF future thermal/fire/root/soil lanes NOT_STARTED / OUTSIDE BASE
```

| Stage | Specification | Exit evidence | Blocks |
|---|---|---|---|
| V0 | [Product, profile and evidence closure](00-product-profile-and-evidence-closure.md) | Geometry, wood, wind, cut, state, corpus, thresholds, capacities and stop rules are exact | every code stage |
| V1 | [Serial structural formulation oracle](01-serial-structural-formulation-oracle.md) | `VEGETATION-BEAM-REF-P1 = PASS`; one formulation/integrator/state is frozen | main-roadmap activation, V2, VG |
| V2 | [Rooted tree graph and analytical wind](02-rooted-tree-graph-and-wind.md) | `VEGETATION-TREE-P1 = PASS` without collision/fracture | V3 |
| V3 | [Section damage and graph fracture](03-section-damage-and-graph-fracture.md) | `VEGETATION-FRACTURE-P1 = PASS` under prescribed loads/cuts | V4 |
| V4 | [PhysX coupling and trail-tree vertical](04-physx-coupling-and-trail-tree.md) | `VEGETATION-COUPLING-P1 = PASS`; production command loop and handoff work | V5 |
| V5 | [Exact active persistence](05-exact-active-persistence.md) | `VEGETATION-PERSISTENCE-P1 = PASS` | V6, VF stateful lanes |
| V6 | [Forest LOD and performance](06-forest-lod-and-performance.md) | `VEGETATION-LOD-P1` plus the V0 THOTH workload pass | V7 |
| V7 | [Production promotion](07-production-promotion.md) | Consumer-backed Accepted decision and all declared checks | shipped claim |
| VG | [GPU correspondence mirror](vg-gpu-correspondence.md) | Aggregate report for pinned devices | no authority or promotion stage |
| VF | [Future thermal, root and continuum lanes](vf-thermal-roots-and-continuum.md) | Independent lane-specific profiles and checks | no base-tree stage |

## Program invariants

- CPU serial reference before a runtime tree, parallelization or GPU work.
- Sparse rooted graph and section state are authority; visual mesh follows.
- No scalar tree HP, scripted fall threshold or visual fracture as gameplay
  fact.
- Private `f64` is accepted only inside a fixed step; complete
  future-affecting state crosses one fixed-point publication boundary.
- Wind is an immutable revision-bound forcing projection, not presentation or
  wall-clock noise.
- PhysX writes rigid state; the living solver writes structural state; graph
  split and detached-body creation publish together or neither does.
- Exact active persistence precedes modal/sleep persistence or forest
  streaming.
- LOD uses canonical simulation facts and manifest integer budgets, never the
  renderer camera or measured frame time.
- Public contracts appear only with the V4/V7 production consumer and an
  Accepted promotion decision.
- External source code, scans, material tests, large trajectories, captures
  and performance reports remain outside Git; bounded hashes/summaries may be
  checked in.

## Worktree and main-roadmap protocol

1. Close V0 in documentation before creating structural solver code.
2. Create a dedicated worktree from the documentation checkpoint and implement
   only V1. A `Pass` requires curves and roots, not compilation or a visual
   animation.
3. After V1 PASS, merge the evidence checkpoint and change the main R8 row from
   `PLANNED / NOT_ACTIVE` to an active integration track.
4. Each stage is a coherent commit boundary. A failed run records the first
   discriminator and leaves the stage open; thresholds are not tuned after the
   result.
5. If the V0 forest workload misses budget after two evidence-backed V6
   optimization cycles, stop as `RESEARCH_ONLY`. GPU authority, a smaller
   tree/branch gate or a larger budget requires a new explicit decision.

## Whole-program non-goals

Full-tree volumetric FEM, arbitrary fibre simulation, arbitrary-height or saw
cutting in the first vertical, flexible falling crowns, secondary fragment
fracture, tree-to-tree damage, contact leaves, growth, decay, snow/ice, fire,
embers, root failure, deformable soil, water exchange, general weather/CFD,
cross-region streaming and generic vegetation/solver plugin APIs.
