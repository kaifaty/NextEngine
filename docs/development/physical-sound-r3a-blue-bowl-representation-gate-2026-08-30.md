# Physical sound R3A Blue Bowl representation gate

| Field | Value |
| --- | --- |
| Date | 2026-08-30 |
| Status | `R3A_V1_COMPLETE / REJECT_REPRESENTATION / NO_NEURAL_TRAINING` |
| Scope | One REALIMPACT Blue Bowl, five impact positions, one exact published listener condition; external research only |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |
| Roadmap | [Physical sound synthesis Roadmap V4](../plans/physical-sound-synthesis-roadmap.md) |

## Decision

Reject the first R3A compact representation. The hash-closed internet source,
five contact positions, fit/development/field-holdout roles and bounded
extraction all reproduce. The query-seeing development oracle also reproduces,
but neither the 512-scalar modal-plus-residual record nor its equal-budget
sparse-DCT alternative preserves the real transfer response well enough to
beat the nearest fit contact and pass the absolute level, spectrum, envelope,
modal-frequency and decay limits.

This is a representation rejection, not a neural-model result. No optimizer
ran, no neural checkpoint exists, and R3B training remains prohibited. The
fifth Blue Bowl contact, method holdout and admission shadow remain unopened.
Authored clips remain mandatory.

## Source and split freeze

The selected source is the official REALIMPACT `6_Bowl` archive at repository
revision `fca2bd6cbb7e9f96ac61328d2a0d51594bf01987`. The source publishes
force-deconvolved float32 transfer responses rather than raw impact recordings.
R3A keeps that semantic, aligns each response peak to sample 512, applies one
fit-only scale unchanged to development and evaluates the first three seconds
at 48 kHz.

The exact listener condition is azimuth `0°`, distance offset `0 mm` and
microphone `7`. It is a sampling policy, not an arbitrary-radiation claim.
Metadata-only selection fixes source-ordered impacts before any selected audio
read:

| Impact | Mesh vertex | Audio row | Role | Position, metres |
| ---: | ---: | ---: | --- | --- |
| 0 | `35950` | `7` | fit | `[-0.03610739, -0.0520688, 0.03753229]` |
| 1 | `21823` | `607` | fit | `[0.00936602, -0.0155827, 0.00078345]` |
| 2 | `10221` | `1207` | fit | `[-0.05887816, -0.0508747, 0.0790166]` |
| 3 | `25307` | `1807` | representation development | `[-0.04113073, -0.0616422, 0.06258844]` |
| 4 | `31104` | `2407` | field holdout | `[0.04350088, -0.064883, 0.06707589]` |

Future calibration, method-holdout and admission-shadow groups are commitments
only: REALIMPACT `22_Cup`, REALIMPACT `33_WoodWineGlass` and ObjectFolder Real
`60_Beer_Glass`, respectively. None is opened by this experiment.

