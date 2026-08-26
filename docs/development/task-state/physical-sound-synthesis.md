# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `P0_WOOD_ACCEPTED / GLASS_09_Q30_DEMO_PASS / P1_BLOCKED` |
| Updated | `2026-08-27` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture plus isolated fixed-point impact audition and feature-gated reference-demo experiment |
| Definition of done | Calibrate an external quality oracle that ranks held-out matched impact candidates better than any single uncalibrated metric while preserving the completed demo isolation and no P1/shipping claim |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** The selected DiffSound `09` thin-container archetype
  survives 48-kHz Q30 numerically and perceptually and now runs as an explicit
  opt-in 16-mode reference-demo voice; Glass-H remains the default.
- **Why:** Product-owner A/B judged Q30 B good, possibly better. Its demo PCM is
  within one S16 LSB of B; a 16-voice lab tick measured `1.683 ms` p99 locally.
- **Next action:** Stop coefficient tuning. Build one controlled exact-geometry
  glass corpus with held-out strike position/force before physical transfer.
- **Current blocker:** No controlled matched-reference corpus, held-out human
  preference labels, calibrated threshold, measured
  whole-mixer budget, Accepted consumer ADR or complete impulse/effective-mass/
  acoustic-material contact projection exists.
- **Do not retry:** Treating synthetic-target match as glass identity, blind preset tuning, or using FAD, CLAP, ViSQOL, an aesthetic
  model or a general audio model as the sole quality judge. Also retain the ban
  on universal material sound and raw PhysX-callback mixing.
- **Reconsider when:** P0 produces a measured quality/cost point and a concrete
  production impact consumer is selected.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Research report](../physical-sound-synthesis-research-2026-08-26.md) and [DiffSound trial](../physical-sound-diffsound-trial-2026-08-26.md) | `GLASS_09_Q30_DEMO_PASS / PHYSICAL_ID_OPEN` | Q30 tracks aligned f64 at `115.43 dB` SNR; product-owner A/B accepted B. Demo post-scale stays within one S16 LSB. Wrong geometry keeps material and wall thickness non-physical. |
| [Quality-evaluation research](../physical-sound-quality-evaluation-research-2026-08-26.md) | `CLASSICAL_Q0_Q1_IMPLEMENTED / HUMAN_CALIBRATION_OPEN` | Matched classical descriptors and blind A/B are available; no single automatic metric or uncalibrated control run is an admissible quality judge. |
| [Steel calibration](../physical-sound-steel-calibration-2026-08-26.md) and [wood/glass calibration](../physical-sound-wood-glass-calibration-2026-08-26.md) | `WOOD-B_ACCEPTED / GLASS-D-F_REJECTED / GLASS-G_PARTIAL_ACCEPT / GLASS-H_WEAK_PREFERENCE` | Keep H as provisional baseline and G as its close control; stop near-neighbor tuning. |
| [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md) | `Proposed` | Candidate presentation ownership, content split, excitation boundary, fallback and P0–P2 sequence are explicit. |
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
- **Uncertainty:** Q30 passed the bounded human A/B, but upper-partial balance,
  tail length and physical transfer remain unmeasured on controlled recordings.
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
### D-007 — Quality needs a human-calibrated ensemble, not one metric

- **Observation:** The current exact PCM failed human audition; published audio
  metrics measure different and exploitable notions of quality.
- **Evidence:** The quality-evaluation report records impact-perception studies,
  FAD sample/embedding dependence, ViSQOL's generative limits and DCASE's final
  human ranking after automatic shortlisting.
- **Decision:** Build an offline oracle from hard signal checks, matched modal/
  decay/spectral descriptors, physical control relations, frozen embeddings,
  held-out human preference and separate cost. Codex consumes the report and
  asks for human review on uncertainty/disagreement.
- **Rejected alternatives:** A single FAD/CLAP/ViSQOL/aesthetic score, direct
  prose judgment by a general audio model, or fitting and evaluating on the same
  object/impact split.
- **Consequences:** Q0/Q1 selected useful steel/wood candidates, but the
  DiffSound pair now falsifies reference fidelity as a glass-identity proxy.
  Classical/CLAP screens remain diagnostics until held-out human calibration.
- **Uncertainty:** The smallest local reference corpus and human-label count are
  chosen only after measuring recording and label variance.
- **Reconsider when:** A frozen single metric demonstrably outperforms the
  ensemble on held-out local human judgments without losing failure diagnosis;
  current evidence gives no reason to expect that.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A compact modal core plus fused transient can evoke glass | Product-owner selected `09` and accepted Q30 B; exact opt-in demo voice preserves it within one S16 LSB | Physical transfer beyond this one proxy-geometry archetype remains unmeasured | Exact scaled glass mesh plus controlled held-out strike positions/forces |
