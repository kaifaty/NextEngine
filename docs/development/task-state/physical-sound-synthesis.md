# Physical sound synthesis — current task state

| Field | Value |
| --- | --- |
| Status | `PS2_PITCHER_COMBINED_PROTOCOL_REJECTED / CERAMIC_METHOD_DEVELOPMENT_CLOSED / IRON_SKILLET_METHOD_TRANSFER_REJECTED / FIXED_TAIL_TIMING_MISMATCH_SUPPORTED / BROADBAND_COMMON_POLE_SYNTHETIC_CONTROL_SUPPORTED / EXISTING_IRON_COUNTERFACTUAL_NEXT / MECHANICS_BLOCKED / PLANTER_AUDIO_SEALED / AUTHORED_CLIP_FALLBACK / EIGHT_EXACT_CLAIMS_OPEN / PASS_DISABLED / P1_BLOCKED` |
| Updated | `2026-08-28` |
| Task key | `physical-sound-synthesis` |
| Scope | Proposed architecture plus isolated fixed-point impact/demo and external controlled-corpus experiments |
| Definition of done | Demonstrate an external selective validator whose automatic pass has measured bounded false-pass risk on grouped real/mutation holdouts, with automatic clip fallback for OOD and no per-sound human gate |
| Authority | Working context only; Accepted SPEC/ADR, roadmap and exact future ProductCheck evidence outrank this file |

## Resume in 60 seconds
- **Current conclusion:** Repeated synthetic report `dcd83252…0094` supports
  the discovery-to-resynthesis candidate: 7 retained modes, maximum
  `0.095876 Hz/0.190671/s` errors and post-transient NRMSE `0.049273`.
- **Why:** Product-owner constraint dated 2026-08-27. Evidence is claim-scoped:
  external `E1` synchronized, `E2` transfer, `E3` identified-real and `E4`
  synthetic sources receive only the credit their bytes/metadata establish.
- **Next action:** Hash-close a read-only counterfactual on the already acquired
  Iron rows before any real-transfer claim; do not access new payload.
- **Current blocker:** Synthetic method support has no independent real-object
  validation. Real observation, mechanics and every exact-domain claim remain
  unproven.
- **Do not retry:** Treating synthetic-target match as glass identity, blind preset tuning, or using FAD, CLAP, ViSQOL, an aesthetic
  model or a general audio model as the sole quality judge. Also retain the ban
  on universal material sound, raw PhysX-callback mixing and local recording;
  do not grow ObjectFolder gzip prefixes, weaken the Freesound direct-endpoint
  policy or repeatedly probe HTTP 403 without a changed route/host condition.
- **Reconsider when:** P0 produces a measured quality/cost point and a concrete
  production impact consumer is selected.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Research report](../physical-sound-synthesis-research-2026-08-26.md) and [DiffSound trial](../physical-sound-diffsound-trial-2026-08-26.md) | `GLASS_09_Q30_DEMO_PASS / PHYSICAL_ID_OPEN` | Q30 tracks aligned f64 at `115.43 dB` SNR; product-owner A/B accepted B. Demo post-scale stays within one S16 LSB. Wrong geometry keeps material and wall thickness non-physical. |
