# Screen-space fluid prototype in the preview bridge — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ10` |
| Status | `RUN / G2 G3 G5 PASS / G4 HUMAN PASS / G1 PARTIAL` — evidence `docs/development/nonlocal-gpu-screen-space-fluid-evidence-2026-09-02.md` |
| Parent | ADR-102 (Proposed); research note `docs/development/water-rendering-research-2026-09-02.md` |
| Purpose | first screen-space fluid pass in the SDL3/Ash adapter, fed by the live particle stream of `water-preview` |

## Frozen scope

- Stream: the research tool publishes the fluid particle positions of every
  emitted frame next to the height-field surface (frame version 2; version
  1 readers keep working).
- Adapter: `ParticleSurfaceProfileV1`/`ParticleSurfaceUpdateV1` (ADR-102),
  one instanced sprite depth pass and one thickness pass at half
  resolution, a fixed narrow-range smoothing pass (radius from the
  projected particle size, capped), one composite pass over the swapchain
  image using an opaque scene-colour copy, all after the world pass and
  before the UI overlay. Shaders are a separate suite compiled with the
  host glslangValidator and pinned in the shader manifest with their own
  provenance.
- Preview: `--surface particles|mesh|both` selects the fed path; the
  height-field ring stays the default until the gates pass.

## Frozen gates (spill-narrow flush, 960 steps, 1920x1080, RTX 3080)

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 correctness | every published set inside bounds and capacity; rejected batches leave the prior set; zero validation errors from the Vulkan validation layer when enabled | pass |
| G2 cost | particle pass GPU time (depth + thickness + smoothing + composite) p95 from timestamp queries | `<= 2.0 ms` |
| G3 stability | with the fixed preview camera, the fraction of pixels whose fluid coverage flips between consecutive rendered frames, measured on captured frames 100..104 | `<= 0.5%` of the viewport |
| G4 visibility | captures at solver steps ~100 and ~400 show the jet as a continuous falling stream and both bodies at once | human review |
| G5 no root change | one catalog/snapshot/frame plan for the run; report hashes identical with and without the pass | pass |

## Frozen constants and apparatus (recorded before the first run)

- Particle profile: capacity `65,536`, radius `35 mm` (`0.7 x` the
  `50 mm` profile spacing), absorption `[1.2, 0.5, 0.25] /m`, refraction
  strength `0.08`, thickness scale `1.0`, bounds = solver box plus one
  surface pixel pitch (the same envelope the ring declares).
- Smoothing: two separable narrow-range passes, sigma `1.5 x` the projected
  radius clamped to `1..=24` px, low band `2 r`, high band `4 r`.
- G3 apparatus: the composite stage writes alpha `0` on fluid pixels; the
  swapchain is presented with opaque composite alpha (the pass falls back
  otherwise), so alpha in a developer capture is the coverage channel. The
  preview captures a burst of consecutive rendered frames
  (`--capture-frames 5` from `--capture-frame 100`) and reports the flip
  fraction of every consecutive pair.
- G5 apparatus: `--surface particles` collapses the ring update to one
  zero-area triangle instead of removing the declared object, so the
  catalog, snapshot and frame-plan roots are compared byte-for-byte across
  `mesh`, `particles` and `both`.
- Solver step to rendered frame mapping at stream rate `1.0` and the
  16.67 ms interactive pacing: rendered frame `N` shows about step `4 N`.
  G4 captures: rendered frame 25 (~step 100) and 100 (~step 400).

Do not tune radius, filter iterations or tint after seeing the results.

## Outcome (revision 1)

| Gate | Result |
| --- | --- |
| G1 | publications all accepted inside bounds/capacity; Vulkan validation layer not installed on the host, so that clause is `NOT MEASURED` |
| G2 | `320 µs` p95 (`particles`), `281 µs` (`both`) — PASS |
| G3 | `0.195%` (`particles`), `0.385%` (`both`) — PASS |
| G4 | human review of captures at rendered frames 50/100/200 (~steps 100/200/400): continuous jet, both bodies — PASS |
| G5 | roots identical across `mesh`, `particles`, `both` — PASS |

The step-to-frame mapping assumed above (`4 N`) was observed as about
`2 N`; the G4 captures were moved accordingly. The apparatus corrections are
listed in the evidence document. No shading constant was changed after the
first run.
