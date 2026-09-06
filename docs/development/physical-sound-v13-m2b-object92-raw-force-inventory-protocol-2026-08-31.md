# Physical Sound V13-M2b — object-92 bounded raw-force inventory protocol

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Status | `PREREGISTERED / RAW_PREFIX_ACQUISITION_ALLOWED / SAMPLE_DECODE_FORBIDDEN` |
| Parent | [M2a result](physical-sound-v13-m2a-object92-source-role-freeze-result-2026-08-31.md) |
| Target | ObjectFolder Real `92 / Glass_Red / Glass` |
| Claim ceiling | `CanonicalImpactField`; raw force is onset/scalar-energy metadata only |
| Product effect | None; authored clips remain authoritative |

## Authorized question

Does a bounded prefix of the exact official `91–100` raw archive contain one
complete raw microphone/force/metadata quartet for every M2a object-92 contact,
with raw and compact microphone identities agreeing, before any signal sample
is numerically decoded?

M2b may download exact compressed byte ranges, inspect tar structure, hash
selected member payloads and parse 64-byte WAV container headers. It may not
convert microphone or force PCM bytes to numeric samples, parse the numeric
value in `striking_force.yaml`, estimate onset/energy/spectrum, listen to audio
or change the M2a object and role partition.

## Frozen parent identity

M2b accepts only the repeated M2a external outputs with:

- manifest SHA-256
  `b30f88978bd6dc749661c92a13093a97d6eda98771c75b058cc6ff149b5ea6b1`;
- report SHA-256
  `aafcbfdd551a0f18b0ec6fb390b093abb19825162455f49822fde718bacc0276`;
- role-root SHA-256
  `a271bcd6d5cbdc2c0d61114b40f3080909ddd224d8930414524131597157b867`;
- exact contact set `0…35` and role counts `8/6/12/3/4/3`;
- all M2a PCM/force sample-decode counters equal to zero.

Any mismatch stops before network or raw-prefix access.

## Frozen remote source

| Field | Value |
| --- | --- |
| URL | `https://download.cs.stanford.edu/viscam/ObjectFolder_Real/audio/audio_data_91_100.tar.gz` |
| Full bytes | `38,866,981,268` |
| ETag | `"63e3844c-90ca71194"` |
| Last-Modified | `Wed, 08 Feb 2023 11:15:24 GMT` |
| Range support | `bytes` |

Every HTTP response must use identity encoding, status `206`, the requested
inclusive `Content-Range`, exact byte count and the same full length. A changed
ETag or Last-Modified stops acquisition. Redirects may be followed only to the
same HTTPS host.

The previously acquired optional seed is eligible only at exact identity:

| Seed field | Value |
| --- | --- |
| Range | `bytes=0-536870911` |
| Bytes | `536,870,912` |
| SHA-256 | `5ef9789ad023233377aa60d58f66100f0bb62dd5df52556e051e757d13797313` |

The seed contains object-91 structure only and grants no object-92 signal
evidence. If absent or mismatched, the acquisition starts from byte zero.

## Bounded acquisition algorithm

1. Build one external contiguous prefix from byte zero. Fetch missing ranges
   as exact non-overlapping `1 GiB` chunks; the final chunk at a checkpoint is
   not shortened.
2. Inventory only at exact checkpoints `4 GiB`, `8 GiB` and `12 GiB`.
3. At each checkpoint, stream gzip/tar from byte zero. An expected truncated
   gzip/tar tail is not source failure. A malformed member before the tail,
   duplicate selected path or unexpected object-92 structure is failure.
4. Stop at the first checkpoint containing every expected object-92 quartet.
   The chosen prefix size and SHA-256 then become immutable M2b evidence.
5. If `12 GiB` lacks any expected member, return
   `DATA_INSUFFICIENT_ACQUISITION`. Do not fetch another byte, switch object,
   reduce roles or keep only visible contacts.

The hard ceiling is `12,884,901,888` bytes. With the exact optional seed, the
maximum new network payload is `12,348,030,976` bytes; without it, the maximum
is the full `12 GiB`. At most twelve range responses may contribute payload.

## Expected raw structure

For every contact ID `0…35`, the selected parent is directory
`92/audio/<contact>/` and must contain exactly one regular file of each kind:

- `mic.wav`;
- `Force.wav`;
- `metadata.yaml`;
- `striking_force.yaml`.

Other object-92 files may exist but are not emitted or interpreted. The
inventory must require:

- all 36 raw microphone and all 36 raw force WAVs are PCM16 mono, `48,000 Hz`,
  `288,000` frames and `576,044` bytes;
- each raw microphone SHA-256 equals the compact M2a microphone SHA-256 for the
  same object/contact key;
- each quartet is complete and no selected path is duplicated;
- selected YAML is hash/byte committed only; no YAML scalar is parsed;
- the exact M2a role is copied by contact parent and cannot be recomputed from
  the raw source.

## Semantic read budgets

| Counter | Maximum |
| --- | ---: |
| Contributing HTTP range responses | `12` |
| New compressed network bytes with exact seed | `12,348,030,976` |
| Final contiguous prefix bytes | `12,884,901,888` |
| Inventory checkpoints | `3` |
| Selected contact parents | `36` |
| Selected raw members hash-committed | `144` |
| WAV headers parsed | `72 x 64` bytes |
| Raw/compact microphone identity comparisons | `36` |
| Microphone PCM sample values decoded | `0` |
| Force PCM sample values decoded | `0` |
| Striking-force numeric values decoded | `0` |
| Protected-role signal values decoded | `0` |
| Member payloads emitted | `0` |

Compressed prefix bytes scanned and selected member bytes hashed are reported
separately from signal sample values. The prefix, reports and any acquisition
state stay outside Git.

## Access order and claim boundary

All roles remain sample-decode closed during M2b. A pass binds `Force.wav` only
as the future source for onset and scalar canonical-energy normalization. It
does not authorize frequency-domain division, shared FRF estimation or an
arbitrary-force `MeasuredTransferField` claim.

M3 still uses synthetic known truth only. After M3 passes under its own
protocol, M4 may decode only the eight object-92 `formula_fit` microphone/force
pairs. Development, representation holdout, both validator roles and admission
shadow retain the M2a order.

## Decision

Two local inventories over the same frozen prefix must produce byte-identical
manifest/report JSON. The only success decision is
`READY_FOR_M2C_REALIMPACT_CONTROL_FREEZE` with all required identities and
zero signal-value counters.

Any source mismatch, malformed selected member, raw/compact identity mismatch
or incomplete 12-GiB prefix fails closed. A pass gives source-qualification
evidence only: no formula, ML, quality, validator, atlas, runtime or public
contract credit.
