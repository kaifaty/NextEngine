# PS-2 REALIMPACT multi-listener acquisition pilot

Date: 2026-08-28

## Outcome

`physical-sound-registry realimpact-row --profile
green-goblet-listener-block-0-v1` now acquires and verifies one exact
15-microphone Green Goblet block from the published REALIMPACT archive. Two
complete runs are byte-identical and return
`MultiListenerAcquisitionPilotOnly`.

The pilot proves that source rows `0..14` are 15 distinct listener responses
for the same published impact vertex, azimuth and gantry distance. It does not
fit or validate a spatial model. It does not establish cross-object transfer,
absolute amplitude, glass identity, naturalness, exact-domain admission,
validator `Pass`, content authority or runtime readiness.

## Primary-source identity

The [official REALIMPACT project](https://samuelpclarke.com/realimpact/)
describes 15 microphones on a gantry, 10 azimuth angles and four distances for
600 listener locations per impact point. The frozen [official repository
revision](https://github.com/samuel-clarke/RealImpact/tree/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987)
provides the exact preprocessing lineage:

- `preprocess_measurements.py`, SHA-256
  `db55f2017a037fb50a7b8b59542e85e35a7c7d5581b15fb75b25dbdf67737bbc`,
  appends microphone recordings `1..15` inside each valid metadata row and
  records `(angle, distance, microphone index)` in that same loop;
- `preprocess_annotations.py`, SHA-256
  `66c4ab81d39a1ef4e26a72561dd2fa5238224f31ed089b27ca9a80d33852985a`,
  writes `angle.npy`, `distance.npy` and `micID.npy` with the same nesting;
- the [REALIMPACT paper](https://arxiv.org/abs/2306.09944) independently states
  that each object is struck at five points and measured at 600 field points.

The adapter does not infer row order from the paper or a filename. It verifies
the frozen scripts and the published arrays themselves.

## Exact block identity

The decoded `vertexID`, `vertexXYZ`, `angle`, `distance`, `micID` and
`listenerXYZ` arrays prove the following for every row in the block:

| Property | Frozen value |
| --- | --- |
| Dataset object | `93_GreenGoblet` |
| Source rows | `0..14` |
| Impact vertex | `31676` |
| Impact position | `[-0.02058199, -0.0394915, 0.1566662]` m |
| Azimuth | `0` degrees |
| Gantry distance offset | `0` mm; listener x is `0.23` m |
| Listener y | `-0.04345` m |
| Microphone IDs | `0..14`, exactly once each |
| Listener z | `-0.91, -0.78, -0.65, -0.52, -0.39, -0.26, -0.13, 0, 0.13, 0.26, 0.39, 0.52, 0.65, 0.78, 0.91` m |
| Audio shape | 15 rows × 208,323 mono float32 samples at 48 kHz |

All 15 row hashes are distinct. Row 0 retains its earlier SHA-256
`104dd97391bf6319ccbd4dfdf48569be58097bf1f90cf8cea3ae3cb2f7498ec9`;
the new manifest binds every remaining row hash, byte offset and listener
position.

## Bounded retrieval and deterministic outputs

The command binds the PS-2 corpus-plan report and the successful modal/damping
V2 report before acquisition. It validates the archive ETag, modification
time, complete central directory, ZIP entry metadata, six annotation arrays,
the mesh and a fixed 16 MiB compressed prefix of `deconvolved_0db.npy`.

| Artifact | SHA-256 |
| --- | --- |
| Frozen listener-block profile | `c2301789813b836b84ffb9a41380af02421490073d6585c488e276f1a5677e28` |
| Typed 15-row manifest | `fcf44d41bdd54ad3bc9df27b6c8ccc4d5ccd470a1f786641e64461e794c850de` |
| Acquisition report | `cef5d381f5a6d9a6a1666503a8ed7e44c70758cd4e6f9e636f8001a111d57680` |
| Concatenated raw float32 block | `8bcffd0a9f57fd101a803228f7e8aa66d469f82d99ad6e43950318aae0f875ca` |
| Provenance review | `9f59f1221b407fdd80e99b623a3b2555e369bcd7414b1d54a5e337a267f2cc64` |
| 16 MiB compressed prefix | `582ff65d41d7425886392694304e2fdb5f9e1b21cf4a6e289ed97481cf4049a0` |

The total HTTP range payload is `17,341,032` bytes, or
`0.007501426435283812` of the `2,311,697,935`-byte archive. Two final runs
produce byte-identical profile, manifest, report, provenance and block bytes.
The legacy Green Goblet row-0 report and manifest also remain byte-identical.

External results are stored under:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-green-goblet-listener-block-v1-final/`
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-green-goblet-listener-block-v1-repeat-final/`

The source data, block, reports and derived values remain outside the
repository.

## Claim boundary

Allowed credit is limited to:

- exact row-to-microphone identity;
- one same-impact/same-angle/same-distance published block;
- availability of a relative multi-listener development response.

The manifest explicitly prohibits absolute-amplitude credit, spatial-model
validation, cross-object generalization, exact material/support and matched
cross-tier claims, corpus admission, physical-sound `Pass` and runtime content
authority. Raw force, material-composition revision, repeat identity and a
versioned support fixture remain unavailable.

The strong variation in raw peak/RMS across microphones is only a diagnostic
that the block is non-degenerate. It is not itself a calibrated radiation
model or a quality score.

## Reproduction

```text
cargo run -p xtask -- physical-sound-registry realimpact-row \
  --profile green-goblet-listener-block-0-v1 \
  --source-bundle <frozen-external-source-directory> \
  --corpus-plan-report <frozen-external-corpus-plan-report.json> \
  --transfer-calibration-report <frozen-external-transfer-v2-report.json> \
  --output <new-empty-external-directory>
```

Focused controls verify the frozen 15-row profile, unique hashes and the ban on
spatial-validation credit. A semantic failure control rejects a changed V2
prerequisite even if its surrounding report shape remains valid.

## Decision and next action

The multi-listener data path is feasible, so `NotEvaluableSingleListenerRowPerObject`
is no longer an acquisition limitation for the development object. Spatial
participation remains unevaluated because there is no frozen multi-object
calibration/holdout experiment.

Before opening any additional listener responses:

1. freeze the exact Green Goblet development, Shell Plate calibration and
   Skull Cup spatial-holdout split;
2. declare candidate participation models, normalization, metrics, gates and
   exact bounded archive ranges;
3. acknowledge that row 0 of every object was already used by the separate
   modal/damping experiment while rows `1..14` of Shell Plate and Skull Cup
   remain unopened for spatial variation;
4. only then acquire Shell Plate for selection and open Skull Cup once for the
   spatial-response decision.

If the preregistered test cannot separate source participation from residual
microphone/gain effects, retain the 15-row blocks as E2 observations and keep
spatial participation unavailable. The eight exact-domain blockers, PS-3,
AV-P0D and production P1 remain unchanged.
