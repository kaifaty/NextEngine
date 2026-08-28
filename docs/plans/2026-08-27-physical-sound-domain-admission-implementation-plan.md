# Physical sound domain admission implementation plan

## Outcome

Build an external, hash-closed research loop that can fit mathematical source
models, automatically admit only bounded acoustic domains, and select authored
clip fallback for uncertainty. The plan implements SPEC-45 P0 evidence first;
it does not activate a roadmap stage, add a shipping content schema or replace
the current `AudioMixerV1` clip path.

Milestone order, activation gates and the product handoff are summarized in the
[physical-sound subsystem roadmap](physical-sound-synthesis-roadmap.md).

The first target matrix is deliberately bounded:

- rigid impacts only;
- thin glass vessel, thin metal vessel/shell and dry hardwood block families;
- exact declared geometry/support revisions;
- multiple force bands and impact positions;
- fixed listener/radiation conditions until spatial evidence exists.

“All glass”, “all metal” and “all wood” are non-goals.

## Architecture boundary

- Corpus, recordings, generated WAVs, learned features, weights and reports
  remain external and are referenced by hashes.
- Repository code owns validation, deterministic descriptors, split audits,
  mutation recipes and report schemas.
- Research manifests are current-only experimental formats, not public engine
  contracts.
- Production P1 waits for a concrete consumer, a promoting ADR under ADR-046,
  the complete SPEC-26 contact projection and cooked PresentationOnly content.
- Every missing/unsupported/uncalibrated condition selects the existing clip
  fallback; no per-sound human approval queue exists.

## Work packages

### 1. Domain/formula research registry — implemented

Deliver:

- `xtask physical-sound-registry` with an external-only V1 manifest;
- sorted bounded formula-family and acoustic-domain records;
- explicit material/object/geometry/support/excitation/listener axes;
- hash-closed parameter, corpus, validator-evidence and fallback references;
- only `Candidate`, `Reject` and `FallbackOutOfDomain` while AV-P0C admission
  authority is absent;
- deterministic report plus positive and malformed-input tests.

Exit criterion: two repeated runs are byte-identical; missing hashes,
cross-reference errors, invalid bounds, repository-local inputs and a forged
`Pass` all reject before report publication.

Minimal external authoring shape:

```json
{
  "schema": "nextengine.experimental-physical-sound-research-registry.manifest.v1",
  "registry_id": "physical-sound-p0",
  "formula_families": [{
    "id": "modal-residual",
    "revision": "v1",
    "source_class": "rigid_impact",
    "equation_id": "damped-modal-sum-with-bounded-residual",
    "model_definition": {"path": "model.txt", "sha256": "<64 hex>"},
    "parameter_schema": {"path": "parameters.schema.json", "sha256": "<64 hex>"}
  }],
  "domains": [{
    "id": "thin-steel-vessel-impact",
    "revision": "v1",
    "decision": "candidate",
    "formula_family_id": "modal-residual",
    "material_family": "steel",
    "object_family": "thin-vessel",
    "geometry_family": "axisymmetric-shell",
    "support_condition": "freely-supported",
    "geometry_scale_metres": {"minimum": 0.05, "maximum": 0.5},
    "relative_impact_speed_metres_per_second": {"minimum": 0.1, "maximum": 5.0},
    "impact_position_ids": ["rim", "wall"],
    "listener_condition_ids": ["fixed-near-field"],
    "parameter_set": {"path": "parameters.json", "sha256": "<64 hex>"},
    "corpus_manifest": {"path": "corpus.json", "sha256": "<64 hex>"},
    "fallback_clip": {"path": "fallback.wav", "sha256": "<64 hex>"},
    "fallback_provenance": {"path": "fallback.provenance.txt", "sha256": "<64 hex>"}
  }]
}
```

All paths resolve from the external manifest directory. Run:

```text
cargo run -p xtask -- physical-sound-registry \
  --manifest /external/registry.json \
  --output /external/empty-report-directory
```

### 2. AV-P0C temporal-spectral descriptor and mutation substrate — implemented

Deliver:

- a separate built-in temporal feature profile alongside frozen AV-P0B
  classical features;
- bounded STFT evolution descriptors for spectral flux, adjacent-frame
  distance, centroid/flatness motion, active-bin turnover and early/late
  spectral change;
- deterministic controls that distinguish evolving modal content from a
  stationary tail without using material labels;
- external mutation-family support for stationary white/coloured tails,
  frozen spectral envelope, shuffled decay and removed modal birth/death.

