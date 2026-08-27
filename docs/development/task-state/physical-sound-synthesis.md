# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `P0_CONTROLLED_MUTATIONS_AND_GROUPED_RISK_MEASURED / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-27` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture plus isolated fixed-point impact/demo and external controlled-corpus experiments |
| Definition of done | Demonstrate an external selective validator whose automatic pass has measured bounded false-pass risk on grouped real/mutation holdouts, with automatic clip fallback for OOD and no per-sound human gate |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** AV-P0C rejects stationary/frozen failures, but misses
  shuffled wood envelopes and rejects most real objects; `Pass` stays disabled.
- **Why:** Holdout/shadow false-pass `1/3` groups with `1/3` and `0/3` real
  coverage; the 95% Wilson upper false-pass risk is `0.7923`.
- **Next action:** Keep pack/splits frozen, add amplitude-envelope trajectory
  features, then broaden real object families before new source work.
- **Current blocker:** Unsafe risk/coverage, only three parent groups per split,
  spatial/force evidence, mixer budget, Accepted ADR and complete contact signal.
- **Do not retry:** Treating synthetic-target match as glass identity, blind preset tuning, or using FAD, CLAP, ViSQOL, an aesthetic
  model or a general audio model as the sole quality judge. Also retain the ban
  on universal material sound and raw PhysX-callback mixing.
- **Reconsider when:** P0 produces a measured quality/cost point and a concrete
  production impact consumer is selected.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Research report](../physical-sound-synthesis-research-2026-08-26.md) and [DiffSound trial](../physical-sound-diffsound-trial-2026-08-26.md) | `GLASS_09_Q30_DEMO_PASS / PHYSICAL_ID_OPEN` | Q30 tracks aligned f64 at `115.43 dB` SNR; product-owner A/B accepted B. Demo post-scale stays within one S16 LSB. Wrong geometry keeps material and wall thickness non-physical. |
