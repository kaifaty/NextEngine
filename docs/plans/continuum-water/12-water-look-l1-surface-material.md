# Water look L1 — surface material on the dynamic ring

| Field | Value |
| --- | --- |
| Research ID | `WL1` |
| Status | `RUN / G1-G3 PASS / G4 HUMAN` (2026-09-03) |
| Parent | plan `continuum-water/11` item 1 (L1); ADR-101 dynamic surface ring; SPEC-04 desktop adapter; research `docs/development/water-look-dlss-research-2026-09-03.md` |
| Purpose | the three water quads stop reading as flat blue textures: a water material with Fresnel-weighted sky reflection, a sun specular lobe and the ring normals, drawn through a separate shader suite; nothing in gameplay, the checkpoint or the frame plan changes |

## Frozen scope

- **Profile.** `DynamicSurfaceProfileV1` gains `shading:
  DynamicSurfaceShadingV1` (`Opaque` = today's B0 world suite,
  `WaterSurface` = the new suite). The game declares its three water
  rings as `WaterSurface`; every other ring stays `Opaque`.
- **Suite.** `water_surface` (vertex: the B0 vertex program; fragment:
  water material) compiled offline with the pinned Linux `glslang`
  15.1.0 of the fluid suite, hashes pinned in `manifest.json`, provenance
  recorded. Same pipeline layout as the world suite (frame, texture,
  shadow sets and the draw push constants); fixed state: no blend, depth
  test and write on, no culling (the surface is visible from below).
- **Material (renderer-local constants).** Schlick Fresnel with
  `F0 = 0.02` on the ring normal against the view vector; reflected sky
  from the sky-gradient colours by the reflected direction's elevation;
  Blinn-Phong sun specular (exponent `240`) scaled by the frame sun
  intensity and the sampled shadow; the water body is the catalog base
  colour lit by the hemisphere ambient and `0.6` of the shadowed diffuse;
  the same world-distance fog as B0. Alpha stays `1`.
- **Draw path.** A ring whose profile is `WaterSurface` binds the water
  pipeline for its draw and the world pipeline is rebound after it; the
  shadow pass is unchanged (the crate still shadows the water).
- **Verification.** Desktop crate tests (profile shading round trip,
  manifest hashes, pipeline suite construction where a device exists);
  `play`, `persistence-replay`, `water-present`, `visual_presentation`
  and `host-check` unchanged; a capture run of `apps/game` in release
  (`--maximum-frames 362 --capture-frame 360`).

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | gameplay, checkpoint and frame-plan roots identical to WB1: `play`, `persistence-replay`, `water-present` and the presentation tests pass with unchanged pins | pass |
| G2 one plan | the capture run reports `frame_plan_explicit_invalidations = 0`, three dynamic surface draws in the last frame, the particle pass available | pass |
| G3 cost | rendered frames per gameplay tick in the release capture run not below the WB1 release run (`6338 / 1195 = 5.3`) at `960 x 540` | `>= 5.0` |
| G4 look (human) | the capture shows the basin surface shading changing with the view angle and a sun glint; the crate's shadow still falls on the water | human |

Do not change the Fresnel `F0`, the specular exponent or the diffuse
weight after seeing the captures; a change is a new revision with its own
capture.

## Result (WL1, 2026-09-03)

| Gate | Result |
| --- | --- |
| G1 roots | the change lives in the desktop adapter and the game's ring profiles; `host-check` (workspace tests, clippy, boundary scan) passes with unchanged pins; the presentation stage, the checkpoint and the frame plan are untouched — PASS |
| G2 one plan | release capture runs (`--maximum-frames 362 --capture-frame 360`): `frame_plan_explicit_invalidations = 0`, `dynamic_surface_draws = 3` in the last frame, `particle_surface_available = true` — PASS |
| G3 cost | `362` rendered frames over `68` gameplay ticks (`5.3` frames per tick) at `960 x 540`, unchanged against the WB1 release run — PASS |
| G4 look (human) | captures `wl1-real-120.png` (spawn view, basin at the left) show the surface lit by the reflected sky with the view-angle Fresnel; the sun glint depends on the camera; the crate's shadow still falls on the water — HUMAN |

Apparatus (recorded): a magenta-fragment experiment confirmed that the
three water rings and only they draw through the water suite. One capture
of the very first release run showed only the sky, the avatar and the
carried box (no floor, no scene); five later captures at the same frame
index rendered the full scene with the same command line, so the
observation is recorded as unexplained and unreproduced; watch for it in
later revisions.
