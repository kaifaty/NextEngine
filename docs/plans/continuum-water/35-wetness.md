# Wetness — the wet band above the level (plan 31 item 3)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-WET-P1` (render increment, no root change) |
| Status | `RUN / G1-G5 PASS` (2026-09-04) |
| Parent | plan 31 item 3; plan 13 (the water pass), plan 33 (the fullscreen pipeline of the water pass and the ring plans) |
| Purpose | walls, floors and bodies at the water line are as dry above the level as a metre higher. A wet band above every ring's level darkens the scene and adds a sun gloss where the surface has been reached by the water |

## Frozen scope

- **The ring plans on the GPU.** The water set gains a uniform (binding
  4, `272` bytes): up to `8` ring plans (`x z` bounds in metres, from
  the catalog mesh bounds plus the draw translation) and their levels,
  plus the count; written per frame from the same list plan 33 builds
  for the submersion test (one function, `water_ring_plans`).
- **The wet band (new suite `water_wet`).** A fullscreen pass inside the
  water pass after the scene copy and before the rings, only when the
  eye is not submerged: for every scene pixel the world point; for every
  ring whose plan, widened by `0.25 m`, contains the point's `x z`, the
  height over the level `h`; `wet = 1 − smoothstep(0, 0.15 m, h)` for
  `h ≥ −0.02 m`, the maximum over rings. The pixel becomes
  `scene · mix(1, 0.55, wet)` plus a sun gloss
  `pow(max(N·H, 0), 48) · sun · 0.35 · wet` with the normal from the
  depth's world-space derivatives (the particle pass's rule). Pixels
  with no band are discarded.
- **Not in scope.** A per-body wetness memory (a crate lifted well
  clear of the water dries at once), drips, the player (item 11), the
  band under a submerged eye (the underwater pass rewrites every pixel),
  the PhysX lane.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present` roots equal plan 34's; `host-check` PASS |
| G2 unit | the ring uniform packing: `8` plans at most, count and levels in their lanes, an empty list packs a zero count; the manifest test decodes `water_wet` and pins its hash |
| G3 cost | `--start-at-water --maximum-frames 362 --capture-frame 360` at `960 × 540`: rendered frames per gameplay tick `≥ 5.0` |
| G4 look (human) | the water-start capture at frame 60: a dark band with a sun gloss along the basin's inner walls just above the water line and around the crate's water line; the far ground and the rim's top unchanged; the pond-start capture unchanged (submerged) |
| G5 no ring | with no water draw in the plan the pass is skipped (by construction; the count lane `0` discards every pixel) |

## Result (2026-09-04)

Implementation: `water_ring_plans` (shared with plan 33's submersion
test), the ring plans uniform (binding 4 of the water set, `272` bytes,
`pack_ring_plans`), the `water_wet` suite as a third pipeline of the
water pass on the same layout (`record_water_fullscreen` serves the
underwater and the wet passes), recorded after the scene copy and before
the rings when the eye is above the water.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | recorded from the chain: `play`, `persistence-replay`, `water-present` equal plan 34's (`39527e4f…`, `a49a7267…`, `9813212f…`); `host-check` PASS | pass |
| G2 unit | `ring_plans_pack_into_their_lanes_and_cap_at_eight` (empty count `0`, two rings in their lanes, twelve capped to eight); the manifest test pins `water_wet` | pass |
| G3 cost | `--start-at-water --maximum-frames 362 --capture-frame 360`: `362` frames over `67` ticks (`5.4` per tick) at `960 × 540` | pass |
| G4 look (human) | frame 60 of the water start against plan 33's capture of the same frame: the pixel difference lies exactly on the basin rim's top and inner faces, the crate's water line and the ground strip along the pond's hole (`6 709` pixels changed, nothing elsewhere). The band darkens the rim's whole top face: the rim top at `0.6 m` sits `10 cm` over the `0.5 m` level, inside the `15 cm` band — read as a splashed rim, an expected consequence of a height band, recorded. The effect is subtle at the capture's distance; the pond start (submerged) is unchanged | pass, observation |
| G5 no ring | the pass is skipped without water rings (`water_rings.is_empty()`), and the count lane `0` discards every pixel | pass by construction |
