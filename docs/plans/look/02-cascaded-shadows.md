# Look L2 — cascaded shadow maps with hardware PCF (scene look item 2)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L2` (research report, not a product check) |
| Status | `RUN / G1-G6 PASS / G7 HUMAN` (2026-09-05) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 2; plan [`01`](01-hdr-chain-and-physical-lighting.md) (the lighting block the cascades ride in); the B0 shadow map (`2048²`, one `32 m` box around the camera, 3x3 PCF with nearest taps) |
| Purpose | one `32 m` shadow box means nothing beyond `16 m` of the camera casts a shadow (the dam at the falls has none) and a `1.6 cm` texel with nearest taps gives hard, stepped edges. Three concentric cascades in a `2048²` array, linear compare taps (hardware 2x2 PCF) under a 3x3 kernel, a normal-offset receiver, so shadows cover the whole `60 m` scene with soft edges near the camera |

## Frozen scope

- **Cascades.** Three orthographic cascades centred on the camera's plan
  position, extents `12`, `36` and `108 m` (texels `0.6`, `1.8` and
  `5.3 cm`), each snapped to its texel as the single box is today; near,
  far and eye distance scale with the extent (`extent / 8`, `1.5 x extent`,
  `0.75 x extent`). The frame block's `shadow_view_projection` stays the
  first cascade's matrix; the lighting block grows by three matrices and
  the extents (`368 -> 576` bytes) so every lit program reads the
  cascades from set 0 binding 1.
- **Map.** The shadow map becomes a `2048²` depth array of three layers
  (one image; per-layer attachment views, one array view for sampling);
  the shadow pass renders the plan's casters once per layer, the cascade
  index in the spare lane of the draw push block (`material_params.w`);
  the compare sampler filters linearly.
- **Receiver.** `sun_visibility(position, normal, n_dot_l)`, shared by
  the world, reflection and water programs: the first cascade whose
  projection holds the point inside a `2 %` margin, the receiver moved
  along its normal by `1.5` texels of that cascade, a slope-scaled depth
  bias scaled by the cascade's depth range, a 3x3 kernel of linear
  compare taps (`4x4` effective); outside every cascade the sun is
  unshadowed.
- **Contract.** Set 2 binding 0 becomes a depth-compare sampler over a
  2D array (`sampler2DArrayShadow`); the interface string records it and
  the block size; the golden, `manifest.json` and the runtime check move
  together. As in plan 01 the contract hash rides the render content
  profile into every root: the roots move and are re-pinned (recorded).
- **Not in scope (recorded).** Cascade blending at the borders (a seam
  is accepted), contact shadows (need the depth in the world pass; item
  3's territory), PCSS.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots recorded (they move with the contract hash, as plan 01 found); creator-smoke refreshed; `host-check` PASS |
| G2 contract | golden, manifest and runtime check updated together; every recompiled module's hash pin refreshed; PROVENANCE records the suites |
| G3 unit | a cascade matrix maps a point moved by one cascade texel along the light's side axis to exactly `1 / 2048` of the map; the third cascade holds points `50 m` from the camera and the first does not; the CPU cascade selection mirrors the shader rule (points at `4`, `15`, `50 m` select cascades `0`, `1`, `2`; `200 m` none); the lighting block is `576` bytes with the cascades at the recorded offsets |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `SHADOW_CASCADES active layers=3` on stderr, water draws `8`, `frame_plan_explicit_invalidations = 0` |
| G5 cost | GPU frame mean at `960 x 540` after within `1.5 x` the before (`213 / 212 µs` at the spawn / falls; the casters draw three times) |
| G6 fallback | without a shadow map the no-shadow suite renders as today (`SHADOW_MAP_FALLBACK`; the existing decode test of the suite) |
| G7 look (human) | at the falls the dam and the works cast shadows on the ground; near the avatar the shadow edge is soft, not stepped |

## Result (2026-09-05)

Implementation: the shadow map as a three-layer `2048²` depth array with
per-layer attachment views and a linear compare sampler (`resources.rs`),
the shadow pass rendering the casters once per layer with the cascade in
the push block's spare lane (`shadow.rs`, `shadow_depth.vert`), the cascade
matrices (`12 / 36 / 108 m`, texel-snapped, near, far and eye scaled with
the extent) in the lighting block (`576` bytes, `pipeline.rs`, `sky.rs`),
`sun_visibility` shared by the world, reflection and water programs, the
interface contract at `b0.v4` (`dd4b71e3cd9e…`).

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | play `085d381942f4`, persistence-replay `d3878fa23b0c`, water-present `127ff9664de6`, audio-scene `0c4bc9f1149b` (moved with the contract hash, as frozen); creator-smoke refreshed (the project block and five probes); `host-check` PASS | pass |
| G2 contract | golden `dd4b71e3cd9e6a4f034c17fc63b8fbeb8169140d3002da33600472f29460d6dc` in the contracts test, `manifest.json` and the runtime check; `24` module pins refreshed; PROVENANCE records the suites | pass |
| G3 unit | `shadow_cascades_snap_to_texels_and_select_by_extent` (one texel along the side axis moves the projection by `2 / 2048` in NDC; `50 m` inside the third cascade and outside the first; `4 / 15 / 50 / 200 m` select `0 / 1 / 2 / none`), the lighting block's cascades at `368` and extents at `560` (`exposure_maps_middle_grey`) | pass |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `SHADOW_CASCADES active layers=3 extents=12/36/108`, `dynamic_surface_draws=8`, `frame_plan_explicit_invalidations=0` | pass |
| G5 cost | GPU frame mean before `213 / 212 µs`, after `256 / 253 / 253 µs` (spawn / lake / falls), p95 `274 / 271 / 271`: `1.20 x` | pass |
| G6 fallback | the no-shadow suite is untouched by the receiver change (it declares the grown block only) and still decodes in the shader test; the `SHADOW_MAP_FALLBACK` path is unchanged | pass |
| G7 look (human) | `l2-falls-60.png`, `l2-spawn-60.png` (outside Git): the dam and the stairs shade the ground at the falls, the avatar's shadow edge is soft; the cascade seams are not visible in these views | human |

Recorded as open: blending at the cascade borders, contact shadows (item
3, with the depth in hand), PCSS.
