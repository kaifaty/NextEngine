# PhysX water presentation lane in the game (ADR-106 step 2)

| Field | Value |
| --- | --- |
| Research ID | `PHYSX-WATER-PRESENT-R2` (research report, not a product check) |
| Status | `DEMO (first cut, 2026-09-04) / lane increments to be frozen` |
| Parent | ADR-106 (Proposed); plan `continuum-water/23` (the probe and the GPU SDK profile); ADR-102 (the particle surface pass); task-state D-011 |
| Purpose | the PhysX PBD fluid drives the ADR-102 particle pass inside the game process, optional and fail-closed, so the user can see volumetric water on the exact level; the authority and every root stay untouched |

## What the first cut is

Built before this text was frozen, on the user's request to see the
water live; recorded here as a developer demo, not as a gated increment.

- **Bridge.** A persistent GPU fluid (`ne_physx_fluid_create` / `step` /
  `read` / `destroy`, `NativeFluid` in Rust): a box with a floor and four
  walls, open at the top, seeded on the spacing grid inside a block; the
  plan 23 probe runs on the same object. `gpu_library_path()` resolves
  the GPU library from `NEXTENGINE_PHYSX_GPU_LIBRARY`, the SDK directory
  or the `xtask physx setup` cache locator.
- **Game.** Feature `physx-water` of `apps/game` (off by default; needs
  the prepared SDK) and the flag `--physx-water`: the fluid's floor is the
  basin's exact initial level (`0.5 m`), its walls the rim's inner faces,
  a `1.0 x 0.6 x 1.0 m` block of water seeded `1 m` above the surface
  (`0.05 m` spacing, up to `16,384` particles); the fluid steps at `60
  Hz` from the frame clock (at most four steps per frame) and publishes
  positions and velocities to the particle pass every frame, replacing
  the stage's droplets while the demo runs; the run starts at the water
  view. Without the GPU library the run prints `PHYSX_WATER_FALLBACK`
  and continues with the stage's droplets.
- **Run.** `cargo run --release -p next_game --features physx-water --
  --interactive --physx-water` (after `cargo run -p xtask -- physx
  setup` on a host with the NVIDIA driver and a CUDA toolkit).
- **Captures (outside Git).** Frames `30`, `70`, `140` from the water
  start: the block falling, the impact on the surface, the basin full of
  splash droplets over the exact ring.

## What the lane still needs (to be frozen as increments)

1. Emission from the stage's edge records (jets, falls) and the boxes'
   immersion changes instead of a seeded block; absorption of particles
   that settle at the exact surface (the fluid is a transient layer on
   the level, never a second water body).
2. Boundaries from every cell's geometry and the committed body poses
   (the crate) so the fluid reacts to bodies visually.
3. The particle pass fed with neighbour counts and anisotropic kernels
   from the fluid, spray from its statistics; the ring hidden under a
   thick fluid layer or blended with it.
4. A run option surface (not only a build feature), the capability probe
   at start, device-loss handling, and bounded statistics reported
   through the adapter (particles, step cost, fallback reason).
5. ADR-106 acceptance on the probe and the demo evidence; the lane's
   research report gates (particle counts, bounds, cost per frame, the
   human look) frozen per increment.
