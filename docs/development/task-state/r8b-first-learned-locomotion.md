# R8b first learned locomotion — current task state

| Field | Value |
| --- | --- |
| Status | `SELECTED / ACTIVE_R&D / FIRST_RUN_COMPLETE / V1_REWARD_SCALE_BLOCKED / NO_AUTHORITY` |
| Updated | 2026-09-04 |
| Task key | `r8b-first-learned-locomotion` |
| Scope | Produce the first visible learned standing and bounded forward start/stop checkpoints on the frozen Stage 0 V1 humanoid |
| Definition of done | Five fixed held-out CPU PhysX episodes pass the complete 3,600-tick standing gate, then five pass the bounded forward/start/stop gate, with exact manifests, zero declared safety events and replay-bound visual evidence when capture is available |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact run artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R8b is the selected post-v1 WIP. Start with learned
  standing, but retire its dimensionally broken V1 reward before another run;
  forward start/stop, damage and broader skills remain deferred.
- **Why:** R8a is complete and the foundation-first order remains correct, but
  the first exact run proves the accepted frozen V1 standing reward is not a
  usable optimizer objective. More PPO compute cannot repair raw-unit scale.
- **Next action:** Do not rerun standing V1. Define one engine-owned standing
  V2 reward profile with bounded, commensurate units, close future generic-run
  metrics in their manifests, then preflight a new generation before asking
  for another compute budget.
- **Current blocker:** The frozen standing V1 reward sums raw
  micronewton-metre effort and microradian action deltas beside unit-scale
  posture terms, so PPO learns to reduce actuation and fall sooner. The first
  run also exposed that the generic manifest closes checkpoints but not
  `metrics.jsonl`; the source fix applies only to future runs. In addition,
  `MODEL-MIRROR-P1` remains `NOT_RUN` because the CPU recorder/correspondence
  reader NPZ layouts differ and no V1 GPU recorder exists.
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
| R8b evaluation/correspondence | `NOT_RUN` | A five-seed evaluation cannot rescue a checkpoint whose training survival is about one second; CPU/Isaac and runtime claims remain blocked |

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

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: the frozen V1 body can learn complete-episode standing with the smallest MLP profile | Procedural standing and the body/actuation path execute | The only full run optimized incompatible raw reward scales and therefore did not test body learnability | A new bounded-unit standing profile and smallest pre-training reward-scale probe |
| H2: immutable standing initialization improves bounded forward start/stop | Foundation-first curriculum removes most simultaneous objectives | No R8b walking comparison exists | Compare declared standing-parent initialization with the smallest clean control under one fixed budget |
| H3: Isaac can shorten iteration without changing candidate admissibility | Descriptor/mirror infrastructure exists | Current correspondence readiness is `NOT_RUN` | Exact mirror preflight followed by CPU final evaluation |

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

1. Keep `r8b-standing-seed42-v1` immutable and excluded from checkpoint
   selection. Its missing metrics binding cannot be repaired in place.
2. Add one engine-owned standing V2 environment identity whose reward
   components and coefficients are bounded and hash-closed on CPU and Isaac;
   validate scale/order with golden vectors and a no-training probe.
3. Admit that exact environment/profile under a new external generation and
   request explicit approval for the smallest discriminating optimizer run.
   Only a promising result proceeds to the five frozen evaluation seeds and
   later `MODEL-MIRROR-P1` work.

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

- **Workspace state:** The first external optimizer run completed, but standing
  V1 is retired from further optimizer use. A source change now closes metrics
  for future generic runs and makes checkpoint selection reject incomplete
  metrics closure; it does not alter the immutable completed manifest.
- **Checks:** Run process exit, sample count, finite metrics and all 21 declared
  checkpoint hashes pass. The generic diagnostic tool rejects this manifest
  family and confirms the missing metrics binding; held-out evaluation and
  correspondence are intentionally not run.
- **Remaining risk:** Learned standing feasibility remains unknown because the
  first objective was dimensionally broken. A standing V2 identity and full
  CPU/Isaac correspondence are still required before any authority claim.
  Kimodo retarget yield, physical admissibility, learning benefit and exact
  license closure remain unmeasured and deferred.
- **Promotion needed:** None for the priority change. Runtime learned-policy
  promotion still requires its consumer-backed schemas, parity, multi-seed
  quality, replay and fallback gates.
