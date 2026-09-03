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
| L8 DLSS-ready outputs | motion vectors, thin G-buffer (albedo, normal + roughness, linear depth), jitter, HUD after the scene composite, per-group mask; no SDK | render tests; no vendor dependency in the workspace |

DLSS itself: revisit when NVIDIA documents DLSS 5 for native Linux
Vulkan, or when ray-traced water reflection/refraction is chosen (then
DLSS Ray Reconstruction on Linux through the `dlss_wgpu` pattern, as an
optional adapter feature behind an environment-provided SDK path).

## 2. Acceleration (recorded FAILs and known costs)

| Item | Reading | Frozen target | Planned change |
| --- | --- | --- | --- |
| WB1 G6 buoyancy batch | `260 us` per `64 x 64` batch in release (plan 08) | `<= 20 us` | intern the four exchange-tuple identifiers (one `SchemaId` set per world, not four strings per record); index volumes by plan rectangle so a body clips only against candidates; keep the same integer law and byte-identical records |
| WR1 box integrator | `~5.6 us` per box per tick, `116-121 us` mean for sixteen boxes (plan 10) | `<= 200 us` (met in steady state) | build the obstacle set once per substep and reuse it across boxes and the capsule sweep; optional |
| R8d flow step G6 | `183 us` at `64` cells / `256` edges (plan 07) | `<= 50 us` | in-place stepping without cloning the two maps; `i64` fast path when no `i128` is needed |
| Debug interactive path | `~17 fps` in a debug build (`481` frames over `851` ticks) against `~160 fps` in release | none (release is the supported run) | optional: memoize the presentation frame per published snapshot on the worker, avoid re-hashing the step input twice per tick |

Each item is a revision of its plan with the same gate and its own
evidence; no law, coefficient or record layout changes.

## 3. Scale practices (SPEC-38 2.2, in ADR-103 order)

| Practice | Content | Check |
| --- | --- | --- |
| Lattice tier | large water areas as a grid of cells with the same exact per-tick step and the same edge kinds; cell bound raised with a frozen budget | `CONTINUUM-WATER-LATTICE-P1` |
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
