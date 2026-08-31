# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V8 / R3A_V8_SYNTHETIC_PREFLIGHT_NEXT / R3B_NOT_AUTHORIZED / FALLBACK_REQUIRED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-31` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed external neural contact-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, bakes an exact bounded clip atlas or later cooked coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** V5 waveform-RVQ is rejected, not ML as a whole. V8
  freezes a materially different explicit-modal representation: global
  frequency/damping, neural contact-to-mode-shape/gain field and bounded
  residual/excitation components.
- **Exact evidence:** [V5-C rejection](../physical-sound-r3a-v5c-capacity-frontier-and-development-result-2026-08-31.md)
  plus the [V8 research decision and frozen preflight](../physical-sound-v8-explicit-modal-neural-rebaseline-2026-08-31.md).
- **Next action:** Implement and repeat V8-SYNTH on the two opened exact-hash
  NISR FEM label files. Do not select or read any fresh real V8 object yet.
- **Spend rule:** Every row `2407`, method holdout and admission shadow remain
  sealed. V5 did not authorize a new representation holdout.
- **Deployment rule:** The first neural success may bake an ordinary bounded
  contact clip atlas offline. Runtime neural inference remains forbidden;
  deterministic modal distillation becomes optional later optimization.
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
| R3A V4 | `REJECTED_BEFORE_DEVELOPMENT / REPRODUCIBLE` | Three bounded modal/residual capacities fail spectrum on every fit contact; development and sealed rows stay unread. |
| R3A V5 | `REJECTED_ON_DEVELOPMENT / REPRODUCIBLE` | All capacities fit the byte budget but fail real spectrum/modal transfer; no capacity or holdout is selected. |
| R3A V8 | `PREREGISTERED / SYNTHETIC_PREFLIGHT_NEXT` | Opened FEM truth may test renderer recovery and held mode-shape learning only; no real quality credit. |
| R3B+ | `NOT_AUTHORIZED` | No real contact-to-modal field, validator release, baked contact atlas, admitted domain or runtime promotion exists. |

## Compressed V4–V5 transition history

- V4 fit-only modal/residual capacities repeat and meet cost budgets but fail
  spectrum on all twelve fit contacts; development was not read.
- V5 preflight and CUDA runner controls repeat, learn on their internal task,
  keep active RVQ codes and reload checkpoints exactly. They prove a working
  substrate, not physical transfer.
- V5-C then passes anti-collapse/cost gates but fails fresh real development.
  Exact implementation, metrics and rejected alternatives remain in the linked
  V4/V5 evidence reports; none may be retuned on opened objects.

## Material transition: V5-B collapse isolated and V5-C bootstrap selected

- **Observation:** The automated step-2000 gate rejects factorized V5-B on
  amplitude, spectrum, code use and output diversity. Removing RVQ produces
  the same near-silent constant output, while normalized continuous bootstrap
  learns distinct validation waveforms.
- **Evidence:** Implementation `ed5e65ca`; [bounded research report](../physical-sound-r3a-v5b-collapse-and-v5c-bootstrap-2026-08-31.md).
- **Conclusion:** The primary defect is optimizing the full clamped perceptual
  loss from near silence, not encoder explosion or RVQ capacity alone.
- **Decision:** Supersede end-to-end-from-step-zero V5-B with staged V5-C.
  Keep the full perceptual loss evaluation-only, disable implicit 50k runs and
  compare bounded capacities. Development and holdout remain forbidden.
- **Reconsideration condition:** A fully quantized V5-C checkpoint must retain
  bootstrap signal and pass every frozen anti-collapse check; otherwise stop
  this codec family before development.

## Material transition: V5-C development rejects the neural representation

- **Observation:** Paired `6/12/24 kbps` runs pass internal anti-collapse and
  record-cost checks, but all three fail every real development object.
- **Evidence:** [Exact result](../physical-sound-r3a-v5c-capacity-frontier-and-development-result-2026-08-31.md),
  repeated development report `74a6f4ea…bbb4`; spectrum and modal frequency
  fail `12/12`, decay fails `11/12`, sealed/method/shadow reads remain zero.
- **Conclusion:** More bitrate or longer training in this V5-C family is not
  evidence-backed. Internal validation did not predict physical transfer.
- **Decision:** Close R3A V5 as `REJECT_NEURAL_REPRESENTATION`; authorize no
  V5 holdout, R3B field, baked atlas or runtime promotion.
- **Rejected alternatives:** Threshold/loss/postfilter tuning on opened rows,
  selecting 24 kbps from internal loss, or continuing any rejected checkpoint.
- **Reconsideration condition:** A materially different preregistered modal/
  decay-preserving representation and new source-disjoint development data.

## Material transition: V8 explicit-modal neural rebaseline

- **Observation:** Primary prior art converges on explicit damped resonances,
  contact-dependent modal gains and a bounded residual. NISR publishes exact
  FEM frequencies and boundary mode shapes in small individually hashable
  files; ObjectFolder Real can later supply fresh real impact development.