| [Quality research](../physical-sound-quality-evaluation-research-2026-08-26.md), [automated validation](../physical-sound-automated-validation-research-2026-08-27.md), [AV-P0A](../physical-sound-validator-av-p0a-2026-08-27.md) and [AV-P0B](../physical-sound-corpus-benchmark-av-p0b-2026-08-27.md) | `AV_P0B_REAL_MATERIAL_BENCHMARK_MEASURED / NO_ACCEPTANCE_AUTHORITY` | External corpus: 15 real objects, 30 published variants and 13 engine candidates. BEATs real identity is `15/15`; engine shadow is `8/13`. Manifest/report hashes are frozen; recordings/models remain external and non-distributable. |
| [Automated steel search](../physical-sound-steel-automated-search-2026-08-27.md) | `BEATS_3_OF_3 / PANNS_0_OF_3 / FALLBACK_OUT_OF_DOMAIN` | Sparse tuning failed `0/165`; critical-band roughness yielded a BEATs-positive profile, but independent PANNs rejects it. No steel/demo promotion; v3 report/WAV repeat is byte-identical. |
| [Steel residual v4](../physical-sound-steel-residual-v4-2026-08-27.md) | `V4_REJECTED / FALLBACK_OUT_OF_DOMAIN` | YCB adds aluminium-container and steel-skillet families. Real metal flatness is about `−16/−17 dB` versus v3 `−53 dB`; v4 reaches flatness but not real spectral dynamics. Original PANNs stays `0/39`, BEATs `7/39`; no joint profile or promotion. |
| [Controlled glass corpus](../physical-sound-controlled-glass-corpus-2026-08-27.md) | `CONTROLLED_SYNTHETIC_CORPUS_PASS / HUMAN_REFERENCE_OPEN` | Exact geometry and 15 force/position conditions are reproducible; Q30 and physical controls pass, but whole-vector IDW is an inadequate spatial model and the corpus has no real matched recording. |
| [Steel calibration](../physical-sound-steel-calibration-2026-08-26.md) and [wood/glass calibration](../physical-sound-wood-glass-calibration-2026-08-26.md) | `WOOD-B_ACCEPTED / GLASS-D-F_REJECTED / GLASS-G_PARTIAL_ACCEPT / GLASS-H_WEAK_PREFERENCE` | Keep H as provisional baseline and G as its close control; stop near-neighbor tuning. |
| [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md), [Kronland/split evidence](../physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md) and [subsystem roadmap](../../plans/physical-sound-synthesis-roadmap.md) | `GLASS_16_OF_16 / REJECT_PARENTS_48_OF_35 / PROJECT_DISJOINT_SPLIT_VERIFIED` | Eight E3 projects repeat at 64 objects/135 recordings. Four dual-role projects support a frozen `2/2/2/2` split; every partition has target and reject evidence, but no domain admission or quality credit. |
| [Blue Bowl](../physical-sound-realimpact-blue-bowl-cross-tier-ps2-2026-08-28.md), [Shell Plate](../physical-sound-realimpact-shell-plate-range-pilot-ps2-2026-08-28.md) and [Skull Cup](../physical-sound-realimpact-skull-cup-range-pilot-ps2-2026-08-28.md) evidence | `FIVE_REALIMPACT_E2_OBJECTS / CROSS_TIER_SHELL_AND_VESSEL` | All bounded profiles reproduce twice. Object 6 links E3 to E2; 51 adds a broad shell and 60 a narrow Beer_Glass/SkullCup transfer. No E3 group is added and all five rows remain fallback-only. |
| [Exact-domain claim matrix](../physical-sound-domain-claims-matrix-ps2-2026-08-28.md) | `DOMAIN_EVIDENCE_INCOMPLETE / ONE_CROSS_TIER_LINK / EIGHT_REQUIRED_CLAIMS_OPEN` | The hash-closed gate accepts only the reviewed Blue Bowl identity, confirms useful E2/E3 observations and returns fallback. Numeric object 94 is rejected as a false join. Every frozen partition has zero exact-domain-eligible objects. |
| [Internet-source feasibility](../physical-sound-internet-source-feasibility-ps2-2026-08-28.md) | `REVIEWED_SOURCES_CANNOT_CLOSE_V1 / INTERNET_NATIVE_TRANSFER_CANDIDATE` | Seven frozen primary artifacts across REALIMPACT, ObjectFolder Real and AV-MSF reproduce byte-identically. Zero of eight blockers close. REALIMPACT may calibrate normalized transfer only; exact-domain admission remains disabled. |
| External REALIMPACT transfer V1 reports `ad1e7514…c321` | `INVALID_METRIC_CONFOUND / BYTE_IDENTICAL_REPEAT` | The frozen `2 dev / 1 calibration / 1 holdout / 1 reserve` run selected 16 modes and crossed its provisional holdout thresholds, but repeated low-frequency peaks shared damping bins and reused tail matches. The apparent recall is not admissible modal evidence. V1 remains immutable negative lineage. |
| [REALIMPACT transfer calibration](../physical-sound-realimpact-transfer-calibration-ps2-2026-08-28.md) and V2 reports `01346767…c444` | `RELATIVE_MODAL_DAMPING_SUPPORTED / SPATIAL_UNAVAILABLE / BYTE_IDENTICAL_REPEAT` | Injective V2 selected 16 modes on Shell Plate and passed the fresh Skull Cup gates: recall `0.5625`, frequency error `28.0099` cents, decaying fraction `0.5625`, tail RMSE `6.0133` dB. One listener row per object prevents spatial credit. This is extractor-transfer evidence, not material identity, quality, admission or runtime authority. |
| [REALIMPACT multi-listener acquisition](../physical-sound-realimpact-multilistener-acquisition-ps2-2026-08-28.md) and reports `cef5d381…7680` | `MULTILISTENER_DEV_BLOCK_VERIFIED / SPATIAL_MODEL_NOT_EVALUATED` | Two bounded runs prove Green Goblet rows `0..14` share vertex/angle/distance and span microphone IDs `0..14`; manifest `fcf44d41…50de` and raw block `8bcffd0a…75ca` repeat exactly. No calibration/holdout or spatial claim exists. |
| [REALIMPACT vertical spatial calibration](../physical-sound-realimpact-spatial-calibration-ps2-2026-08-28.md) and reports `abc13a9c…989d` | `RELATIVE_VERTICAL_SPECTRAL_PARTICIPATION_SUPPORTED / BYTE_IDENTICAL_REPEAT` | Manifest `077b9a46…f2cb` freezes Green/Shell/Skull before new row access. Shell selects RBF; selection `0aea2b62…0320` precedes Skull. Skull passes median `4.6881 dB`, p90 `13.0040 dB`, persistent median `4.0082 dB`, baseline ratio `0.7623` and improved fraction exactly `0.5`. Credit is one fixed-angle/distance vertical line only. |
| [REALIMPACT multi-object spatial-axis extension](../physical-sound-realimpact-spatial-extension-ps2-2026-08-28.md) and reports `77a1f9e9…356a` | `FIXED_RBF_MULTIOBJECT_AXIS_REJECTED / BYTE_IDENTICAL_REPEAT` | Green passes `6/6`; frozen Blue/Glass evaluation requires both. Glass passes `4/6`; Blue `3/6` and its base ratio `0.9923`/improved fraction `0.4375` reject the generic RBF. Preserve Blue/Glass as holdout evidence; next use a fresh split for shape conditioning. |
| [REALIMPACT shape-conditioned spatial calibration](../physical-sound-realimpact-shape-spatial-calibration-ps2-2026-08-28.md) and reports `3d18358b…4962f` / `ffb17687…aad6` | `BBOX_CONDITIONED_BANDWIDTH_CALIBRATION_REJECTED / BYTE_IDENTICAL_REPEATS / HOLDOUT_UNOPENED` | Ten fresh development objects fit one bbox/aspect/impact-conditioned object bandwidth. Both calibration candidate gates pass and p90 improves, but median ratio `1.0122` fails `0.95` and max ratio `1.0242` fails `1.0`. Do not retune; preserve two ceramic holdouts for a frequency-conditioned candidate. |
| [REALIMPACT frequency-conditioned spatial calibration](../physical-sound-realimpact-frequency-spatial-calibration-ps2-2026-08-28.md) and reports `48a150d7…19b` / `42b6605d…983` | `FREQUENCY_CONDITIONED_BANDWIDTH_CALIBRATION_REJECTED / BYTE_IDENTICAL_REPEATS / HOLDOUT_UNOPENED` | Twelve development objects fit 192 per-mode `kL`/shape targets. Both fresh candidate gates pass, but median ratio `1.0146` fails `0.95` and max ratio `1.0266` fails `1.0`; the frequency coefficient collapses near zero. Retire RBF-bandwidth tuning and research modal-radiation representation. |
| [REALIMPACT modal-radiation representation diagnostic](../physical-sound-realimpact-modal-radiation-representation-ps2-2026-08-28.md) and reports `7cbf7c59…f25` | `COMPLEX_MULTIPOLE_REPRESENTATION_REJECTED / BYTE_IDENTICAL_REPEAT / NO_FRESH_DATA_OPENED` | Order-3 axisymmetric complex multipoles pass `14/14` absolute gates but lose all comparison gates: median ratio `1.2506`, max `2.0861`, p90 `+7.2513 dB`, improved fraction `0.3482`. Do not raise order on opened rows; prove a classical surface-mode/BEM target first. |
| [Analytical boundary-solver control](../physical-sound-bem-analytical-control-ps2-2026-08-28.md) and reports `6f74a309…a689` | `CLASSICAL_BOUNDARY_SOLVER_ANALYTICAL_CONTROL_REJECTED / FOUR_OF_FIVE_NUMERIC_GATES_PASS / BYTE_IDENTICAL_REPEAT` | The 320-panel sphere stays within `2.1251%`, `0.1827 dB`, `0.2993°` and `0.0002 dB` direction span, but median error is `2.8589x` the 80-panel result. Preserve the harness; diagnose convergence synthetically before BEM/FFAT oracle credit. |
| [BEM panel-quadrature discriminator](../physical-sound-bem-quadrature-discriminator-ps2-2026-08-28.md) and reports `7460750e…b48d` | `SEVEN_POINT_PANEL_QUADRATURE_HYPOTHESIS_REJECTED / V1_REPORT_PRESERVED / BYTE_IDENTICAL_REPEAT` | Fine median error is `1.0447x` control and fine/coarse ratio is `2.6109`; ordinary higher regular-panel quadrature is not the missing control. Escalate to independent Galerkin/singular treatment. |
| [Triaxial prescribed-mode evidence](../physical-sound-bempp-triaxial-full-angular-cooker-ps2-2026-08-28.md) and [elastic FEM/Bempp evidence](../physical-sound-fem-eigenmode-bempp-cooker-ps2-2026-08-28.md), reports `a58f28b9…90ef` / `a71515fa…a667` / `c3101130…b476` | `ELASTIC_FEM_MODE_TO_BEMPP_SUPPORTED / FULL_ANGULAR_NEAR_TO_FAR_COOKER_SUPPORTED / COARSE_PROTOCOL_REJECTED / BYTE_IDENTICAL_REPEATS` | Coarse FEM is an immutable negative. The refined `128/512/2048` schedule passes all 16 FEM/Bempp gates; the cooker rejects `m=0` and selects degree 2 at `0.030975` max error across 112 held far conditions. Credit remains synthetic only. |
| [REALIMPACT geometry-spatial-transfer preregistration](../physical-sound-realimpact-geometry-spatial-transfer-preregistration-ps2-2026-08-28.md), report `c2f51cff…01ef` | `REAL_SPATIAL_TRANSFER_PROTOCOL_FROZEN / RESERVED_AUDIO_BYTES_ZERO / BYTE_IDENTICAL_REPEATS` | Pitcher calibration and Planter one-shot holdout, payload identities, `90/510` 3D split, geometry-spectral Bempp/cooker candidate, controls, gates and fallback are immutable. Next run geometry-only preflight; no real-transfer credit exists. |
| [REALIMPACT geometry-only preflight](../physical-sound-realimpact-geometry-preflight-ps2-2026-08-28.md), report `2fb9fd0f…e25d` | `GEOMETRY_PREFLIGHT_V1_REJECTED / RESERVED_AUDIO_BYTES_ZERO / BYTE_IDENTICAL_REPEATS` | Published OBJ indices form disconnected triangle soup. Exact-coordinate welding later recovers one closed manifold per object without tolerance/repair; preregister that single V2 change before reuse. |
| [REALIMPACT exact-weld geometry V2](../physical-sound-realimpact-exact-weld-geometry-preflight-ps2-2026-08-28.md), report `c1c86f78…7e5d` | `EXACT_WELD_GEOMETRY_PREFLIGHT_SUPPORTED / BYTE_IDENTICAL_BLOCKS / RESERVED_AUDIO_BYTES_ZERO` | Exact welding closes both surfaces; `8192/2048` topology, 64 modes and residual gates pass. Geometry blocks are frozen; next preregister Pitcher calibration before audio. |
| [REALIMPACT Pitcher calibration preregistration](../physical-sound-realimpact-pitcher-calibration-preregistration-ps2-2026-08-28.md), report `2ae1bc0b…9c72` | `PITCHER_CALIBRATION_PROTOCOL_FROZEN / BYTE_IDENTICAL_REPORTS / RESERVED_AUDIO_BYTES_ZERO` | Exact Pitcher geometry, one 512 MiB prefix, 600-row decoder, extractor/mapping, solver, `90/510` split, controls, gates and stop-before-Planter fallback are immutable. Next implement parity controls and execute the bounded calibration. |
| [REALIMPACT Pitcher runner preflight](../physical-sound-realimpact-pitcher-runner-preflight-ps2-2026-08-28.md), report `6e60d71f…fd2d` | `PITCHER_RUNNER_PREFLIGHT_SUPPORTED / EXTRACTOR_PARITY_PROVEN / RESERVED_AUDIO_BYTES_ZERO` | Exact Rust fixture repeats; Python recovers all 16 modes within `3.03e-12`; geometry/mapping/split pass and the bound Rust spatial projector is implemented. Audio execution stays disabled until a final execution manifest closes Bempp/cooker choices. |
| [REALIMPACT Pitcher execution preflight](../physical-sound-realimpact-pitcher-execution-preflight-ps2-2026-08-28.md), report `94d5e1e6…9b32` | `PITCHER_EXECUTION_PREFLIGHT_SUPPORTED / ONE_PREFIX_REQUEST_AUTHORIZED / RESERVED_AUDIO_BYTES_ZERO` | Manifest `8e791327…ba45` binds script, environment, original bbox centre, directions, Bempp/cooker, exact range/decoder and all frozen gates. The local full-angular control and extractor parity repeat; the one exact Pitcher request is next, while Planter remains sealed. |
| [REALIMPACT Pitcher serializer repair](../physical-sound-realimpact-pitcher-serializer-repair-ps2-2026-08-28.md), report `9c5c9ca8…471c` | `PITCHER_PREFIX_ACQUIRED / ROWS_DECODED / SERIALIZER_LINEAGE_REPAIR_PREFLIGHT_SUPPORTED / CALIBRATION_DECISION_NOT_PUBLISHED` | One request yielded prefix `a0dd7006…6cf5` and block `182f2010…1e0f`. Analysis failed at JSON serialization; repair `603c1185…28e3` then rejected correct parent lineage before block access. Successor `f51a6046…db7e` binds both failures, changes no numeric path, disables acquire/decode and repeats locally. |
| [REALIMPACT Pitcher calibration](../physical-sound-realimpact-pitcher-calibration-ps2-2026-08-28.md) and [causal audit](../physical-sound-pitcher-causal-audit-ps2-2026-08-28.md), reports `8bd5323c…1aea` / `68c79a37…5a57` | `PITCHER_COMBINED_PROTOCOL_REJECTED / OBSERVATION_ADMISSION_FAILED / BYTE_IDENTICAL_AUDIT / PLANTER_SEALED` | Frequency/field comparisons fail strongly, but the consumed observation also fails the earlier V2 decay gate and selects `11/16` peaks below `500 Hz`. Preserve the combined rejection without uniquely blaming mechanics; test observation first on unopened `78_CeramicCup`. |
| [REALIMPACT Ceramic Cup observation result](../physical-sound-realimpact-ceramic-cup-observation-result-ps2-2026-08-28.md), reports `b6d25bc6…6a0c` / `9f1c2311…daf0` / `56591bb8…3fd9` | `CERAMIC_CUP_OBSERVATION_REJECTED / REPEATED_DECAY_GATE_FAILURE / MECHANICS_BLOCKED / PLANTER_SEALED` | Exact acquisition and 600-row decode repeat. Four V2 gates pass; decay is `0.25 < 0.50`, with `13/16` peaks below `500 Hz`. Freeze a fixed-axis offline diagnostic; do not try another object, tune or run physics. |
| [Ceramic Cup observation diagnostic result](../physical-sound-ceramic-cup-observation-diagnostic-result-ps2-2026-08-28.md), report `47b578ac…2603` | `SHARED_DECAY_MISMATCH_SUPPORTED / LISTENER_LOCAL_REJECTED / SIMPLE_LOW_FREQUENCY_CAUSE_REJECTED / NO_NEW_PAYLOAD_OR_PHYSICS` | `23/27` rows fail; every axis exceeds the frozen shared threshold. Low/high fitted-decay fractions are `0.3583/0.2252`, rejecting the cutoff hypothesis. Next prove a multi-output estimator synthetically before reusing real rows. |
| [Ceramic Cup adaptive counterfactual result](../physical-sound-ceramic-cup-adaptive-counterfactual-result-ps2-2026-08-28.md), report `2ec3b03e…d6b7` | `CERAMIC_ADAPTIVE_COUNTERFACTUAL_REJECTED / CERAMIC_METHOD_DEVELOPMENT_CLOSED / SALIENCE_SELECTOR_SYNTHETIC_CONTROL_NEXT` | Only `6/16` fits are valid; decay/improvement are `0.375/0.1875`. Primary source uses ±`10%` salience suppression rather than V2's dense low-frequency selection. |
| [Iron selector/tail diagnostic](../physical-sound-realimpact-selector-tail-diagnostic-result-ps2-2026-08-28.md), [subband result](../physical-sound-subband-common-pole-control-result-ps2-2026-08-28.md) and [broad-band result](../physical-sound-broadband-common-pole-control-result-ps2-2026-08-28.md), reports `02551f29…48d1` / `3fcb5fc8…2284f` / `dcd83252…0094` | `FIXED_TAIL_TIMING_MISMATCH_SUPPORTED / BROADBAND_COMMON_POLE_SYNTHETIC_CONTROL_SUPPORTED / EXISTING_IRON_COUNTERFACTUAL_NEXT` | New-seed input discovery, duplicate pruning, amplitudes and reconstruction pass all 15 synthetic gates. Real transfer, onset/tail choice, quality, mechanics and admission remain open. |
| [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md) and current `AudioSceneSnapshotV1`/`AudioMixerV1` | `CURRENT_BASELINE_OBSERVED` | Clip playback, canonical PCM and gameplay/output separation remain the promoted baseline; the physical source synth is isolated experimental code. |
| [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md) versus current Rust `ContactEventV1` | `IMPLEMENTATION_GAP_OBSERVED` | Normative contact facts include velocity/impulse/effective mass/tags, but current record omits them; production audio must close the existing projection rather than consume raw callbacks. |
| `xtask physical-sound-lab` external audition and cost report | `PASS / NON_GATING_COST` | Frozen baselines remain exact; selected Q30 WAV SHA is `c912806c…b9c823`. On Ryzen 3950X, 16 voices cost `1.483/1.683 ms` p50/p99 per 1,600-frame lab tick, `5.05%` of that window; this is not a whole-engine budget. |
| Glass-object set and DiffSound audition | `DIFFSOUND_AND_Q30_PERCEPTUAL_ACCEPT / UNCALIBRATED` | Selected `09` and transferred Q30 B sound glass-like; identity and reference fidelity remain separate, and one archetype is not a generic glass model. |
| Product-owner wood/glass audition, 2026-08-26 | `WOOD-B ACCEPT / GLASS-D-F FAIL / G PARTIAL_ACCEPT / H WEAK_PREFERENCE` | H is tentatively better but hard to distinguish from G; preserve both and require a stronger discriminator. |
| Feature-gated demo enabled/disabled regression | `PASS` | Committed `Begin` contact changes only PCM; runtime, RPG and physics checkpoint state remain identical. |
| 120-frame SDL/Ash reference demo with `physical-sound-selected-glass` | `PASS / DEBUG_FUNCTIONAL` | Selected-Q30 run: 56 simulation ticks, active audio, 112,000 queued samples, zero drops/faults; 49 debug underruns grant no platform/performance credit. |
| `audio-scene`, `play`, `host-check` | `PASS` | Baseline audio/play roots remain valid; workspace fmt/clippy/tests and boundary policy pass on Rust 1.97.1. |
| Candidate `AUDIO-PHYS-*` checks | `NOT_RUN / NOT_PROMOTED` | Local experiment checks do not create source/content/platform/performance or P1 admissibility. |