| H2: The complete SPEC-26 contact projection is sufficient for impact excitation | It includes identity, point, velocity, impulse bounds, effective mass and tags | Current implementation omits the decisive numeric fields; solver-force fidelity is untested | Close one fixture projection and compare against exact synthetic excitation/control PCM |
| H3: Fixed-point reference resonators can meet both exact PCM and quality | Selected `09` repeats exactly; Q30 has `1.136e-7` RMS error and passed human A/B | One archetype is not a stability envelope or whole-mixer result | Sweep bounded frequency/damping/duration limits only against the controlled corpus |
| H4: Rolling/scraping can use the ordinary committed contact stream | Rolling/contact synthesis prior art exists | High-quality work identifies micro-collision, chattering and stick-slip gaps | P2 speed/load/roughness corpus with resting/separation controls; add one flexible-contact counterfactual only if it fails |
| H5: Physical synthesis fits a useful whole-mixer budget | 16 selected voices cost `1.683 ms` p99 in the isolated lab tick; cooked payload is 1,536 bytes | Measurement excludes normal mixer, callback/device and varied voices; no product budget exists | Measure full mixer/callback p95/p99 on a declared production consumer before setting a budget |
| H6: A calibrated ensemble can rank candidates well enough for mostly autonomous iteration | Impact perception has interpretable spectral/temporal cues; automatic embedding/aesthetic models and human protocols exist | No local reference corpus, preference labels or held-out agreement measurement exists | Collect labels, then compare each metric and ensemble on leave-one-object/position-out human judgments |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), [SPEC-00](../../architecture/00-product-contract.md) and [SPEC-01](../../architecture/01-system-architecture.md).
2. [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md), [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../../architecture/30-presentation-extraction-and-render-content.md), ADR-027/046/058/071.
3. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
4. [Research report](../physical-sound-synthesis-research-2026-08-26.md) and
   [quality-evaluation research](../physical-sound-quality-evaluation-research-2026-08-26.md).
5. [Roadmap](../../roadmap.md) only for a future scheduling/scope decision.

## Next action

1. Preserve all frozen baselines, references and reports; freeze wood-B and
   retain glass-D/F only as rejected metal-like anchors.
2. Keep source/generated `09` artifacts external and retain the implemented Q30
   voice only as an explicit opt-in; do not replace Glass-H or authored clips.
3. Use exact geometry and controlled impacts for physical transfer. Collect human
   labels, validate leave-one-object or
   leave-one-position-out ranking and only then run bounded parameter search.
4. Compare reference-fitted candidates and select or reject a bounded quality/
   cost point.
5. Only on measured success, write the promoting consumer ADR and close the
   contact-projection/content/check plan before runtime code.
6. Roll back to the unchanged clip baseline if P0 fails or no bounded profile
   survives.

## Do not retry

- Universal acoustic material/body schema before a consumer — one model does
  not cover rigid contact, cloth, fluids, fire and biological sources.
- Raw PhysX callback to mixer — violates engine-owned stable projection and
  leaves replay/order/backend semantics undefined.
- Runtime eigensolver/FEM — preprocessing provides the compact runtime model.
- Runtime neural residual first — there is no measured residual or bounded
  classical comparator yet.
- Blind acoustic tuning after the failed audition — without frozen references
  and held-out human calibration it only optimizes the latest impression.
- Single-metric or single-model judge — FAD/CLAP/ViSQOL/aesthetic/audio-language
  outputs are complementary diagnostics and individually gameable.
- “Thousands of synthesized birds” based on Lyrebird — the cited repository is
  mostly field-audio data, not evidence for that claim.

## Handoff

- **Workspace state:** laboratory now includes the selected 16-mode 48-kHz Q30
  voice, external WAV/cost report and explicit `physical-sound-selected-glass`
  demo route; Glass-H default, public schemas and ownership are unchanged.
- **Checks:** local synthesis determinism/distinction, demo enabled/disabled
  authoritative-state regression, 120-frame SDL functional launch,
  `audio-scene`, `play`, focused clippy/tests, boundary scan and broad
  `host-check` pass. Candidate `AUDIO-PHYS-*`, content, persistence, formal
  platform and performance promotion checks remain not promoted or not run.
- **Remaining risk:** broader acoustic quality, calibration, complete contact-signal
  sufficiency, callback/whole-mixer cost, propagation
  integration and content-author workflow are all unmeasured.
- **Quality status:** wood-B passed; glass-D/F failed; G/H improved weakly.
  Selected `09` is reproduced at `86.04 dB` SNR, transferred to Q30 at
  `115.43 dB` and human-accepted; broader held-out ranking remains open.
- **Promotion needed:** Concrete consumer plus later Accepted ADR under ADR-046;
  then exact content/contact/DSP profiles and ProductChecks.