| [Quality research](../physical-sound-quality-evaluation-research-2026-08-26.md), [automated validation](../physical-sound-automated-validation-research-2026-08-27.md), [AV-P0A](../physical-sound-validator-av-p0a-2026-08-27.md) and [AV-P0B](../physical-sound-corpus-benchmark-av-p0b-2026-08-27.md) | `AV_P0B_REAL_MATERIAL_BENCHMARK_MEASURED / NO_ACCEPTANCE_AUTHORITY` | External corpus: 15 real objects, 30 published variants and 13 engine candidates. BEATs real identity is `15/15`; engine shadow is `8/13`. Manifest/report hashes are frozen; recordings/models remain external and non-distributable. |
| [Automated steel search](../physical-sound-steel-automated-search-2026-08-27.md) | `BEATS_3_OF_3 / PANNS_0_OF_3 / FALLBACK_OUT_OF_DOMAIN` | Sparse tuning failed `0/165`; critical-band roughness yielded a BEATs-positive profile, but independent PANNs rejects it. No steel/demo promotion; v3 report/WAV repeat is byte-identical. |
| [Steel residual v4](../physical-sound-steel-residual-v4-2026-08-27.md) | `V4_REJECTED / FALLBACK_OUT_OF_DOMAIN` | YCB adds aluminium-container and steel-skillet families. Real metal flatness is about `−16/−17 dB` versus v3 `−53 dB`; v4 reaches flatness but not real spectral dynamics. Original PANNs stays `0/39`, BEATs `7/39`; no joint profile or promotion. |
| [Controlled glass corpus](../physical-sound-controlled-glass-corpus-2026-08-27.md) | `CONTROLLED_SYNTHETIC_CORPUS_PASS / HUMAN_REFERENCE_OPEN` | Exact geometry and 15 force/position conditions are reproducible; Q30 and physical controls pass, but whole-vector IDW is an inadequate spatial model and the corpus has no real matched recording. |
| [Steel calibration](../physical-sound-steel-calibration-2026-08-26.md) and [wood/glass calibration](../physical-sound-wood-glass-calibration-2026-08-26.md) | `WOOD-B_ACCEPTED / GLASS-D-F_REJECTED / GLASS-G_PARTIAL_ACCEPT / GLASS-H_WEAK_PREFERENCE` | Keep H as provisional baseline and G as its close control; stop near-neighbor tuning. |
| [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), [AV-P0C](../physical-sound-validator-av-p0c-2026-08-27.md) and [implementation plan](../../plans/2026-08-27-physical-sound-domain-admission-implementation-plan.md) | `CONTROLLED_MUTATIONS_MEASURED / SELECTIVE_PASS_DISABLED` | Registry V1 forbids forged `Pass`; 36 controlled negatives are hash-closed. Provisional threshold `0.404906` yields holdout/shadow false pass `1/3`, Wilson upper `0.7923`, so clip fallback remains mandatory. |
| [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md) and current `AudioSceneSnapshotV1`/`AudioMixerV1` | `CURRENT_BASELINE_OBSERVED` | Clip playback, canonical PCM and gameplay/output separation remain the promoted baseline; the physical source synth is isolated experimental code. |
| [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) versus current Rust `ContactEventV1` | `IMPLEMENTATION_GAP_OBSERVED` | Normative contact facts include velocity/impulse/effective mass/tags, but current record omits them; production audio must close the existing projection rather than consume raw callbacks. |
| `xtask physical-sound-lab` external audition and cost report | `PASS / NON_GATING_COST` | Frozen baselines remain exact; selected Q30 WAV SHA is `c912806c…b9c823`. On Ryzen 3950X, 16 voices cost `1.483/1.683 ms` p50/p99 per 1,600-frame lab tick, `5.05%` of that window; this is not a whole-engine budget. |
| Glass-object set and DiffSound audition | `DIFFSOUND_AND_Q30_PERCEPTUAL_ACCEPT / UNCALIBRATED` | Selected `09` and transferred Q30 B sound glass-like; identity and reference fidelity remain separate, and one archetype is not a generic glass model. |
| Product-owner audition, 2026-08-26 | `PERCEPTUAL_FAIL` | The current output only remotely resembles the target; engineering checks cannot support an acoustic-quality claim. |
| Product-owner wood/glass audition, 2026-08-26 | `WOOD-B ACCEPT / GLASS-D-F FAIL / G PARTIAL_ACCEPT / H WEAK_PREFERENCE` | H is tentatively better but hard to distinguish from G; preserve both and require a stronger discriminator. |
| `xtask physical-sound-eval` Q0 run | `PASS / UNCALIBRATED` | Nine hash-frozen WAVs produce bounded signal, multiresolution spectrum, modal-assignment and per-band decay reports under evaluator profile hash `e1e57d9b…66af1`; every entry correctly remains `NeedsReference`. |
| Q1 self/mismatch controls and blind bundle | `PASS / CONTROL_ONLY` | Self-match is zero on every matched distance; steel-center versus glass-corner yields `28.7939 dB` spectral RMSE, `0.642964` modal cost and spectral/modal/high-band-decay tags. Seed `42` emits two blinded pairs; repeated report SHA-256 is `65a6d437…da81`. |
| Feature-gated demo enabled/disabled regression | `PASS` | Committed `Begin` contact changes only PCM; runtime, RPG and physics checkpoint state remain identical. |
| 120-frame SDL/Ash reference demo with `physical-sound-selected-glass` | `PASS / DEBUG_FUNCTIONAL` | Selected-Q30 run: 56 simulation ticks, active audio, 112,000 queued samples, zero drops/faults; 49 debug underruns grant no platform/performance credit. |
| `audio-scene`, `play`, `host-check` | `PASS` | Baseline audio/play roots remain valid; workspace fmt/clippy/tests and boundary policy pass on Rust 1.97.1. |
| Candidate `AUDIO-PHYS-*` checks | `NOT_RUN / NOT_PROMOTED` | Local experiment checks do not create source/content/platform/performance or P1 admissibility. |

## Decisions that still constrain the work

### D-001 — Physical audio is a presentation consumer, not a world owner

- **Observation:** Gameplay hearing already consumes deterministic
  `AcousticFactV1`; PCM/device state is presentation-only.
- **Evidence:** SPEC-08, SPEC-30 and ADR-027 prohibit presentation feedback and
  second physical writers.
- **Decision:** Consume only committed immutable physical facts; source synth,
  propagation and mixer state remain reconstructible and unsaved.
