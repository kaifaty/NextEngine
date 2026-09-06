# Look L4 — temporal anti-aliasing on the plan 18 motion vectors (scene look item 4)

| Field | Value |
| --- | --- |
| Research ID | `SCENE-LOOK-L4` (research report, not a product check) |
| Status | `RUN / G1-G7 PASS / G8 HUMAN` (2026-09-05) |
| Parent | [`00-scene-look-roadmap.md`](00-scene-look-roadmap.md) item 4; plan `continuum-water/18` (the jitter and the motion vectors); plans [`01`](01-hdr-chain-and-physical-lighting.md) (the HDR scene target the history lives in) and [`03`](03-ambient-occlusion.md) (the prepass and the noisy occlusion this filters) |
| Purpose | every edge in the scene is a hard staircase, the sun's glints and the occlusion's per-pixel noise flicker, and the far checker shimmers. A temporal resolve on the sub-pixel jitter of plan 18 and the G-buffer's motion vectors, with variance clipping against ghosting, converges the HDR scene over frames before the tone map |

## Frozen scope

- **Resolve.** After the scene passes (world, water, fluid) and before
  the tone map, a fullscreen program reads the scene colour, the
  previous resolved history, the G-buffer's motion (pixels, jitter
  removed) and linear depth, and writes the new history: the history
  reprojected by the motion (bilinear), clipped to the current 3x3
  neighbourhood's mean `± 1.0` standard deviation per channel (variance
  clipping), blended `0.9` history and `0.1` current; the current colour
  alone on the first frame, off-screen reprojections and where the
  history is absent. Pixels without depth (the sky) reproject through the
  previous frame's view-projection instead of the motion target. The
  resolved history is copied back into the scene target, so the tone map,
  the captures and the HUD are unchanged.
- **Jitter.** With the pass present the projection jitter of plan 18 is
  on regardless of the option; `--no-temporal-aa` (`DesktopRunOptions::
  temporal_aa = false`) disables the pass for comparisons and leaves the
  jitter to its option.
- **Contract.** None: the pass owns its descriptor sets (set 0 the B0
  frame set, set 1 its inputs) and pushes its own block. The roots stay
  those of plan 03.
- **Evidence.** `TEMPORAL_AA active blend=0.9 history=R16G16B16A16_SFLOAT`
  printed once, or `TEMPORAL_AA_FALLBACK <reason>` (no G-buffer prepass,
  no sampleable history format).
- **Not in scope (recorded).** Sharpening, a Catmull-Rom history fetch,
  responsive-AA masks for the water's particles, DLSS itself (the buffers
  for it exist since plan 18).

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `water-present`, `audio-scene` PASS with roots equal plan 03's (`69c970d7df92`, `6d39f516e0f2`, `d65863602dc1`, `ee401044ed9d`); `host-check` PASS |
| G2 suites | the `taa` suite in `manifest.json` with its module pin; PROVENANCE records it; the interface contract hash unchanged |
| G3 unit | the variance clip returns a history inside the neighbourhood's box and leaves one inside it untouched; the blend with the first-frame flag returns the current colour; the fallback reasons |
| G4 render | spawn, lake and falls at `300` frames, capture at `60`: `PASS`, `TEMPORAL_AA active`, water draws `8`, invalidations `0` |
| G5 cost | GPU frame mean after within `1.2 x` the before (`332 / 314 / 323 µs`) |
| G6 fallback | `--no-temporal-aa` renders as plan 03 (`TEMPORAL_AA_FALLBACK: disabled by option` once); the fallback reasons unit-tested |
| G7 stability | at the spawn with a static camera, the mean absolute difference between frames `59` and `60` (RGB, `0..255`) with the jitter on and the pass off (`--projection-jitter --no-temporal-aa`) against the same with the pass on: the pass brings it to at most `30 %` of the jitter-only value |
| G8 look (human) | the far checker and the boxes' edges without staircases; no visible ghost trail behind the idle avatars |

## Result (2026-09-05)

Implementation: `taa.rs` (two `R16G16B16A16_SFLOAT` history targets, one
set per target reading the other, the resolve into this frame's target and
its copy back into the scene target), `taa.frag`, the jitter of plan 18 on
whenever the pass exists, `DesktopRunOptions::temporal_aa` and
`--no-temporal-aa`, `TEMPORAL_AA active` / `TEMPORAL_AA_FALLBACK`.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | play `69c970d7df92`, persistence-replay `6d39f516e0f2`, water-present `d65863602dc1`, audio-scene `ee401044ed9d`: plan 03's; `host-check` PASS | pass |
| G2 suites | `taa_suite` in `manifest.json` with `taa.frag`'s pins; PROVENANCE records it; the interface hash unchanged (`a269a93095e5…`) | pass |
| G3 unit | `history_is_clipped_and_blended`, `fallback_reasons_name_each_missing_precondition` | pass |
| G4 render | spawn, lake and falls at `300` frames: `PASS`, `TEMPORAL_AA active blend=0.9 history=R16G16B16A16_SFLOAT`, water draws `8`, invalidations `0` | pass |
| G5 cost | GPU frame mean `365 / 349 / 354 µs` against `332 / 314 / 323`: `1.10 / 1.11 / 1.10 x` | pass |
| G6 fallback | `--projection-jitter --no-temporal-aa`: `TEMPORAL_AA_FALLBACK: disabled by option` once, the frame as plan 03's with the jitter | pass |
| G7 stability | mean absolute difference between frames `59` and `60` at the spawn (RGB `0..255`, below the HUD): jitter only `2.011`, with the pass `0.173`: `8.6 %` | pass |
| G8 look (human) | `l4-spawn-60.png`, `l4-falls-60.png` (outside Git): the far checker and the boxes' edges without staircases; no ghost behind the idle avatars | human |

Recorded as open: a sharpening pass (the resolve softens the frame by
its bilinear history fetch), a responsive mask for the water's droplets,
DLSS on the same buffers.
