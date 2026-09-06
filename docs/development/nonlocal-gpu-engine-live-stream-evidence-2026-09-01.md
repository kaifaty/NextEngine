# Nonlocal GPU engine live stream evidence — 2026-09-01

## Result and claim ceiling

`ENGINE_VULKAN_LIVE_SURFACE_SUPPORTED_BOUNDED`: the original five-iteration
Nonlocal GPU solver now drives the production SDL3/Ash Vulkan renderer while
it runs. The 4k falling-dam lane renders at real time with a 60 Hz surface;
the 16k lane renders at real time with a 15 Hz surface and at `0.53x` real
time with a 30 Hz surface.

Two processes, one direction. `nonlocal-feasibility --game-surface-stream`
steps the solver on one persistent advected device state and writes the
edge-aware NGQ5 surface of every K-th step as a versioned binary frame to
stdout. `xtask water-preview --stream-binary` spawns it, validates every frame,
reconstructs normals and hashes the update on a worker thread, paces
publication by simulation time and publishes through the D-039 dynamic
surface ring. No byte flows back to the solver; the renderer never sees
particles, only bounded surface meshes.

This remains a developer presentation tool under Proposed SPEC-38/ADR-076.
The solver stays an external research binary outside the Cargo workspace, the
extractor is still the CPU reference, and the stream lane does not reproduce
the accepted corpus roots because the persistent-state path keeps float state
on the device and audits diagnostics only every 60 frames. CPU DFSPH remains
the fallback.

## Frozen inputs and source boundary

```text
branch                          codex/water-research
base before change              45935492b14d79af763a99fa8b3d81d716616781 (D-039 commit)
extractor cuda_baseline.cu      2ffb7e77a0518fd42aceb6eb0f6206e7747e09839c44d7aff515b83883fa3685
extractor cuda_baseline.hpp     9724e077b6bb8b2278aa3306d71d16f15dde0623011d9549642cdb4f00434b30
extractor main.cpp              4eeb7afbe0e8a972461c73d05d46da6f3aedd1a159962915f7280fa510c67841
extractor binary                fde1ea55b375fd5a1445b331f8648d6dcb1189909f18b57697319c7b5114f0e8
xtask water_preview.rs          2b52d53655511fc0b438a6fd68bc6e2f851642cbd427d4524bff5d225b365ce4
xtask water_stream.rs           b3ad1c2c58af5694a2c32b209a6bfc73bce49ae92fb8e13eee9145b7a3d6f195
release xtask                   f618b38dc8c6459afd46c5490aa1e2e3ff368829dbc496a622468a9a90ae66c3
```

The desktop adapter is unchanged from the D-039 commit.

## Stream protocol and solver loop

Frame (little-endian): magic `NEWS`, version `1`, step, cycle, box minimum and
maximum (6 x f64), vertex and triangle counts (u64), simulation seconds,
extraction milliseconds and physics milliseconds since the previous frame,
then `f64 x,y,z` vertices in OBJ writer order and zero-based `u32` triangle
indices. The reader rejects a bad magic or version, counts outside the bounded
profile, non-finite values, vertices outside the box plus one pixel pitch,
truncated payloads and out-of-range indices.

Solver loop per cycle:

1. `game_visual_particles` seeds the lane; the fixture is marked `advected`
   and its neighbor grid margin equals the largest basin extent so the
   persistent grid covers the whole basin. Without that margin the grid sized
   from the initial column lost particles at step 65 (`local_solve`).
2. One `CudaBaseline` per cycle; `execute(audit, advance = true)` hands the
   published positions and velocities to the next step on the device.
3. Every K-th step downloads only the published positions; every 60th emitted
   frame runs the full diagnostic capture (finite state, pair capacity,
   maximum degree). Frame steps hand a particle snapshot to one worker thread
   that runs the raw observer, NGQ5 extraction and serialization with
   single-slot back-pressure, so frames are never reordered or dropped by the
   solver process.
4. `SIGPIPE` is ignored; a consumer that stops reading yields the bounded
   `stream_closed` failure and the JSON summary on stderr.

Per-step cost of the same lane while the loop was refined (4k, RTX 3080):

| Loop variant | wall per step | note |
| --- | ---: | --- |
| rebuild fixture and solver every step (corpus loop) | `29.8 ms` | `~1.5 ms` GPU, rest allocation and capture |
| persistent advected solver, capture every frame | `10.1 ms` | failed at step 65 until the grid margin covered the basin |
| light position download, audit every 60 frames | `5.8 ms` | extraction serial on the solver thread |
| extraction on a worker thread | `4.15 ms` | real time is `4.17 ms` at 240 Hz |

