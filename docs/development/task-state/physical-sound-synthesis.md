# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / DECODED_FIT_REJECTED / LOWER_LEVEL_MASKS_SHAPE_REGRESSION / NO_LIVE_JOBS.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest primary:** `sonicgauss-waveform-assessment-2026-09-06`,60triples10.444s
  recorded→baseline→decoded-fit, e.g.
  [Glass94](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-waveform-assessment-2026-09-06/object-94-contact-0-comparison.wav),
  [Plastic97](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-waveform-assessment-2026-09-06/object-97-contact-0-comparison.wav).
  120fullstereoWAV+latent/waveNPZ`sonicgauss-waveform-render-2026-09-06`,gain1.
  Freshsource-free process;straceNOtarget/corpus/network;20baselinesreplayEXACT.
- Correctedfit`sonicgauss-waveform-odd-fit-2026-09-06/adapter.safetensors` SHA
  `8d702d110f04ff497c3a1425bff545ff771b6f8f358c2c0ed57a07b118d199c3`.
  Freshzero262144paramresidual;72Adamsteps/batch1/lr1e-4/seed42/twocompletepasses.
  Full50Eulersteps+VAEbackprop,checkpointed;exactinitiallatent/PCMparityverified.
  FFT20HzspectralL1+.25×2msenv+.1×logRMS;odd4096boundaryextension,rawWAVunchanged.
  Publishedweightsfrozen/versionchecked,peak4286MiB. FixednoiseperobjectONLY.
  Initialeven-reflectiontrialFAILEDunchangedringerasuretest(.7628<.8),ownprocess
  stoppedexit143(lastlogged6),no checkpoint. Retained`sonicgauss-waveform-fit-2026-09-06`.
  Oddextensiontest.9893passes;no tolerancechange. No otherjobs killed/restarted.
- **REJECT despite raw-rule PASS:** rawTRAIN.89217→.77502(30/36),DEV
  .82450→.76287(17/24),all4DEVrawmeansimprove. IndependentIIRaudiblespectrum
  DEV.96156→.87239(18/24),envelope.84651→.82515(11/24),logRMS.43974→.63541(8/24).
  Wood14levelworse;Plastic97spectrum/envelope/levelworse. CombinedruleREJECT.
  ContactmatchedcountsDEV13→12/24;NOTphysicalresponsevalidation.31testsPASS.
- **Decisive posthoc shape audit:** independentIIRthenunitRMS ONLYinmetric;
  NOoutputgain/EQ. TRAINnormalizedshape1.07640→1.19690(10/36),DEV
  .91139→1.01524(4/24);candidatequieterthanreference28/36TRAIN,18/24DEV.
  `shape-diagnostic.json`: apparentrawspectralimprovement≠bettershape/realism.
- **Next:** boundedresearch escalation BEFOREanotherlikefit: discriminatepaired
  waveformregressionunderone-to-many/unobservedexcitation vsweak/missingphysical
  conditioning vsobjective/capacitymismatch. Then executable discriminator+media;
  NO RMSweight/loss/normalization/seed/epoch/per-objectgain sweep onexposedDEV.
  Bothsharedflowanddecodedfitsfailedactualcriterion;don'trepeatcodec/inventories.
- 36TRAIN(objects2/6/12/24/66/95),24DEV(14/75/94/97),sixauthor-first/object.
  ALLknownpretrainingTRAIN;localobjectsplit≠pristineunseenobject. Source2.98sfit
  vsfull3sevalexplicit. Absolutephysicalsize/force/strikermaterialstillabsent.
- **Prior codec:**`sonicgauss-codec-probe-2026-09-06`,64NPZ/376WAV,AUDIO-INPUTnotnewgeneration.
  Modebeatsbaseline60/60rawANDaudible;samplebetterraw57/60butaudiblemeansworse.
  Codecnotsolebottleneck;quietring/pulselossesreal. Iron66/Glass94~96/99%sub20Hz.
  Eraseringbutretain3Hz:rawerror.004259vsaudible.890216;NOTanEQrepair/hearinggate.
- **Prior flowfit:**`sonicgauss-shared-contact-{fit,render,assessment,diagnostic}-2026-09-06`,
  adapterSHA6c2f1eb1…,120steps;rawDEV.82450→1.21726(3/24wins),ALL4objectmeansworse.
  PairedflowMSEimprovesTRAIN.45550→.44231(36/36),DEV.47047→.46790(18/24).
  No flow/attention/lr/epochsweeps;codecanddecodedcounterchecksareALREADYcomplete.
- `sonicgauss-cohort-data-2026-09-06`:10PLY+60WAV,89,707,492payloadbytes;
  68,690,214networkbytes,pinned206ranges/CRC/SHA,fullarchiveSHA NOTverified.
  Datasetrevisione905b8cb…,correctedprojection93d397c1…,range-root`sonicgauss-range-2026-09-06`.
  Objects2/6/12/14/24/66/75/94/95/97;66IronNOTsteel. No36/70authorval/80/41/92payload.
  Separateinputs.json(noaudio)/corpus.json(refprovenance). StracefitreadsEXACT36TRAIN;
  stracerenderreadsNOreference/corpus/network. DiagnosisonlyreadsDEVafterfit.
- Baseline20WAV+10geometrycaches`sonicgauss-cohort-baseline-2026-09-06`,gallery34.722s.
  ASTraw/RMS.005:Iron66/Ceramic75genTickvsrecordedringing;NOTphysicaljudge.
- Generation50steps/seed0/noCFG/full131072stereosamples44100/gain1/noEQ/crop.
  Cached baseline replaysEXACT all20priorcases; originalfullGSreencodingstilldrifts
  firstatencoder(.0019566features→.0037788wave),notprovenkernel/duplicatevoxelcause.
  Previous`sonicgauss-conditioning-report-2026-09-06`:geometry/appearanceaffectoutput;
  originalfusionONEpositionkey,Q/K/Vgrad0/0/.05807,attentioncan'tselectGS tokens,
  butfullresidual+decoderSTILLcanlearnposition. NewresidualdidNOTprovequalityfix.
- Source`sonicgauss-source-2026-09-06`,assets`sonicgauss-assets-2026-09-06`unchanged;
  pilotpins17sources+5weights(HF57b06047),strictloads,noT5/pickle/network.
- Reproduce `physical_sound_sonicgauss_shared_fit.py` stagesfit/render/assess/diagnose;
  flags--source/--assets/--data/--baseline/--fit/--generated/--output per--help.
  PYTHONPATHexternal`sonicgauss-python-2026-09-06`+`mmaudio-python-2026-09-05`;
  HF_HUB_OFFLINE=1,TRANSFORMERS_OFFLINE=1,OMP_NUM_THREADS=4,OPENBLAS_NUM_THREADS=4.
  NativeSDPA/segment_reducecompat;DO NOTdisableflashbecausePTv3patch1024→128.
  No livejobs; fullmodel replay≠cached repeatability, no productperformance claim.
- **Previous learned:** [2D reference→base→fine-tuned](/home/kaifaty/.codex/experiments/nextengine/physical-sound/neuralresonator-finetune-evaluation-2026-09-06/case-048-comparison.wav).
  Shared328000parameter/100steps,48TRAIN16DEV. SpectralL1.32938→.27148(12/16wins),
  envelope.29593→.26575(15/16);case061worseboth,physicalratioerror.191%→.586%worse.
  FitSHAfed24c81…,standalonecoeff/PCMEXACT;NOTrealism/3D/newtopology;no2Dsweeps.
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
- **Evidence/reproduction/pins/controls/failures:** [text-generation pilot](../physical-sound-text-generation-pilot.md).

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
