# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V4 / GEOMETRY_AWARE_MODAL_CONTACT_FIELD / R2_LISTENER_FIELD_REJECTED / R3A_V1_REPRESENTATION_REJECTED / R3A_V2_RESEARCH_NEXT / SHADOW_SEALED / FALLBACK_REQUIRED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-30` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture, external neural acoustic-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, cooks to exact bounded coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R3A V1 freezes and repeats the new Blue Bowl contact
  task, but both 512-scalar query-seeing representations fail the real
  development gate. Neural training remains forbidden.
- **Exact evidence:** [R3A Blue Bowl representation gate](../physical-sound-r3a-blue-bowl-representation-gate-2026-08-30.md),
  manifest `6ad5b42c…847b`, extraction `4d364a28…b827`, repeated oracle
  `34567bc2…ef01`.
- **Why this does not reject neural physical sound:** The oracle rejects one
  local-peak modal estimator, sparse-DCT residual and 512-scalar budget. It
  does not test a structured-pole/multiresolution codec or a geometry-aware
  contact model; SPEC-08 still owns environmental spatialization.
- **Next action:** Run a bounded R3A V2 research cycle for a materially
  different structured-pole/multiresolution or learned-codec representation
  and a new unopened development projection.
- **After that:** Freeze and run exactly one new representation oracle; only a
  pass may authorize one geometry-aware exact-object field.
- **Current blockers:** a sufficient unopened corpus/representation, learned quality,
  multi-impact internet coverage,
  independent validator risk, exact-domain admission, production contact
  projection and a player-visible consumer are open.
- **Product boundary:** SPEC-45 remains `Proposed`; clips remain mandatory;
  model inference, datasets, weights and validator do not enter runtime or Git.

## Current program state

| Stage | State | Exact consequence |
| --- | --- | --- |
| R0 real-data boundary | `COMPLETE` | Signal semantics and five disjoint roles are frozen; absent axes stay absent. |
| R1 honest controls | `COMPLETE` | Transfer nearest/linear and waveform Q30/DCT controls are immutable. |
| R2 direct/phase field | `REJECTED` | Do not tune the opened 15-row time-domain latent family. |
| R2B dense data/representation | `COMPLETE` | 600 rows, `420 context / 180 query`, complex inverse and three controls repeat; no quality credit. |
| R2C separable complex field | `COMPLETE / REJECTED` | Data-only and Helmholtz candidates repeat; both collapse and fail `4/5` endpoints. |
| R2D trainability gate | `COMPLETE / V2_PASS` | V2 passes all unchanged gates twice; normalized reports, checkpoints and WAVs repeat with zero query reads. |
| R2E low-rank coefficient field | `COMPLETE / REJECTED` | Exact context fit loses every query endpoint; post-reject oracle proves representation plus interpolation limits. |
| R3A new data/representation | `V1 COMPLETE / REJECTED; V2 NEXT` | Blue Bowl source/splits/extraction repeat, but modal+residual and sparse DCT both fail. Freeze a materially different representation on new unopened development data. |
| R3B+ exact-object/validator/admission | `BLOCKED` | No contact-field model, validator release, admitted domain or runtime promotion exists. |

## Material transition: R2C rejection and Roadmap V3

- **Observation:** Both frozen R2C candidates complete deterministic GPU
  training and cooking, yet output near-silent query WAVs. The no-physics and
  Helmholtz variants have nearly identical level/spectrum failure.
- **Evidence:** Training report hashes are `12f89e3…63bd` and
  `3833c2a3…4a8b`; evaluation report is `63c2eab6…bb1b`; the repeated
  context-only diagnostic is `1263e02f…a8f`.
- **Conclusion:** Physics regularization is not the primary cause and rank 96
  is not yet the limiting representation ceiling. Loss sampling, energy
  scaling and clipped optimization fail before held-listener generalization.
