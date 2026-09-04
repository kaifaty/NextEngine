# PhysX water lane — floating boxes as colliders in the fluid (ADR-106 step 4)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R3` (research report, not a product check) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | ADR-106 (Accepted 1.0); plan 25 (emission and absorption); plan 24 (the increment list); plan 17 (`WaterFloatingBoxV1`) |
| Purpose | the exact model's floating boxes (the crate) exist in the presentation fluid as kinematic colliders driven by their committed poses, so poured or splashed water flows around and over the crate instead of through it; nothing flows back to the authority |

## Frozen scope

- **Bridge.** `ne_physx_fluid_set_box(fluid, slot, centre, half_extents)`
  (metres): up to `16` kinematic box actors in the fluid scene, created
  on first use with the fluid's material, moved with
  `setKinematicTarget` before the next step; `ne_physx_fluid_clear_box`
  removes one. Rust: `NativeFluid::set_box` / `clear_box`.
- **Lane.** On every published stage frame the lane sets one collider
  per `WaterPresentationFrameV1::boxes` entry (the box's plan and
  height from its bounds) whose plan centre lies inside the fluid box;
  a box that leaves the frame clears its slot. Pure
  `colliders_for_frame(frame, fluid_box) -> Vec<(centre, half)>`.
- **Statistics.** `LaneStats` gains `inside_colliders_max`: the largest
  count, over the frames, of readback particles whose position lies
  inside a collider box shrunk by one spacing (particles resting on the
  surface of a collider are not "inside").
- **Not in scope.** The avatar capsule, static scene geometry beyond the
  rim, two-way forces on the boxes (presentation never pushes the
  authority), rotation of the boxes (the exact boxes do not rotate).

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | `host-check`, `play`, `persistence-replay`, `water-present` PASS with unchanged roots and digest | pass |
| G2 collider rule | unit test: a frame with the crate yields one collider at the box's centre with its half extents; a box outside the fluid box yields none; an empty frame none | pass |
| G3 no penetration | a clean `--physx-water --physx-water-pour` session of `600` frames: `inside_colliders_max <= 2` percent of the peak particle count | pass |
| G4 cost | step plus readback plus upload mean `<= 4,000 us` per frame over that session | pass |
| G5 look | frame `100` of the pour session: the block's water parts around the crate and runs off it | pass |

The slot count, the shrink rule and the thresholds are frozen; a change
after the run is a new revision.

## Result (2026-09-04)

Implementation: `ne_physx_fluid_set_box` / `ne_physx_fluid_clear_box`
(`16` kinematic slots, `setKinematicTarget` before the next step),
`NativeFluid::set_box` / `clear_box`, `colliders_for_frame` and
`particles_inside` in the lane, `inside_colliders_max` in the
statistics.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `host-check`, `water-present` (digest `88655c7b…` unchanged), `play`, `persistence-replay` PASS | pass |
| G2 collider rule | unit test: the crate yields one collider at `[6.5, 0.55, 2.0]` with half extents `0.25 m`; a box `10 m` east yields none; an empty frame none; a particle at the crate's centre counts as inside, one at its face does not | pass |
| G3 no penetration | pour session of `600` frames (`controls=0`): peak `5,270` particles, `inside_colliders_max 0` | pass |
| G4 cost | `781 us` mean, `5,586 us` max per frame over the pour session | pass |
| G5 look | frame `100` of the pour session: the block's water bursts around the crate and over the ring; the crate stays visible inside the splash | pass |

Captures stay outside Git.
