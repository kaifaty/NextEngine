# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / FIRST_SOURCE_FREE_3D_CONTACT_PAIR / EXACT_REPLAY_FAILS / NO_LIVE_JOBS.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest primary:** [3D neural contact A→B](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-contact-pair-2026-09-06/comparison.wav),6.444s.
  [Recorded A→neural A→recorded B→neural B](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-contact-assessment-2026-09-06/reference-generated-comparison.wav),13.944s.
  Published shared SonicGauss, NO new training. ObjectFolderReal6 glass bowl,
  disclosed canonical TRAIN parent realimpact-6-bowl--objectfolder-real-object-6.
  Author TRAIN first two records: contacts16/35, selected before results.
  3DGS+contact→PTv3→position/fusion→TangoFlux→VAE; no target audio at inference.
- Source root `sonicgauss-source-2026-09-06`: Sonic7a5687af…, SplatFormer446ffb5…,
  Pointceptc4aa232…, TangoFluxfb364c2…. Runner pins17sourcefiles/fullfiveweighthashes.
  Assets`sonicgauss-assets-2026-09-06`: HFmodel57b06047…,2.506GBverified;
  strict VAE365/Tango243/GS537/position7/fusion12keys.219unusedT5keys explicitly
  excluded, no tokenizer/pickle/upstreampackageinitializers/download at inference.
- Datasetrevisione905b8cb…; `sonicgauss-range-2026-09-06` contains bounded ZIP64
  acquisition scripts, directorymetadata/TRAINJSON/onePLY/two assessmentreferences.
  HTTP206/ranges,CRC/selectedSHA checked; full24.7GBarchiveSHA NOTverified.
  PLYSHA92892268d640c54e60482562fb712b0a6b562a15917d6256bb7fe475dd058303,
  5629642bytes.12250retainedGaussians,12130uniquegrid384voxels. No deduplication.
  No authorvalJSON/protectedresponse/otherobjectpayload. References acquired only
  AFTER generation, in assessment; strace freshinference confirms no target reads.
- Externaloverlay`sonicgauss-python-2026-09-06`: SpConv-cu1182.3.8/Cumm0.7.11,
  gin0.5/plyfile1.1.3/addict2.4/timm1.0.20; plus existingMMAudiooverlaytorchvision.
  PTv3retains1024patches,stride2/2/4/4,FP16QKV,serialization/padding. Explicit
  SDPAforraggedFlashAttention and torch.segment_reduceforCSR compatibility.
  DO NOT simplydisableflash: upstreamalsochangespatches1024→128. CPUequation
  controls pass; GPUvsFP64maxabs.00038962/relativeRMS.00026727,notbitwisebackendparity.
- Seed0/50steps/noCFGsamebothcontacts. Full131072stereosamples/44100Hz=2.972154s,
  request3s;gain1,noEQ/crop/gates. ASTChink-clinktop1both/raw+RMS.005; references
  Ding/Clangtop1. Coarseclass only,NOTmaterial/realism/positionvalidation.
  Source sub20Hzenergy32.7%/54.6%,neural43.5%/45.5%;notgenerator-onlyartifact.
  Referenceaudiblepeakbins13099/784Hz;generatedboth3896Hz,notfullspectralassessment.
- **Exact replay FAILS:**A→BrelativeRMS.07017;A→repeatA.0017715(~39.6×smaller).
  Freshprocessalso differs; raw arrays/latents/failedrepeats retained in
  `sonicgauss-contact-pair-2026-09-06` and `sonicgauss-contact-replay-2026-09-06`.
  Duplicate voxels observed but NOTproven driftcause. No retry-to-green.
  Warm.61–.65s excludesstartup,notruntimeperformance. All jobs terminal.
- **Next:** discriminate whether actual geometry/contact differences survive
  shared3Dencoder and frozen decoder, with geometry/appearance and cached-
  conditioning controls BEFORE training. Do not start per-objectfits/seed/EQ
  sweeps or more2Dloss tuning. KnownTRAINbowldemo is NOTnew-object evidence.
  SonicGauss as-is lacks absolute size/force/striker controls (collisionbelow).
