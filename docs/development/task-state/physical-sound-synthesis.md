# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / POURING_PHASE_CONTROL_NOT_LEARNED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifact:** [middle-pour real/base/power comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-phase-seeds-2026-09-05/middle-comparison-seed314.wav),
  27.48s/16kHz, first source-order glass then PET, gain1. Same-root seeds2718/1618
  comparisons also retained. No recording enters neural inference. Full30 cases
  ×2 phases ×3 seeds ×2 models, real/oracle controls; not physical admission.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Impacts:** prior improves only1/14 matched event crops; retain base, no LoRA
  sweep. Prefix/old unmatched-empty failures are superseded; exact extraction
  boundary and evidence in pilot note. Striker margins are not material proof.
- **Physical-control source:** `cluster-texture-training-grid-2026-09-05`,
  Figshare29438288v5, CC-BY4.0,60 records/242 files/86.69MB: wood0/steel65/glass74,
  Urethane probe,20–60mm/s ×0.5/1N. Same surfaces, not new objects. Keep
  measured/commanded controls separate. Acquisition details in pilot note.
- **Crossed-velocity result:** fixed rank4/4,804params/1500steps. Neural versus
  interpolation spectrum RMSE30:1.904/1.759;40:1.690/1.729;50:1.850/1.788dB.
  Pooled1.815/1.758,11/36 wins. Stop this three-surface capacity/epoch/basis
  tuning; the apparent40mm/s gain is not robust. All folds disclosed development.
  No pretraining; physical inputs -> stationary Gaussian texture, not impacts.
- **Friction countercheck:** clean AND machine-mic surface retrieval30/30;
  not independent quality validation, nor proof of a noise-only generator.
  No exact NLMS preprocessing replay; audit JSON in speed50 root.
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
- **Pouring fits:**245985params conditional STFT flow U-Net, seed53/1500updates,
  batch6/lr3e-4,64 Euler/32 phase-reconstruction steps. Fixed16k/FFT512/hop256,
  256×256 patches; material, shape, dimensions, duration, elapsed fraction inputs.
  Original first4.08s/seed314 evaluation now extended to2 phases and3 seeds.
  Base spectrum10.903dB; power/envelope9.581,28/30 gains; onset-balanced8.195.
  Power candidate median level-7.441dB versus base-8.995; CV error0.327 versus
  0.579. Extra0.25*t² endpoint statistic; no default change or runtime promotion.
- **Causal checks:** base64/256 integration steps do not fix its deficit.
  Material-only swaps favour wrong glass label13/13, including power-candidate
  centered spectra; PET correct17/17. No reliable material-identity claim.
  Toy endpoint penalty changes flow optimum, not proven cause of audible error.
- **Phase/seed check:** `pouring-flow-phase-seeds-2026-09-05`, all30 recordings,
  first/middle ×314/2718/1618. Power spectrum improves all6 groups,151/180 wins;
  centered shape improves only1618. Middle CV error slightly worsens0.224->0.230.
  Mean middle-minus-first CV: real-0.297,oracle-0.268,base+0.007,power+0.014.
- **Training-side control:** `pouring-flow-training-phase-control-2026-09-05`,
  first source-order recording from13 training objects, same phases/seeds.
  Mean CV change real-0.573,oracle-0.523,base+0.021,power+0.029. The temporal
  failure is NOT solely unseen-object transfer; representation preserves most
  of the change. No unique cause established; full values in pilot note.
- **AST:** normalized Water/Pour top5 succeeds for all360 development and156
  training-generated clips despite that failure. Real/oracle development60/58
  of60, training26/23 of26. Coarse identity is not naturalness/physical control.
  `--ast-rms .005` remains optional, logs gain; raw scores retained. `--device
  cuda` matches CPU top10 order/flags on180 raw+120 normalized identical WAVs,
  max score delta2.24e-6. CPU default, CLAP unchanged; mel warning remains.
- **Prior controls:** codec checks reject gross corruption; ESC-50 physical
  attributes are null. No posterior-mean, duration/CFG or caption-threshold retries.
- **Next action:** bounded research, then small known-object first/middle
  conditional-fit discriminator versus shuffled phase and exact-spectrogram
  controls. Distinguish weak learned conditioning/optimization from insufficient
  phase information; oracle and training checks constrain the hypotheses.
  No generic capacity/epoch/loss sweep, source/stack search, AST threshold tuning,
  modal-MLP restart, invented labels or protected reuse. Keep playable outputs.
- **Verification:** phase/seed and training-control generation plus GPU tags
  complete;51 focused tests, Ruff and694 WAV checks pass. Slow duplicate CPU
  jobs deliberately terminated after CPU/GPU parity check; no jobs left running.
  No Cargo/ProductCheck or engine audition: external Python lab only.
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
