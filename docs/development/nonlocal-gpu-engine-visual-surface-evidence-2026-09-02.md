# Nonlocal GPU engine visual surface evidence — 2026-09-02

## Result and claim ceiling

`LIVE_WATER_VISUAL_SURFACE_BOUNDED`: the live bridge can now be watched until
the window is closed, one rendered frame can be copied back to a PNG as human
evidence, and the streamed height field no longer shows the pits of the raw
sphere-cap projection. The pit fix is a new frozen presentation variant
(NGQ6 revision 2, a 5x5 grayscale closing before the unchanged NGQ5
pipeline); its revision 1 (a dome envelope) was refuted by its own gate and is
recorded here. A static basin mesh gives the water a visual reference.

Everything remains presentation-only under Proposed SPEC-38/ADR-076. The
frozen NGQ5 sphere model stays the reference for corpus evidence and is still
selectable; the closing model is the live default of the developer bridge.
Captured PNGs are developer evidence, not a correctness oracle (SPEC-04).

## Frozen inputs and source boundary

```text
branch                          codex/water-research
base before change              653d4cee (D-044 commit)
extractor cuda_baseline.cu      b16d83ba0a64dfb53cca2b0402b795265d052c65378375d7c26a998e4ecf3e8a
extractor cuda_baseline.hpp     fa35362589fbf978ab7169a1f50e5cd32a0e7b9a9860536703066eb62fb3aa7d
extractor main.cpp              1f0e5eb9520a202ba95d7e6f5c9dbe27f6af47db87e49693677545bace9cde2d
extractor binary                908860ea2b68deecf6a85a4c4fedb70507439026c7ad8a3b16ddea3b9f552c5c
desktop graphics.rs             10739382d31dbac18e4e8bab6fecef60015021d2780f8ab7e84e163dfe984524
desktop run_state.rs            fbae7a1546b787f1797529e3ccd559381a564ba92cb5a7c5dd97c867e232815e
desktop resources.rs            08cdc40fb457f4d04ca19bf8931f4013f48f2a5112c3bfbae979967a2b7f8021
xtask water_preview.rs          6998b0a9a9982ce6515accc521c13daecf4be5c9015cd638eb82d99a83b50ccd
release xtask                   86d02b6dfb90e0599591d025da88daa7c3ddd5e851256dd515d6b3c1975574ad
```

## Frame capture and run-until-close

`DesktopRunOptions::frame_capture` names one rendered frame index. The
swapchain is then created with transfer-source usage (a surface without it
fails closed with `PRESENTATION_FRAME_CAPTURE_UNSUPPORTED`); after rendering
that frame the image is transitioned to transfer-source, copied into a
host-visible buffer and transitioned to present. The copy is read after the
device is idle at finish and reported as tightly packed sRGB RGBA8
(`DesktopCapturedFrameV1`). `water-preview --capture-frame N --capture-png
file` writes it as PNG with the workspace's existing zlib and CRC crates;
`--until-close` removes the frame bound. The monitor runs at 200 Hz with the
adapter's FIFO present mode, so no software frame limit was added.

Checks after this adapter change: `platform` PASS, `host-check` PASS on
Rust `1.97.1` (the first rerun caught a dead-code lint in the no-feature
`xtask` build, fixed before the passing rerun).

## What the first capture showed

The first live 16k frame (`85caec85...`, step 100 of the dam break, sphere
model) showed a blue sheet with dark vertical streaks along its edges and
banded structure. Measured on the streamed sphere frames of the 16k lane
(`gpu` stream, every 4 steps), the lattice ridge at the 50 mm particle period
has only `7 mm` amplitude, but a large share of interior pixels sit far below
the maximum of their 8 neighbours:

| Step | interior pixels | drop `> 50 mm` | drop `> 100 mm` | p95 drop | max drop |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 40 | `25,122` | `8.2%` | `5.7%` | `0.123 m` | `0.431 m` |
| 100 | `33,466` | `20.6%` | `12.6%` | `0.209 m` | `0.453 m` |
| 200 | `47,069` | `17.7%` | `10.0%` | `0.162 m` | `0.365 m` |
| 400 | `49,298` | `20.0%` | `8.6%` | `0.126 m` | `0.240 m` |
| 480 | `50,244` | `19.3%` | `7.2%` | `0.113 m` | `0.240 m` |

These are not lattice texture: the sphere-cap projection covers a pixel only
within `r = 25 mm` of a particle centre, circles at spacing `2r` cover at
most `pi / 4` of the plane, and any uncovered pixel shows the particle below.
Even settled water (step 480) keeps a fifth of its pixels in such pits.

## NGQ6 revision 1 (dome envelope): refuted

Frozen before running: height source = maximum over particles within
`H = 2 * spacing = 0.1 m` of `y + r * (1 - (d / H)^2)`, sphere mask unchanged,
gate `lift p95 <= 2r` and no pixel above the particle ceiling. Result on the
first eleven frames of both lanes (raw JSON `2cb1944a...` 4k,
`4196eb2d...` 16k):

