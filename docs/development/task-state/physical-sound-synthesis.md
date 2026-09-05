# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / SETTING_BRIDGE_REJECTED / BROADER_OBJECT_DATA_NEXT.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Retained reference-free result:** [base/full/centered bridge comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),
  13.74s,glass10/seed2718,published PCM. Centered frozen-generator bridge retains
  water (raw/RMS-controlled AST and harder CLAP8/8), not physical calibration.
  References enter metrics only. No promotion or source/cache at generation.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Latest trial rejected:** [real/old/new/swapped/style-only](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-setting-development-2026-09-05/comparison.wav),
  45.8s,glass18/PET30,middle,seed2718. Separate recording-setting branch retains
  water but shape8.025 versus old7.692/style-only7.817dB. No replacement.
- **Impacts:** prior improves1/14 matched crops; no LoRA sweep/material claim.
- **Friction:** Figshare29438288v5/CC-BY4,60 records; neural/interpolation
  1.815/1.758dB,11/36 wins. No capacity/epoch/basis sweeps; details in note.
- **Rain:** DataSuds10.23708/I0QYNM V2/CC-BY4.0; CSV verified,TSV rejected.
  Stationary model loses temporal structure, AST fails real wet. No MLP sweep.
- **Pouring source:** `sound-of-water-source-2026-09-05`, Bagad et al.,
  HF `bpiyush/sound-of-water` revision12575460ee39d6adaebbe5aff531a5f4a24a627b.
  Dataset redistribution unspecified; do NOT inherit separate software MIT.
  Local research only.123 verified full48kHz WAVs, annotation-only clean/constant/
  water selection. Author Test I/II/III and YouTube not used; no foreign code run.
  93 train recordings/13 objects; whole containers18(glass13),30(PET17) excluded.
  Approximate constant flow is not measured ml/s or exact liquid level.
- **STFT base:** retained, not promoted; wrong glass material wins13/13.
- **STFT failures:** splices/envelope/relative level/static timbre/EQ; no phase/
  gain/EQ/envelope/step sweep or guard weakening. Exact configurations in note.
- **AST:** raw and RMS0.005 retained; CUDA/CPU correspondence checked, mel
  warning remains. No threshold/seed tuning. Codec checks reject gross corruption;
  ESC-50 physical attributes null. No posterior-mean/duration/CFG retries.
- **Pitch/head:** teacher falling-tone error1382cents, same corpus; moving-band,
  144param and temporal-noise paths fail. No head/filter/noise/phase sweeps.
- **CVAE:** posterior/prior and rec/critic continuations fail; phase2/32 no rescue.
  No CVAE/critic/capacity/epoch/phase sweeps; exact evidence in pilot note.
- **Old codec cache:** `pouring-oobleck-cache-2026-09-05`,279TRAIN posteriors64x88,
  normalization includes variance. Original/plain/skip2000 dev AST0/12; no repeat.
  Target-injected trajectories are not source-free. Codec identity in pilot note.
- **Scratch flow rejected:** single-crop control learns water but full279 TRAIN
  coverage of10000-step model gives AST11/harder CLAP6/both2. Affine2000 coupling
  gives AST/harder CLAP0/30. Solver Euler64/midpoint128/256 does not rescue it.
  No mostly-unused-channel/PCA/mean-mode, epoch/seed/coupling/step sweeps; see note.
- **Evaluator confound:** six-prompt CLAP hides scratch/crunch. Keep the disclosed
  harder13-prompt diagnostic and raw/RMS0.005 AST; no threshold/prompt retuning.
- **All-TRAIN audit:** `pouring-training-codec-audit-2026-09-05`,279 real/posterior
  pairs. Raw AST252/216; harder CLAP267/232. Codec loses35 source positives,
  especially late phases, but most targets retain water. No filtering/codec fit.
- **Bridge:** `pouring-tango-bridge-2026-09-05`,265728params,200steps,seed53,
  unchanged TangoFlux367005e9/T5/codec. Native645x64 TRAIN posteriors, NOT old
  normalized88 cache. Exact bridge/offset hashes in pilot note and result.
  Full bridge rejected: raw AST4/8,RMS-controlled3/8,harder CLAP1/8.
