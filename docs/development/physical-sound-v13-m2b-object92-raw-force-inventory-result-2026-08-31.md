# Physical Sound V13-M2b object-92 raw-force inventory — exact result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `SOURCE_INCOMPLETE_OBJECT92` |
| Target | ObjectFolder Real `92 / Glass_Red / Glass` |
| Protocol | [M2b protocol](physical-sound-v13-m2b-object92-raw-force-inventory-protocol-2026-08-31.md) |
| Next authorized step | `M2C_REALIMPACT_CONTROL_FREEZE`; object 92 remains closed |
| Product authority | None; authored clips remain authoritative |

## Result

M2b stops repeat-exact before signal decode. The `8 GiB` prefix contains all
36 raw microphone WAVs, all 36 paired force WAVs, all 36
`striking_force.yaml` files and only 35 of 36 `metadata.yaml` files. The sole
missing member is:

`92/audio/35/metadata.yaml`

The streaming inventory reached the first object-93 tar header after object
92, so extending to the `12 GiB` ceiling cannot recover this member. Under the
preregistered complete-quartet rule, object 92 is source-incomplete. The
runner does not drop contact 35, repartition roles or reinterpret 35 complete
parents as success.

All 36 raw microphone hashes equal their compact M2a counterparts. The failure
is narrowly structural, not an audio-identity mismatch. Microphone PCM, force
PCM, striking-force numeric values and every protected-role signal remain
undecoded.

## Exact identity

- protocol SHA-256:
  `8a9611b51b652c6a4d6e7e87059be7173b32f8fd01ced85125b2b20d999db67b`;
- acquisition runner SHA-256:
  `b124edf5c47ef379f86be3a0fd8ea9bf9702c8a8c107bfd77252991a67005da4`;
- inventory runner SHA-256:
  `35e00a43ca74e031c3edc48396ce8c7c1e301118edd271df308ace0b37dac541`;
- acquired prefix SHA-256:
  `ec9bdbc238d8bf876aeaced3635cf8e27487eb0511caa5a5a9d0dec7d552cdab`;
- acquisition report SHA-256:
  `740345e80ea44cc058768e010b3fdccb689975483f90daae37b7267f0d88eb09`;
- repeated inventory manifest SHA-256:
  `7762182dfeee652c778b82b6ec584efc5f7e0e040491deb2e842c77fc5963464`;
- repeated inventory report SHA-256:
  `874bfb7031b9c6fcaf327fccd513e38bc7b147c4e95de8ddbbfffdbaec1c142a`.

External evidence root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m2b.QsGuPy`

Final inventories `inventory-8g-identity-a` and
`inventory-8g-identity-b` are byte-identical for both manifest and report.
The dataset prefix, chunks and generated JSON stay outside Git.

## Bounded acquisition

The exact prior `512 MiB` prefix seed was reused. Eight cumulative HTTP range
responses contributed `8,053,063,680` new bytes, yielding one contiguous
`8,589,934,592`-byte prefix. Every response preserved the frozen ETag,
Last-Modified, full length and exact Content-Range.

The `4 GiB` checkpoint was honestly insufficient: it exposed 11 complete
parents and 50 of 144 selected members. The `8 GiB` checkpoint exposed object
92 through the next-object boundary and therefore made the missing member
terminal. No 12-GiB request was made.

## Final inventory

| Check | Exact result |
| --- | ---: |
| Complete contact parents | `35 / 36` |
| Selected members | `143 / 144` |
| Raw microphone WAVs | `36 / 36` |
| Raw force WAVs | `36 / 36` |
| `striking_force.yaml` | `36 / 36` |
| `metadata.yaml` | `35 / 36` |
| Raw/compact microphone identity | `36 / 36` |
| WAV headers | `72 / 72` |
| Next object header reached | `true` |

Every WAV header is PCM16 mono, `48,000 Hz`, `288,000` frames and `576,044`
bytes. This is container/identity evidence only.

## Read accounting

| Counter | Exact value |
| --- | ---: |
| Prefix bytes hash-verified | `8,589,934,592` |
| Compressed bytes consumed to object-93 header | `7,012,976,640` |
| Selected member bytes hashed | `41,477,510` |
| WAV header bytes parsed | `4,608` |
| Raw/compact microphone comparisons | `36` |
| Microphone / force sample values decoded | `0 / 0` |
| Striking-force numeric values decoded | `0` |
| Protected-role signal values decoded | `0` |
| Member payloads emitted | `0` |

## Consequence and next step

Object 92 cannot enter `SourceQualified` in this revision and receives no
formula, ML, quality, validator or atlas credit. Its M2a role partition remains
historical evidence; contact 35 is not removed after observing the source
defect. A later method cannot silently reopen it without a separately
preregistered source-policy revision and explicit reconsideration evidence.

M2c may still bind the previously published RealImpact derived-response lane
as an independent control, because that work neither consumes object-92 signal
nor weakens this stop. In parallel, the next canonical-source decision must be
made from the still-fresh candidates `59/82/93` or a newly censused internet
source, with source axes and acquisition cost frozen before access. M3 remains
blocked until M2 has a viable canonical source or explicitly rebaselines the
oracle as source-independent work.
