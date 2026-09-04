# R8b first learned locomotion — current task state

| Field | Value |
| --- | --- |
| Status | `SELECTED / ACTIVE_R&D / V3_STANDING_NOMINAL_GATE_PASS / WALKING_V1_V2_V3_FAILED / RESEARCH_REQUIRED / MODEL_MIRROR_P1_FAILED / NO_RUNTIME_AUTHORITY` |
| Updated | 2026-09-04 |
| Task key | `r8b-first-learned-locomotion` |
| Scope | Produce the first visible learned standing and bounded forward start/stop checkpoints on an anatomically meaningful successor to the frozen Stage 0 V1 humanoid |
| Definition of done | Five fixed held-out CPU PhysX episodes pass the complete 3,600-tick standing gate, then five pass the bounded forward/start/stop gate, with exact manifests, zero declared safety events and replay-bound visual evidence when capture is available |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact run artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** V3 standing passes, but all three command-only walking
  attempts fail. Dense V3 survives five full GPU horizons yet travels only
  `0.104/7.258 m`; canonical CPU falls at tick `156` in all five episodes and
  moves `-0.822 m`.
- **Why:** V2's zero-gradient boundary was real, but removing it was
  insufficient. V3 materially increases the dense tracking return while the
  policy remains essentially stationary on GPU and diverges backward on CPU.
  Velocity tracking alone has not produced a gait under this lesson.
- **Next action:** Stop optimizer runs. The persistent-problem rule now requires
  a bounded research cycle that discriminates missing gait/contact curriculum,
  action/observation limitations and CPU/Isaac dynamics before any new profile
  or compute budget.
- **Current blocker:** Current CPU PhysX 5.9 / Isaac 5.1 GPU execution exceeds
  fixed correspondence limits. This no longer blocks the one discriminator,
  but still blocks runtime promotion, broader commands and any further budget.
- **Do not retry:** Never run/evaluate/resume standing V1, walking V1/V2/V3,
  the broad V1 PPO checkpoints or any R123–R141/TRAIN-5 artifact unchanged;
  they are either falsified for this question or have incompatible/rejected
  authority. Do not substitute PPO/noise tuning for the required research.
- **Kimodo finding:** NVIDIA Kimodo is a plausible future offline source of
  synthetic reference candidates, not a controller. It is excluded from the
  current R8b lineage and may be reconsidered only as a new post-foundation
  experiment with separate provenance, retarget and physical-admission gates.
- **Reconsider when:** Broaden beyond standing and forward start/stop only
  after both declared checkpoints pass. Reopen motion-reference work only by a
  separate roadmap decision satisfying its own byte-reversible lineage gate.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Roadmap R8](../../roadmap.md) | `R8B SELECTED / STAGE0_V1_BODY_RETIRED / BIOMECHANICS_SUCCESSOR_REQUIRED` | Keeps the visibility-first lineage active but prohibits further optimizer work on the toy V1 body |
