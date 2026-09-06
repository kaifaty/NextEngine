# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / NINE_INTERNET_NEURAL_TEACHERS_AUDIBLE / SHARED_STUDENT_NEXT.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest primary:** [wood block→steel mug→ceramic bowl→plastic dish](/home/kaifaty/.codex/experiments/nextengine/physical-sound/objectfolder2-render-2026-09-06/wood-steel-ceramic-plastic.wav),13.5s.
  Nine PUBLISHED per-object neural teachers, NOT ournewsharedfit/generalization.
  `objectfolder2-render-2026-09-06/cohort.json`:127WAVs,9exactfirst-contactreplays,
  6testsPASS,allfinite/peak≤.5. Alljobs terminal;no runtime/demo/promotion.
- **Source:** ObjectFolder2/rhgao revision3c6cd8930b2dcbadb6d94dadf2745c956bdcd236;
  `objectfolder2-source-2026-09-06` auditedaudioDDSP/MLP/CSV/paper/license.
  `objectfolder2-range-2026-09-06/family-extraction.json`:9completemesh/checkpoint
  pairs from first320MiB of3.77GBarchive;individualhashes,NOTfullarchivechecksum.
  IDs7/11/23/29/54/66/75/82/88;34–1965modes. **OF2 IDs≠SonicGauss/OF-Real IDs**.
  Demo23mesh+weightsEXACTmatchesarchive(Steelassigned);notinferredfromtimbre.
  GSOCCBY4confirmed;originalmeshtermsretained;23/29CSVoriginalURLsNone.
- `physical_sound_objectfolder2.py`:strictweights_only+numericNumPyallowlist,
  reviewedhash-pinnedASTdeclarations,noimports/CUDA/optimizerexecution;
  rawsigned3-axisgainsat4fixedactualmeshvertices,frequency/dampingfromsource.
  Authoroutputpeaknormalizationerasesforceandbreakszero;omitted,commonper-object
  auditiongainused. Trainfromrawgenerated.npz,NOTaudition-normalizedWAVs.
  NativeCPUFIRcomparisonrelativeL2=6.459e-5after2sampledelay;notCUDA-bitclaim.
  Oneauthor-demoqueryoutsidecoordinatebounds;noinputclamporreselectedvertices.
- Numericbit-scalingFAILonwoodblock29underflow;retained. PCMhalf165/double337
  samplesdiffer≤1.4013e-45. Other8objectsbitpass;all9zeroexact. ExplicitULP
  roundingboundpassesall27scale/signchecks;0.51-vs0.5negativecontrolFAILasexpected.
  Thisisnumerics,NOTrealism/absolutephysicalforcevalidation. See latestpilot.
- **Next primary:** oneSHAREDstudent onTRAIN7/23/29/66/75/82,openDEV11/54/88,
  fromcohort.json;no studenttrainedyet. Needcommonrepresentationforvariablemodes,
  no silent1965→small-headtruncation. Produceheld-objectWAVswithsimplecontrols.
  DoNOTstartanotherinventory/per-objectfitcyclewhilethiscohortcantestlearning.
  TeacherhasNOpressure/radiation/listenerstageorstrikermaterial. SignedAudioNet
  gainsareNOTcontactself-admittance;do notfeedthemintoHertzfeedbackaspositiveports.
- **Pressure control retained:** `guitar-fsi-probe-2026-09-06`,DaRUS-3248V1,
  12WAVs/5tests;48statepassiveROMpressureerror6.12%,.1514freqL2insidepaperband.
  No-FSI→pressure/backzero,topmoves. No newbasis/order/epochsweeps orper-guitarfit.
  FixedmatricesNOmesh/recording/family;near-holepressureNOTfar-fieldmic.See pilot.
- **Our latest shared NN retained:** `modal3d-passive-fit-2026-09-06`,6805a313…;
  `modal3d-passive-render-2026-09-06`:63source-freeWAVs,25tests/59exactreplays.
  R_i=a_i(p)a_i(q),PSDself=a²(noabs/clamp),5064fieldparams,frozenbaaa9af5…freq.
  On48heldcuboidsNNspectrum.18726vsold.21774butinterp.15806:quality_advantageFALSE.
  `modal3d-passive-factorial-2026-09-06`:interp-freq/neural-port.13119,NOTnewNN.
  Keepasstrongcontrol;NOcuboidmesh/pulse/field/frequency/epoch/capacitysweeps.
  HertzusesbothE/nu;weakR5mmfailsimpulse+5.253%,coupledNN1.12%error,notrealism.
  Modefieldsfloat32duplicatepointbitcheckFAIL2.66e-7relative;PSDunaffected.
  `modal3d-corrected-data-2026-09-06`:43/48crossconvergence,allDEV;5TRAINwarnings.
  `modal3d-port-data-2026-09-06`addsactualports;crossgaincheck≠selfportconvergence.
- **SonicGauss two-sample fit REJECTED:** `sonicgauss-energy-fit-2026-09-06`,
  adaptera4c23b52…,fixedDEVaudiblespectrum.961561→.972393(5/24),envelopeworse.
  Freshenergyimproves.24%butshapeonly.34%;notquality.360WAV+80comparisonsinpilot.
  No lossweight/gain/seed/epoch/timing-sweeps,SEDvariants orcodec/inventorycycles.
- **Waveform candidateREJECTED:**`sonicgauss-waveform-odd-fit-2026-09-06`,
  adapter8d702d11…,72steps/262144params/full50Euler,frozenweights,oddFFTretained.
  DEVaudible.96156→.87239butlevel.43974→.63541,shape.91139→1.01524(4/24wins);
  18/24quieter;120WAV+60triplesinpilot. No repeatedloss/gain/phase/fitvariations.
  Wood14level/Plastic97all3metricsregress;oldraw-onlyrulepasses,newcombinedREJECT.
- 36TRAIN(objects2/6/12/24/66/95),24DEV(14/75/94/97),sixauthor-first/object.
  ALLknownpretrainingTRAIN;localobjectsplit≠pristineunseenobject. Source2.98sfit
  vsfull3sevalexplicit. Absolutephysicalsize/force/strikermaterialstillabsent.
- Codecprobe isAUDIO-INPUT,notgeneration/EQrepair;flowfit6c2f1eb1…REJECTED.
  See pilot;noflow/attention/lr/epoch/codec/pulse/phase/gain sweeps.
- `sonicgauss-cohort-data-2026-09-06`:10PLY+60WAV,pinnedranges/CRC/SHA;fullarchiveSHA NOTverified.
  Datasetrevisione905b8cb…,correctedprojection93d397c1…,range-root`sonicgauss-range-2026-09-06`.
  Objects2/6/12/14/24/66/75/94/95/97;66IronNOTsteel. No36/70authorval/80/41/92payload.
  Inputs/corpus separated;TRAIN-onlyfit/no-reference-rendertracesinpilot.
- SonicGauss pins/reproduction:pilot/`physical_sound_sonicgauss_shared_fit.py`.
  Cachedreplay≠GSreencode;keep17source/5weightpins,noT5/pickle/network.
  KeepnativeSDPA/flash(disablingchangesPTv3patch1024→128);fusionnotsolecause.
- Prior2DNeuralResonator fit improves12/16spectra but worsensphysicalratios;see pilot.No2Dsweeps.
- SonicGauss doubledscale→EXACTsameinputs:`sonicgauss-input-probe-2026-09-06`.
  Oldmodal-frequency-control.wav isDiffSoundmath,NOTneural/sizeevidence.
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
