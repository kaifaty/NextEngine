# PS-2 REALIMPACT Skull Cup bounded-range E2 pilot — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `FIVE_REALIMPACT_E2_OBJECTS / SKULL_CUP_RANGE_PROFILE / E3_DISCOVERY_NEGATIVE / FALLBACK_ONLY` |
| Scope | Published current-only `E2TransferResponse`; no E3, split, admission, quality or production credit |
| Frozen profile | `skull-cup-row-0-v1` |
| Acquisition report | `2831d328ce49c277030cc6b6dd655c2d6bbedac4e482d7b51ebd3c03b2f1b36f` |
| Inventory manifest/report | `05c3cc6fb20c2522413ff86f5a60dce725de807204f2874f9fb1117ea51ea493` / `f393c8bd0ccadd54ffbf767df2c21384ca57cf5457c8b58d0076a6e1e8e6c45a` |
| External roots | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-realimpact-skull-cup-e2-range-v1-final/` and `-repeat-final/` |

## Question and decision

The immediate PS-2 target was two additional object-bound Glass E3 groups.
Two bounded primary-source audits did not expose an honest route:

| Candidate | Exact observation | Decision |
| --- | --- | --- |
| ObjectFolder public repositories | The unimported `fork_vis` demo contains ObjectFolder objects 36 `Portion_Cup_White / Polycarbonate` and 52 `Fork_Small / Steel`. The official `material-classification` benchmark at commit `e19af7d2a81d7830a01be63a01c99816dc237bec` loads object-keyed `audio_examples/*.npy` spectrograms from an 81,946,646,898-byte Dropbox bundle, not original recordings. | No new Glass E3 bytes; do not download the full processed bundle. |
| [YCB-impact sounds OSF project](https://osf.io/4tcp6/) | The paper and workbook bind Glass materials and physical YCB objects. The robot's vertical object-named folders contain no Glass object, while horizontal Glass clips are grouped by poke speed across physical objects without per-clip object identity. Manual raw WAVs are long capture sessions without a published stable session-to-object manifest. | Useful material-only E2 discovery, but no object-bound E3 credit. |

The roadmap fallback was therefore exercised instead of weakening the evidence
tier. `60_SkullCup` adds one exact REALIMPACT geometry/transfer row. The
[REALIMPACT paper](https://arxiv.org/abs/2306.09944) states that its physical
objects came from ObjectFolder; the official
[ObjectFolder-Real table](https://objectfolder.stanford.edu/objectfolder-real-download)
identifies numeric object 60 as `Beer_Glass / Glass`, while REALIMPACT uses the
alternate archive label `60_SkullCup`. Composition revision remains unavailable
and is not inferred.

## Bounded source and exact result

The profile validates the official archive HTTP identity, EOCD, central
directory, all required ZIP members, six decoded NPY arrays, the decoded mesh
and a fixed compressed prefix of the transfer array.

| Measurement | Frozen value |
| --- | --- |
| Archive | `2,328,619,384` bytes; ETag `6433e696-8acbe978`; modified `2023-04-10T10:36:06Z` |
| Central directory | offset `2,328,618,103`; `1,259` bytes; SHA-256 `61d780fe1198cb1b7b57d0add264f2256565b6a8fe63901ca82df7b44f68a6c0` |
| Range payload | `1,670,815` bytes, `0.071751%` of the archive |
| Transfer array | `3000 x 209549`, float32, 48 kHz; ZIP CRC32 `5a7f5d39` |
| Selected row | row `0`, mesh vertex `2764`, position `[0.01403121, -0.0412921, 0.1441321]` m |
| Listener | `[0.23, -0.04345, -0.91]` m; angle `0`; distance offset `0`; microphone `0` |
| Row | `209,549` samples, `4.3656041666666665` s; SHA-256 `817da17e6f4b069f0a9f367d2c564a82cf14eb254f699b846c6770543672bb41` |
| Level | peak `408.8901672363281`; RMS `7.159311351319438` |
| Audition WAV | SHA-256 `1ac5578bd24c0f27c3c0e0e398e8e60d1afb2b3abbe527defeb568990ce1ab77` |
| Acquisition metadata | SHA-256 `6019807e8e1f545b63d7da883162bcf089fae5d1fcadb13604e575be2e78f0b0` |
| Provenance review | SHA-256 `5012ff5657461cd83e914812881866b7d7c5cc0e07a15f482bc6392da3b10b7e` |

The `47,810`-vertex mesh has SHA-256
`b818bcefe67dc863f6b8dc656a40b02a31a3d12f124ad80fbe45c494ff4a5b5a`
and bounds `[-0.0446902, -0.04388442, -0.0007535]` to
`[0.04464547, 0.04635854, 0.1507005]` metres. Its approximately
`89 x 90 x 151` mm extent adds a narrow vessel transfer distinct from Blue Bowl
and Shell Plate. The row-zero coordinate equals mesh vertex 2764 exactly.

## Reproducibility and validator controls

Two independent online acquisitions and two V2 inventory audits are
byte-identical. The typed adapter now freezes five exact REALIMPACT pilots. A
focused negative control replaces the Skull Cup transfer hash with the Shell
Plate hash and is rejected as not matching the frozen `60_SkullCup` pilot.

The adapter grants only geometry, impact/listener position, object identity,
real-recording identity and force-deconvolved transfer. Raw force-profile bytes,
material-composition revision, repeat identity and support-fixture revision
remain explicitly unavailable.

## Consequence and next action

REALIMPACT now contributes five typed E2 objects, but E3 Glass coverage remains
`14/16`, reject parents remain `38/35`, and every entry remains in `dev`. This
pilot does not create an eighth E3 project, a Glass target group, a partitioned
holdout or a `Pass` decision.

The next source cycle should require object-bound original audio before any
download: exact physical-object identity, raw recording bytes, stable source
revision and repeat identity where published. Processed spectrogram-only or
material-aggregated routes stay discovery-only. After two more Glass E3 objects
exist, audit project-disjoint split feasibility before assigning any
calibration, holdout or shadow partition.