## Decisions that still constrain the work

### D-001 — Physical audio is a presentation consumer, not a world owner

- **Observation:** Gameplay hearing already consumes deterministic
  `AcousticFactV1`; PCM/device state is presentation-only.
- **Evidence:** SPEC-08, SPEC-30 and ADR-027 prohibit presentation feedback and
  second physical writers.
- **Decision:** Consume only committed immutable physical facts; source synth,
  propagation and mixer state remain reconstructible and unsaved.
- **Rejected alternatives:** Deriving AI hearing from waveform energy or
  letting audio write materials/contact/gameplay state.
- **Consequences:** Feature enabled/disabled/faulted must preserve every
  gameplay, ledger, physics, acoustic-fact and persistence root.
- **Uncertainty:** Exact presentation epoch/preroll behavior for canonical
  capture remains to be calibrated.
- **Reconsider when:** Never for authority; only a new superseding Accepted ADR
  could change the product contract.
### Stable carried decisions

Detailed rationale is retained in SPEC-45 and the linked dated research. The
resume-critical consequences are:

| ID | Decision | Reconsideration condition |
| --- | --- | --- |
| D-002 | Start with cooked modal rigid impact; persistent contact and other source classes remain separate. | Exact-domain evidence shows another bounded source model is required. |
| D-003 | Acoustic profiles remain PresentationOnly and separate from physics materials. | A concrete consumer proves a field is a shared physical source of truth. |
| D-004 | Production wiring waits for the engine-owned committed contact projection and `PHYS-COLLISION-P1`. | The projection and its product evidence exist. |
| D-005 | Classical cooked reference, offline fitting and clip fallback precede any runtime learning. | A controlled residual target and immutable non-neural fallback exist. |
| D-006 | R8 remains an isolated experiment with no stage activation or shipping claim. | A roadmap slot and player-visible consumer are selected. |

