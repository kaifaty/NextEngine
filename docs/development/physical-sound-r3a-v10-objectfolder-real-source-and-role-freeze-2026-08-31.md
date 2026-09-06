# Physical sound R3A V10 — ObjectFolder Real source and role freeze

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `A0_COMPLETE / ZERO_DECODE_REPEAT_PASS / READY_FOR_V9_REAL_FIT` |
| Target | ObjectFolder Real `60 / Beer_Glass / Glass` |
| Independent representation holdout | ObjectFolder Real `22 / Rinsing_Cup / Glass` from a different raw archive |
| Exact manifest | `8a30cef02684ddbc338d03a36e7cfab3dca2d5273e6d1d1fa885410e56048728` |
| Exact report | `3522c67729f543131688d676a9439c1a9f002fc21d4e168bfafac264af2e00d3` |
| Product effect | None; authored clips remain authoritative |

## Decision

Accept the official ObjectFolder contact-localization bundle as the bounded V9
real source and freeze its published train/validation/test roles before any
waveform sample is numerically decoded.

- Object `60` official train/val/test become V9 `fit/development/
  exact_object_query`.
- Object `22` official test becomes the archive- and object-disjoint
  `representation_holdout`; its other 25 contacts remain protected and unused.
- Only object-60 `fit` waveform decode is authorized next.
- Development, exact-object query, representation holdout, method holdout and
  admission shadow remain closed.

This closes Roadmap V10 A0. It gives no real-quality, contact-field, validator,
atlas, admission or runtime credit.

## Primary-source proof chain

The mapping is supported by three independent primary-source observations.

