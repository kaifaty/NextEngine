# REALIMPACT geometry-spatial-transfer preregistration — PS-2 — 2026-08-28

## Outcome

The next real-data discriminator is frozen before either reserved acoustic
payload is opened. External manifest
`5be5f195ddc5b124e7220959efc9bb69c0bf469f495b2906e7da9f7515aae576`
binds the source evidence, exact archive and payload identities, object split,
geometry candidate, controls, listener split, metrics, gates, opening order and
fallback. Two repository-validator runs produce byte-identical report
`c2f51cffef3cac1e7b62de5b187fbd8b16e8fa89f0e8c9d755c4752405dd01ef`
and decision `RealSpatialTransferProtocolFrozen`.

The report records zero preregistration network requests and zero reserved
audio payload bytes read. This is protocol evidence only. It grants no real
spatial-transfer, material, quality, corpus-admission, runtime or ProductCheck
credit.

## Why this candidate

The prior coordinate-only, bounding-box/frequency-conditioned and compact
axisymmetric radiation candidates all failed frozen comparison gates. The
synthetic chain has since established an independent Bempp field oracle, a
non-spherical prescribed mode, one converged elastic FEM eigenmode and the
repository full-angular near-to-far cooker. The remaining falsifiable question
is whether an object-derived surface mode preserves useful real listener
transfer outside its fitting coordinates.

The two reserved objects are hollow ceramic vessels. Treating either scanned
surface as a filled linear-elastic solid would silently invent its wall volume.
The frozen candidate therefore uses a deliberately weaker geometry-spectral
proxy:

1. build the cotangent Laplace-Beltrami stiffness and lumped area mass matrices
   on a deterministic reduced surface;
2. solve the generalized eigenproblem and interpret the first 64 nonconstant
   eigenvectors as scalar normal-displacement candidates under a biharmonic
   frequency law;
3. use `65_PitcherCeramic` only to select one global spectral scale and an
   injective frequency mapping;
4. reuse that scale unchanged on `63_SmallPlanterCeramic`;
5. project each selected normal mode through pinned Bempp-cl and the already
   validated full-angular cooker; and
6. compare its held-coordinate relative magnitude to the frozen 3D RBF and
   normalization-listener constant controls.

This proxy is not called an elastic shell. It does not identify wall thickness,
support, elastic constants or material. Failure is useful: it would show that
geometry-spectrum plus measured frequency is insufficient and justify a real
shell formulation rather than another empirical RBF or harmonic-order grid.

## Frozen lineage

The manifest requires these exact prerequisites:

| Artifact | SHA-256 / required result |
| --- | --- |
| Frequency-conditioned REALIMPACT calibration manifest/report | `92ebe6a7…42f8` / `42b6605d…983`, rejected with ceramic holdout unopened |
| Compact modal-radiation manifest/report | `686c1d42…a0ff` / `7cbf7c59…8cf25`, rejected without fresh access |
| Refined elastic FEM→Bempp manifest/report | `1adfe3d6…abf7` / `a71515fa…a667`, every synthetic gate passes |
| Full-angular FEM-mode cooker manifest/report | `d4daf0f3…4cf1` / `c3101130…b476`, source gates pass, `m=0` rejects and a full-angular candidate selects |

The validator reads and hashes only those JSON prerequisites plus the
preregistration manifest. It contains no HTTP client, archive reader, range
fetch or NPY decoder. The reserved payloads are bound from already opened ZIP
metadata by archive URL/size/ETag/central hash, entry name/CRC32/offset/sizes,
sample count and required bounded prefix size; no payload digest is invented.

## Object-disjoint one-shot split

The two still-closed identities retain their prior roster hashes. Ascending
hash order selects the first for calibration and preserves the second for a
single holdout opening:

| Role | Object | Selection SHA-256 | Mesh SHA-256 | Audio CRC32 |
| --- | --- | --- | --- | --- |
| Calibration | `65_PitcherCeramic` | `65960d7f…b3e1d` | `8c94449d…f06c` | `53f05f2d` |
| Holdout | `63_SmallPlanterCeramic` | `66324101…fd13` | `b8ff190a…b71ca` | `6182e222` |

The opening order is conjunctive:

1. the preregistration validator must publish the exact report above;
2. a geometry-only preflight must repeat byte-identically before any
   calibration audio;
