# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / RAW_SIZE_RESPONSE_REJECTED / ANALYTIC_POLE_TRANSPORT_AVAILABLE.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest primary:** [NN base profiles with analytic resizing](/home/kaifaty/.codex/experiments/nextengine/physical-sound/objectfolder2-size-final-2026-09-06/steel-wood-ceramic-profile-size-variants.wav),27s;
  steel23/wood29/ceramic66, each .8/1/1.25 linear size. SixTRAIN size probe,
  no new fit or target audio/FEM. Compare own baseline, NOT real-sound accuracy.
  RawNN lowest natural frequency moves mean.6165%,max2.026% despite resizing;
  required-law mean absolute error22.211%,0/12 common-band mode-count matches.
  `physical_sound_objectfolder2_size_probe.py`:transport lambda/r², restore
  Rayleigh decay/damped frequency; identity exact. Gains HELD, not amplitudephysics.
  Commonband≤10kHz undamped atbaseline; no unseen above-Nyquist coverage claim.
  `objectfolder2-size-final-2026-09-06`:44WAV QA/36exactreplays/29tests PASS.
  Final run source hash precedes lint-only `.items()` loop edit; WAVs unchanged.
- `objectfolder2-channel-fit-2026-09-06`:69712parameter NN,128bands×3slots;
  sixTRAIN7/23/29/66/75/82/openDEV11/54/88;2000Adam.001seed42.
  Weightsf5e5e578d4ecb5cc81f9c12a85b61d6647b491e47e57036f61c1222bf3ad90ad.
  RawDEVspec3.46905/env7.34727/level.83842REJECT;TRAIN.74552/.59787/.83701.
  OutputchannelshelpTRAINbutnottransfer;no channelwidth/mask/loss/epoch sweeps.
- **Cause verified:** publishedOF2suppTable1Rayleighalpha/beta reproducesall5309
  FULLsource modes with maxrelative6.66e-16 (`objectfolder2-rayleigh-source-2026-09-06`).
  Independent learneddecay violatesmateriallaw (ceramic88min.1468vsminimum3).
  `physical_sound_objectfolder2_rayleigh.py` computes stable low quadratic root
  from predictedDAMPEDfrequency+assignedmaterial,NOTfitted/targetcoefficients.
  Keep source-freef/g/occupancyEXACTfixed;learneddampingretainedinNPZ.
- `objectfolder2-rayleigh-{render,assess}-2026-09-06`:DEV1.16898/1.16439/.57924,
  TRAIN.74507/.59873/.83194. BigrawfailurepartlyfixedbutREJECT_QUALITY_ADVANTAGE.
  Priorcompact5b569809… remainsbetterDEVspec/env(.98730/.56224),level.64982;
  nearest/sizecontrolslevel.49116/.40940. No promotion, no arbitrarymaterialproof.
  Alljobs terminal;105newWAVfullQA,72exactreplays,9f/g/maskidentities,25testsPASS.
- `objectfolder2-shared-diagnostic-2026-09-06`:sixTRAINoraclepole/fieldswaps:
  truepoles doNOT fixfieldfailure; target-assisted,NOTnewNN. Detailsinpilot.
- **Research/probe DONE:** `objectfolder2-band-qr-probe-2026-09-06`,7oracleWAVs.
  SixTRAINfirstcontacts:128fullMelbands,≤3modesretained;>3signedsum spectral.05160
  vsenergyGram.23196. EnergyperbandPASS3.99e-15buttotalcrossbanderrorup26.55%.
  Rejectindependentbandenergyasfix;no Gram/phase/band-count sweeps. QRfixed
  inverseill-conditioning,NOTsound;DeepModalresearch/failureevidenceinpilot.
- `objectfolder2-compact-data-2026-09-06`: signedpacking usesONEpole/band across
  all32contacts; TRAINcommonpoleoracle spectral.05714/env.02530/level.01148.
  Allsource modesaccountedfor,NOTlossless. Raw9targets preserved;inputsEXACTcopied.
- **Next:** address shared shape learning/data coverage, not another per-material
  fit. Inspect available OF2 cohort coverage and physical normalization before
  expanding same-source geometry/audio pairs; keep object-disjoint evaluation.
  Size evidence rejects relying on this raw size input; reuse analytic transport
  for same-shape pitch/decay, not proof of new shapes/materials/amplitude realism.
  No repeated decoder/mask/phase/loss/capacity/epoch/seed sweeps on these six bodies.
- **Source:** ObjectFolder2/rhgao revision3c6cd8930b2dcbadb6d94dadf2745c956bdcd236;
  `objectfolder2-source-2026-09-06` auditedaudioDDSP/MLP/CSV/paper/license.
  `objectfolder2-range-2026-09-06/family-extraction.json`:9completemesh/checkpoint
  pairs from first320MiB of3.77GBarchive;individualhashes,NOTfullarchivechecksum.
  IDs7/11/23/29/54/66/75/82/88;34–1965modes. **OF2 IDs≠SonicGauss/OF-Real IDs**.
  Demo23EXACTmatchesarchive;GSOCCBY4/originalmeshtermsretained;23/29URLsNone.
