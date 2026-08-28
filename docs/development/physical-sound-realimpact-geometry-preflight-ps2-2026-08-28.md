# REALIMPACT geometry-only preflight — PS-2 — 2026-08-28

## Outcome

The first frozen geometry preflight is reproducibly rejected before any
reserved acoustic payload access. Two runs emit byte-identical report
`2fb9fd0f515146dd305901e65c88a9131af495d55c4dbf4c822f5e5b59cae25d`
with decision `GeometryOnlyPreflightRejected` and
`reserved_audio_payload_bytes_read = 0`.

The exact `transformed.obj` entry ranges are the only network reads: `634482`
compressed bytes for `65_PitcherCeramic` and `673846` for
`63_SmallPlanterCeramic`, `1308328` total. Both raw SHA-256 identities match
the preregistration. No geometry block, eigensolve, Bempp input, calibration
audio or holdout audio is published.

## Frozen V1 failure

The manifest-bound parser preserved each OBJ vertex index. Both published OBJ
files are triangle soup: their face corners are duplicated rather than shared.
The original topology therefore has nearly three used vertices per triangle:

| Object | Vertices | Faces | Components | Boundary edges |
| --- | ---: | ---: | ---: | ---: |
| `65_PitcherCeramic` | 48418 | 16140 | 16139 | 48418 |
| `63_SmallPlanterCeramic` | 47732 | 15912 | 15910 | 47732 |

Unmodified quadric simplification retains that disconnected structure.
Pitcher produces `8191/2048` components on the `8192/2048`-face meshes;
Planter produces the same. The topology gate requires one connected,
orientable, closed two-manifold with every edge incident to exactly two faces,
so both return `GeometryUnsupportedFallback`. Weakening that gate would feed
invalid surfaces to the cotangent operator and Bempp.

This is a setup rejection, not evidence against the geometry-spectral acoustic
hypothesis. It grants no real-transfer, material, quality, admission or runtime
credit.

## Post-rejection discriminator

After V1 was frozen and rejected, a metadata-only diagnostic tested exactly
one non-tunable explanation: duplicated OBJ positions. Exact bitwise coordinate
welding, with no tolerance and no audio access, yields:

| Object | Exact unique vertices | Degenerate faces | Components | Boundary / nonmanifold edges | Signed volume |
| --- | ---: | ---: | ---: | ---: | ---: |
| `65_PitcherCeramic` | 8070 | 0 | 1 | `0 / 0` | `0.0014100024113 m³` |
| `63_SmallPlanterCeramic` | 7958 | 0 | 1 | `0 / 0` | `0.0002857486532 m³` |

Both exact-weld meshes pass the previously frozen original-surface topology
predicate without hole filling, tolerance selection, remeshing or face repair.
This diagnostic is not retroactively part of V1 and cannot change its decision.
It provides the smallest falsifiable V2: preregister exact-coordinate welding
before simplification, retain every downstream target/gate unchanged, and
repeat geometry-only preflight before any Pitcher audio.

## Reproduction

```text
lab/.venv/bin/python \
  lab/scripts/physical_sound_realimpact_geometry_preflight.py \
  --manifest <external-preregistration-manifest.json> \
  --preregistration-report <external-preregistration-report.json> \
  --output <external-geometry-run>
```

The script validates the preregistration and source hashes, refuses any entry
other than the two hashed `transformed.obj` paths, requires HTTP `206` and the
exact `Content-Range`, bounds each response below 2 MB, verifies raw-deflate
size/hash, and reports zero reserved audio bytes. Run A and B reports compare
byte-identically.

## Smallest next action

Freeze a V2 geometry-preflight manifest that changes only OBJ vertex assembly
to exact bitwise coordinate welding before the existing `8192/2048` reduction.
Do not introduce a positional tolerance, topology repair, shell-thickness
guess, audio access or changed eigensolver/Bempp/spatial gate in that package.
