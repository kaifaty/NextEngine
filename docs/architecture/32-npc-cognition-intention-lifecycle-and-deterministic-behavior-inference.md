# SPEC-32: NPC cognition, intention lifecycle and deterministic behavior inference

| Поле | Значение |
|---|---|
| ID | SPEC-32 |
| Статус | Proposed |
| Lifecycle | Consumer-driven R4 proposal |
| Версия | 0.1 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-27](27-motor-observation-action-and-deterministic-inference.md), [SPEC-33](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-050](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md) |
| Заменяет | отсутствует |

## Статус и scope

Документ предлагает future public/runtime contract и не добавляет Rust types в
этом changeset. Он не меняет Accepted semantics SPEC-06, SPEC-08, SPEC-13,
SPEC-19, SPEC-20 или SPEC-21. Названия `*V1` ниже являются design target для
первого production R4 consumer, а не текущей schema registry.

До совместной promotion с ADR-050/SPEC-33:

- current Agent Runtime использует authored utility/HTN и tactical fallback;
- current save/replay не обязан содержать перечисленные behavior records;
- proposed checks имеют результат `NOT_RUN(NO_PRODUCTION_CONSUMER)`;
- model runtime, bundles и GPU route не являются shipped capability.

## Цель и invariants

SPEC-32 задаёт границу между derived drives, strategic/tactical learned policy,
Agent executive, intention state, existing subsystem owners и validated
gameplay execution.

- Strategic и tactical roles имеют независимые profiles, observations,
  candidates, recurrent state, cadence and fallback.
- Model выбирает только из canonical engine-built candidates и возвращает
  scores плюс bounded next recurrent state. Arbitrary goal, ID, waypoint,
  command payload или tool call из model output запрещены.
- Applied decision exact per seed: engine квантует scores и выполняет
  canonical selection через named SPEC-21 RNG. Evaluator не использует RNG.
- Every future-affecting active/suspended goal, emergency frame, tactical mode,
  recurrent state, decision reference and RNG state is authoritative under its
  owner and participates in save/replay after promotion.
- Learned/fallback decision проходит один executive and validator path. Model
  не получает hidden first-party mutation capability.
- GPU/provider/evaluator types, ECS storage, task handles, raw pointers,
  filesystem paths, model sessions and vendor tensor objects не входят в
  public contract.
- Wall time and completion order cannot select an authoritative decision.
- LLM/audio output остаётся optional untrusted candidate and never blocks a
  behavior boundary.

## Authority and derived views

| State/fact | Owner/source of truth | Разрешённая behavior projection |
|---|---|---|
| Character resources, skills, faction, relationships, quest/dialogue | RPG Framework / Mechanics per SPEC-19/13 | immutable revision-bound facts only |
| Perception, uncertainty, memory records and relationship recollections | Agent Perception / Memory Service per SPEC-06 | bounded filtered facts with stable IDs/revisions |
| Calendar, schedules, population tier and logical location | Future World Services owner per SPEC-20 promotion | immutable schedule/tier/activity view; no duplicate values |
| Navigation topology, query and `RoutePlan` | World Services per SPEC-08 | canonical navigation candidates/query profiles |
| Planner-visible abilities | Mechanics Runtime per SPEC-13 | capability-filtered `MechanicAffordance` candidates |
| Motor availability | Motor Runtime per SPEC-14/27 | immutable `MotorCapabilityView`; no joint/action state |
| Goals, emergency frame, tactical mode and behavior recurrent state | Agent Runtime after promotion | `AgentIntentionStateV1` and `BehaviorPolicyStateRecordV1` |
| Model bytes, schemas and immutable profiles | Assets / Project Composition | exact content hashes; no runtime mutable weights |
| Gameplay mutation | owning domain after validated `WorldCommand` | `AgentIntent`/`InvokeAbility` proposal only |

### `AgentDriveViewV1`

`AgentDriveViewV1` is a derived read-only Agent projection. It is not a second
mutable needs database and MUST be reproducible from exact source revisions.

```text
AgentDriveViewV1 {
  schema_version: 1,
  subject: PersistentId,
  decision_tick: SimulationTick,
  source_revisions: CanonicalMap<OwnerId, u64>,
  fatigue: DriveValueV1,
  morale: DriveValueV1,
  threat: DriveValueV1,
  social_contact: DriveValueV1,
  derived_view_hash: Hash256,
}

DriveValueV1 {
  drive_id: NamespacedId,
  value_raw: i32,
  fixed_point_descriptor_id: NamespacedId,
  evidence_fact_ids: CanonicalSet<FactId>,
  validity: Current | Unavailable,
}
```

