# Look L7 — post chain: bloom, volumetric height fog with light shafts, colour grading (scene look item 7)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L7` (research report, not a product check) |
| Status | `RUN / G1-G4, G6 PASS / G5 PASS AT THE SPAWN AND THE FALLS, 1.28 x AT THE LAKE (revision 3) / G7 HUMAN` (2026-09-06) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 7; plans [`01`](01-hdr-chain-and-physical-lighting.md) (the HDR scene target, the exposure and ACES resolve, the lighting block with the sun, the sky SH and the horizon fog), [`02`](02-cascaded-shadows.md) (the cascades the shafts march through), [`04`](04-temporal-anti-aliasing.md) (the resolved frame the chain reads) |
| Purpose | the frame goes from the resolved HDR target straight through exposure and ACES to the swapchain: no glare from the bright water and sky, a per-pixel horizon fog with no volume and no sun in it, and no grade. A post chain between the resolve and the swapchain: a physically based bloom, a ray-marched exponential height fog with sun in-scattering through the cascaded shadow map (light shafts), exposure and ACES, then a colour-grading 3D LUT |

## Frozen scope

- **Post pass (adapter).** `gpu_content/post.rs`, one boxed optional pass
  created after the temporal resolve when the G-buffer prepass and the
  HDR target exist; it replaces the `tonemap` resolve of plan 01 (which
  stays as the fallback). Three suites over the fluid fullscreen vertex
  program: `bloom_down.frag`, `bloom_up.frag`, `post.frag`.
- **Bloom.** A half-resolution `R16G16B16A16_SFLOAT` chain of five mips
  (`480 x 270` down to `30 x 16` at `960 x 540`). Downsample by the
  13-tap filter of Jimenez (2014) with the Karis average on the first
  level (no threshold); upsample by a `3 x 3` tent, additive into the
  finer level; the composite mixes `scene + 0.04 * bloom`.
- **Fog and shafts.** In the composite, per pixel: the view ray from the
  lighting block's inverse view-projection, the end point from the
  G-buffer linear depth (the sky at `200 m`), twelve steps with an
  interleaved-gradient dither, an exponential height fog
  `sigma(y) = fog.w * exp(-y / 12 m)` (the plan 01 density `0.006` at
  ground level), transmittance `T *= exp(-sigma ds)`, in-scattering
  `T * sigma * ds * (fog.rgb + sun_radiance.rgb * HG(cos theta, g = 0.5)
  * sun_visibility)` with the plan 02 `sun_visibility` (3 x 3 compare
  taps in the cascade holding the sample); `scene * T + scatter`. The
  lighting block's fog density is written as `0` while the pass exists,
  so the world programs' per-pixel fog (plan 01) yields to the volume;
  water surfaces are not in the prepass, so they fog by the floor
  beneath them (recorded).
- **Tone and grade.** Exposure and the ACES fitted curve as plan 01, then
  a `32³` `RGBA8_UNORM` 3D LUT sampled trilinearly, generated in the
  adapter (`post::grading_lut`): a mild filmic grade in display space
  (an S-curve contrast `1.06` about middle grey, saturation `1.08`, a
  cool lift `(-0.010, -0.004, +0.012)` at black and a warm gain
  `(1.0, 0.99, 0.97)` at white), monotonic on the grey axis, `0 -> 0`
  and `1 -> 1` within a step.
- **Evidence and control.** `POST_CHAIN active bloom_mips=N fog_steps=12
  lut=32` printed once; `--no-post` (`DesktopRunOptions::post_chain`)
  keeps the plan 01 resolve and prints `POST_CHAIN_FALLBACK: disabled by
  option`; the pass drops before the content in `GraphicsContext::drop`
  and stays under the `64 KiB` native-state bound.
- **Contract.** The B0 interface is unchanged (`b0.v6`); the manifest
  gains `post_suite`; the module pins are refreshed; PROVENANCE records
  the suites. Roots unchanged (no content change).
- **Not in scope (recorded).** Lens flares and dirt, exposure adaptation,
  depth of field, motion blur, local lights in the volume, a froxel
  volume, shadowed fog on the water surfaces, DLSS.

