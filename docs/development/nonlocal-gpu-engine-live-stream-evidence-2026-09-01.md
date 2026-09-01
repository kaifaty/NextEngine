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

## Checks

- `cargo test -p xtask water_` without and with `desktop-sdl-ash`: PASS
  (`9` / `11` tests, including stream frame parsing and capacity);
- `cargo clippy -p xtask --all-targets` for both feature sets: clean;
- `cargo fmt --check -p xtask`: PASS;
- extractor Release build: PASS; standalone 4k/16k stream trials: PASS;
- three live Release runs: PASS, one dynamic draw per frame, `0` dropped
  timing samples, child summaries captured after `stream_closed`;
- `git diff --check`: PASS;
- desktop adapter unchanged since its `platform`/`host-check` PASS;
- screenshot/capture: `NOT_RUN`; the backend still has no readback path.

Decision: retain the two-process stream as the developer path to watch the
Nonlocal solver live in the engine. Next raise 16k to real time with ordered
extraction workers or GPU extraction, and move the ring to device-local
memory; both are presentation-only and keep the solver boundary unchanged.
