# Physical sound R3A V8 — ObjectFolder Real source and gate freeze

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / ZERO_WAVEFORM_DECODE_INVENTORY_NEXT / FIT_CLOSED` |
| Parent result | [V8 synthetic preflight pass](physical-sound-r3a-v8-synthetic-preflight-result-2026-08-31.md) |
| Candidate source | ObjectFolder Real object `91`, `Glass_Green`, published material `Glass` |
| Product effect | None; clips remain authoritative and R3B/runtime remain closed |

## Authorized question

Can a bounded explicit-modal representation fit new real glass impacts at the
source's fixed microphone condition before any contact interpolation is
attempted?

This revision freezes the source, contact roles and evaluation order. It does
not yet decode a waveform. The first implementation step may only hash selected
archive members and validate their RIFF headers. Only an exact inventory pass
may authorize numerical decoding of the three fit contacts.

## Primary source and archive identity

The official [ObjectFolder Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
states that every object has `30–50` six-second impact recordings, strike
locations on the mesh and force profiles. It lists object `91` as
`Glass_Green / Glass` and publishes this archive:

`https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_91_100.tar.gz`

Observed HTTP identity before role freeze:

| Field | Value |
| --- | --- |
| Full archive bytes | `38,866,981,268` |
| ETag | `"63e3844c-90ca71194"` |
| Last-Modified | `Wed, 08 Feb 2023 11:15:24 GMT` |
| Range support | `bytes` |

The official archive is a single very large gzip stream, so a complete archive
hash would require downloading all ten objects. The bounded experiment instead
binds an exact prefix and exact selected-member hashes:

| Prefix field | Value |
| --- | --- |
| HTTP range | bytes `0–536870911` inclusive |
| Prefix bytes | `536,870,912` |
| Prefix SHA-256 | `5ef9789ad023233377aa60d58f66100f0bb62dd5df52556e051e757d13797313` |

This prefix contains six object-91 contact directories in archive order. The
first five complete contact records are frozen below. Selection used only tar
member names/order and the official object label; no PCM sample, spectrum,
force value or perceptual result participated.

## Frozen contact roles

| Archive order | Contact | Role | Allowed access after inventory |
| ---: | ---: | --- | --- |
| 1 | `18` | `fit` | Decode only after exact inventory pass |
| 2 | `12` | `fit` | Decode only after exact inventory pass |
| 3 | `4` | `fit` | Decode only after exact inventory pass |
| 4 | `20` | `development` | Header/hash only until fit gate passes |
| 5 | `27` | `sealed` | Commitment only; never decode in this representation revision |

Contact `9` is visible later in the prefix but has no role and must remain
unused. Existing Blue Bowl, Large Swan, Plastic Bin and Purple Scoop
development evidence cannot select this V8 revision.

For every selected contact, the inventory expects exactly:

- `mic.wav` — recorded impact waveform;
- `Force.wav` — measured force-profile waveform;
- `striking_force.yaml` — published scalar force metadata;
- `metadata.yaml` — recording gain/padding metadata.

The bounded prefix does not expose a contact-coordinate artifact. Although the
official dataset description claims coordinates, this revision records that
axis as unavailable. Object `91` may test representation fit/development but
cannot authorize a contact-position field until an exact coordinate binding is
retrieved from an official or official-benchmark source.

## Expected member commitments

The inventory implementation must independently recompute these values from the
prefix. It emits no member payload.

| Contact | Member | Bytes | SHA-256 |
| ---: | --- | ---: | --- |
| 18 | `Force.wav` | `576044` | `69429fa37362c6a46aff4fc0f7312375fc56e68a16d3e96ce518620e666d77a9` |
| 18 | `mic.wav` | `576044` | `4cfbfee343d0a336f48f22168645b847d388a05c13142fd4688a36c52b02b5db` |
| 18 | `striking_force.yaml` | `30` | `934ea4934f5f7ffd7391569fea137af480f0c115a7a7f7552032bae69d1707d7` |
| 18 | `metadata.yaml` | `35` | `8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd` |
| 12 | `Force.wav` | `576044` | `e0cbb5421c62ee782bae47b8eb62db5cc1e7841e907d8bd1dc82bff17d30f6df` |
| 12 | `mic.wav` | `576044` | `53d8bf10c9dba1b1ad13b441c45442b3a1ec4c8ecfb83c884b601677487df73d` |
| 12 | `striking_force.yaml` | `31` | `082fb8338864144915f97e43e97fbba2a81b24de91618c4956bb01b3f56f9ca4` |
| 12 | `metadata.yaml` | `35` | `8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd` |
| 4 | `Force.wav` | `576044` | `cd2260da461f4caaa4e44135e1b3372a19fc4ebcc7a4bb39dd21045d181a808e` |
| 4 | `mic.wav` | `576044` | `55ec5ea105c449bb9e5851e221ec114ac17d8f889f1b44d7e9df7446d997ee39` |
| 4 | `striking_force.yaml` | `31` | `45114ff2f7a6ab759339465f02ec1a69717e19af39fb0576fa32d5ff0f1bdcaf` |
| 4 | `metadata.yaml` | `35` | `8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd` |
| 20 | `Force.wav` | `576044` | `9a59e098345d8d422e0848030f5fb70243e983a69fbb76a2fc75027eb66c2abd` |
| 20 | `mic.wav` | `576044` | `5197764481d296f7162b05a95ba5074b4ef4eebe858f7fbd489dfdad5ae03aee` |
| 20 | `striking_force.yaml` | `31` | `7461c09a89991e57b853568a96ec7ddbe8e612907f843930e9ceabc091c712b9` |
| 20 | `metadata.yaml` | `35` | `8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd` |
| 27 | `Force.wav` | `576044` | `f7d21df0a54d46a0be23615181e8310669123d641ddb4d6f086a8f8906a3ccaf` |
| 27 | `mic.wav` | `576044` | `321453de9634d2e2a15a346a69d3a5c1f3233b5bbf5b1da53b8d8dcc4a5332e5` |
| 27 | `striking_force.yaml` | `31` | `b6cc4d23de4b920fd67b72a13ac0c680d15782ec1904b7b8d3ffd0460df1089a` |
| 27 | `metadata.yaml` | `35` | `8d9867990f1c05870ef713f714f70a512c18833de55ffeb312efb473b5e3d2fd` |

Every WAV header must be PCM16 mono, `48,000 Hz`, `288,000` frames. Header
inspection is permitted; reading numerical sample frames is not.

## Zero-decode inventory gate

The inventory returns `READY_FOR_V8_REAL_FIT_EXTRACTION` only if:

1. prefix byte count and SHA-256 match exactly;
2. every selected member appears exactly once with regular-file type, exact
   byte count and SHA-256;
3. all ten WAV headers match the frozen PCM boundary;
4. no unselected member content is hashed or emitted;
5. waveform samples decoded, fit samples decoded, development samples decoded
   and sealed samples decoded are all zero;
6. no member payload, audio feature, model weight or WAV is written to Git or
   to the inventory output;
7. two runs produce byte-identical manifest/report JSON.

Failure returns `STOP_V8_REAL_SOURCE_MISMATCH` and does not authorize fit
extraction.

## Frozen fit/development order

Only an inventory pass may implement this later sequence:

1. Extract and numerically decode contacts `18`, `12` and `4` only.
2. Use `Force.wav` as an explicit measured excitation input. `mic.wav` remains
   `recorded_impact_waveform`; it is never mislabeled as a transfer response.
3. Align onset from the force channel under one frozen threshold and retain the
   native `48 kHz` rate; no resampling is permitted.
4. Initialize object-global modal frequencies from fit-only multiresolution
   spectra and damping from fit-only band-envelope regressions.
5. Compare modal-only with modal plus one deterministic filtered residual.
   The only permitted capacity successor is one preregistered per-contact
   damping ablation; no post-development capacity search is allowed.
6. Require every fit contact to pass byte cost, finiteness, level, spectrum,
   envelope, decay and modal-frequency gates before contact `20` is decoded.
7. If fit passes, freeze all artifacts and evaluate development contact `20`
   once against nearest-fit, modal-only and residual controls.
8. Contact `27`, method holdout and admission shadow remain unread regardless
   of the development result.

The exact numeric initializer, loss weights, capacities and thresholds must be
committed with the fit runner before the three fit PCM payloads are decoded.
This document fixes the source and order but does not authorize an underspecified
optimizer run.

## Consequence

An inventory pass proves only that the fresh real source slice is exact and
usable. A later fit pass would prove representation sufficiency on training
contacts only. R3B, real contact interpolation, one source-disjoint holdout,
automatic validation and engine integration all remain closed.
