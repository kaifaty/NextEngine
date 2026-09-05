# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / TEMPORAL_POURING_GENERATOR / LEVEL_AND_ENVELOPE_MISMATCH.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [neural pouring audition](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-audition-2026-09-05/generated.wav),
  4.08s/16kHz, glass cylinder10cm high/7cm diameter,15s pour at fraction0.2,
  seed2718, explicit audition gain10. No recording at inference; not calibrated.
  [New-container comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-2026-09-05/comparison.wav):
  first source-order glass/PET cases, real -> neural -> oracle,27.48s, gain1.
  Coarse water event recognized; exact material/geometry response unproven.
- **Exact current evidence and reproduction:**
  [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Latest roots: `pouring-flow{,-standalone,-audition}-2026-09-05`, under
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
- **Impacts:** prior improves only1/14 matched event crops; retain base, no LoRA
  sweep. Prefix/old unmatched-empty failures are superseded; exact extraction
  boundary and evidence in pilot note. Striker margins are not material proof.
- **Physical-control source:** `cluster-texture-training-grid-2026-09-05`,
  Figshare29438288v5, CC-BY4.0,60 records/242 files/86.69MB: wood0/steel65/glass74,
  Urethane probe,20–60mm/s ×0.5/1N. Same surfaces, not new objects. Keep
  measured/commanded controls separate. Range reader resolves canonical URL
  per request (10s redirect expiry); mini lacks raw audio. Details in pilot note.
- **Crossed-velocity result:** fixed rank4/4,804params/1500steps. Neural versus
  interpolation spectrum RMSE30:1.904/1.759;40:1.690/1.729;50:1.850/1.788dB.
  Pooled1.815/1.758,11/36 wins. Stop this three-surface capacity/epoch/basis
  tuning; the apparent40mm/s gain is not robust. All folds disclosed development.
  No pretraining; physical inputs -> stationary Gaussian texture, not impacts.
- **Counterchecks:** repeat scatter contributes13–17% of neural MSE.
  Matched speed/load surface retrieval is30/30 for BOTH clean and machine-mic
  spectra. That score is not independent quality validation. It does not prove
  the generator uses only noise. Upstream noncausal NLMS inspected, not executed;
  no exact archive-preprocessing replay. Audit JSON in speed50 root.
- **Rain baseline:** source DataSuds10.23708/I0QYNM V2/CC-BY4.0, exact
  original-CSV units/timestamps verified; rejected converted TSV remains unused.
  Only three full WAVs; cross-site43958/43957 unfetched. Small wet-day spectral
  gain7.469/7.580dB is not waveform quality. Stationary spectrum loses temporal
  structure; AST fails real wet controls. No further rain-spectrum MLP sweep.
  Artifacts/units/controls in pilot note and `amazon-rain-neural-2026-09-05`.
- **Pouring source:** `sound-of-water-source-2026-09-05`, Bagad et al.,
  HF `bpiyush/sound-of-water` revision12575460ee39d6adaebbe5aff531a5f4a24a627b.
  Dataset redistribution unspecified; do NOT inherit separate software MIT.
  Local research only.123 full48kHz WAVs/110261540bytes plus README/train CSV,
  125 files publisher/local hash verified. Annotation-only clean/constant/water
  selection. Author Test I/II/III and YouTube files not used. No foreign code run.
  93 train recordings/13 objects; whole containers18(glass13),30(PET17) excluded.
  Approximate constant flow is not measured ml/s or exact liquid level.
- **Pouring fit:**245985params conditional STFT flow U-Net, seed53/1500updates,
  batch6/lr3e-4,64 Euler/32 phase-reconstruction steps. Fixed16k/FFT512/hop256,
  256×256 patches; material, shape, dimensions, duration, elapsed fraction inputs.
  All30 excluded-object recordings get fixed FIRST4.08s/seed314 evaluation.
  Neural/nearest/oracle spectrum RMSE10.903/11.343/0.168dB;16/30 neural wins,
  but PET group loses8.831/7.088dB. Median neural level-8.995dB; envelope CV
  0.599 versus real1.179/oracle1.106. Not robust physical generalization.
- **Pouring validator:** unchanged AST Water/Pour top5 real30/30,oracle30/30,
  neural29/30; simple negative controls pass. Useful coarse-event check, not
  material/naturalness acceptance. NumPy mel-filter warning remains. No tuning
  to these scores. Exact model/results/raw tags in pouring root and pilot note.
- **Prior controls:** codec checks reject gross corruption. No posterior-mean,
  duration/CFG or caption-bank threshold retries; exact evidence in pilot note.
  ESC-50 physical attributes are null; no material-identity validation for steel.
- **Next action:** improve this temporal generator, not another source search.
  Discriminate conditioning error versus flow integration/level-loss bias with
  swapped controls and step refinement; keep the original checkpoint. If the
  integrator is not causal, test a focused level/envelope-aware training loss
  with a new playable output and unchanged full development comparison. No
  physical-accuracy claim from AST or reconstruction; no generic epoch sweep.
  No repeated generic-caption/glass SFT sweeps, modal-MLP restart, invented labels,
  protected reuse or new stack. Demo/base unchanged; no product admission.
- **Verification:** pouring fit/inference/AST terminal;40 focused
  tests and Ruff pass. Source units/hashes, no-reference inference and WAV
  checks verified; exact evidence in pilot note. No Cargo/ProductCheck
  or engine audition: external Python lab only.
- **All goal requirements remain open beyond this baseline:** independent
  robust validation, robust audible gains from learning, broader physical
  controls, demonstrated new-condition generalization and engine integration.

## Preserve these constraints

- The user will not record impacts, hit glass or supply force-sensor data.
  Use internet sources. Preserve source attribution and applicable terms;
  unknown/incompatible redistribution terms exclude distribution.
- Generate playable media at each meaningful experiment checkpoint. Keep all
  candidates and honest failures; protocols, inventories and validators do
  not replace the audible deliverable. Supporting-only debt is currently zero.
- This broad goal does not authorize runtime neural weights or gameplay
  authority changes. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md)
  is Proposed; report-only waveform generation/authored-asset research is
  allowed. Production retains ordinary clips and an authored fallback.
  Promotion requires the existing ADR/consumer/ProductCheck workflow.