- **Rejected alternatives:** Deriving AI hearing from waveform energy or
  letting audio write materials/contact/gameplay state.
- **Consequences:** Feature enabled/disabled/faulted must preserve every
  gameplay, ledger, physics, acoustic-fact and persistence root.
- **Uncertainty:** Exact presentation epoch/preroll behavior for canonical
  capture remains to be calibrated.
- **Reconsider when:** Never for authority; only a new superseding Accepted ADR
  could change the product contract.
### D-002 — Start with cooked modal rigid impact

- **Observation:** Modal impact has strong prior art and the smallest input
  surface; scraping requires temporal micro-contact/stick-slip detail.
- **Evidence:** O'Brien et al. 2002, Raghuvanshi/Lin 2006 and Zheng/James 2011
  as summarized in the research report.
- **Decision:** Retain modal P0 for accepted steel/wood, but the next glass-only
  counterfactual may add bounded deterministic transient micro-events; P1 still
  waits for a production consumer and P2 remains after P1.
- **Rejected alternatives:** Start with fracture, liquids, fire, cloth,
  footsteps, voice or birds.
- **Consequences:** Do not force every rigid material through one resonator;
  glass transient work stays presentation-only and creates no fracture fact.
- **Uncertainty:** Perceptual quality and the required mode count on the exact
  corpus are unmeasured.
- **Reconsider when:** Already met for glass identity after D and F; retain the
  modal path only where human evidence supports it.
### D-003 — Acoustic content is separate from physics material

- **Observation:** `PhysicsMaterialDescriptorV2` has contact parameters but no
  elastic, damping, radiation, acoustic geometry or calibration identity.
- **Evidence:** Current material contract and modal-synthesis source models.
- **Decision:** Candidate `AcousticMaterialProfileV1`, `ModalSoundModelV1` and
  `PhysicalSoundBindingV1` are separate PresentationOnly roles linked by exact
  revisions.
- **Rejected alternatives:** Add Young's modulus/damping/radiation directly to
  the physics material or infer acoustic identity from renderer material/name.
- **Consequences:** Cooking and fallback are content-package concerns; acoustic
  tuning cannot change collision response.
- **Uncertainty:** Minimal authoring/calibration UI and cooker inputs remain
  open.
- **Reconsider when:** A concrete consumer proves one field is truly a shared
  physical source of truth rather than presentation calibration.
### D-004 — Close the engine-owned contact projection before production wiring

- **Observation:** SPEC-26 normatively names relative velocity, impulse bounds,
  effective mass and material tags, but the current Rust contact record omits
  them.
- **Evidence:** Direct comparison of SPEC-26 and
  `crates/contracts/src/physics/contact.rs` on 2026-08-26.
- **Decision:** P0 may use exact synthetic fixtures. The requested P0.5 demo
  may use only committed `Begin` contacts plus an explicitly provisional
  adjacent-snapshot estimator behind an off-by-default feature. Production P1
  still waits for a consumer-driven contact projection and
  `PHYS-COLLISION-P1` evidence.
- **Rejected alternatives:** Raw PhysX callback/solver pointer/manifold data,
  callback-count noise or presenting the P0.5 identity/velocity proxy as a
  physical material/impulse contract.
- **Consequences:** The physical-audio draft creates no parallel contact
  contract.
- **Uncertainty:** Whether the existing planned normalized projection is
  sufficient for perceptual impact and persistent contact remains unmeasured.
- **Reconsider when:** The engine-owned projection is implemented and the P1/P2
  controls isolate a missing physical quantity.
### D-005 — Classical reference and fallback precede runtime learning

- **Observation:** Hybrid residuals and differentiable fitting are useful, but
  no measured Next Engine residual exists.
- **Evidence:** Impact-sound residual work and DiffSound in the research report.
- **Decision:** Use analytic/cooked modal reference first; allow offline fitting
  and authored residual/fallback; runtime neural residual is deferred.
- **Rejected alternatives:** Mandatory end-to-end neural audio or online model
  adaptation.
- **Consequences:** Network/model/provider failure cannot affect the first
  track; any later learned path needs immutable lineage and non-neural fallback.