- `physical_sound_objectfolder2.py`:strictweights_only+numericNumPyallowlist,
  reviewedhash-pinnedASTdeclarations,noimports/CUDA/optimizerexecution;
  Authoroutputpeaknormalizationerasesforceandbreakszero;omitted,commonper-object
  auditiongainused. Trainfromrawcoefficients,NOTaudition-normalizedWAVs.
  Oneauthor-demoqueryoutsidecoordinatebounds;noinputclamp. Detailsinpilot.
- Teacher29underflowbit-scalingFAIL;ULPbound27PASS/all9zeroexact;detailsinpilot.
- Fullpreparepack preserves34–1965modes/32points,512meshvertices+log3sizes+
  materialonehotinputs. No target count/poles/audio at standaloneinference.
  TeacherhasNOpressure/radiation/listenerstageorstrikermaterial. SignedAudioNet
  gainsareNOTcontactself-admittance;do notfeedthemintoHertzfeedbackaspositiveports.
- **Pressure:** `guitar-fsi-probe-2026-09-06`,48stateROMpressureerror6.12%.
  No-FSI→pressure/backzero,topmoves. FixedmatricesNOmesh/recording/family;
  near-holepressureNOTfar-fieldmic. No basis/order/epochsweeps/per-guitarfit.
- **Prior coupled NN control:** `modal3d-passive-fit-2026-09-06`,6805a313…;
  `modal3d-passive-render-2026-09-06`:63source-freeWAVs,25tests/59exactreplays.
  R_i=a_i(p)a_i(q),PSDself=a²(noabs/clamp),5064fieldparams,frozenbaaa9af5…freq.
  On48heldcuboidsNNspectrum.18726vsold.21774butinterp.15806:quality_advantageFALSE.
  `modal3d-passive-factorial-2026-09-06`:interp-freq/neural-port.13119,NOTnewNN.
  Keepasstrongcontrol;NOcuboidmesh/pulse/field/frequency/epoch/capacitysweeps.
  HertzusesbothE/nu;weakR5mmfailsimpulse+5.253%,coupledNN1.12%error,notrealism.
  Modefieldsfloat32duplicatepointbitcheckFAIL2.66e-7relative;PSDunaffected.
  CorrectedFEM43/48crossconvergence,allDEV;5TRAINwarnings. Crossgain≠selfportproof.
- **SonicGauss CLOSED variants:** energy a4c23b52…spectral/envelopeworse;
  waveform8d702d11…18/24quieter,shape/levelworse;flow6c2f1eb1…REJECTED.
  Codecprobe isAUDIO-INPUT,notgeneration/EQrepair. Exactruns/pins/resultsinpilot.
  No loss/gain/phase/seed/epoch/timing/attention/SED/codec/inventory sweeps.
- SonicGauss36TRAIN2/6/12/24/66/95,24DEV14/75/94/97;ALLauthorpretrainingTRAIN.
  `sonicgauss-cohort-data-2026-09-06`:10PLY60WAV,partialarchiveSHAonly;
  e905b8cb…data/93d397c1…projection. No36/70authorval/80/41/92payload.
  66IronNOTsteel;absolutephysicalsize/force/strikermaterialabsent.
  Keep17source/5weightpins/noT5pickle;cachedreplay≠GSreencode. NativeSDPA/flash
  disablingchangesPTv3patch1024→128. Reproductioninpilot/shared_fit.py.
- No2DNeuralResonator sweeps:physicalratiosworse. SonicGauss doubledscale→EXACT
  sameinputs(`sonicgauss-input-probe-2026-09-06`);oldmodalcontrol isDiffSoundmath.
- **Retained:** [water](/home/kaifaty/.codex/experiments/nextengine/physical-sound/pouring-tango-bridge-centered-2026-09-05/comparison.wav),13.74s/glass10seed2718,
  8/8coarseWater,notmaterial/flowcalibrated. [Rubber/glass hybrid](/home/kaifaty/.codex/experiments/nextengine/physical-sound/texture-hybrid-dc-glass-standalone-2026-09-05/comparison.wav),3.15s/40mm/s/.5N/90mm,
  NN+48TRAIN513tapunitDCFIR,notfrictionrealism. User:waternormal;rubber/glassunfamiliar.
  [Five glass impacts](/home/kaifaty/.codex/experiments/nextengine/physical-sound/syncfusion-adapter-standalone-2026-09-05/glass-rigid-motion-adapter.wav),863459fb…;notgeometry/size/force/striker.
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