### D-008 — Transfer V1 is preserved as an invalid metric lineage

- **Observation:** The repeated report passed provisional thresholds while its
  mode list exposed duplicate tail assignments and identical damping tracks
  for multiple unresolved peaks.
- **Evidence:** V1 report SHA-256
  `ad1e75149f8feb7016c7fb2f8c652a16e40d95112b112ddb14dda1e00a27c321`;
  V2 manifest/report SHA-256
  `52dbdc59235dfe88f1533dab5c1b11e1226318e1e50727300b84bdb94441e383`
  and `01346767b596630061fe437e98d5213bf426e49e5b96c22565a77acfea50d444`.
- **Decision:** Do not reinterpret V1 as modal success or retune it after the
  opened Shell holdout. V2 requires injective matching and frequency-
  resolution separation and used the previously unopened Skull row once; keep
  its credit limited to relative modal/damping transfer.
- **Rejected alternatives:** Raising the V1 threshold after inspection,
  reporting the formal threshold pass, or opening Skull during diagnosis.
- **Consequences:** V2 used Shell Plate for selection and Skull Cup once as the
  fresh holdout. Its injective 16-mode extractor passes the frozen relative
  modal/damping gates, but spatial fitting remains blocked on multi-listener
  rows and all exact-domain claims remain open.