- **Evidence:** [V8 research decision](../physical-sound-v8-explicit-modal-neural-rebaseline-2026-08-31.md),
  including exact source revisions, competing hypotheses, dataset roles and
  frozen synthetic gates.
- **Conclusion:** V5 failure justifies no ML-wide rejection. It does justify
  ending opaque waveform compression and testing modal identity as an explicit
  inductive bias before another real run.
- **Decision:** Roadmap V8 starts with deterministic modal recovery plus a
  neural coordinate-to-3D-mode-shape field on two opened NISR Glass objects.
  A pass authorizes only a fresh real protocol; R3B remains closed.
- **Rejected alternatives:** Longer V5, another bitrate, tuning against the
  four opened objects, treating synthetic audio as realism evidence, or loading
  a neural model in runtime.
- **Reconsideration condition:** If V8-SYNTH passes and repeats, freeze a new
  ObjectFolder Real fit/development revision before reading its waveforms. If
  it fails, diagnose the renderer/field rather than opening real data.

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
| D-013 | First neural deployment target is an offline-baked contact clip atlas, not runtime inference and not mandatory modal distillation. | A later measured consumer proves that bounded clip assets cannot meet variation/cost needs. |
| D-014 | V8 learns an explicit contact-to-mode-shape/gain field around global frequency/damping; synthetic FEM is substrate evidence only. | Fresh real development and holdout reject this factorization or require a preregistered spatial-damping/residual successor. |

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Explicit modal identity plus a neural spatial field carries held impact sound | Differentiable modal prior art and exact NISR field labels; V5 failure clusters on spectrum/modes | V4 fixed modal factorization failed fit spectrum; no V8 real result exists | V8-SYNTH, then fresh ObjectFolder Real fit/development |
| H2: Geometry-aware exact-object contact learning is possible | REALIMPACT/ObjectFolder geometry and AV-MSF few-shot evidence | No representation has passed fresh real development/holdout | Blocked until V8 independently passes R3A |
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
- More V4 poles, PCA ranks, spectral bins or development evaluation after the
  fit-only rejection.
- Original tiny-uniform V5 codebook initialization; it uses one code per active
  stage in the frozen full-model smoke despite improving reconstruction L1.
- Full perceptual-loss training from a near-silent decoder, with or without
  factorized RVQ; both collapse on internal validation by step `2000`.
- Lowering the anti-collapse thresholds or resuming a rejected V5-A/V5-B
  checkpoint.
- Longer V5-C training, a fourth bitrate, or evaluator/loss/gain/postfilter
  tuning after the four V5 development contacts were opened.
- Universal `material -> sound` coefficients before exact-object evidence.
- Prompt-to-waveform as the engine path; it may remain authored/report-only.
- Local microphone/hammer capture, raw PhysX-callback mixing or runtime neural
  inference first.

## Required context

Read in precedence order:

1. [Agent routing](../../architecture/agent-routing.md), SPEC-00 and SPEC-01.
2. SPEC-08/24/26/30, ADR-027/046/058/071 and
   [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
3. [Roadmap V8](../../plans/physical-sound-synthesis-roadmap.md),
   [implementation plan](../../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md)
   and [neural strategy](../physical-sound-neural-acoustic-field-strategy-2026-08-30.md).
4. [R2E result](../physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md),
   [R3A V1](../physical-sound-r3a-blue-bowl-representation-gate-2026-08-30.md),
   [R3A V2](../physical-sound-r3a-v2-large-swan-dac-oracle-result-2026-08-30.md)
   [R3A V3](../physical-sound-r3a-v3-native-ndac-result-2026-08-30.md),
   [R3A V4](../physical-sound-r3a-v4-fit-probe-and-neural-rebaseline-2026-08-31.md),
   [R3A V5 preflight A](../physical-sound-r3a-v5-neural-preflight-a-2026-08-31.md),
   [R3A V5 training runner preflight](../physical-sound-r3a-v5-training-runner-preflight-2026-08-31.md),
   [V5-B/V5-C bounded research](../physical-sound-r3a-v5b-collapse-and-v5c-bootstrap-2026-08-31.md)
   [V5-C development result](../physical-sound-r3a-v5c-capacity-frontier-and-development-result-2026-08-31.md)
   and [V8 rebaseline](../physical-sound-v8-explicit-modal-neural-rebaseline-2026-08-31.md).
5. [Main product roadmap](../../roadmap.md) for scheduling/promotion facts.

## Handoff

- **Workspace:** Exact V3A/V3B, V4 rejection and V5 runners/tests are in Git;
  V8 protocol is frozen but its runner is the next commit. Datasets,
  checkpoints, arrays and WAVs remain external.
- **Isolation:** Blue Bowl, Large Swan, Plastic Bin and Purple Scoop development
  contacts are opened. Every row `2407`, method holdout and admission shadow
  remains sealed.
- **Quality:** No neural representation, field, validator release, baked atlas,
  admitted formula record or runtime integration exists. Clip fallback is
  authoritative.
- **Next commit boundary:** V8-SYNTH runner, focused tests and repeated external
  report. It must read only the two opened NISR label files; authored clips
  remain the product path.
