# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `PS2_THREE_E3_PROJECT_GROUPS / GLASS_5_OF_16 / TWO_REALIMPACT_E2_OBJECTS / E3_EXPANSION_NEXT / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-27` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture plus isolated fixed-point impact/demo and external controlled-corpus experiments |
| Definition of done | Demonstrate an external selective validator whose automatic pass has measured bounded false-pass risk on grouped real/mutation holdouts, with automatic clip fallback for OOD and no per-sound human gate |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact future ProductCheck evidence outrank this file |

## Resume in 60 seconds
- **Current conclusion:** PS-2 uses published internet data only. AV-MSF, YCB
  Impact and CMU Heller validate as 13 E3 objects/33 recordings in three source
  groups; Glass coverage is `5/16` and 17 recordings. Two exact REALIMPACT
  object rows have typed E2 evidence but remain fallback; `Pass` stays disabled.
- **Why:** Product-owner constraint dated 2026-08-27. Evidence is claim-scoped:
  external `E1` synchronized, `E2` transfer, `E3` identified-real and `E4`
  synthetic sources receive only the credit their bytes/metadata establish.
- **Next action:** Preserve the byte-identical GreenGoblet bounded-range import,
  then search another published project for stable object-level Glass E3
  identities/repeats; do not count E2 objects as validator groups.
- **Current blocker:** Eleven Glass object groups, 35 reject parents and
  complementary force/geometry/support E2/E1 claims remain open.
- **Do not retry:** Treating synthetic-target match as glass identity, blind preset tuning, or using FAD, CLAP, ViSQOL, an aesthetic
  model or a general audio model as the sole quality judge. Also retain the ban
  on universal material sound, raw PhysX-callback mixing and local recording.
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
| [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), [internet corpus policy](../physical-sound-internet-corpus-policy-ps2-2026-08-27.md), [AV-MSF multi-object pilot](../physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md), [independent YCB E3 pilot](../physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md), [independent Heller E3 pilot](../physical-sound-heller-independent-e3-pilot-ps2-2026-08-27.md), [Greatest Hits discriminator](../physical-sound-greatest-hits-discriminator-ps2-2026-08-27.md), [typed REALIMPACT E2 adapter](../physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md), [GreenGoblet range pilot](../physical-sound-realimpact-green-goblet-range-pilot-ps2-2026-08-27.md) and [subsystem roadmap](../../plans/physical-sound-synthesis-roadmap.md) | `PS2_THREE_E3_PROJECT_GROUPS / GLASS_5_OF_16 / TWO_REALIMPACT_E2_OBJECTS` | Three E3 projects repeat at 13 objects/33 recordings; Glass is `5/16`. Greatest Hits proves material events but no stable object identity. Two REALIMPACT objects now have typed fallback E2 rows; GreenGoblet uses only 1.61 MB of ranges from its 2.31 GB archive. No source has admission credit. |
| [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md) and current `AudioSceneSnapshotV1`/`AudioMixerV1` | `CURRENT_BASELINE_OBSERVED` | Clip playback, canonical PCM and gameplay/output separation remain the promoted baseline; the physical source synth is isolated experimental code. |
| [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) versus current Rust `ContactEventV1` | `IMPLEMENTATION_GAP_OBSERVED` | Normative contact facts include velocity/impulse/effective mass/tags, but current record omits them; production audio must close the existing projection rather than consume raw callbacks. |
| `xtask physical-sound-lab` external audition and cost report | `PASS / NON_GATING_COST` | Frozen baselines remain exact; selected Q30 WAV SHA is `c912806c…b9c823`. On Ryzen 3950X, 16 voices cost `1.483/1.683 ms` p50/p99 per 1,600-frame lab tick, `5.05%` of that window; this is not a whole-engine budget. |
| Glass-object set and DiffSound audition | `DIFFSOUND_AND_Q30_PERCEPTUAL_ACCEPT / UNCALIBRATED` | Selected `09` and transferred Q30 B sound glass-like; identity and reference fidelity remain separate, and one archetype is not a generic glass model. |
| Product-owner wood/glass audition, 2026-08-26 | `WOOD-B ACCEPT / GLASS-D-F FAIL / G PARTIAL_ACCEPT / H WEAK_PREFERENCE` | H is tentatively better but hard to distinguish from G; preserve both and require a stronger discriminator. |
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
### D-007 — Quality uses an internet corpus and automatic selective ensemble

- **Observation:** Per-sound audition cannot scale, and the product owner will
  not perform physical capture; required real evidence must be found online.
- **Evidence:** Product-owner constraint 2026-08-27 plus the automated-validation
  report, external corpus pilots and selective-risk results.
- **Decision:** Build a versioned internet-source/corpus/formula registry and an
  offline oracle from claim-scoped `E1`–`E4` evidence, specialists, grouped
  holdouts and calibrated risk. Outcomes remain `Pass`, `Reject` or clip fallback.
- **Rejected alternatives:** Local mic/hammer acquisition, a live human queue,
  one general score/model, or inventing absent axes across datasets.
- **Consequences:** Force hardware is no blocker. Bounded fetch/cache/archive
  handling, three-project E3 normalization and two REALIMPACT E2 object rows are
  complete. Glass is `5/16`; expand only explicitly identified independent E3/E2
  evidence, and keep insufficient evidence fallback-only.
- **Uncertainty:** Published sources may not cover every force/geometry/support
  axis or the powered group count.
