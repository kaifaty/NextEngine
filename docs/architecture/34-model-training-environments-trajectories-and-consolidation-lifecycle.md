# SPEC-34: Model-training environments, trajectories and consolidation lifecycle

| Field | Value |
|---|---|
| ID | SPEC-34 |
| Status | Proposed |
| Lifecycle | Optional R8 behavior and optional R5/R8 motor R&D proposal |
| Version | 1.16 |
| Last verified | 2026-09-05 |
| Normative dependencies | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-12](12-vertical-slice-conformance.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-33](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [SPEC-35](35-deterministic-humanoid-training-substrate.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](adr/048-direct-exact-project-lock.md), [ADR-053](adr/053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-054](adr/054-bounded-strategic-adaptation-and-two-tier-sleep.md), [ADR-058](adr/058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-064](adr/064-canonical-flat-command-locomotion-environment.md), [ADR-065](adr/065-curriculum-flat-command-locomotion-profile.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-067](adr/067-stage0-profile-identity-and-curriculum-hash-closure.md), [ADR-068](adr/068-static-morphology-cache-and-action-chunk-field-closure.md), [ADR-070](adr/070-biomechanics-reference-tracking-training-environment.md), [ADR-100](adr/100-bounded-standing-reward-profile.md) |
| Supersedes | SPEC-34 1.15; predeclares a separately closed final-weight inference evaluation |
| Additional dependencies 1.16 | [ADR-110](adr/110-applied-command-stop-window.md), [ADR-111](adr/111-final-weight-corrected-walking-evaluation.md) |
| Additional dependencies 1.14 | [ADR-109](adr/109-observable-sole-lift-and-return.md) |
| Additional dependencies 1.13 | [ADR-108](adr/108-observable-periodic-walking-credit.md) |
| Additional dependencies 1.12 | [ADR-106](adr/106-walking-reference-and-leg-clearance-audit.md), [ADR-107](adr/107-canonical-cpu-walking-learner.md) |
| Related Proposed lane | [SPEC-44](44-neural-assisted-world-simulation.md), [ADR-080](adr/080-neural-assistance-as-bounded-proposals.md) |

## Status and scope

SPEC-34 defines the Proposed common reset/step/trajectory/reward/dataset/run/
export data plane for Strategic, Tactical and future Motor training lanes.
ADR-058/SPEC-35, ADR-064, ADR-065, ADR-070 and ADR-100 accept only the bounded
first-humanoid standing V1/V2, flat-command/curriculum environments plus the
biomechanics `MotorReferenceTrackingProfileV1` and
`MotorTrainingEnvironmentManifestV3`. Existing seed/reset/step/trajectory
records, protocol-v2 client/recorder boundary and Isaac correspondence mirror
remain current for both manifest versions. V3 adds immutable task-input and
corpus provenance roots; it does not accept dataset consolidation, PPO,
learned artifacts, creator SDK/UI or any other lane. Those remain Proposed.
ADR-106 also accepts walking V4/V5 environment diagnostics, preserving frozen
older consumers. Their reachability reports grant no optimizer, gait-quality
or MODEL-MIRROR admission; V5 has a distinct body/action identity and cannot
inherit standing weights implicitly.
ADR-107 separately admits one direct canonical CPU walking V5 experiment,
with CUDA neural computation and a CPU-specific hash-closed generation. It
supersedes the no-optimizer sequencing only for that run; it does not change
the common lifecycle, authorize Isaac training or promote learned quality.
[ADR-108](adr/108-observable-periodic-walking-credit.md) admits one diagnosed
V6 canonical-only periodic-credit successor under the same generation closure
and quality boundary. Its observation/reward identities are new; body, action
and safety identities remain those of V5. No other lane is affected.

[ADR-109](adr/109-observable-sole-lift-and-return.md) admits one diagnosed V7
successor with actual sole-height observations and per-foot lift/return costs.
It retains the canonical generation closure and final quality boundary; nested
predeclared diagnostic evaluations are hash-closed but never checkpoint
selection or stage advancement. The common training lifecycle remains Proposed.

[ADR-110](adr/110-applied-command-stop-window.md) adds the distinct V8 command
schedule and native controls only. It repairs the applied stop window without
reinterpreting V7, launching an optimizer or admitting a checkpoint/evaluation.
[ADR-111](adr/111-final-weight-corrected-walking-evaluation.md) subsequently
admits only final-weight inference transfer into a separate V8 evaluation.
Verify the completed source generation/profile/checkpoint closure and exact
body/action/observation compatibility, record both identities and retain old
results. Its corrected load-and-release metric cannot relabel a frozen run;
no optimizer state is resumed and no runtime authority is granted.

ADR-056 makes Strategic/Tactical training optional R8 quality work; this data
plane is not a prerequisite for deterministic R4 or v1. Motor training remains
an independent optional track behind the procedural baseline. ADR-066 defines
a family-based hierarchical target: first fixed humanoid, then typed contact/
action chunks, within-family graph/shared-joint transfer, bounded equipment/
injury/weapon/parkour, distillation/compiled-student/rollout and additional-
family profiles. It does not restore a universal Mamba foundation requirement.

Production `headless` is canonical. Accelerated simulators, trainers,
experiment trackers and inference runtimes are private replaceable adapters.
PyTorch, PPO, MAPPO, Isaac Lab, MLflow, ONNX Runtime and Mamba types do not
enter public gameplay contracts.

## Invariants

- Environment reset and step use production-shaped inputs and the same
  observation/proposal/validator/commit paths as gameplay.
- Rewards and labels read immutable facts, receipts and committed events; they
  never mutate the world or define success by a private trainer flag.
- Exact environment, schema, project/content and seed closure is validated
  before the first mutable episode.
- Every future-decision-affecting policy state is explicit both before and
  after a step. Hidden evaluator or session state invalidates the record.
- `terminated` means a domain terminal condition; `truncated` means an external
  declared limit. The two are never collapsed into an ambiguous `done` bit.
- Runtime weights are immutable. Offline training or consolidation produces a
  new immutable candidate bundle and never edits active state.
- Dataset/checkpoint/run/model-output bytes stay outside Git. Only bounded
  schemas, manifests, generated/CC0 fixtures and golden vectors may enter it.
- Player gameplay trajectories are not collected by default. Ingestion is
  explicit opt-in with consent, redaction, purpose and retention metadata.

## Determinism classes

The manifest names the guarantee for each phase:

1. `CanonicalEnvironmentReplay`: identical manifest, initial closure, episode
   ID, recorded input and RNG states produce byte-exact canonical reset, step,
   trajectory, reward-component and authoritative root records wherever the
   governing runtime SPEC requires exactness.
2. `EvaluatorCorrespondence`: trainer/export/runtime raw tensors may use a
   declared numeric tolerance only as a diagnostic; canonical actions,
   candidate choices, fixed-point recurrent state and applied roots remain
   exact where required by SPEC-27/SPEC-32.
3. `StatisticalTrainingOutcome`: stochastic optimization is accepted across a
   pre-registered finite seed set and held-out suite. The manifest fixes sample
   budgets, evaluation mode, episode count, confidence/effect-size method,
   practical thresholds and complete baseline identities.

No document or report may call class 3 training byte-exact. No class 2 raw
tolerance may excuse a different canonical runtime decision or state root.

## Common environment contract

### `ModelLaneV1`

The lane is exactly `Strategic`, `Tactical` or `Motor`. A lane-specific profile
binds the engine-owned observation, proposal/action, recurrent-state and reward
schemas. A record cannot change lane mid-episode.

SPEC-44 future world-solver advice does not expand this closed enum or reuse
`Motor` as an alias. It may reuse the lineage, split, dataset, run and immutable
bundle principles of this SPEC in external research. A new lane/profile schema
is introduced only with a production consumer and Accepted successor decision;
until then every `WORLD-NEURAL-*` artifact is Proposed and non-current.

### `ModelTrainingEnvironmentManifestV1`

```text
ModelTrainingEnvironmentManifestV1 {
  schema_version: 1,
  environment_id: NamespacedId,
  environment_revision: u32,
  lane: Strategic | Tactical | Motor,
  engine_build_hash: Hash256,
  project_lock_hash: Hash256,
  schema_registry_hash: Hash256,
  content_manifest_hash: Hash256,
  scenario_fixture_hash: Hash256,
  observation_schema_hash: Hash256,
  candidate_or_action_schema_hash: Hash256,
  proposal_schema_hash: Hash256,
  policy_state_schema_hash: Hash256,
  reward_profile_hash: Hash256,
  termination_profile_hash: Hash256,
  rng_derivation_profile_hash: Hash256,
  canonical_stream_descriptors: CanonicalSet<RngStreamDescriptorV1>,
  correspondence_profile_hash: Hash256,
  resource_limits: TrainingEnvironmentLimitsV1,
  input_provenance_root: Hash256,
  manifest_hash: Hash256,
}
```

Every referenced byte sequence resolves from an immutable closure before reset.
The manifest contains no filesystem path, credential, trainer object, GPU
device, process handle or mutable database connection.

### Seed and episode identity

`TrainingSeedPlanV1` treats seeds as required inputs. It stores one run root
seed, a versioned content-addressable derivation algorithm and the complete map
of derived named streams. Streams are separated at least by lane, environment
profile, episode, vector slot, policy sampling, opponent/curriculum and
evaluation. Ambient RNG, process ID, current time and worker order are
forbidden.

```text
EpisodeIdentityV1 {
  run_id: Id128,
  environment_manifest_hash: Hash256,
  split: Train | Validation | HeldOut | Correspondence | Retention,
  episode_ordinal: u64,
  vector_slot: u32,
  episode_seed_descriptor_hash: Hash256,
}
```

Parallel execution closes a logical step in stable `(episode_ordinal,
vector_slot)` order. Worker completion order is never record order or RNG
identity.

### Reset and step

```text
reset(environment_hash, EpisodeIdentityV1, InitialClosureRefV1)
  → EpisodeResetRecordV1

step(episode_id, step_index, ProposalEnvelopeV1)
  → ModelTrainingStepRecordV1
```

`EpisodeResetRecordV1` binds exact source revisions, initial save/fixture root,
initial observation and candidate/action hashes, initial policy-state record
and hash, pre/post named RNG states, owner-segment roots and reset reason.

`ModelTrainingStepRecordV1` contains at least:

```text
ModelTrainingStepRecordV1 {
  schema_version: 1,
  lane: ModelLaneV1,
  episode: EpisodeIdentityV1,
  step_index: u64,
  simulation_tick_before: u64,
  simulation_tick_after: u64,
  source_revision_root: Hash256,
  observation_hash: Hash256,
  candidate_or_action_set_hash: Hash256,
  pre_policy_state_hash: Hash256,
  proposal_hash: Hash256,
  proposal_disposition: Applied | Rejected | FallbackApplied,
  applied_receipt_hash: Option<Hash256>,
  committed_event_root: Hash256,
  post_policy_state_hash: Hash256,
  next_observation_hash: Hash256,
  next_candidate_or_action_set_hash: Hash256,
  reward_component_root: Hash256,
  reward_total: FixedPointValueV1,
  terminated: bool,
  truncated: bool,
  termination_reason: Option<NamespacedId>,
  truncation_reason: Option<NamespacedId>,
  failure: Option<StableTrainingFailureV1>,
  pre_rng_state_root: Hash256,
  post_rng_state_root: Hash256,
  authoritative_root_before: Hash256,
  authoritative_root_after: Hash256,
  record_hash: Hash256,
}
```

The record stores references to canonical bytes defined by the lane SPEC; it
does not duplicate an ECS snapshot or vendor tensor. Invalid proposals are
recorded as a typed disposition/failure and cannot be relabelled as successful.

### Reward records

`RewardComponentRecordV1` contains component ID/version, fixed-point value,
weight, cited immutable fact/receipt/event revisions, profile hash and
component hash. Components sort by ID, use checked fixed-point accumulation and
form `reward_component_root`. A reward without cited production facts, with an
unknown component, overflow or direct-mutator source rejects the step/run.

Reward profiles are lane-specific and immutable. Changing shaping, clipping,
normalization or terminal bootstrap semantics creates a new profile hash. The
record preserves the distinction between domain termination and truncation so
trainers can apply the declared bootstrap rule correctly.

### Structured progress supervision

A future physical-task consumer MAY add bounded phase/progress/success/failure
labels to trajectory records. Every label cites immutable structured engine
evidence: source task/intent/skill revisions, predicate IDs, committed
physics/contact/query/`PhysicalOutcome` roots and the exact owner revision that
validated a terminal result. Labels are sorted by stable ID, use bounded
integer or fixed-point values and never become a private trainer success flag.

Rendered frames, camera images, depth buffers and visual embeddings are not
inputs to this proposed data plane. The engine already owns the relevant
physical facts and explicitly selects the capability-filtered structured
features exposed to the policy. A future perception-as-gameplay consumer would
require its own architecture decision and `PerceptionFrame` contract; it is not
silently introduced through training data.

## Trajectory and dataset closure

### `TrajectoryManifestV1`

A trajectory contains reset record hash, ordered contiguous step-record hashes,
terminal/truncation disposition, cumulative reward-component roots, final
owner roots and trajectory root. Missing/duplicate/reordered steps, a broken
pre→post policy-state/RNG/root chain or post-terminal step reject the complete
trajectory.

Trajectory sources are exactly:

- `CanonicalHeadless`;
- `AcceleratedMirror` with correspondence profile/result;
- `AuthoredTeacher` or `ProceduralTeacher` through production inputs;
- `OptInGameplay` with consent/redaction/retention closure;
- `SyntheticGenerated` with generator/config/license provenance.

### `DatasetManifestV1`

```text
DatasetManifestV1 {
  schema_version: 1,
  dataset_id: NamespacedId,
  dataset_revision: u32,
  lane_set: CanonicalSet<ModelLaneV1>,
  trajectory_roots: CanonicalSet<Hash256>,
  split_assignment_root: Hash256,
  environment_manifest_roots: CanonicalSet<Hash256>,
  teacher_and_procedural_source_root: Hash256,
  lineage_parent_roots: CanonicalSet<Hash256>,
  consent_policy_hash: Hash256,
  redaction_policy_hash: Hash256,
  retention_policy_hash: Hash256,
  license_and_notice_root: Hash256,
  content_hash: Hash256,
}
```

Train/validation/held-out/retention split membership is fixed before evaluation
and based on stable scenario/content/provenance identity, not worker or file
order. Duplicate trajectory content across prohibited splits rejects the
dataset. Unknown license, consent or redaction status excludes the affected
data rather than weakening the fallback.

Canonical first-party corpora originate from production `headless`. Gameplay
capture is opt-in and purpose-bound; no default telemetry, microphone/dialogue
collection or silent future-training consent exists.

Motion-corpus admission validates the intended training, commercial use,
derivative/model-output and redistribution rights independently. A
noncommercial, no-derivatives, unknown or otherwise incompatible license
excludes the affected bytes from a commercial/distributable candidate; a
popular research dataset is not an implicit grant. Own or commercially
licensed capture is the default production source. Dataset bytes remain
outside Git even when their manifest/provenance is admissible.

## Accelerated mirror correspondence

`EnvironmentCorrespondenceManifestV1` binds canonical headless build/profile,
mirror adapter/build/config, shared reset/action corpus, observation/action/
event/state comparison rules and allowed diagnostic numeric tolerances.

For every corpus episode, both environments receive the same logical inputs and
seed descriptors. The suite compares reset facts, step termination/truncation,
canonical observation/action projections, proposal dispositions, committed
outcomes and declared state roots. Any mismatch outside the exact profile
blocks use of the mirror-generated corpus for artifact promotion.

Passing correspondence does not make the mirror authoritative. Every candidate
bundle still receives final held-out and integrated evaluation in production
`headless`.

## Training run and evaluation manifests

### `ModelTrainingRunManifestV1`

The run manifest records:

- run ID, lane, exact dataset/environment/config/tool/source hashes;
- algorithm-family label and private-adapter identity without making it public
  gameplay API;
- root seed, complete derived seed map and named RNG profile;
- vector slot count and stable episode assignment profile;
- sample/step/episode/compute budgets and stop reason;
- parent/checkpoint provenance, curriculum revisions and normalization state;
- for multi-agent runs: opponent snapshot pool, matchmaking/curriculum rules,
  self-play lineage and privileged-critic schema hash;
- every produced checkpoint/result reference, including failed runs;
- exact export and evaluation manifest references.

Checkpoints, optimizer state and logs remain external content-addressed bytes.
The repository manifest may refer to them only when redistribution and hygiene
rules permit; a reference never grants access or license.

### `StatisticalEvaluationManifestV1`

Before an acceptance run, the manifest fixes:

- complete candidate and baseline bundle identities;
- exact train/validation/held-out/retention/correspondence suites;
- seed set and derivation profile;
- sample budgets and evaluation points;
- deterministic or stochastic deployment-matching evaluation mode;
- evaluation episode count or power-analysis rule;
- primary/secondary metrics, worst-case safety assertions and candidate
  coverage metrics;
- confidence interval/significance method, effect-size method, alpha where
  used and minimum practical improvement/non-inferiority thresholds;
- failure, missing-seed and early-stop handling.

Single-seed, best-checkpoint cherry-picking, training-return-only evaluation or
same-fixture train/eval claims cannot promote a candidate. Offline estimates
alone are insufficient; final production-headless rollouts are required.

## Export and immutable candidate bundles

`ModelExportManifestV1` binds source checkpoint, source run/config/tool hashes,
input/output/state schema hashes, deterministic export transform, model-format
profile, operator/capability closure, numeric profile, output bytes/hash and
target parity corpus.

The first proposed runtime format profile is fixed-shape standard-op ONNX with
explicit recurrent state and no stochastic/custom Mamba operator. Framework or
fused training kernels may differ; their output must lower to this portable
one-step contract or the candidate is rejected.

Publication follows:

```text
external checkpoint
  → deterministic export
  → reopen and validate exact bytes
  → schema/operator/correspondence/runtime parity
  → held-out/statistical/safety/retention evaluation
  → immutable candidate bundle or rejection
```

An active session never changes bundle. A promoted candidate is selectable
only through a new exact project lock and new session. Malformed schema/hash/
provenance/license input is rejected before evaluator creation or artifact use.

## Offline consolidation lifecycle

`OfflineConsolidationManifestV1` binds parent bundle, immutable corpus/dataset,
seed plan, consolidation algorithm/config/tool hashes, retention and new-task
suites, joint Strategic/Tactical suite where applicable, and child export
identity. It creates a new child bundle; it cannot update the parent or active
project/save.

Strategic Hope-inspired consolidation additionally records the parent
multi-timescale state schema family and proves that no runtime optimizer or
hidden session state is required. The child must pass retention and current
task non-regression thresholds before publication. Parent/child lineage is
acyclic and content-addressed.

## Lane specializations

### Strategic

Uses the SPEC-32 strategic candidate/state contract and SPEC-33 curriculum.
Teacher path is authored Utility + bounded GOAP demonstration → behavior cloning →
bounded RL. Hope-inspired state is compared against a GRU with the same public
envelope and against the deterministic Utility + bounded GOAP fallback.

### Tactical

Uses the SPEC-32 composite candidate/mask contract. Reference training profile
is set/cross-attention encoders plus GRU and one masked score per canonical
candidate: behavior cloning → recurrent PPO/IPPO → gated MAPPO/CTDE and
opponent-pool self-play. A privileged critic is training-only; actor inputs and
export graphs contain only engine-visible semantic facts. Leakage or latent
learned communication is a hard failure.

### Motor

Uses SPEC-14 `BodySchema`/skill/family target and SPEC-27 observations, action
schemas, safety clamp and explicit policy state. The Proposed curriculum is
ordered; a later stage cannot claim support without retaining earlier-stage
behavior:

0. BodySchema compiler, articulation, fixed PD/SPD, deterministic observation,
   replay and batched environment; authored standing is stable without neural
   policy.
1. Licensed motion ingestion, canonical skeleton, retarget, contact detection
   and physical-feasibility validation.
2. Reference tracking with randomized starts and perturbed states.
3. Velocity/facing/start/stop/turn/crouch locomotion and interruption.
4. Slopes, stairs, heightfields, narrow/moving and varied-friction terrain.
5. Pushes, missed contacts, stumble, brace, safe fall, ragdoll and get-up.
6. Motion prior/multi-skill, transition data, specialist teachers and
   distillation.
7. Typed physical primitives, ContactPlan and closed-loop
   `PhysicalActionChunk` tracking without natural-language inputs.
8. Explicit physical parameters plus bounded no-gradient dynamics adaptation.
9. Equipment, carried load, global/local fatigue and pickup/drop.
10. Damage, ROM/force/sensor changes, topology masks and recovery fallback.
11. Manipulation, grips, carry, throw/catch and weapon classes.
12. Authored/contact-planned parkour specialists and safe missed-contact
    failure.
13. Within-family morphology randomization, local-graph/global-attention/shared-
    joint policy and held-out
    morphology/topology evaluation.
14. Optional `K`-candidate cloned-physics chunk evaluation under one canonical
    checkpoint and fixed-point scoring profile.
15. Exact in-engine rollout, sim-to-sim correspondence, residual/adaptation
    fine-tuning, final distillation and portable export.

The morphology-transfer hypothesis is evaluated by an explicit Proposed
profile rather than inferred from aggregate reward. It binds the source and
target `BodySchema`/instance revisions, morphology-family relation, training
and held-out split, zero-shot or few-shot mode, demonstration/step budget,
baseline and candidate identities, shared semantic skill/subgoal set and the
complete metric suite. Arbitrary-topology and cross-regime support cannot be
inferred from a within-family result.

The minimum ablation ladder is fixed body → morphology randomization →
graph-conditioned policy → multi-embodiment co-training → shared semantic
physical subgoals → explicit transfer objective → actuator/topology faults.
Each rung retains the same held-out tasks, source/target identities and compute
budget where the comparison requires it. Transfer reporting includes task
success, zero-shot success, few-shot sample efficiency, recovery after
disturbance/actuator failure, contact stability, energy, safety violations,
inference latency and the number of body-specific rules. Claims are labelled
`WithinFamilyTransfer` or `CrossFamilyResearch`; neither label grants a runtime
route or `Supported` status.

Specialist teachers are trained independently for bounded locomotion regimes,
flight, climbing, manipulation/weapon, recovery and other contact-rich skills.
Their immutable trajectories first supervise a family/shared-latent student;
only then does joint continuous-control RL fine-tune the student. PPO is the
first simple optimizer profile for the position/velocity-target actor, with
GAE, normalized advantages, declared reward scaling and hard safety enforced
outside reward. A later algorithm must beat this equal-budget baseline.

Actor and critic do not have to share the same conditioning. The runtime actor
receives only engine-visible SPEC-27 facts. The training critic SHOULD be more
strongly conditioned on morphology, capabilities, skill/regime and MAY use
declared family value heads so physically unequal bodies do not share a
miscalibrated baseline. Critic-only privileged facts are named in the training
manifest, never exported and never copied into actor input, runtime state,
action identity or replay.

A morphology-conditioned motion prior MAY be trained as structured motion
inpainting over masked poses/keypoints, contact schedules, object trajectories,
styles and typed physical goals. Text captions are optional offline authoring/
provenance labels only: dataset construction compiles them into the same stable
IDs and numeric fields before a training batch, and runtime text conditioning
is not a requirement. Pose/keypoint values here are training conditions and
loss targets; before runtime publication they compile into the ADR-068 chunk's
root/center-of-mass, effector/object, contact, force and support fields. They do
not add pose/keypoint wire fields to `PhysicalActionChunk`. The prior proposes
a chunk; it never owns pose, contact or actuator authority.

Damage/fault distribution explicitly varies disabled/locked/removed actuators
or nodes, reduced torque/ROM, changed mass/load, sensor delay/loss and topology
changes. Health/power/enabled masks are observations. A hidden fault may be
inferred only through the explicit bounded temporal state.

After the family-wide teacher passes quality and parity, an optional
morphology hypernetwork MAY generate an immutable small MLP/GRU child or
adapter for one exact BodySchema/projection. The child has an ordinary bundle
hash, compatibility key, golden corpus, fallback, retention and runtime parity
gate. Topology/effective-projection change invalidates it; generation never
mutates weights in an active gameplay session.

The first learned target is the ADR-066 fixed humanoid MLP at 60 Hz with
residual joint-position targets and fixed engine PD at 240 Hz. TCN and GRU are
the first adaptation comparators. Mamba is only an equal-parameter/context/
latency comparator for long-history adaptation, motion generation or temporal
planning; export success alone cannot promote it. Adaptive gains, direct torque
and muscles require separate profiles/gates.

Motor reward profiles cite immutable production facts and separate task,
pose/velocity/keypoint tracking, contact/object, style and recovery components
from energy, smoothness, impact and joint-limit costs. Curriculum also varies
command acceleration, transition density, delay, parameter/contact
randomization, perturbations and mid-episode changes. Reward never replaces a
hard actuator/safety limit.

The first Proposed replaceable toolchain is Isaac Lab/PhysX + ProtoMotions +
RSL-RL + PyTorch. Production `headless` remains canonical; each accelerated
mirror passes body/joint/axis/actuator/contact correspondence and every
candidate receives final headless evaluation. Export is fixed-shape standard-
op ONNX with explicit state and a private ONNX Runtime adapter; MuJoCo/MJX or
another stack may replace it behind the same records.

The motor benchmark profile covers standing/balance, command locomotion,
terrain, jumps/landing, recovery, manipulation/weapons, fatigue/damage,
morphology, skill retention/transitions and runtime latency/memory/replay. It
records task success/fall rate, velocity/facing error, contact precision/timing/
slip, impact, energy, saturation/jerk, recovery time, adaptation half-life/
regret, zero-shot transfer, few-shot adaptation budget, cross-sim gap,
body-specific rule count, retention and p50/p95 inference latency.

The Proposed toolchain profile is informed by
[Isaac Lab](https://developer.nvidia.com/isaac/lab),
[ProtoMotions](https://github.com/NVlabs/ProtoMotions),
[RSL-RL](https://github.com/leggedrobotics/rsl_rl),
[MuJoCo MJX](https://mujoco.readthedocs.io/en/stable/mjx.html),
[MuJoCo Playground](https://github.com/google-deepmind/mujoco_playground),
[MyoSuite](https://github.com/MyoHub/myosuite),
[KINESIS](https://github.com/amathislab/Kinesis),
[ONNX Runtime](https://github.com/microsoft/onnxruntime),
[PhysX articulations](https://nvidia-omniverse.github.io/PhysX/physx/5.6.1/docs/Articulations.html),
[Contact-Anchored Policies](https://arxiv.org/abs/2602.09017),
[Latent Action Diffusion](https://arxiv.org/abs/2506.14608),
[GCNT](https://arxiv.org/abs/2505.15211),
[Shared Modular Recurrence](https://arxiv.org/abs/2506.08630),
[HyperDistill](https://arxiv.org/abs/2402.06570),
[MorFiC](https://arxiv.org/abs/2603.14554),
[MaskedMimic](https://research.nvidia.com/labs/par/maskedmimic/),
[Random Joint Masking](https://arxiv.org/abs/2403.00398),
[Gemini Robotics 1.5](https://arxiv.org/abs/2510.03342),
[Gemini Robotics 2](https://deepmind.google/blog/gemini-robotics-2-brings-whole-body-intelligence-to-robots/) and
[Gemini Robotics ER 2](https://deepmind.google/blog/gemini-robotics-er-2-powering-robotics-with-video-understanding-task-orchestration-and-multi-robot-collaboration/).
These are replaceable implementation references, not public contract types.
The Gemini sources motivate hierarchical task/action separation, bounded
reference chunks, progress supervision and transfer evaluation only. Their
vision inputs, closed model architecture, robot-specific low-level stack and
reported embodiment breadth are neither runtime requirements nor evidence that
Next Engine supports an arbitrary morphology.

Candidate motion sources include
[AMASS](https://amass.is.tue.mpg.de/),
[LAFAN1](https://github.com/ubisoft/ubisoft-laforge-animation-dataset),
[Motion-X](https://github.com/IDEA-Research/Motion-X) and
[MoCapAct](https://arxiv.org/abs/2208.07363). Their presence here does not grant
training or redistribution rights: the exact corpus license/provenance record
must pass `MODEL-DATA-GOVERNANCE-P1`, and incompatible noncommercial,
no-derivatives or unknown terms exclude that corpus from the candidate.

## Stable failure semantics

| Failure | Required outcome |
|---|---|
| Manifest/schema/project/content/hash mismatch | Reject before first episode or evaluator use; preserve source bytes and fallback. |
| Ambient/missing seed, duplicate episode identity or unstable vector merge | Invalidate run; no trajectory/dataset/artifact promotion. |
| Broken step/state/RNG/root chain or post-terminal step | Reject the complete trajectory. |
| Reward without immutable citations or direct success mutation | Invalidate step, run and derived artifact. |
| Unknown consent/redaction/license/provenance | Exclude/reject affected data before training or distribution. |
| Mirror correspondence mismatch | Block mirror-derived corpus and candidate promotion; evaluate/fallback in canonical headless. |
| Missing declared seed/run, cherry-picked result or held-out overlap | Statistical gate fails; candidate remains unpromoted. |
| Hidden recurrent cache, stochastic/custom unsupported export op | Reject candidate before runtime parity. |
| Runtime/trainer canonical decision or state-root mismatch | `NONDETERMINISTIC_RESULT`; reject exact artifact/adapter pair. |
| Retention or joint evaluation failure | Reject child bundle; parent/project lock remain unchanged. |
| Runtime training or active-session hot swap request | Deny capability; active weights/state remain unchanged. |

## Proposed ProductChecks

For the SPEC-35 fixed-humanoid consumer, `MODEL-DATAPLANE-P1` and
`MODEL-MIRROR-P1` become current Stage 0 gates. Other rows remain
`NOT_RUN(NO_PRODUCTION_CONSUMER)` until their corresponding consumer exists.

| ID | Scenario | Required result / fallback |
|---|---|---|
| `MODEL-DATAPLANE-P1` | Recreate reset/step/trajectory/reward records twice from one exact manifest, including vector worker/completion permutations | Byte-exact canonical records and state/RNG/root chains; malformed hash/schema/provenance rejects before use. |
| `MODEL-MIRROR-P1` | Run one correspondence corpus through production headless and the accelerated mirror | Every profile-required projection/result corresponds; any mismatch blocks mirror-derived artifact promotion. |
| `MODEL-EXPORT-P1` | Export explicit-state candidate and compare trainer → portable graph → Windows/Linux runtime one-step/rollout corpus | Exact canonical decisions/actions and state roots, no hidden/custom/stochastic op; failure retains planner/procedural fallback. |
| `MODEL-STATISTICS-P1` | Pre-registered multi-seed held-out comparisons for Hope vs GRU, Tactical PPO/MAPPO vs authored baseline, fixed-humanoid MLP vs procedural reference, TCN vs GRU adaptation, shared vs morphology-conditioned/family-head critic, and any optional Mamba experiment vs the matching simpler comparator | Complete seed set, confidence/effect size and practical thresholds pass; single/best seed, export availability or architecture label cannot promote. |
| `MODEL-CONSOLIDATION-P1` | Parent + immutable corpus → offline child → retention/new-task/joint evaluation | A new content-addressed child is published only after every gate; parent, project, world and save bytes remain unchanged. |
| `MODEL-DATA-GOVERNANCE-P1` | Opt-in gameplay, teacher, synthetic and malformed provenance/license/consent/redaction fixtures | Only declared data enters a dataset; unknown/incompatible input is rejected and no protected bytes enter Git/package. |
| `MOTOR-DISTILL-P1` | Specialist teacher trajectories → family/shared student → optional morphology-compiled child | Required old/new skills, structured chunk semantics, exact runtime action/state parity, limits and fallback are retained; failed child is not published. |
| `MOTOR-ROLLOUT-P1` | Fixed `K` candidate chunks and one canonical checkpoint under worker/completion permutations | Ordered fixed-point evidence, winner and selected-chunk roots are exact; branch state/RNG/effects never leak. |

Applicable learned-lane ProductChecks in SPEC-27 or SPEC-33 remain required.
SPEC-32 deterministic R4 checks are independent and never wait for this data
plane. A common data-plane pass cannot substitute for policy quality, gameplay
integration, safety, persistence/replay or runtime performance checks.

## Promotion boundary

SPEC-34 may become `Accepted` only with at least one production lane consuming
its manifests through canonical headless reset/step, immutable dataset/run/
export closure and project-lock activation, with `MODEL-DATAPLANE-P1`,
`MODEL-EXPORT-P1`, applicable correspondence/statistical/governance checks and
the lane's own ProductChecks passing. Scaffolding, a local notebook, smoke
trainer or model file is not a production consumer under ADR-046.
