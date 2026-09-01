# Nonlocal GPU engine dynamic surface evidence — 2026-09-01

## Result and claim ceiling

`ENGINE_VULKAN_DYNAMIC_SURFACE_SUPPORTED_BOUNDED` for the five accepted NGQ5
keyframes of both falling-dam lanes.

The SDL3/Ash desktop adapter gained a presentation-only dynamic surface
boundary: a composition root declares one exact catalog mesh revision with a
fixed vertex/index capacity, the adapter allocates one host-visible
vertex/index ring per frame slot once, and the frame source publishes
immutable `DynamicSurfaceUpdateV1` payloads next to (or instead of) a new
`PresentationSnapshotV3`. `xtask water-preview` now accepts repeated `--mesh`
keyframes, declares that surface and cycles the keyframes on a deterministic
pump schedule.

Across 600 rendered frames the render-content catalog, the presentation
snapshot and the B0 frame plan are each built once. The frame planner reports
`1` miss and `599` hits, the catalog root is constant, and every frame submits
one indexed draw that consumes the ring instead of the immutable catalog
geometry. No value feeds back into physics; the update hash is excluded from
every snapshot, frame-plan and gameplay root.

This remains a developer presentation tool under Proposed SPEC-38/ADR-076. The
keyframes are still produced by the CPU NGQ5 extractor, the ring is
host-visible rather than device-local, and no runtime/gameplay consumer exists.
CPU DFSPH remains the fallback.

## Frozen inputs and source boundary

```text
branch                          codex/water-research
base before change              35df24d0d9cdb95b200b9db112b01906d94420bc
extractor cuda_baseline.cu      2d6beee423b5c42239a85d57bcda1538fa05cc71e1217dc309a8d63a8f6bb1b8
extractor binary                b676b7228accf72258d6296ff6f8df2234829109ffac08ed526b190af7461984
desktop dynamic_surface.rs      c05596b3f90c2bb29038f1ac989fb8304f486a7b45d72239b533474f83a5c075
desktop gpu_content.rs          28747773d719f701013ab7f9a1ec1453d524299df5b2f6545d43da75c9066946
desktop graphics.rs             b25d1355e8156279c721e91d3aecc419af1d838f3d71f7ac753c916f6d2064f6
desktop prepared_run.rs         3f652bb9902ad0568dcc1e0e8cb3ba9d392328e904da9a9d79d3dcd34724ab9d
xtask water_preview.rs          1d4d16110df8cfb27ac938bd9e29356d0d3221362c0aff8951e4c17cbb8eab89
release xtask                   296e1edbfe69180d6376de61177d2d9ef546c9e1094d19fb26d6437432cb850f
```

The only extractor change writes every accepted keyframe as
`<prefix>-<lane>-step<N>-surface.obj` in addition to the final
`<prefix>-<lane>-surface.obj`. Extraction, gates and roots are untouched: the
rerun reproduces corpus result root
`a98f189b5fc4b35099376745605bc9e2851a1b4b6605cfa3c9e581d6e2adf391` exactly,
and the final OBJ files hash to the accepted `23ba8765...` (4k) and
`0a20f711...` (16k). Raw keyframe JSON SHA-256:
`cc45fbd29b89bc7f02e9690f4ab3d1c8f0e2b04e654f23810ba3fc3d95d75a0d`.

```text
/tmp/nextengine-nonlocal-game-quality-build.zdvs81/nonlocal-feasibility \
  --game-surface-prototype --frames /tmp/nonlocal-edge-surface-keyframes \
  > /tmp/nonlocal-edge-surface-keyframes.json
```

Keyframe OBJ SHA-256 and mesh sizes (vertices / triangles):

| Step | 4k | 16k |
| --- | --- | --- |
| 0 | `31200b90...` `6,400 / 12,482` | `654b2fa2...` `25,600 / 50,562` |
| 24 | `461424ac...` `6,400 / 12,482` | `d67b4979...` `25,600 / 50,562` |
| 48 | `02a947c0...` `6,832 / 13,330` | `d53b351a...` `26,512 / 52,370` |
| 72 | `2af6b012...` `8,546 / 16,706` | `cd9b4561...` `30,180 / 59,650` |
| 96 | `23ba8765...` `10,071 / 19,708` | `0a20f711...` `33,634 / 66,510` |

