# Physical sound corpus benchmark AV-P0B — 2026-08-27

| Field | Result |
| --- | --- |
| Scope | External hash-closed rigid-impact corpus/feature benchmark; no runtime, content or public contract |
| Command | `cargo run -p xtask -- physical-sound-benchmark --manifest <external-json> --output <external-empty-directory>` |
| Manifest | `nextengine.experimental-physical-sound-corpus-benchmark.manifest.v1` |
| Optional feature matrix | `nextengine.experimental-physical-sound-corpus-feature-matrix.v1` |
| Report | `nextengine.experimental-physical-sound-corpus-benchmark.report.v1` |
| Decision | Always `NoAcceptanceAuthority` |
| Current result | `AV_P0B_FOUNDATION_IMPLEMENTED / REAL_CORPUS_LICENSE_REVIEW_BLOCKED` |

## Outcome

AV-P0B now has an executable external data boundary and a deterministic
grouped benchmark. It verifies every consumed license-review record, WAV and
external feature matrix by SHA-256; keeps all corpus/model data outside Git;
rejects object/family leakage across `development`, `calibration`, `holdout`
and `shadow`; and evaluates every admitted feature profile with the same
one-nearest-neighbor baseline and split rules.

The report deliberately has no quality `Pass`. `Measured` means only that the
declared corpus and grouped tasks were evaluated. Invalid signal input becomes
`InputRejected`; incomplete tasks become `Unavailable`. Neither result can
change AV-P0A, accept generated audio, replace a clip or promote SPEC-45.

## Corpus-source check

The two best primary-source candidates remain useful but are not yet admitted:

