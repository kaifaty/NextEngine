# Nonlocal GPU presentation surface: small components — 2026-09-02

## Result and claim ceiling

`NGQ9 REV 1 RUN / G1 G2 G3 G4 PASS / ALL-COMPONENTS POLICY SELECTED FOR THE
LIVE PREVIEW`: keeping every 8-connected component of the raw wet mask
with at least `9` pixels (`--surface-components all`) retains `100%` of the
raw wet pixels on every frame of the narrow spill drain, is byte-equivalent
between the CPU and GPU extractors (`241/241` frames, identical raw and
closed masks and mesh counts, depth difference `6.5e-8 m`), keeps the NGQ6
closing gates at `0` failures and `0` ceiling violations, and costs the
same. The frozen NGQ5 largest-component policy dropped up to `32%` of the
wet pixels on the same run (minimum retention `0.683`, mean `0.855`) and,
on this multi-body scene, fails its own mask gate under `verify` (area
ratio `0.925`, coverage `0.911` at step `296`). The accepted corpus keeps
`largest`; only the live preview default changes. This is a
presentation-candidate finding under Proposed SPEC-38/ADR-100.

## Frozen inputs

```text
plan              docs/plans/nonlocal-gpu-full-step-performance/23-surface-small-components.md
lane              spill-narrow, --spill-lip flush, 960 steps, surface every 4, two density-only layers
threshold         GAME_SURFACE_MIN_COMPONENT_PIXELS = 9 (one sphere-cap footprint ~ 13 pixels)
extractor binary  0ab204fa7c0f03ef...
```

## Gates

| Gate | `all` | `largest` (reference) |
| --- | --- | --- |
| G1 CPU/GPU equivalence (`verify`) | `241` frames, `0/0/0` mismatches, depth `6.5e-8 m` | run stops at step `296` on the CPU mask gate (`74` frames equivalent before that) |
| G2 retention (min / mean) | `1.000 / 1.000` PASS | `0.683 / 0.855` |
| G3 closing gates | `0` failures, `0` ceiling violations, lift p50 `9.4 mm` | `0` failures (GPU path) |
| G4 capacity | within the box capacity | within |
| extraction per frame (GPU) | `1.55 ms` | `1.45 ms` |

Apparatus note: the CPU mask gate required exactly one closed component
(NGQ5, single-body corpus). Under `all` it now allows as many closed
components as raw components were retained (bodies may merge under the
close, never split); the corpus path is unchanged. The failure message
now carries the sub-gate values.

## Capture

`spill` (wide), flush lip, rendered frame `200`, solver step `104`, PNG
`7dd623dfa91f5513...`: the jet is continuous into the lower tank instead
of the broken fragments of the largest-only frame `196af8d3d0e2de56...`.

## Decision

- The live preview defaults to `--surface-components all`; `largest`
  remains available and stays the corpus path.
- ADR-100/SPEC-38 wording of the candidate surface path follows: "every
  8-connected component of at least one particle footprint" for the live
  candidate; the accepted corpus roots keep the largest-only stage.
