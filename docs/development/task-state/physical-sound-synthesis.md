# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / GAUSSIAN_SKIP_AUDIO_REJECTED / SINGLE_CROP_LEARNABILITY_NEXT.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest reference-free experiment:** [three-way comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-latent-gaussian-skip-evaluation-2026-09-05/ablation-comparison.wav),
  13.74s, retained-base/matched-zero-plain/Gaussian-skip; glass10,seed2718/gain1.
  Both new2000-step runs REJECTED: novel AST0/12, CLAP plain10/12/skip7/12.
  No source/cache at inference;892992param flow + frozen Oobleck decoder.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Impacts:** prior improves1/14 matched crops; no LoRA sweep/material claim.
- **Friction:** `cluster-texture-training-grid-2026-09-05`, Figshare29438288v5/
  CC-BY4,60 records; neural/interpolation1.815/1.758dB,11/36 wins. No capacity/
  epoch/basis sweeps. Retrieval30/30 is not quality; exact data/limits in note.
- **Rain:** DataSuds10.23708/I0QYNM V2/CC-BY4.0; original CSV verified,
  converted TSV rejected. Stationary model loses temporal structure, AST fails
  real wet controls. No rain-spectrum MLP sweep; exact data/results in note.
- **Pouring source:** `sound-of-water-source-2026-09-05`, Bagad et al.,
  HF `bpiyush/sound-of-water` revision12575460ee39d6adaebbe5aff531a5f4a24a627b.
  Dataset redistribution unspecified; do NOT inherit separate software MIT.
  Local research only.123 verified full48kHz WAVs, annotation-only clean/constant/
  water selection. Author Test I/II/III and YouTube not used; no foreign code run.
  93 train recordings/13 objects; whole containers18(glass13),30(PET17) excluded.
  Approximate constant flow is not measured ml/s or exact liquid level.
- **Pouring base:**245985param conditional STFT flow,53/1500updates,64 Euler/
  32 reconstruction,16k/FFT512/hop256/256² patches. Retained, no promotion.
- **Prior checks:**256 Euler does not fix deficit; wrong glass material wins13/13.
- **Prior phase/envelope fits:** paired extension AST120/180; seed2718 fails60/60,
  not explained by decoder seed. Envelope120/360 headroom failures; splices fail.
  No retries/absolute-envelope sweep/guard weakening; exact runs in pilot note.
- **Prior relative controls:** gain confounding unproven; relative level/static
  timbre/EQ fail absolute spectrum or shape/CV. No gain/static-EQ sweeps.
- **AST:** raw and RMS0.005 retained; CUDA/CPU correspondence checked, mel
  warning remains. No threshold/seed tuning. Codec checks reject gross corruption;
  ESC-50 physical attributes null. No posterior-mean/duration/CFG retries.
- **Temporal diagnostic:** classical ridge falsely tracks noise; not real labels.
  Frozen Sound of Water pitch model has falling-tone error1382 cents and same
  corpus exposure, not independent validation. Full provenance in pilot note.
- **Teacher/head reuse:** `pouring-resonance-head-2026-09-05`,4993params,
  OOF277.5 vs simple357.0 cents,6/13 wins. Do not retrain; exact teacher/head
  provenance in pilot note. Corpus overlap is not clean evidence.
- **Prior renderer rejects:** moving-band/144-parameter adapter worsen spectra
  even with teacher guidance. No tiny-adapter/filter sweep; exact runs in note.
- **Temporal noise decoder rejected:** full/single-record and phase-refinement
  all fail semantics. No smooth-noise/phase sweep; exact runs in note.
- **Phase oracle:** exact magnitude AST0->6/6 after consistency; CLAP6/6 both. See note.
- **CVAE rejected:** original and rec/critic continuations fail posterior/prior
  semantics; phase2/32 mismatch is not a rescue. No CVAE/critic/capacity/epoch/
  phase-budget sweeps. Exact runs/coarsening/research in pilot note.
- **Frozen waveform codec:** `pouring-wave-codec-2026-09-05`,cached TangoFlux
  revision367005e9, Oobleck SHA d73619a1. Six disclosed4.08s crops: mean AST5/6,
  posterior15/18,both levels; CLAP6/6+18/18. Glass18 middle fails AST throughout.
  Input16k resampled44.1k/dual mono,64x88 latent. Reconstruction is NOT the goal.
- **Latent fit/cache completed:** `pouring-oobleck-cache-2026-09-05`,93train
  recordings/13objects ×first/middle/last=279crops; posterior mean/std/controls.
  Original2000-step flow AST0/12dev; no quality claim. Train-only normalization
  includes posterior variance;64Euler, no phase algorithm. Do not redo cache/fit.
- **Trajectory probe:** `pouring-latent-trajectory-probe-2026-09-05`. Start with
  target/noise at t0/.5/.875/1: development AST0/1/9/9 of12,CLAP9/9/12/12;
  training AST0/1/6/6 raw (normalized.5=0),CLAP2/6/6/6. Late privileged paths
  preserve water; target injection is NOT a reference-free solution.
- **Gaussian skip completed:** `pouring-latent-zero-plain-2026-09-05` and
  `pouring-latent-gaussian-skip-2026-09-05`,same cache/2000steps/seed53/zero output.
  Field t0 MSE1.385->0.995,t.9841.439->1.046, but dev spectrum8.512->8.860dB,
  CVerror0.241->0.378; both AST0/12. Endpoint improvement is NOT audio quality.
  No skip-weight/epoch/lr/capacity sweep. All fits terminal; don't repeat.
- **Variance discriminator:** skip directory `latent-variance-diagnostic.json`.
  Every channel has >90% mean-signal fraction (average95.18%); mean covariance
  rank54/61 for95/99% variance. Excess posterior-noise/mostly-unused-channel
  hypothesis rejected here. No mean-mode/channel-mask/PCA training from this.
- **Next action:** one-crop learnability control using cache row0 (first training
  record/first phase), same frozen codec and Gaussian-skip model,2000steps.
  Compare source-free fitted samples with exact one-crop Gaussian-posterior
  decoder controls. Separate optimizer/learner failure from multi-example path
  learning; a successful memorization is NOT new-object generation or the goal.
- **Verification:**106 focused tests, Ruff,138 new WAVs, CLI exact replay; all jobs terminal.
  No runtime/default/ProductCheck promotion. Full multi-event goal remains open.

## Preserve these constraints

- The user will not record impacts, hit glass or supply force-sensor data.
  Use internet sources. Preserve source attribution and applicable terms;
  unknown/incompatible redistribution terms exclude distribution.
- Generate playable media at each meaningful experiment checkpoint. Keep all
  candidates and honest failures; protocols, inventories and validators do
  not replace the audible deliverable. Latest checkpoint has new reference-free
  WAVs; supporting-only debt is zero. Keep the full goal, not only pouring.
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
- MP3s: `ps2-freesound-wine-glass-v1/research`, same author/pack/train family;
  not known identical objects. Do not infer wall thickness or force.

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