| [ADR-064](../../architecture/adr/064-canonical-flat-command-locomotion-environment.md) | `Accepted`; frozen standing and flat-command V1 environments | Reuse exact engine-owned BodySchema, observation, action, safety and CPU PhysX boundaries |
| [ADR-065](../../architecture/adr/065-curriculum-flat-command-locomotion-profile.md) | Prior broad V1 PPO held-out survival about 135–137 ticks; `768/768` falls; bounded V2 first stage exists | Do not repeat broad commands; standing precedes only `0..0.75 m/s` forward start/stop |
| [Stopped TRAIN-4 state](humanoid-motor-training.md) | `R141_INVALID / STOP_NO_RETRY` | R8b consumes no motion corpus, reference tracker, R123–R141 cache/witness or rejected checkpoint |
| [Kimodo research](../kimodo-motion-training-research-2026-09-03.md) | Kimodo is an offline 30 Hz kinematic generator; SOMA output still needs an exact 23-DoF retarget and CPU PhysX admission | Keep it out of R8b; preserve one separate post-foundation `KIMODO-MOTION-P0` candidate |
| R8b standing profile | `nextengine.isaac-rsl-rl.rtx3080-standing.v1`; `128 x 32 x 1,000 = 4,096,000` transitions; seed `42`; five held-out seeds `1001..1005` | Exact first-run budget and evaluation matrix are frozen before model output |
| R8b external generation | `nextengine.training.generation.r8b-standing.v1`; manifest `094dd6ae…eb42f`; descriptor `50c55442…fc11`; USD `5524a778…e54d` | The old TRAIN-4 store and checkpoints are not reachable from the admitted closure |
| Canonical CPU V1 zero-action control | One production motor-lab slot falls at tick `220`; first tick reaches root velocity `1.205196 m/s` and joint speed `19.880161 rad/s` | The immutable `1.050 m` V1 reset has a real contact transient; do not misreport it as tangent-ground |
| R8b Isaac reset smoke | `PASS`, four slots, seed `1001`, exact automatic reset errors all zero; 10 zero-action ticks remain finite with max joint speed `8.995819 rad/s`, root speed `0.477639 m/s`, angular speed `0.541279 rad/s`, height overshoot `0.037621 m` | Pipeline/reset execution is ready only under the explicit legacy-V1 smoke envelope; this is not policy quality or correspondence evidence |
| R8b optimizer run | `r8b-standing-seed42-v1` completed all `4,096,000` transitions on clean commit `3095a9e0…`; manifest `2debfb23…`, 21 checkpoint hashes verified, final `model_999.pt` `993da891…`; metrics file `6091cb6a…` has 1,000 finite records but its hash is absent from the immutable manifest | Pipeline execution passed, but the run is not fully hash-closed, grants no quality claim and is rejected by current checkpoint selection |
| R8b standing outcome | Early-20 mean episode length `101.51`, final-20 `62.39`, best `111.16` at iteration 9 versus required `3,600`; final TensorBoard projection reports effort `-241,795,296`, action-rate `-1,513,819`, upright `0.480` and pose `-21.325` | Classify the first content failure as `Environment`: raw-unit penalties dominate and improvement in scalar return is anti-correlated with standing |
| Standing V2 reward identity | ADR-100; environment `nextengine.motor.env.humanoid-standing.v2`; manifest `b39b4ae2…5f04`; eight components each `[0,65536]` Q16; total `[-148768,114688]` | Reuse the frozen V1 body/control/termination while changing only the optimizer objective and current translator closure |
| Standing V2 CPU no-training probe | Production PhysX motor-lab, seed root `42…42`, one slot, 32 zero-action ticks; no terminal; all component/total bounds pass; observed total `32697..104730` Q16 | Reward scale is executable and commensurate on the canonical CPU path; this is not policy quality or CPU/Isaac correspondence |
| Standing V2 generation and GPU preflight | Generation `nextengine.training.generation.r8b-standing.v2`, manifest `8ab18b39…dee`; exact descriptor `89299e79…c90` and USD `5524a778…e54d`; four-slot ten-step reset/reward smoke passes with exact resets and bounded rewards | The intended clean V2 inputs execute on the RTX 3080; this is preflight, not correspondence or policy quality |
| Standing V2 optimizer run | `r8b-standing-v2-seed42-v1` completed `4,096,000` samples; metrics `3d734c3d…cf`, 1,000 finite records; all 21 checkpoint hashes close; final `model_999.pt` `0d7eb3e4…890` | Reward-scale fix succeeds numerically and materially improves training survival, but does not itself grant a standing claim |
| Standing V2 learning outcome | Early-20 reported mean length `99.72`; final-20 `1,721.33`; peak reported mean `2,623.84/3,600` at iteration `818`; final value loss `6.876`, maximum `260.783` | The optimizer acquires longer survival on the toy body; this grants neither an anatomical nor a complete-standing claim |
| Standing V2 deterministic evaluation | Seed `1001`: final `model_999.pt` terminates at tick `100`; peak-region `model_800.pt` terminates at tick `97`; neither truncates at `3,600` | Stop the five-seed matrix at the first failed gate and reject both obvious checkpoint selectors; do not start walking or promote authority |
| Stage 0 V1 physical-model audit | Exact descriptor `13f01daf…bd5`: 24 bodies and 24 sphere colliders; 23 revolute joints, all axis `+X`, all hard limits `±1.5 rad`; identical isotropic inertia tuples. Generated USD preserves all 23 joints as X-axis revolutes with `±85.9436693°` limits | The viewer exaggerates the bead-like appearance but does not invent it. This schema is a deterministic toy discriminator, not an anatomically credible humanoid for product learning |
| `model_800.pt` frame audit | Exact 132-frame capture: reset at frame `33`; root height progresses approximately `1.10, 1.02, 1.07, 1.03, 0.90, 0.83, 0.75, 0.63, 0.49, 0.38, 0.27 m` through frames `40..110`; legs cross and the pelvis collapses before the deterministic tick-`97` termination | Falsifies a single bad-frame or viewer-only explanation; failure is a reproducible whole-episode collapse on the declared body |
| Biomechanics standing environment | ADR-101; body `nextengine.body.humanoid-biomechanics-raja-1700.v2`; material V3 descriptor `6751853a…f027`; environment `nextengine.motor.env.humanoid-biomechanics-standing.v1`; manifest `1b60550d…73bb` | Native PhysX proves two distinct sole contacts and 32 reset/step ticks with complete actuator safety, contact classification, running termination and bounded reward |
| Biomechanics external generation and smoke | Generation `nextengine.training.generation.r8b-biomechanics-standing.v1`, manifest `1155e289…f95`; descriptor bytes `fb72047e…0647`; humanoid USD `5ea8a3b9…3834`; four-slot reset/reward smoke passes on RTX 3080 | The exact current-body inputs execute in Isaac; this is GPU preflight rather than correspondence evidence |
| Biomechanics standing optimizer run | `r8b-biomechanics-standing-seed42-v1` completes `1,024,000` samples and 250 metric records on clean commit `e72df2ce…`; run manifest `4e952700…ab7e`, metrics `8f7163d3…ab1b`, 11 checkpoint hashes close; final `model_249.pt` `59a89d33…ad81` | The isolated discriminator completes without non-finite metrics; early/final 20-iteration mean episode length rises `34.645 -> 322.325`, but training aggregates alone are not a standing claim |
| Biomechanics final-checkpoint GPU evaluation | Predeclared `model_249.pt`; seeds `1001..1005` each complete with length `3599`, return `6247.1797`, `terminated=0`, `truncated=1`, fall component `0`; evaluation manifest hashes `be850111…8f97` / `a8cda116…be28` / `40ea82c8…835` / `dca83d5f…51e3` / `46e99e3e…6a3` | Nominal deterministic Isaac standing repeatability passes. Identical per-seed results expose that standing reset has no seed-dependent perturbation; GPU contact-safety closure and CPU authority remain open |
| Canonical CPU evaluation of biomechanics `model_249.pt` | Manifest `62a43513…3076`; checkpoint terminates at tick `1` with `terminal.self-collision`; right forearm/pelvis impulse is about `0.297335 N·s` | Reject the checkpoint and all earlier contact-blind quality claims; preserve artifacts only as negative evidence |
| Contact-complete Isaac counterfactual | Exact 14-ground/91-self pair classifier makes the same `model_249.pt` terminate at step `2` with `self_collision=1`; a zero residual reaches a symmetric pelvis/forearm contact at step `3` around `0.26..0.30 N·s` because the neutral AABB clearance is only about `0.4 mm` | Confirms the omitted GPU terminal and a narrow CPU/GPU contact-boundary mismatch; do not weaken the safety threshold or change morphology before testing learnability |
| Contact-correct 25-iteration discriminator | Run manifest `d7e7bc0c…ca0b`, metrics `b0da57ea…071d`, final `model_24.pt` `9616d933…9bde`; mean episode length `3.19 -> 17.94`, self-collision share falls about `0.803 -> 0.177`; deterministic evaluation manifest `337e0d8a…2c33` reaches tick `68` then joint-safety terminates | The unchanged body/profile has usable learning signal. Run the full fresh 250-iteration discriminator; do not yet redesign the body or start walking |
| Contact-correct 250-iteration V2-body discriminator | Run manifest `629a6961…f141`, metrics `e08dde63…7b3f`, final `model_249.pt` `166c469e…775`; mean episode length improves `3.19 -> 143.61`, but final evaluation self-collides at GPU tick `139` and CPU tick `135` | The optimizer learns, but the `405 um` neutral pelvis/forearm gap makes the body identity inadmissible; preserve the run as negative evidence |
| ADR-102 V3 no-training controls | Exact neutral pelvis/forearm AABB clearance is `45,405 um`; zero residual reaches canonical CPU timeout at tick `3,600` without a safety event and Isaac tick `109` without self-contact before a physical forward fall | Immediate geometry/contact blocker is removed; begin one fresh hash-closed V3 standing run rather than requiring the hand-written fallback to solve standing |
| V3 standing optimizer | Generation manifest `e82416ec…ea5b`; run manifest `e185f9f9…dab6`; metrics `cca93b87…dd6e`; final `model_249.pt` `254f9d3d…f41d`; all 1,024,000 samples and 250 metric records close on clean commit `b6734160…fac4` | Final-20 mean length is `389.25`, peak mean `474.59`; training is healthy but quality is decided only by deterministic evaluation |
| V3 final-checkpoint Isaac evaluation | Manifest `4f8bb898…eae6`; seed `1001` reaches length `3,599`, return `6194.1616`, one truncation and zero declared safety/contact/fall terminals | Contact-complete nominal GPU standing passes |
| V3 final-checkpoint CPU evaluation | Manifest `2cea5beb…b645`; five episodes each reach tick `3,600` and `terminal.timeout`, two-sole occupancy is `1.0`, minimum root height `0.943284 m`, maximum tilt `11.5313 deg` and final backward lean `10.1897 deg` | Canonical nominal standing gate passes; this checkpoint may become an immutable parent only after the remaining R&D boundary is declared |
| [V3 paired-mirror investigation](../r8b-model-mirror-p1-investigation-2026-09-04.md) | `MODEL-MIRROR-P1 FAILED`; independent closed-loop joint/root-velocity RMSE is `0.05634 rad / 0.06655 m/s`; exact CPU and GPU action tapes lose the other plane at ticks `171 / 116` | ADR-103 permits one R&D-only walking discriminator; promotion remains blocked and thresholds remain unchanged |
| [ADR-103](../../architecture/adr/103-r8b-rd-only-walking-discriminator.md) | User-authorized one-run exception with distinct profile, standing-weight initialization and mandatory CPU evaluation | The one budget is consumed; the negative result grants no second budget or runtime authority |
| [ADR-104](../../architecture/adr/104-r8b-discriminating-walking-objective.md) | Supersedes only ADR-103's no-second-budget clause for one V2 reward/schedule discriminator | Run only after its exact no-training controls; no broader curriculum or runtime authority follows |
| [Forward start/stop R&D result](../r8b-forward-start-stop-rd-result-2026-09-04.md) | The completed 1,024,000-sample run is safe for five GPU episodes but travels only `0.084 m` against `3.944 m` commanded; CPU final travels `-0.025 m` on average and terminates all five episodes | Reject the walking claim. Freeze the lineage and repair the reward/evaluation discriminator without training before any new budget decision |
| [Walking training research](../r8b-walking-training-research-2026-09-04.md) | Primary NVIDIA/ETH sources and local counterfactuals identify permissive tracking/positive posture bonuses and a non-proving stop schedule before PPO | ADR-104 selects one V2 fixed `0.5 m/s` lesson with compact tracking, posture costs and unchanged PPO/body/controller/safety |
| Walking V2 optimizer and GPU evaluation | Run manifest `99a72819…e4f3`; final checkpoint `5099b8d3…038e`; GPU evaluation `47878e6f…5f3a` | All 250 metrics are finite and five GPU episodes survive, but mean travel is only `0.123/7.258 m`; reject walking |
| Walking V2 canonical CPU sweep | Final evaluation `68e9539a…5119` falls at tick `116`; checkpoint 25 evaluation `0f6e6d9c…be46` survives five horizons but moves `-0.176 m`; all other saved checkpoints fall | No V2 checkpoint satisfies even signed-forward progress; preserve the lineage as negative evidence |
| [ADR-105](../../architecture/adr/105-r8b-dense-tracking-walking-counterfactual.md) | V2 stationary `0.5 m/s` error has zero compact reward and no local gradient; exact dense Q16 goldens are monotonic | Admit one same-seed/same-budget V3 run changing only the tracking kernel |
| Walking V3 optimizer and evaluation | Run manifest `d4a95969…c23d`; final checkpoint `da6924cc…e60`; GPU evaluation `1fe4c2cc…b81`; CPU evaluation `2dbe861e…968` | All `1,024,000` samples close. GPU completes `5/5` horizons safely but achieves only `0.104/7.258 m`; CPU falls at tick `156` in `5/5`, moves `-0.822 m` and reaches about `-59.85°` pitch. Reject dense tracking as a sufficient gait-discovery fix and stop further optimizer work pending research |