Source mapping:

- fatigue and morale derive from current RPG/Mechanics facts;
- threat derives from current PerceptionFrame and bounded Memory facts;
- social-contact derives from calendar/schedule, memory and relationship views.

Missing source is `Unavailable` with an explicit candidate mask/fallback rule;
it is never implicit zero. Project MAY add another drive only through a new
versioned schema/profile naming its owner, exact derivation and bounds. Policy
output cannot write a drive value.

## Common behavior inference model

### Role, cadence and subject key

`BehaviorPolicyRoleV1` is exactly `Strategic` or `Tactical`. Canonical row key:

```text
(decision_tick, subject PersistentId, policy_role, policy_id)
```

For one subject/role/boundary exactly one active policy route exists.
Duplicate/conflicting routes reject learned work; insertion or discovery order
cannot select one.

Strategic work is due only in `Simulated` and `Active` according to immutable
integer cadence and deterministic phase. Tactical work is due only in `Active`
and when the executive declares a tactical boundary. `Abstract`/`Dormant`
generate no behavior-inference row.

`BehaviorDecisionCadenceV1` declares integer period, deterministic subject
phase derivation, maximum logical deferral, starvation limit and eligible tier
set. Deferral is a recorded logical scheduler decision ordered by stable key;
measured load does not decide it.

### Candidate envelope

Every candidate has a common engine-owned prefix:

```text
BehaviorCandidateV1 {
  candidate_id: NamespacedId,
  candidate_kind: closed role-specific enum,
  target: Option<StableBehaviorTargetV1>,
  source_owner: OwnerId,
  source_revision: u64,
  capability_hash: Hash256,
  precondition_hash: Hash256,
  payload_schema_hash: Hash256,
  payload: CanonicalBinary,
}
```

`StableBehaviorTargetV1` may contain a `PersistentId`, `AssetId`, stable region,
activity, affordance, navigation-query-profile or speech-act ID. It cannot
contain `RuntimeEntityId`, raw nav poly/waypoint, ECS row, backend handle or
free-form text identity.

Engine candidate builders:

1. read one immutable revision-bound source closure;
2. apply hard capabilities, tier, quest/safety and schema filters;
3. construct all eligible bounded candidates;
4. reject duplicate canonical candidate IDs or conflicting bytes;
5. sort by `(candidate_kind tag, candidate_id canonical bytes, target canonical
   bytes, source_owner, source_revision, payload hash)`;
6. create a fixed-width mask and exact candidate-set hash;
7. truncate only by a declared deterministic priority rule before hashing;
   silent tail drop or evaluator-specific padding row is forbidden.

Model sees the exact sorted rows/mask and returns one score for every row.
It cannot add, remove, reorder or rename a candidate.

### Score quantization and sampling

`BehaviorScoreQuantizationProfileV1` declares source dtype, exact finite
IEEE-754 decode, rational scale/offset, signed integer descriptor, bounds,
round-to-nearest-ties-to-even and invalid-value policy. V1 raw score proposal
is binary32; negative zero canonicalizes to positive zero. NaN, infinity,
overflow, wrong dtype/shape/length or score for a masked candidate rejects the
entire subject result.

For each valid row engine converts score once to canonical `i32 score_raw`.
The sampling profile then performs exactly one declared method:

- `ArgMaxStable`: maximum `score_raw`, tie by canonical candidate order; or
- `WeightedQ0_32`: engine transforms declared bounded integer logits/weights
  with a versioned integer-only lookup/profile and samples using
  `uniform_below_u32`/`sample_q0_32` from one named SPEC-21 stream.

No host `exp`, model-runtime sampler or floating distribution is authoritative.
Candidate-set hash, quantized score vector, chosen candidate and pre/post RNG
state are included in the decision commit. A project may choose different
named profiles for strategic and tactical roles, but profile selection is
locked before world creation.

### `BehaviorInferenceProfileV1`

```text
BehaviorInferenceProfileV1 {
  schema_version: 1,
  profile_id: NamespacedId,
  policy_role: Strategic | Tactical,
  policy_id: NamespacedId,
  policy_bundle_hash: Hash256,
  model_hash: Hash256,
  observation_schema_hash: Hash256,
  output_schema_hash: Hash256,
  policy_state_schema_hash: Hash256,
  cadence: BehaviorDecisionCadenceV1,
  maximum_candidate_count: NonZeroU16,
  score_quantization_profile_hash: Hash256,
  sampling_profile_hash: Hash256,
  rng_stream_descriptor_hash: Hash256,
  required_evaluator_capabilities: CanonicalSet<BehaviorEvaluatorCapabilityV1>,
  resource_envelope: BehaviorResourceEnvelopeV1,
  fallback_profile_id: NamespacedId,
}
```

