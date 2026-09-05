# Physical sound synthesis — current task state

Updated: 2026-09-05. Working context, not architecture authority.
Status: ACTIVE_GOAL / SYNCFUSION_AUDIO_CONDITIONING_GAIN / MATERIAL_UNVERIFIED.

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
- **Latest media:** [glass comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/syncfusion-condition-discriminator-2026-09-05/glass-comparison.wav),
  [wood comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/syncfusion-condition-discriminator-2026-09-05/wood-comparison.wav):
  2real TRAIN events→text→mean audio prototype→individual audio condition.
  New4WAV are reference-aided diagnostic oracles,NOT source-free generalization.
  Same seed42/150steps/FP32/full5.46s/shared.5gain;16/16attacks,0extras,0weightupdates.
  200ms spectral distance text→prototype glass9.698→4.583,wood8.259→3.304dB;
  individual also improves. Glass still slightly nearer wood templates;no quality
  or material admission. AST fails real references too;NOT qualified judge/reward.
  Evidence motivates learned text/audio conditioning adapter,not generator sweep.
  Source`syncfusion-train-prototypes-verified-2026-09-05`:first2glass/wood recordings,
  105TRAIN annotations visited,author train_shard_1MD5 verified,SHA7284c9dd…;
  CC-BY4/Zenodo12634671. Initial range scan429 terminal;full fetch recovery done.
  No author val/test;29tests,all jobs terminal. Exact data/CLI/proof in pilot note.
- **Previous source-free timing:**`syncfusion-explicit-times-2026-09-05`:text+times,
  12/12attacks. Empty schedule peak2.319/RMS.0328 rejected,not silence;still open.
  Assets`syncfusion-assets-2026-09-05`,SHAa25584b1…;no blanket production clearance.
- **MMAudio impact:** video2/9,delayed3/8timing matches;no prompt/seed sweep. Apple
  CLIP research-only excludes product development;NOT engine candidate weights.
- **Earlier MMAudio water:** coarse Water top5,not quality win;CLI/results in note.
  Initial headroom failure terminal;use`water-generated-full.mp4`,not short mux.
- **Friction corrections closed:** endpoint/fullsampler/level-shape all lose to
  ordinary FM overall. Latest48TRAIN,200updates/arm,72tests/481WAV. Envelope
  FM1.600/new1.605dB,shape2.386/2.406,level1.041/1.112. No loss/weight/epoch sweeps.
  Standalone PCM exact;FLOAT samples exact,PEAK metadata differs.
- **Window/DC/repeat closed:** windowed inference did not improve audio;DC-only
  removal barely changes envelope1.593→1.591;real-repeat level.191 vs neural1.087.
  24real pairs,not48independent records;no window/normalization/randomness sweep.
- **Retained source-free baseline:** [neural/hybrid glass](/home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/comparison.wav),3.15s,40mm/s,.5N,90mm;NO audio/sensor/surfaceID input.
  Neural+48TRAIN spectrum bank,513tap motion-gated FIR,unitDC(v2);0newweights.
- **Hybrid:** unitDC FIR improved shape118/120new,72/72old;glass offset.2344s
  unresolved. Not quality admission;no EQ/tap/gating sweeps. Details in note.
- **Surface lineage:** `texture-surface-transfer-2026-09-05`,162560params,48TRAIN
  0/2/65/67/74/77,repeat0,20/30/50/60;all4/66/76 held development,not pristine.
  Weights6d36e47c…/c259a9bc…;bank5c440edd…;no runtime/demo/model replacement.
  Coefficients10mm/min≠audio20–60mm/s;no geometry. Glass category-mean still wins.
- **Timed countercheck:**`texture-full-event-2026-09-05` stale running JSON TERMINAL;
  corrected`texture-full-event-evaluation-2026-09-05`,19overlapping WAVs exact.
- **Impact diagnostics:** silence-learning/precision remedies contradicted;
  source-aided one-step previews are NOT source-free generation. Details in note.
