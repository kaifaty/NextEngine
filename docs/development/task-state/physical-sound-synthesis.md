# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `DRAFT_SPEC_COMPLETE / IMPLEMENTATION_NOT_SCHEDULED` |
| Updated | `2026-08-26` |
| Task key | `physical-sound-synthesis` |
| Scope | Research and Proposed architecture for a bounded physics-driven sound-source layer |
| Definition of done | Primary-source research, SPEC-45, routing and traceability agree on authority, first vertical, fallback and evidence; no runtime/public-contract/roadmap claim |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact future ProductCheck evidence outrank this file |

## Resume in 60 seconds

- **Current conclusion:** A presentation-only modal source-synthesis layer is
  feasible enough for a bounded P0/P1 experiment. Start with cooked rigid
  impacts; keep authored clips as fallback; do not schedule rolling/scraping or
  other source classes until impact evidence closes.
- **Why:** Primary work demonstrates offline modal preprocessing plus real-time
  contact excitation, while high-quality persistent contact requires richer
  micro-collision/stick-slip treatment. Current Next Engine authority
  boundaries already separate gameplay acoustic facts from PCM.
- **Next action:** If explicitly scheduled, freeze the tiny P0 steel/wood/glass
  corpus, reference recordings/offline solver, metrics and resource profile;
  implement no runtime contract yet.
- **Current blocker:** No roadmap slot, exact P0 corpus, quality metric, numeric
  recurrence, mode/voice budget or current complete contact-event
  implementation exists.
- **Do not retry:** A universal “all sounds from physics materials” design or a
  raw PhysX-callback mixer path; both erase required source-model and
  engine-owned-contract boundaries.
- **Reconsider when:** P0 produces a measured quality/cost point and a concrete
  production impact consumer is selected.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Research report](../physical-sound-synthesis-research-2026-08-26.md) | `REPORT_ONLY` | Modal rigid impact is credible; scraping and other modalities need separate evidence; no implementation is proven. |
| [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md) | `Proposed` | Candidate presentation ownership, content split, excitation boundary, fallback and P0–P2 sequence are explicit. |
| [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md) and current `AudioSceneSnapshotV1`/`AudioMixerV1` | `CURRENT_BASELINE_OBSERVED` | Clip playback, canonical PCM and gameplay/output separation exist; no physical source synth or new check result is claimed. |
| [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) versus current Rust `ContactEventV1` | `IMPLEMENTATION_GAP_OBSERVED` | Normative contact facts include velocity/impulse/effective mass/tags, but current record omits them; production audio must close the existing projection rather than consume raw callbacks. |
| Candidate `AUDIO-PHYS-*` checks | `NOT_RUN` | No source, contact, content, PCM, platform or performance claim is admissible. |

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
- **Decision:** P0 offline reference, then P1 production hammer/drop impact;
  P2 rolling/scraping only after P1.
- **Rejected alternatives:** Start with fracture, liquids, fire, cloth,
  footsteps, voice or birds.
- **Consequences:** The first implementation needs one compact modal cooker and
  reference resonator, not a universal procedural-audio framework.
- **Uncertainty:** Perceptual quality and the required mode count on the exact
  corpus are unmeasured.
- **Reconsider when:** P0 falsifies the modal approach at bounded quality/cost.

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

### D-004 — Close the engine-owned contact projection before runtime wiring

- **Observation:** SPEC-26 normatively names relative velocity, impulse bounds,
  effective mass and material tags, but the current Rust contact record omits
  them.
- **Evidence:** Direct comparison of SPEC-26 and
  `crates/contracts/src/physics/contact.rs` on 2026-08-26.
- **Decision:** P0 may use exact synthetic fixtures; production P1 waits for a
  consumer-driven contact projection and `PHYS-COLLISION-P1` evidence.
- **Rejected alternatives:** Raw PhysX callback/solver pointer/manifold data,
  callback-count noise or presentation-side guessing.
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
- **Uncertainty:** Pure modal output may not reach the eventual perceptual bar.
- **Reconsider when:** Two bounded classical/calibration cycles leave a named,
  measured residual that a small immutable model demonstrably removes.

### D-006 — No roadmap or shipping claim from the research draft