| Candidate | Primary evidence | Result |
| --- | --- | --- |
| ObjectFolder Real | The [official download page](https://objectfolder.stanford.edu/objectfolder-real-download) describes 100 real objects, 30–50 six-second impact recordings per object, mesh coordinates and force profiles. The first ten-object audio archive advertised there returned `Content-Length: 36,367,088,523` on 2026-08-27. | Technically suitable but too large for an unreviewed trial. The [ObjectFolder repository](https://github.com/rhgao/ObjectFolder) declares CC BY 4.0 for ObjectFolder, while the separate Real download page does not explicitly state that the same grant covers its recording archives. Do not infer archive rights from proximity; a corpus review record must close this. |
| REALIMPACT | The [official project](https://samuelpclarke.com/realimpact/) and [paper](https://ai.stanford.edu/~rhgao/publications/RealImpact.pdf) describe 150,000 recordings, 50 objects, five impact points, 600 microphone positions, force profiles and material labels. | Excellent location/listener control, but the [official repository](https://github.com/samuel-clarke/RealImpact) says raw-data/code packaging remains incomplete. Its MIT `LICENSE` names software and does not explicitly license the recording archive. Do not admit the WAVs until dataset terms are reviewed. |

This is not a legal conclusion. It is the conservative engineering boundary:
license text or a project-owner clarification must be reviewed once per frozen
corpus subset. AV-P0B removes per-sound listening, not provenance review.

## Manifest boundary

The current-only manifest declares:

- a bounded benchmark ID and sorted corpus sources;
- source revision, HTTPS origin, attribution and SPDX expression;
- an external hash-frozen review record with exact status
  `approved_external_benchmark_only` and redistribution `external_only`;
- sorted entries with source, partition, object, object-family, material,
  impact position, listener position, force band and origin;
- exact WAV path/hash for each entry;
- optional frozen embedding matrices with model revision/hash, dimension and
  `cosine` or `euclidean` distance.

All paths are manifest-relative and must resolve outside the repository.
Object IDs and object-family IDs may occur in exactly one partition. The real
development gallery must contain at least two materials, and each development
material must have at least two objects and two families. Every development
object must expose at least two impact positions. Generated and mutated entries
cannot enter the development gallery.

The license status is a declaration backed by exact reviewed bytes. The tool
validates its schema and hash but makes no independent legal judgment.

Minimal shape (the real manifest must contain all four partitions and enough
development objects/families/positions):

```json
{
  "schema": "nextengine.experimental-physical-sound-corpus-benchmark.manifest.v1",
  "benchmark_id": "rigid-impact-real-subset-v1",
  "corpus_sources": [{
    "id": "reviewed-source",
    "revision": "frozen-release-v1",
    "source_url": "https://example.org/official-dataset",
    "attribution": "Required dataset attribution",
    "license": {
      "spdx_id": "CC-BY-4.0",
      "review_status": "approved_external_benchmark_only",
      "redistribution": "external_only",
      "review_record": {"path": "license-review.txt", "sha256": "<sha256>"}
    }
  }],
  "external_feature_sets": [{
    "id": "beats",
    "model_revision": "official-checkpoint-revision",
    "model_sha256": "<weights-sha256>",
    "dimensions": 768,
    "distance": "cosine",
    "matrix": {"path": "beats-features.json", "sha256": "<sha256>"}
  }],
  "entries": [{
    "id": "glass-001-position-00",
    "source_id": "reviewed-source",
    "partition": "development",
    "object_id": "glass-001",
    "object_family_id": "thin-glass-vessel",
    "material": "glass",
    "impact_position_id": "position-00",
    "listener_position_id": "listener-00",
    "force_band": "measured-medium",
    "origin": {"kind": "real"},
    "audio": {"path": "wav/glass-001-position-00.wav", "sha256": "<sha256>"}
  }]
}
```

An external feature matrix uses its own schema, repeats the feature-set ID and
contains sorted `{"id": ..., "values": [...]}` records matching every corpus
entry exactly.

## Feature and task baseline

The built-in `classical-av-p0b-v1` vector reuses the AV-P0A WAV parser and
analysis path. It contains three gain-normalized log-spectrum signatures plus
bounded spectral centroid/bandwidth/flatness, crest, attack, temporal-centroid,
modal-count and band-decay scalars. The deterministic baseline is one nearest
neighbor; ties use the lexicographically smallest gallery entry ID.

Hash-frozen matrices allow the same tasks to compare BEATs, Human-CLAP,
Audiobox Aesthetics or a future specialist without adding Python, Torch,
network access or model weights to Cargo. The matrix must contain exactly one
finite fixed-dimensional vector for every sorted manifest entry. Missing
optional components are reported as `NotProvided`, never downloaded.

Each feature profile receives these tasks:

| Task | Leakage guard |
| --- | --- |
| development leave-object-out material | Exclude the target `object_id` from the real development gallery. |
| development leave-family-out material | Exclude the target `object_family_id`. |
| development leave-position-out object | Exclude the target object/impact-position group. |
| development leave-listener-out object | Exclude the target object/listener-position group; `Unavailable` for a single-listener corpus. |
| development leave-force-out object | Exclude the target object/force-band group; `Unavailable` when force bands are absent. |
| calibration material | Train/gallery on real development only. |
| holdout material | Train/gallery on real development only; holdout never tunes the feature extractor. |
| shadow material | Train/gallery on real development only; shadow never tunes model/threshold selection. |

Reports include micro and macro accuracy, confusion counts, unavailable-target
counts and origin-group metrics. Generated revisions and mutation families are
reported separately and never enter the gallery, giving the baseline explicit
generator-out and mutation-out measurements when those entries exist.

## Verification and limitations

Behavioral coverage includes:

- strict manifest parsing and rejection of object-partition leakage;
- rejection of generated development-gallery entries;
- exact sorted feature-matrix coverage and dimensions;
- grouped held-out classification with a real-only gallery;
- end-to-end external license/WAV hash verification;
- byte-identical repeated reports;
- explicit `NoAcceptanceAuthority` in the serialized report.

The repository contains no dataset, recording, feature cache or model artifact.
The end-to-end test uses generated test WAVs only. It proves the benchmark
boundary and leakage guards, not real-corpus quality.

## Next admission step

1. Obtain explicit dataset-license coverage or a project-owner clarification
   for one small ObjectFolder Real glass/steel/wood subset; record the review
   externally and freeze every selected WAV hash.
2. Freeze object/family partitions before inspecting scores, including at
   least two positions per development object and untouched shadow objects.
3. Run the classical baseline, then externally cache official BEATs,
   Human-CLAP and Audiobox matrices with exact model/hash lineage.
4. Compare macro errors and generator/mutation failures. AV-P0B still cannot
   produce quality `Pass`.
5. Only AV-P0C may add a specialist and a separately calibrated selective-risk
   threshold; it must retain AV-P0A hard/causal rejection and clip fallback.
