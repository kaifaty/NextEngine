# Physical Sound V13-M2a — ObjectFolder object-92 source/role freeze protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / ZERO_SAMPLE_DECODE_ONLY` |
| Roadmap | [V13 M2](../plans/physical-sound-synthesis-roadmap-v13.md) |
| Parent shortlist | [M1c result](physical-sound-v13-m1c-historical-census-and-shortlist-result-2026-08-31.md) |
| Target | ObjectFolder Real `92 / Glass_Red / Glass` |
| Claim ceiling | `CanonicalImpactField`, one unpublished fixed microphone condition |
| Product effect | None; authored clips remain authoritative |

## Authorized question

Can one fresh, vessel-like glass object be bound to exact published microphone,
contact-coordinate and geometry identities, then partitioned into six immutable
contact-parent roles before any waveform sample is numerically decoded?

M2a may hash selected WAV members and inspect their container headers. It may
decode contact coordinates and object geometry because they are source
metadata. It may not convert a PCM payload to sample values, inspect a
spectrum, estimate an onset, choose a mode, listen to audio or read a force
sample. A passing result authorizes only the separately preregistered M2b raw
force acquisition step.

## Signal-blind candidate decision

The exact M1c shortlist SHA-256 is
`0bd87fc3e62bfa6e574c4c206e795282223a4ecbd6221790e1ff4c60a113f6d7`.
Its fresh candidates are `59 / Soap_Dish`, `82 / Can`, `92 / Glass_Red` and
`93 / Vase`, all published as `Glass` and all with zero direct and path-token
historical exposure.

Candidate `92` is selected without opening audio because it is the only fresh
candidate present in the already hash-closed ObjectFolder contact-localization
split with all of these axes:

- `36` published microphone/contact keys;
- `36` matching finite contact coordinates;
- one `1024 x 3` object point cloud;
- one published object scale;
- a complete official `26 train / 3 val / 7 test` partition;
- raw archive lineage in `audio_data_91_100.tar.gz` for a later force-only
  acquisition.

The other three candidates are not fallback choices in this revision. `59`
and `93` have a published scale but no complete contact-localization split;
`82` has neither the selected split nor a scale entry. Object names and source
coverage decide M2a; no waveform-derived or perceptual fact participates.

## Frozen compact sources

All payloads remain outside Git. The runner must receive these exact external
regular files and fail closed on any byte or hash mismatch:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| M1c `shortlist.json` | `3,375` | `0bd87fc3…f6d7` |
| `DATA_real/audio.tar.gz` | `463,486,373` | `14a15b96…a2c9` |
| `DATA_real/contacts.tar.gz` | `121,961` | `310c45e1…eeee` |
| `DATA_real/global_gt_points.tar.gz` | `2,359,020` | `29ebf37b…7e51` |
| `DATA_real/split.json` | `19,258` | `77ea2d99…674e` |
| `DATA_real/scale.json` | `2,652` | `39c84eaf…0b8a` |

The publication and benchmark identities remain:

- ObjectFolder Real download page:
  `https://objectfolder.stanford.edu/objectfolder-real-download`;
- contact-localization repository commit
  `4bb002f519cab9d250bbbe045a6df0248bf1639f`;
- raw archive URL:
  `https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_91_100.tar.gz`;
- observed raw archive identity: `38,866,981,268` bytes, ETag
  `"63e3844c-90ca71194"`, Last-Modified
  `Wed, 08 Feb 2023 11:15:24 GMT`, byte ranges accepted.

The compact microphone waveform is the canonical response signal. The raw
force source is a later onset and scalar-energy normalization input only. No
frequency-domain division or arbitrary-force transfer claim is authorized.

## Frozen official split

The runner must recover exactly these contact IDs from the official split:

| Publisher role | Contact IDs in publisher order |
| --- | --- |
| `train` | `3,20,18,15,27,4,34,30,0,23,29,16,24,7,5,14,26,21,19,32,8,6,17,25,10,1` |
| `val` | `9,11,13` |
| `test` | `33,12,35,2,28,22,31` |

The union must be exactly `0…35`, with no duplicate object/contact key. The
published scale must be exactly `0.3435394121395228`.