Exit criterion: descriptor repetition is exact, signal bounds hold, positive
controls separate from stationary controls, and the benchmark still emits
`NoAcceptanceAuthority`.

Current checkpoint: the deterministic descriptor, stationary/evolving unit
controls and `physical-sound-mutations` external pack generator are
implemented. The frozen pack copies 19 real entries and derives 36 hash-closed
controlled negatives across stationary-white, stationary-coloured,
frozen-spectrum and shuffled-envelope families. Controlled negatives explicitly
declare `expected_validator_outcome: reject`; existing published transformations
remain `unspecified`. Mutation parent identity and partition are validated.
Repeated generation is deterministic by parent WAV hash and mutation family.

Run the frozen mutation and measurement sequence with separate empty external
directories:

```text
cargo run -p xtask -- physical-sound-mutations \
  --manifest /external/corpus-manifest.json \
  --output /external/controlled-mutation-pack

cargo run -p xtask -- physical-sound-benchmark \
  --manifest /external/controlled-mutation-pack/manifest.json \
  --output /external/controlled-mutation-evaluation
```

### 3. Selective specialist calibration — PS-2 grouped evidence expansion active

Deliver:

- the smallest classifier/ranker that improves grouped holdout error over the
  deterministic nearest-neighbour controls;
- leave-object, leave-family, leave-source, leave-generator-revision and
  leave-mutation-family-out reports;
- calibration-only threshold selection and confidence-bounded false-pass risk
  versus coverage;
- a frozen shadow set that cannot participate in fitting or threshold choice;
- automatic `Pass | Reject | FallbackOutOfDomain` policy.

Exit criterion: the chosen numeric policy is pre-registered from the measured
risk/coverage curve, all mutation ladders pass monotonically, and automatic
`Pass` is enabled only for covered domains. Until then every candidate remains
non-authoritative or fallback.

Current checkpoint: benchmark report v4 retains the frozen temporal profile and
adds a separate amplitude-envelope specialist plus temporal/amplitude consensus.
All profiles standardize from real development entries only, select provisional
thresholds only from calibration, group controlled negatives by parent using the
most permissive child, and report observed plus 95% Wilson upper false-pass risk.

On the unchanged AV-P0C pack the consensus threshold
`0.9305864784564901` produces:

- real coverage `2/3`, `1/3`, `2/3` on calibration/holdout/shadow;
- grouped false passes `0/3`, `0/3`, `0/3`;
- rejection of both frozen shuffled-envelope wood B4/B5 controls and every
  stationary/frozen control;
- byte-identical repeated reports.

PS-1 is therefore complete. Automatic `Pass` remains disabled because only
three mutation parent groups per partition leave the 95% Wilson upper risk at
`0.5615`, real metal/glass coverage remains incomplete and exact domain axes
are absent from the measured corpus. PS-2 must still add independent real object
families; the numeric policy is now pre-registered below. Do not start AV-P0D
while that evidence gate is open. See the [PS-1 evidence](../development/physical-sound-validator-ps1-2026-08-27.md).

PS-2 now adds an external-only pre-registration command without changing the
public/runtime contract:

```text
cargo run -p xtask -- physical-sound-registry corpus-plan \
  --manifest /external/ps2-corpus-plan/manifest.json \
  --output /external/ps2-corpus-plan-report
```

The frozen glass-vessel planning revision binds exact acquisition axes,
controlled-real source and provenance hashes, canonical grouping keys,
calibration-only threshold selection, sealed shadow, mutation monotonicity and
mandatory OOD/unavailable fallback. Its policy requires zero false-pass parents,
a 95% Wilson upper risk at most `0.10`, power `0.95` against unsafe risk `0.20`
and useful-coverage Wilson lower bound at least `0.80`. Deterministic sizing
requires 35 reject parents and 16 in-domain groups; the declared 40/40 plan is
`PlanPowerSufficient`. This is planning authority only. The recordings and
matched real-family evidence remain open; see [PS-2 evidence](../development/physical-sound-corpus-plan-ps2-2026-08-27.md).

The next bounded pilot is also implemented. `physical-sound-registry
corpus-inventory` verifies external float32 transfer bytes, acquisition and
provenance hashes, exact impact/listener positions, finite samples and
cross-partition group leakage. One REALIMPACT GlassGoblet row validates as
`DevelopmentPilotOnly` and automatically becomes `FallbackOutOfDomain` because
the downloadable archive lacks force-profile bytes, composition revision,
repeat identity and support-fixture revision. It opens no calibration, holdout
or shadow and gives no PS-2 release credit. See [pilot evidence](../development/physical-sound-realimpact-pilot-ps2-2026-08-27.md).

