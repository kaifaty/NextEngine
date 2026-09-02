# Presentation surface: small components — revision 1

| Field | Value |
| --- | --- |
| Research ID | `NGQ9` |
| Status | `RUN / G1 G2 G3 G4 PASS / ALL SELECTED FOR THE PREVIEW` (evidence: `docs/development/nonlocal-gpu-surface-small-components-evidence-2026-09-02.md`) |
| Parent | NGQ5 frozen extraction (largest 8-connected component), NGQ8 spill scene |
| Observation | the user sees small bodies of water flicker or vanish, for example while the lower tank starts to fill: the frozen extractor keeps only the largest connected component of the wet mask, so a second body is dropped until it outgrows the first, and then the first vanishes |

## Frozen change

`--surface-components all` keeps every 8-connected component of the raw
wet mask whose pixel count is at least `9` (one sphere-cap footprint is
about `13` pixels at the `12.5 mm` pitch); `largest` keeps the NGQ5
behaviour and remains the corpus path (`extract_presentation_surface`
defaults to it, so every accepted root is unchanged). Closing, fill,
5x5 closing and bilateral stages are untouched and run on the retained
mask as before. The live preview defaults to `all`.

## Frozen gates (spill-narrow flush, 960 steps, surface every 4)

| Gate | Definition | Pass |
| --- | --- | --- |
| G1 equivalence | `--extractor verify` under `all`: identical raw and closed masks and mesh counts on every frame, depth difference `<= 1e-6 m` | pass |
| G2 retention | minimum over frames of retained raw wet pixels divided by raw wet pixels, `all` versus `largest` | `all >= 0.95`; `largest` reported |
| G3 closing gates | NGQ6 lift/ceiling gates on the retained mask | `0` gate failures, `0` ceiling violations |
| G4 capacity | mesh vertices and indices within the preview capacity for the box | pass |
| cost | extraction per frame p95 | report |

Do not tune the pixel threshold after seeing the results.

## Result

`all`: retention `1.000` minimum, CPU/GPU equivalent on `241/241` frames
(depth `6.5e-8 m`), `0` closing gate failures, same cost. `largest`:
retention `0.683` minimum, and the CPU mask gate fails on this multi-body
scene at step `296` (area ratio `0.925`). One apparatus correction before
the final run: the CPU mask gate allows as many closed components as raw
components were retained under `all`; the corpus path is unchanged.
