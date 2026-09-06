# Water look L8 — DLSS-ready outputs (HUD-less colour, thin G-buffer, motion vectors, jitter, group mask)

| Field | Value |
| --- | --- |
| Research ID | `WL8` |
| Status | `RUN` (2026-09-03) |
| Parent | plan `continuum-water/11` item L8; research note `docs/development/water-look-dlss-research-2026-09-03.md`; ADR-090 (no vendor SDK in the workspace); the closed `B0ShaderInterfaceV2` contract (`interface_contract_sha256`) |
| Purpose | the desktop adapter emits, every frame, the buffers a temporal upscaler or denoiser consumes (HUD-less scene colour, albedo + group mask, normal + roughness, screen-space motion vectors, linear depth) and can jitter the projection, without changing the frame plan, the B0 shader contract or any gameplay root; no DLSS SDK is integrated |

## Frozen scope

- **HUD-less colour.** The sky, world, water and fluid passes render into
  an offscreen scene colour target of the swapchain's format; the frame
  blits it to the swapchain and draws the UI overlay on the swapchain
  afterwards. The capture of `color` reads the swapchain (HUD included)
  as before; `scene` reads the offscreen target.
- **G-buffer suite (`gbuffer`).** A separate shader suite with its own
  pipeline layout, after the fluid pass, drawing the frame plan again
  with `LESS_OR_EQUAL` depth test and depth write into the frame's depth
  attachment, into four `32`-bit attachments of the swapchain extent:
  `albedo_mask` `R8G8B8A8_UNORM` (base colour times texture, alpha =
  group mask), `normal_roughness` `R8G8B8A8_UNORM` (world normal encoded
  `0.5 n + 0.5`, alpha = roughness), `motion` `R16G16_SFLOAT` (screen
  pixels from the previous frame to this one, `+x` right, `+y` down, the
  DLSS convention), `linear_depth` `R32_SFLOAT` (view-space distance
  along the camera axis in metres). Set 0 of the suite: binding 0 a
  `160`-byte uniform (`view_projection`, `previous_view_projection`,
  `viewport` (width, height, 1/width, 1/height), `jitter` (current xy,
  previous xy, in pixels)), binding 1 a storage buffer of previous model
  matrices (`64` bytes per draw; the buffer holds `GBUFFER_MAX_DRAWS =
  4_096` entries and the pass skips draws beyond it). Push constants:
  the B0 draw block (`model`, `base_color_factor`, `80` bytes) followed
  by `meta` (`draw_index`, `group`, `roughness`, `0`) — `96` bytes. The
  B0 texture set (set 1) is reused for the albedo.
- **Motion history.** The adapter keeps, per frame, the model matrices
  of the plan's draws keyed by `(scene_record_hash, occurrence)`; the
  previous matrix of a draw is the entry of the previous rendered frame
  with the same key, or the current matrix when absent (zero object
  motion on the first frame). `previous_view_projection` is the
  previous rendered frame's jittered view-projection. Vertex
  displacement of dynamic surfaces (the ring's waves, at most `20 mm`)
  is not in the motion vector; the ring's translation is.
- **Group mask.** Adapter-local: `1` environment (indirect draws), `2`
  characters (skinned draws), `3` water surfaces (`WaterSurface`
  rings), `4` other dynamic surfaces; stored as `group / 255` in the
  albedo alpha. The sky and the fluid pass write no G-buffer sample
  (cleared: albedo `0`, mask `0`, normal `(0, 0, 0)`, motion `0`, linear
  depth `0`).
- **Jitter.** `DesktopRunOptions::projection_jitter: bool` (default
  `false`; `apps/game --projection-jitter`): the column-major
  projection's `[8]` and `[9]` elements (the `z` column's `x` and `y`
  lanes, which multiply `w_clip` after the perspective divide; the
  frozen text first named `[12]`/`[13]`, the translation lanes, which
  would scale with depth — corrected before the run) receive minus the
  Halton(2, 3) sub-pixel offset of the rendered frame index modulo
  `16`, `2 (h - 0.5) / extent` in NDC with the Vulkan `y` sign; the
  same jittered projection drives the world,
  water, reflection and G-buffer passes, the shadow pass is unjittered.
  No temporal resolve exists, so the default stays off.
- **Capture.** `apps/game --capture-buffer {color|scene|albedo|normal|motion|depth}`
  (default `color`) selects the image the existing capture copies; the
  copy stays a raw `4`-byte-per-pixel readback written as an `8`-bit
  PNG, so `motion` and `depth` captures are decoded numerically (half
  and single floats) rather than viewed.
- **Not in scope.** DLSS itself, any NGX or Streamline dependency,
  motion vectors of the fluid particles, per-vertex displacement motion
  of the ring, a temporal resolve.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 no authority change | `render-performance` pinned `frame_plan_hash` unchanged, `interface_contract_sha256` unchanged, `play` and `persistence-replay` roots unchanged, `host-check` PASS | pass |