The complete acquisition block is now executable too. A raw synchronized entry
requires equal microphone/force dimensions, calibrated-newton force with a
positive impact, unique repeat identity, composition/fixture evidence and
microphone/force calibrations by hash. Complete controls become only
`ResearchEligible`; the old REALIMPACT pilot remains byte-identical fallback.
This is an `E1` import contract for already published evidence, not a local
capture requirement. See [bundle evidence](../development/physical-sound-acquisition-bundle-ps2-2026-08-27.md).

The active [internet corpus policy](../development/physical-sound-internet-corpus-policy-ps2-2026-08-27.md)
retires product-owner/local physical recording and removes force hardware as a
roadmap blocker. The generic official-source registry, bounded external
content-addressed fetch/cache and auditable `E1` synchronized / `E2` transfer /
`E3` identified-real / `E4` synthetic capability matrix are implemented by
`physical-sound-registry internet-sources`.

The measured [internet source/cache pilot](../development/physical-sound-internet-source-pipeline-ps2-2026-08-27.md)
repeats byte-identically across two fresh online caches and one offline run.
Hash-closed ObjectFolder metadata supports only `E4` synthetic lineage. The
first 36,367,088,523-byte ObjectFolder-Real acoustic archive is discovery-only
because the publisher provides no SHA-256; it was not downloaded and grants no
`E1`--`E3` credit.

The first real-source adapter and payload are now implemented. The
[AV-MSF E3 pilot](../development/physical-sound-av-msf-e3-pilot-ps2-2026-08-27.md)
validates one official glass object and two real 44.1 kHz recordings from an
immutable Git commit. Two fresh online caches and an offline run produce the
same report, and the adapter grants only `E3IdentifiedRecording`. This closes
adapter existence, not Package 3: the sample has one object group and no E2
spatial transfer or E1 excitation evidence.

The E3 normalization increment is now implemented by
`physical-sound-registry identified-corpus`. It reruns the source audit against
exact cached bytes, requires all four adapter-backed E3 capabilities, derives
publisher/project/revision, object and recording groups, rejects partition
leakage and measures target coverage against the frozen power report. The
[complete-card AV-MSF pilot](../development/physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md)
validates 10 objects/20 recordings in two fresh caches plus an offline repeat.
All objects share one source group and remain in development; Glass contributes
2 of the required 16 object groups. The report has only
`DevelopmentCoverageMeasured / NO_CORPUS_ADMISSION_AUTHORITY`.

The bounded REALIMPACT E2 increment is now implemented by corpus-inventory V2.
The first typed profile freezes the official repository revision, five exact
source files and one GlassGoblet transfer row with exact metadata/audio/
provenance hashes. It grants only transfer, geometry, impact/listener, object
and real-recording E2 capabilities. Three V2 reports repeat byte-identically;
the historical V1 report is unchanged. Missing raw force, composition, repeat
and fixture axes preserve `FallbackOutOfDomain`; see
[typed E2 evidence](../development/physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md).

The first independent E3 increment is now implemented too. The
`ycb-impact-identified-recording-v1` adapter freezes the official YCB Impact
robot component, exact Wineglass/Skillet lid metadata and eight repeated
48 kHz recordings. Its source-specific OSF redirect policy binds the approved
bucket path to the expected artifact hash without enabling generic redirects.
The [combined pilot](../development/physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md)
repeats across two fresh online caches and offline audits. AV-MSF plus YCB now
measure two publisher/project/revision groups, 12 objects and 28 recordings;
Glass contributes 4 groups/12 recordings against the required 16 groups. All
entries remain in development and no upstream split is promoted to holdout.

The measured re-plan and next independent E3 increment are now implemented.
`heller-impact-identified-recording-v1` freezes the versioned CMU KiltHub
Impact Events archive/notes and grants only one explicitly named Glass-vase
object/event group with five repeats. Its Figshare redirect and ZIP extraction
are source-specific and bounded; mirror/red-vase rows and multiple impactors on
one target are excluded. The [combined pilot](../development/physical-sound-heller-independent-e3-pilot-ps2-2026-08-27.md)
repeats across two fresh caches and offline audits. AV-MSF + YCB + Heller now
measure three publisher/project/revision groups, 13 objects and 33 recordings;
Glass contributes 5 groups/17 recordings, leaving 11 of 16 groups open.

