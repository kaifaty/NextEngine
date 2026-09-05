# Look L1 — HDR chain, analytic sky and physical lighting (scene look item 1)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L1` (research report, not a product check) |
| Status | `RUN / G2-G6 PASS / G1 FAIL-AS-FROZEN (roots moved with the contract hash, recorded) / G7 HUMAN` (2026-09-05, revision 1) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 1; D-L01 (engine systems first); SPEC-04, SPEC-30; the B0 shader interface contract (`crates/contracts/src/render_content/profile.rs`) |
| Purpose | the scene is shaded with Lambert plus a constant hemisphere, written straight to an sRGB swapchain with no exposure or tone map, under a gradient sky that ignores the sun. A 16-bit float scene target with an exposure and ACES tone-map pass, an analytic sky whose spherical-harmonic irradiance replaces the hemisphere constant and whose horizon gives the fog its colour, a sun in radiance units, and GGX specular with metallic and roughness read from the material record |

## Frozen scope

- **HDR chain.** When the G-buffer suite exists and the device offers
  `R16G16B16A16_SFLOAT` as a colour attachment, the scene target
  (`gbuffer.scene_color`), the water's scene copy and the reflection
  target take that format; the scene-to-swapchain copy becomes a
  fullscreen `tonemap` suite (exposure multiply, the ACES fitted curve,
  linear output into the sRGB swapchain, which encodes). Without the
  suite or the format the adapter keeps the current 8-bit path and
  prints `RENDER_HDR_FALLBACK <reason>` once. Captures read the
  swapchain as today (already tone-mapped).
- **Lighting block.** Set 0 gains binding 1, a `LightingUniforms` block
  (`std140`, 256 bytes): `inverse_view_projection` (64), `sun_radiance`
  (rgb, `w` = exposure), nine `sky_sh` coefficients (RGB irradiance
  SH2, 9 × 16), `fog` (rgb radiance at the horizon, `w` = density),
  `sky_params` (sun elevation, turbidity, ground albedo, reserved). The
  interface contract string gains
  `set0-binding1-uniform-buffer-vertex-fragment-min256` and the
  push-constant entry grows to `size96` with `material-params-f32x4`
  (metallic, roughness, emissive intensity, reserved); the golden SHA
  in the contracts tests, `manifest.json` and the runtime check move
  together and are recorded. The 208-byte frame block is unchanged.
- **Analytic sky.** The Preetham model (turbidity `2.5`) evaluated in a
  new `sky` fragment program from the view ray (the inverse
  view-projection), with a sun disc and the model's horizon; the same
  model, sampled on the CPU over the sphere (a fixed `64 x 32` grid,
  ground hemisphere at the ground albedo times the ground's irradiance
  from the sky and the sun; revision 1, see below),
  projected onto SH2 once per session (the sun is constant), fills
  `sky_sh` and the fog colour. Units: the model's zenith luminance is
  normalised so that the sky irradiance on an upward face is `1.0`;
  the sun's irradiance on a face normal to it is `5.0` (the daylight
  ratio of direct to diffuse); the exposure is the constant that maps a
  middle-grey (`0.18`) upward face in full sun to middle grey on the
  display through the tone curve (revision 1, see below).
- **Shading.** `b0_textured`, `b0_textured_no_shadow`, `b0_reflect`:
  diffuse `albedo (1 - metallic) / π` under the SH irradiance plus the
  sun, and Cook-Torrance GGX (`D` GGX, `V` Smith height-correlated,
  Schlick `F0 = mix(0.04, albedo, metallic)`) for the sun; roughness
  and metallic from the catalog's material record for the draw's
  material revision (the adapter builds the map; the frame plan is
  untouched). The water programs (`water_surface`, `water_scene`,
  `water_under`, `water_wet`) replace the hemisphere constant with the
  SH irradiance and the sun intensity with the sun radiance, and the
  analytic reflected sky of the water pass becomes the model evaluated
  along the reflection vector. The fluid (PhysX lane) composite keeps
  its own block and is scaled by the exposure only (recorded follow-up).
- **Not in scope (recorded).** Auto exposure (needs a readback), the
  Hosek-Wilkie tables, a time of day (the sun stays the B0 constant),
  local lights, the fluid lane's own lighting, any content change.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots equal plan 42's; `render-performance`'s frame plan hash unchanged; `host-check` PASS |
| G2 contract | the interface manifest golden updated in the same commit as `manifest.json`; the shader hash test with every recompiled module; PROVENANCE records the compiler line |
| G3 unit | SH2 projection of a constant sky yields a constant irradiance within `1 %`; the Preetham zenith luminance is positive for sun elevations `5..=90°`; the exposure constant maps middle grey in full sun to middle grey on the display (revision 1); the ACES fit is monotone on `0..=16` with `f(0) = 0`; the material map yields the catalog's metallic and roughness for every reference material |
| G4 render | release capture runs at the spawn (default), the lake and the falls, capture at frame `60`: `PASS`, `RENDER_HDR_CHAIN active` on stderr, water draws `8`, `frame_plan_explicit_invalidations = 0` |
| G5 cost | the mean GPU frame time from the adapter's timestamps at `960 x 540` in the spawn run, before and after, the after within `1.3 x` the before (both recorded) |
| G6 fallback | with the tonemap suite absent or the float format unsupported the frame renders through the current path (`create_hdr_chain` returns `None`, `RENDER_HDR_FALLBACK` once); unit level |
| G7 look (human) | the three captures before and after: the sky reads as a sky with a sun, shadows are not black, the checker no longer clips, the water keeps its glints without blowing out |

