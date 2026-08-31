# Physical sound neural acoustic field — implementation plan

| Field | Value |
|---|---|
| Date | 2026-08-31 |
| Status | `IN_PROGRESS / N0.4A_V8_OBJECT91_FIT_PROTOCOL_FROZEN / FIT_RUNNER_NEXT / N0.4B_NOT_AUTHORIZED / RESEARCH_ONLY` |
| Strategy | [Neural acoustic field strategy](../development/physical-sound-neural-acoustic-field-strategy-2026-08-30.md) |
| Roadmap | [Physical sound synthesis roadmap](physical-sound-synthesis-roadmap.md) |
| Architecture | [SPEC-45](../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), `Proposed` |

## Objective

Demonstrate whether an offline explicit-modal neural model can learn an
impact/contact-position mode-shape or gain field better than compatible frozen
classical baselines while preserving frequency and damping explicitly. The
first game-facing experiment is a bounded, offline-baked clip atlas and
listener spatialization remains a separate layer.

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
- Neural encoder, quantizer, decoder and contact field remain external. Runtime
  receives only ordinary baked clips plus bounded coverage/fallback metadata.
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

Status: `COMPLETE / REJECTED / REPRODUCIBLE / NO_CANDIDATE_SELECTED`. See the
[R2C result and failure diagnostic](../development/physical-sound-listener-field-r2c-result-2026-08-30.md).

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

Outcome: both `341,410`-parameter candidates complete `8,000` deterministic
GPU steps and repeat byte-identically, but each fails four of five endpoints.
Mean level error is about `53 dB`; no checkpoint is selected. Context-only
failure research shows that rank 96 retains `99.64%` of energy, while the
trained data-only objective is `1.0498x` the zero predictor and all logged
steps reach gradient clipping. Helmholtz value and rank capacity are rejected
as primary causes. Do not retry this separable SIREN/objective revision with a
nearby architecture, rank, seed, step or physics-weight grid.

### N0.3D — Context trainability and energy-preservation gate

Status: `V1_FIXED_STEP_REJECTED / V2_PASS / COMPLETE`. See
[R2D V1 evidence](../development/physical-sound-listener-field-r2d-trainability-v1-result-2026-08-30.md)
and [R2D V2 evidence](../development/physical-sound-listener-field-r2d-trainability-v2-result-2026-08-30.md).

Purpose: prove that the objective, sampling, optimizer and real cooker can
retain the signal before another held-listener candidate is authorized.

Deliverables:

- a hash-closed context-only profile binding objective terms, sampling,
  normalization, optimizer, clipping diagnostics and inverse/PCM cooker;
- zero and global-mean predictors plus exact context-only low-rank oracles;
- identity, one-row and small spatial-block micro-overfit controls;
- absolute level, multi-resolution spectrum, complex reconstruction, waveform,
  active-bin and gradient/clipping reports;
- two repeated full-context fits with zero query, method-holdout and shadow
  audio reads.

Exit criteria:

- identity and micro-overfit controls pass their preregistered cooker metrics;
- full-context fit strictly improves zero and global-mean controls and stays
  within the frozen low-rank oracle envelope;
- output energy does not collapse, values remain finite, and clipping
  saturation satisfies the frozen policy;
- repeated artifacts meet their declared exact/tolerance policy;
- failure returns `REJECT_TRAINING_SUBSTRATE` and does not open query audio.

Fallback: preserve R2B representation/data readiness but reject the current
training substrate. Change one preregistered objective/optimization hypothesis
per revision; do not spend query evidence on trainability debugging.

Commit boundary: context controls, micro-overfit runner/tests and immutable
trainability decision. No query WAVs or candidate quality report.

V1 freezes a `99.6396%`-energy rank-96 basis and repeats byte-identically. All
coefficient, cooker, oracle-proximity, clipping and zero/global-mean gates pass.
The one-row task passes completely; the eight-row and full-context tasks miss
only mean absolute log energy at `0.007785` and `0.005062` against the unchanged
`0.005` limit. V1 therefore returns `RejectTrainingSubstrate` and does not
authorize N0.3E. V2 may change only fixed learning rate to one deterministic
decay schedule ending near zero; basis, objective, initialization, steps,
tasks, cooker, metrics and thresholds remain frozen.

