# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / EPIC_LEARNED_WAVS / MATERIAL_CONTROL_UNPROVEN.

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
- **Latest learned impacts:** [real/base/learned/wrong pair](/home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-pair-event-matched-2026-09-05/comparison.wav),
  71.552s, six unordered pairs,fixedseed2718. [Wood/glass seed314](/home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-pair-event-matched-2026-09-05/pair2-314.wav)
  uses a pair excluded from bridge training; NOT proven physical generalization.
 45TRAIN clips/P01–03,7dev/P04/P07,10240-factorized parameters,200updates.
  All14 cases retain prefix/full29.9537s/onset3s WAVs. Later activity exists in
  every full horizon; crops are bounded attack windows, not complete events.
- **Impact result:** shape base5.975546/matched5.962152/wrong5.964974dB;
  wins8/14 vsbase,5/14 vsnext pair. Heldwood/glass4/4 wins against those controls,
  but ALL-six-material ranking gives correcttop1 only2/14 (heldranks1,2,2,3).
  No reliable material control. Raw AST leads Breaking(seed314)/Door(2718),
  including base; not a qualified pair judge. No bridge capacity/epoch/seed sweep.
- **Impact reproducibility:** `physical_sound_epic_pair_bridge.py --model PATH
  --pair 'wood / glass collision' --seed 314 --output NEW` defaults to full/event;
  `--event-matrix` evaluates all14 without fitting. Zero/upstream-loss/full-model/
  water+rain guards exact; CLI event/prefix/full byte-exact in both formats.
 98 corrected WAVs verified; bridge/hash/source identities in pilot note.
  Original `epic-pair-bridge-2026-09-05` prefix evaluations are superseded,
  preserved unchanged; do not reuse their quiet-seed metrics for quality claims.
- **Retained reference-free result:** [base/full/centered bridge comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),
  13.74s,glass10/seed2718,published PCM. Centered frozen-generator bridge retains
  water (raw/RMS-controlled AST and harder CLAP8/8), not physical calibration.
  References enter metrics only. No promotion or source/cache at generation.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Latest trial rejected:** [real/old/new/swapped/style-only](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-setting-development-2026-09-05/comparison.wav),
  45.8s,glass18/PET30,middle,seed2718. Separate recording-setting branch retains
  water but shape8.025 versus old7.692/style-only7.817dB. No replacement.
- **Earlier impacts:** generic LoRA improves1/14 matched crops; no LoRA sweep.
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
- **Water information:** within-object ridge3.630 vs global4.390dB; leave-object
  5.351 vs4.601. Setting-only4.062, correlated not measured room/mic causality.
  No precise physical/level calibration, evaluator acceptance or new-object claim.
- **Preserved guards:** publish exact PCM, never weaken.98 headroom.
  `requires_grad=False` is required for exact frozen-generator replay despite
  no_grad; no kernel-cause claim. Keep failures rather than overwrite/retry green.
- **New source:** [EPIC real-pair preview](/home/kaifaty/.codex/experiments/nextengine/physical-sound/epic-material-pairs-source-fixed-2026-09-05/comparison.wav),
  37.297s,24 real TRAIN clips,6 unordered material pairs,19 videos/5 participants.
  Four clips each: metal/glass,metal/wood,wood/glass,metal/plastic,metal/ceramic,
  plastic/wood. NOT neural generation. Metadata revision57a922f0; full identities,
  fixed source-order/no-overlap selection and runnable acquisition in pilot note.
  CC-BY-NC4, noncommercial research only. No validation/test acquisition.
  Object IDs/striker/geometry/force/velocity unknown; don't invent them.
- **Pair validator:** frozen CLAP4/24 on six fixed prompts (chance expectation4).
  Not qualified as sole material validator/reward; don't tune prompts or drop
  failures. One sequential-vs-seek decode matches except4 one-LSB samples.
  Whole remote MP4 MD5 unverified (partial access), local WAV SHA checks pass.
- **Next:** discriminate poor transferable material information in kitchen labels
  from weak bridge transfer: broaden TRAIN participant coverage and test a
  participant-separated real-positive/wrong-label control before another fit.
  Next learned trial retains playable/all-pair comparisons. No tuning on current
  dev, no material-validator claims from CLAP. Stop13-container adapter family.
- **Verification:** focused Python tests/Ruff and exact replay; details in note.
  All jobs terminal. No runtime/default/ProductCheck promotion; full goal open.

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