Capabilities are vendor-neutral closed operation/numeric/resource limits.
Profile contains no provider, device name, CUDA/DirectML type, runtime session,
path or environment variable. Every hash resolves in exact project/content
closure before activation. Runtime does not discover `latest` model or mutate
weights.

## Strategic contracts

### `StrategicBehaviorObservationV1`

Observation binds:

- subject identity, decision tick, tier, cadence phase and active profile;
- immutable `AgentArchetypeDefinition` revision, personality traits and
  relevant skill/motor-capability views;
- `AgentDriveViewV1` and its source revisions;
- current authored schedule/activity and bounded upcoming schedule facts;
- bounded perception/memory/relationship facts ordered by stable relevance
  key then fact ID;
- active goal, bounded suspended goal stack and emergency summary;
- canonical available strategic activities;
- valid `GoalSuggestionCandidateV1` values admitted before this boundary;
- all source/candidate/schema/profile/state hashes.

The observation cannot contain free-form prompt history, vendor embedding,
raw dialogue text as action identity, hidden quest state, presentation state or
mutable RPG/world reference.

### `StrategicGoalProposalV1`

Applied proposal selects exactly one candidate and projects:

```text
StrategicGoalProposalV1 {
  proposal_id: Id128,
  subject: PersistentId,
  decision_tick: SimulationTick,
  candidate_set_hash: Hash256,
  selected_candidate_id: NamespacedId,
  goal_kind: RoutineActivity | Sleep | Rest | SocialContact |
             StartConversation | SeekHelp | TravelToActivity |
             Wait | ResumeSuspended | CancelCurrent,
  target: Option<StableBehaviorTargetV1>,
  source: Authored | Learned | LlmSuggestion | PlannerFallback,
  priority_band: HardConstraint | Routine | Survival | TacticalEfficiency,
  expires_at_tick: SimulationTick,
  interruption_policy: NonInterruptibleByBehavior |
                       SuspendForEmergency | ReplaceAtStrategicBoundary,
  cited_fact_revisions: CanonicalSet<(FactId, u64)>,
}
```

Goal kind is closed. Content-specific activity lives behind stable activity ID
and preconditions, not a new arbitrary model output kind. `LlmSuggestion`
denotes provenance of an already validated engine candidate; it gives no
authority advantage.

Strategic fatigue may select `Sleep` outside authored schedule. Executive MUST
revalidate current safety, available compatible sleep place/reservation,
navigation capability, hard quest constraints and expiry. Failure rejects the
proposal and applies declared `Rest`/`Wait`/schedule fallback; it does not
teleport NPC or invent a bed.

## Tactical contracts

### `TacticalBehaviorObservationV1`

Observation binds:

- current bounded emergency frame and tactical mode;
- capability-filtered perception facts and uncertainty;
- canonical navigation candidates/query profiles from World Services;
- planner-visible `MechanicAffordance` candidates from Mechanics Runtime;
- current `MotorCapabilityView` values, without motor tensors/actions;
- current support/allies, threats, yielding/help/speech facts;
- bounded strategic context: goal kind/target/priority and suspension state;
- hard safety/quest constraints and candidate masks;
- candidate/profile/recurrent/source hashes.

Hidden exact enemy state, renderer visibility, raw physics/contact backend,
raw nav waypoints and arbitrary LLM text are forbidden.

### `TacticalDecisionProposalV1`

The selected candidate kind is exactly:

```text
UseAffordance
RequestNavigation
Fight
Flee
Yield
CallForHelp
RequestConversation
ResolveEmergency
HoldSafe
```

`UseAffordance` references one canonical planner-visible affordance.
`RequestNavigation` references a stable goal plus `NavigationQueryProfileId`;
World Services constructs `RoutePlan`. `Fight`/`Flee` are bounded modes that
still decompose to navigation/affordance candidates. `Yield` changes only
Agent state. `RequestConversation` creates a speech-act/dialogue request, not a
committed RPG dialogue transition. `ResolveEmergency` references the current
emergency frame and a declared resolution code.

Proposal includes decision/candidate/profile IDs, target, cited fact revisions,
expiry and no arbitrary payload. It becomes executable only after executive
and common capability/domain validation.

