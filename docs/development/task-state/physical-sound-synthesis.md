# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V11 / A1R_FORCE_ONSET_PASS / V9_REAL_FIT_REJECTED / FRF_KNOWN_TRUTH_NEXT / REAL_QUALITY_NOT_PROVEN / FALLBACK_REQUIRED / P1_BLOCKED` |
| Updated | `2026-08-31` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed external neural contact-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, bakes an exact bounded clip atlas or later cooked coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** V9-SYNTH passes every frozen known-truth, exact-repeat,
  field and cost gate. Neural query waveform NRMSE is `0.472x` nearest; the
  shared boundary/contact record cost `25,888/104` bytes.
- **Exact evidence:** [V9 synthetic result](../physical-sound-r3a-v9-time-varying-residual-synthetic-result-2026-08-31.md),
  manifest `15616a58…36c9`, report `6df03217…779a`.
- **A0 evidence:** [Beer Glass/Rinsing Cup source freeze](../physical-sound-r3a-v10-objectfolder-real-source-and-role-freeze-2026-08-31.md),
  repeated manifest `8a30cef0…8728`, report `3522c677…00d3`.
- **A1 evidence:** [repeat-exact Beer Glass rejection](../physical-sound-r3a-v10-beer-glass-real-fit-result-2026-08-31.md),
  report `7d7bb630…9db9b`; contacts `18/29` fail the frozen onset gate before fit.
- **A1R evidence:** [repeat-exact fit result and V11 research](../physical-sound-r3a-v10-a1r-force-onset-fit-result-and-v11-research-2026-08-31.md),
  report `5c9e87e…c2ca`; force onset passes, every real V9 fit contact fails.
- **Next action:** implement the V11-B1 synthetic force→response oracle and
  reject ill-conditioned transfer estimates before opening fresh real data.
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
| R3A V8 | `REAL_FIT_REJECTED / REPRODUCIBLE` | Fit contacts `18/12/4` repeat exactly; both bounded capacities fail spectrum/modal identity, while development `20` and sealed `27` remain unread. |
| R3A V9 | `CLOSED / REAL_FIT_REJECTED / REPRODUCIBLE` | Force onset is valid; all four object-51 contacts fail spectrum/modal identity under unchanged gates. |
| R3B+ | `NOT_AUTHORIZED` | No real contact-to-modal field, validator release, baked contact atlas, admitted domain or runtime promotion exists. |

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

## Material transition: V8 real fit rejects the residual representation

- **Observation:** Two runs repeat every artifact. Global and per-contact
  damping records cost `63,392/63,520` bytes, yet spectrum and modal endpoints
  fail on every fit contact; envelope passes everywhere.
- **Evidence:** [Exact result and bounded research](../physical-sound-r3a-v8-object91-fit-result-and-v9-residual-research-2026-08-31.md),
  report `9c49abc8…d438`; development/sealed decoded counters are zero.
- **Conclusion:** sparse global complex bins are the causal bottleneck. A dense
  phase-preserving residual recovers spectrum but costs `214,564` bytes; global
  magnitude-shaped noise remains inadequate. Damping is not the main cause.
- **Decision:** close V8 before development. Start V9 with explicit modes plus
  a compact neural time-varying filterbank residual on synthetic known truth,
  then require entirely source-disjoint real roles.
- **Rejected alternatives:** another V8 bin/damping capacity, threshold repair,
  stationary random-phase noise, or using object `91` to select V9.
- **Evidence correction:** the frozen official page maps object `91` to
  `Solid_Spoon / Steel`, not Glass. The byte-level negative remains valid, but
  it supplies no Glass-material evidence.

## Material transition: V9-SYNTH passes

- **Observation:** two complete runs repeat manifest/report bytes and pass all
  frozen gates; neural latent/waveform errors are `0.521x/0.472x` nearest.
- **Evidence:** [Exact result](../physical-sound-r3a-v9-time-varying-residual-synthetic-result-2026-08-31.md),
  report `6df03217…779a`; every protected/real counter is zero.
- **Decision:** authorize only source research and a new hash-closed real
  protocol with exact contact coordinates. No real quality or R3B credit.

## Material transition: A0 real source and roles pass

- **Observation:** official benchmark code and data bind the same
  `(object, contact)` key to raw PCM, published coordinates and point cloud.
  Beer Glass `60` and Rinsing Cup `22` each expose IDs `0…29`; a known object-91
  processed WAV exactly matches its raw microphone hash.
- **Evidence:** [A0 source freeze](../physical-sound-r3a-v10-objectfolder-real-source-and-role-freeze-2026-08-31.md),
  manifest `8a30cef0…8728`, report `3522c677…00d3`; two runs are byte-identical
  and every waveform decode counter is zero.
- **Decision:** close A0 and authorize only the 22 object-60 fit WAVs under the
  [frozen fit protocol](../physical-sound-r3a-v10-beer-glass-real-fit-protocol-2026-08-31.md).
  Development/query/object-22 holdout stay closed; missing force/normal/
  listener axes narrow the claim.

## Material transition: A1 rejects at source synchronization

