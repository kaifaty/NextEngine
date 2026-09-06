# Physical sound V24 X0 — disclosed Blue Bowl real pilot protocol

| Field | Value |
| --- | --- |
| Date frozen | `2026-09-01` |
| Status | `FROZEN_BEFORE_IMPLEMENTATION / EXISTING_DISCLOSED_VALUES_ONLY / SEALED_CONTACT_UNOPENED` |
| Roadmap package | V24 `X0` |
| Exact object | REALIMPACT `6_Bowl` / ObjectFolder `6 Blue_Bowl / Glass` |
| Product effect | None; disclosed development control only, never Metal admission |

## Question and bounded claim

Can the V24 evidence plane turn one already disclosed, cross-tier internet
object into honest real lane records without inventing force, support,
composition, listener or contact claims?

X0 reuses only previously opened development evidence. It cannot provide a
fresh holdout, validator calibration, Metal identity, arbitrary-force transfer
or release-quality claim. REALIMPACT supplies publisher-deconvolved transfer
responses; ObjectFolder supplies independent identified recordings of the same
physical object. Their signal semantics remain separate.

## Frozen predecessor identities

| Artifact | SHA-256 |
| --- | --- |
| REALIMPACT preflight manifest | `d9554d29c85fa601d72c6b35a0b126c038f2cf2338bc36d5d805da6505e7a6f9` |
| REALIMPACT preflight report | `a8c50ec3f6a41a2805eeb4752c56b878e2e4aff4c788f965afdde5bb552e6461` |
| Four-contact metadata | `c935a3fc8ef6cc43a0dafcea2b4aacf67d520ba08d17553193ccb3412c6e8457` |
| Four-contact array | `1971f01a202cbcbe36d7ac7f16c92b6d559f06a432eb948ae782c1ae43a11ba3` |
| Extraction report | `caf41093d51134332579af95d8e571c90225fcad4316dd2b50f1b79c9bd0bb24` |
| Exact REALIMPACT mesh bytes | `f23127b45b0b163c796b4ef44d707b59fac58c4ef6e94e7676a4d88470e7f973` |
| ObjectFolder identified report | `ade80e0fd6eab2c2ae1cc2a47720f573a358082dcfef9237d730e394ba89c882` |
| ObjectFolder source report | `993007be837df3baf7fea5a49d7603451a02f96a15cb24ca8b5927be8d1adc03` |

The implementation receives external paths plus these hashes and rejects drift
before parsing. It may bounded-refetch the exact REALIMPACT mesh from the
already frozen official archive identity; it may not fetch or decompress the
large transfer member again.

## Exact transfer rows

The source array is little-endian float32, shape `4 x 230215`, sample rate
`48 kHz`. It contains only these previously authorized canonical-listener rows:

| Impact | Row | Sample role | Mesh vertex | Position metres |
| ---: | ---: | --- | ---: | --- |
| `0` | `7` | context | `35950` | `[-0.03610739,-0.0520688,0.03753229]` |
| `1` | `607` | context | `21823` | `[0.00936602,-0.0155827,0.00078345]` |
| `2` | `1207` | context | `10221` | `[-0.05887816,-0.0508747,0.0790166]` |
| `3` | `1807` | query | `25307` | `[-0.04113073,-0.0616422,0.06258844]` |

All four rows use V3 lane `exact_real_transfer`, semantics
`force_deconvolved_transfer_response`, split role `development` and one
physical object group. The canonical listener is exactly publisher condition
`azimuth=0 degrees, distance offset=0 mm, microphone_id=7`; its published
coordinate must verify as `[0.23,-0.04345,0.0] m` before row emission.

Impact normals, raw force, impulse/energy, exact support and composition are
absent. Material family `Glass` is allowed only through the separately
hash-bound ObjectFolder cross-tier identity. Geometry and each impact/listener
point carry their own evidence. The builder emits one exact unmodified f32le
file per row; no alignment, crop, resample, denoise, gain match or normalization
is allowed in X0.

REALIMPACT contact `4`, row `2407`, mesh vertex `31104`, remains
`sealed_not_decompressed`. X0 checks only its metadata commitment and the
predecessor's zero decoded-sample count. It never requests or derives the row.

## Identified recording rows

The ObjectFolder demo contributes exactly three `recorded_impact_waveform`
rows, all in development and with no contact/listener/force/support claim:

| Recording | Sample role | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `000` | context | `675243` | `8efe186609fd47205fc935e1f0f7e1a14701f2c0744144afca1de904454a494c` |
| `020` | context | `695082` | `db03b0905df0708c5e19e7ccc703058949d2629faf93aee468ffa09e45666ac0` |
| `039` | query | `629585` | `50b126b12fc06e43dc6f6e600dc13c0b320fa7f80b3527e7b17580ca94f2a2aa` |

These rows use lane `identified_real_recording`. The source report grants only
object/material/real-recording/repeat identity. The MP4/MP3 container is kept
as exact source audio; decoding and canonical preprocessing belong to M0/V0,
not X0.

## Outputs and combined V3 boundary

X0 atomically emits to a fresh external directory:

```text
mesh.bin
transfer-row-000.f32le ... transfer-row-003.f32le
x0-lineage-report.json
x0-lane-records.json
report.json
```

The canonical `mesh.bin` uses T0's `NEMESH01` encoding after exact OBJ parse;
the report binds the original mesh hash and derived mesh hash separately.
`x0-lane-records.json` contains four exact-transfer and three identified-real
V3 row fragments. It is not a standalone V3 manifest.

After both T0 and X0 pass, one bounded assembler combines:

- T0 synthetic rows for all five roles;
- X0's disclosed development real rows;
- their typed lineage reports and exact artifacts.

Only that combined manifest runs through
`physical-sound-registry neural-data-plane`. This preserves D0's requirement
that a V3 manifest contain all three lanes and all five roles.

## Gates and stop rules

Two clean owning-entry executions must be byte-identical and prove:

- exact predecessor hashes and typed `Validated` lineage;
- exact array dtype/shape, finite values and four distinct row hashes;
- exact mesh vertex count `47738`, coordinate bounds and impact vertex matches;
- exact canonical listener tuple/coordinate;
- exactly `4` transfer plus `3` identified rows with declared sample roles;
- missing axes stay absent in the fragments and final projection;
- sealed row `2407` ID may appear only as a commitment, while its audio hash,
  samples and materialized row remain absent;
- combined V3 A/B passes the D0 command and keeps all authorities false.

Tests must corrupt every predecessor/artifact hash, dtype/shape, finiteness,
contact vertex/position, listener, material evidence, lane semantics, missing
axis, sealed-row access counter and output occupancy.

Any drift, missing cache artifact or mesh-refetch failure returns
`FallbackOutOfDomain` without substitution or larger transfer download. X0
`Pass` authorizes only a disclosed-real M0 development control. It does not
unseal contact `4`, any protected object, V0 calibration, Metal M1 or runtime.