## Intention and emergency lifecycle

### `AgentIntentionStateV1`

```text
AgentIntentionStateV1 {
  schema_version: 1,
  subject: PersistentId,
  revision: u64,
  active_strategic_goal: Option<CommittedStrategicGoalV1>,
  suspended_goals: Vec<CommittedStrategicGoalV1>,
  emergency_frame: Option<EmergencyFrameV1>,
  tactical_mode: Idle | Navigating | Fighting | Fleeing | Yielding |
                 CallingForHelp | Conversing | Recovering,
  last_strategic_decision_ref: Option<BehaviorDecisionRefV1>,
  last_tactical_decision_ref: Option<BehaviorDecisionRefV1>,
  causal_command_refs: CanonicalSet<CommandId>,
  authoritative_state_hash: Hash256,
}
```

Suspended stack maximum is profile-bound and at least one; v1 proposal default
is four. Push, pop, replace and cancellation are checked integer operations and
publish one new revision atomically. Goal cycles, duplicate active/suspended
identity, overflow, stale fact revision or invalid causal reference reject the
whole intention transition.

### `EmergencyFrameV1`

Frame contains stable frame ID, trigger fact IDs/revisions, threat/support
summary, opening tick, expiry/max duration, suspended goal reference, allowed
tactical modes, resolution condition set and revision. It cannot store a copy
of health, relationship, navigation route or quest state.

Lifecycle:

```text
None
  → Open(trigger validated, strategic goal suspended at most once)
  → Active(tactical decisions update mode/frame revision)
  → Resolved | Expired | SupersededByHostileAction
  → RevalidateSuspendedGoal
  → Resume | CancelAndReplan | SafeFallback
```

Nested trigger updates the current frame or replaces it by a canonical stronger
frame according to profile; it does not push the strategic goal again.
Emergency expiration without a safe resolution invokes tactical fallback and
revalidation; it does not silently restore a stale goal.

### Yielding semantics

Entering `Yielding` requires one valid tactical decision commit. Perception
publishes a revision-bound `YieldingFactV1` with subject, start/expiry, cited
decision and confidence/source. It is observable to other actors according to
normal perception, not globally broadcast hidden state.

`Yielding`:

- provides no invulnerability, damage filter, Mechanics status or RPG field;
- does not force another actor to stop attacking;
- may lead content to propose Dialogue/Relationship operations separately;
- ends on committed negotiation/dialogue resolution, no remaining relevant
  threat, expiry/fallback, or a new hostile action by the yielding subject.

After exit, executive revalidates the suspended strategic goal as for any
emergency resolution.

## Recurrent state and atomic decision publication

### `BehaviorPolicyStateRecordV1`

Strategic and tactical state records are separate even if model architecture
or width coincides.

```text
BehaviorPolicyStateRecordV1 {
  schema_version: 1,
  subject: PersistentId,
  policy_role: Strategic | Tactical,
  policy_id: NamespacedId,
  bundle_hash: Hash256,
  observation_schema_hash: Hash256,
  output_schema_hash: Hash256,
  state_schema_hash: Hash256,
  route_generation: u64,
  state_generation: u64,
  state_tick: SimulationTick,
  values_raw: Vec<i32>,
  last_candidate_set_hash: Hash256,
  last_quantized_score_root: Hash256,
  last_applied_decision_hash: Hash256,
  consecutive_learned_unavailable: u16,
  fallback_phase: Learned | PlannerFallback,
  authoritative_state_hash: Hash256,
}
```

State schema declares exact width, fixed-point descriptors, initial values,
input normalization and output conversion. `S = 0` still has a record. Raw
evaluator floats, sessions and caches are not authoritative. Non-finite,
out-of-range, missing or partial next state rejects the complete decision/state
pair. Strategic state cannot be copied into tactical state or vice versa.

`authoritative_state_hash` covers every preceding future-decision-affecting
field except itself via registered `CanonicalBinaryV1` and domain-separated
SHA-256. Consumer recomputes and exact-compares it before inference, save/load,
replay restore or field use.

### `BehaviorDecisionCommitV1`

```text
BehaviorDecisionCommitV1 {
  key: (decision_tick, PersistentId, BehaviorPolicyRoleV1, policy_id),
  observation_hash: Hash256,
  candidate_set_hash: Hash256,
  quantized_score_root: Hash256,
  selected_candidate_id: NamespacedId,
  applied_decision_hash: Hash256,
  prior_policy_state_hash: Hash256,
  next_policy_state_hash: Hash256,
  prior_intention_state_hash: Hash256,
  next_intention_state_hash: Hash256,
  rng_stream_id: RngStreamId,
  prior_rng_state_hash: Hash256,
  next_rng_state_hash: Hash256,
  source: Learned | PlannerFallback,
}
```

