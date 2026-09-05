# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / TRAIN_CENTERING_REJECTED / MODULATION_INTERFACE_NEXT.

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
- **Latest trial rejected:** [real/old/new/swapped comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-center-trained-development-2026-09-05/comparison.wav),
  36.64s,glass18/PET30,middle,seed2718. Training-time centering retains water,
  but shape error8.106 versus old7.687dB; wins2/8. No model replacement.
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
- **Pouring base:**245985param conditional STFT flow,53/1500updates,64 Euler/
  32 reconstruction,16k/FFT512/hop256/256² patches. Retained, no promotion.
- **Prior checks:** STFT256 Euler no rescue; wrong glass material wins13/13.
- **Prior STFT paths:** paired extension AST120/180; envelope120/360 unsafe;
  splices,relative level/static timbre/EQ fail. No phase/gain/EQ/envelope sweep
  or guard weakening; exact experiments and confounds in pilot note.
- **AST:** raw and RMS0.005 retained; CUDA/CPU correspondence checked, mel
  warning remains. No threshold/seed tuning. Codec checks reject gross corruption;
  ESC-50 physical attributes null. No posterior-mean/duration/CFG retries.
- **Pitch/head:** teacher falling-tone error1382cents, same corpus; moving-band,
  144param and temporal-noise paths fail. No head/filter/noise/phase sweeps.
- **CVAE:** posterior/prior and rec/critic continuations fail; phase2/32 no rescue.
  No CVAE/critic/capacity/epoch/phase sweeps; exact evidence in pilot note.
- **Codec:** cached TangoFlux rev367005e9/Oobleck SHA d73619a1;16k resampled
  44.1k dual mono,64x88 latent. Initial six controls mostly pass; all-TRAIN below.
- **Cache:** `pouring-oobleck-cache-2026-09-05`,279TRAIN posteriors; normalization
  includes posterior variance. Earlier original/plain/skip2000 dev AST0/12;
  don't repeat. Target-injected late trajectories are NOT source-free; see note.
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
- **Development:** first source-order recording of each excluded container18/30,
  first/middle,seeds314/2718. Raw/RMS AST and harder CLAP8/8,real4/4,base2/2.
  Matched shape RMSE7.687 versus base7.756/swapped7.813dB; wins5/8 and6/8.
  Marginal, correlated evidence; no calibrated response/generalization claim.
  Joint object-control swap does not isolate material; level also uncalibrated.
- **Preserved export failures:** use exact published PCM, never loosen .98 guard.
  Generator requires_grad=False is needed for exact replay despite no_grad;
  no kernel-cause claim. Failed reports/WAVs retained; see note.
- **Signal:** `pouring-tango-bridge-signal-2026-09-05`,first TRAIN recording per13
  objects,middle,sigmas.2/.5/.8,paired posterior/noise. Common-only explains98.35%
  of active improvement; tail worsens. Silence-dominated learning falsified here.
- **Training centering:** `pouring-tango-bridge-center-trained-2026-09-05`,same
  200steps/cache/draws, differentiable mean across279controls eachstep. Active
  matched-vs-swapped gain0.003654,22/39 wins, but worse disclosed dev above.
  Raw/RMS AST and harder CLAP8/8 hypothetical and dev; semantic-only success.
  CLI automatically loads frozen mean without TRAIN bank; hashes in pilot note.
- **Next:** no more global text/pooled centering/epoch/seed variants. Inspect
  frozen Tango audio-layer modulation; zero-initialized residual feasibility,
  zero/bypass exactness then condition-swap and playable output before largerfit.
  PAVAS v2 residual AdaLN prior art/limits in note: it trains diffusion blocks,
  code pending; not a drop-in or evidence our frozen tiny model must succeed.
- **Verification:**138 tests, Ruff, new automatic-offset CLI exact;64 new WAVs
  PCM/hash audited. All jobs terminal. No further training started.
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