- **Reconsider when:** Never for V1; a separately hash-closed V2 may supersede
  only the extraction method, not the historical report.

### D-009 — Listener blocks require source-order proof and preregistered opening

- **Observation:** The official preprocessing loops over 15 microphones inside
  each valid impact condition, but array position alone would still be an
  inference without checking the published annotation bytes.
- **Evidence:** Frozen script hashes plus repeated acquisition profile/manifest/
  report/block SHA-256 `c2301789…7e28`, `fcf44d41…50de`,
  `cef5d381…7680`, `8bcffd0a…75ca`; preregistration `077b9a46…f2cb`,
  selection `0aea2b62…0320` and spatial report `abc13a9c…989d`.
- **Decision:** Grant multi-listener acquisition credit only when scripts and
  row arrays agree on impact, angle, distance, microphone and listener
  coordinates. The Green/Shell/Skull experiment may grant only its
  preregistered fixed-angle/distance vertical-line claim; keep the selected RBF
  immutable for further object holdouts.
- **Rejected alternatives:** Treating 15 adjacent rows as listeners by shape
  alone, reporting raw amplitude variation as a spatial model, or inspecting
  Shell/Skull blocks before freezing the decision rule.
- **Consequences:** Green is development, Shell calibration and Skull the
  one-shot holdout. Rows `1..14` are now opened under the frozen manifest and
  repeat exactly. The later Blue/Glass axis extension rejects the generic RBF;
  those objects remain holdout evidence and cannot be used for retuning. A
  separate 10/2/2 split rejects one bbox-conditioned object bandwidth during
  calibration; its per-mode frequency-conditioned successor also fails fresh
  calibration. Both ceramic holdouts remain unopened.