Applied decision, next role-specific policy state, complete next intention state
and consumed RNG state become visible atomically, or none does. Fallback uses
the same commit form and validators. A decision without corresponding state/
RNG/intention link, or a partially published recurrent state, is invalid.

Strategic and tactical decisions never share one atomic commit because they
have separate due boundaries. Tactical commit may open/update/resolve emergency
state; the following strategic boundary sees only the committed result.

## Inference requests, results and failure semantics

Runtime closes due work by canonical row key, partitions by exact profile hash
and fixed maximum batch size, and binds every request to:

- observation/candidate-set/profile hashes;
- route/state generations and prior full state hash;
- current intention-state hash;
- named RNG stream identity/state hash;
- all cited owner revisions.

Result echoes exact batch hash, row keys, profile and state bindings, one finite
score per candidate and exact next-state shape. Before any row commits, whole
envelope, key set and shapes validate. Duplicate, extra, missing, stale,
reordered, partially decoded or incompatible result is rejected for the
complete affected subject; no neighboring row or previous score is substituted.

Logical failure routes:

| Failure | Required result |
|---|---|
| Preflight evaluator/model/profile unavailable | Select declared deterministic utility/HTN profile before affected boundary; record fallback commit. |
| Canonical evaluator failure result | Reject learned result and apply declared fallback for that exact boundary. |
| Non-finite/overflow/wrong shape/candidate order/state mismatch | Reject whole subject learned pair; apply declared fallback. |
| Stale/late result before a current boundary | Reject by revision/hash; current boundary uses its declared fallback. |
| Late result after commit | Discard; changes no state/RNG/decision. |
| Fallback input/validator unavailable | Preserve prior authoritative state, emit stable diagnostic and stop the affected conformant commit; never fabricate a decision. |
| Wall deadline/watchdog miss | Mark exact run nonconforming/`Fail`; wall time does not select fallback or another candidate. Watchdog may stop uncommitted work. |

Fallback cannot bypass constraints, affordance/navigation validation or
`AgentIntent → WorldCommand` path. When GPU is absent at activation, planner
route is the declared operational profile, not a wall-time failure.

## Navigation, mechanics and physical execution

- Strategic/tactical candidates may choose navigation goal and query profile;
  only World Services creates and owns `RoutePlan`.
- Route progress, invalidation and stuck resolution return revision-bound facts;
  policy cannot claim traversal success.
- Tactical affordance selection references one candidate from the current
  `MechanicAffordance` catalog. Mechanics Runtime revalidates target, cost,
  cooldown, phase, resources and actual outcome.
- Motor capability masks candidate feasibility. `PendingActivation` cannot
  produce joint action; executive waits, replans or chooses fallback.
- Physical layer receives only validated `PhysicalAvatarIntent`; contact and
  hit success are proven by physics and committed through normal outcome
  commands.

## LLM, speech-act and prosody integration

One engine-owned `SpeechActCandidateV1` supports player↔NPC and NPC↔NPC. It
contains speaker/addressee IDs, closed act kind, optional stable topic/target,
cited facts, expiry, dialogue/session preconditions and text provenance. The
semantic act is validated independently of generated wording.

`GoalSuggestionCandidateV1` is admitted to strategic candidate construction
only on the next declared strategic boundary after ai-host result assignment.
It contains a closed goal kind/target, cited fact revisions, expiry and
provenance; it cannot carry an arbitrary WorldCommand or priority above hard
constraints. Late, invalid or unsupported suggestion is discarded and an
authored strategic set remains complete.

`ProsodyAnnotationCandidateV1` contains only closed emotion/prosody class,
bounded intensity/confidence, source utterance/turn identity, pack/model hash
and expiry. Perception validator checks provenance, bounds, participant and
turn freshness and either publishes an uncertainty-tagged perception fact or
rejects it. Behavior observation consumes only that fact; raw waveform,
embedding or model tensor is forbidden.

Speech generation, ASR and TTS may complete late without delaying strategic or
tactical work. Authored line/template/subtitle and `TextOnlyFallback` remain
complete.

## Persistence and replay after promotion

The Agent owner segment MUST atomically persist:

- complete `AgentIntentionStateV1` for every relevant NPC;
- separate strategic/tactical `BehaviorPolicyStateRecordV1` records;
- route/state generations and exact profile/bundle/schema hashes;
- pending admitted LLM goal/speech candidates only if they can affect a future
  declared boundary;
- scheduler cadence/deferral state and named behavior RNG states;
- last atomic `BehaviorDecisionCommitV1` chain needed to prove continuity.

Save/load/save recomputes all full-record hashes and preserves exact bytes.
Unknown/missing model or schema, identity mismatch, corrupt intention stack,
broken prior→next chain or incompatible project closure rejects load before
world mutation and preserves source save. Silent model downgrade is forbidden;
an explicitly project-allowed planner fallback load path requires its own
recorded route transition and replay incompatibility result.

Replay does not resample an unrecorded model route. It re-executes the exact
closed observations/candidates/model or declared logical fault, consumes the
same named RNG state, and compares quantized scores, selected candidate,
intention/recurrent/RNG commits, resulting commands/events and state roots.
First mismatch is `NONDETERMINISTIC_RESULT`; retry cannot regenerate a passing
decision.

## Budget and 100-NPC behavior

Strategic/tactical work is charged to the single ADR-016 `agent-planning` row,
not a per-NPC allowance. Candidate construction, inference staging, result
validation, fallback, queue handling and decision commit all count.

R4 workload declares exact NPC membership/tier, strategic/tactical periods,
phase, eligible emergency boundaries, maximum deferral and starvation age.
Only `Simulated`/`Active` receive strategic work and only `Active` receives
tactical work. Deterministic cadence reduction may defer eligible work within
its manifest bound; it cannot drop mandatory emergency resolution or choose a
different applied decision from measured runtime load.

## Proposed ProductCheck

All checks below are `NOT_RUN(NO_PRODUCTION_CONSUMER)` in this docs-only
changeset and do not change current global ProductCheck mapping.

| ID | Scenario | Required result / fallback |
|---|---|---|
| `BEHAVIOR-SCHEMA-P1` | Strategic/tactical schema corpora with every candidate kind, ordering/mask/quantization boundary and malformed/stale/reordered/missing/extra/non-finite result | Exact observation/candidate/score roots; every invalid input rejects the whole affected decision/state pair before publication; declared planner fallback only. |
| `BEHAVIOR-DETERMINISM-P1` | Same seeds, inputs and exact bundles on Windows/Linux with worker, request and completion permutations | Exact strategic/tactical applied decisions, intention/recurrent/RNG commits, command/event/state roots; wall miss is nonconforming and never selects a different decision. |
| `BEHAVIOR-STATE-P1` | Evaluate → emergency suspend → save/load/replay → resolve → resume/cancel for stateless/stateful policies | Exact goals, bounded stack, emergency/tactical mode, both recurrent states, RNG continuation and commit chains; every tamper fails before field use. |
| `BEHAVIOR-FALLBACK-P1` | GPU/model absent at preflight plus injected logical evaluator/model/schema faults | Utility/HTN path executes the same mandatory routine/combat/dialogue loop without tick stall or direct mutation; invalid fallback input stops instead of fabricating. |
| `BEHAVIOR-R4-P1` | One production `routine → fatigue/sleep → threat → fight/flee/yield → dialogue → resume/replan` vertical | Both learned policies are observably applied in one run, all effects pass AgentIntent/affordance/WorldCommand owners, and deterministic fallback completes the same mandatory loop. |
| `BEHAVIOR-100NPC-P1` | Exact 100-NPC R4 workload across tiers/cadence/deferrals | Strategic only for Simulated/Active, tactical only Active; queue/cadence exact, maximum deferral bounded, no starvation/drop/unowned span, ADR-016 row met or stage remains open. |
| `BEHAVIOR-COMMS-P1` | Player↔NPC and NPC↔NPC speech acts; late/invalid LLM, ASR/TTS/prosody and ai-host absence | One protocol, authored fallback, validated uncertainty facts, no direct goal/mutation and no behavior tick wait. |

`BEHAVIOR-TRAIN-P1` belongs to SPEC-33 and is a joint promotion prerequisite.

## Promotion boundary

SPEC-32 may become `Accepted` only together with ADR-050/SPEC-33 and the
minimal public Rust schemas actually consumed by one production R4 vertical.
That promotion updates reciprocal Accepted owners, save/replay schema and
mapped checks. A test-only model call, type scaffold or isolated benchmark is
not a consumer under ADR-046.
