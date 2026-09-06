# Physical sound synthesis — current task state

Updated: 2026-09-06. Working context, not architecture authority.
Status: ACTIVE_GOAL / PHYSICAL_SHARED_AND_WARMSTART_REJECTED / OPERATOR_TRAINING_OPEN.

## Resume in 60 seconds

- **Full goal:** neural realistic impacts, friction, rolling, destruction, water,
  rain; both interacting materials, shape/size, force/speed, flow/rain intensity;
  new combinations without a target recording; internet data and automated
  training/validation/improvement without per-sound approval; eventual engine use.
  Neither audio reconstruction nor category-only generation satisfies this.
- **Latest audible discriminator:** [reference→standalone NN→NN+physics→P1+physics](/home/kaifaty/.codex/experiments/nextengine/physical-sound/objectfolder2-ritz-probe-assess-2026-09-06/contact-0-comparison.wav),12s;
  openDEV88, common gain ALL48contacts/variants. Two other contacts24/47 alongside.
  NN+physics improves standalone but loses to classical P1+physics: mean spectrum
  1.7559→.6522 versus.1400; envelope1.6170→.1381 versus.01446. REJECT NN advantage.
  Generation freshly assembles P2 from geometry, no target K/M/U/audio reads.
  Same32vectors/one inverse correction each; P1 initialization3.02s versus NN.074s,
  shared factor34.77s; total71.26s, not equal total costs or a runtime speed claim.
  Raw NN subspace Ritz frequencies68–174kHz: NO audible modes, rejection retained,
  no pitch repair. NN correction27/32audible, mean full32frequencyerror23.10%; P1
  32/32,2.81%. Residualmax NN.602/P1.130, neither converged. Surface-only training
  does not constrain interior fields; this is NOT a NeuralSound reproduction.
- **Latest standalone model:** physical-shared73e60eaa…; one74432parameter fit,
  TRAIN59/66/78,2000Adam.001seed42;32contact labels/body,16withheld. DEV88 remains
  whole-object opendevelopment, NOT pristineholdout. Geometry→frequency+vectorfield;
  physical per-mode port-matrix loss, not old bandGram or signedAudioNetg.
  [Three standalone DEV contacts](/home/kaifaty/.codex/experiments/nextengine/physical-sound/objectfolder2-physical-shared-assess-2026-09-06/object-88-three-neural.wav),9s.
  Beats nearest on66heldcontacts, loses59/78; DEV88 spectrum1.7559 versus1.1457,
  frequencyerror17.29%. Reject shared replacement; no per-object NN fit.
- **Physical data:** explicit approximate fTetWild59/66/78/88, fixed ceramics;
  P2 sixrigid+32elastic, residuals<1e-7, K/M/U retained externally. 66 requires
  421791DOFs/600.8s/15.76GiB peakRSS; explicit500000DOF option, default200000 kept.
  66/78/88 distance checks2048samples/direction, NOT allpoints/Hausdorff; volume
  differences.3271/.2074/.0984%. Strict59/78 failures preserved, no nativeexactclaim.
  Physical displacement, NOT pressure/radiation/realism;32modes notfullband.
  57tests/Ruff PASS; shared52WAVQA/replays/traces, newFEM12WAVQA/replays, Ritz15WAVQA/
  12replays/3galleries/geometry-onlytrace PASS. All jobs terminal; pins in pilot.
- Expansion:next256MiB,ten complete pairs,partial archive SHA only;30Iron/96Glass
  unsupported. Eight prepared bodies/32contacts; newTRAIN47/59/72/78/91.
  BEFORE audio reserved37–46cups/53possible54kin;newDEV37/40/53. CombinedDEV6objects,
  FOURfamilies;originalroles unchanged,mod5acquisitionrole separate,noholdoutreuse.
- Magnitude ef22695e… and11TRAIN81cc9a90… regress oldDEV; REJECTreplacement.
  Preserve compact5b569809…(.9873/.5622/.6498oldDEV); pins/cohort in pilot.
- Magnitude oracle: count alone insufficient; poles/fields both wrong. Sign/scalar
  fixes REJECT; signedMSE favorsquietfields, NOTproof of OF2sourcegaugeerror.
- Size: rawNN fails.8/1.25 law22.211%; analytic transport only≤10kHz, heldgains
  NOT amplitudephysics. PublishedRayleigh law5309modes matches6.66e-16; decay-only
  fix stillfailsDEV. No size/count/decay sweeps; exact diagnostics in pilot.
- **Closed OF2:** bandGram worsens spectrum; noGram/phase/band/decoder/mask/
  capacity/epoch/seed sweeps. Compact all-modes-accounted is lossy; pins in pilot.
- **Scale:** native mesh/source intervals agree;CSV23=.082 vs .116984,47=1.2 vs
  .191112. No CSV resizing/absolute calibration; scalar xyz span≠AABB longest side.
- **Next:** smallest shared operator-consistent TRAIN experiment: couple learned
  subspace to K/M through Ritz/residual, not independent poles and sparse surface
  labels. Compare against P1+onecorrection; geometry-only new-object inference.
  No further frozen-NN correction-count, epoch/width/seed/material-by-material
  sweeps. Reconsider only with a changed physical training mechanism. Full goal
  unchanged. Existing pytetwild0.4.2/scikit-fem12.0.2 overlays; details in pilot.
- **Source:** ObjectFolder2/rhgao revision3c6cd8930b2dcbadb6d94dadf2745c956bdcd236;
  `objectfolder2-source-2026-09-06` auditedaudioDDSP/MLP/CSV/paper/license.
  `objectfolder2-range-2026-09-06/family-extraction.json`:original9pairs/320MiB;
  IDs7/11/23/29/54/66/75/82/88,34–1965modes. **OF2IDs≠SonicGauss/OFRealIDs**.
  Demo23EXACTmatchesarchive;GSOCCBY4/originalmeshtermsretained;23/29URLsNone.
- Source loader:strictweights_only+numericNumPyallowlist/hash-pinnedASTdeclarations,
  noimports/CUDA/optimizerexecution. Authorpeaknormalization erasesforce/breakszero;
  omitted. Train rawcoefficients,NOTauditionPCM. Noauthorqueryclamp. Teacher29
  underflowbit-scalingFAIL,ULP27PASS/all9zeroexact; fullpack34–1965modes, in pilot.
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