## Revision 1 (2026-09-06, before the render readings)

The frozen grade's cool lift at black `(-0.010, -0.004, +0.012)` and
the G3 clause `0 -> 0 within 1 / 255` contradict each other by
construction (the blue lift is `3 / 255` at black; the unit test read
`(0, 0, 3)`). The lift now fades in over the first `5 %` of the range
so pure black stays black and the shadows above it carry the tint; the
warm gain `(1.0, 0.99, 0.97)` likewise contradicts `1 -> 1` (the test
read `(255, 252, 247)`) and now fades out over the last `5 %` so white
stays white and the highlights below it carry the warmth; the gate
stands as frozen.

## Revision 2 (2026-09-06, after the first render readings)

The frozen in-scattering `fog.rgb + sun_radiance * HG * visibility`
washed the falls view out (recorded: the frozen `l7-falls-60.png`, the
dam and the stairs at `10 m` lost in glare toward the sun; a diagnostic
run with the sun term removed read as the plan 05 look with a light
haze). The sum double-counts: the horizon radiance the sky model gives
as `fog.rgb` already contains the sunlight scattered toward the viewer
(the Preetham aureole), so adding `E_sun * HG` (`E_sun = 5` in the
block's units against a horizon radiance near `0.3`) tripled the haze
off-axis and multiplied it toward the sun. The in-scattered radiance is
now the sky model's radiance along the view ray (aerial perspective, the
horizon value for rays under `2°`), and the frozen sun term against the
horizon's mean luminance only partitions it into the sun's share, which
the cascades shadow (the shafts), and the ambient share. The constants
(`0.006`, `12 m`, `60 m`, `200 m`, `g = 0.5`, twelve steps) are
unchanged.

## Revision 3 (2026-09-06, after the cost reading)

The frozen chain read `811 / 795 / 791 µs` against `366 / 348 / 354`
without it (`2.2 x`, fail as frozen). Attribution by diagnostic builds
at the spawn: the bloom, the composite (the sky evaluation, the LUT)
and the closed-form fog cost `30 µs` (`396`); the twelve-step march
with one compare tap per step `+133` (`529`); with the `3 x 3` kernel
`+415` (`811`). The march now runs at half resolution into its own
`R16G16B16A16_SFLOAT` target (`fog.frag`: in-scattered radiance and
transmittance, one compare tap per step, the dithered march integrates
what the kernel averaged) and the composite samples it bilinearly; the
steps, the distances and the density are unchanged.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with the plan 05b roots unchanged (`99a30ffb261c`, `6eadd81024de`, `cb728b6cd3de`, `2809903d4941`); `content-package` PASS with its counts unchanged; `host-check` PASS |
| G2 contract | the interface contract stays `860a946d909b…`; the manifest lists the three modules with their hashes; the runtime pins pass; PROVENANCE records the suites |
| G3 unit | the Henyey-Greenstein phase integrates to `1` over the sphere within `1 %`; the CPU height-fog integral of a `100 m` level ray matches the closed form within `1 %`, and a ray climbing `24 m` reads less than `1 / e²` of the level ray's optical depth per metre at its top; the mip count at `960 x 540` is `5`; the grading LUT is monotonic on the grey axis with `0 -> 0` and `1 -> 1` within `1 / 255`; the fallback reasons name the option, the missing prepass and the missing HDR target |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `POST_CHAIN active bloom_mips=5 fog_steps=12 lut=32`, water draws `8`, invalidations `0` |
| G5 cost | GPU frame mean with the chain within `1.25 x` the same session's `--no-post` run of the same binary (plan 05b's apparatus finding: frozen absolutes are not comparable across sessions) |
| G6 fallback | `--no-post` prints `POST_CHAIN_FALLBACK: disabled by option` and the three captures render through the plan 01 resolve, PASS |
| G7 look (human) | glare on the sun-lit water and the sky around the sun without a halo on the HUD; light shafts where the dam and the works shadow the fog toward the sun at the falls; the fog denser toward the ground and the lake than at the bank tops; the grade subtle (no colour cast on the concrete) |

