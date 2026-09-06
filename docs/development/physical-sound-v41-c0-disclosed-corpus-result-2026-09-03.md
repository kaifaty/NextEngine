# Physical sound V41 C0 disclosed corpus result

Date: `2026-09-03`

Result: `COMPLETE / REPEAT_EXACT / 139_RECORDS / 278_CONTENT_OBJECTS / 67_ROLE_DISJOINT_PARENTS / B0_V0_AUTHORIZED`

Roadmap authority: [Roadmap V41](../plans/physical-sound-synthesis-roadmap-v41.md)

## Outcome

C0 turns the permanently disclosed internet evidence into one reproducible ML
input plane. Two independent executions consume separate copies of the D1
roster, identified report and content-addressed cache and produce byte-identical
external trees. The result contains no protected data, model, generated
candidate, checkpoint or runtime asset.

| Input/evidence | Rows | C0 role |
| --- | ---: | --- |
| identified internet recordings | 135 | material/object waveform supervision and acoustic pseudo-targets |
| REALIMPACT force-deconvolved transfers | 4 | geometry, impact-position, listener-position and transfer supervision |
| total | 139 | `44 generator_train / 70 generator_development / 25 validator_calibration` |

The 139 rows represent 67 physical parents after REALIMPACT `6_Bowl` and
ObjectFolder object `6` are kept as one Blue Bowl parent. No physical parent or
D1 alias component crosses roles.

## Frozen implementation

- owner: `lab/scripts/physical_sound_v41_c0_disclosed_corpus_v1.py`,
  `57,813 bytes`,
  `sha256=5f2e7ecfd46f89f89d21205d148faf2974fd44e1118d62a44e989b1a1b98cc8c`;
- profile: `lab/profiles/physical-sound-v41-c0-disclosed-corpus.v1.json`,
  `7,028 bytes`,
  `sha256=439df3285a027b3070e507c8136ff525ff29ced8e025819e1fb0429140df61a5`;
- tests: `lab/tests/test_physical_sound_v41_c0_disclosed_corpus_v1.py`,
  `13,996 bytes`,
  `sha256=5a579a38bed48761035e5eadd58ad09fd442321e9361f9c1193f75d7062b3a10`.

The profile hash-binds SPEC-45, Roadmap V41, the D1 owner/profile/result, the C0
owner and the exact D1 roster bytes/root. It also binds the identified corpus
manifest/report, internet-source manifest, four REALIMPACT manifests, FFmpeg,
libarchive and NumPy/SciPy versions. Any input, environment, role, parent or
authority drift rejects before publication.

## Canonicalization and targets

Every source payload is verified by byte count and SHA-256 before decode.
CMU's ZIP and SoundPacks' RAR5 are read in memory with exact member identity;
WAV, MP3, Ogg/Vorbis and M4A/MP4 are decoded through the bound single-threaded
FFmpeg binary. A seekable temporary file supports M4A and is deleted before
publication. No network protocol or request is used.

The canonical signal path is frozen as:

1. mono decode at the declared source rate;
2. deterministic `scipy.signal.resample_poly` to `48 kHz` with Kaiser beta `5`
   and constant padding;
3. strongest derivative-energy onset, `10 ms` pre-trigger and a fixed
   three-second segment;
4. DC removal, symmetric peak normalization to `0.95` and PCM16 WAV encoding;
5. content addressing of the WAV and its canonical JSON acoustic target.

Each acoustic target records 24 log-band energies, spectral centroid,
bandwidth, rolloff, flatness, a 48-bin transient envelope, global decay and up
to 12 modal peaks with fitted `T60`, Q and confidence. These values are
waveform-derived pseudo-targets, not measured elastic constants. Across the
official corpus the extractor finds `1,650` modes; `1,311` have finite fitted
decay, and every row has between 7 and 12 modal peaks.

## Observed-axis masks

Unknown fields are not imputed:

| Axis | Observed rows |
| --- | ---: |
| waveform, material identity, object identity, acoustic pseudo-target | 139 |
| geometry, impact position, listener position, transfer response | 4 |
| absolute force, material composition, revisioned support condition | 0 |

