# Physical Sound V13-M2a object-92 source/role freeze — exact result

| Field | Value |
| --- | --- |
| Date | `2026-08-31` |
| Decision | `READY_FOR_M2B_RAW_FORCE_ACQUISITION_PROTOCOL` |
| Target | ObjectFolder Real `92 / Glass_Red / Glass` |
| Claim ceiling | `CanonicalImpactField`, one fixed unpublished microphone condition |
| Protocol | [M2a protocol](physical-sound-v13-m2a-object92-source-role-freeze-protocol-2026-08-31.md) |
| Product authority | None; authored clips remain authoritative |

## Result

M2a passes twice byte-for-byte. Object `92` is selected from the M1c fresh
shortlist using source metadata only, its complete compact microphone/contact/
geometry identity is hash-closed, and all `36` contact parents are assigned to
six immutable roles before the first selected WAV member is opened.

The runner hashes each selected WAV and parses only its 64-byte RIFF header.
PCM sample values, force sample values, network requests, newly acquired raw
bytes and emitted waveform payloads are all exactly zero. This authorizes only
an M2b raw-force acquisition protocol; no real formula fit is authorized.

## Exact identity

- protocol SHA-256:
  `7fd63e5e2fd7db0ac8a44e2d660a5fc48011eb7451fd12170934a214e66bf57a`;
- runner SHA-256:
  `8362f907194613833deb1347916de35095bacf2acb9eb4e91b28204e5b8bac78`;
- manifest SHA-256:
  `b30f88978bd6dc749661c92a13093a97d6eda98771c75b058cc6ff149b5ea6b1`;
- report SHA-256:
  `aafcbfdd551a0f18b0ec6fb390b093abb19825162455f49822fde718bacc0276`;
- role-root SHA-256:
  `a271bcd6d5cbdc2c0d61114b40f3080909ddd224d8930414524131597157b867`.

Repeated external outputs:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m2a.Xu3KaC/run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v13-m2a.Xu3KaC/run-b`.

Both `manifest.json` files compare equal and both `report.json` files compare
equal.

## Why object 92

M1c exposed four fresh glass candidates: `59`, `82`, `92` and `93`. Without
reading their waveform samples, only `92` has the complete selected
contact-localization chain: `36` microphone keys, matching coordinates, a
`1024 x 3` point cloud, scale `0.3435394121395228` and official
`26 train / 3 val / 7 test` roles. That metadata fact, not a listening or
spectral result, makes `92` the M2 exact object.

Candidates `59`, `82` and `93` remain fresh. They are not silently consumed as
backups and cannot replace object 92 after a later negative result in this
revision.

## Immutable roles

| Role | Count | Contact IDs |
| --- | ---: | --- |
| `formula_fit` | `8` | `8,32,19,26,16,21,29,1` |
| `generator_development` | `6` | `10,23,6,3,34,15` |
| `representation_holdout` | `12` | `0,4,5,7,14,17,18,20,24,25,27,30` |
| `validator_calibration` | `3` | `9,11,13` |
| `validator_method_holdout` | `4` | `2,12,31,33` |
| `admission_shadow` | `3` | `22,28,35` |

The split is complete and contact-parent-disjoint. Fit uses eight of 36
contacts (`22.2%`), selected by the frozen coordinate-only farthest-point
policy. Publisher validation remains validator calibration; publisher test is
divided by the frozen SHA-256 rule between method holdout and one-shot shadow.

## Source and geometry checks

| Check | Result |
| --- | --- |
| M1c shortlist | exact `3,375` bytes and SHA-256 |
| Compact source archives/JSON | all five exact byte counts and SHA-256 values |
| Official split | exact `26 / 3 / 7`, union `0…35` |
| Contact/audio keys | exact equality for `36/36` parents |
| WAV contract | `36/36`, PCM16 mono, `48 kHz`, `288,000` frames |
| Coordinates | `36 x 3` finite float64 values |
| Point cloud | finite float64 `1024 x 3`, SHA-256 `782c8944…3abe` |
| Role algorithm | exact expected six-role partition |

Point-cloud bounds are
`[-0.234268,-0.042048,-0.016215]…[0.107617,0.039621,0.019240] m`.
Numeric microphone pose, support state, wall thickness, microphone calibration
and composition beyond the publisher label `Glass` remain absent.

## Read accounting

| Counter | Exact value |
| --- | ---: |
| Source file bytes hash-verified | `465,992,639` |
| Selected WAV members hash-committed | `36` |
| Selected WAV payload bytes hashed | `20,737,584` |
| WAV header bytes parsed | `2,304` |
| Contact-coordinate scalar values decoded | `108` |
| Point-cloud scalar values decoded | `3,072` |
| Selected split pairs / scale values | `36 / 1` |
| PCM / force sample values decoded | `0 / 0` |
| Network requests / raw bytes acquired | `0 / 0` |
| Waveform payloads emitted | `0` |

Hashing selected source bytes proves identity but is not a numerical signal
decode. Every real role remains sample-decode closed.

## Decision and next step

M2a is complete. M2b must be preregistered before network acquisition. It may
retrieve the object-92 raw archive prefix in resumable `1 GiB` increments from
byte zero, with a hard `12 GiB` ceiling, and may inventory raw microphone/
force/metadata quartets without decoding sample values. Failure to expose all
36 paired parents at that ceiling is `DATA_INSUFFICIENT_ACQUISITION`.

M3 remains blocked until M2 also binds the separately scoped RealImpact
derived-response control. M4 real fitting, protected roles, model selection,
validator calibration, atlas cooking, runtime integration and any public
contract remain unauthorized.
