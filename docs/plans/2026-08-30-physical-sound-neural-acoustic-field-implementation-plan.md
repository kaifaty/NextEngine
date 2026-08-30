# Physical sound neural acoustic field — implementation plan

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Status | `IN_PROGRESS / N0.1_COMPLETE / N0.2_COMPLETE / N0.3_TIME_DOMAIN_FAMILY_REJECTED / N0.3B_DENSE_COMPLEX_FIELD_DATA_READY / N0.3C_COMPLEX_TRAINING_NEXT / RESEARCH_ONLY` |
| Strategy | [Neural acoustic field strategy](../development/physical-sound-neural-acoustic-field-strategy-2026-08-30.md) |
| Roadmap | [Physical sound synthesis roadmap](physical-sound-synthesis-roadmap.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |

## Objective

Demonstrate whether an offline neural model can learn an impact/listener
acoustic field better than the frozen Q30 modal plus DCT baseline, while the
game-facing result remains a bounded, deterministic cooked representation.

The plan does not authorize a public schema, runtime neural inference,
checkpoint distribution, production contact wiring or a shipping claim.

## Fixed constraints

- Training and inference are external research operations.
- Datasets, raw audio, generated WAVs, model weights, optimizer state and large
  feature caches stay outside the repository.
- Repository code may contain manifests, adapters, compact synthetic fixtures,
  reports and deterministic cook/validation logic.
- Published source fields are never completed by guessing.
- Authored clips remain the mandatory fallback.
- Generator and validator use separate calibration/holdout/shadow evidence.
- The current frozen Q30/DCT path remains an immutable waveform-domain
  baseline.
- `force_deconvolved_transfer_response` and `recorded_impact_waveform` are
  different tasks. They cannot share a sample loss or baseline without an
  explicit, hash-bound excitation model.

## Work packages

### N0.1 — Freeze the data projection and split manifest

Status: `COMPLETE / V2_SIGNAL_SEMANTICS / REAL_PROJECTION_BYTE_IDENTICAL /
SHADOW_CONTENTS_NOT_MATERIALIZED`.
See [PS-2N0 evidence](../development/physical-sound-neural-data-plane-ps2n0-2026-08-30.md).
Real evidence is recorded in the [R0–R1 report](../development/physical-sound-neural-real-boundary-r0-r1-2026-08-30.md).

Deliverables:

- a versioned external row projection for object, geometry, support, impact,
  listener, recording and provenance claims;
- explicit `recorded_impact_waveform` or
  `force_deconvolved_transfer_response` semantics for every row;
- independent optional impact-point and outward-normal claims, so one missing
  axis cannot erase another published axis;
- deterministic projection from the existing source/corpus registry;
- `train/development/calibration/method_holdout/admission_shadow` group roles;
- duplicate/derived-parent leakage audit;
- capability report that marks every missing axis explicitly;
- tiny checked-in synthetic fixtures for positive and failure paths.

Exit criteria:

- two offline runs produce byte-identical manifests and reports;
- object/source/mutation parents cannot cross partitions;
- no shadow row contents are exposed to later development commands;
- missing geometry, support, force or listener data cannot be promoted by an
  object/material label;
- a transfer response cannot enter a waveform-only baseline or loss;
- no model is trained in this package.

Fallback: if the data cannot support impact/listener conditioning, narrow N0.2
to the strongest exact object/axis instead of manufacturing a broad domain.

Commit boundary: projection contract, fixtures, focused tests and evidence.

### N0.2 — Export the deterministic classical benchmark

Status: `COMPLETE / Q30_WAVEFORM_EXPORT_PASS / SYNTHETIC_EXACT_AB_PASS /
TRANSFER_CONTROLS_FROZEN / REAL_DEVELOPMENT_REPEAT_PASS`. See
[N0.2 baseline evidence](../development/physical-sound-classical-baseline-export-ps2n0-2026-08-30.md).

Deliverables:

- the existing Q30/DCT exporter for compatible waveform/synthetic rows;
- a separate transfer-domain nearest-neighbour/interpolation baseline for
  force-deconvolved rows;
- canonical mode ordering, residual descriptor and coverage metadata;
- per-row acoustic features needed by both neural candidates and validator
  specialists;
- exact A/B report and PCM hashes on synthetic fixtures and permitted
  development rows.

Exit criteria:

- existing frozen hashes and negative controls do not change;
- the exporter is deterministic and fails closed on stale lineage;
- baseline outputs are available for every supported benchmark task;
- transfer and waveform rows are never compared under incompatible semantics;
- unsupported rows are `FallbackOutOfDomain`, not partial success.

Fallback: retain Q30 as the waveform/synthetic baseline and mark unsupported
transfer rows explicitly until a transfer-domain baseline exists. Do not
reinterpret force-deconvolved responses as recorded impacts.

Commit boundary: baseline exporter plus non-regression evidence.

### N0.3 — Fixed-impact listener-field pilot

Status: `V1_AND_PHASE_ALIGNED_REJECTED /
TRAINING_EVALUATION_AND_DIAGNOSTIC_REPRODUCIBLE /
DENSE_COMPLEX_FIELD_REBASELINE_NEXT`. See the
[R2 V1 result](../development/physical-sound-listener-field-r2-v1-result-2026-08-30.md)
and [phase-aligned result and failure research](../development/physical-sound-listener-field-r2-phase-research-2026-08-30.md).

Deliverables:

- one exact object/impact block with multiple published listener positions;
- constant/rank-one and frozen classical interpolation controls;
- a compact listener-conditioned modal gain field;
- modal-only and modal-plus-residual ablations;
- fixed seeds, environment lock and deterministic cooked replay.

Exit criteria:

- held-out listeners beat both classical controls on every preregistered
  primary aggregate;
- relative amplitude, decay, finiteness and exact cook do not regress;
- the ablation identifies the component responsible for improvement;
- failure publishes `REJECT_LISTENER_FIELD` or `DATA_INSUFFICIENT` without a
  free hyperparameter search.

Fallback: stop the current representation hypothesis or narrow it to the
supported listener axis. Do not claim unseen impact-position support.

Commit boundary: preregistration, runner, compact fixtures and immutable
listener-field report. Weights and datasets remain external.

The direct and propagation-delay-aligned time-domain latent family is now
closed. The phase successor improves four of five rank-4 endpoints but misses
the unchanged P95 spectrum criterion. A query-informed subspace oracle also
fails held-query level and spectral aggregates, so width/rank/epoch/seed/speed
and threshold tuning are forbidden on this opened slice.

### N0.3B — Dense complex-field data sufficiency preflight

Status: `COMPLETE / READY_FOR_COMPLEX_FIELD_TRAINING /
NO_QUALITY_ADMISSION_OR_RUNTIME_AUTHORITY`. See the
[R2B dense complex-field preflight](../development/physical-sound-r2b-dense-complex-field-preflight-2026-08-30.md).

Purpose: test whether published spatial coverage and a phase-preserving field
representation can support an honest R2 experiment before training a larger
model.

Deliverables:

- one hash-closed Green Goblet fixed-impact projection covering the full
  published 600-position REALIMPACT semicylinder;
- a preregistered grouped split by complete gantry columns or equivalent
  spatial blocks, with no individual-neighbour leakage;
- coordinate coverage, spacing, spatial-frequency and missing-axis report;
- one frozen complex STFT or log-magnitude plus continuous-phase target with
  exact forward/inverse-cook parameters and error;
- nearest/linear plus simple complex-field controls on the grouped split;
- a preregistered exterior-air Helmholtz/physics constraint and no-physics
  ablation for the later candidate;
- zero method-holdout and admission-shadow payload reads.

Exit criteria:

- two projection/preflight runs are byte-identical;
- every row binds published coordinates, signal semantics, source hashes and
  spatial group before splitting;
- query groups are absent from representation fitting and learned feature
  normalization;
- inverse cooking is finite, bounded and within a preregistered numeric error;
- all classical controls and primary aggregates are available before the first
  optimizer step;
- the report returns `READY_FOR_COMPLEX_FIELD_TRAINING`,
  `DATA_INSUFFICIENT` or `REJECT_COMPLEX_FIELD_REPRESENTATION`.

Fallback: if the full published block or grouped split cannot support the
task, stop R2 for this object and search another internet source. Do not return
to the 15-row latent or use local capture.

Commit boundary: data projection, grouped split, representation preflight and
immutable sufficiency report. Dataset payloads remain external.

The V3 projection and full preflight have now repeated byte-identically. The
frozen split is `420 context / 180 query` by complete angle planes; query rows
contribute neither normalization nor fit features. The complex STFT inverse
passes at `-153.348 dB` worst NRMSE, `9.031e-9` maximum absolute error and at
most one PCM16 LSB. Three controls are evaluated on all 180 queries before any
optimizer step. The exact decision is `ReadyForComplexFieldTraining`, not a
model or quality pass.

### N0.3C — Frozen dense complex-field physics ablation

Status: `NEXT / TWO_CANDIDATES_ONLY / QUERY_EVALUATION_AFTER_FREEZE`.

Purpose: determine whether the ready complex representation supports a learned
listener field and whether a bounded exterior-air Helmholtz residual adds
value under an otherwise identical protocol.

Deliverables:

- one hash-closed training manifest freezing environment, model shape,
  initialization, seed, optimizer budget and the R2B data/representation
  lineage before optimizer step one;
- `dense_complex_field_data_only_v1` and
  `dense_complex_field_helmholtz_v1`, sharing one
  coordinate/time/frequency MLP that emits real and imaginary pressure;
- context-only complex-STFT L1 plus log-magnitude L1 data loss;
- Helmholtz weights `0` and `0.0001`, speed `343 m/s`, frequency band
  `93.75–12,000 Hz` and deterministic midpoint collocation;
- two independent training runs per candidate with tolerance-bound checkpoint
  and context-loss reproducibility;
- frozen checkpoint selection without query feedback, followed by one Rust
  metric evaluation on the 180 query rows.

Exit criteria:

- no query audio contributes to normalization, fit, early stopping or
  checkpoint selection;
- both candidates remain finite and cook through the frozen inverse/PCM path;
- repeated training satisfies the preregistered tolerance and cooked
  prediction hashes repeat exactly from each frozen checkpoint;
- `GO_COMPLEX_LISTENER_FIELD` requires one candidate to be strictly lower than
  every frozen control on all five primary aggregates;
- otherwise return `REJECT_COMPLEX_FIELD` with the failed endpoints and no
  nearby width/weight/seed/step grid.

Fallback: preserve both candidate results and keep authored clips. A failure
may motivate a new representation only after bounded failure research; it does
not reopen the 15-row time-domain latent family.

Commit boundary: training manifest/runner, compact synthetic failure tests,
external two-run lineage and immutable evaluation decision. Dataset,
features, MLflow state, checkpoints and generated WAVs remain external.

### N0.4 — Object-specific impact/listener few-shot field

Entry condition: N0.3 proves the representation and published data provides
multiple impact and listener conditions for one exact object.

Deliverables:

- external, hash-closed training manifest for one object family with multiple
  impact and listener observations;
- a model with global frequency/damping parameters, conditioned gain field and
  compact residual output;
- training/evaluation runner with fixed seeds and environment lock;
- ablations for modal-only, modal-plus-gain and modal-plus-gain-plus-residual;
- cooked deterministic replay of every prediction.

Exit criteria:

- held-out impact/listener results beat the classical baseline on the
  preregistered primary aggregates;
- all hard, causal, energy-scaling and exact-cook controls pass;
- results repeat within the declared training tolerance and cooked PCM repeats
  exactly;
- failure/OOD conditions choose fallback.

Fallback: a failure closes the current representation hypothesis. Do not tune
another residual family unless the ablation identifies one missing statistic.

Commit boundary: model interface/runner, compact fixtures and a report. Weights
and datasets remain external.

### N0.5 — Shared geometry-conditioned surrogate

Entry condition: N0.4 proves the representation on held-out positions and
listeners.

Deliverables:

- geometry encoder and shared modal/radiation heads trained only on the frozen
  train role;
- object- and family-disjoint method-holdout evaluation;
- comparison with object-specific few-shot adaptation;
- calibrated coverage/OOD output;
- synthetic-teacher ablation separating physics pretraining from real-data
  fitting.

Exit criteria:

- a frozen result either supports a bounded shared domain or explicitly
  rejects zero-shot transfer;
- no source-project or near-duplicate leakage exists;
- a shared candidate cannot hide object failures behind a pooled mean;
- the cooked representation remains bounded and exact.

Fallback: retain object-specific few-shot fields. A failed universal model is
not a failure of exact-object neural cooking.

Commit boundary: shared-model experiment and immutable decision report.

### N0.6 — Freeze the neural feasibility benchmark

Deliverables:

- one manifest binding data, roles, preprocessing, baseline, candidates,
  metrics, environment, seeds and output hashes;
- per-object/position/listener distributions and failure clustering;
- report-only direct-waveform upper bound when reproducible;
- explicit `GO_LISTENER_FIELD`, `GO_EXACT_OBJECT`, `GO_SHARED`,
  `REJECT_REPRESENTATION` or `DATA_INSUFFICIENT` decision.

Exit criteria:

- development choices are complete before method-holdout access;
- the report repeats from immutable external inputs;
- no admission-shadow data is read;
- the selected candidate and its claim envelope are frozen for PS-3/PS-4.

Commit boundary: benchmark manifest, report and roadmap/task-state update.

### N1 — Independent Validator Release V1 (`PS-3`)

Deliverables:

- deterministic hard/causal gates;
- independently trained/frozen specialist and representation heads;
- grouped calibration/holdout risk and coverage report;
- immutable thresholds, OOD and unavailable-component policy;
- reward-hack, mutation and generator-family controls.

Exit criteria:

- one release obtains useful coverage at the preregistered false-pass bound, or
  honestly freezes as fallback-only;
- validator training evidence excludes generator method-holdout and admission
  shadow;
- repeated reports are byte-identical where deterministic and tolerance-bound
  where a declared external model is involved;
- no live human approval is required.

Fallback: `FallbackOutOfDomain`-only. PS-4 remains closed.

Commit boundary: validator release and risk report.

### N2 — Neural cooker and one-shot admission (`PS-4`)

Entry condition: one N0.6 candidate and Validator Release V1 are both frozen.

Deliverables:

- strict external neural-output validator and deterministic cooker;
- canonical quantization into the reference modal/residual renderer;
- exact lineage from model/data/checkpoint to cooked record;
- one declared admission-shadow evaluation;
- immutable `Pass`, `Reject` or `FallbackOutOfDomain` record.

Exit criteria:

- candidate does not inspect validator thresholds or shadow during training;
- hard/causal failure cannot be offset by an acoustic score;
- accepted cooked PCM repeats exactly;
- invalid model output, OOD or unavailable inference chooses the authored clip;
- shadow is not reopened to tune the same revision.

Fallback: preserve the rejection and counterexample, then change one
preregistered hypothesis or stop the domain.

Commit boundary: cooker, one-shot evidence and admission decision.

### N3 — Neural-Cooked Formula Base V1 (`PS-5`)

Entry condition: N2 admits at least one exact domain.

Deliverables:

- three independent bounded targets: thin metal vessel/shell, thin glass
  vessel and dry hardwood block;
- exact evidence, model/cooker/validator lineage and fallback for every record;
- negative/rejected records retained as non-repeatable knowledge;
- coverage and cost summary suitable for a future consumer selection.

Exit criteria:

- each claimed target has its own untouched-shadow admission;
- no result broadens from an object/geometry/support envelope to a generic
  material class;
- every record can be reconstructed into the deterministic reference profile
  without model inference.

Fallback: fewer admitted targets remain useful research artifacts but do not
satisfy Formula Base V1.

Commit boundary: registry revisions and research admission evidence.

### N4 — Production rigid-impact vertical (`PS-6`)

Entry condition: the main roadmap selects a player-visible consumer and a
separate ADR-046 decision promotes the minimum contracts.

Deliverables and checks remain those in the subsystem roadmap: complete
committed contact projection, consumer-driven content roles, existing mixer
integration, fallback, whole-mixer budget and affected ProductChecks.

The first production vertical consumes only a cooked record. Runtime neural
inference is not part of N4.

## Evaluation matrix

| Axis | Development task | Method-holdout task | Admission consequence |
|---|---|---|---|
| Impact position | interpolate known-object positions | unseen positions and object groups | narrow or reject spatial claim |
| Listener position | reconstruct held-out microphones | unseen listeners/object groups | narrow or reject radiation claim |
| Excitation | bounded energy scaling | unseen permitted energy bands | hard reject on causal failure |
| Object geometry | known/few-shot object | object- and family-disjoint | choose shared or few-shot claim |
| Residual | coloration/autocorrelation ablations | unopened real groups | select compact residual or reject |
| Runtime | cook and canonical PCM | repeat on frozen predictions | reject non-bounded/non-exact output |
| OOD | synthetic and source-held-out negatives | foreign objects/conditions | fallback outside calibrated coverage |

## Verification policy

Each package runs the minimum focused checks for the files it changes. External
experiments record exact commands, environment revisions, source hashes and
repeat policy. Documentation-only commits use `git diff --check` plus direct
link/path/identifier validation. No package receives product credit from a
successful Git commit or a report-only model result.

## Immediate queue

1. Preserve both direct and phase-aligned time-domain fields as immutable
   `REJECT_LISTENER_FIELD` results; do not tune their ranks, width, epochs,
   speed or thresholds on opened development rows.
2. Treat the byte-identical V3 acquisition/preflight decision
   `ReadyForComplexFieldTraining` as data/representation authority only.
3. Freeze one N0.3C manifest with the shared complex-pressure MLP, seed,
   optimizer budget, context-only preprocessing and exact R2B lineage.
4. Train only the data-only and `0.0001` Helmholtz ablations twice each; track
   external lineage without opening query, method holdout or admission shadow.
5. Freeze checkpoints without query feedback, then evaluate each once on the
   180 grouped queries against all three controls and the unchanged
   five-endpoint conjunctive rule.
6. Proceed to impact/listener few-shot N0.4 only if R2C passes and an internet
   source closes the required impact axis.