Generated OBJ and run JSON remain under `/tmp` and are not committed.

## Boundary semantics

- `DynamicSurfaceProfileV1 { mesh_revision, vertex_capacity, index_capacity }`
  is declared once in `DesktopRunOptions::dynamic_surfaces`. The revision must
  exist in the exact catalog with one triangle primitive; capacity is bounded
  by `1,048,576` vertices and `3,145,728` indices and never grows at runtime.
- `DynamicSurfaceUpdateV1` validates a closed triangle payload (non-empty,
  triangle multiple, in-range indices, one non-zero snorm16 normal per vertex)
  and binds a content-only canonical hash. Republishing the same payload under
  a later sequence keeps the hash, so a ring that already holds it skips the
  copy.
- Publication fails closed with typed codes for an undeclared revision,
  capacity excess, a vertex outside the catalog mesh bounds, or a sequence
  regression. One publication batch is staged and committed atomically.
- The catalog mesh remains the stable identity, material binding and declared
  presentation envelope; `water-preview` cooks the first keyframe inside the
  union bounds of all keyframes, so every update stays inside the bounds the
  frame plan validated.
- Both the shadow pass and the world pass bind the slot ring for a dynamic
  draw and rebind the immutable stream afterwards. Device loss or a swapchain
  format change rebuilds the rings from the declared profiles.
- The adapter report and per-frame timing sample expose publications, ring
  refreshes, copied bytes, the current update hash and a separate
  `dynamic_surface_upload` CPU phase. Nothing enters the frame-plan hash.

## Exact invocation

```text
cargo build --release -p xtask --features desktop-sdl-ash

K=/tmp/nonlocal-edge-surface-keyframes-falling-dam-4k   # or -16k
target/release/xtask water-preview \
  --mesh $K-step0-surface.obj --mesh $K-step24-surface.obj \
  --mesh $K-step48-surface.obj --mesh $K-step72-surface.obj \
  --mesh $K-step96-surface.obj \
  --frames 600 --hold 8 --extent 1280x720 \
  > /tmp/nonlocal-engine-water-dynamic-4k-a.json      # repeated as -b

target/release/xtask water-preview --mesh $K-surface.obj \
  --frames 600 --extent 1280x720 \
  > /tmp/nonlocal-engine-water-static-4k.json
```

With hold `8` and five keyframes the schedule publishes a new keyframe every
eight pumps: `75` publications per 600-frame run, each refreshed once per
frame slot (`150` ring refreshes), `450` frames reuse an already uploaded ring.

## Release results on RTX 3080 (dynamic keyframes)

| Metric | 4k A | 4k B | 16k A | 16k B |
| --- | ---: | ---: | ---: | ---: |
| rendered frames | `600` | `600` | `600` | `600` |
| frame-plan misses / hits | `1 / 599` | `1 / 599` | `1 / 599` | `1 / 599` |
| publications / ring refreshes | `75 / 150` | `75 / 150` | `75 / 150` | `75 / 150` |
| copied bytes | `59,024,040` | `59,024,040` | `219,557,280` | `219,557,280` |
| dynamic draws in last frame | `1` | `1` | `1` | `1` |
| CPU extract+submit p95 | `179 us` | `174 us` | `464 us` | `467 us` |
| GPU duration p95 | `92 us` | `172 us` | `471 us` | `592 us` |
| critical-path p95 / p99 | `182 / 245 us` | `186 / 216 us` | `537 / 583 us` | `592 / 609 us` |
| critical-path max | `603 us` | `526 us` | `753 us` | `1,021 us` |
| refresh-frame upload p95 / max | `158 / 192 us` | `140 / 189 us` | `469 / 664 us` | `476 / 633 us` |
| idle-frame upload p95 | `0 us` | `0 us` | `0 us` | `0 us` |
| frame-source p95 | `66 us` | `67 us` | `206 us` | `196 us` |
| dropped timing samples | `0` | `0` | `0` | `0` |
| engine-owned device allocation | `29,969,872 B` | `29,969,872 B` | `33,407,200 B` | `33,407,200 B` |

Exact presentation roots (identical across A and B):