- **Learned EPIC impacts:**154TRAIN/19participants,Top1 only3/14,heldwood/glass
  shape7.028→7.081dB. No replacement/data-size/capacity/epoch/seed sweep;see note.
- **Retained reference-free result:** [base/full/centered bridge comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),
  13.74s,glass10/seed2718; water raw/RMS AST+hardCLAP8/8, not physical calibration.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Earlier impacts:** LoRA1/14,bridge-OFF3/14;no LoRA/prompt sweeps or kitchen-spectrum quality claims.
- **Friction:** Figshare29438288v5/CC-BY4,60records;PSD neural/interpolation1.815/1.758dB,11/36wins,no sweep.
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
- **Retained water bridge:** TRAIN-centering removes98.397% common correction,
  restores AST/raw/RMS/hardCLAP8/8;material swap still fails. Centering/AdaLN/setting
  branches did not transfer;no13-container capacity/epoch/layer/setting/seed sweeps.
  Water CLI/profile/weight identities and failed output receipts are in the note.
- **Preserved guards:** publish exact PCM, never weaken.98 headroom.
  `requires_grad=False` is required for exact frozen-generator replay despite
  no_grad; no kernel-cause claim. Keep failures rather than overwrite/retry green.
- **EPIC probe:**`epic-information-2026-09-05`,161TRAIN/19participants/84videos,
  CC-BY-NC4 local research;P04/P07 excluded,no author val/test. Only7wood/glass;
  no object IDs/striker/geometry/force/velocity. AST macro24.30%,glass pair0/7;
  NOT qualified judge/reward,no threshold/prompt tuning. Full C1 controls in note.
- **Pair validator:** CLAP4/24 on6prompts(chance4),not qualified material judge/reward.
  Sequential/seek4one-LSB differences;remoteMP4 partialMD5 unverified,localWAV SHA passes.
- **Friction discriminator:**`texture-coefficient-discriminator-2026-09-05`,784WAV;
  TRAINmu interpolation wins shape119/120,levels worse onsteel/glass. Last2arms
  TARGET-AIDED. Glassmu loses equal mixing;best2-spectrum oracle limited. Both
  input limitations AND generator loss;not proof all nonlinearmu mappings fail.
- **Gradient discriminator:** gain-only insufficient;12TRAIN category gradients
  align. No evidence to introduce PCGrad;no gradient/weight sweeps.
- **Next:** bounded source-free learned conditioning adapter;first determine
  usable authorTRAIN material/action/motion groups and recording-disjoint dev split.
  Compare text AND fixed prototype baselines;produce WAV immediately,not support-only
  work. Geometry/force unknown;no author test/protected roles or generator/prompt/
  seed/prototype-count/guidance sweep. Empty-schedule failure remains open.

## Preserve these constraints

- User feedback: water sounds normal; rubber/glass seems normal but unfamiliar.
  Not friction realism admission. Prefer familiar water/impact/rain auditions.
- The user will not record impacts, hit glass or supply force-sensor data.
  Use internet sources. Preserve source attribution and applicable terms;
  unknown/incompatible redistribution terms exclude distribution.
- Generate playable media at each meaningful experiment checkpoint. Keep all
  candidates and honest failures; protocols, inventories and validators do
  not replace audible output. Latest SyncFusion discriminator jobs all terminal;
  full multi-event goal stays open; familiar auditions are not mandatory approval.
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
- Frozen transfer:nearest old parameters beat MLP9/9 later strikes,not new objects.
  Automatic16train/11later-strike cycle:expanded/anchored MLPs lose analytic control.
- No parameter-MLP size/epoch retries: sparse data/nonunique targets/loss mismatch
  unresolved; audio-to-parameter reconstruction is also the wrong goal interface.
- MP3s: `ps2-freesound-wine-glass-v1/research`,same author/pack,not known identical objects;no thickness/force inference.

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
- Random-phase residual/codec/prompt-waveform≠admitted physical formulas;
  waveform models are allowed as separate report-only generators with explicit claims.