- **Decision:** Rebaseline the [canonical roadmap](../../plans/physical-sound-synthesis-roadmap.md)
  to V3. Insert a query-free trainability gate, then separate time/frequency
  basis learning from spatial coordinate prediction.
- **Rejected alternatives:** Larger SIREN, more steps, another seed/rank,
  Helmholtz-weight grid, relaxed endpoints or query-driven debugging.
- **Consequences:** R3 remains blocked. R2B data readiness is preserved, R2C
  is immutable negative knowledge, and method holdout/admission shadow remain
  sealed.
- **Remaining uncertainty:** An energy-preserving objective may still fail;
  even a passing context fit may not beat interpolation on grouped listeners.
- **Reconsideration condition:** A new frozen context revision passes all
  micro-overfit/trivial-oracle gates and a subsequent one-shot grouped query
  result identifies a different limiting factor.
## Material transition: R2D V1 fixed-step rejection

- **Observation:** Two byte-identical V1 runs pass one-row, coefficient,
  cooker, oracle-proximity, clipping and trivial-control gates. Only eight-row
  and full-context mean absolute log energy miss `0.005`, at `0.007785` and
  `0.005062`.
- **Evidence:** [R2D V1 result](../physical-sound-listener-field-r2d-trainability-v1-result-2026-08-30.md),
  manifest `3fe41295…5f28`, repeated report `350a1e6b…0afd`.
- **Conclusion:** Representation, objective direction and cooker are supported;
  fixed terminal AdamW step is the remaining falsifiable cause.
- **Decision:** Preserve V1 as rejected. V2 changes only to deterministic
  learning-rate decay and retains every original gate; N0.3E stays blocked.
- **Rejected alternatives:** Relax `0.005`, add steps at fixed rate, modify
  basis/loss/tasks or inspect query audio.
- **Reconsider when:** V2 repeats and either passes every unchanged gate or
  identifies a different single failing boundary.

## Material transition: R2D V2 trainability pass

- **Observation:** Half-cosine `0.05 -> 0.00001` makes all three unchanged V1
  tasks pass; full-context log-energy falls to `4.54e-9` with zero clipping.
- **Evidence:** [R2D V2 result](../physical-sound-listener-field-r2d-trainability-v2-result-2026-08-30.md),
  manifest `47920fce…f6d1`, repeated report `898ee201…0875`.
- **Conclusion:** The basis/objective/cooker training substrate preserves the
  signal; the R2C silence collapse was an optimization failure, not a rank-96
  capacity failure at the context-fit boundary.
- **Decision:** Close R2D and authorize exactly one separately frozen N0.3E
  data-only coordinate field. Query-informed tuning remains forbidden.
- **Remaining uncertainty:** Coordinate generalization may still lose to the
  frozen nearest/linear controls on the 180 grouped held listeners.
- **Reconsider when:** One immutable N0.3E result passes or rejects the entire
  low-rank coordinate field under the unchanged endpoints.

## Material transition: R2E rejection and Roadmap V4

- **Observation:** The frozen harmonic field reaches essentially zero context
  objective and repeats exactly, yet is worse than every frozen control on all
  five query endpoints.
- **Evidence:** Training report `c23a82d0…f790`, checkpoint
  `8b96da41…355d`, evaluation `efab8e2c…2327`. Two post-reject projection
  diagnostics repeat report `b6dcc5fc…47ac2`.
- **Conclusion:** Optimization is no longer the blocker. Coordinate
  interpolation fails, and the context-only rank-96 basis is also insufficient
  on query (`0.2760` NRMSE, `92.38%` retained energy, failed mean spectrum).
- **Decision:** Close the R2 listener-field family and rebaseline to
  [Roadmap V4](../../plans/physical-sound-synthesis-roadmap.md). The first
  neural product task becomes geometry-aware contact-position sound at one
  canonical listener condition. Radiation is a separate later claim.
- **Rejected alternatives:** A larger harmonic/MLP field, rank/seed/step grid,
  query-informed basis selection, relaxed endpoints or blaming only the
  high-frequency bins.
