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

## Revision 2: spray split and thickness smoothing (user observation)

Plan 24 revision 2. The research tool appends one fluid-neighbour count
per particle (radius `0.1 m`) to every stream frame (version 3); the
adapter packs it next to the position (stride `16` B), skips particles
below `6` neighbours in the surface splat and draws them after the
composite as `12 mm` discs at alpha `0.35`; two separable Gaussian passes
smooth the thickness target with the depth filter's sigma.

```text
tool binary       5dbe75b2fd698514...
shader suite      fluid_surface, eight modules (manifest.json)
runs              particles x3, mesh x1; --capture-frame 100 --capture-frames 5, --capture-frame 200
```

| Measure | `particles` (3 runs) | `mesh` baseline |
| --- | --- | --- |
| pass GPU p95 / p99 / max | `487..492 / 491..496 / 494..504 µs` | n/a (`210 µs` whole frame) |
| whole-frame GPU p95 | `548..552 µs` | `210 µs` |
| coverage flips, max pair | `0.60..0.66%` | n/a |
| spray fraction max / last | `0.78..0.80% / 0` | n/a |
| rendered frame interval | `19..20 ms` | `19.8 ms` |
| device allocations | `91.8 MB / 36` | |
| roots | identical to revision 1 | identical |

G2 passes with a `0.17 ms` increase for the two thickness passes and the
spray pass. G3 fails under the frozen definition (`0.66% > 0.5%`), but
the rendered frame interval more than doubled against revision 1 in every
run of this session, including the baseline without the pass, so each
compared pair spans `2.3` stream frames instead of `1.0`; per stream frame
the flip fraction is `0.26..0.29%` (revision 1: `0.19%`). The pass itself
is unchanged between frames without a new set (`0` flips).

Look (captures at rendered frames 100 and 200): single splash particles
are soft dots instead of sphere contours; clusters of two to five
particles above the pool keep their contours because each member has at
least six neighbours; the refracted interior of the upper tank shows
fewer, softer bands. Candidates for a next revision, not applied: a
cluster-size criterion (connected component under a link distance) instead
of the neighbour count, and a larger depth-splat radius on the pool sheet.

## Revision 3: clusters, sub-droplets, streaks (user observation)

Plan 24 revision 3. Stream frame version 4 appends a 16-bit
connected-component size per particle (link `0.075 m`); the bridge
derives velocities from consecutive received frames; the adapter packs
position, flags and velocity (`32` B per particle), classifies spray by
cluster (`< 16`) or neighbours (`< 6`), draws `12` capsule sub-droplets
of `4 mm` per spray particle jittered inside the `35 mm` sphere and
stretched by `|v| / 60 s` (cap `0.15 m`) at alpha `0.5`, and grades the
surface splat radius from `0.6` at the spray threshold to `1.0` at `20`
neighbours.

```text
tool binary       b073cb990ef46080...
shader suite      fluid_surface, eight modules, 240-byte uniform
runs              particles x3 (frames 100..104), particles (frame 200), mesh x1
```

| Measure | `particles` (3 runs) | `mesh` baseline |
| --- | --- | --- |
| pass GPU p95 / max | `451..455 / 478..1073 µs` | n/a (`163 µs` whole frame) |
| whole-frame GPU p95 | `509..515 µs` | `163 µs` |
| coverage flips, max pair (raw) | `0.088..0.122%` | n/a |
| stream frames per publication | `1.03..1.07` | `1.08` |
| coverage flips per stream frame (G3n) | `0.085..0.118%` | n/a |
| spray fraction max / last | `0.78..0.82% / same` | n/a |
| rendered frame interval | `6.5 ms` | `6.7 ms` |
| device allocations | `85.1 MB / 36` | `50.5 MB / 27` |
| roots | identical to revision 1 | identical |

Look (captures at rendered frames 100 and 200, ~steps 150 and 300): the
jelly spheres are gone; single falling particles read as short white
streaks; the upper tank surface and the upper half of the jet are the
clear refracting surface. The jet tail and the thin, fragmented pool on
the lower floor are classified as spray by the cluster rule and render
as bright white streak clusters, a foam look rather than clear water.
The classification is correct under the frozen rule (those components
have fewer than sixteen particles); the look is a shading question for a
next revision: refracted-scene tint instead of white, fewer or fainter
sub-droplets, or a cluster rule restricted to airborne components.

Apparatus note: revision 2's raw flip gate depended on the presentation
rate (the interval varied `6.5..20 ms` across this session); the
normalised G3n figure is the comparable one from now on.

## Revision 4: anisotropic kernels, lonely shrink, cleanup (after Particles4All)

Plan 24 revision 4. The research tool appends per particle a smoothed
position and a symmetric kernel matrix (stream version 5): neighbours
within `0.1 m` with weight `1 - (r/R)^3`, Laplacian smoothing `0.9`,
covariance eigendecomposition, axes `max(s_i, s_1 / 4) * r / s_ref`
capped at `2 r`, and a lonely blend to an isotropic kernel of radius
`0.5 r` below `3` neighbours. The adapter splats the ellipsoid (view-ray
intersection for depth, chord for thickness) and runs a 2D narrow-range
cleanup of radius `4` px after the separable depth filter. Spray pass and
radius grading are off.

```text
tool binary       8c44c88c4438142e...
shader suite      fluid_surface, eight modules, 256-byte uniform, 64-byte particles
runs              particles x3 (frames 100..104), particles (frame 200), mesh x1
```

| Measure | `particles` (3 runs) | `mesh` baseline |
| --- | --- | --- |
| pass GPU p95 / p99 / max | `447..449 / 458..464 / 461..469 µs` | n/a (`163 µs` whole frame) |
| whole-frame GPU p95 | `503..506 µs` | `163 µs` |
| coverage flips per stream frame (G3n) | `0.087..0.103%` | n/a |
| producer presentation block mean / max | `15.5..15.8 / 17.0..18.7 ms` | `15.7 / 18.3 ms` |
| stream real-time ratio | `0.998..1.0` | `1.0` |
| particle upload per run | `131 MB` (64 B x 12,000 x 2 slots per publication) | n/a |
| device allocations | `89.3 MB / 36` | `50.5 MB / 27` |
| roots | identical to revision 1 | identical |

Look (captures at rendered frames 100 and 200, ~steps 150 and 300): the
upper tank reads as one smooth refracting surface, the jet as a
continuous ribbon from the lip to the floor, the lower pool as a thin
sheet with a few small droplets; the sphere beads of revisions 1-2 and
the foam streaks of revision 3 are gone. Faint lattice bands remain on
the tank surface at grazing angles.

Producer cost is the open item: `15.5 ms` mean per frame on one worker
is at the `16 ms` frame budget and the maximum exceeds it; real time
holds only because three extractor workers overlap. Candidates for a next
revision, not applied: compute the kernels on the device next to the
extractor (the solver already owns the neighbour lists), or compute them
only for particles with a free-surface neighbour deficit and send the
isotropic kernel for the rest.
