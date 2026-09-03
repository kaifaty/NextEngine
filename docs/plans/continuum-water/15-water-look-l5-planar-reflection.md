# Water look L5 — planar reflection of the scene

| Field | Value |
| --- | --- |
| Research ID | `WL5` |
| Status | `RUN / G1-G3 PASS / G4 HUMAN` (2026-09-03) |
| Parent | plan `continuum-water/11` item L5; plan 13 (the water pass that samples it); plan 14 (the normals that distort it) |
| Purpose | the crate, the avatar and the scenery reflect in the water: a mirrored scene pass into a screen-sized target that the water pass samples at the pixel's own position; the analytic sky stays where nothing reflects |

## Frozen scope

- **Reflection pass.** Before the opaque world pass, when the water pass
  exists and the plan has a camera: the frame plan's draws (static,
  skinned and `Opaque` rings; `WaterSurface` rings excluded) render with a
  mirrored camera into a screen-sized colour target (swapchain format,
  cleared to alpha `0`) with its own depth target. The mirror plane is the
  basin's level plane `y = h`: the view-projection becomes
  `P * V * R(h)` with `R` the reflection about the plane, the camera
  position is mirrored for the view-dependent terms, the shadow map and
  its matrix stay those of the true camera, and front-face culling
  replaces back-face culling for the mirrored winding.
- **Clip.** The `b0_reflect` suite (the B0 vertex program plus a
  fragment that discards below the plane; the plane height rides the
  spare `w` lane of the frame block's camera position, which only this
  suite reads) keeps geometry under the water out of the reflection.
- **Sampling.** The water pass gets set 3 binding 3: the reflection
  target. Because a point on the plane projects to the same pixel through
  the mirrored and the true camera, the water pixel samples the reflection
  at its own screen position, offset by the ring normal times `0.02`,
  and mixes it over the analytic reflected sky by the sample's alpha.
  The Fresnel, glint, refraction, absorption and shoreline of plans 12-14
  are unchanged.
- **Plane height.** Renderer-local: the translation `y` of the
  `WaterSurface` ring draw whose catalog mesh has the largest horizontal
  extent (the basin; the ring is authored at local `y = 0` and the
  presentation binding adds the exact level), read from the plan by the
  adapter (revision 2: "first ring" was a tie between three rings that all
  translate only in `y`, and picked vessel A at `1.5 m`). One plane per
  frame; the two vessels use the basin's plane (recorded limitation). A
  frame without a water ring draw skips the reflection pass and the water
  samples alpha `0`.
- **Cost.** One extra scene pass per frame with the same draw list.
- **Availability.** Requires the water pass; without it nothing changes.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | `host-check`, `water-present` unchanged; the presentation stage, checkpoint and frame plan untouched | pass |
| G2 one plan | capture counters: `frame_plan_explicit_invalidations = 0`, three water draws | pass |
| G3 frame cost | frames per tick in the `--start-at-water` release capture at `960 x 540` | `>= 4.0` (one extra scene pass) |
| G4 look (human) | the capture shows the crate and the avatar reflected in the basin, the sky where nothing reflects, no geometry from under the surface in the reflection | human |

Do not change the plane rule, the distortion offset or the clip after
seeing the captures; a change is a new revision with its own capture.

## Result (WL5, 2026-09-03)

| Gate | Result |
| --- | --- |
| G1 roots | the change lives in the desktop adapter (a pass, a suite, set 3 binding 3); `host-check` passes; the stage, checkpoint and frame plan are untouched — PASS |
| G2 one plan | release capture runs: `frame_plan_explicit_invalidations = 0`, three water draws through the water pass, `15` plan draws of which `10` indirect and `2` skinned re-recorded by the reflection pass (water rings skipped) — PASS |
| G3 frame cost | `122` frames over `23` ticks (`5.3` per tick) in the capture runs, `600` over `112` in the L4 run — PASS (`>= 4.0`) |
| G4 look (human) | from the start view at the basin's south edge (`wl5-side2.png`): the crate's mirror image under the crate, the sky reflected across the surface at the grazing angle, foam ring and waves; the avatar's reflection falls on the floor in front of the basin and not on the water from this camera — HUMAN |

Readings (recorded): the first plane rule ("first water ring draw")
tied between the three rings and selected vessel A's `1.5 m` plane, which
clipped every reflected object; the largest-extent rule fixes the basin.
The `--start-at-water` view moved from the west edge (camera behind the
avatar inside the relay arch at low pitch) to the south edge
(`[5.3, 0.9, 0.2] m`, yaw `0`, pitch `-12`), where the crate and its
reflection are in view. Diagnostics that painted a target through the
water quad's own UVs were invalid: the authored water quads carry a
constant UV, so they showed one texel; screen-position sampling and a
per-frame draw count were the valid probes.