Standalone stream trials (frames to a file, no renderer):

| Lane | steps / every | steps per wall second | execute wall / step | observer + extraction per frame |
| --- | ---: | ---: | ---: | ---: |
| 4k | `960 / 4` | `241` | `1.49 ms` | `5.1 + 5.7 ms` |
| 16k | `480 / 8` | `129` | `2.00 ms` | `19.2 + 21.5 ms` |

Raw trial summaries: `12106be7...` (4k), `b4e597f5...` (16k).

## Exact invocation

```text
cmake --build /tmp/nextengine-nonlocal-game-quality-build.zdvs81 \
  --target nonlocal-feasibility -j2
cargo build --release -p xtask --features desktop-sdl-ash

B=/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility
target/release/xtask water-preview --stream-binary $B \
  --stream-lane 4k --stream-steps 960 --stream-every 4 --stream-cycles 0 \
  --stream-rate 1.0 --frames 900 --extent 1280x720 \
  > /tmp/nonlocal-engine-water-stream-4k-every4.json
```

`--stream-lane 16k --stream-every 8` and `--stream-every 16` were run the
same way. `--stream-cycles 0` restarts the dam until the renderer finishes;
`--stream-rate` scales stream seconds per wall second (`0.5` is slow motion,
`0` publishes every frame as it arrives).

## Release results on RTX 3080 (900 rendered frames each)

| Metric | 4k every 4 | 16k every 8 | 16k every 16 |
| --- | ---: | ---: | ---: |
| surface cadence | `60 Hz` | `30 Hz` | `15 Hz` |
| frames received / published / skipped | `341 / 338 / 2` | `91 / 91 / 0` | `87 / 85 / 1` |
| published stream seconds / wall seconds | `5.633 / 5.638` | `3.000 / 5.639` | `5.600 / 5.633` |
| real-time ratio | `0.999` | `0.532` | `0.994` |
| last published step / cycle | `392 / 1` | `720 / 0` | `384 / 1` |
| solver steps completed before close | `1,440` | `776` | `1,504` |
| ring refreshes / copied bytes | `675 / 415,800,016` | `182 / 436,100,720` | `170 / 403,253,512` |
| frame-plan misses / hits | `1 / 899` | `1 / 899` | `1 / 899` |
| render critical p95 / p99 / max | `244 / 294 / 558 us` | `854 / 1,000 / 1,387 us` | `734 / 988 / 1,886 us` |
| refresh-frame upload p95 / max | `189 / 249 us` | `909 / 1,297 us` | `931 / 1,781 us` |
| frame-source p95 | `118 us` | `1,077 us` | `420 us` |
| extraction per frame p95 | `6.17 ms` | `25.6 ms` | `25.9 ms` |
| physics per frame p95 | `6.17 ms` | `16.1 ms` | `29.1 ms` |
| dropped timing samples | `0` | `0` | `0` |

Raw JSON SHA-256: `7247489f...` (4k every 4), `f9469ce4...` (16k every 8),
`2cd2d074...` (16k every 16). The window ran uncapped at roughly 160 frames
per second; pacing therefore reflects simulation time against wall time, not
frame count.

## Observations

- The 4k lane is real time end to end: solver, extraction, transfer, ring
  refresh and raster all fit the 240 Hz step budget with a 60 Hz surface.
- The 16k lane is bounded by the CPU reference extractor (`~41 ms` per frame
  for observer plus bilateral pass on one thread), not by the GPU solver
  (`2.0 ms` per step) or the renderer (`<1 ms` critical path). Real time
  needs either two to three ordered extraction workers or the planned GPU
  extraction.
- 16k publication costs `~1 ms` on the render thread (`with_sequence` clone
  of a `1.5 MB` payload) and `~0.9 ms` ring refresh; both would vanish with a
  device-local, compute-written ring.
- Dam restarts are visible as a hard cut when `--stream-cycles 0` wraps; that
  is the tool's loop, not a solver event.

## Ordered extraction workers (same day, follow-up)