- Reproduce via `lab/scripts/physical_sound_sonicgauss_pilot.py`:
  `--source SOURCE --assets ASSETS --input RANGE --output NEW_EXTERNAL`.
  Above2PYTHONPATHoverlays,HF_HUB_OFFLINE=1,TRANSFORMERS_OFFLINE=1,
  OMP_NUM_THREADS=4,OPENBLAS_NUM_THREADS=4. Allgeneratedassets outsideGit.
- **Previous learned:** [2D reference→base→fine-tuned](/home/kaifaty/.codex/experiments/nextengine/physical-sound/neuralresonator-finetune-evaluation-2026-09-06/case-048-comparison.wav).
  Shared328000parameterlastlayer,100Adamsteps,48TRAIN(8ownshapes×3materials×2contacts),
  16DEV(4othermasks×2othernumerictuples×2contacts),sameconvexpolygonfamily.
  SpectralL1.32938→.27148(12/16wins),2msenvL1.29593→.26575(15/16);case061worseboth.
  FitSHAfed24c81…, standalonecase048coeff/PCMEXACT;no referenceaudio input.
  Density/stiffness/dampingdirectionsretained,butratioerror.191%→.586%worse.
  Notrealism/3D/newtopology. No more2Dloss/epochsweep; full evidence inpilotnote.
- NeuralResonatorcheckpointfa46fa22…/sourceceab3770… in`neuralresonator-assets-2026-09-06`.
  Ownmask/contact/rho,E,nu,alpha,beta→32parallel×2IIR. Safeweights-only+inertmetadata,
  no picklefallback. 12quadraticFEM2Drefpairs:10/12peakswithin1.6%,two7.44/35.58%.
  Meshrefinementdoesn'tfixmodalprominence/timing. Worldscale2matchesauthornotebook;
  changingphysicalscaleleavesneuralinputsame. FFTphase/magnitudehybridsworseall12,
  noncausal/misaligned,notcausalisolation. Fullreference/fitnotes inpilot.
- **SonicGauss scale collision:** own8Gaussianfixture doubledgeometry/contact/
  logscales producesEXACTsame8preprocessednetworkinputs. Positivecontrolsrelative
  contact/shape/appearancechange. `sonicgauss-input-probe-2026-09-06`,4tests.
  Oldmodal-frequency-control.wav is fittedDiffSoundmath, NOTneural/sizeevidence.
- **Retained:** [water](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),13.74s/glass10seed2718,
  8/8coarseWater,notmaterial/flowcalibrated.
  [Rubber/glass hybrid](/home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/comparison.wav),3.15s/40mm/s/.5N/90mm,
  NN+48TRAIN513tapunitDCFIR,notfrictionrealism. User:waternormal;rubber/glassunfamiliar.
  [Five shared-adapter glass impacts](/home/kaifaty/.codex/experiments/nextengine/physical-sound/syncfusion-adapter-standalone-2026-09-05/glass-rigid-motion-adapter.wav),
  seed42/150steps5/5noextra,notgeometry/size/force/striker. OriginaladapterSHA863459fb….
- SyncFusionexpanded490TRAINvs239 failsold68heldembedding.15514→.15912;woodworse,
  oneextraglassrigidattack. No promotion/refit. Bothshards2/3fullyverified,nojobs;
  shard3onlyannotations,notdecoded/trained. No DINO/FOV/crop/ridge/embedding/
  datascale/epoch/seed sweeps. Realglassstill7/8shape→metal,notvalidator/reward.
- **Evidence/reproduction:** [text-generation pilot](../physical-sound-text-generation-pilot.md).
  Includes exact source/weight pins, acquisition receipts, controls and failures.

## Preserve these constraints

- User will NOT record impacts, hit glass or supply force sensors. Internet only.
  No mandatory per-sound listening approval; do not claim listening notperformed.
