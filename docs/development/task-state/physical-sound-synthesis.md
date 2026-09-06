# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / SHARED_3D_CONTACT_FIT_REJECTED / FLOW_LOSS_NOT_AUDIO_QUALITY / NO_LIVE_JOBS.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest primary:** 60 recorded→baseline→shared-fit comparisons, each10.444s,
  in`sonicgauss-shared-contact-assessment-2026-09-06`, e.g.
  [DEV glass94/contact0](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-shared-contact-assessment-2026-09-06/object-94-contact-0-comparison.wav),
  [failed DEV ceramic75/contact0](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-shared-contact-assessment-2026-09-06/object-75-contact-0-comparison.wav).
  All120 full stereo generated WAVs +latent/wave NPZ in`sonicgauss-shared-contact-render-2026-09-06`.
  One262144parameter contact-query→64geometry-token attention residual,zero-init,
  shared120Adamsteps/lr1e-4/batch4/seed42. Published five models frozen/version-
  checked/no gradients; no per-object parameters/gain/EQ/seed/checkpoint selection.
- Fit`sonicgauss-shared-contact-fit-2026-09-06/adapter.safetensors` SHA
  `6c2f1eb11d949fefe94a8049a85433f791dcd09fa93077db4a5ff7fc0f67f1f7`.
  Teachers2.98s,stereo/no amplitude normalization,SciPy resampling,VAE posterior
  MODE(authorusesSAMPLE);logit-normal discrete flow MSE,conditional/noCFGdropout.
  36TRAIN contacts(objects2/6/12/24/66/95),24DEV(14/75/94/97),sixauthor-first/object.
  All ten KNOWN pretraining TRAIN; localobjectsplit≠pristineunseenobject.
- **REJECT:** full-wave relativeMRSTFTL1 TRAIN.892170→.897237(13/36wins),
  DEV.824499→1.217264(3/24wins); ALL4DEVobjectmeansworse,ceramic75 1.06241→2.57052.
  Matchedvscyclicnextcontact TRAIN17→16/36,DEV13→13/24; notcontactvalidation.
  `decision.json`automaticallyrejectsanyDEVobjectmeanregression/nooverallimprovement;
  ruleaddedAFTERthisrun,notpreregistered. PasswouldNOTprovephysicalquality.
- **Decisive countercheck:** `sonicgauss-shared-contact-diagnostic-2026-09-06`:
  fourpairedfixednoise/timestepdraws/contact,NOtraining. FlowMSE TRAIN
  .455501→.442313(36/36wins),DEV.470465→.467904(18/24wins).
  Optimization worked onitsobjective butdecoded spectralerror worsened; do NOT
  equateflowlosswithrealism orrepeatattention/lr/epochs/seedsweeps. Changes do not
  fixmissingabsolute size/force/strikermaterial orcalibratephysicalresponse.
- **Next primary:** a shared-model improvement must be judged on decoded audio
  with object-level non-regression, not merely flow loss. Beforeanotherfit,
  discriminate representation/codec limits and waveform-objective mismatch on
  this retained cohort; no more data/per-bowl tuning to hide the failure. Need
  evidence-backed decoded-audio change +new audible pairs, notnewprotocol/roadmap.
- `sonicgauss-cohort-data-2026-09-06`:10PLY+60WAV,89,707,492payloadbytes;
  68,690,214networkbytes,pinned206ranges/CRC/SHA,fullarchiveSHA NOTverified.
  Datasetrevisione905b8cb…,correctedprojection93d397c1…,range-root`sonicgauss-range-2026-09-06`.
  Objects2/6/12/14/24/66/75/94/95/97;66IronNOTsteel. No36/70authorval/80/41/92payload.
  Separateinputs.json(noaudio)/corpus.json(refprovenance). StracefitreadsEXACT36TRAIN;
  stracerenderreadsNOreference/corpus/network. DiagnosisonlyreadsDEVafterfit.
- Published baseline20WAV+10geometrycaches`sonicgauss-cohort-baseline-2026-09-06`;
  [ten-object gallery](/home/kaifaty/.codex/experiments/nextengine/physical-sound/sonicgauss-cohort-baseline-2026-09-06/gallery.wav),34.722s.
  ItsassessmentMRSTFT.853852;matched11/20. ASTraw/RMS.005:Iron66/Ceramic75genTick
  versusrecordedringing;glass6/94/95Chink;wood12/14Tickoftenalsorecorded. NOTjudge.
- Generation50steps/seed0/noCFG/full131072stereosamples44100/gain1/noEQ/crop.
  Cached baseline replaysEXACT all20priorcases; originalfullGSreencodingstilldrifts
  firstatencoder(.0019566features→.0037788wave),notprovenkernel/duplicatevoxelcause.
  Previous`sonicgauss-conditioning-report-2026-09-06`:geometry/appearanceaffectoutput;
  originalfusionONEpositionkey,Q/K/Vgrad0/0/.05807,attentioncan'tselectGS tokens,
  butfullresidual+decoderSTILLcanlearnposition. NewresidualdidNOTprovequalityfix.
- Source`sonicgauss-source-2026-09-06`:Sonic7a5687af/Splat446ffb5/Pointc4aa232/Tangofb364c2.
  Assets`sonicgauss-assets-2026-09-06`:HF57b06047,2.506GB;pilotpins17sources+5weights.
  StrictVAE365/Tango243/GS537/PE7/fusion12;219T5keysunusedexplicitly,nopickle/network.
- Reproduce `physical_sound_sonicgauss_shared_fit.py` stagesfit/render/assess/diagnose;
  flags--source/--assets/--data/--baseline/--fit/--generated/--output per--help.
  PYTHONPATHexternal`sonicgauss-python-2026-09-06`+`mmaudio-python-2026-09-05`;
  HF_HUB_OFFLINE=1,TRANSFORMERS_OFFLINE=1,OMP_NUM_THREADS=4,OPENBLAS_NUM_THREADS=4.
  NativeSDPA/segment_reducecompat;DO NOTdisableflashbecausePTv3patch1024→128.
  No livejobs; fullmodel replay≠cached repeatability, no productperformance claim.
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