## Decisions that still constrain the work

### D-001 — Use an independent V1 command lineage

- **Observation:** R141 stopped before solve because its accepted orientation
  source was not byte-reversible; its downstream authority was never earned.
- **Evidence:** The stopped TRAIN-4 task state and its exact R141 report.
- **Decision:** R8b begins from frozen V1 standing/command environment
  identities and new manifests. It does not repair, relabel or inherit the
  motion-reference lineage.
- **Rejected alternatives:** R142, rejected TRAIN-5 resume, substituting an
  R120 witness, or treating prior checkpoints as initialization.
- **Consequences:** Every successor preflight must continue to prove zero
  dependency on the stopped corpus and checkpoint roots.
- **Uncertainty:** Body learnability remains unmeasured because the first
  optimizer objective had incompatible reward units.
- **Reconsider when:** Only a separate roadmap decision may reopen the
  motion-reference problem; it cannot be folded into R8b.

### D-002 — Optimize for the first visible foundation skill

- **Observation:** The previous policy was asked to learn backward, strafe,
  turn and high-speed translation before it could remain upright.
- **Evidence:** ADR-065 records `768/768` held-out falls for the broad V1 run.
- **Decision:** First admit complete-episode standing. The next distinct run
  admits only the existing first curriculum stage: zero or forward
  `0..0.75 m/s`, without strafe, yaw or backward commands.
