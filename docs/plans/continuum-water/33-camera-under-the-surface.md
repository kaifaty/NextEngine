# The camera under the surface (plan 31 item 1, part B)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-UNDER-P1` (render increment, no root change) |
| Status | `RUN / G1-G5 PASS at revision 2` (2026-09-04; revision 1 failed G4 by reading) |
| Parent | plan 31 item 1; plan 32 (the pond that holds the camera); plan 13 (the water pass), plan 15 (the reflection), ADR-102 (the particle pass) |
| Purpose | with the camera below a water surface the frame today shows the pond walls through clear air and no surface at all (the ring is drawn from above only). The frame gains the two things a submerged eye sees: the water between the eye and everything else (fog and colour by the path through the water) and the surface from below (Snell's window with the scene above inside the critical cone, a silvery mirror beyond it) |

## Frozen scope

- **Submersion state (adapter, pure).** From the frame plan's camera
  position and the current water rings: the eye is submerged when its
  `x z` lie inside one ring's plan (the ring's update bounds plus its draw
  translation) and its `y` lies below that ring's level (the draw
  translation `y`). One function, unit-tested, evaluated once per frame;
  presentation-only, read by no root.
- **The water between (new suite `water_under`).** When submerged, a
  full-screen pass runs inside the water pass after the scene copy and
  before the rings: for every pixel the world point behind it (scene
  depth through the inverse view-projection) gives the path through the
  water from the eye, clipped at the level plane when the point lies
  above the level; the pixel becomes `scene · T + INSCATTER · (1 − T)`
  with `T = exp(−ABSORPTION · path)` (plan 13's absorption per metre)
  and the renderer-local `WATER_UNDER_INSCATTER = [0.05, 0.18, 0.28]`.
  The same sets as the ring draw (the frame block, the water set), the
  fullscreen triangle of the particle pass.
- **The surface from below (`water_scene.frag`).** The ring draws its
  back face when submerged (the water uniform gains a `submerged` lane).
  With the flipped normal: Snell's law for water to air (`1.333`);
  beyond the critical angle the pixel is the mirror (the in-scatter
  colour, the reflection of the water itself); inside it the scene above
  through the refracted offset, blended by Schlick's Fresnel of the
  refracted angle; the path from the eye to the surface point fogged by
  the same rule as the pass above. No second reflection pass (the
  mirrored underwater scene) in this increment.
- **The droplet layer.** The particle pass is skipped while submerged
  (the droplets belong to the air side).
- **Statistics.** `submerged_frames` in the adapter's outcome and on the
  session line.
- **Not in scope.** Light shafts, a mirrored underwater reflection, the
  audio low-pass (plan 31 item 2), any change to roots or the stage.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play` and `persistence-replay` roots and the `water-present` digest equal plan 32's recorded values; `host-check` PASS |
| G2 unit | the submersion state: inside the plan below the level → submerged; above the level, or outside the plan below it → not; the manifest carries the `water_under` suite and its module decodes |
| G3 cost | `--start-at-pond --maximum-frames 362 --capture-frame 360` at `960 × 540`: rendered frames per gameplay tick `≥ 5.0` |
| G4 look (human) | the capture at frame 60 from `--start-at-pond`: the surface from below with Snell's window (the sky and the far wall's top inside the cone, the silvery mirror beyond), the near wall and the avatar clear, the far end of the pond fogged; the `--start-at-water` capture keeps its look (the eye above the water, no fog) |
| G5 above water | the water-start session reports `submerged_frames = 0`; the pond session reports `submerged_frames ≥ 100` of 120 |

## Result (2026-09-04)

Implementation: `water_submersion_level` (pure, `gpu_content/water.rs`)
over the plan's water draws (the catalog mesh's `x z` bounds plus the draw
translation, the level from the translation); the `water_under` suite
(`fluid_screen.vert` + `water_under.frag`) as a second pipeline of the
water pass on the same layout, recorded after the scene copy and before
the rings; the `under` lane of the water uniform (level, submerged);
the back-face branch of `water_scene.frag`; the particle pass skipped
while submerged; `submerged_frames` in the adapter outcome and on the
session line.

### Revision 1 (plan 13's absorption on the underwater path)

| Gate | Reading | Verdict |
| --- | --- | --- |
| G4 look | `p33-pond-60-rev1.png`: the avatar 4 m away nearly dissolved in the fog, the walls readable only within a metre or two, the far end a flat blue — plan 13's absorption per metre (`1.2, 0.5, 0.25`, chosen for a look down into shallow water) loses red within a metre of path | **fail** |

### Revision 2 (a clear-pool absorption for the underwater path)

The underwater path (the water-between pass and the ring's back face)
takes its own renderer-local constant
`WATER_UNDER_ABSORPTION_PER_METRE = (0.45, 0.12, 0.06)`; plan 13's
constant stays for the look from above. Nothing else changed.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `play` `255dec16…`, `persistence-replay` `aba2347c…`, `water-present` `0abd6769…` (plan 32's tree; plan 32's record of the `persistence-replay` root is corrected there), `host-check` PASS | pass |
| G2 unit | `eye_inside_the_plan_below_the_level_is_submerged` (inside/below, above, outside, the basin case, no rings); the manifest test decodes `water_under` and pins both module hashes | pass |
| G3 cost | `--start-at-pond --maximum-frames 362 --capture-frame 360`: `362` frames over `68` ticks (`5.3` per tick) at `960 × 540`, `submerged_frames 357` | pass |
| G4 look (human) | `p33-pond-60.png`: the avatar 4 m away and the near walls clear with a blue cast, the far end of the pond fading into the in-scatter colour, the surface above a silvery-blue ceiling; the `--start-at-water` capture unchanged (the basin from above, the sunken pond visible at the right). Snell's window is not in this frame: at the start's `12°` pitch the frame's top edge sits about `48°` from the vertical, the critical angle itself; a player looking up sees it (no scripted look-up in this plan) | pass, the window unread |
| G5 above water | water start `submerged_frames 0` (of 120); pond start `115` of 120 | pass |

Observation: the first `5` frames of each session are not submerged (the
frame plan carries no water draws until the first stage publication),
which is why `115` and `357` rather than the full counts.
