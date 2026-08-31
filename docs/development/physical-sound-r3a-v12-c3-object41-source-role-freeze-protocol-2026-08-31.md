# Physical Sound R3A V12-C3 — object-41 source/role freeze protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED_BEFORE_RUNNER / PCM_VALUES_CLOSED` |
| Target | ObjectFolder-Real `41 / Wrench_Large / Steel` |
| Scope | Fresh exact object, canonical recorded setup, normalized instrument counts |
| Prerequisite | [C1 pass](physical-sound-r3a-v12-c1-acquisition-coverage-oracle-result-2026-08-31.md) and [C2 source pass](physical-sound-r3a-v12-c2-internet-source-zero-decode-inventory-2026-08-31.md) |
| Product effect | None; authored clips remain authoritative |

## Question

Can one fresh ObjectFolder object be bound to paired force/microphone trials,
coordinates and geometry with six immutable roles and exact read budgets before
any signal sample is decoded?

C3 does not fit a modal model and cannot receive real-quality, material-family,
ML, validator, atlas or runtime credit. A pass authorizes C4 to read only the
`estimator_fit` channels under a separately committed fit protocol.

## Target and source identity

Object `41 / Wrench_Large / Steel` is selected because it is:

- fresh in the current physical-sound evidence history;
- the first object in its official `41–50` raw archive, avoiding unrelated
  object prefixes;
- represented by exactly `35` matching compact audio/contact keys plus one
  point-cloud key and scale `0.25329818850723695`;
- large enough for six contact-parent-disjoint roles.

The object is absent from the official contact-localization train/val/test
split. No published split is attributed to it. C3 uses the hash partition below
and does not inspect waveform, force scalar, coordinate or geometry values when
assigning roles.

Frozen official identities:

- website source commit:
  `d058ba09e7e7a1a8358d64d7b48e5f588b377eb8`;
- contact-localization commit:
  `4bb002f519cab9d250bbbe045a6df0248bf1639f`;
- raw URL:
  `https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_41_50.tar.gz`;
- raw length `40,775,630,586`, ETag `"63e36ee1-97e6abefa"`, Last-Modified
  `Wed, 08 Feb 2023 09:44:01 GMT`;
- compact audio SHA-256:
  `14a15b96dda7c3933a4a5829dc5ee21e1f9c2fe29f7c558925df24e87097a2c9`;
- compact contacts SHA-256:
  `310c45e11e40ceafc9c9fc1379059848ff664a7c2999fea5aabea479fc61eeee`;
- compact point clouds SHA-256:
  `29ebf37bb88bc5c654b096f31552e4677fc156135930da72c82e306c5c517e51`;
- split/scale SHA-256:
  `77ea2d99724197ad9e88c3ad90f49885c582dea6835fdc0ad533e5b9ce78674e` /
  `39c84eafabe999b1503710a9d601b0bc94fc6e358461aa2e5988b34b6b6d0b8a`.

The exact object table comes from the pinned website source. Unpinned live HTML
cannot change the object name, material or archive mapping.

## Bounded raw prefix

Only inclusive HTTP range `bytes=0-4294967295` is eligible: exactly `4 GiB`.
The prefix is external and must match the response range/length plus a frozen
SHA-256 recorded by the C3 manifest. It may be streamed once for source
inventory and rescanned locally for repeat evidence.

The budget was chosen from a force-value-blind prefreeze: the first `512 MiB`
prefix has SHA-256
`eca66c012e1a07e07f7fc6642460f02fb89e16d8dedb8568beb0e99cb97786d5`
and contains five complete structural trial quartets with IDs
`5,15,27,33,34`. No microphone or force sample was decoded.

Hard stop:

- if all `35` object-41 trial quartets are not complete inside `4 GiB`, decide
  `DATA_INSUFFICIENT_RAW_PREFIX`;
- do not extend the range, change object, reduce roles or pick only visible
  contacts in this revision;
- never fetch the full `40.8 GB` archive as a fallback.

## Immutable role partition

