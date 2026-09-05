# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / TEMPORAL_NOISE_DECODER_REJECTED / SYNTHESIS_DISCRIMINATOR_NEXT.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest reference-free experiment:** [temporal decoder](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-temporal-decoder-2026-09-05/first-audition/generated.wav),
  H10cm/diameter7cm/duration15s/start0.1, excitation seed2718, gain1.
  [Four-profile comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-temporal-decoder-2026-09-05/comparison.wav):
  glass10/glass16/PET10/glass10fast; base/temporal/static,54.96s. REJECTED:
  classified as noise, not water. No recording/teacher/base needed at inference.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Impacts:** prior improves1/14 matched crops; no LoRA sweep/material claim.
- **Friction:** `cluster-texture-training-grid-2026-09-05`, Figshare29438288v5/
  CC-BY4,60 records, wood0/steel65/glass74, Urethane20–60mm/s ×0.5/1N.
  Same surfaces, measured/commanded differ. Neural/interpolation1.815/1.758dB,
  11/36 wins: no capacity/epoch/basis sweeps. Clean AND machine-mic retrieval
  30/30 is not quality proof. No exact NLMS replay; details in pilot note.
- **Rain:** DataSuds10.23708/I0QYNM V2/CC-BY4.0; original CSV verified,
  converted TSV rejected. Three full WAVs; cross-site43958/43957 unfetched.
  Spectral gain7.469/7.580dB is not quality; stationary model loses temporal
  structure, AST fails real wet controls. No rain-spectrum MLP sweep; see note.
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
- **Prior phase fits:** paired93-record extension loses semantic stability
  (AST120/180; seed2718 fails60/60), not explained by decoder seed. No retry.
- **Envelope failure:**120/360 raw headroom failures, spectrum/CV worsen,
  AST88/180 vs base180/180. Stage splices do not fix semantic instability.
  No absolute-envelope sweep/guard weakening; exact runs in pilot note.
- **Prior relative controls:** level delta improves4.656->3.883 but absolute
  spectrum worsens; gain confounding unproven. Static32-band timbre metadata
  loses to global curve, EQ worsens shape/CV. No gain/static-EQ sweeps.
- **AST:** raw and RMS0.005 retained; CUDA/CPU correspondence checked, mel
  warning remains. No threshold/seed tuning. Codec checks reject gross corruption;
  ESC-50 physical attributes null. No posterior-mean/duration/CFG retries.
- **Temporal diagnostic:** classical ridge gets synthetic tones right but also
  gives smooth noise paths/positive shuffle margin. Not automatic real labels.
  Frozen Sound of Water model improves inspected real tracks but falling-tone
  median error1382 cents; direction/context dependent. Same corpus exposure,
  NOT independent-data validation. Full provenance/code in pilot note.
- **Teacher/head reuse:** checked `sound-of-water-pitch-model-2026-09-05`;
  `pouring-sow-pitch-probe-2026-09-05` full-sequence pseudo-targets, not confidence.
  `pouring-resonance-head-2026-09-05`:4993params/13 records/1000 updates,
  OOF277.5 vs simple357.0 cents,6/13 wins. Do not retrain; overlap is not clean.
- **Fixed output band rejected:** spectrum9.918->11.101dB; privileged teacher
  also worsens9.238->9.873. Predictor-only explanation insufficient; see note.
- **Input adapter rejected:**144 parameters, matched/shuffled600-step fits,
  spectrum7.655/8.026/8.086dB,0/12 wins each. Privileged teacher does not rescue
  it. Exact evidence in note; no tiny-adapter/epoch sweep or retraining.
- **Temporal decoder:** `pouring-temporal-decoder-2026-09-05`,63619 trainable
  params, frozen4993-param head,93 records/1200 steps. GRU produces65 noise-gain
  bands and resonance strength/width; FFT1024/hop256 filtered Gaussian excitation.
  Multiscale waveform reconstruction; no Griffin-Lim or reference input. Synthetic
  moving-filter control improves2.807->1.323 loss vs static1.819; not real quality.
- **Real rejection:** two disclosed objects/first records/two phases/three seeds.
  Base/temporal spectrum7.655/8.256, centered5.990/5.067, CV error0.441/0.791.
  Temporal AST0/12 development and0/12 novel at raw AND normalized level; CLAP
  novel0/12 vs base12/12 and real4/4. Do not promote from centered spectrum.
- **Single-record probe:** `pouring-temporal-decoder-fit-check-retry-2026-09-05`,
  first training record/container1,300 updates. Centered error3.585->2.136,
  CV error0.840->0.390, but AST0/6 before AND after vs real2/2. Memorization only.
  First `...fit-check-2026-09-05` failed before optimizer step1 (eval-mode cuDNN
  GRU);8 before/real WAVs retained and labelled partial. Fixed train mode; both
  fits terminal, weights preserved. Do not rerun either completed fit.
- **Next action:** exact-record magnitude with Gaussian-noise phase versus
  iterative phase reconstruction, same FFT1024/time grid, first training record
  only. Produce labelled oracle WAVs and use existing semantic checks to separate
  missing spectral detail from excitation/phase limitations BEFORE another fit.
  Current decoder has no learned stochastic event latent; cause not yet isolated.
- **Verification:**86 tests, Ruff,127 WAVs checked; source-free CLI exact replay.
  Frozen head/weights verified. All jobs terminal;
  no Cargo/ProductCheck/engine audition or default/runtime promotion.
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
