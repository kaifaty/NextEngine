# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / RELATIVE_LEVEL_LIMITED_GAIN / STATIC_TIMBRE_CORRECTION_REJECTED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifacts:** [source-free pouring audition](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-relative-level-audition-2026-09-05/generated.wav),
  glass H10cm/diameter7cm/duration15s/progress0.2, seed2718/decoder314, gain10.
  [Timbral comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-relative-timbre-2026-09-05/comparison.wav)
  is FAILED: glass then PET middle; real/base/global/conditioned/shuffled,
  seed2718, gain1. No target recording enters neural inference/correction.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Impacts:** prior improves only1/14 matched crops; retain base, no LoRA sweep.
  Extraction evidence in pilot note. Striker margins are not material proof.
- **Friction source:** `cluster-texture-training-grid-2026-09-05`,
  Figshare29438288v5/CC-BY4,60 records: wood0/steel65/glass74, Urethane probe,
  20–60mm/s ×0.5/1N. Same surfaces, not new objects; measured/commanded differ.
- **Crossed velocity:** rank4/4804params, pooled neural/interpolation spectrum
  1.815/1.758dB,11/36 wins. Stop three-surface capacity/epoch/basis tuning;
  40mm/s gain was not robust. Disclosed development; stationary texture, not impact.
- **Friction countercheck:** clean AND machine-mic retrieval30/30, not quality
  validation or proof of noise-only generation. No exact NLMS replay; see note.
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
  wins13/13, PET correct17/17. Endpoint penalty is not proven cause; see note.
- **Prior phase controls:** power wins151/180 spectrum pairs, middle CV worsens.
  Base/power miss phase CV change on new AND13 training objects; oracle does not.
- **Two-patch probe:** first training record/container1, matched/shuffled600-step
  fits; correct template6/6 versus parent/shuffled3/6. Phase is learnable here.
  Noisy target identifies phase for98% of uniform times in exact two-endpoint
  toy; possible shortcut, not a measurement of what the network learned.
- **Paired extension:** same93 train records,600 extra updates, all30 disclosed
  development recordings ×2 phases ×3 seeds. Spectrum wins158/180 but CV worsens;
  normalized AST120/180 vs parent180/180. Seed2718 fails all60; reject promotion.
- **Independent checks:** crossed decoder seeds do not explain full failure.
  CLAP base12/12, paired10/12 water; PET2718 favours birds. Neither judge is
  naturalness authority; AST/CLAP disagreement retained in pilot note.
- **Stage/envelope failures:** exact-endpoint oracle succeeds, stage splices
  do not cleanly restore semantics. Separate32-bin envelope flows fail120/360
  raw headroom checks, worsen spectrum/CV, normalized AST88/180 vs base180/180.
  Exact runs retained in pilot note; no absolute-envelope sweeps/guard weakening.
- **Relative level:** ridge head on same93 paired records learns only global
  phase slope-25.2779dB. Delta RMSE4.656->3.883 on two disclosed objects;
  absolute fit3.829, training mean3.981. Gain confounding NOT established.
  Absolute spectrum worsens9.923->11.177; CV unchanged. AST normalized180/180,
  CLAP12/12 preserved. Limited amplitude control, not new neural/timbral quality.
- **Relative timbre:**32-band normalized paired spectra, ridge0.01. Excluding
  each13 training object from head fitting: conditioned/global mean RMSE
  3.956/3.867; disclosed two objects3.445/3.188. Metadata adds no robust gain.
  Generated EQ worsens absolute shape5.060->5.605dB and CV; normalized AST
  still90/90. Retain base, reject static EQ; no additional gain/EQ sweeps.
- **AST:** raw and RMS0.005 retained; CUDA/CPU correspondence checked, mel
  warning remains. No threshold/seed tuning. Codec checks reject gross corruption;
  ESC-50 physical attributes null. No posterior-mean/duration/CFG retries.
- **Next action:** test time-resolved resonance on existing training recordings
  against synthetic rising/falling and shuffled-time controls, with a playable
  reconstruction/control. Bagad et al.v1 sections3–4/6.1 motivate this but also
  show generic pitch detectors fail; spectral argmax is not ground truth.
  Static-color failure does not prove labels lack all physical information.
  No invented liquid heights, protected reuse or new static-head sweep.
- **Verification:**67 focused tests, Ruff and1323 WAV/hash checks pass;
  raw/normalized AST complete. No jobs running. Cargo/ProductCheck/engine
  audition not run: external Python lab only, no runtime promotion.
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
