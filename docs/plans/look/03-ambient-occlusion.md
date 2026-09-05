# Look L3 — ground-truth ambient occlusion from the G-buffer prepass (scene look item 3)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L3` (research report, not a product check) |
| Status | `RUN, REVISION 1 / G1-G3, G5, G6 PASS / G4 METRIC FAIL AS FROZEN (4.9 %) / G7 HUMAN` (2026-09-05) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 3; plans [`01`](01-hdr-chain-and-physical-lighting.md) (the SH ambient the occlusion scales), [`02`](02-cascaded-shadows.md) (the set 2 layout it extends); plan `continuum-water/18` (the G-buffer whose normals and linear depth it reads) |
| Purpose | boxes meet the ground without any darkening at the contact, so they float visually; the sky's SH irradiance reaches every crevice at full strength. Horizon-based ambient occlusion (GTAO) computed from the G-buffer's linear depth and normals, blurred depth-aware, and multiplied into the SH term of the world material |

## Frozen scope

- **Prepass.** The G-buffer pass moves before the reflection and world
  passes (it clears its own depth); its normal and linear-depth targets
  become sampleable. The world pass still clears and redraws depth.
- **Occlusion pass.** A fullscreen program over the G-buffer: for each
  pixel the world position from the linear depth and the lighting
  block's inverse view-projection, the normal from the G-buffer, three
  slices with five steps per side inside a `1 m` world radius (the
  screen radius from the pixel's angular size, clamped to `2..=64`
  pixels), interleaved-gradient noise per pixel, the GTAO slice
  integral (the projected normal's angle, the horizons clamped to its
  hemisphere), written to an `R8` target; then a depth-aware separable
  5-tap blur (two passes). The sky (no depth) reads `1`.
- **Consumption.** Set 2 gains binding 1, a colour sampler over the
  occlusion target (a `1 x 1` white image until the pass exists and
  when it does not); `b0_textured` multiplies its SH irradiance by the
  sampled occlusion. The reflection and water programs do not read it.
- **Evidence.** `--capture-source ao` captures the final occlusion
  target as grey. The adapter prints `AMBIENT_OCCLUSION active
  radius=1.0m slices=3 steps=5` once, or `AMBIENT_OCCLUSION_FALLBACK
  <reason>` (no G-buffer, no sampleable `R8`).
- **Contract.** `b0.v5`: `set2-binding1-combined-image-sampler-fragment-count1`.
  The roots move with it and are re-pinned (recorded, as plans 01-02).
- **Not in scope (recorded).** Temporal accumulation of the occlusion
  (item 4's TAA will filter it), bent normals, specular occlusion,
  contact shadows.

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots recorded; creator-smoke refreshed; `host-check` PASS |
| G2 contract | golden, manifest and runtime check updated together; module pins refreshed; PROVENANCE records the suites |
| G3 unit | the slice integral mirror gives `1.0` for unoccluded horizons at a normal along the view and less for horizons inside `±45°`, monotone in the horizon; the fallback reasons (no G-buffer, no `R8`) |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `AMBIENT_OCCLUSION active`, water draws `8`, invalidations `0`; an `ao` capture at the spawn: the mean occlusion in a `40 px` band along the base of the gate's pad is at least `10 %` below the mean over open ground |
| G5 cost | GPU frame mean after within `1.3 x` the before (`256 / 253 / 253 µs`) |
| G6 fallback | without the pass the world material samples the white image (unit: the fallback reasons; the binding is written at content creation) |
| G7 look (human) | the gate's pad, the vessels and the crate meet the ground with a darkening at the contact; open ground stays unchanged |

## Reading of the frozen scope (2026-09-05)

Full-resolution occlusion (three slices, five steps per side, two blur
passes at `960 x 540`): spawn, lake and falls `PASS` with
`AMBIENT_OCCLUSION active radius=1.0m slices=3 steps=5`, water draws `8`;
GPU frame mean `441 / 402 / 421 µs` against `256 / 253 / 253` before:
`1.7 x`. **G5 fails as frozen** (`1.3 x`); the reading is recorded and
the algorithm was not changed.

## Revision 1 (2026-09-05, after the G5 reading)

The occlusion and its blur run at half resolution (`480 x 270` targets;
the world material samples the result through its clip position, which
the linear sampler upscales), the standard placement of this pass. The
gates are re-run in full and both readings stay in this record.

## Result of revision 1 (2026-09-05)

Implementation: `ao.rs` (the pass over the prepass's linear depth and
normals, `R8` targets at half resolution, a linear sampler, one pipeline
layout for the occlusion and blur programs), `ao.frag` and
`ao_blur.frag`, the G-buffer prepass moved before the reflection and
world passes (it clears its own depth), set 2 binding 1 with a `1 x 1`
white placeholder (`WhiteTexture`), `b0_textured` sampling the occlusion
through its clip position, `--capture-buffer ao`, the contract at
`b0.v5` (`a269a93095e5…`). Two corrections during development: the
occlusion pass is dropped before the passes it reads (a teardown
segfault) and it is boxed (the native state bound of `64 KiB`); the
per-slice clamp of the integral was removed (an open plane read `0.75`;
the projected-normal weight makes the slice sum one only in the average,
now covered by `open_hemisphere_visibility`).

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | play `69c970d7df92`, water-present `d65863602dc1`, audio-scene `ee401044ed9d`, persistence-replay `6d39f516e0f2` (its first run in the chain returned `SESSION_RUNTIME_FAILED` while the gate sessions ran beside it; the rerun alone passed); creator-smoke validated without a refresh; `host-check` PASS | pass |
| G2 contract | golden `a269a93095e5c1d1440e0b29af98d9d9f0b9ea317ae59273263f73c905c3e24f` in the contracts test, `manifest.json` and the runtime check; `26` module pins refreshed; PROVENANCE records the suite | pass |
| G3 unit | `slice_integral_is_one_when_open_and_falls_with_the_horizons` (open horizons `1.0`, `±45°` gives `0.5`, monotone in the positive horizon, an open hemisphere at tilts `0..1.3 rad` averages to `1.0 ± 1 %`), `fallback_reasons_name_each_missing_precondition` | pass |
| G4 render | spawn, lake and falls at `300` frames: `PASS`, `AMBIENT_OCCLUSION active radius=1.0m slices=3 steps=5`, water draws `8`, invalidations `0`; the `ao` capture at the spawn (`480 x 270`): open ground `0.952`, the `40 px` band along the pad's base `0.906` (`4.9 %` below), the band at the avatars' feet on the pad `0.635` (`33 %` below) | **metric fails as frozen** (the pad's base is a `10 cm` step whose contact band is thinner than the frozen `40 px`; the contact under the avatars carries the effect); recorded |
| G5 cost | GPU frame mean `332 / 314 / 323 µs` against `256 / 253 / 253` before: `1.30 / 1.24 / 1.28 x` | pass (at the bound at the spawn) |
| G6 fallback | the white placeholder is written at content creation and restored before the pass is dropped on a resize; the reasons are unit-tested | pass |
| G7 look (human) | `l3r-spawn-60.png`, `l3r-spawn-ao-60.png` (outside Git): the avatars and the crate darken the pad at their feet, the pads' sides darken at the ground, open ground is untouched | human |

Recorded as open: temporal filtering of the occlusion (item 4), the
per-pixel noise on vertical faces visible in the occlusion capture, a
contact band metric placed on a taller step.