For each contact ID `0…34`, compute lowercase hex
`SHA256("nextengine-v12-c3-object41-role-freeze-v1:<id>")`, sort by
`(digest,id)`, then take fixed contiguous counts `16/5/4/4/3/3`:

| Role | Contact IDs in hash order | Decode authority after C3 |
| --- | --- | --- |
| `estimator_fit` | `30,13,24,14,19,9,6,15,5,33,2,29,4,16,31,34` | C4 fit only, after a separate protocol |
| `generator_development` | `23,32,18,0,10` | Closed until fit passes |
| `representation_holdout` | `1,25,17,21` | Closed until development passes |
| `validator_calibration` | `3,12,26,11` | Closed until a generator claim exists |
| `validator_method_holdout` | `28,22,8` | Closed until validator is frozen |
| `admission_shadow` | `27,7,20` | Closed until generator, validator and cooker pass |

Canonical hash-order root:
`235e418fb62febf3673af2606f7a9efb3b856ac67526bda0ae6464834abe0620`.

Every contact has exactly one role. No role may borrow a contact, coordinate,
force scalar, PCM header or payload from another role. The small validator
roles protect these contacts but are not by themselves sufficient for C7
bounded-risk release; independent negative/mutation evidence remains required.

## Runner stages

### `freeze`

Create a canonical external manifest binding:

- protocol and runner hashes;
- official source/commit/HTTP identities;
- `4 GiB` prefix range and expected SHA-256 supplied by the acquisition step;
- all compact artifact hashes;
- exact object/contact set, role partition and source limitations;
- all per-role member and sample budgets;
- Python/NumPy environment.

Freeze reads no source file and generates no sample.

### `preflight`

Run twice from the manifest without source arguments. It must reproduce one
canonical report with:

- decision `C3_SOURCE_ROLE_FREEZE_FROZEN`;
- all source, metadata, raw, microphone and force bytes read `0`;
- all decoded sample counters `0` for every role;
- every role authorization `false`;
- network requests `0`.

### `inventory`

Read only the exact external artifacts bound by the manifest. The runner may:

- stream tar headers and skip irrelevant video/image payloads;
- hash selected raw `mic.wav`, `Force.wav`, `metadata.yaml` and
  `striking_force.yaml` members;
- parse only WAV container headers, not PCM sample values;
- hash compact microphone members and require `35/35` raw/compact identity;
- decode coordinate NPY and one point-cloud NPY strictly as metadata;
- read split and scale JSON.

It may not convert, unpack or numerically inspect a microphone/force PCM sample;
parse striking-force scalar values; select contacts from any observed signal or
coordinate; emit a WAV; or read another object.

## Required checks and counters

- exact object key set is `0…34` in raw microphone, raw force, compact
  microphone and coordinate sources;
- every raw trial has the four expected members under one object/contact key;
- all microphone and force headers are PCM16 mono, `48,000 Hz`, `288,000`
  frames; paired headers agree;
- every raw microphone hash matches its compact counterpart;
- every coordinate is finite shape `(3,)` and the point cloud is finite shape
  `(1024,3)`; scale is the frozen finite positive value;
- all six roles are nonempty, disjoint and complete;
- selected signal sample values decoded are `0`; protected roles also report
  `0` payload emission and `0` sample decode;
- network requests during runner execution are `0`;
- repeated inventory runs produce byte-identical manifest/report;
- all generated reports stay external; no dataset, prefix, NPY, WAV or report
  enters Git.

The report distinguishes compressed prefix bytes scanned, selected member bytes
hashed, metadata values decoded and signal sample values decoded. Hashing a WAV
payload is not reported as sample decode, but it grants no signal access to
model code.

## Decision

`READY_FOR_C4_ESTIMATOR_FIT` requires every check above. It authorizes only the
frozen `estimator_fit` role and still requires a separate C4 fit protocol before
decode.

Any mismatch yields a stable rejection or `DATA_INSUFFICIENT_RAW_PREFIX` and
keeps all roles closed. Missing listener pose, exact support and SI channel
calibration remain explicit claim limitations, not C3 repair targets.