- **Uncertainty:** Q30 passed the bounded human A/B and exact synthetic corpus,
  but real upper-partial balance, tail length and physical identification remain
  unmeasured on controlled recordings.
- **Reconsider when:** A controlled target justifies a materially separated
  residual experiment; neural runtime remains deferred.
### D-006 — Isolated roadmap experiment, no stage activation or shipping claim

- **Observation:** The product owner explicitly requested an independent
  experiment in the demo; R8 permits isolated pre-v1 experiments that do not
  affect mandatory gameplay.
- **Evidence:** Current roadmap and ADR-046 consumer-driven contract rule.
- **Decision:** Record the completed P0/P0.5 experiment in the R8 table without
  activating R8, changing baseline-audio completion or promoting P1.
- **Rejected alternatives:** Add an implementation stage, public V1 records or
  current-audio completion claim merely because the laboratory is audible.
- **Consequences:** Further tuning is allowed inside the experiment; production
  integration still requires a concrete consumer and Accepted ADR.
- **Uncertainty:** Priority relative to current gameplay/audio gaps is a product
  decision.
- **Reconsider when:** A roadmap slot and concrete player-visible impact
  consumer are chosen.
### D-007 — Quality needs an automatic selective ensemble, not per-sound approval

- **Observation:** Per-sound audition cannot scale to the intended source/parameter space; published metrics remain partial and exploitable.
- **Evidence:** The automated-validation report combines local CLAP/FAD failure,
  public impact corpora, decomposed audio evaluators and selective risk control.
- **Decision:** Build a versioned domain/formula registry plus an offline oracle
  from hard checks, temporal/modal/spectral specialists, grouped holdouts and
  calibrated selective risk. Outcomes are `Pass`, `Reject` or automatic clip
  `FallbackOutOfDomain`; no live human audit queue.
- **Rejected alternatives:** A single FAD/CLAP/ViSQOL/aesthetic score, direct
  prose judgment by a general audio model, or fitting and evaluating on the same
  object/impact split.
- **Consequences:** Q0/Q1 selected useful steel/wood candidates, but the
  DiffSound pair now falsifies reference fidelity as a glass-identity proxy.
  AV-P0A automates physical controls. AV-P0B and the YCB expansion show that
  BEATs/PANNs can recognize grouped real metal while white-residual v4 moves one
  head toward metal and the other toward glass. Registry V1 forbids a forged
  `Pass`; AV-P0C now measures grouped risk but the shuffled-envelope controls
  falsify the current temporal head as a complete quality judge.
- **Uncertainty:** Three parent groups per split cannot bound false-pass risk;
  spatial/force behavior and subjective naturalness remain unmeasured.
- **Reconsider when:** A frozen simpler validator matches the ensemble's grouped
  holdout risk, OOD behavior and failure diagnosis.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A compact modal core plus fused transient can evoke glass | Product-owner selected `09`; exact synthetic geometry now preserves position/force controls through Q30 | Synthetic FEM has no matched real recording and uses idealized support/damping/radiation | Surface mode-shape interpolation, then same-geometry recorded impacts |
