# Physical sound PS-2 — Greatest Hits bounded-inventory discriminator

Date: 2026-08-27
Status: `BOUNDED_RANGE_INVENTORY_PROVEN / E3_OBJECT_IDENTITY_REJECTED / NO_DOWNLOAD_ADAPTER / NO_CORPUS_ADMISSION_AUTHORITY`

## Question

Can the published Greatest Hits dataset supply leakage-safe identified Glass
objects without downloading its complete 20 GB low-resolution archive?

## Official source and frozen surface

The official [Visually Indicated Sounds project page](https://andrewowens.com/vis/)
publishes full-resolution videos and labels as a 50 GB ZIP, low-resolution
videos and labels as a 20 GB ZIP, precomputed sound features as a 1 GB ZIP and
CC BY 4.0 dataset terms. The exact project-page bytes reviewed on 2026-08-27
have SHA-256
`65c75da6de5db2840fe23f59849e0fc439e49ec644d127ec84a2b68e31d969a0`.

The authors' [paper](https://arxiv.org/html/1512.08512) states that the dataset
contains 977 videos and 46,577 actions. A video may probe different objects in
one scene. Annotators label individual actions with material, hit/scratch and
reaction categories; these semantic labels were not used to train the original
model. The reviewed arXiv HTML bytes have SHA-256
`42d08b72a5a5c6774b6554b28914fd06db18f5c2e1828c8370840bdbbe3fe2e8`.

## Bounded remote inventory result

The low-resolution archive is
`https://web.eecs.umich.edu/~ahowens/vis/vis-data-256.zip`. Its frozen HTTP
identity was:

| Field | Value |
|---|---:|
| Content length | 21,142,601,241 bytes |
| ETag | `4ec327e19-5cc7fa3687c80` |
| ZIP entries | 5,865 |
| Central-directory bytes | 734,805 |
| Central-directory SHA-256 | `0108ab0ab7b0848b981ada74371d324014eef7a931e3d6b2cc0342e919ddc9f0` |
| WAV / label TXT / MP4 entries | 1,954 / 979 / 2,931 |

The server honors exact byte ranges. Reading the ZIP64 end records, the complete
central directory and only the compressed local records for 979 text files
required about 2 MB of returned metadata rather than the 20 GB archive. Every
local record was checked against its central-directory method, size and CRC32
before raw-deflate decoding. The 979 decoded files total 1,283,710 bytes; a
canonical sorted `path / byte count / SHA-256` index hashes to
`aaa529de3da5aae1eb18b8afa46dc6a186157831dea880648bb2ab9ed1f472f1`.

The labels reproduce all 46,577 actions. The material inventory contains 382
`glass` actions across 31 video IDs: 270 `hit`, 105 `scratch` and seven actions
without an action label. Only three of those videos contain no other non-null
material label; the other 28 contain two to five material labels.

This proves that bounded ZIP inventory and selected-entry retrieval are
technically possible. It does not prove object identity.

## Why E3 is rejected

The downloadable labels contain timestamp, material, action and reaction. They
do not publish a stable target-object ID. The archive path identifies a video,
not an object, and the paper explicitly permits several objects in one scene.
Consequently neither `video_id` nor `video_id + material` can become an E3
object group without inventing identity from visual proximity. Even the three
single-material videos may contain more than one Glass object or repeated views
of an object present elsewhere.

The download host also omitted the InCommon intermediate certificate during
the review. Its leaf certificate was valid for `*.eecs.umich.edu` on the review
date, but the ordinary system verifier failed with curl exit 60,
`unable to get local issuer certificate`. Unverified TLS was used only for this
read-only discriminator. It is prohibited in the registry and no fetch adapter
is added.

Greatest Hits therefore receives no E3 object/repeat credit and does not move
Glass beyond `5/16`. Its material-event labels may be reconsidered for a future
non-admission classifier study only if that study has its own leakage groups;
they cannot populate the powered object-group corpus.

## Next action

Do not download the 20 GB archive and do not retry video-as-object grouping.
The next bounded candidate is the official REALIMPACT
`93_GreenGoblet.zip`: it has a valid HTTPS origin, an exact 2,311,697,935-byte
response and byte-range support. Inspect its ZIP central directory and prove an
exact object-level E2 row retrieval path without downloading the full archive.
If that fails, keep the object fallback-only and move to the next published
source. FillImpact is a strong future E1/E3 candidate, but its July 2026 paper
does not yet publish dataset bytes or a repository and therefore is not an
executable source today.