## Revision 1 (2026-09-05, before the gate run)

Two corrections of the model found on the development captures, recorded
before any gate reading: the SH lower hemisphere takes the ground's
irradiance from the sky and the sun (the frozen text had the sky only, which
left every face turned from the sun without its main fill); the exposure
rule keys on middle grey through the curve (`0.18` in full sun lands on
`0.18` displayed) instead of white before the curve at `0.8`, the usual
calibration and `0.6` of a stop brighter. The fog density becomes `0.006`
per metre (it was `0.035` with a dark blue colour; with the horizon's
radiance as the colour that density washed everything past `30 m` white).

## Order of work

A: the float target and the tonemap pass (G4, G5, G6). B: the lighting
block, the sky program, SH and fog (G2, G3). C: GGX with the material
map and the water programs on the block (G3, G7). Captures after each
step stay outside Git.

## Result (2026-09-05)

Implementation: the `R16G16B16A16_SFLOAT` scene target with the `tonemap`
suite (`gbuffer.rs`, `hdr.rs`), the scene format chosen once per swapchain
(`select_scene_format`, `RENDER_HDR_CHAIN` / `RENDER_HDR_FALLBACK`), the
`LightingUniforms` block at set 0 binding 1 written per frame from the
Preetham model (`sky.rs`: SH2 over the sky and the sun-lit ground, the
horizon fog colour, the middle-grey exposure, the inverse view-projection),
`sky_analytic` in place of `sky_gradient`, GGX with metallic and roughness
from the catalog's material records through the `96`-byte push block, the
water programs on the same block, the interface contract at `b0.v3`
(`dd7daad5544b…`). The game's closing report gains the GPU frame time
(`render timing:` line, a bounded timestamp buffer of `4 096` samples).

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS, but their roots moved: play `581414e9941d` (plan 42: `64ac5dac81d8`), persistence-replay `0e978ef5bb76` (`7fb9966adcbc`), water-present `c0bc658aa9b9` (`6fbf15e511fc`), audio-scene `236fe2eeb1cd` (`ed0da7bedc3a`). Cause: the B0 render content profile carries the shader interface manifest hash (`shader_interface_manifest_sha256`, `profile.rs`), the profile rides the project lock, and the lock rides every root; the same lock change moved the creator-smoke scenario's project hashes and its five probed roots (`smoke.scenario.json` refreshed from the run, as commit `8bfe54cf` did for WB1). The frame plan is untouched. `host-check` PASS after the refresh | **fail as frozen** (the clause assumed the contract hash stayed outside the roots); recorded, roots re-pinned |
| G2 contract | golden `dd7daad5544bdb0aeb28e4e468e52b0da290c6732caff164e592c0e371beabcd` in the contracts test, `manifest.json` and the runtime check; `24` module hash pins refreshed; PROVENANCE records the compiler line | pass |
| G3 unit | `sh_of_a_constant_sky_is_pi_times_its_radiance`, `zenith_luminance_is_positive_for_daylight_elevations`, `the_model_is_normalised_and_reads_as_a_clear_sky`, `exposure_maps_middle_grey` (revision 1), `aces_is_monotone_and_zero_at_zero`, `half_floats_decode`, `capture_conversion_tone_maps_and_encodes`, `matrix_inverse_round_trips`; the material map is read from the catalog in `PreparedContent::from_catalog` (its values reach the push block; `uniform_and_push_blocks_have_the_declared_layouts`) | pass |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `RENDER_HDR_CHAIN active target=R16G16B16A16_SFLOAT`, `dynamic_surface_draws=8`, `frame_plan_explicit_invalidations=0` | pass |
| G5 cost | GPU frame mean at `960 x 540`: before `195 µs` (p95 `212`), after `214 / 207 / 212 µs` (p95 `231 / 225 / 229`) at the spawn / lake / falls: `1.10 x` | pass |
| G6 fallback | `hdr_chain_falls_back_for_each_missing_precondition` (the tonemap suite, the offscreen target, the float format); the frame keeps the 8-bit path with the reason printed once | pass |
| G7 look (human) | `l1g-spawn-60.png`, `l1g-lake-60.png`, `l1g-falls-60.png` (outside Git) against `l1-before-*`: a sky with a sun and a bright horizon, soft sky-lit shadows, the sun-lit checker at middle grey; the scene reads dark overall (see the finding) | human |

**Finding (content, not the chain).** The reference materials were
authored against the old shading (`albedo × (0.5 + 0.95 cos θ)` with no
`1/π`), and their albedos are low: the floor's light checker squares
are about `0.08` linear, the works' walls about `0.12`. Under a calibrated
sun and sky they land where such albedos belong, at a fifth of middle
grey. The engine side is now correct; the brightness the user expects
comes from item 5's materials (real albedos in `0.2..0.6`), not from an
exposure bias, which was not applied.

Recorded as open: the Preetham horizon's magenta cast at low turbidity
(Hosek-Wilkie would fix it), the fluid lane's composite (its own block,
LDR values into the HDR target), emissive colour (the intensity lane
multiplies the base colour), auto exposure.
