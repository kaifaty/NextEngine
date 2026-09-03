# Physical sound V46 D0 — synthetic-source preflight result

| Field | Value |
| --- | --- |
| Date | `2026-09-03` |
| Status | `COMPLETE / REPEAT_EXACT / NO_TRUSTED_SYNTHETIC_TEACHER / ZERO_DATASET_PAYLOAD_ACCESS` |
| Decision | `NoTrustedSyntheticTeacher` |
| Sources | NISR v5 and VibraVerse, evaluated independently |
| Claim | Metadata/generation provenance only; no dataset row, training, validator, protected, cooker, demo or runtime authority |
| Next | D1 external corpus compiler with zero admitted external synthetic-modal rows; B0 retains analytic and Clatter controls |

## Pre-access seal

Commit `b6b74d5d` froze the [protocol](physical-sound-v46-d0-synthetic-source-preflight-protocol-2026-09-03.md),
profile, offline owner and failure tests before any dataset payload, archive
range, NPZ value, modal label or audio sample was opened.

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| [`physical-sound-v46-d0-source-preflight.v1.json`](../../lab/profiles/physical-sound-v46-d0-source-preflight.v1.json) | `6,131` | `7236b6071f09944ad64ca6d690f66a48eb89bad8b11eeaadb5e6eb7f18276fc5` |
| [`physical_sound_v46_d0_source_preflight_v1.py`](../../lab/scripts/physical_sound_v46_d0_source_preflight_v1.py) | `26,821` | `26ee0ebac9f8801c9e8f6c8a8b7e610b51cbb7abf5131db6784bc1f89e92efee` |
| [Protocol](physical-sound-v46-d0-synthetic-source-preflight-protocol-2026-09-03.md) | `6,278` | `214d394f2afa49722e039e242eb08ef82cbbeba608734f224a997d621abeec78` |
| [`test_physical_sound_v46_d0_source_preflight_v1.py`](../../lab/tests/test_physical_sound_v46_d0_source_preflight_v1.py) | `7,868` | `5b60ffed88864a65fac9bd15b5313809d6c17d8c52a6162d34a9fa86a6031616` |

The profile also binds the exact V45 R0 source ledger and T0 Recipe V3
profiles. The owner has no network, NumPy, audio, ML or runtime import. It reads
only a fixed external eight-file metadata inventory and publishes only after
both source decisions and all zero-payload gates match the sealed expectation.

## Exact metadata acquisition

After the seal, eight requests fetched only exact revision metadata:

- NISR Hugging Face revision API, bounded root-tree page and README, plus HEAD
  of the generation-document URL declared by that README;
- VibraVerse Hugging Face revision API, root tree, README and material table.

The run read `4,558,989` external metadata bytes. Mutable API counters were not
projected into source identity. The stable facts are repository ID, commit,
public/gated state, required tags, complete sibling paths, tree entries and
content hashes.

No dataset payload request was made. In particular, D0 did not request the
NISR object-1 NPZ files or any byte range from a VibraVerse tar shard.

## Per-source results

### NISR v5 — `SyntheticTeacherUntrusted`

[NISR](https://huggingface.co/datasets/BumsooKim00/nisr-dataset) is public and
ungated at commit
`20368791bcd7829e04ae3eb07c10aa0bb370e38a`. The exact inventory contains
`92,299` paths with path root
`4155cc469d13732714d2db991ab84a7064f8603bb52a1fd46979a2613f62e4a7`;
the README hash remains
`fb6f6382597289e4bc0a7e959fda7a85034d290e23b079497272e1a78a645325`.

The README states only the high-level chain `FEM (LOBPCG) -> modal feature ->
modal sound` and declares an external GitHub `GENERATE.md` as the full
pipeline. The post-seal HEAD request returned HTTP `404`. The exact dataset
revision also contains no generator source or manifest, and no artifact binds
the dataset to an exact generator revision or upstream ObjectFolder asset.

Failed gates:

- executable or complete generation manifest;
- upstream asset identity;
- artifact-to-generator revision binding.

Reason: `DeclaredGenerationDocumentUnavailable`. The preselected object-1
Glass sample remains `NotOpenedBecauseGenerationProvenanceFailed`.

### VibraVerse — `SyntheticTeacherUntrusted`

[VibraVerse](https://huggingface.co/datasets/technetium66/VibraVerse) is public
and ungated at commit
`8099f137e9a9171e758c0528fb4a38a7b5d6aab2`. The exact inventory contains `49`
paths with path root
`6202e6d296439b75c684bc05dd038a9d73bba9ca326a9cf709a00a3e4b178777`.
README and material-table hashes reproduce as
`ed86532abb7113098346dda7a9271ea3a77a01c87b40dffe04b291ce9e522534`
and
`568c5cda982f29df911032c691c22c076b8d9af4d231d2d8c145efe55771754c`.

The revision contains those two descriptive files plus `.gitattributes` and
`46` tar shards. It contains no executable generator, environment/configuration
record or generation manifest. The high-level paper/card description does not
bind individual upstream/generated assets, tetrahedralization, solver,
damping/excitation configuration and output artifacts to exact revisions.

Failed gates:

- executable or complete generation manifest;
- upstream asset identity;
- artifact-to-generator revision binding.

Reason: `GenerationLineageIncomplete`. The preselected first-complete-object
scan of the first `64 MiB` of shard zero remains
`NotOpenedBecauseGenerationProvenanceFailed`.

## Repeat-exact execution

Two fresh output directories over the post-seal external metadata have no
`diff -qr` difference:

| Artifact | Bytes | SHA-256 |
| --- | ---: | --- |
| `access.json` | `536` | `202b653246c91ee1d96682de0cb0a8c718355364c5eb033f671b8dd30841931c` |
| `report.json` | `1,464` | `36fcbadb973315024b63d60c1ea6371f58a415fb3459c56f949d7cfe21848b08` |
| `sources.json` | `3,929` | `a2659461b874c0059570be8c075e854c9f47004f3b91ffa0ef2f14dbf1c8153b` |

All six aggregate gates pass: source decisions remain separate, expected
fail-closed decisions reproduce, no bulk or dataset payload access occurs, no
source becomes trusted with missing provenance and the sealed profile/owner
bindings match.

All eight dataset-value counters are zero: archive-range bytes, audio files,
dataset payload files, modal numeric values, model values, PCM samples,
protected values and waveform bytes.

## Focused verification

Six focused tests pass. They cover canonical profile/dependency bindings,
NISR's broken declared generation document, VibraVerse shard-vs-lineage
separation, revision/inventory drift, access-policy/unknown-field failure,
repository-output rejection and the absence of network/audio/model imports.

The physical-sound boundary scan still reports the known pre-existing
`SOURCE_LAYOUT_ESCAPE_HATCH` in
`tools/xtask/src/physical_sound_registry_command/realimpact_transfer_fixture.rs`;
D0 adds no new finding.

## Consequence

D0 is complete, but neither external dataset may supervise Recipe V3. This is
not a dead end and does not weaken the evidence policy:

- D1 builds the external corpus compiler with zero admitted NISR/VibraVerse
  rows and must handle this empty synthetic-teacher lane explicitly;
- B0 compares the existing analytic owner and 36-group Clatter prior;
- the Delft structural-transfer and real-acoustic/validator source tracks remain
  independent next evidence opportunities;
- no NISR/VibraVerse bulk download, bounded sample retry or nearby source
  reinterpretation is allowed until the exact missing lineage appears.

Real support remains `71/105`, deficit `34`. PSEL, real fitting, validator
qualification, protected admission, cooker, demo and runtime/product authority
remain blocked; authored clips remain mandatory.
