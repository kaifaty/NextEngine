# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / POLE_AND_FIELD_ERRORS_SEPARATED / JOINT_QUALITY_REJECTED.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest standalone:** [reference→eleven-body NN→magnitude-factorized NN](/home/kaifaty/.codex/experiments/nextengine/physical-sound/objectfolder2-magnitude-comparison-2026-09-06/new-development-reference-baseline-candidate.wav),27s;
  newDEV37/40(polycarbonatecups)/53(ceramicbowl),samegainpertriple. No target
  acoustics at either generation. Threeobjects but TWO conservativefamilies.
- Expansion:next256MiB,ten complete pairs,partial archive SHA only;30Iron/96Glass
  unsupported. Eight prepared bodies/32contacts; newTRAIN47/59/72/78/91.
  BEFORE audio reserved37–46cups/53possible54kin;newDEV37/40/53. CombinedDEV6objects,
  FOURfamilies;originalroles unchanged,mod5acquisitionrole separate,noholdoutreuse.
- `objectfolder2-magnitude-fit-2026-09-06`:99337total/30851trainableparameters;
  frozen81cc9a90…core pluspositivemagnitudehead;notpureloss/capacitycomparison.
  Same11TRAIN7/23/29/47/59/66/72/75/78/82/91;one2000Adam.001seed42fit.
  Weightsef22695e230b3846e9747f0da98d9a99a088f6f0d5a903a5a5ed2d16b5f1a400.
  NewDEVspec/env/level1.16092/.78925/1.85511→1.33019/.86025/1.60353;
  oldDEV.88234/.73939/1.34949→.94375/.59154/.67852. Levelhelps,spectrumworse:
  REJECTgeneralreplacement. Preserveoldcompact5b569809…(.9873/.5622/.6498oldDEV).
- **Latest diagnostic:** [five steel interventions](/home/kaifaty/.codex/experiments/nextengine/physical-sound/objectfolder2-magnitude-diagnostic-2026-09-06/object-23-comparison.wav),15s.
  Compact target→unchanged NN→oracle count→oracle count/poles→oracle count/field.
  Frozen ef226…;all11TRAIN, NOT new fit/source-free improvement/realism evidence.
  Mean spec/env/level: raw .85287/.63762/.37247; count .85722/.63517/.36303;
  poles .68281/.54897/.41951; field .74861/.30288/.09554. Both errors remain.
  44tests/66WAVQA/55replays/11galleries/11prior-standalone identities PASS/EXACT;
  Pins/TRAIN-only/no-INET trace PASS; jobs terminal. Prior17core identities EXACT.
- `objectfolder2-field-cause-2026-09-06`:11TRAINoraclepoles/count/ranks,
  firstcontactaudio/all32coefficientdiagnostics,NOTstandalonegeneration.
  Sign-only/scalar complete fixes REJECT; signed MSE can prefer quieter fields.
  FEMa→aSphysicalresidues/waveEXACT;OF2signsensitivityNOTproofsourcegaugeerror.
- **Size:** `objectfolder2-size-final-2026-09-06`,rawNNfrequencyshiftmean.6165%
  despite.8/1.25size;required-lawerror22.211%,0/12modecounts. transportlambda/r²
  andRayleighdecayworkswithoutfit,heldgainsNOTamplitudephysics. Commonbaseline
  ≤10kHznaturalbandonly,noabove-Nyquistcoverageornewshape/realismproof.
- **Decay:** publishedOF2Rayleighlaw matches5309fullmodes,maxrelative6.66e-16.
  rayleigh.py usesdamped→undampedstablelowroot,keepsf/g/maskfixed. Priorchannel
  f5e5e578…rawDEV3.469/7.347/.838→analytic1.169/1.164/.579,stillREJECT.
- **Closed OF2:** bandGram worsens spectrum; noGram/phase/band/decoder/mask/
  capacity/epoch/seed sweeps. Compact all-modes-accounted is lossy; pins in pilot.
- **Scale audit:** native mesh/source coordinate intervals agree on11TRAIN;
  CSV23=.082 vs native .116984; CSV47=1.2 vs native .191112. Do NOT resize from
  CSV or claim absolute calibration. Scalar xyz span is not longest AABB side.
- **Next:** native non-cuboid TRAIN59 elastic-operator/physical-port audible
  control before shared operator-residual learning. 4674unique vertices/9344tris,
  indexed edges two-sided, NOT yet self-intersection/volume/tetra/solver verified.
  No silent geometry repair/resize or cuboid substitution. Published ceramic
  parameters; no sign/gain/width/epoch/seed or data-count sweeps. Full goal unchanged.
- **Source:** ObjectFolder2/rhgao revision3c6cd8930b2dcbadb6d94dadf2745c956bdcd236;
  `objectfolder2-source-2026-09-06` auditedaudioDDSP/MLP/CSV/paper/license.
  `objectfolder2-range-2026-09-06/family-extraction.json`:original9pairs/320MiB;
  IDs7/11/23/29/54/66/75/82/88,34–1965modes. **OF2IDs≠SonicGauss/OFRealIDs**.
  Demo23EXACTmatchesarchive;GSOCCBY4/originalmeshtermsretained;23/29URLsNone.
- `physical_sound_objectfolder2.py`:strictweights_only+numericNumPyallowlist,
  reviewedhash-pinnedASTdeclarations,noimports/CUDA/optimizerexecution;
  Authoroutputpeaknormalizationerasesforceandbreakszero;omitted,commonper-object
  auditiongainused. Trainfromrawcoefficients,NOTaudition-normalizedWAVs.
  Oneauthor-demoqueryoutsidecoordinatebounds;noinputclamp. Detailsinpilot.
- Teacher29underflowbit-scalingFAIL;ULPbound27PASS/all9zeroexact;detailsinpilot.
  Fullpack34–1965modes/32points,512vertices/log3sizes/materialonehot;
  no target count/poles/audio at standalone inference.
  TeacherhasNOpressure/radiation/listenerstageorstrikermaterial. SignedAudioNet
  gainsareNOTcontactself-admittance;do notfeedthemintoHertzfeedbackaspositiveports.
- **Pressure:** guitar-fsi:48stateROM6.12%error; no-FSI→pressure/backzero; fixed
  matrices,no mesh/recording/family,near-hole NOT far-field. No basis/order/epoch/per-guitar sweeps.
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
- SonicGauss TRAIN2/6/12/24/66/95,DEV14/75/94/97;ALLauthorpretrainingTRAIN.
  10PLY/60WAV,partial archive SHA only; no36/70authorval/80/41/92payload. Pins in pilot.
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
