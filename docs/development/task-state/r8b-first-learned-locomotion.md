# R8b first learned locomotion — current task state

| Field | Value |
| --- | --- |
| Status | `SELECTED / ACTIVE_R&D / V2_RUN_COMPLETE / REWARD_SCALE_FIXED / STANDING_GATE_FAIL / NO_AUTHORITY` |
| Updated | 2026-09-04 |
| Task key | `r8b-first-learned-locomotion` |
| Scope | Produce the first visible learned standing and bounded forward start/stop checkpoints on the frozen Stage 0 V1 humanoid |
| Definition of done | Five fixed held-out CPU PhysX episodes pass the complete 3,600-tick standing gate, then five pass the bounded forward/start/stop gate, with exact manifests, zero declared safety events and replay-bound visual evidence when capture is available |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact run artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** The bounded-reward V2 discriminator completed all
  `4,096,000` transitions. It fixes the V1 numerical failure and learns much
  longer stochastic rollouts, but it does not produce an admissible standing
  policy: deterministic actor-mean evaluation falls at tick `100` for
  `model_999.pt` and tick `97` for the peak-region `model_800.pt`.
- **Why:** Final value loss is `6.876` instead of V1's approximately `5.33e18`,
  all metrics are finite, and final-20 reported mean episode length is
  `1,721.33` versus V1's `62.39`. The best reported training mean reaches
  `2,623.84/3,600` at iteration `818`, but the held-out deterministic result
  falsifies complete standing under this exact budget/profile.
- **Next action:** Do not spend another unchanged V2 run. First discriminate
  whether the remaining failure is actor-mean versus stochastic-policy quality,
  checkpoint-selection volatility, or a low-height reward exploit; then change
  one identified cause under a new V3 identity.
- **Current blocker:** The required five-seed `3,600`-tick standing gate fails on
  the first deterministic seed, so the remaining seeds and walking stage are
  not run. `MODEL-MIRROR-P1` also remains `NOT_RUN`; this blocks authority even
  if later policy quality passes.
- **Do not retry:** Never run/evaluate/resume standing V1, the broad V1 PPO
  checkpoints or any R123–R141/TRAIN-5 artifact; they are either causally
  invalid for this question or have incompatible/rejected authority.
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
| [Roadmap R8](../../roadmap.md) | `R8B SELECTED / FIRST_RUN_COMPLETE / V1_REWARD_SCALE_BLOCKED` | Keeps the visibility-first lineage active but prohibits another V1 run or runtime promotion |
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
| Standing V2 learning outcome | Early-20 reported mean length `99.72`; final-20 `1,721.33`; peak reported mean `2,623.84/3,600` at iteration `818`; final value loss `6.876`, maximum `260.783` | The body/profile can acquire a partial standing behavior; the exact run remains volatile and below the complete-episode gate |
| Standing V2 deterministic evaluation | Seed `1001`: final `model_999.pt` terminates at tick `100`; peak-region `model_800.pt` terminates at tick `97`; neither truncates at `3,600` | Stop the five-seed matrix at the first failed gate and reject both obvious checkpoint selectors; do not start walking or promote authority |
| R8b correspondence | `MODEL-MIRROR-P1 NOT_RUN` | GPU reset/reward smoke is not paired CPU/Isaac trajectory evidence; CPU/Isaac and runtime claims remain blocked |

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
- **Reconsider when:** A bounded diagnostic distinguishes those explanations;
  do not repeat V2 unchanged.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: the frozen V1 body can learn complete-episode standing with the smallest MLP profile | V2 training mean survival rises from about `100` to a peak `2,623.84` ticks | Final and peak-region deterministic policies fall at ticks `100` and `97`; the exact-budget complete-standing claim is falsified | Compare stochastic-policy and actor-mean rollouts for closed checkpoints before changing PPO or reward |
| H2: immutable standing initialization improves bounded forward start/stop | Foundation-first curriculum removes most simultaneous objectives | No R8b walking comparison exists | Compare declared standing-parent initialization with the smallest clean control under one fixed budget |
| H3: Isaac can shorten iteration without changing candidate admissibility | Descriptor/mirror infrastructure exists | Current correspondence readiness is `NOT_RUN` | Exact mirror preflight followed by CPU final evaluation |
| H4: V2 training reward permits a low-height survival strategy that does not transfer to deterministic standing | Long training episodes can report root-height tracking near zero while remaining above the `0.25 m` fall threshold | One failed deterministic evaluation still has mean height tracking `0.751`; it does not isolate this cause | Inspect closed-checkpoint rollout height/termination distributions and compare them with action-noise mode |

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

1. Keep both completed V1 and V2 external runs immutable. V1 is dimensionally
   invalid for optimization; V2 is the exact negative/partial-learning control.
2. Do not run seeds `1002..1005`, walking, correspondence or another optimizer
   budget until one bounded diagnostic distinguishes stochastic action-noise
   dependence, checkpoint volatility and low-height reward exploitation.
3. If one cause is isolated, freeze the smallest one-variable V3 change and
   its rollback/non-regression check before authorizing more GPU compute.

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
- Kimodo or another generated-motion source inside current R8b — it would
  bypass the declared command-only lineage and cannot rehabilitate TRAIN-4.
- GPU-only quality or video-only acceptance — CPU PhysX trajectories and
  declared safety facts remain the oracle.

## Handoff

- **Workspace state:** Commit `3435b164…` implements ADR-100, bounded standing
  V2 and its hash-closed profile. Its independent external generation and
  optimizer run are complete and immutable; no checkpoint is selected.
- **Checks:** Focused Python/Rust reward checks, workspace `host-check` and
  `persistence-replay` pass. Run sample count, 1,000 finite metrics records and
  all 21 declared checkpoint hashes pass. GPU reset/reward smoke passes.
- **Remaining risk:** Numerical reward scale is fixed, but deterministic policy
  quality fails the first held-out seed. CPU/Isaac correspondence, five-seed
  quality, walking and runtime authority remain blocked. Kimodo retarget yield,
  physical admissibility, learning benefit and exact license closure remain
  unmeasured and deferred.
- **Promotion needed:** None for the priority change. Runtime learned-policy
  promotion still requires its consumer-backed schemas, parity, multi-seed
  quality, replay and fallback gates.
