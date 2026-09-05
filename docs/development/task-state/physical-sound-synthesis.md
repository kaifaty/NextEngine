# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / SHARED_2D_FINETUNE_IMPROVES_LOCAL_TRANSFER_WITH_REGRESSIONS / NO_LIVE_JOBS.

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
- **Latest learned:**[reference→base→fine-tuned](/home/kaifaty/.codex/experiments/nextengine/physical-sound/neuralresonator-finetune-evaluation-2026-09-06/case-048-comparison.wav),4.5s.
  One328000parameterlastlayer fit,100Adamsteps/lr1e-5/batch4/seed42/clip1;
  frozenencoder+earlierMLPlayersexact.48TRAIN(8shapes×3materials×2contacts),16DEV
  (4othermasks×2othernumerictuples×2contacts),sameconvexpolygonfamily,notrealobjects.
  DEVrelative spectralL1 .32938→.27148(12/16wins),2msenvL1 .29593→.26575(15/16).
  All4shapeaveragesimproveboth;case061worsensboth,retained.13tests/64WAVchecks pass.
  `neuralresonator-finetune-fit-2026-09-06/model.pt`SHAfed24c81…;dataroot`...finetune-data...`.
  Standalone`...finetune-standalone.../neural.wav`fromdescriptors+weights;no data.npz/
  targetaudio;raw+coeffsEXACTcase048. Fixed.5gain. Directionsdensity/stiffness/damping
  retained,butratioaccuracyworse(.191%→.586%). No blanketreplacement/realismclaim.
  CausalSOSusedforassessment;FFTtrainingapproxmaxDEVrelativeRMS.001457(recorded).
- **Previous pairs:**`neuralresonator-reference-report-fixed-2026-09-06`,12cases
  (3ownpolygons×2numericmaterials×2contacts),each2.5s numerical→neural.
  10/12dominantpeakswithin1.60%,other2errors7.44%/35.58%;not generalpitcherror.
  Rectanglecombinedoffcenter reference1900Hz vsneural1224. ALL12neuralenergy
  centroids shorter(.357–.832×reference). Reference5refinement onfailure/octagon/
  skewed controls preservespeaks/timing;max32modefreqchange.016–.170%,notmeshfix.
  World=2*(normalized-.5)matchesauthorresultsnotebook;generator defaultuses1.
  Explicit scale1/2control gives2×frequencies withSAME neuralinput;no sizecontrol.
  36WAV finite/headroom/layout/hash/replaychecks,10tests/Ruff pass;nojobs live.
  Initialreference run exit1 ONLYfinalJSON int32serialization,all36WAValready
  produced;fixedint→newrun36WAVEXACT. Bothruns/failure.txt preserved.
  Solveroverlay`neuralresonator-solver-python-2026-09-06`,scikit-fem12.0.2,no venvchange.
- **Retained neural:**[seven numeric-condition impacts](/home/kaifaty/.codex/experiments/nextengine/physical-sound/neuralresonator-replay-2026-09-06/comparison.wav),10.5s.
  Neural Resonator published checkpoint,NOT trainedhere. Own64x64octagon+contact+
  rho/E/nu/alpha/beta→EfficientNetB0+MLP→32parallel×2IIR. No targetaudio/dataset.
  Base785Hz;lowerdensity1785,higher584,stiffer976. Ratios within.191%ofundamped
  sqrt(E/rho)relation onthisshape;damping shortensenergycentroid7.60→5.37ms.
  NOTrealism/new-object/size/striker/force/3D validation. Seven1s32kmonoWAVs;
  .5s gaps,one sharedgain1.11307073,no gates/tailcrop/EQ.7/7coefficient+PCM
  crossprocessreplayEXACT,allpolesstable,6tests/Ruff/media/stracechecks pass.
  `neuralresonator-assets-2026-09-06`:55MBckptSHAfa46fa22…;sourcepin below.
  Safeweights-only load maps11nonstandardglobals to inertdatacarriers,NOTobjects.
  Strict model+encoderload;onlycriterion.fbunused;global_step0isserializedmetadata,
  not measuredtrainingcount. CPU4threads37–63ms/case excludingmodelstartup.
- **Previous:**`syncfusion-extended-audition-2026-09-06`,4comparisons
  glass/wood/glassrigid/metal:real→retained→expanded.490TRAINvs239,glass11recordsvs5;
  original307rows/roles/embeddings EXACT.4WAV,16/16attacks,1EXTRAglassrigid.
  Fixedold references shape improves3/4,woodworse;held68embedding.15514→.15912worse.
  No promotion. Fit`syncfusion-extended-fit-2026-09-06`,SHA6e3e678a…;data`...extended-data...`624rows.
  30tests,Ruff/media checks;all MODEL and download jobs now terminal.