## Frozen role algorithm and exact result

Role assignment is computed from contact metadata before any selected WAV is
opened. The domain separator is
`nextengine-physical-sound-v13-m2a-object92-role-v0`.

1. Within publisher `train`, choose the `formula_fit` seed by minimum SHA-256
   rank of `separator + NUL + "formula-fit-seed" + NUL + contact_id`.
2. Add contacts until there are eight by maximizing the minimum squared
   Euclidean distance to the selected coordinates. An exact-distance tie is
   broken by SHA-256 rank under domain `formula-fit`, then numeric ID.
3. From the remaining publisher-train contacts, choose six
   `generator_development` contacts by the same farthest-point rule against
   the union of fit and already selected development contacts, with domain
   `generator-development`.
4. The remaining publisher-train contacts become
   `representation_holdout`.
5. Publisher `val` becomes `validator_calibration` unchanged.
6. The three lowest SHA-256 ranks of publisher `test` under domain
   `admission-shadow` become `admission_shadow`; the remainder become
   `validator_method_holdout`.

The runner must reproduce this exact immutable partition:

| Next Engine role | Count | Contact IDs |
| --- | ---: | --- |
| `formula_fit` | `8` | `8,32,19,26,16,21,29,1` |
| `generator_development` | `6` | `10,23,6,3,34,15` |
| `representation_holdout` | `12` | `0,4,5,7,14,17,18,20,24,25,27,30` |
| `validator_calibration` | `3` | `9,11,13` |
| `validator_method_holdout` | `4` | `2,12,31,33` |
| `admission_shadow` | `3` | `22,28,35` |

The list order shown for fit/development is the deterministic farthest-point
order. Other role lists are numerically sorted. Any mismatch stops M2a; the
runner must not silently repartition or choose another object.

## Read and output budgets

M2a has these exact semantic budgets:

| Counter | Maximum |
| --- | ---: |
| Network requests | `0` |
| Newly acquired raw-archive bytes | `0` |
| Selected compact WAV members hash-committed | `36` |
| Selected WAV headers parsed | `36 x 64` bytes |
| PCM sample values decoded, all roles combined | `0` |
| Force sample values decoded | `0` |
| Contact-coordinate scalar values decoded | `108` |
| Point-cloud scalar values decoded | `3,072` |
| Selected split pairs decoded | `36` |
| Selected scale values decoded | `1` |
| WAV/member payloads emitted | `0` |

Full-file hashing and compressed archive traversal are provenance operations,
not signal interpretation. The report must still expose their file-byte counts
separately from sample-decode counters. Output is canonical JSON in a fresh
external directory; no source payload or generated report enters Git.

## Access order after M2a

All six roles remain sample-decode closed in M2a. If the repeated inventory
passes, later stages may open them only in this order:

1. M2b acquires raw object-92 quartets without decoding force or microphone
   samples. It uses resumable `1 GiB` prefix increments from byte zero and a
   hard `12 GiB` prefix ceiling. Incompleteness at the ceiling is
   `DATA_INSUFFICIENT_ACQUISITION`, not permission to change object or budget.
2. M3 uses synthetic known truth only; all real roles remain closed.
3. After M3 passes, M4 may decode `formula_fit` microphone and force only.
4. `generator_development` opens once after fit gates and hyperparameters are
   fixed.
5. `representation_holdout` opens once for the frozen formula tournament.
6. `validator_calibration` opens only while building Validator V1, after the
   formula winner is immutable.
7. `validator_method_holdout` opens only after validator methods and thresholds
   are frozen.
8. `admission_shadow` is a one-shot M9 input and cannot be reopened for tuning.

## Pass, stop and non-claims

M2a returns `READY_FOR_M2B_RAW_FORCE_ACQUISITION_PROTOCOL` only when two
independent executions produce byte-identical manifest/report JSON and all
source, membership, header, geometry, split, role and zero-sample counters
match. Otherwise it returns a fail-closed stop and authorizes no successor.

A pass proves only exact source identity and an immutable leakage boundary. It
does not prove that object 92 sounds like the requested thin-walled vessel,
that canonical modal fitting works, that ML beats interpolation, that the
validator is safe, or that any runtime/public contract should change.