V2 applies that sole change as an inclusive-endpoint half-cosine schedule from
`0.05` to `0.00001`. Two runs produce the same `898ee201…0875` normalized
report, all checkpoints and all prediction WAVs match byte-for-byte, and every
unchanged gate passes with zero query/method-holdout/shadow reads. N0.3D is
complete and authorizes exactly one separately frozen N0.3E candidate; it does
not authorize a quality or runtime claim.

### N0.3E — Frozen low-rank spatial coefficient field

Entry condition: `MET`; N0.3D V2 passes reproducibly.

Purpose: separate high-dimensional time/frequency reconstruction from spatial
generalization. Compute one complex basis from context rows only and train a
coordinate network to predict its complex coefficients.

Deliverables:

- a context-only frozen basis and rank selected before query access;
- non-neural coefficient interpolation controls;
- one data-only coordinate-to-coefficient network using the passed N0.3D
  objective/cooker;
- two independent training repeats and one frozen 180-query evaluation;
- failure clustering by angle plane, distance, microphone height and spectrum.

Exit criteria:

- N0.3D gates remain green for the complete candidate path;
- query audio is used only by the frozen final evaluator;
- the candidate strictly beats all three R2B controls on every unchanged
  primary aggregate;
- prediction/cook/report reproducibility satisfies the frozen policy;
- failure returns `REJECT_LOW_RANK_COEFFICIENT_FIELD` or
  `DATA_INSUFFICIENT` without a nearby tuning grid.

Fallback: stop the fixed-impact listener-field representation and keep authored
clips. A physics regularizer may be tested only after a data-only model passes
trainability and demonstrates a held-listener advantage.

Commit boundary: basis/coefficient runner, compact tests, external two-run
lineage and immutable grouped-query decision.

Outcome: complete and rejected. The harmonic field fits context essentially
exactly and repeats byte-for-byte, yet loses all five query endpoints to every
frozen control. The post-reject rank-96 query oracle repeats with NRMSE
`0.2760`, retains `92.38%` query energy and still misses mean-spectrum versus
linear interpolation. Decision:
`RepresentationAndInterpolationBothLimited`. The R2 listener split is retired;
nearby architecture/rank/seed/step tuning is forbidden. See the
[R2E result and V4 research](../development/physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md).

### N0.4A — New internet corpus and representation preflight

Entry condition: `MET`; N0.3E closed the fixed-impact listener task without
claiming that contact-position modeling is infeasible.

Purpose: freeze a new, unopened product-aligned task before another neural
model. The first task predicts sound over contact position for an exact object
at one declared canonical listener condition. Detailed radiation is not a
learned axis.

Deliverables:

- availability/provenance audit for multi-object and multi-impact published
  real data, starting with REALIMPACT and ObjectFolder Real;
- force normalization, peak alignment, three-second or evidence-backed decay
  window and explicit waveform/transfer semantics;
- exact geometry/image/contact coordinate bindings and honest missing
  support/composition metadata;
- new object/source/project grouped train, development, calibration,
  method-holdout and admission-shadow commitments;
- compatible KNN/nearest, modal, Q30/DCT and reproducible DiffSound/FEM
  controls;
- modal plus residual and at least one alternative compact representation
  oracle on query-seeing development contacts;
- sampling-capability report that narrows listener radiation to canonical or
  exact published conditions.

Exit criteria:

- two projections and reports reproduce under the declared policy;
- at least one exact object has enough disjoint contact locations for few-shot
  fit and held-contact evaluation;
- one compact representation beats its preregistered target baseline while
  preserving level, modes, damping, envelope and spectrum;
- method holdout and admission shadow remain unopened;
- result is `READY_FOR_EXACT_OBJECT_FIELD`, `DATA_INSUFFICIENT` or
  `REJECT_REPRESENTATION`.

Fallback: search another published source or retain canonical authored clips.
Do not reuse the opened Green Goblet listener query as method evidence and do
not ask the user for local recordings.

Commit boundary: source/corpus manifest, representation-oracle runner, compact
tests and immutable preflight report. Payloads and feature caches stay external.

