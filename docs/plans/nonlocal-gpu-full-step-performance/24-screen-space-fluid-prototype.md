# Screen-space fluid prototype in the preview bridge — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ10` |
| Status | `REV 1-2 RUN / G2 G5 PASS / G3 REV 2 FAIL UNDER A FRAME-RATE CONFOUND / G7 SPRAY 0.8% / G8 PARTIAL` — evidence `docs/development/nonlocal-gpu-screen-space-fluid-evidence-2026-09-02.md` |
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

## Revision 2 (frozen before running): spray split and thickness smoothing

User observation: isolated splash particles above the pool surface render
as separate sphere contours ("bubbles"), and the particle lattice shows
through the refracted interior as stripes.

Frozen changes:

- Stream frame version 3 appends one fluid-neighbour count per particle
  (neighbours within `0.1 m`, two spacings, self excluded, from a hash
  grid over the emitted positions; the solver's own neighbour lists are
  not used).
- `ParticleSurfaceUpdateV1` carries the counts (packed stride `16` B); the
  profile gains `spray_neighbour_threshold`, `spray_radius_micrometres`
  and `spray_alpha`.
- Surface splat skips particles with fewer neighbours than the threshold;
  a spray pass after the composite draws them as soft discs
  (alpha-blended RGB, depth-tested, swapchain alpha untouched so the G3
  coverage channel stays valid).
- Two separable Gaussian passes smooth the thickness target with the same
  projected-radius sigma as the depth filter, ignoring samples outside the
  silhouette; the composite reads the smoothed thickness.
- Frozen constants: threshold `6` neighbours, spray radius `12 mm`, spray
  alpha `0.35`; every revision-1 constant unchanged.

Gates (spill-narrow flush, 960 steps, 1920x1080, same runs as revision 1):

| Gate | Definition | Pass |
| --- | --- | --- |
| G2 cost | particle pass GPU p95 including spray and thickness passes | `<= 2.0 ms` |
| G3 stability | coverage flips on captured frames 100..104 | `<= 0.5%` |
| G5 roots | catalog/snapshot/frame plan identical to revision 1 | pass |
| G7 spray fraction | particles below the threshold per publication, maximum and last | report, expected `< 5%` |
| G8 look (human) | no isolated sphere contours over the pool surface at frames 100 and 200; interior stripes visibly reduced against the revision-1 captures | human |

Do not change the threshold, spray radius, alpha or the thickness sigma
after seeing the results.

## Revision 2 result

Three `particles` runs and one `mesh` baseline on `spill-narrow`
(`960` steps, `1920x1080`):

| Gate | Result |
| --- | --- |
| G2 | `487..492 µs` p95, `504 µs` max with spray and thickness passes — PASS |
| G3 | `0.60..0.66%` on the pairs with a new set (`0` on the others) — FAIL under the frozen definition |
| G5 | roots identical to revision 1 — PASS |
| G7 | spray fraction maximum `0.78..0.80%`, `0` at the end of the drain — reported |
| G8 | single splash particles now render as soft dots; small clusters of two to five particles above the pool still show sphere contours; interior stripes are reduced but visible at grazing angles — PARTIAL |

Confound recorded, not corrected: the rendered frame interval in every
revision-2 run was `19..20 ms` (also for the `mesh` baseline without the
pass, `210 µs` GPU), against `8.5 ms` in the revision-1 runs, so each new
particle set was `2.3` stream frames apart instead of `1.0`. Normalised
per stream frame the flip fraction is `0.26..0.29%` against `0.19%` in
revision 1. The G3 definition ("consecutive rendered frames") ties the
gate to the presentation rate; a frame-rate-independent apparatus
(flips between consecutive published sets) is a new revision, not a
re-reading of this one.

No constant was changed after the first run.