- Full Tango decode retained, use existing matched`event_window`, NOTprefixonly:
  seed314canstartafter11s despite3srequest. Neveramplifycodecnoise;continuoussources
  differ. Signal/event gates do not prove realism. Preserve.98headroom/failureWAVs.
  Frozenmodelsrequires_gradFalse neededforexactrepeat evenwithno_grad.
- Dataset/weights/caches/clones/WAVs external root:
  `/home/kaifaty/.codex/experiments/nextengine/physical-sound/`.
  Respect attribution/terms;unknown/incompatible redistributionexcludesdistribution.
  No blanketproductionlicense fromsoftwarelicense/modelcard. No unrelatedjobkills.
- [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md)
  Proposed/report-only waveform/authoredassetresearch. No runtimeweights/gameplay
  authority/demo replacement/promotion. Existing authoredfallback remains.
- Do not repurposeprotectedholdout/shadow/validator/one-shotroles. OpenTRAIN/DEV
  cannotbecomecleanholdout;foundationpretrainingoverlapunknown.
- Do not invent exactgeometry/composition/force/velocity/listener fromlabels/audio.
  Perceptualclassand calibratedphysicalresponse are different claims.
- One stable [V46 roadmap](../../plans/physical-sound-synthesis-roadmap-v46.md):
  historicalboundedimpactadmissionprogram,notfullgoal. Lab runsdo notpromoteit.

## Closed paths and guards — do not rediscover

- [V46 D0](../physical-sound-v46-d0-synthetic-source-preflight-result-2026-09-03.md)
  NISR20368791…/VibraVerse8099f137… CLOSED beforepayload for missing exactgenerator/
  assetlineage. NO bulk ORboundedpayloadretry absent exactnewidentityevidence.
  Cardsallowed;NISRmaterialmixtures arehomogeneousblends,notinteractingbodies.
- [Corrected corpus](../physical-sound-v43-d2-c0r-identity-repair-result-2026-09-03.md):
  REALIMPACT/ObjectFolderReal/AV-MSF shareTRAINcomponent;Object6mergedidentity;
  Object80Wood/Ceramicconflict→Unknown. OldAV-MSFvalidatorrolesleaked/superseded.
  Legacyrepairprofileauthorizesbaselineauditsonly,notnewcandidateadmission.
- V12Object41acquisitionOOD forceguardunchanged;noresponse/protectedaccess.
  Object92do notdropcontact35/downloadirrelevant12GBsegment.
- V16–V37spentprotectedfamilies CLOSED;no threshold/role/seed/capacity/contact
  tuning or reconstructingmissingoutputs. M0cphysicalresponse/QSO-v0transferfailed.
- EPIC161TRAIN19participantsCCBYNC4,P04/P07excluded,noauthorval/test/noobjectIDs/
  striker/geometry/force. AST24.3%,CLAPchance:NOTjudge. Bridge3/14,LoRA1/14;
  no furtherprompt/precision/silence/capacity/data-size/epoch/seed sweeps.
- FrictionFigshare29438288v5CCBY4:TRAIN0/2/65/67/74/77repeat0speeds20/30/50/60;
  held4/66/76developmentat40,notpristine. Measuredmu10mm/min≠audio20–60mm/s.
  Endpoint/fullsampler/levelshape/window/DC/gain/PCGrad/phase/loss/FIRsweepsclosed.
- Pouringtermsunknown,notsoftwareMIT;18/30excluded/authorTESTunopened.
  STFT/envelope/EQ/pitch/CVAE/critic/flow/solver/phase/centering/AdaLNvariantsclosed;
  materialswapfails. NativeTango645×64≠old88cache. No13container sweep.
- RainDataSuds10.23708/I0QYNMv2CCBY4:stationarymodeltemporalfailed,ASTrealwetfails;
  no MLPsweep. MMAudioAppleCLIPresearchonly,notenginecandidate;no prompt/seedsweep.
- Keep likedDiffSoundstep150/glassB and firstlikedreconstruction demos. Inputaudio
  reconstruction is NOT source-free generation. No per-recordparameterMLP sweep.
