# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V4 / GEOMETRY_AWARE_MODAL_CONTACT_FIELD / R2_LISTENER_FIELD_REJECTED / R3A_DATA_REPRESENTATION_NEXT / SHADOW_SEALED / FALLBACK_REQUIRED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-30` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture, external neural acoustic-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, cooks to exact bounded coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** The fixed-impact Green Goblet listener field is
  closed. R2E fits context exactly but loses all five held-listener endpoints;
  a repeated query projection oracle shows both rank-96 representation and
  coordinate interpolation limitations.
- **Exact evidence:** [R2E result and V4 research](../physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md),
  evaluation `efab8e2c…2327`, repeated diagnostic `b6dcc5fc…47ac2`.
- **Why this does not reject neural physical sound:** The failed task predicts
  detailed listener radiation from a coarse published grid. The next product-
  aligned task predicts contact-position variation at one canonical listener;
  SPEC-08 continues to own environmental spatialization.
- **Next action:** Audit and freeze a new internet-only multi-object/multi-
  impact corpus and its canonical-listener/contact-position splits.
- **After that:** Compare compact representation oracles on query-seeing
  development contacts before authorizing one geometry-aware modal field.
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
| R3A new data/representation | `NEXT` | Freeze a new canonical-listener multi-impact corpus and pass a compact representation oracle before training. |
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
| H3: A compact modal/residual record carries held contact-position sound | AV-MSF and modal physics provide a matching factorization | No Next Engine oracle exists on a new unopened multi-impact projection | R3A representation-oracle benchmark |
| H4: Geometry-aware exact-object contact learning is possible from published data | REALIMPACT/ObjectFolder Real publish geometry, force and multiple contacts; AV-MSF reports few-shot results | Availability, exact lineage and compatible Next Engine metrics are not frozen | R3A source audit, then one R3B candidate |
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

- **Workspace:** R2E tooling/tests, two exact training repeats, one immutable
  rejection and two post-reject diagnostics are implemented; all heavy
  artifacts remain external. N0.4A/R3A is next.
- **Quality:** No neural candidate, validator release, admitted formula record
  or runtime integration exists. Clip fallback is still authoritative.
- **Isolation:** The R2 query is opened and permanently retired from selection;
  method holdout and admission shadow remain unopened.
- **Next commit boundary:** New source/corpus preflight and compact
  representation-oracle gate, with no model training.