- **Rejected alternatives:** Full curriculum, terrain, pushes, recovery,
  imitation/reference tracking, damage or multi-skill learning in the first
  run.
- **Consequences:** A standing artifact may be immutable parent initialization
  for a separately manifested walking run, never a cross-profile resume.
- **Uncertainty:** Whether standing initialization materially improves the
  first walking checkpoint.
- **Reconsider when:** Both foundation gates pass with exact held-out evidence.

### D-003 — Keep CPU PhysX as evidence authority

- **Observation:** Isaac is an optional accelerated mirror; GPU throughput and
  visual output do not prove canonical runtime behavior.
- **Evidence:** SPEC-35 and ADR-058/064/067.
- **Decision:** CPU PhysX owns admission and final held-out trajectories. Isaac
  may accelerate only after exact descriptor/golden/correspondence preflight,
  and its result remains separately classified.
- **Rejected alternatives:** GPU-only acceptance or treating a video as a
  safety/replay oracle.
- **Consequences:** Optional capture is produced only after its source replay
  root is known; absence of capture does not change the numeric verdict.
- **Uncertainty:** Isaac execution is available, but CPU/Isaac trajectory
  correspondence for this body/profile family is not yet checked.
- **Reconsider when:** A future Accepted profile changes the canonical plane.

### D-004 — Defer Kimodo to a separate post-foundation lineage

- **Observation:** Kimodo generates short kinematic SOMA/G1 trajectories; it
  does not solve Next Engine's 23-DoF feedback controller, physical contacts or
  CPU PhysX admission. Current R8b deliberately consumes no reference corpus.
- **Evidence:** The dated Kimodo research report, current R8b scope, SPEC-34/35
  and the stopped TRAIN-4 state.
- **Decision:** Do not install, ingest or train from Kimodo in the active R8b
  package. Preserve `KIMODO-MOTION-P0` as a future out-of-process candidate
  generator after both foundation gates and a separate roadmap decision.