- **Reconsider when:** Only a new source revision changes preprocessing order or
  a preregistered experiment proves the current split cannot test the intended
  spatial claim.

### D-007 — Quality uses an internet corpus and automatic selective ensemble

- **Observation:** Per-sound audition cannot scale, and the product owner will
  not perform physical capture; required real evidence must be found online.
- **Evidence:** Product-owner constraint 2026-08-27 plus the automated-validation
  report, external corpus pilots and selective-risk results.
- **Decision:** Build a versioned internet-source/corpus/formula registry and an
  offline oracle from claim-scoped `E1`–`E4` evidence, specialists, grouped
  holdouts and calibrated risk. Outcomes remain `Pass`, `Reject` or clip fallback.
- **Rejected alternatives:** Local mic/hammer acquisition, a live human queue,
  one general score/model, or inventing absent axes across datasets.
- **Consequences:** Force hardware is no blocker. Bounded fetch/cache/archive
  handling, eight-project E3 normalization and five REALIMPACT E2 rows exist.
  Glass `16/16`, reject parents `48/35` and the verified four-partition split
  close corpus structure. The matrix proves one cross-tier link; the source
  gate closes zero V1 blockers. REALIMPACT V2 supports relative modal/damping;
  the RBF is only a narrow conditional pilot after fixed, bbox/frequency and
  complex-multipole generalization failures.
