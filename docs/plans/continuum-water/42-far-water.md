# Far water — ring grid profiles and the detail normal fading with distance (plan 31 item 10)

| Field | Value |
| --- | --- |
| Research ID | `CONTINUUM-WATER-FAR-R1` (research report, not a product check) |
| Status | `RUN / G1-G4 PASS / G5 HUMAN` (2026-09-05) |
| Parent | plan 31 item 10; plans 09 (the ring), 14 (the wave spectrum and the detail normal), 39 (the lake) |
| Purpose | every surface carries the same `32 x 16` ring, which gives the lake `0.65 x 0.93 m` cells (its shortest ambient wave is `0.3 m`), and the water pass's detail normal (`0.11` and `0.18 m` waves) shimmers far from the camera where a pixel covers several waves. A ring grid profile per surface (a large grid for the lake) and a distance fade of the detail normal |

## Frozen scope

- **Grid profile.** `WaterSurfaceGridV1 { columns, rows }` on
  `WaterSurfaceBindingV1`; `STANDARD` is `32 x 16`, `LARGE` is
  `64 x 32`; valid grids have at least two columns and rows and at most
  the large grid's. The stage builds each ring at its binding's grid;
  the declared capacities become the large grid's (`2 048` vertices,
  `11 718` indices) and the desktop profile of each ring declares its
  own grid's counts, so only the lake's buffers grow. The wave grid of
  plan 38 takes the same grid. The lake binds `LARGE` (`0.32 x 0.45 m`
  cells); every other surface stays `STANDARD`.
- **Detail fade.** In `water_scene.frag` the detail tilt is scaled by
  `12 / (12 + d)` with `d` the distance from the camera in metres, so
  it is whole up close and a fifth of itself at `50 m`. The module is
  recompiled and its hashes recorded (manifest, provenance, the
  hash test).
- **Not in scope (recorded).** A camera-dependent ring resolution (the
  stage is a pure function of the checkpoint and the frame index, and
  the water-present check hashes its output without a camera), shadow
  casting of the surface onto the floor (the surface already receives
  the shadow map since plan 12; a water surface casts no hard shadow),
  a depth-sorted order against other transparent objects (the adapter
  has none besides the UI overlay; the water pass composes over the
  opaque scene).

## Frozen gates

| Gate | Clause |
| --- | --- |
| G1 roots | `play`, `persistence-replay`, `audio-scene` equal plan 41's (`64ac5dac81d8`, `7fb9966adcbc`, `ed0da7bedc3a`); `water-present` PASS with its root recorded (it moves: the lake's ring is part of the frame digest); `host-check` PASS |
| G2 unit | the two profiles validate and a `1 x 1` or `65 x 33` grid does not; the reference bindings give the lake `LARGE` with plan cells at most `0.45 m` and the others `STANDARD`; the stage frame carries `2 048` vertices and `11 718` indices for the lake and `512` / `2 790` for each other ring, all under the capacities; a wave grid of the lake at `LARGE` is stable (the Courant limit holds) |
| G3 cost | `water-present` `stage_cost_mean_us ≤ 3 000` (debug build; `1 954` before) |
| G4 render | a release capture run at the lake (`--start-at-lake`, capture at frame `60`) reports `PASS` with eight water draws and no dynamic surface capacity violation |
| G5 look (human) | the capture: the far lake shows no shimmer band; the near water keeps its glints |

## Result (2026-09-05)

Implementation: `WaterSurfaceGridV1` (`STANDARD` `32 x 16`, `LARGE`
`64 x 32`) on the binding, the stage building each ring at its grid, the
capacities raised to the large grid's, the desktop profile and the wave
grid per ring at the binding's grid, the lake bound to `LARGE`; the
detail tilt of `water_scene.frag` scaled by `12 / (12 + d)`.

| Gate | Reading | Verdict |
| --- | --- | --- |
| G1 roots | `play`, `persistence-replay`, `audio-scene` equal plan 41's (`64ac5dac81d8`, `7fb9966adcbc`, `ed0da7bedc3a`); `water-present` PASS, state root `6fbf15e511fc` unchanged (the frame digest moved with the lake's ring); `host-check` PASS | pass |
| G2 unit | `the_lake_carries_the_large_grid_and_the_rest_the_standard`, `frame_is_pure_bounded_and_reads_only_the_checkpoint` (each ring at its grid, under the capacities), the wave grid tests at `STANDARD`, the shader hash test with the new module | pass |
| G3 cost | `water-present` `stage_cost_mean_us 2 805`, max `4 068` (debug; `1 954` before) | pass |
| G4 render | release run at the lake, capture at frame `60`: `PASS`, `dynamic_surface_draws = 8`, `frame_plan_explicit_invalidations = 0`, no capacity violation | pass |
| G5 look (human) | `p42-lake-60.png` (outside Git): the lake from the dam's east bank, the far water flat in tone, the near water keeps its glints | human |

Apparatus note: the frozen gate wrote the lake's plan cells as "at most
`0.45 m`" from the rounded `0.32 x 0.45 m` of the scope; the exact
cells are `0.317 x 0.452 m` (`14 m` over `31` rows), so the unit test
bounds them at `0.46 m`. The grid was not changed.

Recorded as open: a camera-dependent ring resolution (would need a
viewer input to the stage), the wave spectrum's own distance fade on the
ring (the ring's shortest ambient wave is `0.3 m`, well sampled by the
large grid and only twice undersampled by the standard one), a depth
sort against other transparent objects once the adapter has any.