V1 result: `REJECT_REPRESENTATION`. The metadata-only Blue Bowl manifest,
four-contact extraction and query-seeing oracle each repeat byte-identically;
the fifth contact remains undecoded. Neither the 32-mode plus sparse-DCT
residual nor equal-budget sparse-DCT target record passes the five-endpoint
gate. Preserve the [exact result](../development/physical-sound-r3a-blue-bowl-representation-gate-2026-08-30.md),
authorize no neural model and start a bounded V2 source/representation research
cycle with a new unopened development projection. Do not tune this family on
the opened Blue Bowl development contact.

V2 result: `INCONCLUSIVE_RESAMPLING_CONTROL`. The new Large Swan source,
source-order `3 fit / 1 development / 1 sealed` split, pinned DAC dependency,
sealed extraction and query-seeing oracle all repeat byte-identically. The
48→44.1→48 kHz no-codec control fails full-band level and decay before the
learned codec can receive representation credit; DAC also fails four absolute
candidate endpoints. Preserve the
[exact result](../development/physical-sound-r3a-v2-large-swan-dac-oracle-result-2026-08-30.md),
keep row `2407` sealed and authorize neither N0.4B nor neural training.

V3 result: `REJECT_LEARNED_CODEC_REPRESENTATION`. V3A froze Plastic Bin and
stopped before metrics when the unguarded NDAC decoder returned `143,992`
samples for the three-second target. An eight-sample guard was derived from a
synthetic control only. V3B then froze unopened Purple Scoop; preflight,
sealed extraction and two native-48 kHz NDAC-75 oracles repeat byte-identically.
Identity is exact zero, but NDAC fails spectrum (`12.1198 dB`) and modal
frequency (`560.81` cents). Preserve the
[exact result](../development/physical-sound-r3a-v3-native-ndac-result-2026-08-30.md),
keep both row-`2407` holdouts sealed and do not search another general codec on
opened targets.

V4 result: `REJECT_FIT_REPRESENTATION`. The implementation is frozen at commit
`e05d5593`; two preflights and fit runs reproduce all JSON/NPY artifacts.
`compact`, `balanced` and `extended` pass shared/per-contact budgets, but every
one of twelve fit contacts fails the spectrum endpoint. No development or
sealed waveform is decoded. Preserve the
[exact V4 result](../development/physical-sound-r3a-v4-fit-probe-and-neural-rebaseline-2026-08-31.md)
and do not run its evaluator, add another capacity or reopen this analytical
family.

V5 neural-representation implementation protocol:

1. Build a hash-closed external training pack from published impact recordings
   whose source/object groups are disjoint from the four development objects,
   future representation holdout, method holdout and admission shadow.
2. Freeze one small 48 kHz mono convolutional encoder, residual vector
   quantizer and decoder with periodic activations. Freeze no more than three
   latent capacities before training.
3. Freeze waveform/SI-SDR, multiresolution complex and log-spectrum,
   multi-scale mel, envelope/decay and modal/pitch loss terms. The unchanged
   five evaluator endpoints remain decision authority.
4. Before real training, require synthetic identity, tiny-corpus overfit,
   non-collapsed codebook usage, exact checkpoint reload and repeat inference.
   These controls read no development waveform.
5. Train on the internet train role plus twelve authorized fit contacts. Open
   the four development contacts once to select the smallest capacity that
   passes every endpoint and nearest-fit comparison. Architecture, losses,
   thresholds, stopping and seeds cannot change afterward.
6. Only a development pass may freeze one new source-disjoint REALIMPACT
   `3 fit / 1 development / 1 sealed` representation holdout. Its fifth
   contact remains sealed.

Only `V5-HOLDOUT = READY_FOR_EXACT_OBJECT_FIELD` opens N0.4B. It authorizes an
external contact-to-latent field and offline asset baking, not runtime neural
inference, public content contracts or production quality.

V5 preflight A result: `READY_FOR_TRAINING_ENVIRONMENT_FREEZE`. Two runs at
implementation `512b35dd` repeat manifest `1cc23496…fb31` and report
`38176a41…3bf3`. The source inventory contains `73` Heller/CMU clips in `17`
groups (`56` train, `17` internal validation); numeric waveform decode,
development reads and real training steps are all zero. Full-model inference
repeats exactly and the 80-step micro-codec overfit repeats with improvement
ratio `0.1499882595`. Preserve the
[exact result](../development/physical-sound-r3a-v5-neural-preflight-a-2026-08-31.md).

