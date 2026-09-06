# Physical sound V46 D0 — synthetic-source preflight protocol

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| State | `FROZEN_BEFORE_DATASET_PAYLOAD_ACCESS` |
| Sources | NISR v5 and VibraVerse at the exact R0 revisions |
| Scope | Metadata, source-tree and generation-provenance preflight only |
| Excluded authority | Dataset-row admission, model training, real parent/project power, validator calibration, protected admission, cooker, demo, public contract and runtime |

## Falsifiable question

For each source independently, does its exact public revision provide enough
lineage to interpret a small modal label as the output of a specified generator
rather than as an opaque number?

A source is `SyntheticTeacherTrusted` only when all of these facts are bound:

1. exact public dataset revision and complete sibling-path inventory;
2. upstream geometry/asset identity and alias component;
3. executable generator or complete generation manifest;
4. exact generator revision, solver/configuration and material/excitation units;
5. artifact-to-generator-revision binding for the candidate sample;
6. a value-independent bounded sample rule frozen below.

Missing evidence returns `SyntheticTeacherUntrusted` for that source before any
NPZ, tar member, waveform or modal value is read. Failure of one source neither
rejects nor grants substitute credit to the other.

## Exact source identities

### NISR v5

- dataset: `BumsooKim00/nisr-dataset`;
- commit: `20368791bcd7829e04ae3eb07c10aa0bb370e38a`;
- README SHA-256:
  `fb6f6382597289e4bc0a7e959fda7a85034d290e23b079497272e1a78a645325`;
- alias component: `stanford-objectfolder-derived-nisr-synthetic`;
- expected complete sibling count: `92,299`;
- path-only sibling root: `4155cc469d13732714d2db991ab84a7064f8603bb52a1fd46979a2613f62e4a7`.

The exact README describes FEM/LOBPCG generation and declares a GitHub
`GENERATE.md` as the full pipeline. D0 requires that declared document to be
public and tied to an exact generator revision. An HTTP error, a living branch
without a dataset-to-code binding, or prose without the per-artifact generator
identity fails the gate.

If every provenance gate passes in a future immutable successor, the frozen
sample is object `1`, material `Glass`, consisting only of its voxel record,
modal feature record and their required manifest/parameter companions, capped
at `8 MiB`. This rule is not authorization to fetch the sample in current D0.

### VibraVerse

- dataset: `technetium66/VibraVerse`;
- commit: `8099f137e9a9171e758c0528fb4a38a7b5d6aab2`;
- README SHA-256:
  `ed86532abb7113098346dda7a9271ea3a77a01c87b40dffe04b291ce9e522534`;
- material-table SHA-256:
  `568c5cda982f29df911032c691c22c076b8d9af4d231d2d8c145efe55771754c`;
- alias component: `vibraverse-objaverse-generated-synthetic`;
- expected complete sibling count: `49`;
- path-only sibling root: `6202e6d296439b75c684bc05dd038a9d73bba9ca326a9cf709a00a3e4b178777`.

D0 accepts neither the dataset card's high-level causal diagram nor the
presence of geometry/audio tar files as complete generation lineage. The exact
revision must expose executable generation code or a complete manifest binding
upstream asset, material assignment, tetrahedralization, modal solver,
damping/excitation synthesis and artifact identity.

If every provenance gate passes in a future immutable successor, the frozen
sample scans at most the first `64 MiB` HTTP range of
`objaverse/dataset_shard_0000.tar` and chooses the first complete object in tar
order. It does not search for a favourable shape, material or spectrum. Current
D0 does not authorize that range read.

## Metadata acquisition boundary

Exactly eight metadata requests are declared:

- dataset revision API, bounded root-tree API and exact README for each source;
- NISR declared generation-document HEAD request;
- exact VibraVerse material-parameter table.

The external metadata directory contains these eight regular files and no
dataset payload. API counters such as downloads or likes are ignored because
they are mutable; the owner projects only exact repository ID/commit,
public/gated state, stable tags, sibling paths, tree entries and content hashes.
All metadata and outputs remain outside the repository.

The sibling root is SHA-256 over each unique path sorted by Unicode code-point
order and encoded as UTF-8 `<path><LF>`. It proves the path inventory observed
at the exact revision without committing the multi-megabyte API response.

## Current predeclared decisions

The profile expects these fail-closed outcomes from the already-visible
metadata, before any data values are opened:

| Source | Decision | Reason |
| --- | --- | --- |
| NISR v5 | `SyntheticTeacherUntrusted` | `DeclaredGenerationDocumentUnavailable` |
| VibraVerse | `SyntheticTeacherUntrusted` | `GenerationLineageIncomplete` |

The expectation does not make the decision true. The owner must mechanically
verify the exact revision, README, source tree and relevant generation evidence.
Any drift rejects publication instead of silently accepting a different source.

## Output and access accounting

Two runs over the same external metadata must create byte-identical:

- `sources.json` with separate observations, gates, decision and sample state;
- `access.json` with declared metadata access and every dataset-payload counter;
- `report.json` with the aggregate `NoTrustedSyntheticTeacher` decision.

The following counters must remain zero: archive-range bytes, dataset payload
files, decoded modal values, decoded audio/PCM, waveform bytes, model values and
protected values. `NoTrustedSyntheticTeacher` is a legitimate D0 result. It
routes D1/B0 to analytic plus Clatter controls and leaves real-source/validator
growth active; it does not authorize a weaker synthetic claim.

## Reconsideration

- NISR may be re-preflighted only when a public exact generator revision and
  dataset-to-generator/artifact binding become available.
- VibraVerse may be re-preflighted only when executable generation code or a
  complete revisioned manifest binds its upstream assets, solver/configuration
  and output artifacts.
- A successor uses a new profile and result. It never edits this decision in
  place or opens the current preregistered sample retroactively.
