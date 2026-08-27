# PS-2 ObjectFolder-Real interactive-demo E3 pilot — 2026-08-28

| Field | Result |
| --- | --- |
| Status | `FIVE_OBJECTS_EVIDENCE_READY / GLASS_9_OF_16 / REJECT_PARENTS_11_OF_35 / PASS_DISABLED` |
| Scope | Published current-only `E3IdentifiedRecording`; no corpus admission, split opening, runtime or content promotion |
| Adapter | `objectfolder-real-demo-identified-recording-v1` |
| Pilot source manifest | `b4c7bc78de806e692de9fdf6c5078b24cb7fa071bbc8ddaf6ad12712394ac091` |
| Repeated pilot source report | `218bc552f3710e8d817ed33c4d03a28d3eb15b29a9f6d00d8aa55f2eb0abc807` |
| Combined source manifest/report | `4aa679ed31dff3b3bb2fd8402733bb6abc7f7ec9d4f26b89cfca0dc11c0a739e` / `993007be837df3baf7fea5a49d7603451a02f96a15cb24ca8b5927be8d1adc03` |
| Combined identified manifest/report | `5f5165668b598311c04a671a27cfd0eba478f483434eaad81f7853ed792f5e0a` / `ade80e0fd6eab2c2ae1cc2a47720f573a358082dcfef9237d730e394ba89c882` |
| External root | `/home/kaifaty/.codex/experiments/nextengine/physical-sound/ps2-objectfolder-real-demo-v1/` |

## Question and decision

The earlier ObjectFolder-Real discriminator correctly rejected growing prefixes
of 34–39 GB single-stream gzip archives: no bounded seek path reached repeated
recordings safely. That archive route remains rejected.

The official site also exposes six interactive object demos backed by public
GitHub Pages repositories. Their individual `StreamingAssets` MP4 files provide
a new bounded route. Five official object/repository bindings are exact and are
imported; the sixth is excluded because the official object ID and repository
folder ID disagree. This is a changed source route, not a retry of the failed
archive-prefix approach.

The adapter grants only `E3IdentifiedRecording` after it validates all of the
following:

- the exact official object-table row binds object ID, object name and material;
- the exact demo-index card binds that object ID to its official repository;
- immutable repository commit and root-tree identities match the profile;
- every raw URL derives from repository, commit, object and clip identity;
- downloaded bytes match exact byte count, SHA-256 and independently computed
  Git blob SHA-1;
- the MP4 has the expected ISO BMFF brands, one video track and one trimmed
  MPEG-1 Layer III stereo 44.1 kHz audio track with 264,997 playable frames;
- all four E3 capabilities bind every source artifact.

The parser is bounded and in-process. It does not invoke ffmpeg, trust a file
extension, extract paths or accept arbitrary codecs/containers. Material,
object, real-recording and repeat identities are admitted; force, geometry,
impact/listener position, support, composition, calibration and transfer axes
remain unavailable.

## Frozen objects

| Object | Name | Material | Role | Selected clips |
| ---: | --- | --- | --- | --- |
| 6 | `Blue_Bowl` | Glass | target | `0, 20, 39` |
| 36 | `Portion_Cup_White` | Polycarbonate | reject parent | `0, 20, 39` |
| 80 | `Spoon_Holder` | Wood | reject parent | `0, 15, 29` |
| 94 | `Salad_Bowl` | Glass | target | `0, 20, 39` |
| 97 | `Stanford_Frisbee` | Plastic | reject parent | `0, 20, 39` |

All five objects share the same publisher/project/revision grouping and remain
in `dev`. Their fifteen recordings do not manufacture five independent source
groups and do not open calibration, holdout or shadow.

The bounded provenance review is SHA-256
`be8835bd8d754d6d9ec742829b0cc4474960632d3459e48c7ab1e725c2a17c30`.
No explicit redistribution grant was found on the reviewed official surfaces,
so `license_expression = NOASSERTION`, policy is
`external_research_only`, and all pages, metadata and media remain outside the
repository.

## Reproducibility and coverage

Two independently fetched cache roots produced byte-identical five-source
reports. The same five sources were then added to both existing complete cache
roots and the explicit-role corpus was rerun twice offline:

| Measurement | Before | After |
| --- | ---: | ---: |
| Publisher/project revisions | 5 | 6 |
| E3 objects | 15 | 20 |
| E3 recordings | 44 | 59 |
| Glass target groups | 7 | 9 |
| Glass recordings | 28 | 34 |
| Missing Glass groups | 9 | 7 |
| Reject-parent groups | 8 | 11 |
| Reject-parent recordings | 16 | 25 |
| Missing reject parents | 27 | 24 |

The combined report remains
`DevelopmentCoverageMeasured / NO_CORPUS_ADMISSION_AUTHORITY`. All 20 objects
and 59 recordings are in `dev`; calibration, holdout and shadow remain empty.
The full minima are still unmet, so `Pass`, PS-3 and AV-P0D remain disabled.

The existing generic Freesound source and identified reports also reproduce at
their previous hashes
`2dc5dcbda2cabe274be46a8c7a3b7a50ef645e8e00144a30f0cb221b9010d92a`
and
`63eeea4a724e4cf1ed0ae43836e4aad641b1114d97c2fefb2270669cf364c5a5`.
The new adapter therefore adds evidence without rewriting historical records.

## Failure controls

| Mutation | Manifest SHA-256 | Exact result |
| --- | --- | --- |
| Change Object 6 material from `Glass` to `Steel` | `22d347332f52e759c59a1b22eaf2a228b946537f864e9455688fda907c85c14b` | Rejected: official metadata does not back the object/material tuple; no report |
| Replace Object 6 clip 0 Git blob identity with zeroes | `ac58a769053181858eb31d91406d69a14da88975514d19ee941a7b4567b5a952` | Rejected: cached payload does not match the declared Git blob; no report |

Unit controls additionally reject arbitrary repository/recording URLs, a
mutated MP4 codec/container and truncated boxes.

## Consequence and next action

ObjectFolder-Real is no longer wholly discovery-only: its huge gzip archives
remain unsuitable, while the exact interactive-demo subset contributes five
bounded E3 objects. The smallest next PS-2 action is still evidence expansion:

1. acquire seven more stable Glass object groups and twenty-four non-Glass
   reject parents from published exact-download sources;
2. keep every new group in `dev` until the complete minima exist;
3. complement E3 identity/envelope evidence with claim-scoped E2/E1 geometry,
   excitation, support and transfer evidence;
4. only then freeze calibration/holdout/shadow and attempt PS-3.

Authored clip playback remains the mandatory production fallback, and this
checkpoint creates no public schema, shipped content role or P1 claim.