- **Reconsider when:** Only an explicit product-owner reversal permits local
  capture; validator simplification still requires equal bounded risk/coverage.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A compact modal core plus fused transient can evoke glass | Product-owner selected `09`; exact synthetic geometry now preserves position/force controls through Q30 | Synthetic FEM has no matched real recording and uses idealized support/damping/radiation | Fit surface mode shapes against published `E2` transfer responses and validate identity on independent `E3` recordings |
| H2: The complete SPEC-26 contact projection is sufficient for impact excitation | It includes identity, point, velocity, impulse bounds, effective mass and tags | Current implementation omits the decisive numeric fields; solver-force fidelity is untested | Close one fixture projection and compare against exact synthetic excitation/control PCM |
| H3: Fixed-point reference resonators can meet both exact PCM and quality | Selected `09` repeats exactly; controlled-corpus Q30 RMS error is at most `7.987e-8` | One synthetic object is not a real quality or whole-mixer envelope | Preserve exact transfer while fitting only against held-out published real evidence |
| H4: Rolling/scraping can use the ordinary committed contact stream | Rolling/contact synthesis prior art exists | High-quality work identifies micro-collision, chattering and stick-slip gaps | P2 speed/load/roughness corpus with resting/separation controls; add one flexible-contact counterfactual only if it fails |
| H5: Physical synthesis fits a useful whole-mixer budget | 16 selected voices cost `1.683 ms` p99 in the isolated lab tick; cooked payload is 1,536 bytes | Measurement excludes normal mixer, callback/device and varied voices; no product budget exists | Measure full mixer/callback p95/p99 on a declared production consumer before setting a budget |
| H6: A selective specialist ensemble can safely automate admitted impact domains | PS-2 rejects invented axes; three E3 projects validate 13 objects/33 recordings and two REALIMPACT E2 objects validate repeatably | Glass is only `5/16`; Greatest Hits has material events but no object identity, both E2 rows are fallback and no powered partitioned corpus exists | Expand stable-object E3 and complementary E2/E1 evidence, then run one sealed shadow only after the plan freezes |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), [SPEC-00](../../architecture/00-product-contract.md) and [SPEC-01](../../architecture/01-system-architecture.md).
2. [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md), [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../../architecture/30-presentation-extraction-and-render-content.md), ADR-027/046/058/071.
3. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
4. [Research report](../physical-sound-synthesis-research-2026-08-26.md), [quality evaluation](../physical-sound-quality-evaluation-research-2026-08-26.md),
   [automated validation](../physical-sound-automated-validation-research-2026-08-27.md), [AV-P0C](../physical-sound-validator-av-p0c-2026-08-27.md), [PS-1](../physical-sound-validator-ps1-2026-08-27.md), [internet corpus policy](../physical-sound-internet-corpus-policy-ps2-2026-08-27.md), [AV-MSF multi-object pilot](../physical-sound-av-msf-e3-multiobject-pilot-ps2-2026-08-27.md), [independent YCB E3 pilot](../physical-sound-ycb-independent-e3-pilot-ps2-2026-08-27.md), [independent Heller E3 pilot](../physical-sound-heller-independent-e3-pilot-ps2-2026-08-27.md), [Greatest Hits discriminator](../physical-sound-greatest-hits-discriminator-ps2-2026-08-27.md), [PS-2 plan](../physical-sound-corpus-plan-ps2-2026-08-27.md), [REALIMPACT E2 adapter](../physical-sound-realimpact-e2-adapter-ps2-2026-08-27.md) and the [implementation plan](../../plans/2026-08-27-physical-sound-domain-admission-implementation-plan.md).
5. [Roadmap](../../roadmap.md) only for a future scheduling/scope decision.

## Next action

1. Preserve frozen Q30, AV-P0A/C, PS-1 and rejected-v3/v4 evidence; keep all
   source/generated artifacts external and do not reinterpret a control pass as
   subjective quality or P1 evidence.
2. Preserve PS-2 hashes/import contract, AV-MSF/YCB/Heller E3 catalog,
   Greatest Hits rejection and both REALIMPACT E2 reports; GreenGoblet bounded
   retrieval is complete, so expand stable-object E3 from Glass `5/16` before
   `Pass`, PS-3 or AV-P0D.
3. Only on measured success, write the promoting consumer ADR and close the
   contact-projection/content/check plan before runtime code.
4. Roll back to the unchanged clip baseline if P0 fails or no bounded profile
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
- Local microphone/hammer capture — required real evidence is internet-sourced;
  unavailable published axes remain unavailable or fallback-only.
- “Thousands of synthesized birds” based on Lyrebird — the cited repository is
  mostly field-audio data, not evidence for that claim.

## Handoff

- **Workspace state:** Registry V1, PS-1, PS-2 plan/`E1` import, bounded
  source/cache/archive audit, three-project E3 normalization, one typed
  REALIMPACT adapter and two frozen E2 object rows exist; public schemas/assets/ownership are unchanged.
- **Checks:** sound tests, Clippy `-D warnings`, fmt, boundary scan and online/
  offline report repeat pass; candidate product/platform/performance checks remain not run.
- **Remaining risk:** eleven missing Glass groups, broader E2 arrays, corpus scale/scope, calibration/OOD,
  spatial transfer, contact sufficiency, mixer cost and authoring are open.
- **Quality status:** no powered multi-source internet corpus exists; no quality,
  corpus admission or production claim exists until claim-scoped evidence passes.
- **Promotion needed:** Concrete consumer, later ADR-046 promotion, then exact content/contact/DSP profiles and ProductChecks.
