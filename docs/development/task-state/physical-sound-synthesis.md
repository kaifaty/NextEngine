# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `ROADMAP_V3 / NEURAL_TRANSFER_FIELD / R2C_REJECTED / R2D_V1_FIXED_STEP_REJECTED / R2D_V2_DECAY_NEXT / SHADOW_SEALED / FALLBACK_REQUIRED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-30` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture, external neural acoustic-field research, deterministic cooker boundary and independent automatic validation |
| Definition of done | A frozen offline model beats honest controls on held-out physical axes, cooks to exact bounded coefficients and is admitted only by an independent selective validator with automatic clip fallback |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** Keep the offline neural route, but retire the joint
  separable complex SIREN plus sampled equal-L1 objective. It collapses toward
  silence before listener generalization.
- **Exact evidence:** [R2C result and failure diagnostic](../physical-sound-listener-field-r2c-result-2026-08-30.md).
  Data-only and Helmholtz runs repeat byte-identically, select no candidate and
  produce about `53 dB` mean level error. Diagnostic A/B hash to
  `1263e02f…a8f`.
- **Why this is not a capacity verdict:** A context-only rank-96 oracle retains
  `99.6396%` total energy with Frobenius NRMSE `0.0600`; the trained data-only
  objective is nevertheless `1.0498x` the zero predictor and all logged steps
  reach gradient clipping.
- **Next action:** Freeze N0.3D V2 with the V1 basis, objective, tasks, steps,
  cooker and thresholds unchanged; replace only fixed learning rate with one
  deterministic decay schedule ending near zero.
- **After that:** Only a passing N0.3D may authorize one N0.3E frozen
  coordinate-to-low-rank-coefficient model and one grouped query evaluation.
- **Current blockers:** learned quality, multi-impact internet coverage,
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
| R2D trainability gate | `V1_REJECTED / V2_DECAY_NEXT` | V1 passes every gate except small-block/full-context log energy; query remains forbidden. |
| R2E low-rank coefficient field | `BLOCKED_BY_R2D` | One data-only neural spatial field after a passing context gate. |
| R3+ exact-object/validator/admission | `BLOCKED` | No model, validator release, admitted domain or runtime promotion exists. |

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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: Energy-preserving context training is sufficient to remove silence collapse | V1 beats zero/mean and approaches oracle through the cooker | Fixed step misses small/full log-energy gate | V2 deterministic learning-rate decay with all other inputs frozen |
| H2: Frozen low-rank basis plus learned spatial coefficients can beat interpolation | Context energy is strongly low-rank; joint basis learning is unnecessary for the first test | No coordinate-to-coefficient candidate has been measured | One N0.3E data-only candidate after N0.3D passes |
| H3: Exact-object impact/listener learning is possible from published data | REALIMPACT exposes multiple vertices/listeners and force metadata | Current bounded claim uses one fixed impact; excitation alignment across impacts is unproven | New hash-closed multi-impact projection only after R2E |
| H4: Automatic validator can reach useful coverage at bounded false-pass risk | Hard/acoustic/corpus components and grouped roles exist | No frozen independent release or shadow result exists | R5 calibration/holdout release after a generator claim exists |
| H5: Cooked neural coefficients fit a useful production budget | Q30 lab reference is compact and exact | Whole-mixer, callback and varied-voice cost are unmeasured | Player-visible consumer plus whole-mixer p95/p99 before promotion |

## Do not retry

- R2 direct/phase ranks, width, epochs, seed, phase speed or opened thresholds.
- R2C separable SIREN with nearby size/step/rank/seed/Helmholtz-weight changes.
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
4. [Roadmap V3](../../plans/physical-sound-synthesis-roadmap.md),
   [implementation plan](../../plans/2026-08-30-physical-sound-neural-acoustic-field-implementation-plan.md)
   and [neural strategy](../physical-sound-neural-acoustic-field-strategy-2026-08-30.md).
5. [R0–R1 boundary](../physical-sound-neural-real-boundary-r0-r1-2026-08-30.md),
   [R2 phase failure](../physical-sound-listener-field-r2-phase-research-2026-08-30.md),
   [R2B preflight](../physical-sound-r2b-dense-complex-field-preflight-2026-08-30.md)
   and [R2C result](../physical-sound-listener-field-r2c-result-2026-08-30.md).
6. [Main product roadmap](../../roadmap.md) for scheduling or promotion facts.

## Handoff

- **Workspace:** R2C tooling and tests are implemented; all heavy artifacts
  remain external. Canonical docs now point to Roadmap V3 and N0.3D next.
- **Quality:** No neural candidate, validator release, admitted formula record
  or runtime integration exists. Clip fallback is still authoritative.
- **Isolation:** R2C training reads zero query audio; failure diagnostic reads
  zero query audio; method holdout and admission shadow remain unopened.
- **Next commit boundary:** N0.3D V2 decayed-step runner and immutable repeated
  decision; no grouped query candidate in that boundary.
