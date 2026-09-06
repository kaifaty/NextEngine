# Physical sound V18 B0 — deterministic global baseline result

| Field | Value |
| --- | --- |
| Date | `2026-09-01` |
| Status | `COMPLETE / REPEAT_EXACT / B0_CAPABILITY_PASS` |
| Protocol | [V18 P0b](physical-sound-v18-p0b-hybrid-truth-protocol-2026-09-01.md) |
| Implementation | Git `51c8088e`; three deterministic Python modules and 11 focused tests |
| Allowed claim | The frozen scale-separated degree-two ridge is sufficient for the bounded synthetic object-global frequency/damping law |
| Product effect | None; no O/F/I execution, real-data credit, public schema, cooked atlas, demo or runtime inference |

## Execution

The implementation and focused tests were committed before any fresh B quality
value was evaluated. Two complete official executions then wrote only to
independent empty external directories:

- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v18-b0-deterministic-global-run-a`;
- `/home/kaifaty/.codex/experiments/nextengine/physical-sound/v18-b0-deterministic-global-run-b`.

Each directory contains `7` files and `153,489` file bytes. Every corresponding
file is byte-identical. The ordered complete-file-map digest is
`bffd8bf5671fcb066f50b90774f26fedaf8087f82fa740586c6fb92884291111`;
manifest SHA-256 is
`d4d9be53b4e0fcf54229f0aef358f8f9824dbe3783581824e7bbaf630ee66a67`
and report SHA-256 is
`5591459f7a4ba35bd0c0f9eadd5d13433fef066014defdaa78830f188222862f`.

The pinned environment was CPython `3.12.13`, NumPy `2.5.2`, SciPy `1.18.0`,
float64 CPU and one numerical-library thread. Real, source, protected, network,
V16-artifact and opened-V17-G0-artifact access counters are exactly zero.

## Frozen identity

- P0b protocol SHA-256:
  `216acd012108725cbcbcf5454f1db707a6ac688e182bfd9fa92d45082280475a`.
- Parent P0a protocol SHA-256:
  `39c1a1e94436a4ddc8b2f3755bab841c4e2b1e541cf2963134fc6445ba6bc77e`.
- Training identity root:
  `64ac4ac74364a20613abbdba10139fc60e019b62aeacb774050fd4d3e730a830`.
- Coefficients SHA-256:
  `3dddb1b00b55055fb6c78ae34638df20c03428c8b2b620295b827a5f4805b7ea`.
- Normalization SHA-256:
  `44c1312b2805207422c967d15d01b7ed54c85462547d390647da8308d7022b01`.
- Predictions SHA-256:
  `1bc5ebbf95fd545526d718cae643b70e1c7eaba0244220ddf755450aaaf69c74`.
- Corpus SHA-256:
  `a183a9850bc77fee734bb2880e5725b4175f0f91ca307d4d6f2a180aa65c8930`.

Implementation hashes embedded in both runs are:

| File | SHA-256 |
| --- | --- |
| `physical_sound_v18_b0_common.py` | `8ad95bdc11b9053e6fe473cccff03c7ea2cee94ce5612371e2a6f72127bcd65f` |
| `physical_sound_v18_b0_model.py` | `abf148ec8725d7ce242b1a3a6457555b6a9e2e1896c5cf772054244a810e2aa2` |
| `physical_sound_v18_b0_oracle.py` | `3cf7503daf2e8eb34a6ec5084335d4b5aee80b7a9f4af7ab424b04f28d9cf227` |

## Result

All `19/19` single-run gates pass, followed by the required full-directory
repeat gate.

| Endpoint | Overall fresh test | Interpolation | Scale transfer | Frozen gate |
| --- | ---: | ---: | ---: | ---: |
| Frequency median | `4.4340 cents` | `5.2724` | `3.3190` | `<=20` |
| Frequency p95 | `15.8907 cents` | `14.7570` | `17.4670` | `<=60` |
| Frequency mean | `5.6796 cents` | `6.2082` | `5.1511` | diagnostic |
| Damping median | `0.0012257` | `0.0015946` | `0.0011626` | `<=0.08` |
| Damping p95 | `0.0047012` | `0.0035638` | `0.0047012` | `<=0.20` |
| Damping mean | `0.0016415` | `0.0017477` | `0.0015353` | diagnostic |

Additional gates:

- every prediction is finite, ordered and inside frozen physical bounds;
- valid static OOD rejects `0/48`; maximum valid score is `0.002671` against
  threshold `0.25`;
- wrong material, support swap and corrupt scale reject `72/72` mutations;
- model serialization round-trips without changing any prediction;
- all seven files repeat byte-for-byte between the independent roots.

## Decision and remaining uncertainty

B0 passes. The deterministic global scaffold is now frozen and its exact model
hash may be consumed by O0/F0/I0 in the declared dependency order. This closes
the only uncertainty introduced by replacing V17's rejected neural global head:
the low-order ridge generalizes to fresh interpolation and scale-transfer rows
without any test-driven method selection.

This result does not show that the synthetic law is a real material formula and
does not justify neural complexity, source access, validator release, cooking
or runtime promotion. The next uncertainty is independent: whether graph-
geodesic coverage can preserve valid contacts while rejecting missing support
and ambient shortcuts on the still-unopened P0a meshes. O0 is the only next
authorized capability execution; F0 and I0 remain blocked.
