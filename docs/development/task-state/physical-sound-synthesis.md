# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V7 / R3A_V5C_6KBPS_QUANTIZED_GATE_PASSED / CAPACITY_FRONTIER_NEXT / LONG_RUN_DISABLED / FALLBACK_REQUIRED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-31` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed external neural contact-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, bakes an exact bounded clip atlas or later cooked coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Staged V5-C `6 kbps` is the first fully quantized
  checkpoint to pass the internal anti-collapse gate. Step `4000` keeps RMS
  ratio `0.191`, spectrum improvement `5.47%`, at least `51` codes per stage
  and diversity `1.99` with stable latent. Full-loss optimization is rejected;
  that loss remains evaluation and checkpoint-selection only.
- **Exact evidence:** [V5-B collapse and V5-C bootstrap](../physical-sound-r3a-v5b-collapse-and-v5c-bootstrap-2026-08-31.md),
  quantized-pass report `047cec3f…229a`, full-loss rejection
  `df805d76…32b9` and stable refinement report `8a41950b…9a60`.
- **Next action:** Run the same bounded step-4000 frontier for `12/24 kbps`,
  compare frozen full validation loss, then choose whether one capacity merits
  a separately authorized longer run.
- **Spend rule:** Development, new objects, row `2407`, method holdout and
  admission shadow remain unread throughout all three capacity runs.
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
| R3A V5 | `V5C 6KBPS QUANTIZED GATE PASSED / FRONTIER OPEN` | Staged bootstrap and frozen RVQ ramp pass internal checks; compare 12/24 kbps before any long run or development read. |
| R3B+ | `BLOCKED` | No contact-to-latent field, validator release, baked contact atlas, admitted domain or runtime promotion exists. |

## Material transition: analytical fit rejected and Roadmap V7

- **Observation:** Two V4 preflights and fit runs reproduce every manifest,
  report, pole bank, basis and bin array. `compact`, `balanced` and `extended`
  pass `4 MiB/64 KiB` budgets but fail spectrum on all twelve fit contacts.
- **Evidence:** Implementation `e05d5593`, fit report `f2127117…9e0` and tree
  `fedfc005…849d`; four development contacts and every row `2407` remain unread.
- **Conclusion:** The tested analytical factorization is insufficient before
  interpolation is even considered. More nearby pole/PCA/bin capacity is not
  an evidence-backed next move.
- **Decision:** Rebaseline to [Roadmap V7](../../plans/physical-sound-synthesis-roadmap.md):
  learn task-specific neural rate allocation first, then a contact-to-latent
  field, and bake validated outputs into ordinary clip assets for the first
  deterministic product experiment.
- **Rejected alternatives:** Evaluate failed fit on development, add capacity,
  weaken endpoints, open a new object, retry a universal codec or add a runtime
  neural decoder.
- **Remaining uncertainty:** A small RVQ autoencoder trained on published
  impact audio with the exact spectral/modal objective may beat universal
  codecs and the failed analytical basis within the record budget.
- **Reconsideration condition:** V5 development and one frozen source-disjoint
  holdout pass; otherwise retain authored clips and stop the model family.

## Material transition: V5 preflight A complete

- **Observation:** Two final V5 runs reproduce canonical manifest/report bytes;
  the 73-clip Heller inventory splits into 56 train and 17 internal-validation
  clips by immutable source groups.
- **Evidence:** Implementation `512b35dd`, manifest `1cc23496…fb31`, report
  `38176a41…3bf3`; the exact full-model repeat passes and the micro-overfit
  ratio is `0.1499882595` against a frozen maximum `0.45`.
- **Conclusion:** The architecture can be constructed deterministically and a
  small RVQ control can learn. This does not establish full-model training,
  codebook health, checkpoint reload or acoustic quality.
- **Decision:** Advance only to an external GPU environment and runner-control
  freeze. Keep `neural_training_authorized=false` until those controls pass.
- **Rejected alternatives:** Start a long run from the CPU environment, treat
  the corpus inventory as decoded training data, open development, add a
  fourth capacity or introduce runtime inference.
- **Remaining uncertainty:** The full loss may be finite and trainable yet fail
  codebook use, checkpoint reproducibility, capacity or the frozen acoustic
  endpoints.
