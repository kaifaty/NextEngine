# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / HERTZ_NEURAL_IMPACTS_AUDIBLE / PASSIVE_CONTACT_RESPONSE_NEXT.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest primary:** `modal3d-impact-render-2026-09-06`,15source-freeWAVs:
  [striker E2→70→200GPa](/home/kaifaty/.codex/experiments/nextengine/physical-sound/modal3d-impact-render-2026-09-06/striker-stiffness.wav),
  [speed .1→.5→1m/s](/home/kaifaty/.codex/experiments/nextengine/physical-sound/modal3d-impact-render-2026-09-06/impact-speed.wav),7.5s each;
  radius/target-stiffnessgalleries+11singleimpacts. Samebaaa9af5…NN,NOnewfit.
  `modal3d-impact-assessment-2026-09-06`:11coupledFEM→weakFEM→weakNNtriples.
  Alljobs terminal,20testsPASS,26fullfiniteWAVs/gain1,22replaysEXACT,noinputaudio/FEM/networkrenderreads.
- **OneSHAREDcorrectedfit:** `modal3d-teacher-effect-2026-09-06/fit`,10000params,
  unchanged1500Adamsteps/lr.001/seed42,weightsbaaa9af5e55adc52e51648acfad437366abc4d65324a1336fd2a33c167b1c516.
  36own3Dclampedcuboids×9contacts,324TRAIN;12×4exposedDEVinterpolation.
  FittraceEXACT36TRAIN NPZ/noDEV;TRAIN-onlymanifestexcludesnumericaldiagnostics.
  Renderreadsweights/owngeneratedoutputsforhashes,NOFEM/targetrecordings/network.
- **Against SAMEmesh4DEV, old→new:** freqmean3.049→.972%(12/12bodies),
  spectrum.270068→.104900(48/48),gainL1.363995→.371654(18/48wins),
  envelope.068975→.069988(21/48),level.032171→.038634(22/48).Notallqualityimproves.
  CorrectedTRAINlinear:freq1.382%,spectrum.085594,env.064100,level.033639,
  gain.369016. NNstillnotbetteroverall;automaticall-fiveoldcomparisonFALSE,
  baselineverdictREPORT_ONLY_PROTOTYPE_BASELINE_ADVANTAGE_NOT_ESTABLISHED.
  Olde426a289…checkpointandoriginalaudio unchanged;no modelpromotion.
- **Familyteacher:** `modal3d-corrected-data-2026-09-06`,pairedmesh3/4all48,
  144NPZ/372fullcontactWAVpairs. TRAINmesh3/DEVmesh4fixedbeforefit,noexclusions.
  43/48localstable:all12DEV;TRAIN2/11/20/29/32failONLYspectrum(.05–.063).
  Allfirst8modesinorder,maxfreqchange.429%;notcontinuum/realismproof.
  Priorone-shape4levels/8vs12testclosed:coarsegriderror,notmodalpermutation;
  no more mesh-level/NNloss/epoch/capacity sweeps to beat interpolation.
- **Contactdriver:** analyticalHertzF=kδ^1.5,kusesBOTH E/nu,m=4πrhoR³/3,
  initialvelocitydrivesforce. 64pointcontinuousquadrature handles8–74us pulses;
  128pointcountercheckrelPCM<1.2e-8. Target(.14,.055,.24),L.18,rho2230,onecontact.
  Separate8modepositive-self-residueFEMfeedbackfirstseparation,energyerror<1.13e-9,
  momentum<2.51e-10. Weakcontactrule10/11PASS;R5mmFAILimpulse+5.253%,spectrum.05193.
  MeanweakFEM/coupled spectrum.015645vsweakNN/coupled.322732:forcefix≠NNqualityfix.
  NumericalelasticpropertiesNOTidentifiedsteel/rubber;no real losses/yield/radiation.
- **Decisive mechanical limit:** sameNNqueryatforce=probe(1,.5)has3negative
  residues,includingmode8−26.624. SignedcrossgainsareNOTself-admittance;NOabs/clamprepair.
  `solve(contact_response=True)` nowreturnsactualself_gains/probe_self_gains,
  nonnegativeandcross²=self*probe. Defaultoutputunchanged. No newteacherTRAINyet.
- **Next primary:** sharedpassivecontact/proberesponse representation+audible
  coupledneuralimpact;research/compare againstretainedinterpolator,not oldpulse/
  epoch/capacitysweeps. CurrentNNstillvelocityNOTpressure;geometry/radiation/
  internet-recordedbodies andremainingeventfamilies necessary forfullgoal.
  Reuseexternal`neuralresonator-solver-python-2026-09-06`scikit-fem12.0.2.
