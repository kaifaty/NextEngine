# REALIMPACT exact-weld geometry preflight V2 — PS-2 — 2026-08-28

## Outcome

The separately frozen exact-coordinate-weld V2 geometry preflight passes for
both still-sealed REALIMPACT objects. Manifest
`85ca065be72c7e282355fa15351d139f1d4a8a9cec80c73e1c097f08b1d1f46f`
changes only OBJ vertex assembly after V1's immutable triangle-soup rejection.
Two clean runs emit byte-identical report
`c1c86f782f5eb55dff292b0d799014a5bf8f8d324c138764d276b032736c7e5d`
and byte-identical geometry blocks:

- Pitcher `bcd54087f27167cbde16077479004deb6be2f8857cec5af8f3381e85e9539acc`;
- Planter `9310910f3d20ef2c865e6dcf502b2540d686dd71b43b13a4069ef43990745431`.

Decision: `ExactCoordinateWeldGeometryPreflightSupported`. Exactly `1308328`
compressed mesh bytes and zero reserved audio payload bytes are read. Success
does not authorize Pitcher audio: a separate calibration manifest must bind
these exact blocks, the frequency scale/mapping implementation and all existing
spatial gates first.

## Single frozen change

V1 report `2fb9fd0f…e25d` remains rejected. V2 groups vertices only when their
three parsed IEEE-754 binary64 coordinate encodings are exactly equal, orders
unique vertices lexicographically, remaps faces without reordering or winding
changes, and rejects degenerates/duplicates. Positional epsilon, quantization,
hole filling, repair, face deletion, normal repair, thickness inference and
audio access are prohibited.

Everything after welding remains V1:

- `fast-simplification 0.1.13` revision `4a193ef`, aggressiveness `7`;
- `8192`-face spectral and `2048`-face BEM targets;
- cotangent Laplace-Beltrami stiffness and barycentric lumped mass;
- 65 generalized eigenpairs, constant discarded, 64 modes retained;
- maximum relative eigen residual `1e-8`; and
- area-weighted collapse-replay field transfer with BEM-mass normalization.

The package ABI requires float32 points/int32 collapses for replay; replay is
used only to recover the deterministic index mapping. Authoritative reduced
points remain the float64 simplifier output, and replay must reproduce the BEM
face array and point count exactly.

## Results

| Metric | `65_PitcherCeramic` | `63_SmallPlanterCeramic` |
| --- | ---: | ---: |
| OBJ vertices → exact unique | `48418 → 8070` | `47732 → 7958` |
| Welded faces | 16140 | 15912 |
| Degenerate / duplicate faces | `0 / 0` | `0 / 0` |
| Welded components / boundary / nonmanifold edges | `1 / 0 / 0` | `1 / 0 / 0` |
| Spectral vertices / faces | `4096 / 8192` | `4098 / 8192` |
| BEM vertices / faces | `1024 / 2048` | `1026 / 2048` |
| Maximum relative eigen residual | `3.14076e-13` | `4.44576e-13` |
| Eigenvalue range, `m^-2` | `166.183 … 8273.015` | `120.935 … 7498.684` |
| Geometry block bytes | 3718246 | 3710422 |

Every welded-original, spectral and BEM topology gate passes. Every object has
64 modes and every eigen residual passes. Run A/B reports and both binary
blocks compare byte-identically.

## Claim boundary

This proves only deterministic geometry setup for the frozen candidate. The
surface eigenvectors remain a scalar biharmonic proxy, not elastic shell modes.
No wall thickness, support, material, measured acoustic transfer, naturalness,
domain admission, `Pass`, runtime role or ProductCheck is established.

Pitcher remains the sealed calibration object. Planter remains the sealed
one-shot holdout. Neither block, report nor mesh may enter the repository; only
their identities and bounded evidence are recorded here.

## Reproduction

```text
lab/.venv/bin/python \
  lab/scripts/physical_sound_realimpact_geometry_preflight_v2.py \
  --geometry-manifest <external-geometry-v2-manifest.json> \
  --output <external-run>
cmp <run-a>/report.json <run-b>/report.json
cmp <run-a>/65_PitcherCeramic-geometry-block-v2.bin \
    <run-b>/65_PitcherCeramic-geometry-block-v2.bin
cmp <run-a>/63_SmallPlanterCeramic-geometry-block-v2.bin \
    <run-b>/63_SmallPlanterCeramic-geometry-block-v2.bin
```

## Smallest next action

Before audio access, freeze a Pitcher-only calibration manifest that binds the
V2 report and Pitcher block, the exact 512 MiB compressed prefix, the 600-row
decoder, 16-mode extractor, spectral-scale dynamic program, Bempp/cooker
inputs, 90/510 listener split, RBF/constant controls, conjunctive gates and
stop-before-Planter fallback. Only then execute Pitcher calibration once.