- **Uncertainty:** Real angle/distance/3D radiation, material identity and
  every exact-domain admission claim remain unevaluated or unsupported.
- **Reconsider when:** Only an explicit product-owner reversal permits local
  capture; validator simplification still requires equal bounded risk/coverage.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: A compact modal core plus fused transient can evoke glass | Product-owner selected `09`; exact synthetic geometry now preserves position/force controls through Q30 | Synthetic FEM has no matched real recording and uses idealized support/damping/radiation | Fit surface mode shapes against published `E2` transfer responses and validate identity on independent `E3` recordings |
| H2: The complete SPEC-26 contact projection is sufficient for impact excitation | It includes identity, point, velocity, impulse bounds, effective mass and tags | Current implementation omits the decisive numeric fields; solver-force fidelity is untested | Close one fixture projection and compare against exact synthetic excitation/control PCM |
| H3: Fixed-point reference resonators can meet both exact PCM and quality | Selected `09` repeats exactly; controlled-corpus Q30 RMS error is at most `7.987e-8` | One synthetic object is not a real quality or whole-mixer envelope | Preserve exact transfer while fitting only against held-out published real evidence |
| H4: Rolling/scraping can use the ordinary committed contact stream | Rolling/contact synthesis prior art exists | High-quality work identifies micro-collision, chattering and stick-slip gaps | P2 speed/load/roughness corpus with resting/separation controls; add one flexible-contact counterfactual only if it fails |
| H5: Physical synthesis fits a useful whole-mixer budget | 16 selected voices cost `1.683 ms` p99 in the isolated lab tick; cooked payload is 1,536 bytes | Measurement excludes normal mixer, callback/device and varied voices; no product budget exists | Measure full mixer/callback p95/p99 on a declared production consumer before setting a budget |
| H6: A selective specialist ensemble can safely automate admitted impact domains | Synthetic controls and exact-weld execution are reproducible | Ceramic failure is shared across listeners; the V2 single-output decay gate is mismatched and no replacement is validated | Prove a 15-output spatial-energy decay estimator on known synthetic modes before any real-row counterfactual |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), [SPEC-00](../../architecture/00-product-contract.md) and [SPEC-01](../../architecture/01-system-architecture.md).
2. [SPEC-08](../../architecture/08-audio-navigation-and-world-services.md), [SPEC-26](../../architecture/26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-30](../../architecture/30-presentation-extraction-and-render-content.md), ADR-027/046/058/071.
3. [SPEC-45](../../architecture/45-physical-sound-synthesis-and-acoustic-presentation.md).
4. [Research report](../physical-sound-synthesis-research-2026-08-26.md), [quality evaluation](../physical-sound-quality-evaluation-research-2026-08-26.md),
   [automated validation](../physical-sound-automated-validation-research-2026-08-27.md), [AV-P0C](../physical-sound-validator-av-p0c-2026-08-27.md), [PS-1](../physical-sound-validator-ps1-2026-08-27.md), [internet corpus policy](../physical-sound-internet-corpus-policy-ps2-2026-08-27.md), prior E3/E2 pilots, [Kronland/split freeze](../physical-sound-kronland-reject-split-freeze-ps2-2026-08-28.md), [exact-domain matrix](../physical-sound-domain-claims-matrix-ps2-2026-08-28.md), [internet-source feasibility](../physical-sound-internet-source-feasibility-ps2-2026-08-28.md), [transfer calibration](../physical-sound-realimpact-transfer-calibration-ps2-2026-08-28.md), [multi-listener acquisition](../physical-sound-realimpact-multilistener-acquisition-ps2-2026-08-28.md), [vertical spatial calibration](../physical-sound-realimpact-spatial-calibration-ps2-2026-08-28.md), [multi-object extension](../physical-sound-realimpact-spatial-extension-ps2-2026-08-28.md), [shape calibration](../physical-sound-realimpact-shape-spatial-calibration-ps2-2026-08-28.md), [frequency calibration](../physical-sound-realimpact-frequency-spatial-calibration-ps2-2026-08-28.md), [modal-radiation diagnostic](../physical-sound-realimpact-modal-radiation-representation-ps2-2026-08-28.md), [PS-2 plan](../physical-sound-corpus-plan-ps2-2026-08-27.md) and the [implementation plan](../../plans/2026-08-27-physical-sound-domain-admission-implementation-plan.md).
