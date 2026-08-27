# Physical sound domain admission implementation plan

## Outcome

Build an external, hash-closed research loop that can fit mathematical source
models, automatically admit only bounded acoustic domains, and select authored
clip fallback for uncertainty. The plan implements SPEC-45 P0 evidence first;
it does not activate a roadmap stage, add a shipping content schema or replace
the current `AudioMixerV1` clip path.

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

### 3. Selective specialist calibration — in progress

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

Current checkpoint: benchmark report v3 standardizes temporal features from
real development entries only, selects a provisional threshold only from
calibration, groups controlled negatives by parent using the most permissive
child, and reports observed plus 95% Wilson upper false-pass risk on calibration,
holdout and shadow. The first measurement is deliberately non-promoting:

- provisional threshold `0.40490598982279524`;
- real coverage `1/3`, `1/3`, `0/3` on calibration/holdout/shadow;
- grouped false passes `0/3`, `1/3`, `1/3`;
- holdout and shadow Wilson upper risk `0.7923`;
- both false passes are shuffled-envelope wood controls, while all exact
  stationary/frozen controls reject at this threshold.

The next smallest validator-only change keeps the corpus, splits and mutation
pack frozen and adds amplitude-envelope trajectory features: frame log-RMS
slope/curvature, monotonicity violations, early/mid/late energy ratios and
energy/spectral-change coupling. After it closes the two counterexamples, add
independent real object families so the grouped confidence bound is informative.
Do not enable registry `Pass` or start AV-P0D while this exit criterion is open.

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