- **Prior common-noise:** `sonicgauss-{common-noise-render,separation}-2026-09-06`:
  contact/noise.11851/.04639/latent.01302;identityreal8/10vsgen6/10;notrealism;see pilot.
- **Two-sample fit REJECTED:** `sonicgauss-energy-fit-2026-09-06`,adaptera4c23b52…,
  fresh262144sharedresidual/72steps/36TRAIN,2freshnoises/step. Frozenweightsunchanged.
  FixedDEVaudiblespectrum.961561→.972393(5/24),envelopeworse;rulesREJECT.
  IndependentDEVenergy4.329549→4.319196(20/24),spectrum1.091254→1.102875(1/24);
  shapeonly~.34%better. Preflightattenuation32→4of60didNOTsufficeforneuralquality.
  energy-{render,candidate-render,fixed-render}:360WAV+80comparisons;see pilot.
  No lossweight/gain/seed/epoch/timing-sweeps,SEDvariants orcodec/inventorycycles.
- **Previous primary/candidateREJECTED:**`sonicgauss-waveform-{render,assessment}-2026-09-06`,
  120source-freeWAV+60recorded/base/candidatetriples,gain1,prior20baselinesEXACT.
  `sonicgauss-waveform-odd-fit-2026-09-06`adapterSHA8d702d11…,72steps/262144params,
  full50Euler+VAEgrad,oddFFTboundary(keep;evenreflectionsfailed),frozenpublishedweights.
  DEVraw.82450→.76287andaudible.96156→.87239butlogRMS.43974→.63541;
  normalizedshape.91139→1.01524(4/24wins),18/24quieterthanreference.
  Wood14level/Plastic97all3metricsregress;oldraw-onlyrulepasses,newcombinedREJECT.
- 36TRAIN(objects2/6/12/24/66/95),24DEV(14/75/94/97),sixauthor-first/object.
  ALLknownpretrainingTRAIN;localobjectsplit≠pristineunseenobject. Source2.98sfit
  vsfull3sevalexplicit. Absolutephysicalsize/force/strikermaterialstillabsent.
- **Prior codec:**`sonicgauss-codec-probe-2026-09-06`,AUDIO-INPUT;modebeatsbase60/60,
  sampleworseaudible. Erasering/keep3Hz:raw.004259vsaudible.890216;notEQrepair.
- **Prior flowfit:**`sonicgauss-shared-contact-{fit,render,assessment,diagnostic}-2026-09-06`;
  adapter6c2f1eb1…;flowMSEimprovesbutDEVraw.82450→1.21726(3/24).Noflow/attention/lr/epochsweeps.
- `sonicgauss-cohort-data-2026-09-06`:10PLY+60WAV,89,707,492payloadbytes;
  68,690,214networkbytes,pinned206ranges/CRC/SHA,fullarchiveSHA NOTverified.
  Datasetrevisione905b8cb…,correctedprojection93d397c1…,range-root`sonicgauss-range-2026-09-06`.
  Objects2/6/12/14/24/66/75/94/95/97;66IronNOTsteel. No36/70authorval/80/41/92payload.
  Separateinputs.json(noaudio)/corpus.json(refprovenance). StracefitreadsEXACT36TRAIN;
  stracerenderreadsNOreference/corpus/network. DiagnosisonlyreadsDEVafterfit.
- SonicGauss pins/reproduction/environment:see pilot and`physical_sound_sonicgauss_shared_fit.py`.
  CachedreplayEXACT≠GSreencode;single-position-keyfusionisNOTsolecause;
  newresidualdidNOTfixquality. Keepstrict17source/5weightpins,noT5/pickle/network.
  KeepnativeSDPA/flash:disablingchangesPTv3patch1024→128;no perfclaim.
- Prior2DNeuralResonator fit improves12/16spectra but worsensphysicalratios;see pilot.No2Dsweeps.
- **SonicGauss scale collision:** own8Gaussianfixture doubledgeometry/contact/
  logscales producesEXACTsame8preprocessednetworkinputs. Positivecontrolsrelative
  contact/shape/appearancechange. `sonicgauss-input-probe-2026-09-06`,4tests.
  Oldmodal-frequency-control.wav is fittedDiffSoundmath, NOTneural/sizeevidence.
- **Retained:** [water](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),13.74s/glass10seed2718,
  8/8coarseWater,notmaterial/flowcalibrated. [Rubber/glass hybrid](/home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/comparison.wav),3.15s/40mm/s/.5N/90mm,
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
