# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / IMPACT_HYPOTHESES_TESTED / CONTROLLED_CODEC_NEXT.

## Resume in 60 seconds

- **Full user goal:** a neural system generates realistic impacts, friction,
  rolling, destruction, water and rain from descriptions of objects/events.
  Sound depends on both materials, shape/size, force/speed and flow/rain
  intensity. It must generate new combinations without a target recording,
  learn from internet data, improve through automatic training/validation
  without per-sound human approval, and eventually supply engine-usable sound.
  Reconstructing an input recording does not satisfy this objective.
- **Discrete-impact rule:** retain the FULL Tango decode and apply the existing
  matched `event_window` to base/candidates. Never revert to prefix-only scoring
  or amplify codec noise. Seed314 can start after11s despite a3s request. This
  known bug was repeated when absent from compact state; corrected without fit.
  Event detection is not quality acceptance; continuous water/rain differ.
- **Latest source-free control:** [fixed material-text matrix](/home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-material-text-control-2026-09-05/comparison.wav),
  71.552s,real/generic-base/material-text/wrong-text for six pairs,seed2718. Bridge
  OFF; six already-frozen prompts, no new training/wording/seed search. Shape7.694
  vs bridge5.946, wins1/14, all-pair top1 3/14. AST metal/glass leads Glass/Clink,
  others inconsistent. Spectrum to unmatched objects is NOT sole quality truth.
- **Signal audit:** `epic-impact-signal-summary-2026-09-05` corrects preview assembly
  only; original477-case report/100WAVs retained.154TRAIN×3sigma FP32 plus15 BF16
  controls. Attack/body improve.000525/.000507; padded tail worsens.00000674.
  Correct condition beats ALL wrong only73/462 attack,77/459 body. Silence-learning
  hypothesis contradicted; no new attack/tail-weight sweep. FP32/BF16 same15-case
  rank1 counts2/15 attack,3/15 body: no precision rescue or precision-training sweep.
  Source-aided one-step preview is diagnosis, NOT source-free generation.
- **Latest learned impacts:** [real/previous/expanded](/home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-expanded-pair-compare-2026-09-05/comparison.wav),
  six unordered pairs,fixedseed2718. Expanded source154TRAIN/19participants,
  same10240-parameter bridge/200updates,113 distinct sampled rows. All wood/glass
  remains excluded from generator training; same7dev/P04/P07. Data frequencies
  and centering change; not isolated proof of participant-count or compute effect.
  All14 cases retain prefix/full29.9537s/onset3s; windows are bounded, not full events.
- **Impact result:** shape new5.945757 vs previous5.962152/base5.975546dB;
  wins8/14 vs previous,7/14 vs base/wrong. All-pair top1 only3/14 vs previous2/14.
  Heldwood/glass worsens7.028456→7.080724,ranks4,2,4,1. No reliable material control.
  Raw AST remains Breaking/Smash(seed314),Door(2718),including base. No replacement.
  After two bridge cycles: NO next data-size/capacity/epoch/seed sweep.
- **Impact reproducibility:** `physical_sound_epic_pair_bridge.py --model PATH
  --pair 'wood / glass collision' --seed 314 --output NEW` defaults to full/event;
  `--event-matrix` evaluates all14 without fitting. Zero/upstream-loss/full-model/
  water+rain/old-base guards and CLI PCM exact; identities/checks in pilot note.
  Old prefix evaluations are superseded; never score their codec-noise prefixes.
- **Retained reference-free result:** [base/full/centered bridge comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),
  13.74s,glass10/seed2718,published PCM. Centered frozen-generator bridge retains
  water (raw/RMS-controlled AST and harder CLAP8/8), not physical calibration.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Earlier impacts:** generic LoRA improves1/14 matched crops; no LoRA sweep.
- **Friction:** Figshare29438288v5/CC-BY4,60 records; neural/interpolation
  1.815/1.758dB,11/36 wins. No capacity/epoch/basis sweeps; details in note.
- **Rain:** DataSuds10.23708/I0QYNM V2/CC-BY4.0; CSV verified,TSV rejected.
  Stationary model loses temporal structure, AST fails real wet. No MLP sweep.
- **Pouring source:** `sound-of-water-source-2026-09-05`, Bagad et al., local research
  only; dataset terms unspecified, NOT software MIT. Author tests unopened;
  containers18/30 excluded. Approximate flow is not measured ml/s/level. See note.
- **Closed pouring families:** STFT/envelope/EQ, pitch heads, CVAE/critic,
  scratch flow/affine coupling and solver/noise/phase retries do not transfer.
  Full279TRAIN audit gives scratch AST11/hardCLAP6/both2; target injection is not
  source-free. Native645x64 Tango posteriors differ from old normalized88 cache.
  Keep the harder13-prompt diagnostic and raw/RMS0.005 AST; don't tune thresholds.
- **Retained water bridge:** posthoc TRAIN-mean centering removes98.397% common
  correction and restores AST/raw/RMS/hardCLAP8/8, but material swap still fails.
  Differentiable centering, audio-AdaLN modulation and separate setting branch
  did not improve transfer. No13-container capacity/epoch/layer/setting/seed sweeps.
  Water CLI/profile/weight identities and failed output receipts are in the note.
- **Water information:** ridge fails new-object transfer; setting association is
  not measured room causality. No physical/level calibration; details in note.
- **Preserved guards:** publish exact PCM, never weaken.98 headroom.
  `requires_grad=False` is required for exact frozen-generator replay despite
  no_grad; no kernel-cause claim. Keep failures rather than overwrite/retry green.
- **EPIC source/probe:** `epic-information-2026-09-05`,161TRAIN/19participants/84videos,
  CC-BY-NC4 local research; P04/P07 excluded,no author val/test. Only7 wood/glass.
  No object IDs/striker/geometry/force/velocity. Fixed leave-participant-out C1 probe
  macro recall AST24.30%,RMS20.11%,spectrum25.47%,duration/gain21.45%; glass pair0/7
  for AST.32label permutations mean16.53%,max21.72%. NOT qualified judge/reward;
  no threshold/prompt tuning. Exact selection, hashes, source previews in note.
- **Pair validator:** frozen CLAP4/24 on six fixed prompts (chance expectation4).
  Not qualified as sole material validator/reward; don't tune prompts or drop
  failures. One sequential-vs-seek decode matches except4 one-LSB samples.
  Whole remote MP4 MD5 unverified (partial access), local WAV SHA checks pass.
- **Next:** controlled friction grid codec feasibility: does frozen Oobleck preserve
  paired speed/load response? Use known commanded/measured fields separately,
  clean/main/machine controls and ONE disclosed shared gain. No new source needed.
  Publish audible codec comparison plus reference-free controlled examples. Do not
  repeat stationary-PSD/PCA/crossed-speed/epoch variants or EPIC bridge/prompt sweeps.
  See the focused friction sections in the pilot note before implementing.
- **Verification:** tests/Ruff/PCM checks in note; all jobs terminal, no promotion.

## Preserve these constraints

- The user will not record impacts, hit glass or supply force-sensor data.
  Use internet sources. Preserve source attribution and applicable terms;
  unknown/incompatible redistribution terms exclude distribution.
- Generate playable media at each meaningful experiment checkpoint. Keep all
  candidates and honest failures; protocols, inventories and validators do
  not replace the audible deliverable. Latest checkpoint adds learned impact WAVs;
  supporting-only debt is zero. Keep the full multi-event goal.
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