- **Shard3 complete:** existing Range resume exited0; PID229438 gone; size
  1955266560/MD5f003e4764debaf68097e2aefd613cd0c/SHA679c9183… verified.
  `syncfusion-extra-train-2026-09-06/train_shard_3.tar`; annotations only inspected,
  no new decoded WAV/cache/fit. Initial exit28/300s and resume history preserved.
  No automatic shard3 categorical refit; extra targets do not add missing inputs.
- **SonicGauss discriminator:** own synthetic GS object and uniformly doubled
  geometry/contact produce EXACT same8tensor inputs through pinned upstream
  normalization. Relativecontact/shape/appearance positive controls change.
  `sonicgauss-input-probe-2026-09-06`;4tests,Ruff/offline replay/media checks pass.
  No weights/full inference/dataset payload. Modal-frequency-control WAV is OLD
  fitted coefficients with frequency halved, NOT neural or physical-size evidence.
  Do not run SonicGauss as-is to learn absolute size/force/striker control.
  Relative shape/contact potential remains untested, not disproven.
- **Real controls still fail:**extended TRAIN banks on SAMEold68held:
  shape mean/recordmacro.4980/.4226,embeddingmean/record.6032/.2837. Embeddingmean
  glass25→91.67%butwood/metalworse;shape still7/8REALglass→metal. No judge/reward.
- **Retained metal:** [same original shared adapter](/home/kaifaty/.codex/experiments/nextengine/physical-sound/syncfusion-metal-standalone-2026-09-06/metal-static-adapter.wav),4/4,0extra,knownTRAINmaterial,no newweights.
- **FOV closed:**`syncfusion-full-frame-audition-2026-09-06`,stockDINO discards17%
  width EACHedge(marker test).224letterbox preservesinputbutheld68 stillworse;
  spectrum only1/3wins,wrongwoodbeatscorrect. No promotion/DINO/crop/ridge sweep.
- **Contact diagnostic:**`syncfusion-contact-localization-2026-09-06`,3 ORIGINAL
  recorded-audio videos,not generation. Motion tracks shaft,not tip;no croptraining.
  Public contact pixel labels not located;not proof lost. No TLS bypass/val/test.
- **Next primary artifact:** bridge to3D geometry-conditioned source-free audio,
  notanother2Dloss/epoch/seed sweep. SonicGauss remains a candidate for RELATIVE
  shape/contact only,not size/force/striker (inputcollision remains). Inspect/load
  publishedweights+one eligible TRAIN/disclosed3DGS object (neverdefaultval or
  protectedObjectFolderaliases),then contact-pair WAVs. Sourcepins in previous
  probe/pilotnote. Do not claim2Dsuccesssolves3Drealism or unifiedwater/rain/etc.
  Current12reference+16DEVcases OPENEDdevelopment,notpristineholdout. No runtime
  promotion. Phase/magnitudeFFTswaps worsenedall12temporalerrors and are noncausal
  hybrids,not evidenceisolatingonecause;no manualphase/gaincorrection sweep.
  Repo`rodrigodzf/neuralresonator`,revceab3770d88caae1c9ee208bea127ec0d0a1e763;
  source/model/dsp/utilities/modal/data/shape/resultnotebook read;assets external.
  CLI`physical_sound_neuralresonator_pilot.py --assets ... --output NEW` usesexisting
  `mmaudio-python-2026-09-05`PYTHONPATH(torchvision0.28);noenvchanges/Lightning/loggers.
- **Retained source-free learned media:** [five glass impacts](/home/kaifaty/.codex/experiments/nextengine/physical-sound/syncfusion-adapter-standalone-2026-09-05/glass-rigid-motion-adapter.wav).
  Adapter17,088params,200steps,seed42,not decoder training. Material+motion+times;
  .4/.9/1.7/3/4.7s schedule5/5,0extras;openat confirms no data/audio reference reads.
  No geometry/size/force/striker claim;comparisons/35tests in pilot note.
  Fit`syncfusion-adapter-fit-2026-09-05`,SHA863459fb…;data307events,239TRAIN/86recordings,
  50recording-dev/21recordings,18combination-dev/3recordings. Exclude entire records
  2015-03-20-02-16-43/02-27-12 and2015-03-27-23-30-55(glass+rigid);previously opened
  examples,generator-pretrained TRAIN,NOT pristine/new-object evidence.
  Heldglassrigid mixed:only2/4referenceevents improve;no size/epoch sweep.