- **Observation:** contacts `18/29` have baseline-noise thresholds greater than
  their full-waveform peaks, although both maxima occur near sample `48,000`.
- **Evidence:** [A1 exact result](../physical-sound-r3a-v10-beer-glass-real-fit-result-2026-08-31.md),
  repeated report `7d7bb630…9db9b`; fit decoded `6,336,000`, protected decoded `0`.
- **Conclusion:** the generic onset rule is incompatible with this compact
  source. V9 quality remains unknown because representation fitting never ran.
- **Decision:** do not repair the opened threshold/subset. A1R requires a new
  source revision with published event time or force-derived onset.

## Material transition: A1R isolates representation failure

- **Observation:** measured force finds the event at `47,999/48,000`; every
  infrastructure gate passes, but all four V9 fit contacts fail spectrum and
  modal-frequency endpoints.
- **Evidence:** [exact result/research](../physical-sound-r3a-v10-a1r-force-onset-fit-result-and-v11-research-2026-08-31.md),
  repeated manifest/model/report `a50e4ade…ca3`/`e64ec94d…42d`/`5c9e87e…c2ca`.
- **Decision:** close V9. V11 first separates measured force from the physical
  transfer response on synthetic known truth; object `51` is diagnostic-only.
- **Limit:** development `9`, sealed `18` and every admission shadow stay closed.

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
| D-014 | Explicit contact-to-mode-shape/gain learning remains around global frequency/damping; synthetic FEM is substrate evidence only. | Fresh source-disjoint real development and holdout reject the factorization. |
| D-015 | V9 time-varying residual is retired after repeat-exact real-fit rejection. | A materially different fresh-source representation, not nearby capacity tuning. |
| D-016 | V11 separates force from transfer response; ML predicts constrained modal residues/radiation or solver proposals, not an unconstrained runtime waveform. | Known-truth and fresh real evidence reject this factorization. |

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Force-normalized transfer identification recovers stable poles | RealImpact uses force deconvolution before modal fitting; A1R force is synchronized | No NextEngine known-truth or real FRF pass exists | V11-B1 synthetic oracle |
| H2: Geometry-aware ML predicts contact modal residues | Coordinates/geometry and physically constrained outputs exist | No fresh real representation/development pass exists | V11-B4 after B3 |
| H3: Automatic validator reaches useful bounded risk | Hard/acoustic/corpus components and grouped roles exist | No frozen independent release or shadow result exists | V11-B6 after a generator claim |
| H4: Baked atlas meets product cost | Offline clips preserve exact output and fallback | Whole-mixer/voice cost is unmeasured | V11-B5 then visible consumer |

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
- More V8 residual bins, another damping capacity, threshold repair or
  stationary random-phase magnitude noise on object `91`.
- Lowering/replacing the V10 Beer Glass onset threshold, selecting only the
  twenty passing contacts or exploiting the observed one-second maxima.
- More V9 modes/bands/DCT coefficients, threshold/loss/postfilter tuning or
  development reads on opened object `51` contacts.
- Universal `material -> sound` coefficients before exact-object evidence.
- Prompt-to-waveform as the engine path; it may remain authored/report-only.
- Local microphone/hammer capture, raw PhysX-callback mixing or runtime neural
  inference first.

## Required context

Read in precedence order:

1. [Agent routing](../../architecture/agent-routing.md), SPEC-00 and SPEC-01.
2. SPEC-08/24/26/30, ADR-027/046/058/071 and
   [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
3. [Roadmap V11](../../plans/physical-sound-synthesis-roadmap-v11.md) and
   [A1R exact result/V11 research](../physical-sound-r3a-v10-a1r-force-onset-fit-result-and-v11-research-2026-08-31.md).
4. [V9 synthetic result](../physical-sound-r3a-v9-time-varying-residual-synthetic-result-2026-08-31.md),
   [A0 source freeze](../physical-sound-r3a-v10-objectfolder-real-source-and-role-freeze-2026-08-31.md),
   [A1 rejection](../physical-sound-r3a-v10-beer-glass-real-fit-result-2026-08-31.md),
   [A1R source freeze](../physical-sound-r3a-v10-a1r-object51-force-source-freeze-2026-08-31.md)
   and [A1R fit protocol](../physical-sound-r3a-v10-a1r-object51-force-onset-fit-protocol-2026-08-31.md).
5. [Main product roadmap](../../roadmap.md) for scheduling/promotion facts.

## Handoff

- **Workspace:** V9 synthetic plus V10 A0/A1/A1R code and evidence are in Git;
  all run outputs, datasets, arrays and WAVs are external.
- **Isolation:** Blue Bowl, Large Swan, Plastic Bin and Purple Scoop development
  contacts are opened. Every row `2407`, method holdout and admission shadow
  remains sealed.
- **Quality:** No neural representation, field, validator release, baked atlas,
  admitted formula record or runtime integration exists. Clip fallback is
  authoritative.
- **Next commit boundary:** V11-B1 known-truth force→response protocol and
  synthetic oracle. Authored clips remain the product path.