1. The official [ObjectFolder Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
   states that every impact recording is accompanied by the coordinate of the
   striking location and that tactile readings use the same 30–50 surface
   points. It lists object `60` as `Beer_Glass / Glass` and object `22` as
   `Rinsing_Cup / Glass`.
2. The official [CVPR 2023 paper](https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf)
   says that audio is recorded at selected surface points along their normal,
   touch is collected at the same surface points, and contact localization
   predicts the surface-location coordinate on the object mesh.
3. The official [contact-localization repository](https://github.com/objectfolder/contact-localization)
   at commit `4bb002f519cab9d250bbbe045a6df0248bf1639f` indexes
   `audio_spectrogram/<object>/<contact>.npy`,
   `contacts/<object>/<contact>.npy` and
   `global_gt_points/<object>.npy` with the same `(object, contact)` key. The
   contact loader returns the first three coordinate values as ground truth.

The published bundle independently confirms the key identity:

- object `60` has exactly audio IDs `0…29` and coordinate IDs `0…29`;
- object `22` has exactly audio IDs `0…29` and coordinate IDs `0…29`;
- both bind one finite `1024 x 3 float64` official point cloud;
- every selected WAV is PCM16 mono, `48,000 Hz`, `288,000` frames;
- processed `audio/91/18.wav` has SHA-256
  `4cfbfee343d0a336f48f22168645b847d388a05c13142fd4688a36c52b02b5db`,
  exactly matching the already frozen raw
  `91/audio/18/mic.wav` member from the V8 source revision.

The last control proves that the benchmark audio bundle preserves at least the
known raw microphone member byte-for-byte rather than transcoding it or
changing the contact ID.

## Exact external sources

Payloads remain outside Git.

| Artifact | Bytes | SHA-256 | Published identity |
| --- | ---: | --- | --- |
| `DATA_real/audio.tar.gz` | `463,486,373` | `14a15b96…a2c9` | Dropbox ETag `1721265166249979d` |
| `DATA_real/contacts.tar.gz` | `121,961` | `310c45e1…eeee` | Dropbox ETag `1721265168037676d` |
| `DATA_real/global_gt_points.tar.gz` | `2,359,020` | `29ebf37b…e51` | Dropbox ETag `1721265170138227d` |
| `DATA_real/split.json` | `19,258` | `77ea2d99…674e` | Dropbox ETag `1721265176279297d` |
| `DATA_real/scale.json` | `2,652` | `39c84eaf…0b8a` | Dropbox ETag `1721265174204746d` |

Raw provenance is kept separately from the compact benchmark bundle:

| Object | Raw archive | Bytes | ETag |
| --- | --- | ---: | --- |
| `60` | `audio_data_51_60.tar.gz` | `34,360,300,077` | `"63e36c0f-80008922d"` |
| `22` | `audio_data_21_30.tar.gz` | `37,503,322,690` | `"63e36d00-8bb5f4a42"` |

The raw archives are separate publication objects. The selected records still
share the ObjectFolder project and collection protocol; the holdout is not a
cross-dataset-project claim.

## Frozen roles

### Object 60 — Beer Glass

| Role | Contact IDs | Decode authority |
| --- | --- | --- |
| `fit` | `0,1,2,3,5,6,7,8,10,12,14,15,17,18,19,21,22,24,26,27,28,29` | Next step only |
| `development` | `4,11,16,23,25` | Closed until fit pass |
| `exact_object_query` | `9,13,20` | Closed until representation holdout pass and separate A2 freeze |

### Object 22 — Rinsing Cup

| Role | Contact IDs | Decode authority |
| --- | --- | --- |
| `representation_holdout` | `12,13,16,17,26` | Closed until object-60 development pass |
| `protected_unused` | Remaining `25` contacts | Closed |

These are the official published train/val/test splits, sorted only for
canonical serialization. No audio-derived feature, coordinate distance or
candidate result selected a role.

## Zero-decode inventory result

Two complete executions produce byte-identical `manifest.json` and
`report.json`:

- run A: `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v10-object60-source-inventory-run-a`;
- run B: `/home/kaifaty/.codex/experiments/nextengine/physical-sound/r3a-v10-object60-source-inventory-run-b`;
- manifest SHA-256: `8a30cef0…8728`;
- report SHA-256: `3522c677…00d3`;
- coordinates decoded as metadata: `60`;
- point clouds validated: `2`;
- committed audio records: `35`;
- WAV headers validated: `36`, including the raw-identity control;
- emitted selected audio payloads: `0`;
- decoded waveform sample values in every role: `0`.

The runner validates external location, complete archive size/hash, official
split and scale, contact/audio ID equality, finite NPY shape/dtype, PCM header,
role partition, selected member hashes and the processed/raw identity control.

## Honest capability boundary

Available:

- exact published contact label coordinates;
- exact object IDs and official point-cloud revisions;
- exact native-rate recorded microphone waveforms;
- material labels and separate raw archive provenance;
- an author-published split that does not depend on our generator.

Unavailable in the selected compact bundle:

- per-contact force waveform, despite its presence in the large raw source;
- contact normal;
- fixture/support state per recording;
- numeric microphone position and radiation condition;
- wall thickness or composition beyond `Glass`;
- a verified mesh-barycentric vertex index.

Therefore A1 tests native recorded-impact reconstruction with a direct impulse/
onset excitation assumption. It cannot claim force-conditioned response,
arbitrary listener radiation, exact support transfer or universal glass.
Coordinates are the official benchmark ground-truth labels. Geometry binding
is exact by object/revision and official loader key; this inventory does not
independently reconstruct a barycentric point on the full-resolution mesh.

## Rejected alternatives

- Object `51 / Fruit_Bowl` is not in the official 53-object contact-localization
  split, so the compact bundle cannot provide its coordinate/audio binding.
- Deriving a surface point from `finger_pose.yaml` was rejected because the
  pose-to-mesh transform and audio/tactile ID alignment were not explicit in
  the raw subset already inspected.
- Downloading the 44.8 GB aggregate Dropbox ZIP was rejected after the server
  ignored an HTTP range request. Direct published members provide the same
  metadata and bounded archives.
- Object `91` remains negative evidence and cannot select V9. It is used only
  for the pre-existing byte-identity control.
- Local microphone or hammer capture remains forbidden by product direction.

## Next action and rollback

Implement the separately frozen
[Beer Glass V9 real-fit protocol](physical-sound-r3a-v10-beer-glass-real-fit-protocol-2026-08-31.md)
and decode only the 22 object-60 fit recordings.

Rollback/non-regression rule: any source, role, PCM-header, contact-set,
coordinate, point-cloud or identity-control mismatch stops before sample
decode. A fit failure keeps every protected role closed and retains authored
clips.