| Lane | frames before failure | lift p95 at failure | mask / mesh mismatches CPU vs GPU | max depth diff |
| --- | ---: | ---: | ---: | ---: |
| 4k | `10` | `0.267 m` | `0 / 0` | `1.3e-8 m` |
| 16k | `10` | `0.215 m` | `0 / 0` | `1.6e-8 m` |

The envelope bridged real vertical gaps in the falling column: the gate did
its job and the variant was not tuned.

## NGQ6 revision 2 (5x5 grayscale closing): accepted

Chosen from an offline experiment on the streamed sphere frames before any
code change (3x3 and 5x5 closing at steps 40/100/480):

| Step | closing | lift p50 | lift p95 | lift max | residual pits `> 50 mm` | `> 100 mm` |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 480 | none | | | | `19.3%` | `7.2%` |
| 480 | 3x3 | `2 mm` | `102 mm` | `254 mm` | `7.5%` | `0.2%` |
| 480 | 5x5 | `5 mm` | `115 mm` | `254 mm` | `2.1%` | `0.0%` |
| 100 | 5x5 | `5 mm` | `207 mm` | `447 mm` | `0.0%` | `0.0%` |

The p95 lift equals the pit depth being filled, so the revision-1 gate was the
wrong observable. Frozen gates for revision 2: sphere mask unchanged
(structural), height never above the local 5x5 raw maximum (structural, still
checked), median lift `<= r = 25 mm` (the bulk surface must not move), plus
the unchanged NGQ5 mask gates. Results with the GPU port verified against the
CPU reference on every frame:

| Run | frames | gate failures | max lift p50 | max lift p95 | min lift | ceiling violations | CPU vs GPU mask / mesh | max depth diff | raw JSON |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 4k verify, 960 / 4 | `241` | `0` | `10.4 mm` | `266 mm` | `-6.9 mm` | `0` | `0 / 0` | `1.2e-8 m` | `d6d6967f...` |
| 16k verify, 480 / 4 | `121` | `0` | `9.7 mm` | `298 mm` | `-9.8 mm` | `0` | `0 / 0` | `1.2e-8 m` | `ab0494cb...` |
| 48k verify, 240 / 4 | `61` | `0` | `8.1 mm` | `490 mm` | `-9.6 mm` | `0` | `0 / 0` | `2.3e-8 m` | `c1a82829...` |

The small negative lifts come from the unchanged bilateral pass after the
closing. GPU cost with the closing (one worker, `gpu` extractor): `0.77 /
1.88 / 2.18 ms` per frame for 4k / 16k / 48k (`6a705eb4...`, `59ca96fb...`,
`f3104c72...`); the lift percentiles use `nth_element` on the live path.

## Live captures (16k, `gpu` extractor, one worker, 60 Hz surface)

| Capture | model | scene | rendered frame | solver step | render critical p95 / p99 | real-time ratio | PNG SHA-256 | raw JSON |
| --- | --- | --- | ---: | ---: | ---: | ---: | --- | --- |
| first look | sphere | water only, far camera | `480` | `~100` | `367 / 432 us` | `0.997` | `85caec85...` | `d148e1fb...` |
| closing | closing | water only, closer camera | `160` | `300` | `359 / 489 us` | `0.988` | `34c8da95...` | `b655f178...` |
| closing | closing | water only, closer camera | `480` | `784` | `338 / 413 us` | `0.996` | `0d2d8271...` | `3ab9b9a5...` |
| basin | closing | water plus basin | `480` | `784` | `347 / 412 us` | `0.995` | `9462e57d...` | `b731a75b...` |

The closing frames show a continuous sheet with a wave train at step 300 and
a choppy surface at step 784; the streaks are gone. The basin (floor plus
four inward-facing walls at the declared envelope, a grey material, static
catalog content) adds a second static draw and the water's shadow on the
floor; the frame plan reports two visible objects and two indexed draws, one
of them dynamic. The camera now sits at `0.9 x` the basin span instead of
`1.5 x`, so 16k roots differ from the D-042 report (catalog `640bd40a...`
with the basin). PNGs stay outside Git.

## Remaining visual work

- The water material is a flat opaque blue under the locked B0 shader; no
  refraction, reflection or foam. Those are optional SPEC-38 stages.
- Front edges still fall to the raw sphere height of partially covered
  boundary pixels; the closing does not lift mask boundaries by design.
- The visible ripple pattern is the solver's own dynamics at five iterations
  and is not a presentation artifact.

## Checks

- `cargo test -p next_desktop_sdl_ash`: PASS (`69` tests);
- `cargo test -p xtask water_` with `desktop-sdl-ash`: PASS (`11` tests);
- `cargo clippy` for `next_desktop_sdl_ash` and `xtask` (both feature sets,
  `-D warnings` without the feature): clean;
- `cargo fmt --check`: PASS; `cargo check -p next_game`: PASS;
- `platform` and `host-check` after the adapter capture change: PASS;
- extractor Release build, three `verify` runs and three `gpu` runs: PASS;
- four live captures: PASS, `0` dropped timing samples;
- `git diff --check`: PASS.