- **Rejected alternatives:** Adding Kimodo references to standing/forward R8b,
  direct G1/SOMA ingestion, runtime text-to-motion, or using Kimodo to resume or
  relabel R123–R141.
- **Consequences:** Kimodo remains excluded from both the retired standing V1
  run and its future bounded-unit successor. A future pilot needs new license
  snapshots, raw-output hashes, deterministic SOMA-to-BodySchema retarget and
  optimizer-free CPU admission.
- **Uncertainty:** Physical admission yield and PPO sample-efficiency benefit
  are unmeasured; model/text-encoder/output licensing still needs exact review.
- **Reconsider when:** Both standing and bounded forward/start-stop gates pass,
  or an explicit roadmap decision changes their order.

### D-005 — Retire standing V1 from optimizer use

- **Observation:** The complete seed-42 run reduced its final-20 mean return
  magnitude by about six times while mean episode length fell from `101.51` to
  `62.39` ticks; its best mean length was only `111.16/3,600`.
- **Evidence:** External run
  `runs/r8b-standing-seed42-v1`, manifest `2debfb23…`, metrics
  `6091cb6a…`, event projection `f3ff7054…`, plus `_standing_rewards` in
  `lab/next_lab/isaac_env.py` where effort/action remain raw microunits.
- **Decision:** Treat V1 standing PPO as a closed negative experiment. Do not
  evaluate, resume, initialize from or rerun any of its checkpoints.
- **Rejected alternatives:** More iterations, a new PPO seed, learning-rate
  tuning or selecting the least-bad intermediate checkpoint; none changes the
  dominant reward units.
- **Consequences:** The next optimizer input needs a new environment identity
  with bounded unit semantics and a newly admitted generation. The completed
  V1 run remains immutable external diagnostic evidence only.
- **Uncertainty:** Whether the unchanged V1 body learns full-episode standing
  after a correct bounded reward remains unmeasured.
- **Reconsider when:** A new profile passes CPU reward golden vectors, Isaac
  reset/reward parity and a small no-training reward-scale probe.

### D-006 — Change only the reward identity for the first V2 discriminator

- **Observation:** PPO target regression inherits reward scale, while the V1
  body, action, reset and simulator paths all execute. Changing several of
  those at once would make a second run uninterpretable.
- **Evidence:** V1 metrics, exact Rust/Torch reward goldens, ADR-100 and the V2
  production CPU no-training probe.
- **Decision:** Keep seed `42`, network, optimizer, `128 × 32 × 1,000` schedule
  and five held-out seeds unchanged. Change only environment/training profile
  identities and their reward/translator closure.
- **Rejected alternatives:** More V1 compute, simultaneous PPO tuning,
  BodySchema/reset edits, motion imitation or a larger budget.
- **Consequences:** The next run is a direct discriminator for H1. It may begin
  only after exact generation, GPU reset/reward and trainer preflight pass.
- **Outcome:** The exact V2 run completed and fixed numerical scale, but the
  first deterministic held-out episode failed at tick `100`; bounded reward
  alone is insufficient for complete standing under this profile and budget.
- **Uncertainty:** Whether the remaining gap is primarily learned action-noise
  dependence, checkpoint volatility or an objective exploit around low height.
- **Reconsider when:** Superseded by D-007; do not repeat V2 on the frozen
  Stage 0 V1 body.

### D-007 — Retire the Stage 0 V1 body from learned-humanoid optimization

- **Observation:** The failed policy was trained on a 24-sphere body whose 23
  nominal yaw/roll/pitch joints are all collinear X-axis revolutes with the same
  symmetric ROM. Knees can hyperextend and feet provide no plantar support area.
- **Evidence:** Exact Stage 0 descriptor and USD, `reference.rs`, the V1 compiler
  restrictions, and every frame of the seed-1001 `model_800.pt` capture.
- **Decision:** Preserve completed V1-body runs as immutable diagnostic evidence,
  but do not tune, resume or train this body again for a learned-humanoid claim.
  The next optimizer candidate must consume an admitted biomechanics successor
  with distinct joint axes, anatomical ROM and support geometry.
- **Rejected alternatives:** Blaming only the browser renderer, PPO/noise tuning
  on the same body, a larger budget, or treating serial same-axis joints as an
  acceptable approximation of compound human joints.
- **Consequences:** The bounded reward remains useful, but body/reset/descriptor
  identities must change together in a new generation. Previous checkpoints are
  not initialization for the successor.
- **Uncertainty:** Whether the existing biomechanics V2 neutral-pose failure is
  a schema geometry issue or an articulation-build/ground-contact issue.
- **Reconsider when:** Never for product learned-humanoid evidence. The V1 body
  may remain only as an explicitly labelled regression/toy fixture.

### D-008 — Preserve the first biomechanics checkpoint as superseded GPU R&D evidence

- **Observation:** The predeclared final checkpoint survives the full nominal
  Isaac horizon five times, but every evaluation is identical because standing
  has an exact zero command and deterministic neutral reset.
- **Evidence:** The closed run/checkpoint hashes and five immutable evaluation
  manifests listed above; all report timeout rather than termination.
