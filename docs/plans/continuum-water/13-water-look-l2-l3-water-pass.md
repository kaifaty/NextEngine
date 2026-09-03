# Water look L2 + L3 — the water pass: shoreline, foam, refraction, depth colour

| Field | Value |
| --- | --- |
| Research ID | `WL2` |
| Status | `RUN (revision 2) / G1-G3, G5 PASS / G4 HUMAN` (2026-09-03) |
| Parent | plan `continuum-water/11` items L2 and L3; plan 12 (WL1); ADR-101 0.3; ADR-102 (the scene-copy technique of the particle pass) |
| Purpose | the water reads as a volume: the scene behind the surface is refracted and absorbed by the water depth, the surface fades softly into walls and the crate with a foam band, on top of the WL1 material; nothing in gameplay, the checkpoint or the frame plan changes |

## Frozen scope

- **Water pass.** `WaterSurface` rings leave the opaque world pass and
  draw in a dedicated pass after the sky and the opaque scene, before the
  particle pass and the UI: the opaque swapchain colour is copied to a
  sampled image (the ADR-102 scene-copy technique), the scene depth
  attachment is sampled read-only, the water is depth-tested against it
  without writing depth, no culling, no blending. The particle pass keeps
  copying the swapchain after the water, so the jet composites over it.
- **Inputs.** The water pipeline keeps the B0 sets 0-2 and the draw push
  constants and adds set 3: the scene colour copy (linear sampler), the
  scene depth (nearest sampler) and a `128`-byte water uniform: the
  inverse view-projection, the viewport size and its reciprocal, the
  absorption per metre `[1.2, 0.5, 0.25]` with the refraction strength
  `0.08` (the ADR-102 constants), the shore band `[0.12 m foam, 0.04 m
  fade, 0.85 foam grey]`.
- **Material (renderer-local constants).** Per pixel: the scene point
  behind the surface from the sampled depth through the inverse
  view-projection; the water path length `t` as the distance from the
  surface point to that scene point; the refracted scene colour sampled at
  `uv + normal.xz * 0.08 * min(t, 1 m)` unless that sample lies in front of
  the surface (then the unrefracted sample); the transmitted colour
  `scene * exp(-absorption * t)` mixed toward the WL1 lit water body by
  `1 - exp(-absorption * t)`; the shore fade `smoothstep(0, 0.04 m, d)`
  between the raw scene colour and the water; a foam band
  `1 - smoothstep(0, 0.12 m, d)` toward the foam grey, where `d` is the
  vertical water depth at the scene point (`surface.y - scene.y`;
  revision 2, see the result); then the WL1 Fresnel with the reflected
  sky, the sun glint and the fog.
- **Availability.** The pass exists when the swapchain carries
  transfer-source usage and the depth format supports sampling; otherwise
  the WL1 material draws inside the world pass as today and the adapter
  prints `WATER_PASS_FALLBACK` once. The depth attachment gains sampled
  usage when its format supports it.
- **Verification.** Desktop crate tests (manifest hashes, uniform layout);
  `host-check`; release capture runs of `apps/game` (`--maximum-frames
  362 --capture-frame 360`) with the counters of plan 12.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots | gameplay, checkpoint and frame-plan roots identical to WL1; `host-check` passes with unchanged pins | pass |
| G2 one plan | the capture run reports `frame_plan_explicit_invalidations = 0`, three dynamic surface draws, the particle pass available | pass |
| G3 cost | rendered frames per gameplay tick in the release capture run at `960 x 540` | `>= 5.0` |
| G4 look (human) | the capture shows the crate's submerged part tinted and refracted, the basin floor darkening with depth, a soft shoreline with a foam band along the basin walls and around the crate | human |
| G5 fallback | with the pass unavailable the frame renders through the WL1 path (unit-level: the pass is `None`, the world pass keeps the water draws) | pass |

Do not change the absorption, refraction strength or shore band constants
after seeing the captures; a change is a new revision with its own
capture.

## Result (WL2, 2026-09-03)

Revision 1 measured the shore band by the ray path length `t`; on the
first capture no band was visible because from a grazing view `t` at a
wall is already long. Revision 2 measures the shore band by the vertical
water depth at the scene point (absorption keeps `t`); the frozen widths
and colours are unchanged. Both revisions were captured with the same
command line.

| Gate | Result |
| --- | --- |
| G1 roots | the change lives in the desktop adapter (a pass, a suite, sampled depth usage); `host-check` passes with unchanged pins; the presentation stage, the checkpoint and the frame plan are untouched — PASS |
| G2 one plan | release capture runs at frames `120` and `360`: `frame_plan_explicit_invalidations = 0`, `dynamic_surface_draws = 3` (all three through the water pass), particle pass available — PASS |
| G3 cost | `362` rendered frames over `68` gameplay ticks (`5.3` frames per tick) at `960 x 540`, unchanged against WB1 and WL1 — PASS |
| G4 look (human) | the spawn-view capture (`wl2r2-120.png`, basin at the left) shows the floor checker refracted through the water with the blue tint of the depth absorption instead of the flat quad; the reference basin has no walls (a `0.5 m` slab on the open floor), so the shore band can only appear around the crate, which the spawn view does not reach; a walk to the basin with the capture flags is the human step — HUMAN |
| G5 fallback | `create_water_pass` returns `None` without transfer-source images, without a sampleable depth format or without a shadow map and prints `WATER_PASS_FALLBACK` once; the world pass then keeps the WL1 draws (`skip_water_surfaces = false`) — PASS by construction (unit tests cover the manifest and the inverse) |

Apparatus (recorded): the pass ends the world rendering instance, copies
the swapchain colour, moves the depth attachment to read-only, draws the
rings with depth test and no depth write, restores the depth layout and
reopens the rendering instance for the particle pass and the overlay; the
depth attachment now carries sampled usage where the format allows it.