```text
4k catalog        ed5b637968adba3fe7946be276f2cb9507bc65661eb0ba438396f9ea2fd351ef
4k snapshot       889311846ddd1aebc2b72982311e033dff010aa89b52dec78d43bc411d66cc69
4k frame plan     f13b8a832308c8a28e0a0b4848aed26ff4c7f96fb848b01ea7721a891f404ef4
4k final update   f30d8f7e360798e60478653cc2d5dbd02fe5b2e612cea61dc01eb3dbcef174ff
16k catalog       bdd9c475a32cf43cfd293fddc972e87309a12e0f9921a534506597c72d73c80f
16k snapshot      3be1373a8cbc76e70e4582ab71d04029575678d6fe5646b68af18aa39c209b6d
16k frame plan    7c3dbf62a5a8613b7c5226f6478292955984dcbaaa804ab152bd5246270bd6d7
16k final update  6bc94b02c370ead2f159c902237374884d88d69beef33c953b4614f5d812a163
```

Raw JSON SHA-256:

```text
4k A   d8b97d58fb834f2d7c59e7c2143bd883cda05224cb5d7ea9f195edf95eeaeda1
4k B   769f607f6b33d0c9855d8affa7f2288d40c684e38860f6648b97916783ccb970
16k A  a1a73823a833eaaef106cf0378c38a8e56df3fd344eea87a998fbf805306124c
16k B  a204588d128e16243f498e5aa06ca63d291bd767cce24a75c671a7cf77a376dc
```

After excluding only timing fields and the frame counters derived from them,
the A and B reports are identical for both lanes.

## Static regression

The single-`--mesh` path is unchanged and reproduces the prior bridge exactly:

| Lane | catalog | snapshot | frame plan | critical p95 / p99 |
| --- | --- | --- | --- | ---: |
| 4k final | `597dd35a...` | `24a9afa9...` | `8eaa50fa...` | `75 / 99 us` |
| 16k final | `7dd5d84a...` | `e4ed73de...` | `aad92e8a...` | `316 / 317 us` |

These roots match `nonlocal-gpu-engine-water-preview-evidence-2026-09-01.md`.
Raw JSON SHA-256: `23d1c06f...` (4k), `5546eba9...` (16k).

## Observations

- The catalog-rebuild question is closed for this tool: refreshing the surface
  costs one host copy per frame slot, and repeated frames cost nothing.
- Copying a 16k keyframe (about `1.46 MB`) into a host-visible ring is
  `~0.47 ms` p95 on the render thread and already exceeds the static 16k raster
  critical path. Host-visible vertex fetch also raises GPU duration from
  `~315 us` (static, device-local) to `~470--590 us`. A device-local ring fed
  by a staging copy or written by GPU compute is the next presentation
  measurement, not a physics change.
- The frame-source maximum of `~41--42 ms` is the first pump in every run,
  including both static runs, and is therefore SDL startup event handling
  rather than keyframe publication; steady-state p95 is `66--206 us`.
- With two frame slots and hold `8`, each publication is uploaded exactly twice.
  Hold `1` would upload once per publication and leave the previous slot's ring
  stale but unread, which the hash check tolerates by design.

## Checks

- `cargo test -p next_desktop_sdl_ash`: PASS (`69` tests, including update
  validation, staged batch rejection and vertex packing);
- `cargo test -p xtask water_preview` without and with `desktop-sdl-ash`: PASS
  (`6` / `8` tests);
- `cargo clippy` for `next_desktop_sdl_ash` and `xtask` (both feature sets):
  clean;
- `cargo check -p next_game`: PASS with the default empty declaration;
- extractor rerun: corpus root and final OBJ hashes exact;
- two 600-frame dynamic Release runs per lane: PASS, `0` dropped samples;
- one 600-frame static Release run per lane: exact prior roots;
- `cargo run -p xtask --features desktop-sdl-ash -- platform`: PASS;
- `cargo run -p xtask -- host-check`: PASS on Rust `1.97.1`;
- `git diff --check`: PASS;
- screenshot/capture: `NOT_RUN`; the backend still has no readback path.

Decision: retain the declared dynamic surface ring as the engine-facing
presentation update boundary. The next smallest step is a device-local ring
(staging copy or compute-written) plus GPU bilateral extraction, timed through
the same `dynamic_surface_upload` phase, so live extraction cost is measured
separately from `3.23 ms` physics and B0 raster.