- **Decision:** Preserve `model_249.pt` and its original evaluations as immutable
  R&D evidence. D-009 supersedes its candidate status after the missing GPU
  contact-safety classification and CPU evaluation rejected it.
- **Rejected alternatives:** Calling five seed labels perturbation robustness,
  treating GPU timeout as CPU authority, or immediately spending a larger PPO
  budget.
- **Consequences:** The next work is a correspondence/admission implementation,
  not another optimizer run. Optional video may illustrate the exact candidate
  but cannot change its evidence status.
- **Uncertainty:** Survival under perturbed resets, hard impacts, self-contact
  and the canonical CPU policy consumer is still unmeasured.
- **Reconsider when:** Never as a candidate; only a fresh contact-correct run may
  earn standing admission.

### D-009 — Reject contact-blind standing and retrain before walking

- **Observation:** `model_249.pt` passes only while Isaac omits the declared
  hard-impact/self-collision terminal. The canonical CPU path and repaired GPU
  path both terminate it immediately on self-collision.
- **Evidence:** CPU manifest `62a43513…3076`, repaired-GPU terminal result, exact
  pelvis/forearm pair facts and the bounded 25-iteration discriminator listed
  above.
- **Decision:** Preserve the old run unchanged as negative evidence. Train one
  fresh standing policy from random weights with pair-complete GPU contact
  termination, then require CPU evaluation before any forward profile or run.
- **Rejected alternatives:** Resuming or initializing from the invalid
  checkpoint, ignoring positive-separation contact impulses, widening safety
  thresholds, or changing the body before the cheap learnability discriminator.
- **Consequences:** The standing profile semantics stay unchanged; this is an
  implementation-conformance repair. The new run gets a distinct run ID and
  clean source commit. Walking remains gated.
- **Uncertainty:** Whether 250 iterations are sufficient for deterministic
  full-horizon contact-safe standing and whether its CPU trajectory remains
  within correspondence tolerance.
- **Reconsider when:** The fresh final checkpoint either passes the GPU/CPU
  gates or fails with a stable clustered terminal that warrants a new bounded
  morphology/controller experiment.

### D-010 — Replace the contact-boundary V2 body with the V3 successor

- **Observation:** The full contact-correct run learns longer survival but its
  final GPU/CPU episodes still self-collide; the V2 neutral pelvis/forearm gap
  is only `405 um` and fails a zero-residual GPU control at tick `3`.
- **Decision:** ADR-102 admits the separately identified V3 body and standing
  V2 environment with `45,405 um` neutral clearance and source-conserving
  serial-carrier projection. Old checkpoints cannot initialize it.
- **Consequence:** A later Isaac fall at tick `109` is valid training input;
  only immediate neutral self-contact blocks the optimizer. Walking remains
  gated on the final V3 checkpoint's complete CPU and mirror evidence.

### D-011 — Permit one R&D-only walking discriminator before mirror repair

- **Observation:** V3 standing passes its nominal GPU/CPU gates, while paired
  correspondence fails and repairing the mirror does not answer whether the
  body/controller has usable forward-learning signal.
- **Evidence:** Final standing manifests, the paired-mirror investigation and
  the user's explicit authorization on 2026-09-04.
- **Decision:** ADR-103 permits one hash-closed foundation-stage walking run
  initialized from standing model weights only, followed by GPU and CPU eval.
- **Rejected alternatives:** Calling the run a resume, weakening P1, granting
  GPU-only authority, starting a broader curriculum or silently adding budget.
- **Consequence:** A CPU failure rejects the experiment. A CPU pass remains
  R&D-only until correspondence and the full promotion matrix pass.
- **Reconsider when:** After the final CPU evaluation; any second optimizer
  budget requires a new explicit decision.

### D-012 — Reject the first walking objective before spending more compute

- **Observation:** The completed policy survives long GPU rollouts but remains
  nearly stationary; the safest CPU checkpoint also moves slightly backward.
- **Evidence:** The dated forward start/stop result and its exact generation,
  run, checkpoint and GPU/CPU evaluation hashes.
- **Decision:** Classify the first broken boundary as `Environment`, reinforced
  by `Evaluation`. Preserve the completed run as negative evidence and spend no
  second optimizer budget under the current reward or command schedule.
- **Rejected alternatives:** Calling survival walking, selecting checkpoint 75
  or 225 as a walking result, tuning PPO, or weakening CPU/mirror requirements.
- **Consequences:** The next work is a no-training discriminator: guarantee at
  least `3 m` commanded forward travel and a final 180-tick stop, then prove the
  standing parent and zero-action policy cannot pass the motion criterion.
- **Uncertainty:** Whether the same body/controller learns once motion reward is
  made discriminating; correspondence remains independently unresolved.
- **Reconsider when:** The no-training controls pass and a user explicitly
  authorizes a separately identified second optimizer budget.

### D-013 — Run one discriminating V2 walking lesson

- **Observation:** V1 pays 64% planar tracking to a stationary policy at the
  representative `0.5 m/s` command, while its schedule does not guarantee the
  final stop. Finite optimizer metrics and retained exploration do not support
  PPO tuning as the first repair.