- **Centering:**98.397% of correction energy is common TRAIN mean. Subtracting
  that fixed mean (all279controls, no refit/sweep) restores all three8/8 checks.
  CLI `physical_sound_pouring_bridge.py render --model BRIDGE --offset CENTERED
  --controls <11 floats> --output NEW` byte-replays both formats; source-free.
- **Development:** first source-order18/30,first/middle,seeds314/2718. Retained
  centered model has marginal gains only; joint control swap does not isolate
  material. Correlated evidence, no physical/level/generalization claim.
- **Preserved export failures:** use exact published PCM, never loosen .98 guard.
  Generator requires_grad=False is needed for exact replay despite no_grad;
  no kernel-cause claim. Failed reports/WAVs retained; see note.
- **Signal:** `pouring-tango-bridge-signal-2026-09-05`,first TRAIN recording per13
  objects,middle,sigmas.2/.5/.8,paired posterior/noise. Common-only explains98.35%
  of active improvement; tail worsens. Silence-dominated learning falsified here.
- **Training centering:** `pouring-tango-bridge-center-trained-2026-09-05`,same
  200steps/cache/draws, differentiable mean across279controls. Larger conditional
  TRAIN signal but worse dev8.106 vs7.687dB; rejected. No centering/epoch sweep.
- **Audio modulation:** `pouring-tango-audio-modulation-2026-09-05`,264696params,
  six audio AdaLN mixers,200steps. Zero/bypass/full-model exact; waveform transfer
  worse0/8 versus old. PAVAS-inspired placement, not its full-backbone training.
  No layer/width/seed/epoch sweep. CLI auto-detects kind; hashes in pilot note.
- **Comparison:** runnable `physical_sound_pouring_bridge_compare.py`, fixed
  excluded-object/crop/seed roles, previous/base/new/swapped plus playable WAV.
- **Metric confound:** old fixed-floor shape metric varies under pure gain.
  New `relative_power_shape_rmse_db` passes gain control; rejection survives.
  Historical fields/reports unchanged, no audio/EQ/gain adjustment.
- **Data audit:** `pouring-control-information-2026-09-05`,all279TRAIN crops,
  absolute profiles, ridge0.01. Leave-record ridge/global3.630/4.390dB; leave-object
  5.351/4.601. Useful same-object signal, no transfer for this simple baseline;
  not proof nonlinear prediction is impossible. No fit/selection from dev.
- **Setting audit:** `pouring-setting-nuisance-2026-09-05`,TRAIN7/31/40.
  Same-object cross-setting distance5.743 vs within3.731dB; correlated observations,
  NOT measured room/mic causality. Setting-only beats controls in leave-object
  ridge4.062/5.351. Details/source preview in note; no author tests reopened.
- **Setting bridge:** `pouring-tango-setting-bridge-2026-09-05`,273920params,
  same200steps/posteriors/draws; centered physical branch +4 learned setting rows.
  Target setting FIXED during wrong-physical controls; physical-disabled ablation.
  Matched beats old2/8 and style-only2/8. Glass slightly better, PET worse.
  Raw/RMS AST/hard CLAP8/8 for each variant. ws-room only1 TRAIN record/update.
  No setting/centering/capacity/seed/epoch sweep. Exact identities in pilot note.
- **Next:** bounded internet search for broader object/material-pair coverage,
  defensible physical descriptors and recording context. Acquire one small
  permitted TRAIN slice + playable examples, then one data-backed learned trial.
  Stop this small-corpus adapter family; don't narrow the goal to pouring.
- **Verification:**114 focused tests, Ruff, setting CLI both formats byte-exact;
  92 new WAVs +10 prior audit WAVs PCM/hash verified. All jobs terminal.
  No runtime/default/ProductCheck promotion; full multi-event goal remains open.

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

- **Liked:** [three glass reconstructions](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-first-2026-09-04/comparison.wav), no generalization.
- **Frozen transfer failure:** [unseen result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-unseen-2026-09-04/result.json).
  Nearest old parameters beat the MLP on 9/9 later strikes, not new objects.
- **Automatic cycle:** [cycle result](/home/kaifaty/.codex/experiments/nextengine/physical-sound/audible-glass-cycle-2026-09-05/result.json).
  16 train/11 later strikes; expanded/anchored MLPs lose to analytic control.
- No parameter-MLP size/epoch retries: sparse data/nonunique targets/loss mismatch
  unresolved; audio-to-parameter reconstruction is also the wrong goal interface.
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
