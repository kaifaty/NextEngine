# Physical sound corpus benchmark AV-P0B — 2026-08-27

| Field | Result |
| --- | --- |
| Scope | External hash-closed rigid-impact corpus/feature benchmark; no runtime, content or public contract |
| Command | `cargo run -p xtask -- physical-sound-benchmark --manifest <external-json> --output <external-empty-directory>` |
| Manifest | `nextengine.experimental-physical-sound-corpus-benchmark.manifest.v1` |
| Optional feature matrix | `nextengine.experimental-physical-sound-corpus-feature-matrix.v1` |
| Report | `nextengine.experimental-physical-sound-corpus-benchmark.report.v1` |
| Decision | Always `NoAcceptanceAuthority` |
| Current result | `AV_P0B_REAL_MATERIAL_BENCHMARK_MEASURED / NO_ACCEPTANCE_AUTHORITY` |

## Outcome

AV-P0B now has an executable external data boundary and a deterministic
grouped benchmark measured on public real recordings. It verifies every
consumed provenance-review record, WAV and external feature matrix by SHA-256;
keeps all corpus/model data outside Git; rejects object/family leakage across
`development`, `calibration`, `holdout` and `shadow`; and evaluates every
feature profile with the same one-nearest-neighbor baseline and split rules.

The report deliberately has no quality `Pass`. `Measured` means only that the
declared corpus and grouped tasks were evaluated. Invalid signal input becomes
`InputRejected`; incomplete tasks become `Unavailable`. Neither result can
change AV-P0A, accept generated audio, replace a clip or promote SPEC-45.

## Corpus-source check

