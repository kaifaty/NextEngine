# Physical Sound R3A V12-C3 object-41 source/role freeze — exact result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `READY_FOR_C4_ESTIMATOR_FIT` |
| Target | ObjectFolder-Real `41 / Wrench_Large / Steel` |
| Claim | Fresh exact object, canonical recorded setup, normalized instrument counts |
| Protocol | [C3 protocol](physical-sound-r3a-v12-c3-object41-source-role-freeze-protocol-2026-08-31.md) |
| Next authorized step | `C4_ESTIMATOR_FIT_PROTOCOL` |
| Product authority | None; authored clips remain authoritative |

## Claim

C3 passes twice byte-for-byte. The frozen `4 GiB` raw prefix contains all `35`
object-41 trial quartets. Every raw microphone has a paired force channel and is
byte-identical to the compact microphone joined to the same published contact
coordinate. Geometry, scale, object identity and six signal-blind roles are
hash-closed before any microphone or force sample value is decoded.

This result authorizes only a C4 fit **protocol**. Every role remains decode-
closed in C3. It gives no modal-fit, real-quality, material-family, ML,
validator, atlas or runtime credit.

## Frozen identity

- Implementation commit: `1a034396`.
- Protocol SHA-256:
  `d8cd32fc127a26d375e234e2abfede4c265624a62cef6c985b3be3b1e25f9560`.
- Runner SHA-256:
  `4531cce43fb5b2b56f41817b19a9b6f3a982e6bd6d1ef888a3a81e2350cb0fad`.
- Compact-inventory dependency SHA-256:
  `ec3c916cd04baf028d7474cdbf392d856705f5db6e210f0d82a79e5ee17555ff`.
- Freeze manifest SHA-256:
  `a6dd159e417624f4db087cfc7289e72ba985a4fde6f88167bcc51a7277a766b3`.
- Repeated zero-read preflight report SHA-256:
  `26f994190d4d04918806a065554f055deed11f897587eb7010c2df223dbdad89`.
- Repeated inventory report SHA-256:
  `a300199fe2c327694483c5613415243eecb897977c2ef3a7da77e1c6b92220d7`.

Raw prefix:

- URL: `https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_41_50.tar.gz`;
- inclusive range: `bytes=0-4294967295`;
- bytes: `4,294,967,296`;
- SHA-256:
  `93ad4484e39a511074cfc48861a3206c9e098df1527569f62ca44c3bf8ab55eb`;
- full archive identity: `40,775,630,586` bytes, ETag
  `"63e36ee1-97e6abefa"`, Last-Modified
  `Wed, 08 Feb 2023 09:44:01 GMT`.

The prefix is an external evidence artifact and is not in Git.

## Six immutable roles

The committed SHA-256 rank partition is complete and contact-parent-disjoint:

| Role | Count | Contact IDs in frozen hash order |
| --- | ---: | --- |
| `estimator_fit` | `16` | `30,13,24,14,19,9,6,15,5,33,2,29,4,16,31,34` |
| `generator_development` | `5` | `23,32,18,0,10` |
| `representation_holdout` | `4` | `1,25,17,21` |
| `validator_calibration` | `4` | `3,12,26,11` |
| `validator_method_holdout` | `3` | `28,22,8` |
| `admission_shadow` | `3` | `27,7,20` |

Role-order root:
`235e418fb62febf3673af2606f7a9efb3b856ac67526bda0ae6464834abe0620`.
Object 41 is absent from the benchmark's official split, so no official
train/val/test claim is made. Role assignment used only object/contact IDs and
the committed salt, not coordinates, force metadata, PCM or results.

## Inventory checks

| Check | Result |
| --- | --- |
| Raw trial quartets | `35/35`, IDs `0…34` |
| Paired force/microphone headers | `35/35` |
| Raw/compact microphone byte identity | `35/35` |
| Header contract | PCM16 mono, `48,000 Hz`, `288,000` frames |
| Coordinate keys | `35/35`, finite `(3,)` |
| Point cloud | finite float64 `(1024,3)`, SHA-256 `3879499c…5f11` |
| Scale | exact `0.25329818850723695` |
| Role partition | six nonempty, disjoint, complete |
| Network requests during runner | `0` |
| Microphone/force samples decoded | `0 / 0` globally and in every role |
| Selected payloads emitted | `0` |

The point-cloud bounds are
`[-0.133685,-0.033388,-0.008957]…[0.119895,0.034511,0.008590] m`.

## Read accounting

- Raw prefix bytes hash-verified: `4,294,967,296`.
- Compressed bytes consumed before all object-41 members were found:
  `3,929,210,880`.
- Selected raw member bytes hashed: `40,325,383`.
- Compact microphone bytes hashed: `20,161,540`.
- Coordinate metadata values decoded: `105`.
- Point-cloud metadata values decoded: `3,072`.
- Microphone sample values decoded: `0`.
- Force sample values decoded: `0`.

Hashing a selected WAV proves identity and pairing but grants no numeric signal
access. The importer parses only its container header.

## Report-only geometry diagnostic

After the partition was immutable, a metadata-only diagnostic measured each
protected contact's nearest distance to a fit contact:

| Role | Mean nearest fit distance | Maximum |
| --- | ---: | ---: |
| Generator development | `23.58 mm` | `37.72 mm` |
| Representation holdout | `25.19 mm` | `35.78 mm` |
| Validator calibration | `21.70 mm` | `31.62 mm` |
| Validator method holdout | `25.66 mm` | `30.24 mm` |
| Admission shadow | `21.85 mm` | `26.06 mm` |

This is not a gate and did not alter roles. It records the actual spatial
extrapolation burden that later contact-field work must face.

## Reproducibility

Both zero-read preflights are byte-identical. Inventory A and B independently
rescan the same hash-closed external inputs and produce byte-identical
`report.json`.

External roots:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c3-object41-freeze`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c3-object41-preflight-a` and `-b`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c3-object41-inventory-a` and `-b`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v12-c3-object41-source-research`.

No dataset, prefix, waveform, array or generated report is committed.

## Decision and remaining risk

C3 is complete. C4 may now preregister a fit-only experiment for the 16
`estimator_fit` contacts. That protocol must freeze force preprocessing,
acquisition-coverage/OOD rules, common-pole estimator, raw-H1 and other honest
controls, thresholds and exact sample budgets before the first signal decode.

Generator development, representation holdout, both validator roles and
admission shadow remain sealed. Numeric listener pose, exact per-object support
and SI calibration are absent, so the maximum admissible claim remains one
canonical recording setup in normalized instrument counts. Authored clips stay
mandatory for every reject, OOD or fault.
