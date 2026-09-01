# Nonlocal GPU engine GPU extraction evidence — 2026-09-02

## Result and claim ceiling

`GPU_SURFACE_EXTRACTION_EQUIVALENT_BOUNDED`: the frozen NGQ5 presentation
extraction (top-down height splat, largest 8-connected component, one 3x3
close, local fill, one-pass bilateral with range sigma equal to the particle
radius) now runs as CUDA kernels inside the stream process. On every frame of
the 4k and 16k falling-dam lanes it reproduces the CPU reference mask and
mesh counts exactly and its depths within `6.9e-8 m`, below the engine's
micrometre position quantum. Per-frame cost falls from `11 / 41 ms` (CPU,
observer plus extraction) to `0.6 / 1.2 ms` (4k / 16k), so one extraction
worker keeps both lanes at real time with a 60 Hz surface and the three to
four CPU cores the live stream used are free.

Presentation-only, developer tool, Proposed SPEC-38/ADR-076. The GPU port
computes the presentation output only; the CPU reference keeps computing the
diagnostics, roots and acceptance gates of NGQ5 and remains the authority for
corpus evidence. Nothing flows back into physics.

## Frozen inputs and source boundary

```text
branch                          codex/water-research
base before change              a06384dd (D-042 commit)
extractor cuda_baseline.cu      ef2bfcbe069ce741daf47c8a691aac8fb7edf0518757232aedc4fb12fb3b7769
extractor cuda_baseline.hpp     18530f8ac5fd88cf9cc5e9668276e1066a21c5420fe3a896f7871306b3f14e1c
extractor main.cpp              6191282949bbd4fd7a39b16342aec334c5d6a796e5bc77e80163eabf00e9a4f5
extractor binary                cf5a2355a0c5b879fd8b00dfdc3615ce1248dd30531d513ed66277565dcbe052
xtask water_preview.rs          c56b734d06dc033c3b5fe60de82cb7e75420114ed360c89fe53d0894862166d4
release xtask                   de393665e51b4892281f7d4cddb13999efc7dada020da53dd2b386a9c37f6037
```

The desktop adapter is unchanged from the D-042 commit. The CPU reference
functions `game_surface_frame` and `extract_presentation_surface` are
untouched; the corpus commands do not use the GPU extractor.

## GPU port

