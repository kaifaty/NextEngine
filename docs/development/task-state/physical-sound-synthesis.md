# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V6 / R2_LISTENER_FIELD_REJECTED / R3A_V1_REJECTED / R3A_V2_INCONCLUSIVE / R3A_V3B_NATIVE_NDAC_REJECTED / R3A_V4_MODAL_BOTTLENECK_NEXT / FALLBACK_REQUIRED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-30` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed external neural contact-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, cooks exact bounded coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Native-48 kHz NDAC-75 is reproducibly rejected as
  the R3A V3B representation ceiling. Identity is exact zero, but spectrum
  (`12.1198 dB`) and modal frequency (`560.81` cents) fail.
- **Exact evidence:** [R3A V3 result](../physical-sound-r3a-v3-native-ndac-result-2026-08-30.md),
  V3B manifest `57dcc3b3…8959`, contacts `2dcab659…0b3c`, repeated oracle
  `bcc24ef9…f8e1`; sealed row samples decoded: `0`.
- **Infrastructure precursor:** V3A Plastic Bin stopped before quality metrics
  because NDAC returned `143,992/144,000` samples. The eight-sample guard was
  derived on synthetic audio only; Plastic Bin was not re-evaluated.
- **Next action:** Implement Roadmap V6 R3A V4-DEV over already-opened contacts:
  shared stable modal poles, contact gains and a learned multiresolution
  residual basis with deterministic inverse at no more than three capacities.
- **Spend rule:** Do not open another object unless one bounded V4-DEV
  representation passes all five endpoints; then freeze it once for V4-HOLDOUT.
- **Training rule:** Neural contact-field training remains forbidden until the
  representation and deterministic cooker pass an unopened V4 holdout.
- **Product boundary:** SPEC-45 remains `Proposed`; clips remain mandatory;
  model inference, datasets, weights, WAVs and validator stay external.

## Current program state

| Stage | State | Exact consequence |
| --- | --- | --- |
| R0 real-data boundary | `COMPLETE` | Signal semantics and five disjoint roles are frozen; absent axes stay absent. |
| R1 honest controls | `COMPLETE` | Transfer nearest/linear and waveform Q30/DCT controls are immutable. |
| R2 listener field | `REJECTED` | Direct/phase, separable and low-rank coordinate fields are closed; do not tune the opened listener query. |
| R2D trainability | `COMPLETE / V2_PASS` | Objective/cooker can fit context exactly; optimization is not the current blocker. |
| R3A V1 | `REJECTED` | 512-scalar modal+DCT and equal sparse-DCT records lose real development structure. |
| R3A V2 | `INCONCLUSIVE_CONTROL` | 44.1 kHz conversion changes full-band level/decay; no codec credit. |
| R3A V3A | `INVALID_INFRASTRUCTURE` | Plastic Bin target opened, no metrics; eight-sample NDAC deficit recorded, holdout sealed. |
| R3A V3B | `REJECTED / REPRODUCIBLE` | Native identity passes; NDAC preserves coarse decay/envelope but loses spectrum/modes. |
| R3A V4 | `NEXT` | Domain-specific modal/gain/learned-residual bottleneck must pass before any field model. |
| R3B+ | `BLOCKED` | No contact-field model, validator release, admitted domain or runtime promotion exists. |

## Material transition: native codec rejected and Roadmap V6

- **Observation:** Two Purple Scoop V3B runs reproduce all reports, codes and
  WAVs. Native identity has zero error. NDAC beats nearest fit on level,
  envelope and decay but is worse on spectrum and modal-frequency endpoints.
- **Evidence:** Preflight `845d4598…66d5`, extraction `7f2d7de4…7164`, oracle
  `bcc24ef9…f8e1`, learned WAV `a6d668ff…2b97`; row `2407` is undecoded.
- **Conclusion:** Sample-rate conversion is no longer the explanation. A
  general perceptual codec can preserve macroscopic sound shape while moving
  narrow resonances required by a physical transfer representation.
- **Decision:** Close nearby general-codec/bitrate/postfilter search. Rebaseline
  to [Roadmap V6](../../plans/physical-sound-synthesis-roadmap.md): learn only
  the task-specific modal residual representation and later the spatial gain
  field; keep the final decoder deterministic.
- **Rejected alternatives:** Retune NDAC on opened Purple Scoop, apply the
  stochastic FlowDec postfilter, relax modal/spectrum thresholds, open either
  row `2407`, or treat perceptual audition as physical fidelity.
- **Consequences:** Purple Scoop row `1807` joins the development corpus;
  V3B does not authorize deterministic distillation or neural field training.
- **Remaining uncertainty:** A multi-contact stable pole bank plus learned
  residual dictionary may preserve real transfer responses within a useful
  cooker budget; V1's 32 local modes do not answer this stronger hypothesis.
- **Reconsideration condition:** One preregistered V4 representation and exact
  inverse pass the opened multi-object frontier and then a new unopened object.

## Durable negative knowledge

- R2 direct/phase and separable listener fields collapse or lose every held
  endpoint; R2E proves both representation and interpolation limits.
- R2D V2 proves the coefficient training/cooker substrate itself can fit;
  further optimizer rescue is not justified.
- R3A V1 rejects one 512-scalar 32-mode+sparse-DCT family, not modal
  factorization at a measured capacity frontier.
