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

Package 3 must next obtain a hash-closed object/material inventory and bounded
object-level retrieval path for Greatest Hits before accepting its 20 GB
minimum official package, or reject that path and select another source. It
must also expand complementary E2/E1 arrays before
calibration/holdout/shadow open.
Separate internet sources may support separate specialist claims, but
unavailable axes remain unavailable and the grouped split/risk policy is
unchanged. Insufficient published coverage keeps the domain fallback-only; it
does not trigger local capture.

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