The first measured subset comes from the public companion sounds for
[Aramaki et al., *Controlling the Perceived Material in an Impact Sound
Synthesizer*](https://kronland.fr/publications/controlling-the-perceived-material-in-an-impact-sound-synthesizer/).
The authors publish five recorded wood objects, five recorded metal objects
and five recorded glass objects, plus their modal-resynthesis and perceptually
tuned variants. The frozen local subset contains all 45 mono PCM16, 44.1 kHz
WAVs: 15 real recordings and 30 published generated variants.

The publication page does not state recording redistribution terms. This is
recorded as `NOASSERTION / unreviewed_public_research_source /
no_repository_or_distribution`, not interpreted as a permission grant. The
files are used only as a local external scientific benchmark; attribution,
source URL, revision and exact hashes are retained. No recording, checkpoint,
feature cache or dataset archive enters Git or a distributed product. This is
a provenance and packaging boundary, not a legal conclusion.

Two larger primary-source candidates remain useful for later spatial and force
coverage:

| Candidate | Primary evidence | Result |
| --- | --- | --- |
| ObjectFolder Real | The [official download page](https://objectfolder.stanford.edu/objectfolder-real-download) describes 100 real objects, 30–50 six-second impact recordings per object, mesh coordinates and force profiles. The first ten-object audio archive advertised there returned `Content-Length: 36,367,088,523` on 2026-08-27. | Technically suitable, but a 36 GB archive is disproportionate before the small material-only experiment establishes the value of spatial coverage. |
| REALIMPACT | The [official project](https://samuelpclarke.com/realimpact/) and [paper](https://ai.stanford.edu/~rhgao/publications/RealImpact.pdf) describe 150,000 recordings, 50 objects, five impact points, 600 microphone positions, force profiles and material labels. | Excellent location/listener control, but even one inspected object package is about 2.31 GB and includes large deconvolution arrays. Defer it until the spatial/force hypothesis is the next discriminator. |

AV-P0B removes per-sound listening, not source attribution, hash freezing or
provenance review. A future distributed dataset or product asset still needs
an independently reviewed redistribution basis.

## Manifest boundary

The current-only manifest declares:

- a bounded benchmark ID and sorted corpus sources;
- source revision, HTTPS origin, attribution, declared SPDX expression and
  measurement scope;
- an external hash-frozen review record with either reviewed status or explicit
  `unreviewed_public_research_source`; the latter must declare
  `no_repository_or_distribution`;
- sorted entries with source, partition, object, object-family, material,
  impact position, listener position, force band and origin;
- exact WAV path/hash for each entry;
- optional frozen embedding matrices with model revision/hash, dimension and
  `cosine` or `euclidean` distance.

All paths are manifest-relative and must resolve outside the repository.
Object IDs and object-family IDs may occur in exactly one partition. The real
development gallery must contain at least two materials, and each development
material must have at least two objects and two families. Every
controlled-impact development object must expose at least two impact positions.
A `material_identity_only` source instead uses literal `unspecified` for
impact/listener/force fields and creates no fake spatial or force evidence.
Generated and mutated entries cannot enter the development gallery.

The provenance status is a declaration backed by exact review-record bytes.
The tool validates its schema and hash but makes no independent legal judgment.

Minimal shape (the real manifest must contain all four partitions and enough
development objects/families/positions):

```json
{
  "schema": "nextengine.experimental-physical-sound-corpus-benchmark.manifest.v1",
  "benchmark_id": "rigid-impact-material-subset-v1",
  "corpus_sources": [{
    "id": "public-research-source",
    "revision": "publication-companion-sounds-2026-08-27",
    "source_url": "https://example.org/official-publication",
    "attribution": "Authors, title, publication and source URL",
    "measurement_scope": "material_identity_only",
    "license": {
      "spdx_id": "NOASSERTION",
      "review_status": "unreviewed_public_research_source",
      "redistribution": "no_repository_or_distribution",
      "review_record": {"path": "provenance-review.txt", "sha256": "<sha256>"}
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
    "id": "glass-001-real",
    "source_id": "public-research-source",
    "partition": "development",
    "object_id": "glass-001",
    "object_family_id": "thin-glass-vessel",
    "material": "glass",
    "impact_position_id": "unspecified",
    "listener_position_id": "unspecified",
    "force_band": "unspecified",
    "origin": {"kind": "real"},
    "audio": {"path": "wav/glass-001-real.wav", "sha256": "<sha256>"}
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
counts, origin-group metrics and deterministic per-entry nearest-gallery
predictions with first/second distances and margin. Generated revisions and
mutation families are reported separately and never enter the gallery, giving
the baseline explicit generator-out and mutation-out measurements when those
entries exist.

## Frozen experiment and result

The split was frozen before scoring. For each material, two real objects form
`development`; the third, fourth and fifth objects form `calibration`,
`holdout` and `shadow`. The three non-development partitions also contain the
publication's modal-resynthesis and perceptual-tuning variants. Each object is
its own family. This small benchmark measures broad material identity only; it
does not measure impact position, listener position, force, steel subtype or
subjective naturalness.

The learned representation is the official Microsoft
[BEATs](https://github.com/microsoft/unilm/tree/master/beats) Iter3+ AS2M
pretrained checkpoint. Extraction is external and deterministic on CPU:
44.1/48 kHz PCM is resampled to 16 kHz, frame embeddings are mean-pooled and
L2-normalized, and the 768-dimensional matrix is frozen before Rust scoring.
The checkpoint SHA-256 is
`d43cbfad4d7b56381c061d7a24774f908d4d94c72961f6eb1d9090ff18cd8d34`.

| Profile / origin | Development leave-object | Calibration | Holdout | Shadow |
| --- | ---: | ---: | ---: | ---: |
| Classical / real | `3/6` | `2/3` | `2/3` | `3/3` |
| BEATs / real | `6/6` | `3/3` | `3/3` | `3/3` |
| Classical / published modal resynthesis | n/a | `2/3` | `2/3` | `2/3` |
| BEATs / published modal resynthesis | n/a | `1/3` | `2/3` | `2/3` |
| Classical / published perceptual tuning | n/a | `2/3` | `2/3` | `2/3` |
| BEATs / published perceptual tuning | n/a | `2/3` | `2/3` | `2/3` |

Thirteen current Next Engine candidates were then added to `shadow` without
changing the gallery or partitions. Their broad-material results are:

| Candidate family | Classical | BEATs | Diagnostic consequence |
| --- | ---: | ---: | --- |
| Wood center/edge/corner | `3/3` | `3/3` | Retain as a successful control. |
| Steel center/edge/corner, evaluated as broad `metal` | `0/3` | `0/3` | BEATs maps all three to glass; current steel is not automatically admissible. |
| Glass variants and selected Q30 | `3/7` | `5/7` | Selected Q30, bottle, thin goblet and Glass-H center/corner pass BEATs; Glass-H edge and thick jar map to wood. |
| All Next Engine candidates | `6/13` | `8/13` | Useful failure clustering, but too weak and narrow for acceptance. |

The selected Q30 glass candidate is the strongest learned-identity control: it
maps to glass with cosine distance `0.388559`; the classical head maps it to
wood. The disagreement proves why one head cannot be acceptance authority.
The complete report's SHA-256 is
`bee40d184e2b8e3cf65485e220f460e06bdcef99e8cf06dc9f92aa55ebd5ce68`;
the 46-entry manifest is
`dee5ce85bb0e57d6228eafbe983a3b4672226cc0ac640b433ccbddcd231e74d7`;
the BEATs matrix is
`9422721294ef7cb24453b7e1fd57ea041d81ef9b5899cbdd8cbaa833eabcca15`.

## Verification and limitations

Behavioral coverage includes:

- strict manifest parsing and rejection of object-partition leakage;
- rejection of generated development-gallery entries;
- exact sorted feature-matrix coverage and dimensions;
- grouped held-out classification with a real-only gallery;
- end-to-end external provenance/WAV hash verification;
- byte-identical repeated reports;
- explicit `NoAcceptanceAuthority` in the serialized report.

The repository contains no dataset, recording, feature cache or model artifact.
The automated end-to-end test uses generated test WAVs only; the separately
hash-frozen experiment above supplies the real-corpus measurement. Repeated
feature extraction and repeated Rust reports are byte-identical.

## Next admission step

1. Use the frozen BEATs/classical failure clusters to tune steel while retaining
   wood and Q30 as non-regression controls; do not optimize on `holdout` or the
   frozen real `shadow` objects.
2. Add a second independent representation or a small rigid-impact specialist
   with exact model/hash lineage. Compare its generator/mutation errors rather
   than selecting it from development accuracy alone.
3. Admit a bounded ObjectFolder Real or REALIMPACT subset only when spatial,
   listener or force coverage is the next hypothesis; keep it external and
   preserve truthful measurement scope.
4. Expand grouped real and mutation holdouts before fitting any selective-risk
   threshold. The current 15 objects cannot establish a useful false-pass
   bound.
5. Only AV-P0C may emit a calibrated automatic `Pass`; it must retain AV-P0A
   hard/causal rejection, frozen shadow evidence and automatic clip fallback.