| G2 CPU tests | Halton(2, 3): the first `16` samples are distinct and inside `(0, 1)`, sample `0` is `(0.5, 0.333..)`; a jittered projection differs from the unjittered one only in `[12]` and `[13]` by the expected NDC offsets; the motion history returns the current matrix on the first frame, the previous frame's matrix for a repeated key, and forgets a key missing for one frame; the G-buffer uniform is `160` bytes and the push block `80` bytes; the shader hash pins match | pass |
| G3 G-buffer evidence | release captures from `--start-at-water` at frame `30` (the crate rising): `motion` has `\|v\| > 0.25 px` only in a bounded region around the crate (at most `3` percent of the pixels) and `< 0.05 px` for `> 95` percent of the pixels; `normal` on the floor region decodes to `n_y > 0.95`; `albedo` alpha carries mask `1` on the floor, `2` on the avatar and `3` on the water; `depth` in the water region is between `2` and `12 m` | pass |
| G4 colour unchanged | the `color` capture at frame `120` with jitter off differs from the plan 17 capture by a mean absolute pixel difference `< 1/255` in the water region; with `--projection-jitter` it differs (`> 0`) and the frame plan is the same | pass |
| G5 allocation | the G-buffer suite adds at most `20` bytes per pixel of the swapchain extent (four attachments plus the scene target) and the previous-model buffer, reported through the adapter's allocation statistics | pass |

Formats, group ids, the Halton period and the capture rule are frozen;
a change after the captures is a new revision with its own evidence.

## Result (2026-09-03)

Implementation: `crates/desktop-sdl-ash/src/gpu_content/gbuffer.rs`
(`GBufferPassState`, `MotionHistoryV1`), the `gbuffer` shader suite,
`projection_jitter` in `pipeline.rs`, the scene target and the G-buffer
pass in `graphics.rs`, `DesktopCaptureSourceV1`, `apps/game
--projection-jitter --capture-buffer --capture-frames`.

Two apparatus corrections before the captures, recorded above: the
jitter lanes are `[8]`/`[9]`, and the push block is `96` bytes (the B0
draw bytes plus `meta`). One defect found by the first motion capture
and fixed before the evidence run: the draw history was keyed by the
scene record hash, which changes with the pose, so every moving draw
read its own current matrix (zero motion); the key is now (mesh,
material, texture revision, occurrence). One teardown defect (the pass
dropped after the device) found by the first run and fixed.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 no authority change | `host-check`, `play`, `persistence-replay`, `content-package`, `platform` PASS; `performance` (frame planner workload, pinned plan hash) PASS; `interface_contract_sha256` unchanged (`204ed27a…`) | pass |
| G2 CPU tests | `halton_jitter_is_distinct_bounded_and_periodic`, `projection_jitter_shifts_ndc_by_a_constant_offset_at_every_depth`, `motion_history_returns_current_then_previous_and_forgets_missing_keys`, `uniform_and_push_blocks_have_the_declared_layouts` (160 / 96 bytes), `checked_in_modules_match_the_offline_manifest` with the two new hashes: all pass | pass |
| G3 G-buffer evidence | `--capture-buffer` captures from `--start-at-water` (`controls=0`): `normal` frame 30, floor region `n_y` mean `1.000`, roughness `0.80`; `albedo` frame 30, mask `1` on the floor (all pixels), `2` on the avatar (majority), `3` on the water (majority), `0` on the sky; `depth` frame 30, water region mean `5.90 m`, max `8.64 m`, with zero on the few sky pixels the region touches; `motion` burst frames 28-35: frames 29 and 34 (the frames that straddle a presentation tick) show `0.23` / `0.26` percent of the pixels moving, all inside the crate's screen box (`x 348..393`, `y 303..358`), mean `(-0.2, -4.0) px` (the crate rising: up is `-y`), `99.7` percent below `0.05 px`; the other six frames are exactly zero, because the presentation pose is constant between ticks (about five rendered frames per tick in release) | pass, with the tick-cadence note |
| G4 colour unchanged | `color` frame 120 against the plan 17 capture: mean absolute difference `0.000` in the water region (max `0`); `scene` against `color`: `0.000` in the water region, `50.1/255` in the HUD region (the HUD is absent from the scene target); `--projection-jitter` `color` frame 120 against no jitter: mean `2.67/255` in the water region, `0.53/255` over the frame (`> 0`), the frame plan hash unchanged by construction (the jitter never enters the plan) | pass |
| G5 allocation | `device_allocation_bytes 57,487,872` after the pass at `960 x 540` (printed by `apps/game` since this plan); the pass adds `960 * 540 * 20 B = 10,368,000 B` of targets (four `4`-byte attachments plus the scene target, `20` bytes per pixel) and `2 * (160 + 262,144) B` of per-slot buffers, `10.9 MB` in total | pass |

Not in the evidence: the fluid particles and the sky write no G-buffer
sample (masked `0`), skinned draws carry their model motion only (the
previous frame's skinned vertices are not kept), and the motion of the
ring's wave displacement is not in the vector. Captures stay outside
Git.
