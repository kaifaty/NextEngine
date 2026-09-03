# Water: next actions after the V1 ladder

| Field | Value |
| --- | --- |
| Status | `PLAN / ORDERED` (2026-09-03) |
| Parent | ADR-104 (water V1 ladder complete: `CONTINUUM-WATER-VOLUME/FLOW/PRESENT/BUOYANCY-P1 = PASS`), task-state `docs/development/task-state/water-volume-authority.md` |
| Purpose | the ordered list of what follows the promotion of the exact water authority; each item becomes its own frozen plan with gates before it runs |

Decisions recorded 2026-09-03: gameplay mechanics for water (player-driven
gates and pumps) and player buoyancy are deferred; the water look comes
first; acceleration of the two recorded cost FAILs and the scale practices
are planned here.

## 1. Water look (first)

Research: `docs/development/water-look-dlss-research-2026-09-03.md`.
Conclusion: DLSS 5 re-lights an already rendered frame, is RTX 50 and
Windows only today and has no documented behaviour on water; the look is
built with vendor-neutral Vulkan techniques on the existing ADR-101 ring
and ADR-102 particle pass, while emitting the buffers DLSS would need.

Increments, each with a capture gate on the reference host
(`apps/game --capture-frame`) and unchanged gameplay roots. Since
2026-09-03 `apps/game --interactive --start-at-water` starts a fresh
session at the basin's south edge (`ReferenceSpawnOverrideV1::at_water()`
through `LaunchRequestV1::spawn_override`: the capsule at `[5.3, 0.9,
0.2] m`, the camera behind it looking along `+z` across the water, pitch
`-12`),
and every capture below starts from that view. The basin carries an
authored rim since plan 16 (2026-09-03): a static body with five box
walls and a compound mesh, one opening on the south side. The adapter also keeps a
scripted-input facility (`DesktopRunOptions::scripted_input`, pushed
through SDL's own event queue) for diagnostics that need real input:

| Step | Content | Gate sketch |
| --- | --- | --- |
| L1 surface material (done 2026-09-03, plan 12) | Fresnel-weighted reflection of the sky gradient, sun glint, ring normals over the catalog base colour, `water_surface` suite on `WaterSurface` rings | roots identical; capture shows angle-dependent shading |
| L2 shore fade and foam (done 2026-09-03, plan 13) | vertical water depth at the scene point from the sampled scene depth: soft edge and foam band at walls and the crate | roots identical; edge visible in capture |
| L3 refraction and depth colour (done 2026-09-03, plan 13) | the water pass after the opaque scene: scene colour copy, normal-offset refraction, absorption by the ray path length (the ADR-102 constants) | roots identical; the crate's submerged half is tinted |
| L4 wave spectrum on the ring (done 2026-09-03, plan 14) | four world-space directional waves with deep-water periods plus the flux ripple, `20 mm` cap, animated detail normal in the water pass | roots identical; stage cost `104 us` |
| L5 planar reflection (done 2026-09-03, plan 15) | one mirrored scene pass about the largest water surface's level into a screen-sized target, sampled by the water pass at the pixel's own position over the analytic sky; front-face culling and a plane clip | one catalog/frame plan per run; `5.3` frames per tick |
| L6 caustics (done 2026-09-03, plan 17) | animated caustic light on the scene point under the surface, computed in the water pass and fading with the path length | roots identical |
| L7 crate wake and splash (done 2026-09-03, plan 17) | ring depression around every floating box; splash droplets from the committed vertical speed through the ADR-102 particle lane | roots identical; capture |
| L8 DLSS-ready outputs (done 2026-09-03, plan 18) | HUD-less scene target copied to the swapchain before the UI overlay; a separate `gbuffer` suite (albedo + group mask, normal + roughness, screen motion vectors from a per-draw model history, linear depth); optional Halton(2, 3) projection jitter (`--projection-jitter`); `--capture-buffer` and `--capture-frames` for evidence; no SDK | frame plan hash, `interface_contract_sha256` and roots unchanged; CPU tests; decoded captures |

DLSS itself: revisit when NVIDIA documents DLSS 5 for native Linux
Vulkan, or when ray-traced water reflection/refraction is chosen (then
DLSS Ray Reconstruction on Linux through the `dlss_wgpu` pattern, as an
optional adapter feature behind an environment-provided SDK path).

## 2. Acceleration (recorded FAILs and known costs)

| Item | Reading | Frozen target | Planned change |
| --- | --- | --- | --- |
| WB1 G6 buoyancy batch (done 2026-09-03, plan 08 revisions 2 and 3) | `260 us` per `64 x 64` batch in release (plan 08) | `<= 20 us` | identifiers validated once per batch, levels once per volume, a plan-rectangle reject before the exact clip (revision 2: `26-29 us` max, `19 us` mean); text identifiers as shared `Arc<str>` (revision 3: `18-25 us` max, `13 us` mean); records byte-identical |
| WR1 box integrator | `~5.6 us` per box per tick, `116-121 us` mean for sixteen boxes (plan 10) | `<= 200 us` (met in steady state) | build the obstacle set once per substep and reuse it across boxes and the capsule sweep; optional |
| R8d flow step G6 (done 2026-09-03, plan 07 revision 2) | `183 us` at `64` cells / `256` edges (plan 07) | `<= 50 us` | `step_in_place` through the backend trait (no clones of the two maps), the standard integer square root, edge states walked in lockstep: `41-46 us` max, `28-29 us` mean; fluxes and levels identical |
| Debug interactive path | `~17 fps` in a debug build (`481` frames over `851` ticks) against `~160 fps` in release | none (release is the supported run) | optional: memoize the presentation frame per published snapshot on the worker, avoid re-hashing the step input twice per tick |

Each item is a revision of its plan with the same gate and its own
evidence; no law, coefficient or record layout changes.

## 3. Scale practices (SPEC-38 2.2, in ADR-103 order)

| Practice | Content | Check |
| --- | --- | --- |
| Lattice tier (done 2026-09-03, plan 19) | `WaterLatticeRegionV1` builds face-sharing cells and open sills inside the existing bounds (`64` cells, `256` edges); the overlap rule became positive-measure (SPEC-38 3.1); the bounds stay until a region needs more | `CONTINUUM-WATER-LATTICE-P1` PASS (`xtask water-lattice`) |
| Activity stepping | cells whose levels and fluxes are settled sleep; a command, a neighbour change or a body wakes them; roots unchanged versus the always-stepped run | same check, sleeping fraction in the report |
| Edge-driven presentation at scale | one presentation record per active edge, not per cell | plan 09 revision |
| Rotational presentation | whirlpools and eddies as presentation-only layers driven by edge flux | ADR-102 increment |

## 4. Deferred (decided 2026-09-03, not scheduled)

- Player-driven gates and pumps through the interaction system (the
  first "communicating vessels" gameplay).
- Player buoyancy and swimming speed (ADR-105 named it a later consumer).
- Promotion housekeeping that remains after the ADR acceptance: the
  remaining terrain clauses of SPEC-38 stay `Proposed` under their own
  heading until a terrain consumer exists.
