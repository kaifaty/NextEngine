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
(`apps/game --capture-frame`) and unchanged gameplay roots:

| Step | Content | Gate sketch |
| --- | --- | --- |
| L1 surface material (done 2026-09-03, plan 12) | Fresnel-weighted reflection of the sky gradient, sun glint, ring normals over the catalog base colour, `water_surface` suite on `WaterSurface` rings | roots identical; capture shows angle-dependent shading |
| L2 shore fade and foam | depth difference between the surface and the scene depth buffer: soft edge at walls and the crate, foam band | roots identical; edge visible in capture |
| L3 refraction and depth colour | water drawn after the opaque pass; normal-offset scene sample; absorption by depth below the level (the ADR-102 constants) | roots identical; the crate's submerged half is tinted |
| L4 wave spectrum on the ring | two FFT cascades or summed Gerstner waves driven only by the exact flux and level (SPEC-38 practice 5); replaces the sine ripple, keeps the feed | roots identical; stage cost within the plan 09 budget |
| L5 planar reflection | mirrored scene pass per water plane (basin, vessels), bounded | one catalog/frame plan per run; cost row |
| L6 caustics | projected animated caustics on the basin floor and the crate, scaled by the level | roots identical |
| L7 crate wake and splash | ADR-102 particles from the exact immersion change; local ring depression under the crate | roots identical; capture |
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