- **Consequences:** The opened Green Goblet split is negative evidence only.
  R3A requires new internet data and a representation-oracle gate before any
  model training. Clip fallback and sealed admission roles remain unchanged.
- **Reconsider when:** A new unopened corpus proves that a bounded modal/
  residual representation carries held contact sounds and authorizes one
  exact-object field.

## Material transition: R3A V1 representation rejection

- **Observation:** Two zero-audio source preflights and bounded four-contact
  extractions repeat exactly. The query-seeing 512-scalar modal-plus-residual
  oracle improves development level and envelope but loses spectrum, modal
  frequency and decay; equal-budget sparse DCT improves only envelope.
- **Evidence:** [R3A V1 result](../physical-sound-r3a-blue-bowl-representation-gate-2026-08-30.md),
  manifest `6ad5b42c…847b`, contact array `1971f01a…1ba3`, repeated oracle
  report `34567bc2…ef01`.
- **Conclusion:** The selected source and seal are usable, but local STFT peak/
  damping extraction plus a sparse whole-window DCT residual is not a
  sufficient compact record for this real glass transfer response.
- **Decision:** Preserve V1 as `REJECT_REPRESENTATION`, authorize no neural
  model and require a new unopened development projection plus materially
  different representation for V2.
- **Rejected alternatives:** More nearby modes/DCT bins, threshold relaxation,
  budget/seed tuning on opened Blue Bowl row 1807, or opening row 2407.
- **Consequences:** R3B and validator/admission work remain blocked; authored
  clips stay mandatory; field holdout, method holdout and shadow remain sealed.
- **Remaining uncertainty:** Structured complex poles, multiresolution
  transient atoms or a learned codec may carry the target; richer
  ObjectFolder Real access may be needed to learn such a representation.
- **Reconsider when:** A hash-closed new source/projection and preregistered V2
  oracle exist before its development audio is opened.

## Stable decisions

| ID | Decision | Reconsideration condition |
| --- | --- | --- |
| D-001 | Physical audio is a presentation consumer; gameplay hearing continues to use deterministic `AcousticFactV1`. | Only a superseding Accepted ADR could change authority. |
| D-002 | Impact is the first source class; rolling, scraping, fracture, cloth, fluid, fire and biological sound need separate models/evidence. | An admitted impact vertical plus a separately scoped source consumer. |
| D-003 | Acoustic profiles stay PresentationOnly and separate from physics-material authority. | A concrete consumer proves a shared physical source-of-truth field. |
| D-004 | Production wiring waits for engine-owned committed contact projection and relevant physics evidence; no raw PhysX callback path. | The projection and its ProductCheck exist. |
| D-005 | First neural value is offline cooking into bounded deterministic coefficients, not runtime inference. | A measured product need and separate ADR define artifact, budget, fault and fallback. |
| D-006 | Required real evidence is internet-sourced; the user performs no local impact recording. | Explicit product-owner reversal only. |
| D-007 | Validation is an independent frozen ensemble with hard gates, acoustic specialists, learned diagnostics and calibrated OOD; no per-sound human queue. | A simpler policy demonstrates equal bounded risk and useful coverage. |
| D-008 | Missing geometry/support/force/listener axes narrow the claim and never get inferred from a label. | A hash-closed published source supplies the axis. |
| D-009 | R2C data-only/Helmholtz separable SIREN revision is retired. | New evidence invalidates the context diagnostic, not merely a new hyperparameter. |
| D-010 | The Green Goblet R2 listener split and rank-96 harmonic family are retired from model selection. | A genuinely denser independent listener corpus or validated radiation solver creates a new split and representation. |
| D-011 | The first neural product task is impact/contact-position sound at one canonical listener; detailed source radiation is later and independent. | An exact player-visible consumer proves radiation must precede contact variation. |

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Energy-preserving context training is sufficient to remove silence collapse | R2D V2 passes all context gates twice | No counterevidence remains at the context-fit boundary | Closed; preserve V2 as the N0.3E training substrate |
| H2: Frozen rank-96 listener coefficients beat interpolation | Context fit is exact | R2E loses all endpoints; query oracle also misses mean spectrum | Closed/rejected for the opened split |
| H3: A compact modal/residual record carries held contact-position sound | AV-MSF and modal physics provide a matching factorization | R3A V1 local-peak modes plus sparse DCT fails real level/spectrum/mode/decay gates despite query access | New unopened V2 structured-pole/multiresolution or learned-codec oracle |
| H4: Geometry-aware exact-object contact learning is possible from published data | REALIMPACT source/geometry/splits now repeat; ObjectFolder Real publishes 30–50 impacts/object; AV-MSF reports few-shot results | No compact representation has passed, so no honest model has trained | R3A V2 representation pass, then one R3B candidate |
| H5: Automatic validator can reach useful coverage at bounded false-pass risk | Hard/acoustic/corpus components and grouped roles exist | No frozen independent release or shadow result exists | R5 calibration/holdout release after a generator claim exists |
| H6: Cooked neural coefficients fit a useful production budget | Q30 lab reference is compact and exact | Whole-mixer, callback and varied-voice cost are unmeasured | Player-visible consumer plus whole-mixer p95/p99 before promotion |