The external CUDA/PyTorch runner boundary is also complete. Two runs at
implementation `90984de2` repeat manifest `75ed5e06…2679`, report
`a4e30d87…b1e1` and checkpoint `74fccfac…7659` byte for byte. Full
loss/backward is finite, waveform L1 improves `24.0%`, train stages use
`[8,7,6,7]` codes and checkpoint continuation is exact. The original uniform
codebook start is rejected after using one code in every stage; deterministic
first-train-latent residual-share initialization is frozen instead. Preserve
the [exact runner result](../development/physical-sound-r3a-v5-training-runner-preflight-2026-08-31.md).

The paired `6/12/24 kbps` frontier and one frozen development evaluation are
complete. All three capacities pass internal anti-collapse and record-budget
checks, but fail the four real development objects: spectrum and modal
frequency fail `12/12` comparisons, decay fails `11/12`, and no candidate
beats nearest fit on the required four endpoints. Two evaluator runs repeat
report `74a6f4ea…bbb4`; sealed, method-holdout and admission-shadow reads are
zero. Preserve the [exact result](../development/physical-sound-r3a-v5c-capacity-frontier-and-development-result-2026-08-31.md).

N0.4A V5 closes as `REJECT_NEURAL_REPRESENTATION`. No V5 capacity, new
representation holdout, N0.4B field or baked atlas is authorized. Do not tune
losses, thresholds, gain, postfilters, stopping or bitrate using the opened
development contacts.

V8 reopens N0.4A only for the materially different explicit-modal hypothesis
frozen in the [V8 rebaseline](../development/physical-sound-v8-explicit-modal-neural-rebaseline-2026-08-31.md).
Its implementation protocol is:

1. Validate two exact-hash opened NISR Glass FEM label files outside Git. Use
   only published frequencies, coordinates, surface encoding and 3D mode
   shapes; infer no missing force, normal ordering or real acoustics.
2. Repeat a deterministic float64 damped-sinusoid recovery control from bounded
   perturbations of known synthetic truth. Require frequency, damping, gain and
   waveform gates before any field credit.
3. For each NISR object, select `ceil(20%)` boundary context positions by
   deterministic farthest-point sampling. Predict all `3 x 20` held mode-shape
   components from coordinate Fourier features and published surface encoding.
4. Compare the coordinate MLP with context-mean and nearest-context baselines.
   Require every object to reach normalized RMSE no greater than `0.50` times
   nearest-neighbour, remain finite/nonconstant and repeat exactly.
5. Publish only manifest, report, hashes and metrics outside Git. Source
   payloads and weights remain external. Real-development and sealed reads are
   zero.
6. A complete synthetic pass returns
   `READY_FOR_FRESH_REAL_MODAL_PROTOCOL`. It authorizes a new preregistered
   ObjectFolder Real fit/development protocol only—not quality, N0.4B, atlas,
   validator or runtime work.
7. A failure returns `STOP_V8_SYNTH_KEEP_CLIPS`; diagnose before selecting or
   reading any fresh real V8 source.

V8-SYNTH result: `READY_FOR_FRESH_REAL_MODAL_PROTOCOL`. Two independent runs
repeat manifest `b2bb931d…6934` and report `391854fe…3b78`. Known modal
frequency/damping/gain recovery passes, and the two held NISR mode-shape fields
reach `0.262x` and `0.097x` nearest-neighbour RMSE. All real, sealed, method and
shadow reads remain zero. Preserve the
[exact result](../development/physical-sound-r3a-v8-synthetic-preflight-result-2026-08-31.md).
The result authorizes only a fresh ObjectFolder Real source/gate freeze.

### N0.4B — Object-specific contact-position few-shot field

Entry condition: N0.4A V8 passes fresh real development and one source-disjoint
representation holdout, and published data provides multiple contact positions
for one exact object.

Deliverables:

- external, hash-closed training manifest for one exact object with multiple
  contact observations at a canonical listener condition;
- geometry-aware contact-to-modal-gain/mode-shape model for the frozen V8
  representation, plus coverage/OOD output;
- training/evaluation runner with fixed seeds and environment lock;
- ablations for nearest modal response, coordinate-only and geometry-aware
  fields;
- deterministic offline decode and baked contact-atlas export for every
  selected prediction.