The Greatest Hits discriminator is complete. Byte-range ZIP inspection recovers
the complete label inventory without the 20 GB archive, but the labels bind
material/action/reaction to scene videos rather than stable target objects.
Because video-as-object would invent E3 identity, the path is rejected and no
download adapter is added. See the
[bounded rejection](../development/physical-sound-greatest-hits-discriminator-ps2-2026-08-27.md).

The second REALIMPACT object path is now complete. `physical-sound-registry
realimpact-row` retrieves EOCD, central directory, six small arrays, the mesh
and a fixed transfer prefix for GreenGoblet through exact HTTPS ranges. The
1.61 MB bounded path reproduces the selected E2 row without downloading the
2.31 GB archive and preserves `FallbackOutOfDomain` for missing raw force,
composition, repeat and fixture evidence.

The next source discriminator is complete too. ObjectFolder-Real has strong
object/force/coordinate metadata, but its 34–39 GB single-stream gzip batches
do not expose a bounded path to repeated audio after large embedded media; the
archive-prefix route is rejected pending an official per-object or seekable
surface. The `freesound-glass-bowl-identified-recording-v1` adapter instead
freezes one explicitly named Glass bowl, eight repeated wood-strike HQ MP3
previews and one canonical pack-page identity projection. Two independent
online caches and offline audits repeat byte-identically. Combined E3 coverage
is now four publisher/project/revision groups, 14 objects and 41 recordings;
Glass contributes 6 groups/25 recordings, leaving 10 of 16 groups open. See
the [bounded evidence](../development/physical-sound-freesound-glass-bowl-e3-pilot-ps2-2026-08-28.md).

The next cached E3 increment is implemented by
`freesound-wine-glass-identified-recording-v1`. It freezes a second Freesound
publisher/project, one pack-specific wine-glass family and three numbered
knife-strike HQ previews. Two imported cache roots and three offline audits are
byte-identical; the current-host bounded direct route receives HTTP 403, so no
independent-online-fetch claim is made and the fetch security boundary remains
unchanged. Combined E3 coverage is now five project/revision groups, 15
objects and 44 recordings; Glass contributes 7 groups/28 recordings, leaving 9
of 16 groups open. See the
[cached evidence](../development/physical-sound-freesound-wine-glass-e3-pilot-ps2-2026-08-28.md).

Explicit reject-parent import semantics are now implemented. Every source in
explicit mode is labelled `target` or `reject_parent`, and the role must match
exact adapter material evidence before report publication. The current corpus
contains seven Glass target groups and eight development-only non-Glass parent
groups/sixteen recordings; twenty-seven of the required thirty-five parents
remain open. Partial roles and material-role mismatches reject, while the prior
implicit report retains its byte-identical hash. This is parent inventory, not
negative-control generation, validator success or measured false-pass risk. See
the [explicit-role evidence](../development/physical-sound-explicit-reject-parent-import-ps2-2026-08-28.md).

The latest Package 3 increment closes both aggregate E3 count requirements.
`soundpacks-glass-recordings-identified-recording-v1` validates one exact RAR5
archive, its readme, four numbered drinking-glass WAVs and three numbered
glass-vase WAVs through a stable SoundPacks page projection, a source-specific
MediaFire resolver and bounded pure-Rust extraction. Two fresh online caches,
an offline replay and two complete combined audits repeat byte-identically.
The [combined evidence](../development/physical-sound-soundpacks-glass-e3-and-split-audit-ps2-2026-08-28.md)
measures eight project revisions, 54 objects and 125 recordings; Glass reaches
`16/16` objects and 46 recordings, while reject parents remain `38/35` objects
and 79 recordings.

Package 3 then moved from aggregate counts to measured split structure.
`physical-sound-registry split-feasibility` initially rejected the corpus
because only three of eight projects carried reject parents. The next increment
expanded the already identified Kronland project with its five Wood originals
and five Metal originals through a stable publication-page projection. Two
online caches, one offline replay and the legacy five-Glass report repeat
byte-identically. The combined evidence now measures 64 objects/135 recordings;
Glass remains `16/16` and 46 recordings, while reject parents reach `48/35` and
89. Four dual-role projects make the `20/30/25/25` project split feasible.

`physical-sound-registry split-freeze` binds the pre-split corpus, plan and
feasibility hashes, applies the frozen seed with bounded deterministic
backtracking and verifies every post-split entry. Two complete partitioned
audits and two verification reports repeat byte-identically. Each of
`dev/calibration/holdout/shadow` has two whole projects plus target and reject
evidence. The [split evidence](../development/physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md)
closes partition structure only. Package 3 now requires an exact-domain E2/E3
claim matrix; unavailable geometry, support, excitation, position,
listener/radiation or real-identity axes remain unavailable and keep that
domain fallback-only rather than triggering local capture or premature PS-3.

