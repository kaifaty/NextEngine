# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / ABSOLUTE_ENVELOPE_REJECTED / RECORD_LEVEL_HYPOTHESIS_OPEN.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifacts:** [stage-splice comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-stage-ablation-2026-09-05/comparison.wav),
  glass first/middle, real/base/matched/base-early/base-late, generator2718.
  [Separate-envelope comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-envelope-flow-2026-09-05/comparison.wav)
  is a FAILED candidate. Shared audition gain0.63339858 on real/base/candidates;
  raw level failures preserved. No target recording enters neural inference.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Impacts:** prior improves only1/14 matched event crops; retain base, no LoRA
  sweep. Prefix/old unmatched-empty failures are superseded; exact extraction
  boundary and evidence in pilot note. Striker margins are not material proof.
- **Physical-control source:** `cluster-texture-training-grid-2026-09-05`,
  Figshare29438288v5, CC-BY4.0,60 records/242 files/86.69MB: wood0/steel65/glass74,
  Urethane probe,20–60mm/s ×0.5/1N. Same surfaces, not new objects. Keep
  measured/commanded controls separate. Acquisition details in pilot note.
- **Crossed velocity:** rank4/4804params, pooled neural/interpolation spectrum
  1.815/1.758dB,11/36 wins. Stop three-surface capacity/epoch/basis tuning;
  40mm/s gain was not robust. Disclosed development; stationary texture, not impact.
- **Friction countercheck:** clean AND machine-mic surface retrieval30/30;
  not independent quality validation, nor proof of a noise-only generator.
  No exact NLMS preprocessing replay; audit JSON in speed50 root.
- **Rain:** DataSuds10.23708/I0QYNM V2/CC-BY4.0; original CSV verified,
  converted TSV rejected. Three full WAVs; cross-site43958/43957 unfetched.
  Spectral gain7.469/7.580dB is not quality; stationary model loses temporal
  structure, AST fails real wet controls. No rain-spectrum MLP sweep; see note.
- **Pouring source:** `sound-of-water-source-2026-09-05`, Bagad et al.,
  HF `bpiyush/sound-of-water` revision12575460ee39d6adaebbe5aff531a5f4a24a627b.
  Dataset redistribution unspecified; do NOT inherit separate software MIT.
  Local research only.123 full48kHz WAVs/110261540bytes plus README/train CSV,
  125 files publisher/local hash verified. Annotation-only clean/constant/water
  selection. Author Test I/II/III and YouTube files not used. No foreign code run.
  93 train recordings/13 objects; whole containers18(glass13),30(PET17) excluded.
  Approximate constant flow is not measured ml/s or exact liquid level.
- **Pouring base:**245985param conditional STFT flow U-Net,53/1500updates,
  batch6/lr3e-4,64 Euler/32 reconstruction,16k/FFT512/hop256/256² patches.
  Material/shape/dimensions/duration/elapsed fraction inputs. Prior power and
  onset variants retained; no default change or runtime promotion.
- **Prior causal checks:**64/256 Euler does not fix deficit; wrong glass material
  wins13/13, PET correct17/17. No material claim. Endpoint penalty changes flow
  optimum, not proven cause of audible error; details in pilot note.
- **Prior phase controls:** power wins151/180 spectrum pairs, but middle CV
  error0.224->0.230. Both base/power miss phase CV change on new AND13 training
  objects; oracle preserves most. Prior broad AST positives are not quality proof.
- **Two-patch probe:** first training record/container1, matched/shuffled600-step
  fits; correct template6/6 versus parent/shuffled3/6. Phase is learnable here.
  Noisy target identifies phase for98% of uniform times in exact two-endpoint
  toy; possible shortcut, not a measurement of what the network learned.
- **Paired extension:** same93 train records,600 extra updates, all30 disclosed
  development recordings ×2 phases ×3 seeds. Spectrum wins158/180 but CV worsens;
  normalized AST120/180 vs parent180/180. Seed2718 fails all60; reject promotion.
- **Independent checks:** crossed decoder seeds do not explain the full failure.
  CLAP real/oracle8/8, base12/12 water, paired10/12; PET2718 favours birds,
  glass2718 remains water. Judges disagree; neither is naturalness authority.
- **Stage localization:** `pouring-flow-stage-ablation-2026-09-05`,13 training
  objects ×8 flow times: paired MSE lower everywhere, phase penalty small.
  Exact-endpoint oracle MSE<1e-12. Frozen stage splice at0.25 gives normalized
  AST base/paired/base-early/base-late12/9/11/9 of12; no clean restoration.
- **Envelope branch:** `pouring-envelope-flow-2026-09-05`,27424param/32-bin
  RMS MLP flows, matched/shuffled1500 steps, same93 train records, frozen base.
  Evaluation stopped at peak guard AFTER fitting, resumed exact checkpoints.
  120/360 generated signals fail raw headroom; common gain only enables audition.
  Matched spectrum first/middle12.572/15.077 vs base10.532/9.315dB; CV worsens,
  phase level change-0.034 vs real-5.390dB. Normalized AST88/180 vs base180/180.
  Reject; no absolute-envelope capacity/epoch sweeps or weakening peak guards.
- **AST controls:** `--ast-rms .005` optional; raw preserved. CUDA matches CPU
  top10/flags on180 raw+120 normalized WAVs, delta2.24e-6. CPU default, CLAP
  unchanged; mel warning remains. Do not tune thresholds or blacklist seeds.
- **Prior controls:** codec checks reject gross corruption; ESC-50 physical
  attributes are null. No posterior-mean, duration/CFG or caption-threshold retries.
- **Next action:** test a simple condition-to-relative-level predictor on93
  training records versus zero-phase/shuffled controls; compare absolute and
  per-record normalized targets on disclosed containers before another flow fit.
  Recording-gain/listener confounding is unproven. No new source/stack, invented
  labels or protected reuse. Preserve base/power; broad goal unchanged.
- **Verification:** two envelope fits, resumed evaluation and AST complete;
  61 focused tests, Ruff and718 written-WAV/hash checks pass; no jobs running.
  No Cargo/ProductCheck or engine audition: external Python lab only.
- **Full goal remains open:** robust quality/control, generalization and integration.

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
- Source MP3s: `ps2-freesound-wine-glass-v1/research`, same author/pack/train
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