- R3A V2 is unusable for codec judgment because its resampling control fails.
- R3A V3A is invalid infrastructure evidence only; never derive quality from it.
- R3A V3B cleanly rejects exact NDAC-75 `800k` at 7.5 kbps on Purple Scoop.

Detailed histories remain in the linked R2/R3 evidence reports; this file is
kept bounded as the current resume surface.

## Stable decisions

| ID | Decision | Reconsideration condition |
| --- | --- | --- |
| D-001 | Physical audio is presentation-only; gameplay hearing remains deterministic `AcousticFactV1`. | A superseding Accepted ADR. |
| D-002 | Impact is first; rolling, scraping, fracture, cloth, fluid, fire and biological sound need separate evidence. | An admitted impact vertical and separately scoped consumer. |
| D-003 | Acoustic profiles stay separate from physics-material authority. | A consumer proves a shared physical source-of-truth field. |
| D-004 | Production waits for engine-owned committed contact projection; no raw PhysX callback path. | Projection and ProductCheck exist. |
| D-005 | Neural inference is offline; runtime receives bounded deterministic coefficients only. | A measured need and separate ADR define runtime model/fault/fallback. |
| D-006 | Real evidence is internet-sourced; the user performs no local impact recording. | Explicit product-owner reversal. |
| D-007 | Validation is an independent frozen ensemble with hard gates and calibrated OOD, not a per-sound human queue. | A simpler policy proves equal bounded risk and coverage. |
| D-008 | Missing geometry/support/force/listener axes narrow the claim and are never inferred from labels. | A hash-closed source supplies the axis. |
| D-009 | The opened Green Goblet listener split and rank-96 field family are retired. | A denser independent corpus or validated solver creates a new task. |
| D-010 | First neural product task is exact-object contact variation at one canonical listener; radiation is later. | A visible consumer proves radiation must precede contact variation. |
| D-011 | A neural waveform decoder is report-only and cannot satisfy cooker admission. | A future Accepted ADR changes runtime determinism policy. |
| D-012 | General neural codecs are closed as the next representation family after clean V3B rejection. | New evidence shows explicit modal-fidelity training and a materially different task-specific objective. |

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A bounded task-specific modal/residual record carries held contact sound | AV-MSF/ObjectFolder use global modal quantities and spatial gains; V3B preserves coarse envelope/decay | V1 and V3B lose spectral/modal structure | V4-DEV three-point capacity frontier plus deterministic inverse |
| H2: Geometry-aware exact-object contact learning is possible | REALIMPACT source/mesh/splits repeat; AV-MSF reports few-shot contact fields | No representation has passed a sealed contact gate | V4-HOLDOUT, then one AV-MSF-shaped R3B model |
| H3: Automatic validator reaches useful bounded risk | Hard/acoustic/corpus components and grouped roles exist | No frozen independent release or shadow result exists | R5 after a generator claim exists |
| H4: Cooked coefficients fit product cost | Q30 reference is compact and exact | Whole-mixer/callback/varied-voice cost is unmeasured | Visible consumer plus p95/p99 before promotion |

## Do not retry

- R2 rank/width/epoch/seed/phase or nearby SIREN/harmonic field changes.
- Query-informed normalization, checkpoint selection, stopping or thresholds.
- More nearby modes/DCT bins on opened Blue Bowl row `1807`.
- DAC 44.1 kHz or post-hoc band filtering on opened Large Swan row `1807`.
- Unguarded NDAC, guard repair on Plastic Bin, or another NDAC
  bitrate/postfilter/metric on opened Purple Scoop.
- Another general perceptual codec presented as modal-fidelity evidence.
- Universal `material -> sound` coefficients before exact-object evidence.
- Prompt-to-waveform as the engine path; it may remain authored/report-only.
- Local microphone/hammer capture, raw PhysX-callback mixing or runtime neural
  inference first.

## Required context

Read in precedence order:

1. [Agent routing](../../architecture/agent-routing.md), SPEC-00 and SPEC-01.
2. SPEC-08/24/26/30, ADR-027/046/058/071 and
   [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
3. [Roadmap V6](../../plans/physical-sound-synthesis-roadmap.md),
   [implementation plan](../../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md)
   and [neural strategy](../physical-sound-neural-acoustic-field-strategy-2026-08-30.md).
4. [R2E result](../physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md),
   [R3A V1](../physical-sound-r3a-blue-bowl-representation-gate-2026-08-30.md),
   [R3A V2](../physical-sound-r3a-v2-large-swan-dac-oracle-result-2026-08-30.md)
   and [R3A V3](../physical-sound-r3a-v3-native-ndac-result-2026-08-30.md).
5. [Main product roadmap](../../roadmap.md) for scheduling/promotion facts.

## Handoff

- **Workspace:** Exact V3A invalid and V3B rejected runners/tests are in Git;
  datasets, checkpoints, compressed members, arrays and WAVs remain external.
- **Isolation:** Blue Bowl, Large Swan, Plastic Bin and Purple Scoop development
  contacts are opened. Every row `2407`, method holdout and admission shadow
  remains sealed.
- **Quality:** No neural field, validator release, admitted formula record or
  runtime integration exists. Clip fallback is authoritative.
- **Next commit boundary:** R3A V4-DEV corpus manifest, bounded modal/residual
  capacity frontier, exact deterministic inverse, focused tests and repeated
  opened-development report. Do not download a new target unless it passes.