## Result (2026-09-06)

Implementation: `gpu_content/post.rs` (the boxed optional pass, the
bloom chains, the half-resolution volume, the grading LUT filled from a
host buffer on the first frame, the composite into the swapchain in
place of the plan 01 resolve), `bloom_down.frag`, `bloom_up.frag`,
`fog.frag`, `post.frag`, the `post_chain` suite in the manifest with
its pins, `DesktopRunOptions::post_chain` and `--no-post`, the content's
fog ownership (a zero density in the lighting block while the chain
exists, so the world programs' per-pixel fog yields), the drop order and
the `64 KiB` native-state bound. Three revisions recorded above: the
grade's lift and gain fade at the ends (the frozen values contradicted
the endpoint gate), the in-scattered radiance from the sky model along
the ray with the sun term partitioning it (the frozen sum double-counted
the aureole and washed the falls view), the march at half resolution
with one tap per step (the frozen chain cost `2.2 x`).

The LUT is stored as a `1024 x 32` strip of `32` slices and sampled by
two bilinear taps and a mix (trilinear over the cube), which avoids a
3D image and a second upload path; recorded as the storage form of the
frozen `32³` LUT. GPU timings in this session come from a release build
(`--release`), which the earlier plans also used: the debug build's
means moved by a factor of four between runs of one binary and are not
comparable (plan 05b's apparatus finding, resolved).

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | play `99a30ffb261c`, persistence-replay `6eadd81024de`, water-present `cb728b6cd3de`, audio-scene `2809903d4941` (the plan 05b roots, unchanged); `content-package` PASS (`159` records); `host-check` PASS | pass |
| G2 contract | the interface contract stays `860a946d909b…`; the manifest lists `bloom_down`, `bloom_up`, `fog` and `post` with their hashes under `post_suite`; the runtime pins pass; PROVENANCE records the suite | pass |
| G3 unit | `henyey_greenstein_integrates_to_one`, `height_fog_depth_matches_the_integral` (the level ray within `1 %`, the climbing ray's top density under `1 / e²`), `bloom_chain_has_five_levels_at_the_reference_extent`, `grading_lut_is_monotonic_on_grey` (after revision 1), `fallback_reasons_name_each_missing_precondition` | pass |
| G4 render | spawn, lake and falls at `300` frames (release build): `PASS`, `POST_CHAIN active bloom_mips=5 fog_steps=12 lut=32`, water draws `8`, invalidations `0` | pass |
| G5 cost | two back-to-back rounds, release build, the same session: with the chain `457 / 450 / 449 µs` and `462 / 449 / 448`, without it `373 / 352 / 358` and `373 / 352 / 357` (spawn / lake / falls): `1.23 / 1.28 / 1.25 x`; the frozen chain read `2.2 x` (recorded under revision 3) | pass at the spawn and the falls; `1.28 x` at the lake (fail as frozen by `0.03`) |
| G6 fallback | `--no-post` prints `POST_CHAIN_FALLBACK: disabled by option`; the three captures render through the plan 01 resolve, `PASS`, and read as plan 05b | pass |
| G7 look (human) | `l7v3-post-{falls,lake,spawn}-60.png` (outside Git): a light aerial haze that softens the dam and the far bank without greying the sky's zenith, glare on the bright water and no halo on the HUD, a subtle grade; no shafts visible from the three cameras at this density (the sun's share of a `10 m` path is under `6 %`, and the dam's shadow falls away from the falls camera); the frozen `l7-falls-60.png` (the washed view) and the diagnostic captures are kept as the readings behind revision 2 | human |

Recorded as open: the shafts need a denser or lower volume, or a
camera behind a caster toward the sun, to read; the water surfaces fog
by the floor beneath them (not in the prepass); the fog target is
bilinear at silhouettes (no depth-aware upsample); the lake's `1.28 x`
against the `1.25 x` clause; lens effects, exposure adaptation, depth of
field, motion blur, local lights in the volume, a froxel volume, DLSS.