Normalization makes cross-project absolute amplitude explicitly incomparable.
The REALIMPACT rows retain published geometry/contact/listener metadata, but the
missing raw force profiles and support-fixture revision prevent absolute-force
or arbitrary-support claims.

## External A/B evidence

External root:

`/home/kaifaty/.codex/experiments/nextengine/physical-sound/physical-sound-v41-c0-2026-09-03`

`run-a` consumes D1/cache/report A and `run-b` consumes D1/cache/report B. A
recursive comparison is empty. Each tree contains 286 files and `41,235,294`
bytes. The SHA-256 of the canonical sorted `path / bytes / sha256` inventory is:

`a9375d8401188e9d7f36bf931a19b45306cbc43e39a40e2b10787fa6d2ec9660`

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access-ledger.json` | 1,011 | `2ef23104fea0f457965e7412e01655e22c0847a36bda0858586402ef92648e57` |
| `corpus-card.json` | 1,415 | `a75dd496c94b26e49de6e49c050208f7abaf865fcc67a33e5485346cbbbed3a4` |
| `manifest.json` | 297,303 | `3ccc89cb4022209b756f470adbc388aa098eaedfcfb056ba0230ad23b71034e9` |
| `profile.json` | 7,028 | `439df3285a027b3070e507c8136ff525ff29ced8e025819e1fb0429140df61a5` |
| `generator_development.json` | 90,450 | `baeeb5ab36806ceb8fbe06bc581d0eb3bf1cf872939390129262eb8134c16f1e` |
| `generator_train.json` | 57,916 | `a1a468cd41328f8dbdc9d2f879fbb90a66ee7448762e7fb703702576d41127ee` |
| `validator_calibration.json` | 31,634 | `7a11eba70172d0c24e23cfbc4a3907a8572e5783d603e7ef4c8d0d00d69551ff` |
| `report.json` | 2,040 | `9a1489f63bd66298c3a19123b98ecf9226c3c0db21d226fa6f7c4d6cfc12879b` |

All 278 PCM/feature objects are referenced exactly by the manifest, their bytes
match their paths and metadata, and no unreferenced content object exists.
Every WAV is mono PCM16 `48 kHz`, 144,000 samples and 288,044 bytes.

## Access and gates

Each run reads `82,553,567` bytes of admitted source payload, verifies
`73,215,961` bytes of source archives and processes `113,477,976` resampled
source samples. Network requests, protected payload/signal, model bytes and
candidate-output reads are all exactly zero.

All eight conjunctive gates pass:

- every payload and environment dependency is hash-verified;
- all 139 rows have complete observed-axis masks;
- canonical PCM is mono `48 kHz` PCM16;
- the exact D1 roster bytes/root are bound;
- record, role and parent counts match the frozen profile;
- no component or physical parent crosses roles;
- protected and model access remain zero;
- publication is atomic and external to Git.

## Verification and authority

- focused C0 suite: `PASS`, `9/9` tests;
- external CLI A/B and recursive byte comparison: `PASS`;
- complete content-reference, object-hash, WAV-header and parent-role audit:
  `PASS`;
- mutations cover payload/D1/authority drift, archive-member absence,
  cross-role parent/component leakage, output escape/overwrite and late atomic
  publication failure;
- repository boundary scan: expected pre-existing `SOURCE_LAYOUT_ESCAPE_HATCH`
  in `realimpact_transfer_fixture.rs`; C0 adds no Rust source or escape hatch;
- runtime ProductChecks: `NOT_RUN`, because C0 is external research tooling
  under SPEC-45 `Proposed` and changes no production consumer.

C0 returns `C0_DISCLOSED_CORPUS_REPEATABLE_B0_V0_AUTHORIZED`. B0 may read only
the train/development projections to establish simple grouped baselines. V0
may read only validator-calibration records and frozen synthetic corruptions.
Neither stage receives protected, admission, cooking, demo, runtime or public
contract authority. Authored clips remain mandatory.