The stream process now runs observer, extraction and serialization on a
bounded pool of worker threads (`--workers`, default `3`). Jobs carry a
sequence number and finished frames are flushed strictly in sequence by the
worker that completes the next expected frame, so parallelism never reorders
or drops a frame; the queue is bounded at two jobs per worker and applies
back-pressure to the solver thread. The frozen extraction algorithm is
untouched: the streamed geometry of the 16k lane is byte-identical for one,
three and four workers after excluding the two per-frame timing fields.

Standalone 16k trials (`480` steps, every `8`, frames to a file):

| Workers | wall per step | note |
| ---: | ---: | --- |
| `1` | `7.05 ms` | extractor-bound |
| `2` | `3.53 ms` | inside the `4.17 ms` step budget |
| `3` | `2.45 ms` | selected default |
| `4` | `1.88 ms` | solver-bound (`1.79 ms` execute wall) |

Raw trial summaries: `f33f81ad...`, `0b200e09...`, `ea98eb44...`,
`beb37598...`. The engine side additionally assigns the update sequence on
the converter thread, so the render thread publishes without cloning the
payload.

Live Release runs with workers (900 rendered frames each):

| Metric | 16k every 8, 3 workers | 16k every 4, 4 workers | 4k every 4, 2 workers |
| --- | ---: | ---: | ---: |
| surface cadence | `30 Hz` | `60 Hz` | `60 Hz` |
| frames received / published / skipped | `172 / 169 / 2` | `341 / 336 / 4` | `341 / 336 / 4` |
| real-time ratio | `0.996` | `0.997` | `0.999` |
| solver steps completed before close | `1,552` | `1,464` | `1,448` |
| ring refreshes / copied bytes | `338 / 806,112,344` | `670 / 1,601,808,152` | `672 / 415,143,944` |
| render critical p95 / p99 / max | `924 / 1,102 / 1,671 us` | `985 / 1,111 / 2,229 us` | `329 / 473 / 1,050 us` |
| refresh-frame upload p95 / max | `950 / 1,601 us` | `914 / 2,087 us` | `265 / 453 us` |
| frame-source p95 | `251 us` | `250 us` | `67 us` |
| extraction per frame p95 | `27.6 ms` | `26.8 ms` | `6.5 ms` |
| dropped timing samples | `0` | `0` | `0` |

Raw JSON SHA-256: `4dc98c95...` (16k every 8), `86db22b5...` (16k every 4),
`bfcf2cef...` (4k every 4). Sources after the follow-up:

```text
extractor cuda_baseline.cu      57a7ff0e7a077625c6ac646fb7f48bcb1d3496c9311a446021bc5856a107eef8
extractor cuda_baseline.hpp     577dff55e573ce28062db14f72cc669d1e338412a9625e9dd4d7760b21c90422
extractor main.cpp              bd2adced60137dc947cc38963b0dac9adf81a008d34051fd74d89e24cfcffec4
extractor binary                0364f7cbd08178754b5836f895b32a4a991b7e94b647f3aa3ec5071c65a3c7d3
xtask water_preview.rs          e87ff42003d0ab35ca08788bafe2c7d67edaf726d37ad9e28a9f71ac8705cce2
release xtask                   17c9ade1d1633d57634a6cf8fdf7c86f7ceb45f068d8946d8cc5bf3d5b1387cc
```

Both lanes are now real time at a 60 Hz surface on this host. The remaining
render-side cost at 16k is the host-visible ring (`~0.9 ms` refresh, host
vertex fetch), which the device-local ring addresses next; GPU extraction
remains the path that removes the CPU extractor entirely.

## Checks

- `cargo test -p xtask water_` without and with `desktop-sdl-ash`: PASS
  (`9` / `11` tests, including stream frame parsing and capacity);
- `cargo clippy -p xtask --all-targets` for both feature sets: clean;
- `cargo fmt --check -p xtask`: PASS;
- extractor Release build: PASS; standalone 4k/16k stream trials: PASS;
- three live Release runs before and three after the worker follow-up: PASS,
  one dynamic draw per frame, `0` dropped timing samples, child summaries
  captured after `stream_closed`;
- worker-count sweep: streamed 16k geometry byte-identical for 1/3/4 workers;
- `git diff --check`: PASS;
- desktop adapter unchanged since its `platform`/`host-check` PASS;
- screenshot/capture: `NOT_RUN`; the backend still has no readback path.

Decision: retain the two-process stream with ordered extraction workers as
the developer path to watch the Nonlocal solver live in the engine at 60 Hz
for both lanes. Next move the ring to device-local memory, then port
extraction to the GPU; both are presentation-only and keep the solver
boundary unchanged.
