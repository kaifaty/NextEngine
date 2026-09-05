# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / PHASE_SIGNAL_LEARNABLE / PAIRED_RECORD_PROMOTION_REJECTED.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Latest primary artifacts:** [two-phase learning comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-two-phase-probe-2026-09-05/comparison.wav),
  36.64s, one TRAINING recording, first/middle real/parent/matched/shuffled.
  [Paired-record candidate](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-flow-paired-records-audition-2026-09-05/generated.wav)
  is source-free glass10cm/7cm,15s/fraction0.2/seed2718/gain10. Retained FAILURE
  candidate, not promoted. Full comparisons/other seeds in pilot note.
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
- **Two-patch probe:** `pouring-flow-two-phase-probe-2026-09-05`, first training
  recording/container1/plastic, two fixed phases. Matched/shuffled600-step fits
  from identical parent, same noise/time draws. Correct template matched6/6,
  parent/shuffled3/6 across3 seeds. Phase input is learnable on this example.
  Gaussian two-endpoint calculation: noisy target alone permits >95% phase
  identification for98% of uniform flow times; possible shortcut, not proof.
- **Paired extension:** `pouring-flow-paired-records-2026-09-05`, same93 train
  recordings,3 paired records/batch, matched/shuffled600 extra updates. All30
  disclosed development recordings ×2 phases ×3 seeds. Spectrum matched wins
  158/180 vs parent; mean level change-3.050dB vs parent-2.585/shuffled-0.486,
  real-5.390. CV errors worsen; phase CV change-0.017 vs real-0.297.
  Normalized AST matched120/180 vs parent180/180; seed2718 fails all60 cases.
- **Adjacent-layer/independent checks:** crossed generator/decoder seeds on
  first glass/PET, both phases: matched generator2718 passes AST1/12 across
  decoders, base36/36 total. Not solely decoder randomness. CLAP32-clip check:
  real/oracle8/8 and base12/12 water, matched10/12; PET2718 both phases favour
  birds, margins worsen11/12 paired cases. Glass2718 remains water in CLAP;
  judges disagree, not calibrated naturalness/material acceptance.
- **AST controls:** `--ast-rms .005` optional; raw preserved. CUDA matches CPU
  top10/flags on180 raw+120 normalized WAVs, delta2.24e-6. CPU default, CLAP
  unchanged; mel warning remains. Do not tune thresholds or blacklist seeds.
- **Prior controls:** codec checks reject gross corruption; ESC-50 physical
  attributes are null. No posterior-mean, duration/CFG or caption-threshold retries.
- **Next action:** inspect per-flow-time training error/phase ablation near pure
  noise, with two-endpoint oracle control, before another full fit. Preserve
  parent/power; paired-record candidate is not an all-round improvement. No
  generic capacity/epoch/loss sweep, new source/stack, invented labels or protected
  reuse. Broad realism/control goal remains unchanged; keep playable artifacts.
- **Verification:** four600-step fits, generation, GPU AST and CLAP complete.
  Inherited93-record exposure stays in `train_ids`; `finetune_ids` distinguishes
  one-record and93-record probes. Metadata clarified, weights/WAVs unchanged.
  54 focused tests, Ruff and773 WAV/hash checks pass; no jobs left running.
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