- **Reconsideration condition:** Runner controls fail twice or require changing
  the already-frozen architecture/loss/capacity identity.

## Material transition: V5 training runner controls complete

- **Observation:** Two CUDA runs reproduce manifest, report and 87,140,277-byte
  checkpoint exactly. L1 improves `24.0%`, train stages use `[8,7,6,7]` codes
  and checkpoint continuation reproduces state, metrics, output and codes.
- **Evidence:** Implementation `90984de2`, manifest `75ed5e06…2679`, report
  `a4e30d87…b1e1`, checkpoint `74fccfac…7659`, tree `607581c4…40df`.
- **Conclusion:** The frozen full-size representation runner is finite,
  short-horizon trainable and resumable on the declared RTX 3080 environment.
  This is training-substrate evidence only.
- **Decision:** Authorize exactly the three frozen capacity runs without
  development reads. Internet internal validation may select checkpoints but
  may not change architecture, loss, capacity, seed or thresholds.
- **Rejected alternative:** Original tiny-uniform embeddings used one code in
  every stage. Keep deterministic first-train-latent residual-share warm start
  and never accept reconstruction improvement with dead codebooks.
- **Remaining uncertainty:** Any or all capacities may fail to learn useful
  impact reconstruction or may later fail the four development contacts.
- **Reconsideration condition:** Two coherent training remediations fail the
  frozen internal-validation criterion; then stop and run bounded research
  instead of tuning development.

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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A task-specific neural latent carries held impact sound within budget | V5-C `6 kbps` passes the fully quantized internal anti-collapse gate | Universal NDAC and V4 fail; no development or holdout pass exists | Bounded V5-C `12/24 kbps` frontier, then one frozen development evaluation |
| H2: Geometry-aware exact-object contact learning is possible | REALIMPACT source/mesh/splits repeat; AV-MSF reports few-shot contact fields | No representation has passed a sealed contact gate | V5-HOLDOUT, then one contact-to-latent field |
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
- Universal `material -> sound` coefficients before exact-object evidence.
- Prompt-to-waveform as the engine path; it may remain authored/report-only.
- Local microphone/hammer capture, raw PhysX-callback mixing or runtime neural
  inference first.

## Required context

Read in precedence order:

1. [Agent routing](../../architecture/agent-routing.md), SPEC-00 and SPEC-01.
2. SPEC-08/24/26/30, ADR-027/046/058/071 and
   [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
3. [Roadmap V7](../../plans/physical-sound-synthesis-roadmap.md),
   [implementation plan](../../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md)
   and [neural strategy](../physical-sound-neural-acoustic-field-strategy-2026-08-30.md).
4. [R2E result](../physical-sound-listener-field-r2e-result-and-v4-research-2026-08-30.md),
   [R3A V1](../physical-sound-r3a-blue-bowl-representation-gate-2026-08-30.md),
   [R3A V2](../physical-sound-r3a-v2-large-swan-dac-oracle-result-2026-08-30.md)
   [R3A V3](../physical-sound-r3a-v3-native-ndac-result-2026-08-30.md),
   [R3A V4](../physical-sound-r3a-v4-fit-probe-and-neural-rebaseline-2026-08-31.md),
   [R3A V5 preflight A](../physical-sound-r3a-v5-neural-preflight-a-2026-08-31.md),
   [R3A V5 training runner preflight](../physical-sound-r3a-v5-training-runner-preflight-2026-08-31.md)
   and [V5-B/V5-C bounded research](../physical-sound-r3a-v5b-collapse-and-v5c-bootstrap-2026-08-31.md).
5. [Main product roadmap](../../roadmap.md) for scheduling/promotion facts.

## Handoff

- **Workspace:** Exact V3A/V3B, V4 rejection and V5 preflight/training-control
  runners/tests are in Git; datasets, checkpoints, compressed members, arrays
  and WAVs remain external.
- **Isolation:** Blue Bowl, Large Swan, Plastic Bin and Purple Scoop development
  contacts are opened. Every row `2407`, method holdout and admission shadow
  remains sealed.
- **Quality:** No neural representation, field, validator release, baked atlas,
  admitted formula record or runtime integration exists. Clip fallback is
  authoritative.
- **Next commit boundary:** External V5-C `12/24 kbps` step-4000
  discriminators and a frozen capacity comparison, with zero development and
  sealed reads.