The exact-domain matrix is now implemented by `physical-sound-registry
domain-claims`. Its external manifest hash-links the frozen plan, verified
partitioned E3 report, split verification, all five REALIMPACT E2 reports and
one reviewed Blue Bowl cross-tier identity. Two runs are byte-identical at
report SHA-256
`e6d078bd792ab45f09045bd272df3b39fafa0cb02c3c4e0fddb064aab95f5b60`.
The decision is `DomainEvidenceIncomplete / FallbackOutOfDomain`: four useful
identity/transfer claims pass, but eight exact requirements remain unsupported
and no partition has an eligible object. Numeric object IDs are not a generic
join; a failure control rejects `94_GlassGoblet` versus ObjectFolder object 94
`Salad_Bowl`. See the [matrix evidence](../development/physical-sound-domain-claims-matrix-ps2-2026-08-28.md).

The next Package 3 increment is therefore evidence feasibility, not formula
tuning: search published sources for same-object composition/geometry/support/
excitation/listener/repeat lineage that can link to an existing frozen E3
object. If the acquisition-shaped V1 domain has no published path, preserve it
as fallback-only and preregister an internet-native plan revision rather than
inventing axis values or requesting local capture.

### 4. AV-P0D autonomous formula search

Deliver:

- a versioned parameter-search manifest and deterministic candidate lineage;
- a Pareto objective over reference fidelity, causal coherence and measured
  runtime cost;
- one source-model-family change per experiment;
- validator and shadow isolation from optimizer inputs;
- automatic registry publication for accepted exact-domain revisions and
  durable negative controls for rejected revisions.

First discriminator after AV-P0C: compare a bounded time-varying coloured
residual against bounded modal interaction for the thin-metal vessel domain.
Do not retry stationary-white gain/T20 grids.

Exit criterion: one complete run starts from frozen corpus/formula/validator
hashes and ends in a reproducible registry decision without per-candidate human
input.

### 5. Material-domain expansion

Apply the same frozen loop independently to:

1. thin metal vessel/shell impact;
2. thin glass vessel impact;
3. dry hardwood block impact.

Each family receives its own formula revision and domain envelope. A successful
domain is not generalized by name. New geometry/support/force coverage creates
a new admission record and may reuse prior evidence only where the registry
proves the axes identical.

Exit criterion: at least one exact domain per family has measured risk,
coverage, untouched-shadow evidence, bounded cost and an authored fallback.

### 6. P1 production promotion

Only after packages 1–5 and a selected player-visible consumer:

- write the promoting ADR;
- freeze `AcousticMaterialProfileV1`, `ModalSoundModelV1` and
  `PhysicalSoundBindingV1` as the smallest consumer-driven content shapes;
- close the SPEC-26 contact projection for velocity, impulse/effective mass
  and material tags;
- cook admitted research records into bounded engine content;
- implement `AUDIO-PHYS-SOURCE-P1`, `AUDIO-PHYS-CONTENT-P1` and
  `AUDIO-PHYS-PCM-P1`, then the applicable focused product checks;
- retain exact clip fallback and prove all authoritative roots unchanged.

Rolling and scraping remain P2 and cannot be inferred from impact success.

## Validation strategy

| Boundary | Minimum evidence |
|---|---|
| Registry/tooling changes | focused `xtask` tests, format, Clippy, external-path/hash negative cases |
| Descriptor changes | deterministic unit controls plus grouped benchmark report |
| Validator release | mutation, grouped calibration/holdout/shadow and risk–coverage report |
| Formula search | exact lineage/repeat and frozen-validator non-leakage |
| Production content/runtime | focused fast/play/content-package/persistence-replay; conditional platform/performance |

No check result may upgrade a broader domain than its manifest declares. A Git
commit records a coherent checkpoint; it is not itself validation evidence.

## Rollback and non-regression

- AV-P0A and AV-P0B report meanings remain historical exact-commit evidence.
- Steel v3 and rejected stationary-residual v4 remain frozen negative controls.
- Glass-H, selected Q30 and wood-B remain isolated experimental controls, not
  generalized material truth.
- Removing or disabling all physical-sound experiments preserves the current
  clip mixer, gameplay facts, physics roots and persistence/replay roots.