- **Prior oracle:**4reference-aided WAVs support text/audio transfer gap,not material
  admission. Shard1MD5/SHA7284c9dd… verified,CC-BY4/Zenodo12634671;no val/test.
- **Previous source-free timing:**`syncfusion-explicit-times-2026-09-05`:text+times,
  12/12attacks. Empty schedule peak2.319/RMS.0328 rejected,not silence;still open.
  Assets`syncfusion-assets-2026-09-05`,SHAa25584b1…;no blanket production clearance.
- **MMAudio impact:** video2/9,delayed3/8timing matches;no prompt/seed sweep. Apple
  CLIP research-only excludes product development;NOT engine candidate weights.
- **MMAudio water:**coarse Water top5,not qualitywin;usefullMP4,not shortmux.
- **Friction corrections closed:** endpoint/fullsampler/level-shape lose toFM;
  window/DC changes don't fixtiming.24real-repeat pairs,not48independent records.
  No loss/weight/epoch/window/normalization/randomness/EQ/tap/gating sweeps.
- **Retained source-free baseline:** [neural/hybrid glass](/home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/comparison.wav),3.15s,40mm/s,.5N,90mm;NO audio/sensor/surfaceID input.
  Neural+48TRAIN bank,513tap unitDC FIR(v2);shape improves,glass offset.2344s remains.
- **Surface lineage:** `texture-surface-transfer-2026-09-05`,162560params,48TRAIN
  0/2/65/67/74/77,repeat0,20/30/50/60;all4/66/76 held development,not pristine.
  Coefficients10mm/min≠audio20–60mm/s;no geometry. Glass category-mean still wins.
- **Retained reference-free result:** [base/full/centered bridge comparison](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),
  13.74s,glass10/seed2718; water raw/RMS AST+hardCLAP8/8, not physical calibration.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
- **Earlier impacts:**no LoRA/prompt/precision/silence-learning retries;EPIC154TRAIN
  Top1 only3/14,not replacement;no data-size/capacity/epoch/seed sweep.
- **Friction:**Figshare29438288v5/CC-BY4,60records;PSD neural loses interpolation.
- **Rain:** DataSuds10.23708/I0QYNM V2/CC-BY4.0; CSV verified,TSV rejected.
  Stationary model loses temporal structure, AST fails real wet. No MLP sweep.
- **Pouring source:**Bagad et al.,dataset terms unspecified,NOT software MIT;
  local research only. Testsunopened,18/30excluded;flowproxy≠measured ml/s/level.
- **Pouring closed:** STFT/envelope/EQ/pitch/CVAE/critic/scratchflow/affinecoupling/
  solver/noise/phase/centering/AdaLN/setting retries failtransfer. No13-container
  capacity/epoch/layer/seed/threshold sweeps. Retained centeredbridge8/8coarsewater,
  materialswap fails. Native645x64 Tango≠old normalized88cache;targetinjection≠sourcefree.
  Keep13-prompt/raw/RMS0.005 controls;exact CLI/weights/failure receipts in note.
- **Preserved guards:** publish exact PCM, never weaken.98 headroom.
  `requires_grad=False` is required for exact frozen-generator replay despite
  no_grad; no kernel-cause claim. Keep failures rather than overwrite/retry green.
- **EPIC:**161TRAIN/19participants,CC-BY-NC4;P04/P07 excluded,noauthorval/test.
  No objectIDs/striker/geometry/force/velocity. AST24.30%,glass0/7,CLAP4/24chance4;
  NOT judge/reward;no threshold/prompt tuning. RemoteMP4 partialMD5 unverified.
- **Frictionmu:**shape improves butlevels failsteel/glass;glassmu losesmixing,
  last2arms TARGET-AIDED. Bothinput andlosslimits,not proof allnonlinear maps fail.
  Gain-only insufficient;12TRAIN gradientsalign,no PCGrad/gradient/weight sweeps.

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
- Frozen transfer:nearest beats MLP9/9laterstrikes;16train/11later cycle also loses.
- No parameter-MLP size/epoch retries: sparse data/nonunique targets/loss mismatch
  unresolved; audio-to-parameter reconstruction is also the wrong goal interface.
- MP3s:same author/pack,not known identical objects;no thickness/force inference.

## Legacy admission evidence and forbidden retries

- [V46 D0](../physical-sound-v46-d0-synthetic-source-preflight-result-2026-09-03.md)
  already closed NISR20368791…/VibraVerse8099f137… before payload for missing exact
  generator/asset lineage. No bulk/bounded payload retry absent that new evidence.
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