- **Observation:** The current roadmap's audio gaps are long-clip streaming and
  zone reverb; no physical-synthesis consumer or milestone is selected.
- **Evidence:** Current roadmap and ADR-046 consumer-driven contract rule.
- **Decision:** Index and route SPEC-45 as Proposed without editing roadmap
  status or current audio completion.
- **Rejected alternatives:** Add an implementation stage or public V1 records
  merely because the research is promising.
- **Consequences:** The next action requires an explicit product scheduling
  choice and a bounded P0 definition.
- **Uncertainty:** Priority relative to current gameplay/audio gaps is a product
  decision.
- **Reconsider when:** A roadmap slot and concrete player-visible impact
  consumer are chosen.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A compact modal model is perceptually useful for the first three object profiles | Established rigid-impact research and commercial-game precedent | No Next Engine corpus, content pipeline or blinded comparison | Render predeclared P0 impulses and compare peaks, decay, envelope/spectrum and perceptual judgments |
| H2: The complete SPEC-26 contact projection is sufficient for impact excitation | It includes identity, point, velocity, impulse bounds, effective mass and tags | Current implementation omits the decisive numeric fields; solver-force fidelity is untested | Close one fixture projection and compare against exact synthetic excitation/control PCM |
| H3: Fixed-point reference resonators can meet both exact PCM and quality | Existing mixer already produces exact integer 48 kHz PCM | Coefficient quantization may detune or over-damp high modes | Compare scalar fixed-point recurrence with high-precision offline reference across the P0 stability envelope |
| H4: Rolling/scraping can use the ordinary committed contact stream | Rolling/contact synthesis prior art exists | High-quality work identifies micro-collision, chattering and stick-slip gaps | P2 speed/load/roughness corpus with resting/separation controls; add one flexible-contact counterfactual only if it fails |
| H5: Physical synthesis fits a useful whole-mixer budget | Modal banks are compact and admit explicit LOD | Current engine has no measured physical-voice/model/callback cost | Profile P0/P1 with declared modes, voices, queue and whole-mixer p95/p99 before setting a budget |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), [SPEC-00](../../architecture/00-product-contract.md) and [SPEC-01](../../architecture/01-system-architecture.md).
2. [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md), [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../../architecture/30-presentation-extraction-and-render-content.md), ADR-027/046/058/071.
3. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
4. [Research report](../physical-sound-synthesis-research-2026-08-26.md).
5. [Roadmap](../../roadmap.md) only for a future scheduling/scope decision.

## Next action

1. Obtain an explicit product decision to schedule P0, without implying P1.
2. Freeze exact neutral geometry/material/calibration inputs, reference source,
   metrics, sample rate/window and candidate recurrence.
3. Render the offline corpus and select a bounded quality/cost point.
4. On success, write the promoting consumer ADR and close the contact-
   projection/content/check plan before runtime code.
5. Roll back to the unchanged clip baseline if P0 fails or no bounded profile
   survives.

## Do not retry

- Universal acoustic material/body schema before a consumer — one model does
  not cover rigid contact, cloth, fluids, fire and biological sources.
- Raw PhysX callback to mixer — violates engine-owned stable projection and
  leaves replay/order/backend semantics undefined.
- Runtime eigensolver/FEM — preprocessing provides the compact runtime model.
- Runtime neural residual first — there is no measured residual or bounded
  classical comparator yet.
- “Thousands of synthesized birds” based on Lyrebird — the cited repository is
  mostly field-audio data, not evidence for that claim.

## Handoff

- **Workspace state:** documentation-only Proposed SPEC, research note,
  task-state and navigation/traceability updates; no runtime/schema/content or
  roadmap change.
- **Checks:** documentation cheap path only; every executable
  `AUDIO-PHYS-*`, physics, play, persistence, content, platform and performance
  check remains `NOT_RUN(NoExecutableChange)`.
- **Remaining risk:** acoustic quality, calibration, contact-signal
  sufficiency, exact fixed-point DSP, cooker design, callback cost, propagation
  integration and content-author workflow are all unmeasured.
- **Promotion needed:** Concrete consumer plus later Accepted ADR under ADR-046;
  then exact content/contact/DSP profiles and ProductChecks.