The [official ObjectFolder Real download page](https://objectfolder.stanford.edu/objectfolder-real-download)
confirms 100 real objects, 30–50 six-second impacts per object, contact
coordinates, force profiles and meshes. Its first ten-object acoustic batch is
available and header-pinned at `36,367,088,523` bytes, but R3A does not download
that 36.37 GB payload merely to satisfy a source-count target. It remains a
richer later source candidate.

## Reproducible acquisition and seal

Two zero-audio preflights independently fetch and validate only the official
metadata arrays and mesh. Before audio access, each manifest binds SHA-256 for
the shared boundary, preflight, streaming extraction and oracle runners; later
stages fail closed if any implementation hash changes. The runs produce
identical artifacts:

| Artifact | SHA-256 or exact value |
| --- | --- |
| Corpus manifest A/B | `6ad5b42c96dcbc5ef901e66598eacb9e140630ebf8030864e12845863f16847b` |
| Source-preflight report A/B | `7e8c89ac51665035c506dfecdce521b3067bc0f13f74904616bb610c2837983d` |
| REALIMPACT metadata compressed bytes | `683,779` |
| Preflight audio bytes | `0` |
| Full compressed audio-member SHA-256 | `9a48abe662040a463e7de818db6fc881ccd79b09a550500c9ec9784d12e1ee3d` |
| Authorized contacts array A/B | `1971f01a202cbcbe36d7ac7f16c92b6d559f06a432eb948ae782c1ae43a11ba3` |
| Extraction report A/B | `4d364a28abd6c1fb8c35b8e8e4c7548c20f27c2a0ee636511ba8d3df74e6b827` |

The raw-deflate member has the exact pinned size `2,393,833,339` bytes. The
streaming decoder reads `1,459,617,792` compressed bytes and stops after
`1,664,915,008` uncompressed bytes, the exact end of development row 1807.
The sealed row begins only at uncompressed byte `2,216,510,148`; therefore its
decoded sample count is exactly zero. Downloading opaque compressed bytes does
not expose the holdout waveform to analysis or selection.

All large source payloads, decoded contacts, WAVs and reports remain in the
external experiment store and outside Git.

## Frozen representation oracle

The target baseline is the nearest fit contact by Euclidean mesh-position
distance. The development contact is closest to fit contact 2 at
`0.0264725 m`. The two query-seeing compact representations have the same
512-scalar budget:

1. 32 damped sinusoidal modes, each encoding frequency, damping and cosine/
   sine gain, plus 192 sparse residual DCT bins encoded as index/value pairs;
2. 256 sparse target DCT bins encoded as index/value pairs.

The modal parameters and residual coefficients may see the development target.
That makes this a representation-capacity oracle, not a learned predictor.
Pass requires all five absolute limits, at least four strict improvements over
nearest fit, a normalized error sum below 80% of baseline and a bounded
non-regression on any already-preserved endpoint.

Two complete oracle runs produce identical report
`34567bc2b984837884eeeb5582e2757c6fc8779588699f3a37f23863cfa8ef01`
and identical four-WAV artifact hashes.

| Endpoint; lower is better | Nearest fit | Modal + residual | Sparse DCT | Absolute limit |
| --- | ---: | ---: | ---: | ---: |
| RMS-level error | `5.3597 dB` | `1.4444 dB` | `6.5815 dB` | `0.5 dB` |
| Gain-matched multiresolution spectrum RMSE | `7.8473 dB` | `8.8587 dB` | `9.8211 dB` | `4.0 dB` |
| Normalized envelope RMSE | `0.05141` | `0.01306` | `0.04001` | `0.20` |
| Median modal-frequency error | `149.43 cents` | `6513.21 cents` | `5119.74 cents` | `100 cents` |
| Relative T60 error | `0.3539` | `2.5961` | `3.7487` | `0.35` |

Modal plus residual improves only level and envelope; it fails four absolute
limits and loses three direct comparisons. Sparse DCT improves only envelope
and fails four absolute limits. The exact decision is
`REJECT_REPRESENTATION`.

## Interpretation

The failure is not explained by lack of access to the development waveform:
both representations are allowed to encode it directly. It therefore isolates
the compact record and extraction method rather than coordinate interpolation
or neural optimization.

The 32 local STFT peaks plus independently estimated exponential damping do
not carry this dense, high-frequency glass response. Sparse whole-window DCT
also spends too much of its small budget on timing and broad-band transient
structure. Merely adding nearby peaks, DCT bins, seeds or thresholds on this
opened development contact would be query-informed tuning and is not
authorized.

The result does not prove that modal contact fields are infeasible. It rejects
this pole estimator/residual codec/budget combination on this development
contact. A genuinely different next hypothesis needs either:

- a new unopened source/object and a structured complex-pole or
  multiresolution transient representation frozen before its development
  contact is opened; or
- a bounded way to access an ObjectFolder Real object with 30–50 contacts,
  enabling a learned or data-derived codec split without downloading an
  indiscriminate 36 GB batch.

The [REALIMPACT paper](https://openaccess.thecvf.com/content/CVPR2023/papers/Clarke_RealImpact_A_Dataset_of_Impact_Sound_Fields_for_Real_Objects_CVPR_2023_paper.pdf)
and [official preprocessing source](https://github.com/samuel-clarke/RealImpact/tree/fca2bd6cbb7e9f96ac61328d2a0d51594bf01987)
remain the authority for the selected transfer semantics and acquisition axes.
External availability does not authorize redistribution or relax Next Engine's
hash, provenance and role-isolation rules.

## Roadmap consequence

R3A V1 is complete and rejected. R3B exact-object neural training stays
blocked. The next commit boundary is a bounded R3A V2 research cycle that
freezes a materially different representation and a new unopened development
projection, or records `DATA_INSUFFICIENT` if no bounded published source can
support it. It must not reuse Blue Bowl row 1807 for selection, open row 2407,
or touch method holdout/admission shadow.
