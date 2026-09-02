# Nonlocal GPU screen-space fluid prototype (NGQ10) — 2026-09-02

## Result and claim ceiling

`NGQ10 REV 1 RUN / G2 G3 G5 PASS / G4 HUMAN PASS / G1 PARTIAL (validation
layer unavailable) / PROTOTYPE ONLY`: the ADR-102 particle surface pass
(sphere depth splat with MIN blend, sphere thickness with ADD blend, two
narrow-range smoothing passes, Fresnel/refraction/absorption composite) runs
in the SDL3/Ash adapter fed by the live particle stream of `water-preview`
at `1920x1080` on the RTX 3080. It costs `0.28..0.32 ms` GPU p95 per frame
for the `12,000` particles of the narrow spill lane, flips at most `0.20%`
of the viewport coverage between consecutive frames (`0.39%` with the
height-field ring drawn underneath), keeps the catalog, snapshot and frame
plan roots byte-identical across `mesh`, `particles` and `both`, and shows
the jet as one continuous falling stream with both bodies visible. This is
a presentation-candidate finding under Proposed ADR-102/ADR-100/SPEC-38;
nothing here enters gameplay, snapshot, replay or frame-plan authority.

## Frozen inputs

```text
plan              docs/plans/nonlocal-gpu-full-step-performance/24-screen-space-fluid-prototype.md
lane              spill-narrow, --stream-spill-lip flush, 960 steps, surface every 4, rate 1.0
window            1920x1080, 230 rendered frames per run, release build of xtask
particle profile  capacity 65,536, radius 35 mm, absorption [1.2, 0.5, 0.25]/m, refraction 0.08, thickness scale 1.0
extractor binary  07762c9c8559e742...
shader suite      fluid_surface (manifest.json), glslang 15.1.0 (96ea85d4...)
```

Commands (one per mode; `--capture-frame 25|50|200` runs used
`--capture-frames 1`):

```text
target/release/xtask water-preview --stream-binary <nonlocal-feasibility> \
  --stream-lane spill-narrow --stream-spill-lip flush --stream-steps 960 \
  --surface particles|mesh|both --extent 1920x1080 --frames 230 \
  --capture-frame 100 --capture-frames 5 --capture-png <scratch>/<mode>-f100.png
```

## Gates

| Gate | `particles` | `both` | `mesh` (reference) |
| --- | --- | --- | --- |
| G1 correctness | `115` publications accepted, `0` rejected, every set inside bounds and capacity; validation layer NOT MEASURED (no Khronos layer installed on the host) | `114/0` | n/a |
| G2 pass cost (GPU p95 / p99 / max, µs) | `320 / 348 / 350` PASS (`<= 2000`) | `281 / 304 / 309` PASS | pass absent |
| G3 coverage flips, frames 100..104 (max over pairs) | `0.195%` PASS (`<= 0.5%`) | `0.385%` PASS | n/a |
| G4 visibility (human) | jet continuous from the lip to the floor at frame 100 (~step 200) and 200 (~step 400); upper tank and lower pool visible at once; frame 50 (~step 100) shows the jet arriving | same | height field only, jet as a thin ribbon |
| G5 roots | catalog `406b6405c1...`, snapshot `c6f5a853bd...`, frame plan `8de30cdadb...` | identical | identical |

Whole-frame GPU p95: `383 µs` (`particles`), `458 µs` (`both`), `172 µs`
(`mesh`). Frame-critical CPU p95: `384 / 458 / 275 µs`. Stream pacing was
real time in every run (`1.92..2.00 s` stream per `1.92..2.00 s` wall,
`2..4` skipped frames of `117..121`); rendered frame `N` showed about step
`2 N`, not the `4 N` assumed in the plan (the adapter renders faster than
the 60 Hz stream on this display), so the G4 captures were taken at rendered
frames `50`, `100` and `200`.

Coverage flips alternate between `0` and about `0.2%` because a new
particle set arrives every second rendered frame; a pair with no new set
has no flip at all, so the pass itself is deterministic per set.

Device allocations: `78.7 MB / 35` with the pass (four screen targets, two
particle slots, two uniforms), `50.5 MB / 27` without. Particle uploads
`31.9 MB` per run (`12,000 x 12 B` per slot refresh).

## Apparatus corrections (recorded, not tuned)

- The composite stage writes alpha `0` on fluid pixels as the coverage
  channel for G3. The pass refuses to run unless the swapchain composite
  alpha is opaque. The first evidence PNGs were written with that alpha and
  looked white in viewers; the preview now writes the PNG opaque and
  measures coverage on the raw capture. No shading value changed.
- `--surface particles` publishes one zero-area placeholder triangle
  through the ring every frame instead of removing the declared object;
  the ring de-duplicates it by hash (`2` refreshes for `115`
  publications), and the preview's refresh bound was relaxed to that case.
- Developer capture became a burst (`DesktopFrameCaptureRequestV1
  { rendered_frame_index, frame_count }`, at most `8`) so G3 could be
  measured inside one run.

## Observations for the next revision (not acted on)

- Refraction through the tank reveals the particle lattice as horizontal
  stripes; the accumulated thickness is per-sphere, not per-volume, so the
  interior looks layered. A smoother thickness (larger thickness radius or
  a blurred thickness target) is the first candidate.
- The upper-tank free surface is flat and specular-free from this camera;
  Fresnel dominates at grazing angles as expected.
- The lower pool at ~step 400 splits into separate droplets; the spacing
  (`50 mm`) is coarse relative to the `35 mm` radius, so isolated particles
  read as beads. Anisotropic splats or a larger radius are the frozen
  follow-ups from the research note, to be decided in a new revision.

## Files

- Adapter: `crates/desktop-sdl-ash/src/particle_surface.rs`,
  `crates/desktop-sdl-ash/src/gpu_content/fluid.rs`, shader suite
  `crates/desktop-sdl-ash/shaders/fluid_*.{vert,frag,spv}` pinned in
  `manifest.json` / `PROVENANCE.md`.
- Bridge: `tools/xtask/src/water_preview.rs` (`--surface`,
  `--capture-frames`, `particle_surface` report block),
  `tools/xtask/src/water_stream.rs` (frame version 2 with particles).
- Research tool: `write_presentation_surface_stream_frame` appends the
  particle block (`GAME_SURFACE_STREAM_VERSION = 2`).