5. [Roadmap](../../roadmap.md) only for a future scheduling/scope decision.

## Next action

1. Preserve every frozen evidence hash externally; never retune opened data or
   reinterpret a control pass as quality, causality or P1 evidence.
2. Freeze new-seed synthetic broad-band band selection, duplicate clustering,
   amplitude/pruning and reconstruction; no real object, physics or Planter.
3. Promote only after measured success and a consumer ADR; otherwise retain the
   unchanged authored-clip fallback.

## Do not retry

- Universal acoustic material/body schema before a consumer — one model does
  not cover rigid contact, cloth, fluids, fire and biological sources.
- Raw PhysX callback to mixer — violates engine-owned stable projection and
  leaves replay/order/backend semantics undefined.
- Runtime neural residual first — there is no measured residual or bounded
  classical comparator yet.
- Blind acoustic tuning or more stationary-white residual gain/T20 — without
  grouped holdouts it optimizes the score; v4 overfills the tail and shifts
  BEATs from metal toward glass.
- Single-metric or single-model judge — FAD/CLAP/ViSQOL/aesthetic/audio-language
  outputs are complementary diagnostics and individually gameable.
- Per-sound `NeedsHumanAudit` as the end-state — uncertainty must select the
  authored fallback; human evidence is optional frozen data/audit, not an asset gate.
- Local microphone/hammer capture — required real evidence is internet-sourced;
  unavailable published axes remain unavailable or fallback-only.
- “Thousands of synthesized birds” based on Lyrebird — the cited repository is
  mostly field-audio data, not evidence for that claim.

## Handoff

- **Workspace state:** Registry V1, PS-1, PS-2 corpus/evidence paths, repeated
  empirical rejections, converged elastic FEM→Bempp, a full-angular Rust cooker
  and synthetic broad-band common-pole support exist; public schemas/ownership are unchanged.
- **Checks:** V2 report `c1c86f78…7e5d` and blocks `bcd54087…9acc` /
  `9310910f…5431` repeat with zero audio; V1 remains rejected. Prior protocol
  and FEM/Bempp/cooker hashes remain exact; both payloads stay sealed.
- **Remaining risk:** eight exact-domain claims, real 3D transfer, calibrated
  OOD/shadow risk, contact sufficiency, mixer cost and authoring are open.
- **Quality status:** V2 supports relative modal/damping extractor transfer;
  the RBF supports only a narrow pilot; wider kernels and compact multipoles
  fail. The matrix remains fallback-only, and no calibrated validator release,
  perceptual quality, corpus admission or production claim exists.
- **Promotion needed:** Concrete consumer, later ADR-046 promotion, then exact content/contact/DSP profiles and ProductChecks.
