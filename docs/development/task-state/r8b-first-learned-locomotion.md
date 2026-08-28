# R8b first learned locomotion — current task state

| Field | Value |
| --- | --- |
| Status | `SELECTED / ACTIVE_R&D / PREFLIGHT_NEXT / NO_RUN_STARTED` |
| Updated | 2026-08-28 |
| Task key | `r8b-first-learned-locomotion` |
| Scope | Produce the first visible learned standing and bounded forward start/stop checkpoints on the frozen Stage 0 V1 humanoid |
| Definition of done | Five fixed held-out CPU PhysX episodes pass the complete 3,600-tick standing gate, then five pass the bounded forward/start/stop gate, with exact manifests, zero declared safety events and replay-bound visual evidence when capture is available |
| Authority | Working context only; Accepted SPEC/ADR, tracked profiles/manifests, exact run artifacts and `docs/roadmap.md` outrank this file |

## Resume in 60 seconds

- **Current conclusion:** R8b is the selected post-v1 WIP. Start with learned
  standing, then only forward start/stop; damage expansion and broader motor
  skills are deferred.
- **Why:** R8a is complete, while the accepted frozen V1 standing and
  curriculum environments already provide the shortest path to a visible
  learned result. The prior broad flat-command PPO attempt fell in every
  held-out episode, so breadth must be earned after foundation stability.
- **Next action:** Use `nextengine-training-runner` to perform a no-training
  preflight of exact V1 identities, external store, active host resources and
  CPU/Isaac capability; publish the exact permitted invocation or stop.
- **Current blocker:** None for read-only preflight. Any training run remains
  blocked until the preflight closes its exact manifest and storage lineage.
- **Do not retry:** Never resume the broad V1 PPO checkpoints or any
  R123–R141/TRAIN-5 artifact; they have incompatible or rejected authority.
- **Reconsider when:** Broaden beyond standing and forward start/stop only
  after both declared checkpoints pass. Reopen motion-reference work only by a
  separate roadmap decision satisfying its own byte-reversible lineage gate.

## Current evidence

| Evidence | Result | Consequence |
| --- | --- | --- |
| [Roadmap R8](../../roadmap.md) | `R8B SELECTED / PREFLIGHT_NEXT` | Authorizes one separate visibility-first lineage; does not authorize R142 or runtime promotion |
| [ADR-064](../../architecture/adr/064-canonical-flat-command-locomotion-environment.md) | `Accepted`; frozen standing and flat-command V1 environments | Reuse exact engine-owned BodySchema, observation, action, safety and CPU PhysX boundaries |
| [ADR-065](../../architecture/adr/065-curriculum-flat-command-locomotion-profile.md) | Prior broad V1 PPO held-out survival about 135–137 ticks; `768/768` falls; bounded V2 first stage exists | Do not repeat broad commands; standing precedes only `0..0.75 m/s` forward start/stop |
| [Stopped TRAIN-4 state](humanoid-motor-training.md) | `R141_INVALID / STOP_NO_RETRY` | R8b consumes no motion corpus, reference tracker, R123–R141 cache/witness or rejected checkpoint |
| R8b run/evaluation artifacts | `NOT_RUN` | No learned quality, visual, Stage 0 or runtime claim exists yet |

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
- **Consequences:** Preflight must prove zero dependency on the stopped corpus
  and checkpoint roots before any optimizer execution.
- **Uncertainty:** Learned standing quality on the current exact host remains
  unmeasured.
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
- **Uncertainty:** Current Isaac installation and mirror readiness are not yet
  checked.
- **Reconsider when:** A future Accepted profile changes the canonical plane.

## Open hypotheses

| Hypothesis | Evidence for | Evidence against | Next discriminator |
| --- | --- | --- | --- |
| H1: the frozen V1 body can learn complete-episode standing with the smallest MLP profile | Procedural standing and canonical standing environment exist | No conforming learned standing run has passed | One bounded multi-seed standing run after preflight |
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

1. Run a read-only/no-training preflight of the frozen V1 standing and first
   curriculum identities, external training store and active host resources;
   freeze the five command-only walking evaluation schedules before observing
   any model output.
2. Record the exact new run/generation manifest and verify that no R123–R141 or
   rejected TRAIN-5 root is reachable.
3. Start nothing if identity, storage, CPU PhysX or required mirror facts are
   ambiguous; preserve the procedural route and report the first blocker.

## Do not retry

- Broad `humanoid-flat-command.v1` pure PPO — ADR-065 records `768/768` held-out
  falls; use standing then only the frozen first curriculum stage.
- R123/R127/R129/R130/R136/R141, R142 or rejected TRAIN-5 checkpoints — the
  stopped lineage has no downstream authority and R8b is deliberately
  independent.
- Trainer-side command/reward overrides — they create a second environment
  authority and invalidate the exact manifest.
- GPU-only quality or video-only acceptance — CPU PhysX trajectories and
  declared safety facts remain the oracle.

## Handoff

- **Workspace state:** Roadmap selection only; no R8b run, checkpoint, dataset
  or generated artifact exists yet.
- **Checks:** Documentation cheap path is required for this selection change;
  executable motor checks begin only with the preflight/run package.
- **Remaining risk:** Learned standing feasibility, time-to-result, Isaac
  readiness and walking transfer benefit are unknown.
- **Promotion needed:** None for the priority change. Runtime learned-policy
  promotion still requires its consumer-backed schemas, parity, multi-seed
  quality, replay and fallback gates.
