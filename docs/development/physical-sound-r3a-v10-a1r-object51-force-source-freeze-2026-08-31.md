# Physical sound R3A V10 A1R — object-51 force-source freeze

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `COMPLETE / ZERO_DECODE_REPEAT_PASS / FORCE_ONSET_FIT_NEXT` |
| Object | ObjectFolder Real `51 / Fruit_Bowl / Glass` |
| Product effect | None; authored clips remain authoritative |

## Result

A bounded `512 MiB` prefix of the official `51–60` raw archive exposes six
complete object-51 contacts with synchronized `mic.wav`, `Force.wav`, scalar
force metadata and recording metadata. The official compact
contact-localization bundle independently exposes the same six microphone WAVs,
their contact coordinates and the object point cloud.

All six raw microphone files are byte-identical to their processed counterparts.
This proves one exact key path:

```text
raw object/contact microphone + synchronized force
  == processed object/contact microphone
  -> published object/contact coordinate
  -> published object point cloud
```

No microphone or force sample value was decoded. Role selection used only the
first six complete contacts in raw archive order.

## Exact source identity

The official website repository is frozen at commit
`d058ba09e7e7a1a8358d64d7b48e5f588b377eb8`. Its exact download page is
`8,668` bytes, SHA-256
`2a4d4f43245dfdafd6f52a66d8c6f6e72822654b6e738f62899d34f1dda58f32`.
It identifies object `51` as `Fruit_Bowl / Glass` and states that recordings
carry strike coordinates and ground-truth contact-force profiles.

Raw archive:

- URL: `https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_51_60.tar.gz`;
- full length: `34,360,300,077` bytes;
- ETag: `"63e36c0f-80008922d"`;
- last modified: `Wed, 08 Feb 2023 09:31:59 GMT`;
- selected HTTP range: bytes `0–536870911`;
- prefix SHA-256:
  `f58e242af7b1b9f500b86e438f2c90ad6cb3f18d29914e1431c69c6a20c78584`.

The compact audio, contact, point-cloud, split and scale inputs retain the A0
hashes. Object `51` has `32` processed audio/coordinate keys (`0…31`), scale
`0.29946099617930483`, and is absent from the benchmark's published
train/val/test split. Therefore A1R roles are our preregistered archive-order
roles, not an attributed official split.

## Frozen roles

| Archive order | Contact | Role | Coordinate, metres |
| ---: | ---: | --- | --- |
| `1` | `27` | fit | `[-0.1052547353,-0.0787997242,-0.0202099895]` |
| `2` | `15` | fit | `[0.1224450372,0.0387423459,0.0289319656]` |
| `3` | `4` | fit | `[0.1040191698,-0.0021197407,-0.0255966478]` |
| `4` | `3` | fit | `[0.0855640519,-0.0803985220,-0.0201971053]` |
| `5` | `9` | development | `[0.0473010219,0.1354795909,0.0001078033]` |
| `6` | `18` | sealed | `[0.0212990322,0.1263078714,0.0222485541]` |

The point cloud is float64 `[1024,3]`, `24,704` bytes, SHA-256
`5c3b25eed3d92bbd85653e4e7540f539fa6bb80bbe71e834b5b9050f74b1d41d`.
Coordinates are official labels, not exact sampled-cloud vertices: nearest
sample distances are `14.4–20.5 mm`. No normal, support state, wall thickness,
composition or numeric microphone pose is published in the selected boundary.

## Reproducibility

Two external runs are byte-identical:

- manifest SHA-256:
  `3041c19d07f78fc9bdfd092ced72e07b5e795b235a286a23ede68108556b6ed5`;
- report SHA-256:
  `b421743e147132305aa0826f3dc2ded0c2a476151e9c2cad11f035995e917cb9`;
- selected raw members: `24`, `6,912,924` hashed bytes;
- raw/processed microphone identities: `6/6`;
- validated WAV headers: `18`;
- microphone/force sample values decoded: `0/0`;
- development/sealed sample values decoded: `0/0`.

Decision: `READY_FOR_A1R_FORCE_ONSET_FIT`.

## Evidence correction and scope

The same exact official page maps object `91` to `Solid_Spoon / Steel` and
object `93` to `Glass_Green / Glass`. Historical V8 documents called object
`91` Glass_Green. Their byte-level experiment remains a real-impact negative,
but its Glass material claim is not admissible under this exact official source
revision and must not select future Glass work.

Object `51` is physically object-disjoint from A1 object `60`, but it shares the
ObjectFolder project and the same `51–60` archive. It can independently test
force-derived synchronization and representation fit; it cannot serve as
project-disjoint validator, generalization or admission evidence.

Primary sources:

- [official ObjectFolder Real page at the frozen commit](https://github.com/objectfolder/objectfolder.github.io/blob/d058ba09e7e7a1a8358d64d7b48e5f588b377eb8/source/_pages/objectfolder-real-download.md)
- [ObjectFolder 2.0 paper](https://ai.stanford.edu/~rhgao/publications/ObjectFolder_CVPR2023.pdf)
- [official contact-localization benchmark](https://github.com/objectfolder/contact-localization/tree/4bb002f519cab9d250bbbe045a6df0248bf1639f)

## Next boundary

Only contacts `27/15/4/3` may decode microphone and force samples under the
separately frozen fit protocol. Development `9`, sealed `18`, the other 26
processed contacts and every later raw member remain closed. Prefix growth,
role changes or a signal-derived threshold require a new preregistered revision.