Exit criteria:

- held-out contact positions beat every compatible frozen classical baseline
  on the preregistered primary aggregates;
- all hard, causal and energy-scaling controls pass;
- results repeat within the declared training tolerance and baked PCM/clip
  hashes repeat exactly;
- failure/OOD conditions choose fallback.

Fallback: a failure closes the current contact-to-modal-field hypothesis. Preserve
ordinary authored clips and do not add runtime inference.

Commit boundary: model interface/runner, compact fixtures and a report. Weights
and datasets remain external.

### N0.5 — Shared geometry-conditioned surrogate

Entry condition: N0.4B proves the representation on held-out contact positions.

Deliverables:

- geometry encoder and shared modal/contact-gain heads trained only on the
  frozen train role;
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
- explicit `GO_EXACT_OBJECT`, `GO_SHARED`, `REJECT_REPRESENTATION` or
  `DATA_INSUFFICIENT` decision; listener-radiation evidence is reported
  separately and cannot be inferred from contact-position success.

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
| Listener position | Canonical condition only in N0.4B; exact-grid diagnostics separate | Future dense/solver-backed radiation holdout | Never infer radiation from contact-field success |
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

1. Preserve direct, phase-aligned, R2C and R2E listener fields as immutable
   rejected revisions; never reuse their opened query for model selection.
2. Preserve N0.3D V2 as proof that the coefficient objective/cooker can train,
   not as evidence that rank 96 generalizes to hidden positions.
3. Preserve N0.4A V1 as a reproducible `REJECT_REPRESENTATION`; Blue Bowl row
   1807 is opened negative evidence and row 2407 remains sealed.
4. Preserve N0.4A V2 as reproducible `INCONCLUSIVE_RESAMPLING_CONTROL`; Large
   Swan row `1807` cannot select post-hoc filters, metrics, bitrate or codec.
5. Preserve N0.4A V3A as invalid infrastructure evidence and V3B as a
   reproducible native-rate learned-codec rejection; do not retry nearby
   codecs, bitrates or postfilters on opened Plastic Bin/Purple Scoop targets.
6. Preserve N0.4A V4 as reproducible `REJECT_FIT_REPRESENTATION`; do not run
   development evaluation or another modal/PCA/bin capacity.
7. Preserve V5-PREFLIGHT-A as `COMPLETE / REPRODUCIBLE`; do not change its
   corpus, architecture, loss or capacities after later observations.
8. Preserve the V5 GPU environment and runner controls as
   `COMPLETE / REPRODUCIBLE`; do not restore the dead-code initialization.
9. Preserve the reproducible V5-C development result as
   `REJECT_NEURAL_REPRESENTATION`; select no capacity and spend no new holdout.
10. Preserve V8-SYNTH as `COMPLETE / REPRODUCIBLE`; do not widen its synthetic
    result into a real-quality claim.
11. Inventory a bounded official ObjectFolder Real slice without waveform
    decode, then freeze exact source hashes, object/contact roles, modal
    initializer, damping/residual ablations and fit/development gates.
12. Keep N0.4B closed. Only a later V8 fresh-real development and
    source-disjoint representation-holdout pass may open it; the product
    continues to use authored clips.

The [object-91 source/gate revision](../development/physical-sound-r3a-v8-objectfolder-real-source-and-gate-freeze-2026-08-31.md)
now freezes the official archive identity, exact 512 MiB prefix, member
commitments and `18/12/4 fit`, `20 development`, `27 sealed` roles. Implement
the zero-decode inventory before any fit extraction.

The inventory now returns `READY_FOR_V8_REAL_FIT_EXTRACTION` twice with exact
manifest `e1651b64…147b` and report `d9888302…dace`; see the
[exact result](../development/physical-sound-r3a-v8-objectfolder-real-inventory-result-2026-08-31.md).
Freeze the complete fit-only runner before decoding contacts `18/12/4`.

The [fit protocol](../development/physical-sound-r3a-v8-object91-fit-protocol-2026-08-31.md)
now fixes native extraction/alignment, 64 measured-force modal responses, two
damping capacities, a `14,500`-bin phase-preserving residual, 63–64 KiB record
costs and unchanged acoustic gates. Implement and commit the runner before the
first numerical fit read.