| H2: The complete SPEC-26 contact projection is sufficient for impact excitation | It includes identity, point, velocity, impulse bounds, effective mass and tags | Current implementation omits the decisive numeric fields; solver-force fidelity is untested | Close one fixture projection and compare against exact synthetic excitation/control PCM |
| H3: Fixed-point reference resonators can meet both exact PCM and quality | Selected `09` repeats exactly; controlled-corpus Q30 RMS error is at most `7.987e-8` | One synthetic object is not a real quality or whole-mixer envelope | Preserve exact transfer while fitting only against held-out real/human evidence |
| H4: Rolling/scraping can use the ordinary committed contact stream | Rolling/contact synthesis prior art exists | High-quality work identifies micro-collision, chattering and stick-slip gaps | P2 speed/load/roughness corpus with resting/separation controls; add one flexible-contact counterfactual only if it fails |
| H5: Physical synthesis fits a useful whole-mixer budget | 16 selected voices cost `1.683 ms` p99 in the isolated lab tick; cooked payload is 1,536 bytes | Measurement excludes normal mixer, callback/device and varied voices; no product budget exists | Measure full mixer/callback p95/p99 on a declared production consumer before setting a budget |
| H6: A selective specialist ensemble can safely automate admitted impact domains | BEATs/PANNs disagreement selects fallback; AV-P0C now has frozen controlled negatives and leakage-safe grouped measurement | Shuffled wood envelopes false-pass on holdout/shadow; real coverage is `1/3` and `0/3`; upper risk is `0.7923` | Add amplitude-envelope trajectory features on the same pack, then broaden independent real objects |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), [SPEC-00](../../architecture/00-product-contract.md) and [SPEC-01](../../architecture/01-system-architecture.md).
2. [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md), [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../../architecture/30-presentation-extraction-and-render-content.md), ADR-027/046/058/071.
3. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
4. [Research report](../physical-sound-synthesis-research-2026-08-26.md), [quality evaluation](../physical-sound-quality-evaluation-research-2026-08-26.md),
   [automated validation](../physical-sound-automated-validation-research-2026-08-27.md), [AV-P0B](../physical-sound-corpus-benchmark-av-p0b-2026-08-27.md), [AV-P0C](../physical-sound-validator-av-p0c-2026-08-27.md), [steel v4](../physical-sound-steel-residual-v4-2026-08-27.md) and the [implementation plan](../../plans/2026-08-27-physical-sound-domain-admission-implementation-plan.md).
5. [Roadmap](../../roadmap.md) only for a future scheduling/scope decision.

## Next action

1. Preserve all frozen baselines, references and reports; freeze wood-B and
   retain glass-D/F only as rejected metal-like anchors.
2. Keep source/generated `09` artifacts external and retain the implemented Q30
   voice only as an explicit opt-in; do not replace Glass-H or authored clips.
3. Preserve AV-P0A's current-only tri-state report and complete control matrix;
   do not reinterpret its `Pass` as subjective quality or P1 evidence.
4. Preserve v3/v4 as `FallbackOutOfDomain` and the exact AV-P0C pack/report as
   negative evidence; do not start a v5 gain grid.
5. Add amplitude-envelope trajectory features against the frozen shuffled wood
   counterexamples without changing the source model or splits.
6. Broaden independent real object families and pre-register bounded risk versus
   coverage before enabling registry `Pass` or starting AV-P0D search.
7. Only on measured success, write the promoting consumer ADR and close the
   contact-projection/content/check plan before runtime code.
8. Roll back to the unchanged clip baseline if P0 fails or no bounded profile
   survives.

## Do not retry

- Universal acoustic material/body schema before a consumer — one model does
  not cover rigid contact, cloth, fluids, fire and biological sources.
- Raw PhysX callback to mixer — violates engine-owned stable projection and
  leaves replay/order/backend semantics undefined.
- Runtime eigensolver/FEM — preprocessing provides the compact runtime model.
- Runtime neural residual first — there is no measured residual or bounded
  classical comparator yet.
- Blind acoustic tuning or more stationary-white residual gain/T20 — without
  grouped holdouts it optimizes the score; v4 overfills the tail and shifts
  BEATs from metal toward glass.
- Single-metric or single-model judge — FAD/CLAP/ViSQOL/aesthetic/audio-language
  outputs are complementary diagnostics and individually gameable.
- Per-sound `NeedsHumanAudit` as the end-state — uncertainty must select the
  authored fallback; human evidence is optional frozen data/audit, not an asset gate.
- “Thousands of synthesized birds” based on Lyrebird — the cited repository is
  mostly field-audio data, not evidence for that claim.

## Handoff

- **Workspace state:** Registry V1, mutation pack and diagnostic grouped-risk
  report are implemented; defaults/public schemas/assets/ownership are unchanged.
- **Checks:** sound `39/39`, Clippy `-D warnings`, fmt, boundary scan and byte
  repeat pass; candidate product/platform/performance checks remain not run.
- **Remaining risk:** corpus scale/scope, envelope discrimination, calibration/OOD,
  spatial transfer, contact sufficiency, mixer cost and authoring are open.
- **Quality status:** wood-B/Q30 remain exact; residual v4 has no joint `3/3`;
  AV-P0C upper risk `0.7923` and low coverage require expansion/specialization.
- **Promotion needed:** Concrete consumer, later ADR-046 promotion, then exact content/contact/DSP profiles and ProductChecks.
