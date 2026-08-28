# Physical sound PS-2 — REALIMPACT Blue Bowl cross-tier E2 increment

| Field | Result |
| --- | --- |
| Status | `THREE_REALIMPACT_E2_OBJECTS / OBJECT_6_CROSS_TIER_E2_E3 / GLASS_9_OF_16 / FALLBACK_ONLY` |
| Scope | One external current-only `E2TransferResponse`; no new E3 group, split, admission, quality or production credit |
| Profile | `blue-bowl-row-0-v1` |
| Acquisition report | `68d3ddf111f87d3d7ecb13f3abb9176c97e72ac5ffe4585a2be555a7db6a7acc` |
| Inventory manifest/report | `88972e827883dddbd085b1c29c94c77de14b680bb76b448daa08d283edf7c58a` / `5132aa224fbe81740de4cec321b2e63d9ea6f8f2df8dc69a04d13f9ea107b8b0` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-blue-bowl-e2-range-v1-final/` |

## Question and bounded source discriminator

Can the next PS-2 increment add useful physical evidence for Glass without
inventing E3 object groups or downloading a multi-gigabyte archive?

The source audit rejected three tempting shortcuts:

- the official [DiffImpact paper and supplement](https://proceedings.mlr.press/v164/clarke22a.html)
  identify one real Glass bowl in the ASMR experiment, but expose no stable
  object-level dataset download with exact repeated files and hashes;
- the Aramaki [material-synthesis examples](https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/)
  expose five original Glass-object sounds, but only one real recording per
  object; synthesized and pitch-tuned derivatives are not real repeats;
- [ObjectFolder-Real](https://objectfolder.stanford.edu/objectfolder-real-download)
  lists eleven Glass objects with 30–50 real recordings each, but the five
  relevant official acoustic archives are 34–39 GB single-stream gzip files.
  They still provide no bounded per-object or seekable route, so the prior
  no-prefix-growth decision remains in force.

The strongest bounded increment is REALIMPACT object 6. The
[REALIMPACT paper](https://arxiv.org/abs/2306.09944) states that its 50 objects
were purchased from the ObjectFolder collection. The official ObjectFolder
table identifies object 6 as `Blue_Bowl / Glass`; the frozen REALIMPACT object
list and archive identify the same numeric object as `6_Bowl`. The existing
ObjectFolder interactive-demo adapter already grants object 6 three real E3
recordings. This package adds complementary E2 geometry and transfer evidence
for that same object; it does not count object 6 twice or change Glass `9/16`.

## Implemented boundary

`physical-sound-registry realimpact-row` now selects one of two exact frozen
profiles instead of embedding GreenGoblet as its only global profile. Both
profiles retain the same fail-closed boundary:

1. only their exact official HTTPS archive URL is accepted;
2. public-IP resolution is pinned and redirects/proxies are disabled;
3. every range must return the frozen `206`, content range, archive length,
   ETag and Last-Modified identity;
4. EOCD, complete central directory, local headers, entry metadata, decoded
   NPY/mesh hashes and the compressed transfer prefix are exact;
5. row 0 must be finite, hash-closed and consistent with the declared NPY
   shape and acquisition axes;
6. the impact coordinate must equal the indexed OBJ vertex exactly;
7. output is published atomically into an external empty directory.

The pre-existing GreenGoblet profile was rerun after the refactor. Every
previous acquisition file remained byte-identical; the only directory present
only in the historical final root was its separately generated inventory
report.

## Exact Blue Bowl result

| Evidence | Value |
| --- | --- |
| Official archive | `https://downloads.cs.stanford.edu/viscam/RealImpact/6_Bowl.zip` |
| Archive identity | `2,397,750,726` bytes; ETag `6433dc5b-8eeac5c6`; `2023-04-10T09:52:27Z` |
| Central-directory SHA-256 | `05fecf1eec74c68968d90870417f0651b18825003e58dedd317315ff1dcd36b1` |
| Total HTTP range payload | `1,734,304` bytes (`0.072330%` of the archive) |
| Mesh | `47,738` vertices; SHA-256 `f23127b45b0b163c796b4ef44d707b59fac58c4ef6e94e7676a4d88470e7f973` |
| Transfer array | float32 little-endian, shape `3000 x 230215`, 48 kHz |
| Selected row | row `0`, mesh vertex `35950`, position `[-0.03610739, -0.0520688, 0.03753229]` m |
| Listener | angle `0`, distance offset `0`, microphone `0`, position `[0.23, -0.04345, -0.91]` m |
| Row payload | `920,860` bytes; SHA-256 `2640223087e3c719f6c2175eba35a652941a09f98209aa2acf93e43b37f1aa23` |
| Duration / level | `4.7961458333` s; peak `193.5970001220703`; RMS `3.3436646138662574` |
| Audition WAV SHA-256 | `d5b715f1a1b17edf96698b0841a151b111622bb3dcfc3b7ca1615ac1d837c418` |
| Acquisition metadata SHA-256 | `f3dc97454a5888ec4a33488703d19e8fa6d5827a76095543c61d3235d3dc5c10` |
| Provenance review SHA-256 | `1e1a7c931fbfb227bffe0df39e684e401adacedf3d2fcafd42f5b989662efcc4` |

Two fresh online acquisitions and two V2 inventory audits are byte-identical.
The inventory grants only `force_deconvolved_transfer`, `geometry`,
`impact_position`, `listener_position`, `object_identity` and
`real_recording`. Raw force-profile bytes, material-composition revision,
repeat identity and support-fixture revision remain explicitly unavailable.

## Consequence and next action

REALIMPACT now contributes three typed E2 objects. Object 6 is the first
cross-tier Glass anchor: its E3 recordings can constrain material identity and
envelope while its E2 row constrains geometry, impact/listener position and
transfer response. That makes it a better future analysis-by-synthesis target,
but it is still only one row under an unversioned support fixture.

PS-2 therefore remains open: Glass is `9/16`, reject parents are `38/35`, all
entries remain in `dev`, and calibration/holdout/shadow plus automatic `Pass`
stay disabled. The next bounded physical-axis increment is REALIMPACT object
51 (`51_ShellPlate` / ObjectFolder Glass object 51) if its exact range profile
adds a distinct shell geometry. In parallel, source discovery continues for
seven true E3 Glass object groups; E2 rows never substitute for them.