- **Evidence:** The dated walking research, NVIDIA Isaac Lab reward/config
  sources, Rudin et al., and exact V1 GPU/CPU results.
- **Decision:** ADR-104 admits exactly one seed-44 V2 run after goldens and
  canonical CPU no-training controls. V2 changes only the task lesson/reward
  identity; standing weights initialize the model only and PPO stays fixed.
- **Rejected alternatives:** More V1 compute, PPO/noise tuning, weakening
  safety, or adding broad/random commands before a first visible gait.
- **Consequences:** V1 remains immutable. V2 must achieve five safe CPU
  timeouts, at least `3 m` signed travel per episode and final-stop forward
  speed MAE `<=0.15 m/s`; mirror failure still forbids runtime promotion.
- **Outcome:** V2 fails. Final GPU travel is `0.123/7.258 m`; no saved CPU
  checkpoint walks. The compact kernel has no gradient at the initial
  stationary `0.5 m/s` error.
- **Reconsider when:** Superseded by D-014; never retry V2.

### D-014 — Test dense tracking without changing PPO or the lesson

- **Observation:** V2 is finite and survives on GPU, but its compact planar
  reward is exactly zero at the inherited policy's representative error.
- **Evidence:** V2 reward decomposition, GPU/CPU evaluation sweep, NVIDIA's
  dense exponential locomotion rewards and exact Rust/Torch V3 goldens.
- **Decision:** ADR-105 admits one V3 seed-44 run with the same body, command,
  coefficients, PPO, initialization and budget. Only planar/yaw tracking uses
  `square(1 / (1 + (error / width)^2))`.
- **Rejected alternatives:** More V2 compute, PPO tuning, weaker terminals, or
  simultaneous curriculum/body/action changes.
- **Consequences:** V3 remains `R&D_ONLY`; the unchanged CPU distance/stop gate
  and failed mirror gate still control any claim.
- **Outcome:** V3 completes all `1,024,000` samples and five safe GPU horizons,
  but travels only `0.104/7.258 m`. Canonical CPU terminates all five episodes
  at tick `156` with a backward fall and `-0.822 m` displacement. Dense tracking
  is therefore not sufficient for gait discovery under the fixed lesson.
- **Reconsider when:** Never retry V3 unchanged. A new experiment requires the
  bounded post-failure research cycle and a distinct falsifiable hypothesis.

### D-015 — Audit walking action reachability before more optimization

- **Observation:** The final GPU policy remains in two-sole contact for all
  `600/600` recorded ticks. Walking residuals are applied over a standing
  reference whose ankle target includes absolute root-forward displacement.
- **Evidence:** The exact GPU trajectory has SHA-256 `ee9bc838…224f`; moving
  velocity is `0.00620 m/s` and there are zero support-state switches. At the
  command's approximate `7 m` endpoint the standing reference plus the
  `+/-0.15 rad` ankle residual leaves only `[0.42, 0.436] rad` after soft ROM.
  Canonical CPU fixed/random and standing-policy-plus-periodic probes produce
  no single-sole state. The final V3 CPU policy is already pitching backward
  before its first non-zero command.
- **Decision:** Treat the origin-anchored standing action base as the earliest
  walking blocker. Before any PPO run, test one walking-only,
  translation-invariant reference with deterministic alternating action tapes
  on CPU and Isaac. Do not change reward or PPO in that discriminator.
- **Rejected alternatives:** Another V3-sized run, PPO tuning, declaring the
  anatomy incapable, adding gait reward simultaneously with the action-base
  change, or accepting GPU survival as CPU correspondence.
- **Consequences:** A later gait-reward experiment is conditional on proving
  left-only and right-only support, a complete alternation, positive forward
  displacement and zero safety terminal in the action-basis audit. Full
  training also remains blocked by failed CPU/Isaac correspondence.
- **Reconsider when:** The translation-invariant action-basis audit either
  passes its paired reachability criteria or isolates residual scale/body
  geometry as the next one-variable failure.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: biomechanics V3 is learnable with pair-complete contacts | Final V3 policy completes one GPU and five CPU 3,600-tick episodes with zero safety terminal | Perturbation robustness is unmeasured | Retain the final checkpoint as the sole nominal standing parent candidate |
| H2: ADR-100's bounded standing objective ports without another reward redesign | Final V3 policy completes the GPU and five CPU nominal gates | Perturbation robustness and paired trajectory correspondence remain open | Freeze the standing checkpoint as the walking parent; do not promote it to runtime authority |
| H3: Isaac can shorten successor iteration without changing candidate admissibility | Descriptor/material/USD closure is exact; the final policy completes both GPU and CPU nominal horizons | `MODEL-MIRROR-P1` fails on joint/root velocity and cannot complete one common action tape | Keep GPU output R&D-only; repair/replace the mirror or explicitly change R&D sequencing |
| H4: admitted standing initialization plus a dense task signal can acquire forward motion | V1/V2 retain survival and V2 exposes a precise zero-gradient boundary | V3 also remains stationary on GPU and falls backward on CPU | Refuted for the fixed V3 lesson; do not add compute unchanged |
| H5: velocity tracking lacks the contact-phase incentive needed to discover a biped gait | NVIDIA H1 and Unitree H1 include duration-aware or phase-conditioned foot signals; the exact V3 GPU trace remains double-support for `600/600` ticks | A gait-signal counterfactual must wait until action reachability is proven | After the action-basis audit, replace only the binary support predicate in a bounded successor |
| H6: the procedural standing reference is a valid walking action base | It preserves an admitted standing policy | Its absolute root-forward term changes ankle action meaning with travelled distance and nearly removes bidirectional authority near `7 m`; no bounded action probe reaches single support | Test a walking-only translation-invariant reference before changing reward or PPO |
| H7: biomechanics V3 is intrinsically incapable of stepping | It has not produced a learned or hand-authored single-support trace under the current residual profile | Separate anatomical joints, real ROM and non-spherical geometry are present; current probes confound body capability with the standing action base | Require paired CPU/Isaac alternating-support reachability after removing only the position anchor |