Per extraction worker: one non-blocking CUDA stream (so it never serializes
with the solver's legacy default stream) and fixed device buffers sized from
the lane box and particle capacity.

1. `surface_splat_kernel`: one thread per particle applies the reference's
   observer bounds check (error flag), the same `ceil/floor` pixel index
   ranges and the same `d^2 > r^2` test in `double`, and records the
   maximum height per pixel with one 64-bit `atomicMax` on the bit pattern
   of the non-negative depth (the reference keeps `depth > current`).
2. The raw mask is downloaded and the largest component is labelled on the
   host with the unchanged `label_surface_mask`; the retained mask is
   uploaded.
3. `surface_dilate_kernel`, `surface_erode_kernel`, `surface_base_depth_kernel`
   (fill from retained 8-neighbours, error flag when none) and
   `surface_bilateral_kernel` (same neighbour order as the reference so the
   `double` sums round identically) produce the closed mask and depth.
4. The closed mask and depth are downloaded; mesh vertex/triangle counts and
   validity are computed on the host and the frame is serialized as before.

`--extractor cpu|gpu|verify` selects the path; `verify` runs both, streams
the GPU frame and fails closed on any closed-mask pixel mismatch, mesh count
mismatch or depth difference above `1e-6 m`. `xtask water-preview
--stream-extractor` passes the mode through (`gpu` default).

## Equivalence gate and its reason

The CPU observer accumulates `dx*dx + dz*dz` and the sphere height in
`long double`; the GPU uses `double`. At pixels whose centre lies exactly on
a particle sphere (the seed lattice at spacing `0.05 m` against pixel
centres at `0.0125 m` produces exact tangencies) `sqrt(r^2 - d^2)` amplifies
rounding noise of either precision to tens of nanometres, so a nanometre gate
fails on the first frame for both lanes with identical masks:

| Lane | first-frame raw depth difference | first-frame final depth difference |
| --- | ---: | ---: |
| 4k | `5.09e-8 m` | `2.91e-8 m` |
| 16k | `8.98e-8 m` | `6.86e-8 m` |

Neither value is a fidelity claim of the reference itself; both are below
the engine's `1 um` position quantum and far below the `12.5 mm` pixel
pitch. The frozen gate is therefore identical masks and mesh counts plus
`1e-6 m` of depth.

## Verification and standalone timing (frames to a file)

| Run | frames | mask mismatch px (raw / closed) | mesh count mismatches | max depth diff | CPU total | GPU total | raw JSON |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 4k verify, 960 steps, every 4 | `241` | `0 / 0` | `0` | `2.91e-8 m` | `2,649 ms` | `115 ms` | `2c1a03cd...` |
| 16k verify, 480 steps, every 4 | `121` | `0 / 0` | `0` | `6.86e-8 m` | `4,995 ms` | `162 ms` | `640ced01...` |

GPU-only streams (`--extractor gpu`), wall per 240 Hz step:

| Lane | workers | steps / every | wall per step | execute wall per step | extraction per frame | raw JSON |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| 4k | `1` | `960 / 4` | `1.41 ms` | `1.40 ms` | `0.57 ms` | `d6165a2c...` |
| 16k | `1` | `480 / 4` | `2.10 ms` | `1.99 ms` | `1.21 ms` | `4457edac...` |
| 16k | `2` | `960 / 4` | `2.13 ms` | `2.02 ms` | `1.23 ms` | `f1fc74f4...` |

The 4k `verify` and `gpu` streams are byte-identical across all `241`
frames. Both lanes are now solver-bound; the `4.17 ms` step budget has more
than two milliseconds of headroom at 16k.

## Live Release runs (900 rendered frames)

| Metric | 16k every 4, 1 worker, gpu | 4k every 4, 1 worker, gpu | 16k every 4, 2 workers, verify |
| --- | ---: | ---: | ---: |
| frames received / published / skipped | `340 / 337 / 2` | `342 / 339 / 2` | `187 / 184 / 3` |
| real-time ratio | `0.997` | `1.000` | `0.550` (CPU reference on the same cores) |
| render critical p95 / p99 / max | `341 / 422 / 695 us` | `87 / 111 / 531 us` | `263 / 298 / 587 us` |
| refresh-frame upload p95 / max | `233 / 384 us` | `39 / 44 us` | `193 / 258 us` |
| extraction per frame p95 | `1.65 ms` | `0.66 ms` | `1.64 ms` (GPU frame) |
| solver steps before close | `1,440` | `1,440` | `788` |
| live verification | n/a | n/a | `192` frames, `0` mismatches, `6.86e-8 m` |
| raw JSON | `4c80ac8c...` | `e9ce0c0b...` | `17730382...` |

## Checks

- extractor Release build: PASS;
- `cargo test -p xtask water_` with `desktop-sdl-ash`: PASS (`11` tests);
- `cargo clippy -p xtask` for both feature sets (`-D warnings` without the
  feature): clean;
- `cargo fmt --check -p xtask`: PASS;
- standalone `verify` on both lanes and one live `verify`: PASS;
- `git diff --check`: PASS;
- desktop adapter unchanged since its `platform`/`host-check` PASS;
- screenshot/capture: `NOT_RUN`.

Decision: make `gpu` the default extractor for the live bridge and keep the
CPU reference for corpus evidence and `verify`. The bridge is now solver-bound
at both lanes; the next lane to bring live is the SPEC-38 production
fixture size (`48k` in a `4 x 2 x 1 m` basin), where the accepted `3.23 ms`
step leaves under one millisecond of the 240 Hz budget.