3. only a fully passing Pitcher calibration may authorize the Planter prefix;
4. Planter opens once; failure is immutable and cannot become development; and
5. every failure selects the declared per-mode RBF or whole-object clip
   fallback instead of weakening a threshold.

## Frozen 3D listener split

REALIMPACT publishes 600 listener positions per impact as ten azimuths, four
distance offsets and fifteen microphone heights. The first impact is used. The
normalization row is `angle=0, distance=0, micID=7`.

The 90 control anchors are the Cartesian product of:

- angles `0, 40, 80, 120, 160` degrees;
- distance offsets `0, 666` mm; and
- microphone IDs `0, 2, 4, 6, 7, 8, 10, 12, 14`.

All other 510 rows are held. Reports must expose three strata so aggregate
success cannot hide a failed axis:

- held height at an anchor angle and distance;
- held angle at an anchor distance; and
- held distance across every angle and microphone.

The candidate consumes no held listener value for spectral mapping or field
fitting. The RBF control may fit only the 90 declared anchors.

## Candidate and gates

Geometry preflight is intentionally before audio. The exact parser,
`fast-simplification 0.1.13` revision, `8192/2048` face targets, collapse
replay, topology checks, eigenvector sign rule and byte-repeat requirement are
manifest-bound. A non-closed/non-manifold reduction returns
`GeometryUnsupportedFallback`; it does not authorize a repair chosen after
seeing calibration sound.

At least 12 of 16 extracted modes must survive eigen residual, GMRES residual,
reference-magnitude and frequency-correspondence gates. Calibration frequency
error is limited to median/p90 `0.20/0.35` octaves; the unchanged-scale holdout
limits are `0.30/0.50`. Every held listener stratum must independently pass the
existing absolute condition gates: median at most `7 dB`, p90 at most `18 dB`,
persistent median at most `8 dB`, at least `0.5` improvement fraction over the
constant, and median-error ratio at most `0.9`.

Pitcher and Planter each must also beat the fixed coordinate RBF with median
error ratio at most `0.95`, p90 regression at most `2 dB` and improvement
fraction at least `0.5`. Reports must repeat byte-identically.

## Independent basis for the protocol

- The [REALIMPACT project](https://samuelpclarke.com/realimpact/), its
  [official repository](https://github.com/samuel-clarke/RealImpact) and the
  [CVPR 2023 paper](https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf)
  establish the measured listener/impact layout and warn that the real object
  sits on a supporting mesh; support identity is therefore prohibited.
- The [libigl spectral geometry tutorial](https://libigl.github.io/libigl-python-bindings/tut-chapter2/)
  documents the cotangent-Laplacian/generalized-mass eigenbasis used here as a
  surface representation, not as an elastic material oracle.
- [SciPy `eigsh`](https://docs.scipy.org/doc/scipy/reference/generated/scipy.sparse.linalg.eigsh.html)
  supplies the pinned sparse symmetric generalized eigenproblem interface.
- The [fast-simplification repository](https://github.com/pyvista/fast-simplification)
  documents deterministic collapse replay needed to transfer a spectral field
  to the BEM mesh; exact output hashes remain a mandatory preflight result.
- Pinned [Bempp-cl](https://github.com/bempp/bempp-cl) remains the independent
  Helmholtz boundary solver established by the preceding synthetic controls.

## Executed verification

```text
cargo test -p xtask realimpact_transfer_preregistration --no-fail-fast
cargo run -p xtask -- physical-sound-registry realimpact-transfer-preregister \
  --manifest <external-preregistration-manifest.json> --output <external-run-a>
cargo run -p xtask -- physical-sound-registry realimpact-transfer-preregister \
  --manifest <external-preregistration-manifest.json> --output <external-run-b>
cmp <external-run-a>/manifest.json <external-run-b>/manifest.json
cmp <external-run-a>/report.json <external-run-b>/report.json
```

Both comparisons pass. The focused tests also prove that prerequisite paths
cannot name the reserved payload, roles follow the frozen hash order and the
`90 + 510 = 600` listener partition is complete.

## Smallest next action

Implement the frozen geometry-only preflight. It may request only the two
hashed `transformed.obj` entries, must produce byte-identical mesh/collapse/
eigenmode/cooker-input reports twice, and must stop with the authored fallback
before any audio if either mesh fails the frozen topology or numeric gates.
