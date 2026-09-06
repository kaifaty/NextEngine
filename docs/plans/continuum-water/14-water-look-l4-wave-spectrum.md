# Water look L4 — wave spectrum on the ring

| Field | Value |
| --- | --- |
| Research ID | `WL4` |
| Status | `RUN / G1, G2, G4 PASS / G3 HUMAN` (2026-09-03) |
| Parent | plan `continuum-water/11` item L4; plan 09 (WP1, the stage this revises); plan 13 (the water pass that shades it); SPEC-38 practice 5 (waves are presentation only) |
| Purpose | the surface stops being a plane: a small always-present wave spectrum in world coordinates plus the flux-driven ripple of WP1, and an animated detail normal in the water pass, so refraction wobbles and the sun glints; the stage stays a pure function of the checkpoint and the frame index |

## Frozen scope

- **Stage (plan 09 revision).** `compute_water_presentation_frame` keeps
  its grid, capacities and jet. The height of a vertex becomes the sum of
  an ambient spectrum and the WP1 flux ripple, clamped to the unchanged
  `20 mm` cap:
  - ambient: four world-space directional sine waves (wavelengths `1.6`,
    `0.9`, `0.5`, `0.3 m`; directions `20`, `110`, `200`, `305` degrees;
    periods from deep-water dispersion `T = sqrt(2 pi lambda / g)` at
    `g = 9.81`: `1.01`, `0.76`, `0.57`, `0.44 s`; amplitudes `2.0`, `1.2`,
    `0.5`, `0.3 mm`) evaluated with the integer `sin_q15` on world
    micrometres, so the pattern is continuous across quads and independent
    of the grid;
  - flux ripple: the WP1 amplitude rule (`20 mm` at `1 L/tick` of incident
    flux) over the two shortest ambient waves at double frequency.
  Normals stay finite differences of the height on the grid.
- **Detail normal (water pass).** The water uniform's spare lane carries
  the run time in seconds (rendered frame index over `60`); the fragment
  perturbs the ring normal by two world-space sine gradients (wavelengths
  `0.18` and `0.11 m`, speeds `0.35` and `0.5 m/s`, normal offset `0.03`)
  before the Fresnel, refraction and glint terms. The WL1 fallback suite
  is unchanged.
- **Verification.** `xtask water-present` (roots identical with and
  without the stage, purity, capacities, `max_ripple <= 20 mm`, cost);
  release capture from `--start-at-water`.

## Frozen gates

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 roots and purity | `water-present` PASS: roots identical every tick, byte-identical recomputation, ripple within the cap | pass |
| G2 cost | stage cost per frame, release, from the `water-present` report | `<= 1.0 ms` (the plan 09 budget) |
| G3 look (human) | the `--start-at-water` capture shows a visibly non-flat surface with moving glints and a wobbling refracted floor | human |
| G4 frame cost | frames per tick in the release capture run | `>= 5.0` |

Do not change the wavelengths, amplitudes, directions or the detail
offset after seeing the captures; a change is a new revision with its
own capture.

## Result (WL4, 2026-09-03)

| Gate | Result |
| --- | --- |
| G1 roots and purity | `cargo run --release -p xtask -- water-present` PASS: roots identical every tick with and without the stage, byte-identical recomputation, `max_ripple 20,000 um` (the clamp reaches the negative cap; the positive side stays one micrometre below the exclusive catalog bound) — PASS |
| G2 cost | `104 us` maximum, `74 us` mean per frame in release (the first implementation measured `1,045 us`: `i128` divisions and five height evaluations per vertex; the apparatus was corrected to one `i64` evaluation per grid vertex with the normals taken from the height buffer, same integer results) — PASS |
| G3 look (human) | the `--start-at-water` capture (`wl4-590.png`) shows the basin and the far vessels with a visible wave pattern instead of a plane; glints depend on the camera — HUMAN |
| G4 frame cost | `600` frames over `112` ticks (`5.4` per tick) — PASS |

Apparatus (recorded): the scripted-input timestamps of `--start-at-water`
now come from SDL's nanosecond tick clock; small synthetic timestamps had
tripped the normalizer's monotonic timebase check in one run.
