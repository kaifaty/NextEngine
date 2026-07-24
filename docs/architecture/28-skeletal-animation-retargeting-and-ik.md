# SPEC-28: Skeletal animation, retargeting and IK

| Поле | Значение |
|---|---|
| ID | SPEC-28 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Physical Embodiment Team |
| Требуемые согласующие | Repository Owner, Architecture Working Group, Runtime Team, Asset & Persistence Team, Rendering Team, Gameplay Extensibility Team, Security & Governance Team, Verification & Evidence Team, Release Engineering |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-04](04-rendering-and-platform.md), [SPEC-05](05-physics-animation-and-motor-control.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-26](26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md), [ADR-024](adr/024-requirement-gate-evidence-and-profile-closure.md), [ADR-027](adr/027-physics-motor-and-animation-layering.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-28 подготовлен как часть consolidated architecture packet 1.8. Он
закрепляет engine-owned skeletal-animation, graph, retargeting, root-motion и
IK contracts поверх neutral content и physical layering ADR-027. Принятие
exact packet root означает только architecture admission: оно не создаёт
runtime implementation, verification gate `PASS`, `vertical-v1`,
`PhysicalCertified` или release-readiness claim.

## Назначение и invariants

SPEC-28 задаёт один backend-neutral contract, по которому cooker, animation
runtime, motor/physics bridge, renderer, headless runner и capture-worker могут
быть реализованы независимо.

- Asset & Persistence Team публикует immutable `NeutralSkeletonV1` и
  `NeutralAnimationV1`; Physical Embodiment Team владеет runtime descriptors,
  graph evaluation, retargeting, IK classification и animation/physics bridge.
- Physics or the validated locomotion controller remains the only owner of an
  embodied transform. Animation pose, marker, cue, root track, retarget result
  and IK result are never an alternative gameplay-state write path.
- Root motion is only a bounded `RootMotionIntentV1` proposal. It MUST pass the
  same production command, capability, precondition, motor/safety and physics
  boundaries as any other movement request and MUST NOT teleport a transform.
- Physical IK produces bounded motor constraints/intents before safety and
  physics. Presentation IK produces only a reconstructible pose delta after
  committed physical state. The two classes have separate schemas, stages,
  hashes and failure semantics and cannot call each other.
- Animation graph, retarget and IK execution use fixed logical ticks, declared
  integer/fixed-point profiles and canonical total orders. Renderer frame rate,
  wall time, worker completion, cache warmth and backend traversal order MUST
  NOT select a graph transition, root intent, physical constraint or command.
- For one accepted production command stream and identical authoritative
  physics inputs, clip blending, presentation graph state, retargeting,
  presentation IK, animation LOD and renderer output MUST change zero gameplay
  hashes. Any animation-derived physical influence is visible as a separately
  validated command/proposal in replay, never as hidden pose feedback.
- `game`, `headless` and `capture-worker` use the same descriptor validation,
  intent graph, root-motion validator, fixed ordering and replay contracts.
  Headless execution does not require a renderer, GPU, window or display.
- Observable animation, retarget, IK or root-motion changes always enter the
  SPEC-15 impact/capture/review flow. Automatic evidence precedes required
  `HumanReviewDecisionV2::Approve`; an agent cannot approve or waive a failure.

## Source of truth и authority split

| State / output | Единственный owner/source of truth | Allowed projection / forbidden duplicate |
|---|---|---|
| Neutral skeleton/animation record, content hash and target-neutral channels | Asset & Persistence Team, exact SPEC-24 content generation | decoded immutable view; no mutable graph/physics state |
| Skeleton compatibility, graph descriptor, intent cursor, retarget/IK descriptors and evaluation schedule | Physical Embodiment Team | immutable diagnostics/presentation views; no renderer-owned transition |
| Accepted command sequence, fixed simulation stages and commit result | Runtime Team | immutable receipts/events; no animation callback mutation |
| Active body/capsule pose, contacts, constraints and physical outcome | Physical Embodiment Team through SPEC-26 physics/motor boundary | quantized snapshot/RenderPose; no graph, IK or renderer write-back |
| Ability/effect timing and gameplay markers | Owning RPG/mechanics domain | animation may emit `PresentationCue`; an animation marker is not a gameplay event |
| Render interpolation, skinning palette, presentation IK delta and GPU cache | Rendering Team | reconstructible presentation only; excluded from gameplay roots |
| Save/replay generation and owner-segment encoding | Asset & Persistence Team | staged copy; no semantic ownership of decoded animation/physical state |

`WorldResidencyTier`, physical LOD and animation LOD remain three distinct
state machines. A committed residency view may constrain whether an embodied
subject is present; physical LOD selects its simulation representation;
animation LOD selects only declared evaluation/presentation work. None may
silently write another owner’s state.

## Public contract boundary

Public animation schemas contain only engine-owned nominal IDs, `AssetId`,
`PersistentId`, canonical hashes, bounded integer/fixed-point values and closed
enums. They contain no ECS storage/component, raw pointer, native task/thread
handle, OS/window/input object, filesystem path, importer record, database
connection, shader/GPU object, physics handle, animation-library object or
other vendor/backend type.

### `SkeletonDescriptorV1`

```text
SkeletonDescriptorV1 {
  schema_version,
  skeleton_asset_id,
  skeleton_revision,
  neutral_skeleton_hash,
  coordinate_profile_id,
  declared_root_joint_keys[],
  joints[],
  compatibility_signature
}

SkeletonJointDescriptorV1 {
  joint_key,
  parent_joint_key?,
  canonical_bind_transform,
  canonical_inverse_bind,
  semantic_roles[],
}
```

`joint_key` is the SPEC-24 record-scoped `JointKeyV1`, not an array index or
runtime handle. Joint records sort by `(tree_depth, joint_key)`. Every parent
resolves in the same exact revision; the parent graph is acyclic; declared
roots equal the parentless set. Bounds remain SPEC-24’s maximum 1,024 joints,
64 roots and 32 semantic roles per joint.

`compatibility_signature` is a domain-separated hash over coordinate/numeric
profile, root set, sorted joint keys, parents and canonical bind transforms. It
does not claim retarget compatibility by itself. Every descriptor/profile hash
in this specification is computed domain-separated over its complete canonical
bytes and stored only by an enclosing manifest/reference. A descriptor never
contains its own hash.

### `AnimationClipDescriptorV1`

```text
AnimationClipDescriptorV1 {
  schema_version,
  clip_asset_id,
  clip_revision,
  neutral_animation_hash,
  source_skeleton_signature,
  duration_us,
  wrap_mode,
  channel_table[],
  marker_table[],
  root_intent_curve?,
  animation_numeric_profile_id
}
```

The descriptor is a runtime-ready immutable projection of
`NeutralAnimationV1`, not a replacement content format. `wrap_mode` is
`Clamp`, `Loop` or `PingPong`. Channels and markers preserve SPEC-24 bounds and
sort by `(joint_key, property, key_time_us)` and `(time_us, marker_id)`.
Duplicate keys/markers with unequal bytes reject the complete clip.

Animation cursor is the exact pair `(completed_wrap_count, local_time_us)`.
Both fields are unsigned 64-bit integers; checked overflow aborts the
uncommitted graph tick.
For `PingPong`, wrap-count parity fixes direction: even is forward and odd is
reverse, starting at zero/forward. Mapping a logical animation tick or rational
playback rate to microseconds uses checked integer arithmetic and
round-to-nearest-ties-to-even from the locked numeric profile. Floating
accumulated time, host media clocks and render delta time are forbidden inputs.

`root_intent_curve` is an authored translation/yaw intent curve in canonical
units. It is sampled before presentation retarget/IK and cannot contain a body
handle, transform pointer, collision result or preaccepted outcome.

### `AnimationGraphDescriptorV1`

```text
AnimationGraphDescriptorV1 {
  schema_version,
  graph_id,
  graph_revision,
  target_skeleton_signature,
  authority_partition,
  parameter_descriptors[],
  node_descriptors[],
  state_machine_descriptors[],
  layer_descriptors[],
  output_descriptors[],
  animation_numeric_profile_id
}
```

Graph parameters use the closed types `Bool`, `I32`, `U32`, `Fixed`,
`NominalId` and bounded fixed-size vectors. Free-form objects, native enum
values and implicit string-to-number conversion are invalid.

Each parameter has one source class: `CommittedIntent`,
`CommittedPhysicsProjection` or `PresentationLocal`. A
`PhysicalIntentGraph` accepts only the first two classes.
`PresentationLocal` includes camera, UI, render and accessibility presentation
facts and is legal only in a `PresentationGraph`.

V1 nodes are closed engine-owned variants:

- `ClipSample`;
- `FixedBlend`;
- `AdditiveLayer`;
- `PoseMask`;
- `StateMachineOutput`;
- `Retarget`;
- `PhysicalIkConstraint`;
- `PresentationIk`;
- `PreviousTickPose`;
- `Output`.

The ordinary node graph is acyclic. Feedback is legal only through an explicit
`PreviousTickPose` node that reads the prior committed presentation snapshot;
it cannot feed `RootMotionIntentV1` or a physical constraint. State-machine
transition cycles are allowed, but a machine may take at most one transition
per animation tick.

`authority_partition` is `PhysicalIntentGraph` or `PresentationGraph`.
Presentation nodes MAY consume committed physical snapshots and immutable
intent-graph outputs. No edge, parameter binding, marker or cached pose may
flow from `PresentationGraph` to `PhysicalIntentGraph`.
`PhysicalIkConstraint` is legal only in `PhysicalIntentGraph`;
`PresentationIk` and `PreviousTickPose` are legal only in
`PresentationGraph`. A `Retarget` node’s purpose MUST match its graph
partition.

### Graph evaluation and fixed ordering

For one logical animation tick the evaluator performs the following stages:

1. validate exact graph, skeleton, clip, numeric, retarget and IK revisions;
2. freeze canonical parameters and committed physics/intent snapshots;
3. evaluate each state machine in `machine_id` order; eligible transitions sort
   by `(priority descending, transition_id ascending)` and only the first wins;
4. sample clips in `(clip_node_id, clip_asset_id)` order from the exact cursor;
5. evaluate acyclic nodes by `(topological_depth, node_id)`;
6. blend layers by `(layer_order, layer_id)` with fixed-point normalized weights;
7. extract root intent from the source curve before presentation retarget/IK;
8. evaluate retarget rules in `(target_depth, target_joint_key, rule_id)` order;
9. evaluate physical IK constraint chains, then publish bounded motor input;
10. after the committed physical snapshot, evaluate presentation IK and publish
    one immutable animation pose snapshot.

Within a stage, task parallelism is allowed only when results merge by the
declared total order. Early completion, hash-map order and backend callback
order cannot affect bytes. Fixed-point overflow, missing total-order key or
unequal duplicate ID aborts the complete evaluation with a stable diagnostic.

Forward marker traversal uses `(prior_time_us, current_time_us]`; reverse
traversal uses `[current_time_us, prior_time_us)`. A `Loop` wrap evaluates
`(prior_time_us, duration_us]` and then `[0, current_time_us]`. A `PingPong`
turn includes the endpoint in the incoming segment and excludes it from the
outgoing segment. Emission sorts by
`(logical_animation_tick, segment_ordinal, traversal_offset_us, marker_id)`;
the dedupe key additionally includes subject, graph/clip hashes and
completed-wrap count. Thus every crossed endpoint/zero marker emits exactly
once. A marker in `PresentationGraph` may produce only `PresentationCue`. A
marker in `PhysicalIntentGraph` may produce a bounded future command proposal,
which still enters the common production command boundary. The marker itself
is never a committed gameplay event.

### Numeric profile, limits and work budgets

`AnimationNumericProfileV1` binds exact fixed-point descriptors for
translation, scale, normalized rotation, blend weights, playback-rate
conversion, root translation/yaw and IK intermediate values. It also binds
versioned interpolation, normalization and solver algorithm IDs plus
round-to-nearest-ties-to-even behavior. Zero-length rotation, non-finite source
value, overflow, underflow outside a declared saturating presentation-only
field or unsupported algorithm ID rejects the complete result.

V1 closed IK algorithms are `TwoBoneAnalyticFixedV1`, `CcdFixedV1` and
`FabrikFixedV1`. Algorithm ID, operand order, fixed iteration count and numeric
profile select one canonical reference behavior and golden-vector corpus;
implementation-specific epsilon, convergence exit or unordered joint walk is
forbidden.

V1 bounds per descriptor are:

| Contract | Bound |
|---|---:|
| Graph parameters / nodes / clip references | 512 / 4,096 / 1,024 |
| State machines / total states / total transitions | 256 / 4,096 / 16,384 |
| Layers / outputs / markers emitted per subject per tick | 256 / 64 / 256 |
| Retarget rules | 1,024 |
| IK chains / joints per chain / iterations per chain | 256 / 64 / 64 |

`AnimationBudgetProfileV1` fixes maximum due intent nodes, pose nodes,
retarget rules, IK iterations, pose joints and cue emissions per logical
boundary. Required physical-intent work MUST fit the locked project profile
before activation and cannot be dropped or downgraded by measured cost.
Presentation work may use only the declared animation-LOD fallback in canonical
subject/graph order. A wall-time miss fails the relevant ANIM/PERF gate and
cannot choose a different root intent, physical constraint or authoritative
outcome in the measured run.

### `AnimationIntentStateV1` and presentation state

`AnimationIntentStateV1` is the bounded Physical Embodiment-owned state needed
to reproduce `PhysicalIntentGraph`: subject `PersistentId`, exact
graph/skeleton/clip/profile hashes, committed action/ability phase reference,
logical animation tick, clip cursors, state-machine IDs and root-intent
sequence. Its updates occur only at the declared fixed commit point and its
exact prior revision is a root-motion precondition.

`AnimationPresentationStateV1` contains presentation graph cursors,
previous-pose cache and interpolation state. It is reconstructible, stored only
in a separate presentation segment when desired and excluded from
authoritative gameplay roots. Losing it MAY reset to the declared graph start
or bind-pose fallback but MUST NOT synthesize a physical intent or gameplay
event.

### `RootMotionIntentV1`

```text
RootMotionIntentV1 {
  schema_version,
  subject_id,
  intent_sequence,
  source_graph_hash,
  source_clip_hash,
  source_action_or_ability_phase_id,
  source_animation_tick,
  interval_us,
  quantized_local_translation,
  quantized_local_yaw,
  locomotion_profile_hash,
  expected_intent_state_revision,
  expected_body_revision,
}
```

It is a non-authoritative proposal. The production path is:

```text
committed action/ability phase + exact clip cursor
→ RootMotionIntentV1
→ common capability/precondition/rate/bounds validator
→ WorldCommandEnvelopeV2 / receipt
→ PhysicalAvatarIntent or deterministic rejection
→ motor/safety layer
→ physics/capsule-controller result
→ committed DomainEvent + immutable pose snapshot
```

In `FullArticulation` and `SimplifiedActiveRagdoll`, root motion is a bounded
motor reference only. In `CapsuleAnimation`, it is a request to the validated
locomotion controller, not direct collider displacement. It cannot bypass
collision, support, speed, acceleration, energy, contact, topology, policy or
physical-LOD guards. Invalid or stale intent publishes no transform, pose,
event or partial receipt result.

Replay records the canonical proposal, command admission/rejection, receipt,
applied physical intent and committed outcome. A different root-motion
proposal is therefore a different explicit command input, never a hidden
animation-dependent gameplay divergence.

### `RetargetProfileV1`

```text
RetargetProfileV1 {
  schema_version,
  profile_id,
  profile_revision,
  source_skeleton_signature,
  target_skeleton_signature,
  purpose,
  root_scale_fixed,
  joint_rules[],
  missing_joint_policy,
  numeric_profile_id
}
```

`purpose` is `PhysicalReference` or `PresentationPose`. A physical-reference
result may create only bounded motor/IK reference features and then passes the
normal safety boundary. A presentation result may create only local render
pose values. Neither purpose writes physics transforms.

Every `JointRetargetRuleV1` names source and target `JointKeyV1`, canonical
rotation offset, translation-scale policy, twist/swing distribution, optional
chain parent and explicit required/optional status. Target joint ownership is
unique. The rule graph is acyclic and evaluates by target depth/key/rule ID.
Runtime name matching, topology guessing, “closest bone”, unordered heuristic
search and silent required-joint omission are forbidden.

`missing_joint_policy` is `RejectRequired`, `UseTargetBindPoseForOptional` or
`IgnoreOptionalPresentation`. It cannot make a required physical reference
optional. Root intent is extracted before presentation retargeting; changing a
presentation retarget profile therefore changes zero command/gameplay hashes.

### IK contracts and separation

`IkRigDescriptorV1` contains an exact skeleton signature, closed
`PhysicalConstraint` or `PresentationPose` class, bounded chain descriptors,
joint/angle/translation limits, fixed iteration counts, target-source classes,
numeric profile and profile hash.

Each `IkChainDescriptorV1` contains stable chain ID, ordered joint keys,
effector key, pole/axis rule, priority, stage, maximum per-tick delta and
fallback. V1 solvers MUST execute exactly the declared iteration count; an
implementation cannot early-exit by backend epsilon, wall time or measured
convergence. Intermediate values use the locked checked fixed-point profile.

`PhysicalIkConstraintV1`:

- reads only committed physical snapshot, validated gameplay/interaction
  target and exact body/rig revisions;
- produces bounded joint/effector targets for the motor/safety layer;
- cannot set joint transforms, contacts, impulses or gameplay outcome;
- evaluates before physics and fails as one atomic constraint set.

`PresentationIkRequestV1`:

- reads the immutable committed physical projection plus the current pre-IK
  local animation pose and bounded presentation target;
- produces `PresentationPoseDeltaV1` after physics;
- cannot feed root intent, motor observation/action, collision/query/contact,
  targeting, command validation or save gameplay state;
- MAY be disabled independently without changing command, event, physics or
  gameplay roots.

A chain cannot appear in both classes in one rig revision. Cross-class target
references and presentation-to-physical graph edges are schema errors.
Foot/hand/look-at presentation IK never proves support, hit, grab or reach;
only physics/query/contact outcomes may do so.

### Pose snapshot and animation LOD

```text
AnimationPoseSnapshotV1 {
  schema_version,
  subject_id,
  logical_animation_tick,
  source_physics_tick,
  skeleton_hash,
  graph_hash,
  clip_cursor_root,
  animation_lod_id,
  ordered_local_joint_pose[],
  presentation_cues[]
}
```

The snapshot is immutable presentation data. It contains no mutable entity,
backend palette/buffer, native resource, physical contact or accepted gameplay
result. It is the animation-local versioned input used to construct the
existing `RenderPose`, not a second physical-pose owner. `PresentationCue` from
a marker is non-authoritative; a mechanic that needs gameplay timing schedules
that timing in its owning command/state machine rather than consuming an
animation marker.

`AnimationLodProfileV1` declares the closed levels:

- `IntentOnly`: evaluate every due physical-intent node; publish no pose;
- `ReducedPose`: evaluate declared required joints/layers at fixed cadence;
- `FullPose`: evaluate the complete locked pose graph;
- `HeldPresentationPose`: reuse a bounded previous presentation pose while all
  due intent work still evaluates;
- `CulledPresentation`: publish no pose while all due intent work still
  evaluates.

Animation LOD never changes physical LOD or `WorldResidencyTier`, and never
skips due `PhysicalIntentGraph` work. Canonical simulation-owned facts may
select a project-declared animation LOD/cadence. Camera/frustum/occlusion and
measured render cost MAY choose only a presentation level and are excluded
from command/gameplay roots. Capture evidence uses one exact pinned LOD/profile
hash; an unpinned adaptive capture cannot be a pixel baseline.

LOD transition happens at a logical animation-tick boundary and publishes one
complete new presentation snapshot or none. Missing optional presentation work
uses the declared bind/held/cull fallback. Missing required physical-intent
work blocks/defer/rejects through its owning command path; it is never silently
reclassified as presentation-only.

## Loading, save, replay and fault recovery

Skeleton, clip, graph, retarget, IK and LOD descriptors are resolved by exact
`ProjectCompositionLock`/`ContentManifestV1` hashes. Async fetch, decode or
resource preparation uses immutable staged results. Completion becomes visible
only through SPEC-21 deterministic result merge at a declared boundary.
Partial skeleton, clip, graph, rig or pose publication is forbidden.

Load validates the complete closure before subject activation:

- schema/content/profile hashes and descriptor bounds;
- skeleton parent graph and compatibility signatures;
- clip channels/timing/root-intent curve;
- graph DAG, state transitions, authority partitions and total-order IDs;
- retarget source/target signatures, rule uniqueness and missing-joint policy;
- IK class, chain ownership, limits, iteration count and target sources;
- LOD required-intent closure and declared fallback.

Save stores exact descriptor/profile hashes and `AnimationIntentStateV1` in the
Physical Embodiment owner segment when root-intent reproduction requires it.
Optional `AnimationPresentationStateV1` is a separate non-authoritative segment
and may be discarded on incompatibility. A failed migration/load preserves the
original save generation and activates no partial subject/graph state.

Replay validates every root-motion proposal/receipt/outcome and exact intent
cursor. Presentation pose, retarget, IK and LOD roots are separate observable
projections; their mismatch fails the corresponding ANIM/capture gate but
cannot be patched back into gameplay state. First divergence reports subject,
logical animation/physics tick, descriptor/profile hash, graph stage, node or
chain ID and first differing canonical field.

## Stable diagnostics and failure semantics

| Code | Required result |
|---|---|
| `ANIM_SKELETON_INVALID` | Reject the complete skeleton/runtime descriptor before subject activation; retain the prior exact descriptor. |
| `ANIM_CLIP_INVALID` | Reject the complete clip and its root-intent curve; optional presentation may use only the declared bind/held fallback. |
| `ANIM_GRAPH_INVALID` | Reject graph activation on cycle, ambiguous transition/order, cross-authority edge or invalid node; publish no partial graph state. |
| `ANIM_RESOURCE_UNAVAILABLE` | Deterministically defer required staged activation or use the exact optional presentation fallback; never skip due intent work. |
| `ANIM_RETARGET_INCOMPATIBLE` | Reject the complete profile/result; never guess a joint mapping or silently omit a required physical reference. |
| `ANIM_ROOT_MOTION_REJECTED` | Reject stale, conflicting, unsupported or out-of-bounds intent before physical mutation; retain prior pose/intent revision. |
| `ANIM_PHYSICAL_IK_INVALID` | Reject the complete physical constraint set before motor actuation; use declared safe motor/controller behavior. |
| `ANIM_PRESENTATION_IK_ESCAPE` | Disable/quarantine the presentation request/rig and preserve all command, physics and gameplay state. |
| `ANIM_LOD_INVALID` | Pin the declared safe evaluation level or block activation; do not skip required intent or alter physical/residency tier. |
| `ANIM_PUBLICATION_ABORTED` | Discard staging/working snapshot and retain the complete prior generation; expose no partial joint palette or graph state. |
| `NONDETERMINISTIC_RESULT` | Fail at the first graph/intent/retarget/IK/LOD divergence; preserve minimized replay and never retry to green. |

## Deterministic and observable evidence rules

- Canonical skeleton/clip/graph/profile bytes and fixed-point pose/proposal
  roots are exact on Windows x86_64 and Linux x86_64. Final raster pixels are
  exact only under the pinned capture-worker profile declared by SPEC-15.
- Backend-local skinning matrices, decompressed float caches and raw solver
  samples MAY use a declared presentation tolerance but are not command,
  contact, physical outcome or gameplay-hash inputs.
- Permutation suites vary descriptor declaration, task completion, worker
  count, render cadence, cache warmth and presentation LOD while retaining the
  same closed inputs. Any command, receipt, event, physical outcome or gameplay
  root difference is `NONDETERMINISTIC_RESULT`.
- Scenarios drive production actions/commands and use immutable probes. Mutable
  graph/pose/physics test backdoors are forbidden.
- ImpactResolver classifies `animation`, `root_motion`, `retarget`, `ik` and
  `animation_lod` as observable. Required automatic gates, replay and boundary
  scans run before capture and hash-bound human review.
- Capture uses exact project/content/replay/graph/skeleton/retarget/IK/LOD,
  camera and toolchain hashes. It includes baseline/success/failure/comparison
  views appropriate to the change plus raw canonical pose/intent/physics roots.
- Missing GPU, encoder or reviewer is `AwaitingCapability`, never `PASS`.
  Automatic evidence cannot synthesize `HumanReviewDecisionV2::Approve`.

## Verification gates

Each row is the canonical `GateDescriptorV1` source for its gate.

All five gates are `AcceptedBaseline`, `Blocking` children of the listed
existing VS descriptors. Their unavailability is non-PASS and cannot be
reclassified as CandidateOnly.

| Gate | Descriptor ID / classification | Subject / applicability | Primary owner | Contributors | Reproducible command/scenario | Pass threshold | Required evidence | Fallback | Requirement IDs | Failure IDs | VS / profile closure |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `ANIM-GRAPH-P1` | `nextengine.animation.graph.v1`; AcceptedBaseline, Blocking | Every admitted v1 skeleton/clip/graph/numeric profile and composition root | Physical Embodiment Team | Asset & Persistence Team, Rendering Team, Runtime Team | `next gate ANIM-GRAPH-P1 --scenario animation-graph-ordering-v1 --graphs 1000 --permutations all --roots game,headless,capture-worker` | 1,000 valid graph/state/parameter/worker/declaration/render-cadence permutations produce exact transition, cursor, marker, node, intent-proposal and canonical pose roots across all roots and shipping targets; every Clamp/Loop/PingPong zero/endpoint/multi-segment boundary vector is exact; 100% cycle, duplicate, cross-authority, invalid-transition, overflow and partial-publication cases reject with the exact diagnostic; public scan finds 0 forbidden type | Skeleton/clip/graph/profile manifests, canonical graph/cursor/marker/pose/proposal roots, ordering/task-merge traces, boundary/negative corpus, composition-root replay diff, publication and public-API scans | Reject candidate graph and retain the prior exact graph; optional presentation uses the locked bind-pose graph, required intent graph blocks activation | REQ-136, REQ-138, REQ-139 | FAIL-056, FAIL-057 | VS-05, VS-07, VS-15 |
| `ANIM-ROOT-MOTION-P1` | `nextengine.animation.root-motion.v1`; AcceptedBaseline, Blocking | Every graph/clip capable of root intent and every accepted/rejected root-motion proposal | Physical Embodiment Team | Runtime Team, Gameplay Extensibility Team, Verification & Evidence Team | `next gate ANIM-ROOT-MOTION-P1 --scenario root-motion-command-boundary-v1 --cycles 10000 --faults all` | 10,000 accepted/rejected/retried/save/replay/LOD cycles route 100% root deltas through canonical proposal, command admission, receipt, motor/safety and physics outcome; every stale, duplicate, collision, bound, capability, body-revision and contact fault yields 0 direct/partial transform mutation; command/receipt/outcome/gameplay roots are exact across roots/targets | Root-intent curves/states/proposals, command bodies/receipts/events, motor/action/contact/physics traces, save/replay roots, fault matrix and direct-write boundary scan | Reject intent, retain prior physical state and use the declared validated locomotion/recovery controller; never teleport | REQ-137, REQ-138, REQ-139 | FAIL-056, FAIL-057 | VS-05, VS-11, VS-14 |
| `ANIM-RETARGET-P1` | `nextengine.animation.retarget.v1`; AcceptedBaseline, Blocking | Every admitted source/target skeleton pair and retarget profile | Physical Embodiment Team | Asset & Persistence Team, Rendering Team | `next gate ANIM-RETARGET-P1 --scenario skeletal-retarget-matrix-v1 --profiles all --permutations 10000` | Every admitted source/target skeleton and declared optional/required-joint case produces exact mapping/local-pose or exact rejection under 10,000 rule/order/worker permutations; 100% signature, cycle, duplicate-target, missing-required, bounds and heuristic-mapping cases reject; presentation profile variation changes 0 command/gameplay roots | Source/target skeletons, retarget profiles/rule graphs, canonical mapping/pose roots, compatibility/negative matrix, replay/gameplay hashes and comparison captures | Use only an exact compatible authored profile; otherwise retain source/bind pose for optional presentation or block required physical reference | REQ-136, REQ-138, REQ-139 | FAIL-056, FAIL-057 | VS-05, VS-07, VS-15 |
| `ANIM-IK-P1` | `nextengine.animation.ik.v1`; AcceptedBaseline, Blocking | Every admitted physical/presentation IK rig, chain and request class | Physical Embodiment Team | Rendering Team, Runtime Team, Verification & Evidence Team | `next gate ANIM-IK-P1 --scenario physical-presentation-ik-separation-v1 --requests 10000 --faults all` | 10,000 chain/order/target/LOD permutations across every V1 algorithm execute the exact fixed iteration/order and produce exact constraint/presentation-pose roots; 100% cross-class edge, invalid target, stale revision, limit, overflow and presentation-write escape cases reject; presentation IK enable/disable changes 0 command/contact/physical outcome/gameplay roots and physical IK never bypasses motor/safety | IK rig/request/constraint/delta manifests, algorithm golden vectors, iteration/order traces, motor/action/contact/state roots, escape/negative corpus, replay hashes and required captures | Reject the complete physical constraint set and use safe motor behavior; independently disable optional presentation IK | REQ-137, REQ-138, REQ-139 | FAIL-056, FAIL-057 | VS-05, VS-07, VS-14, VS-15 |
| `ANIM-LOD-P1` | `nextengine.animation.lod.v1`; AcceptedBaseline, Blocking | Every admitted animation budget/LOD profile, transition and pinned capture profile | Physical Embodiment Team | Rendering Team, Runtime Team, Verification & Evidence Team | `next gate ANIM-LOD-P1 --scenario animation-lod-parity-v1 --transitions 10000 --presentation-permutations all` | 10,000 LOD/cadence/resource/fault transitions publish one complete snapshot or none, never skip due physical-intent work and produce exact root-motion/command/physical/gameplay roots; camera, frustum, render cost, cache and presentation-level permutations change 0 authoritative roots; every pinned capture profile produces its exact pose/media roots | LOD/profile/transition manifests, due-work and resource traces, intent/command/physics/gameplay roots, publication fault matrix, pinned pose/capture roots and comparison media | Pin the safe required evaluation level; use declared held/bind/cull presentation fallback and block when required intent work is unavailable | REQ-138, REQ-139 | FAIL-056, FAIL-057 | VS-05, VS-07, VS-12, VS-15 |

## Requirements

| ID | Нормативное требование | Primary owner | Contributors | Blocking gates | Required evidence | Fallback / fail-closed outcome | VS / profile closure |
|---|---|---|---|---|---|---|---|
| REQ-136 | Every admitted skeleton, clip, animation graph and retarget profile MUST use the bounded engine-owned contracts above, exact content/schema hashes and canonical fixed evaluation/mapping order, with no vendor, backend, ECS, OS or importer public type. | Physical Embodiment Team | Asset & Persistence Team, Rendering Team | ANIM-GRAPH-P1, ANIM-RETARGET-P1 | Skeleton/clip/graph/retarget manifests, canonical bytes/roots, ordering and compatibility corpora, publication/public-API scans. | Reject the incompatible candidate and retain the prior exact graph/profile or declared optional bind-pose fallback. | VS-05, VS-07, VS-15 |
| REQ-137 | Root motion MUST remain a bounded revision-checked intent passing the production `WorldCommandEnvelopeV2`/motor/safety/physics path, while physical IK and presentation IK MUST remain separate one-way authority classes with no direct pose, contact or gameplay mutation. | Physical Embodiment Team | Runtime Team, Gameplay Extensibility Team, Rendering Team | ANIM-ROOT-MOTION-P1, ANIM-IK-P1 | Root-intent/command/receipt/outcome and IK rig/request/constraint/delta corpora, motor/contact/state roots and direct-write/escape scans. | Reject the entire stale/invalid intent or physical constraint set; retain physical state, use safe controller and independently disable optional presentation IK. | VS-05, VS-07, VS-11, VS-14, VS-15 |
| REQ-138 | Fixed graph stages, retarget/IK total orders, logical animation time, deterministic result merge and animation LOD MUST produce exact proposal/physical roots across `game`, `headless` and `capture-worker`; presentation variation MUST change zero gameplay hashes. | Physical Embodiment Team | Runtime Team, Rendering Team, Verification & Evidence Team | ANIM-GRAPH-P1, ANIM-ROOT-MOTION-P1, ANIM-RETARGET-P1, ANIM-IK-P1, ANIM-LOD-P1 | Cross-root/target graph, proposal, command, retarget, IK, LOD, physics and gameplay roots plus worker/order/cadence/resource permutation traces. | Fail at first divergence, preserve minimized replay and pin/reject the candidate profile; retry cannot turn green. | VS-05, VS-07, VS-11, VS-12, VS-14, VS-15 |
| REQ-139 | Every observable animation, root-motion, retarget, IK or animation-LOD change MUST resolve all affected automatic gates and produce hash-bound SPEC-15 capture/evidence under one exact profile before authorized human approval. | Verification & Evidence Team | Physical Embodiment Team, Rendering Team, Security & Governance Team | ANIM-GRAPH-P1, ANIM-ROOT-MOTION-P1, ANIM-RETARGET-P1, ANIM-IK-P1, ANIM-LOD-P1 | ChangeImpactManifest, automatic RunManifests, replay/pose/physics roots, CaptureJob/result, baseline/comparison media, EvidenceBundle and V2 review decision. | Remain `AwaitingCapability` or `HumanReviewRequired`; retain the prior approved baseline and deny admission without automatic PASS plus authorized Approve. | VS-05, VS-07, VS-11, VS-12, VS-14, VS-15 |

## Failure paths

| ID | Trigger | Required result | Primary owner | Contributors | Blocking gates | Required evidence | Fallback / fail-closed outcome | VS / profile closure |
|---|---|---|---|---|---|---|---|---|
| FAIL-056 | Invalid, cyclic, ambiguous, oversized, incompatible or partially available skeleton/clip/graph/retarget/IK/LOD descriptor, missing required joint/resource or publication fault | Reject the complete candidate before subject/graph activation, discard staging and preserve the prior exact descriptor, state and content/save generation; never guess mapping/order or publish a partial pose/rig. | Physical Embodiment Team | Asset & Persistence Team, Rendering Team, Runtime Team | ANIM-GRAPH-P1, ANIM-ROOT-MOTION-P1, ANIM-RETARGET-P1, ANIM-IK-P1, ANIM-LOD-P1 | Descriptor/profile negative corpus, compatibility/order roots, resource/publication fault matrix, preserved generation hashes and public-boundary scan. | Retain the prior exact revision or declared optional bind/held/cull presentation fallback; required intent/reference activation remains blocked. | VS-05, VS-07, VS-11, VS-12, VS-14, VS-15 |
| FAIL-057 | Stale/conflicting/out-of-bounds root intent, nondeterministic graph/retarget/IK/LOD result, presentation-to-physical authority escape or missing observable evidence/reviewer capability | Reject the complete intent/constraint/result before physical or gameplay mutation, preserve prior command/pose/state roots, quarantine the escaping presentation path and report first divergence; no retry, visual fallback or human decision may waive an automatic failure. | Physical Embodiment Team | Runtime Team, Rendering Team, Verification & Evidence Team, Security & Governance Team | ANIM-GRAPH-P1, ANIM-ROOT-MOTION-P1, ANIM-RETARGET-P1, ANIM-IK-P1, ANIM-LOD-P1 | Intent/command/receipt/constraint/physics roots, divergence replay, authority-escape corpus, unchanged gameplay proof, EvidenceBundle/capability and V2 admission report. | Use the safe validated controller and independent presentation fallback; remain `AwaitingCapability`/`HumanReviewRequired` and retain the prior approved baseline. | VS-05, VS-07, VS-11, VS-12, VS-14, VS-15 |

## Technology neutrality

This contract selects no animation graph runtime, skeletal-animation library,
IK solver, retargeting package, renderer, skinning implementation, ECS,
physics backend, job system, importer, shader compiler or platform API.
Replaceable implementations remain private adapters/caches behind these
engine-owned schemas, one-way authority boundaries, deterministic schedules,
fallbacks and exact gates.
