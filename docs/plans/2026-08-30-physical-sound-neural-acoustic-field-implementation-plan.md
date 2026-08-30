# Physical sound neural acoustic field — implementation plan

| Field | Value |
|---|---|
| Date | 2026-08-30 |
| Status | `IN_PROGRESS / PS-2N0_CONTRACT_IMPLEMENTED / SYNTHETIC_FIXTURE_PASS / REAL_PROJECTION_AND_N0.2_NEXT / RESEARCH_ONLY` |
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
- The current frozen Q30/DCT path remains an immutable baseline.

## Work packages

### N0.1 — Freeze the data projection and split manifest

Status: `IMPLEMENTED / SYNTHETIC_FIXTURE_PASS / REAL_PROJECTION_NOT_RUN`.
See [PS-2N0 evidence](../development/physical-sound-neural-data-plane-ps2n0-2026-08-30.md).

Deliverables:

- a versioned external row projection for object, geometry, support, impact,
  listener, recording and provenance claims;
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
- no model is trained in this package.

Fallback: if the data cannot support impact/listener conditioning, narrow N0.2
to the strongest exact object/axis instead of manufacturing a broad domain.

Commit boundary: projection contract, fixtures, focused tests and evidence.

### N0.2 — Export the deterministic classical benchmark

Deliverables:

- one external command that converts a frozen row into the current Q30/DCT
  prediction and structured acoustic record;
- canonical mode ordering, residual descriptor and coverage metadata;
- per-row acoustic features needed by both neural candidates and validator
  specialists;
- exact A/B report and PCM hashes on synthetic fixtures and permitted
  development rows.

Exit criteria:

- existing frozen hashes and negative controls do not change;
- the exporter is deterministic and fails closed on stale lineage;
- baseline outputs are available for every supported benchmark task;
- unsupported rows are `FallbackOutOfDomain`, not partial success.

Fallback: retain the existing renderer as a PCM-only baseline if a structured
field cannot be recovered without changing its semantics.

Commit boundary: baseline exporter plus non-regression evidence.

### N0.3 — Object-specific few-shot field

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

### N0.4 — Shared geometry-conditioned surrogate

Entry condition: N0.3 proves the representation on held-out positions and
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

### N0.5 — Freeze the neural feasibility benchmark

Deliverables:

- one manifest binding data, roles, preprocessing, baseline, candidates,
  metrics, environment, seeds and output hashes;
- per-object/position/listener distributions and failure clustering;
- report-only direct-waveform upper bound when reproducible;
- explicit `GO_FEW_SHOT`, `GO_SHARED`, `REJECT_REPRESENTATION` or
  `DATA_INSUFFICIENT` decision.

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

Entry condition: one N0 candidate and Validator Release V1 are both frozen.

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

1. Freeze N0.1 row projection, roles and leakage audit.
2. Implement N0.2 deterministic baseline export without changing frozen
   Q30/DCT behavior.
3. Preregister N0.3 on the best synchronized multi-impact/multi-listener exact
   object available from published internet data.
4. Train the first few-shot field externally and publish its ablations.
5. Decide `GO_FEW_SHOT`, `GO_SHARED`, `REJECT_REPRESENTATION` or
   `DATA_INSUFFICIENT` before opening a second architecture family.