## Required context

Read these sources in precedence order before acting:

1. [Agent routing](../../architecture/agent-routing.md), especially the motor,
   humanoid-training, learned-Motor and roadmap rows.
2. [Current roadmap](../../roadmap.md), [SPEC-35](../../architecture/35-deterministic-humanoid-training-substrate.md),
   [SPEC-34](../../architecture/34-model-training-environments-trajectories-and-consolidation-lifecycle.md),
   [SPEC-14](../../architecture/14-physical-archetypes-motor-skills-and-policy-lifecycle.md)
   and [SPEC-27](../../architecture/27-motor-observation-action-and-deterministic-inference.md).
3. [ADR-058](../../architecture/adr/058-physx-only-deterministic-humanoid-training-substrate.md),
   [ADR-064](../../architecture/adr/064-canonical-flat-command-locomotion-environment.md),
   [ADR-065](../../architecture/adr/065-curriculum-flat-command-locomotion-profile.md)
   and [ADR-067](../../architecture/adr/067-stage0-profile-identity-and-curriculum-hash-closure.md).
4. [Stopped TRAIN-4 state](humanoid-motor-training.md) for immutable negative
   constraints only; its artifacts are not R8b inputs.
5. The `nextengine-training-runner` current contract before preparing or
   starting any run, and `nextengine-training-diagnostics` before changing an
   experiment after a failure.

## Next action

1. Preserve the completed standing and walking lineages, checkpoints and
   evaluations as immutable external evidence.
2. Treat V1, compact V2 and dense V3 as three failed remediation cycles; do not
   launch another optimizer run or sweep PPO settings.
3. Implement no optimizer change. First pre-register and run the walking-only
   translation-invariant action-basis audit from D-015 on CPU and Isaac.
4. Change residual range only if reachability still fails; change the binary
   support reward only after reachability passes. Each remains a separate
   one-variable discriminator with standing and safety controls.
5. Only after paired reachability and acceptable correspondence may a tiny
   overfit run request compute; a full training budget remains blocked.

## Do not retry

- Broad `humanoid-flat-command.v1` pure PPO — ADR-065 records `768/768` held-out
  falls; use standing then only the frozen first curriculum stage.
- R123/R127/R129/R130/R136/R141, R142 or rejected TRAIN-5 checkpoints — the
  stopped lineage has no downstream authority and R8b is deliberately
  independent.
- Trainer-side command/reward overrides — they create a second environment
  authority and invalidate the exact manifest.
- Standing V1 optimizer/evaluation/resume — the complete seed-42 run shows its
  raw effort/action units dominate the objective, and its metrics are not
  manifest-bound.
- Any further optimizer/evaluation/resume on the frozen Stage 0 V1 body,
  including bounded-reward V2 — its all-sphere, all-X articulation is not a
  credible learned-humanoid foundation.
- Any optimizer/evaluation/resume on biomechanics body V2 / standing V1 — its
  neutral contact boundary invalidates both completed contact-correct policies.
- Walking V1, compact-tracking V2 or dense-tracking V3 — all complete runs lack
  canonical signed forward progress; V3 also falls at tick `156` on CPU.
- Kimodo or another generated-motion source inside current R8b — it would
  bypass the declared command-only lineage and cannot rehabilitate TRAIN-4.
- GPU-only quality or video-only acceptance — CPU PhysX trajectories and
  declared safety facts remain the oracle.

## Handoff

- **Workspace state:** Standing is immutable positive evidence; walking V1/V2/V3
  are immutable negative evidence. No further walking optimizer run is
  authorized.
- **Checks:** V3 closes 250 records and `1,024,000` samples. Final GPU evaluation
  completes `5/5` horizons but travels only `0.104/7.258 m`; canonical CPU
  evaluation falls at tick `156` in `5/5` and travels `-0.822 m`.
- **Remaining risk:** The origin-anchored standing reference is incompatible
  with long translation, the current residual basis has no demonstrated
  single-support reachability, the binary support reward has no gait duration
  or alternation credit, and `MODEL-MIRROR-P1` remains failed.
- **Deferred:** Kimodo remains outside this foundation lineage.
- **Promotion needed:** None for the priority change. Runtime learned-policy
  promotion still requires its consumer-backed schemas, parity, multi-seed
  quality, replay and fallback gates.