## Do not retry

- R2 direct/phase ranks, width, epochs, seed, phase speed or opened thresholds.
- R2C separable SIREN with nearby size/step/rank/seed/Helmholtz-weight changes.
- R2E harmonic listener field, context-rank sweep or any architecture selected
  on the opened 180-query Green Goblet split.
- Query-informed normalization, checkpoint selection, stopping or trainability
  debugging.
- Universal `material -> sound` coefficients or schemas before exact-object
  evidence and a consumer.
- Another unguided manual residual family, blind preset tuning or synthetic
  target match presented as real glass/wood/metal identity.
- Prompt-to-waveform as the primary engine path; it may remain report-only or
  an authored-asset source.
- A single FAD/CLAP/ViSQOL/aesthetic/audio-language score as validator or a
  live per-sound human approval queue.
- Local microphone/hammer capture, raw PhysX-callback mixing or runtime neural
  inference first.

## Required context

Read in precedence order:

1. [Agent routing](../../architecture/agent-routing.md), SPEC-00 and SPEC-01.
2. SPEC-08/24/26/30 and ADR-027/046/058/071.
3. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
4. [Roadmap V4](../../plans/physical-sound-synthesis-roadmap.md),
   [implementation plan](../../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md)
   and [neural strategy](../physical-sound-neural-acoustic-field-strategy-2026-08-30.md).
5. [R0–R1 boundary](../physical-sound-neural-real-boundary-r0-r1-2026-08-30.md),
   [R2 phase failure](../physical-sound-listener-field-r2-phase-research-2026-08-30.md),
   [R2B preflight](../physical-sound-r2b-dense-complex-field-preflight-2026-08-30.md),
   [R2C result](../physical-sound-listener-field-r2c-result-2026-08-30.md),
   [R2D V2 result](../physical-sound-listener-field-r2d-trainability-v2-result-2026-08-30.md)
   and [R2E/V4 result](../physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md).
6. [Main product roadmap](../../roadmap.md) for scheduling or promotion facts.

## Handoff

- **Workspace:** R3A V1 source preflight, sealed streaming extraction,
  equal-budget representation oracle and compact tests are implemented; two
  real repetitions preserve one immutable rejection and all heavy artifacts
  remain external. R3A V2 research is next.
- **Quality:** No neural candidate, validator release, admitted formula record
  or runtime integration exists. Clip fallback is still authoritative.
- **Isolation:** The R2 listener query and Blue Bowl development row 1807 are
  opened and retired from selection. Blue Bowl row 2407, method holdout and
  admission shadow remain unopened.
- **Next commit boundary:** Bounded R3A V2 research and freeze for a materially
  different representation plus new unopened development projection, with no
  model training until that oracle passes.
