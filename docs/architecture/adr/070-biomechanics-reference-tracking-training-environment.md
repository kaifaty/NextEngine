# ADR-070: Biomechanics reference-tracking training environment

| Field | Value |
|---|---|
| ID | ADR-070 |
| Status | Accepted |
| Version | 1.1 |
| Decision date | 2026-08-12 |
| Last verified | 2026-08-13 |
| Normative dependencies | [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-28](../28-skeletal-animation-retargeting-and-ik.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [SPEC-35](../35-deterministic-humanoid-training-substrate.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-069](069-biomechanics-body-schema-v2-and-solver-projection.md) |
| Supersedes | Adds one current training-only reference-tracking consumer for the exact ADR-069 biomechanics generation. It does not change Stage 0 V1/V2 environment identities or promote a learned runtime route. |
| Superseded by | none |

## Context

TRAIN-4 admitted a private, hash-closed motion corpus. TRAIN-5 now has a
concrete consumer for reference phase, horizon, reward and reset semantics.
Leaving those facts in Python would make the trainer a second environment
authority; forcing them into `PhysicalActionChunk` would prematurely publish a
runtime contract that TRAIN-5 does not consume.

The current `MotorTrainingEnvironmentManifestV2` closes command environments
but has no field for a task-input/corpus provenance root. Its reset, step,
trajectory and checkpoint records are otherwise sufficient because an episode
seed plus an immutable reference profile determines clip, phase and every
reference root.

## Decision

### Current profile and contract boundary

Add the training-only profile
`nextengine.motor.env.humanoid-reference-tracker.v1` for the exact
`nextengine.body.humanoid-biomechanics-raja-1700.v2@2` body. Its frozen
repository profile is
`lab/profiles/humanoid-reference-tracker.v1.json`.

Add two current-only alpha contracts:

- `MotorReferenceTrackingProfileV1` contains bounded reference layout, horizon,
  reset, reward, terminal and selection facts plus the exact corpus hashes;
- `MotorTrainingEnvironmentManifestV3` retains the V2 closure and adds
  `task_input_profile_hash` and `input_provenance_root`.

`MotorResetRecordV2`, `MotorStepRecordV2`, `MotorTrajectoryManifestV2` and
`MotorEnvironmentCheckpointEnvelopeV1` remain usable without reinterpretation:
their environment-manifest hash selects V3, the episode seed deterministically
selects the reference, and observation roots already bind the exact reference
features. The three command fields are canonical zero for this no-command
profile. A reference identity is never inferred from them.

No contract contains a filesystem path, NPZ name, framework tensor, source
skeleton or license text. The manifest binds the admitted corpus manifest and
profile hashes; the training host resolves bytes only from the configured
external store and rejects any hash mismatch before scene creation.

### Episode selection and reset

Only the corpus `locomotion` partition is eligible in TRAIN-5. Recovery clips
remain reserved for TRAIN-7. Split is an explicit run mode; train never selects
validation or held-out records.

Clip selection is uniform over eligible clip IDs after canonical ID sorting.
Phase selection is uniform over reference-frame ordinals from an independent
named seed stream. Cursor advances exactly once per 60 Hz motor tick. Future
horizon offsets clamp to the last frame; reaching that last frame terminates
with `terminal.reference-complete`. This is a successful domain termination,
not a time-limit truncation.

Initial reset weights are exact-reference `7000`, small-perturbation `2000`,
neutral-entry `1000` and near-failure `0` basis points. Neutral entry is valid
only for idle/start clips. Near-failure remains disabled until the nominal
tracker passes. Reset constructs a fresh PhysX scene at the selected state;
there is no post-create root teleport or mid-episode pose write.

### Observation and action

The actor observation is one canonical 435-channel integer layout:

- 86 committed dynamic channels: root rotation, root-local linear/angular
  velocity, 23 joint positions, 23 joint velocities, 23 previous applied
  targets and seven classified contact flags;
- one current phase channel;
- four 87-channel reference samples at offsets `0, 4, 8, 16` motor ticks,
  containing root/CoM, joint pose/velocity, six effector trajectories and seven
  contacts in declared root-local/relative frames.

The first critic reads the same layout; there is no privileged input in V1.
Actor output is exactly 23 bounded normalized residuals in signed Q1.30.
`BiomechanicsSafetyController` combines them with the current reference,
enforces target slew/ROM/effort/rate/power/work, and only then drives fixed PD.
The learned path never writes pose, root or torque directly.

### Reward, termination and optimizer boundary

The ordered Q16 reward components and coefficients are profile-owned before a
run: root orientation/height/linear/angular velocity, joint pose/velocity,
CoM, effectors, contacts, sole slip, effort, action rate and terminal failure.
Every value is bounded to `[0,1]`; hard safety is evaluated independently and
cannot be compensated by reward.

The frozen V1 profile remains immutable. A failed TRAIN-5 optimization run may
introduce a new hash and profile ID while preserving the same observation,
action, terminal and PD/safety contracts. The first such child,
`nextengine.motor.env.humanoid-reference-tracker-soft-rom-cost.v1`, adds only
`reward.soft-rom-excursion-cost`: zero inside descriptor soft ROM, then the
maximum directional excursion normalized across the descriptor-owned interval
from soft to hard ROM. A direction with coincident soft/hard bounds contributes
zero. Its coefficient is Q16 `-65536`. This warning signal does not clamp,
weaken or replace immediate hard-ROM termination and cannot authorize a policy
with any hard-safety event.

Immediate failure termination covers hard ROM/actuator/impact failures,
forbidden non-sole locomotion support after the accepted grace window,
non-finite input and declared tracking loss. `truncated` is reserved for an
external run budget and bootstraps only when its next observation is valid.

PPO/GAE, network and optimizer fields remain immutable training configuration,
not public motor wire. The rollout records sampled transformed action and its
log-prob separately from the canonical post-safety applied target. Evaluation
uses the deterministic transformed mean.

### Correspondence and claim boundary

Canonical CPU and Isaac paths consume the same profile/corpus/descriptor
hashes and compare reference selection, phase, feature order, reward
components, terminal facts and applied targets. GPU floating physics uses only
predeclared per-field tolerances; final candidate evaluation returns to the
canonical PhysX/headless path.

This ADR authorizes an environment implementation and TRAIN-5 tracker
experiments after TRAIN-4 `Advance`. It proves no tracker quality, command
locomotion, recovery, export or runtime learned-policy support. It introduces
no `PhysicalActionChunk` consumer.

## Product checks

- `MOTOR-REFERENCE-ENV-P1` covers profile/manifest hashes, split isolation,
  seed/clip/phase/reset permutations, 435-channel order, reference-root chains,
  reward/termination and checkpoint continuation.
- `MODEL-MIRROR-P2` adds reference selection/features/reward/terminal and
  applied-target correspondence for the exact biomechanics descriptor.
- TRAIN-5 additionally requires the pre-acceptance sanity ladder, complete
  held-out report-only metrics, zero hard-safety/forbidden-contact events and
  visual review from the execution plan.

## Rollback

Remove the V3/profile consumer as one unit and retain the TRAIN-4 corpus and
all V1/V2 environments unchanged. Never relabel a different corpus,
observation, reward, reset or horizon profile with these hashes.