- Do not repurpose protected holdout, shadow, validator or one-shot roles.
  Opened development/train evidence cannot become independent test evidence.
  Pretraining overlap of foundation models is unknown, not automatically clean.
- Do not invent geometry, composition, exact force, velocity or listener
  metadata from material labels or audio. Perceptual prompt fidelity and
  calibrated physical response are separate claims.
- Generated WAVs, datasets, weights and caches remain outside Git. Do not
  replace the liked glass demo/profile with a weak research candidate.
- Keep one roadmap. [V46](../../plans/physical-sound-synthesis-roadmap-v46.md)
  records the earlier bounded impact admission program, not proof the broad
  goal is achieved. Individual lab runs do not change product roadmap status.

## Previous audible work — controls, not the endpoint

- **Liked training reconstruction:** [first pilot result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-first-2026-09-04/result.json),
  [comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-first-2026-09-04/comparison.wav).
  User liked three fitted glass reconstructions; no generalization.
- **Frozen transfer failure:** [unseen result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-unseen-2026-09-04/result.json).
  Nearest old parameters beat the MLP on 9/9 later strikes, not new objects.
- **Automatic cycle:** [cycle result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-cycle-2026-09-05/result.json).
  16 train strikes,11 later development strikes. Expanded/anchored MLPs fit
  training but lose to the analytic-only control; exact errors are in the note.
- Do not retry size/epoch variants of that supervised parameter MLP. Sparse
  data, nonunique modal targets and parameter/audio-loss mismatch were not
  isolated. More importantly, audio-to-parameter input is the wrong interface
  for the full goal. An input-dependent analytic fit is not a learned gain.
- The three source MP3s remain in
  `ps2-freesound-wine-glass-v1/research`; same author/pack and generator-train
  family, not known identical objects. Do not infer wall thickness or force.

## Legacy admission evidence and forbidden retries

- [V46 B0](../physical-sound-v46-b0-lane-aware-control-tournament-result-2026-09-03.md)
  remains NoUsefulTeacher, M0 admission blocked, real support 71/105. E0/V0
  source-power work is an admission path, not a prerequisite for report-only WAVs.
- [Corrected corpus](../physical-sound-v43-d2-c0r-identity-repair-result-2026-09-03.md)
  supersedes leaked AV-MSF/ObjectFolder role projections.
  [Disclosed roster](../physical-sound-v40-d0-disclosed-roster-result-2026-09-03.md)
  preserves nine permanently opened families. Opened ObjectFolder Real,
  YCB bj5w8 and REALIMPACT revisions are not clean protected evidence.
- V12 object 41 remains acquisition OOD: do not lower force coverage, select
  favourable contacts or open its response/protected roles. For object 92 do
  not drop contact 35 or download the already-proven irrelevant 12-GiB segment.
- V16–V37 spent protected experiment families stay closed. Do not tune their
  thresholds, roles, seeds, capacities or contacts from opened results, or
  reconstruct missing outputs from failed runs. In particular, M0c physical
  response and QSO-v0 geometry/contact transfer were rejected.
- Stationary random-phase residual, universal codec and prompt-to-waveform
  cannot be relabelled as admitted physical formulas. A waveform model is
  allowed as this separate report-only generator, with its own explicit claims.
- Prior detailed D-001…D-098 history and retired plans are available in Git
  at `ed8b9401:docs/development/task-state/physical-sound-synthesis.md`.
