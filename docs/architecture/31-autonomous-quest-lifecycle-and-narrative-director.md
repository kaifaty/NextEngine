# SPEC-31: Autonomous quest lifecycle и narrative director

| Поле | Значение |
|---|---|
| ID | SPEC-31 |
| Статус | Proposed |
| Версия | 0.2 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |

## Статус предложения

SPEC-31 остаётся Proposed и не изменяет текущий Accepted runtime. Документ не выбирает конкретную LLM, provider, model runtime, database или graph library.

## Назначение и invariants

SPEC-31 определяет автономный lifecycle доступных и принятых квестов, bounded causal continuation после любого исхода и optional asynchronous Narrative Director, который предлагает world-local quest-graph revisions.

- Живой квест MUST существовать как RPG-owned `QuestInstance` до принятия игроком. Принятие является typed state transition, а не созданием нового источника quest state.
- Намерение NPC, committed проблема мира, реплика в диалоге или LLM output сами по себе MUST NOT являться квестом. Квест начинается только после deterministic системного допуска `QuestCandidateV1` в RPG-owned `QuestInstance`.
- Диалог, вопрос «есть работа?», наблюдение и доска объявлений являются способами раскрытия уже допущенной возможности, а не отдельными путями создания или мутации квеста.
- RPG Framework MUST оставаться единственным владельцем quest state, variables, participants, engagement, autonomy policy, outcome, graph revision и causal history.
- World Services MAY поставлять committed time/population/schedule facts, но MUST NOT менять Quest aggregate или narrative graph напрямую.
- Agent Intelligence MAY строить `NarrativeDirectorCandidateV1`, но candidate остаётся untrusted proposal до полной deterministic validation и atomic `WorldCommand` commit.
- Main-story anchors, protected facts и ending reachability MUST оставаться authored. Generated graph MAY добавляться только через authored `NarrativeExtensionSlotV1`.
- Отсутствие, crash, timeout или incompatible `ai-host` MUST NOT блокировать simulation tick и MUST NOT менять mandatory outcome. `TemplateNarrativeDirector` является обязательным deterministic fallback.
- Generated definition, graph revision и text candidate принадлежат конкретному world/save. Runtime MUST NOT записывать их обратно в project source, package или authored asset.
- Replay MUST использовать exact recorded narrative candidate/admission input и MUST NOT вызывать LLM, network или model заново.

## Determinism class и external-effect boundary

Live LLM generation является nondeterministic external effect, а не deterministic system. Authoritative replay-equivalence определяется так:

> Два исполнения эквивалентны, если при одинаковых initial checkpoint, project/content/schema hashes, closed ingress batches и exact recorded narrative completion bytes они производят одинаковые request assignments, command/rejection/outcome/event/graph/state roots в `game`, `headless` и `capture-worker`.

Fresh live runs не обязаны получать одинаковый LLM text или proposal. После назначения result в exact SPEC-21 completion batch canonical bytes, result hash и assigned tick становятся записанным external input. Compare points находятся:

1. после canonical `NarrativeDirectorRequestV1` snapshot/hash;
2. после current/next completion assignment;
3. после candidate validation/rejection;
4. после atomic graph revision and RPG outcome commit;
5. на save/replay checkpoint boundary.

Worker order, provider session, response latency и wall clock не входят в equivalence predicate. Read-only replay поддерживается текущей replay machine; branching/counterfactual regeneration не принимается этим SPEC.

## Source of truth и ownership

| State | Единственный owner/source of truth | Не является source |
|---|---|---|
| Quest instance, engagement, state, variables, participants, outcome and causal lineage | RPG Framework `Quest` aggregate | journal UI, script coroutine, planner goal, generated prose |
| Private NPC intent, motive and delegation preference | Agent Intelligence `AgentIntent`/actor state | Quest aggregate, dialogue line, journal |
| Committed need/problem facts used by a quest | Existing owning RPG/World aggregate, exposed through immutable `WorldNeedViewV1` | copied mutable quest or director state |
| Quest candidate before admission | Proposal producer; immutable untrusted value | authoritative Quest state or promised reward |
| Current admitted quest graph revision and extension-slot occupancy | RPG Framework narrative graph registry | prompt, provider thread, worker cache |
| Calendar, deadlines, schedules, population and committed world facts | World Services | host wall clock, renderer visibility |
| Request scheduling, bounded snapshot selection and candidate routing | Agent Intelligence | RPG mutation authority |
| Completion assignment, command admission and atomic publication | Runtime | worker/provider completion order |
| Generated content/save/replay encoding and closure | Asset & Persistence | mutable provider cache or project source |
| Main-story anchors, protected facts and extension slots | Authored project content resolved by ProjectCompositionLock | generated candidate |
| Generated presentation text | World-local content-addressed candidate after admission | quest state or gameplay identity |

Every cross-subsystem edge uses immutable versioned views, exact revisions/hashes and proposal/command boundaries. No shared mutable narrative database is permitted.

## From intent to quest

The normative pipeline is:

`AgentIntent / committed world facts → QuestCandidateV1 → opportunity admission → QuestInstance(Latent) → disclosure → Offered → activation → Accepted → QuestOutcomeV1`.

Each arrow is explicit. A later stage MUST NOT be inferred from prose, UI visibility,
planner state or model output.

### Inputs that are not quests

Existing SPEC-06 `AgentIntent` remains the private untrusted Agent Intelligence
proposal describing what one actor wants to achieve. Its bounded
quest-opportunity projection contains actor and beneficiary
`PersistentId`s, desired-outcome predicate IDs, exact causal fact references,
urgency, delegation policy, secrecy/access constraints, validity interval,
declared reward capacity and intent revision/hash. It MAY change or disappear as
the actor plans; that alone creates no Quest state.

`WorldNeedViewV1` is an immutable revision-bound view over already committed
facts such as shortage, threat, vacancy, damage or faction demand. It creates no
second owner for those facts. A need MAY exist without an NPC requester and MAY
produce ordinary simulation behavior without becoming a quest.

Only intents/needs for which authored policy permits delegation or player
intervention are evaluated as quest opportunities. Personal habits, goals the
actor can and intends to complete normally, unverifiable wishes and purely
cosmetic conversation remain ordinary AI/world behavior.

### `QuestCandidateV1`

`QuestCandidateV1` is a bounded immutable proposal. It contains:

- exact source intent/need/fact revisions and causal event IDs;
- requester, beneficiary and participant bindings;
- typed goal, success, failure and cancellation predicate IDs;
- availability predicates and one `QuestDisclosurePolicyV1`;
- autonomy/deadline policy, `QuestPriorFactPolicyV1` and consequence-policy IDs;
- `DifficultyEnvelopeV1`, `QuestRewardContractV1` and allowed term variants;
- authored definition/extension-slot, schema, content, policy and provenance hashes.

A candidate cannot choose its authoritative Quest `PersistentId`, grant XP,
reserve arbitrary world resources, commit an outcome or contain code, prose as a
predicate, raw `WorldCommand` or unknown mutable target. Authoring data, package
logic, an NPC planner, `TemplateNarrativeDirector` and an optional LLM director
all submit the same candidate semantics and pass the same RPG validators.

### Opportunity admission

`WorldCommand::AdmitQuestOpportunity` is the single-instance admission path.
`WorldCommand::AdmitQuestGraphRevision` MAY admit several generated
opportunities atomically, but MUST run the same per-opportunity validation
inside its graph transaction.

Admission requires all of the following:

1. exact current source revisions and authorized candidate producer;
2. a meaningful player decision or intervention rather than an NPC's ordinary
   self-executable plan;
3. typed observable success, failure and cancellation predicates;
4. bounded participants, duration, autonomy, consequences and graph lineage;
5. at least one reachable disclosure channel and one deterministic
   no-player/expiry policy;
6. valid difficulty evidence, reward capacity/policy and registered term variants;
7. no duplicate, mutually exclusive conflict, protected-fact violation or
   mandatory-anchor dependency.

Successful admission atomically creates the runtime-derived Quest ID,
`QuestInstance(Latent)`, frozen source lineage and `QuestOpportunityAdmitted`.
Failure creates no Quest object. Admission runs at declared quest-decision
boundaries after relevant intent/fact revisions; it does not wait for the player
to start a conversation.

### Opportunity decision matrix

| Situation | Admit as Quest? | Initial disclosure | No-player development |
|---|---|---|---|
| NPC can and intends to solve it in ordinary schedule | No | none; ordinary dialogue/activity MAY mention it | NPC/world resolves it normally |
| NPC can solve it but explicitly seeks help, speed or delegation | Yes, if player agency and outcomes are verifiable | `Solicited` or `Public`; `Direct` when urgency policy allows | actor may solve it, withdraw it or let it expire |
| NPC cannot solve a low-urgency problem | Yes | `Solicited`, `Contextual` or `Public` | persists, changes or is taken by another actor |
| NPC cannot solve an urgent problem | Yes | `Direct` plus optional messenger/public fallback | failure, escalation or another actor may resolve it |
| Secret or trust-gated intent | Yes, but remains `Latent` while predicates fail | `Direct`/`Solicited` after access predicate | may become irrelevant or be exposed contextually |
| Player observes a committed problem without requester | Yes when an authored opportunity policy matches | `Contextual` | world cause progresses independently |
| Goal is already resolved or source revision is obsolete | No new Quest, or an existing Quest becomes `Superseded` when its autonomy policy permits | no new offer | committed cause/outcome remains |
| Goal cannot be verified by registered predicates | No | rumor or ordinary dialogue only | no Quest contract |

This is an admission matrix, not an exhaustive content taxonomy. Projects MAY
add authored opportunity policies, but cannot add a hidden creation path.

## Disclosure and player commitment

### `QuestDisclosurePolicyV1`

The closed disclosure channels are:

| Channel | Semantic trigger |
|---|---|
| `Direct` | Requester/messenger initiates contact when authored contact, urgency and access predicates hold |
| `Solicited` | Player issues a semantic work/help inquiry to an eligible actor/faction |
| `Contextual` | Player observes an authorized fact, place, item, event or conversation consequence |
| `Public` | Player accesses an authored board, notice, faction feed or messenger projection |

Policy stores the allowed channels, typed trigger/access predicates, authored
priority, disclosure window, repeat/cooldown rules and stable policy hash. For
equal priority, eligible opportunities sort by `(admission_world_tick,
quest_id)`; container iteration, localized text, LLM wording and contact arrival
order are forbidden.

`QueryQuestOpportunities` is a read-only bounded query used by dialogue and UI.
`WorldCommand::DiscloseQuestOpportunity` revision-checks the chosen latent
instance and atomically commits `Offered`, channel/source metadata and
`QuestDisclosed`. Every channel uses this same path. A dialogue line is rendered
from the resulting immutable projection; it never creates or mutates the Quest.

A semantic player inquiry such as «есть работа?» queries already admitted,
currently eligible latent opportunities. Direct approach, observation and
public notices select from the same set at declared boundaries. Repeating the
same disclosure is idempotent.

### Acceptance, decline and prior facts

`WorldCommand::AcceptQuest` performs a second `QuestActivationV1` validation:
the Quest and source revisions are current, availability/deadline and
participant/resource predicates still hold, there is no exclusive conflict and
the player satisfies declared eligibility. It then freezes the selected term
variant, `ChallengeAssessmentV1`, reward contract revision and activation tick
and commits `Accepted` atomically. A stale offer is rejected without mutation;
ordinary world evaluation MAY subsequently resolve it as `Superseded`.

Decline is not a terminal quest outcome. `WorldCommand::DeclineQuestOffer`
records `QuestOfferDispositionV1` with attempt count, last disclosure
event/channel and exact `declined_until_world_tick`. The Quest remains
`Offered`, can keep developing under its autonomy policy and MAY be accepted
later. Permanent withdrawal uses an explicit `Cancelled` or `Superseded`
outcome, not a UI-only refusal flag.

`QuestPriorFactPolicyV1` is closed:

- `ProspectiveOnly` (default) counts only facts committed after activation;
- `SinceAdmission` may count matching facts committed after opportunity admission;
- `CitedCommittedFacts` may additionally recognize only exact fact IDs captured
  and validated during admission/activation.

The validator records every recognized fact ID. It MUST NOT search arbitrary
history or silently auto-complete a newly offered quest because similar prose
describes an earlier action.

## Challenge and reward contract

`DifficultyEnvelopeV1` declares allowed bounds and evidence requirements for
seven typed factors: danger, goal complexity, access/distance, time pressure,
social/legal risk, required resources and uncertainty. The candidate MAY cite
evidence, but the RPG validator recomputes normalized integer factors and
`DifficultyBand` through a project-locked assessment-policy hash.

At activation, `ChallengeAssessmentV1` freezes source evidence revisions,
factor vector, integer score, band and policy hash. Intrinsic challenge does not
scale from the current player level and is not recalculated after acceptance
because the player became stronger, found a shortcut or delayed completion.
Changed world conditions before acceptance require a fresh assessment.

`QuestRewardContractV1` keeps four concerns separate:

1. promised world reward that the requester/faction can actually offer,
   including exact capacity/reservation policy;
2. system progression reward such as XP, derived only from the validated
   challenge band and registered outcome modifier;
3. committed world/faction/relationship consequences;
4. closed authored term variants and compensation/failure policy.

NPCs, scripts and LLM output MAY propose only a reward envelope or one registered
term variant. They cannot set final XP or arbitrary effects. The accepted
contract is revision-frozen; later inability to deliver a promised world reward
must become an explicit causal outcome/compensation path, not a silent rewrite.
Free-form reward negotiation is outside V1.

## Quest lifecycle contract

### `QuestEngagementStateV1`

Every live `QuestPayloadV1` gains one closed engagement value:

- `Latent` — admitted world-local opportunity not yet disclosed to the player;
- `Offered` — disclosed through one admitted channel but not accepted;
- `Accepted` — player has committed a successful `QuestActivationV1`;
- `Resolved` — terminal `QuestOutcomeV1` is committed.

`Latent`, `Offered` and `Accepted` remain ordinary states of one durable Quest aggregate. Unload, NPC absence, journal visibility or player distance MUST NOT create/delete the instance.

`QuestPayloadV1` additionally contains source/admission lineage, disclosure
policy and history, offer disposition, prior-fact policy, selected/frozen
challenge and reward revisions, graph revision and autonomy policy. UI may hide
or group these states but cannot reinterpret them.

| From | Trigger/command | To | Required rule |
|---|---|---|---|
| no Quest | admitted `AdmitQuestOpportunity` or graph batch | `Latent` | derive ID and publish the complete aggregate atomically |
| `Latent` | eligible `DiscloseQuestOpportunity` | `Offered` | record exact disclosure channel/source |
| `Offered` | valid `AcceptQuest` | `Accepted` | revalidate and freeze activation/challenge/reward terms |
| `Offered` | `DeclineQuestOffer` | `Offered` | update disposition/cooldown only |
| `Latent`, `Offered` or `Accepted` | policy-authorized terminal command | `Resolved` | commit exactly one `QuestOutcomeV1` |

There is no backward engagement transition. Reopening, recovery or sequel
creates a causally linked new opportunity rather than rewriting resolved
history.

### `QuestAutonomyPolicyV1`

Every new quest definition MUST explicitly declare one closed policy:

| Mode | Background progression | Terminal authority |
|---|---|---|
| `PlayerProtected` | Only listed nonterminal world transitions MAY commit | Terminal transition requires a player-originated validated action |
| `WorldReactive` | Listed committed `DomainEvent`/world-fact predicates MAY select listed transitions | Listed world transitions MAY commit a terminal outcome |
| `DeadlineBound` | Listed transitions MAY commit before deadline | Exact World Services `world_tick` boundary selects one declared expiry/failure transition after warning/grace policy |

Legacy migration with no field maps to `PlayerProtected`. New definitions with omitted/unknown mode fail schema validation. Policy contains exact allowed transition IDs, event/fact schema IDs, definition/policy hashes and, for `DeadlineBound`, integer `deadline_world_tick`, ordered warning boundaries, grace policy and terminal transition ID. Host date/time/timezone is forbidden.

World facts may continue to accumulate for `PlayerProtected`, but the validator MUST reject an autonomous terminal transition. Changing autonomy mode for a live Quest is itself a revision-checked RPG transition.

### `QuestOutcomeV1`

`QuestOutcomeV1` contains schema version, quest ID, before/after quest revisions, one closed outcome `Success | Failure | Expired | Cancelled | Superseded`, causal command/event IDs, definition/policy hashes and optional continuation-policy ID.

Outcome is a fact only after atomic Quest commit. Failure is not rollback: committed consequences remain authoritative. Compensation, recovery or sequel is a future command/quest chain.

### `NarrativeHookV1`

A committed outcome or allowed world event MAY open an immutable hook:

```text
NarrativeHookV1 {
  schema_version,
  hook_id,
  source_event_id,
  optional_source_quest_id,
  source_outcome,
  world_tick,
  source_fact_refs[],
  extension_slot_ids[],
  continuation_depth,
  policy_hash,
  hook_hash
}
```

`hook_id` is runtime-derived causal identity. Facts are revision/hash-bound immutable references. Hooks sort by `(world_tick, source_event_id, hook_id)` and are consumed only by atomic graph commit. Generated continuation depth is limited to 8; exhaustion commits the authored deterministic closure outcome and schedules no further generated request.

## Dynamic quest graph contract

### `NarrativeAnchorV1` и `NarrativeExtensionSlotV1`

`NarrativeAnchorV1` identifies an immutable authored story beat, protected fact or ending with exact definition/content hash and static reachability requirements.

`NarrativeExtensionSlotV1` identifies an authored attach point, permitted node/edge/transition primitive sets, participant/resource constraints, allowed consequence classes, maximum occupancy and fallback template policy. A candidate cannot invent a slot.

Mandatory anchors MUST remain present and reachable through an authored path that does not depend on any generated node, model response or network availability. Candidate validation uses the exact current graph revision and project-locked guardrail profile.

### `QuestGraphRevisionV1`

`QuestGraphRevisionV1` is immutable content-addressed world-local data:

```text
QuestGraphRevisionV1 {
  schema_version,
  graph_revision,
  parent_graph_hash,
  project_composition_lock_hash,
  schema_registry_hash,
  guardrail_profile_hash,
  admitted_patch_hash,
  ordered_node_hashes[],
  ordered_edge_hashes[],
  occupied_extension_slots[],
  causal_command_id,
  graph_hash
}
```

Graph revision increments exactly once per accepted patch. Node/edge order is canonical by stable type tag and ID bytes. A rejected patch leaves revision/hash/slot occupancy unchanged.

### `QuestGraphPatchV1`

Patch may contain only:

- generated quest definitions composed from registered predicate, transition, effect and presentation primitives;
- explicit edges between new nodes and allowed extension slots;
- initial `QuestCandidateV1` declarations using causal spawn slots rather than caller-supplied runtime IDs; each passes the ordinary opportunity-admission validator before any instance is created;
- locale-tagged presentation text candidates that never act as semantic identity;
- exact cited hook/fact/participant/resource revisions.

One candidate is bounded to 32 new graph nodes, 96 edges, 16 new Quest instances and 64 unique participants. Arbitrary code, prompt/tool schema, raw `WorldCommand`, mutable field patch, filesystem path, credential, vendor object, provider session or unknown primitive is forbidden.

Cycles require exact maximum visits and non-zero cooldown policy. An unbounded cycle, missing terminal/fallback path or branch that becomes the only path to a mandatory anchor rejects the complete candidate.

## Narrative Director contracts

### `NarrativeDirectorRequestV1`

Request contains:

- protocol/schema, request and idempotency IDs;
- exact graph/world/calendar revisions and hashes;
- project composition, schema registry, content and guardrail profile hashes;
- deterministic bounded projections of live quests, open hooks, extension slots, participants, resources and authorized facts;
- delegation-approved `AgentIntent` projections and `WorldNeedViewV1` values
  MAY appear only inside the same authorized fact ceiling; unrelated private
  motives are excluded;
- allowed primitive/policy IDs;
- exact `deadline_world_tick` and cancellation key;
- request hash.

Hard ceilings are 256 KiB serialized request, 256 facts, 128 quest projections and 64 hooks. Selection sorts by authored priority, stable relevance key and stable ID; worker, storage and hash-map order are forbidden.

Request contains no save bytes, complete hidden quest graph, secrets, arbitrary tools, filesystem paths or mutable objects.

### `NarrativeDirectorCandidateV1`

Candidate contains request/idempotency IDs, request hash, expected graph/world revisions, one `QuestGraphPatchV1`, cited fact/hook/participant revisions, locale-tagged text, adapter/protocol/model/content hashes, generation parameters/provenance, redaction classification and candidate hash.

Serialized candidate is limited to 1 MiB and the graph bounds above. Free-form prose cannot encode a gameplay effect; only typed patch primitives are evaluated.

### Commit command and events

The only authoritative graph mutation is `WorldCommand::AdmitQuestGraphRevision`. Its body contains candidate hash/bytes reference, request/expected revisions, exact policy/content/schema hashes and capability. It does not contain authoritative command ID or preselected PersistentIds.

Runtime/RPG validation derives command/graph/object IDs through ADR-022 causal slots and atomically commits graph revision, created Quest aggregates, consumed hooks and ordered events:

- `QuestGraphRevisionAdmitted`;
- `QuestOpportunityAdmitted`;
- `QuestDisclosed`;
- `QuestActivated`;
- `QuestOfferDeclined`;
- `QuestEngagementChanged`;
- `QuestOutcomeCommitted`;
- `NarrativeHookOpened`;
- `NarrativeHookConsumed`.

Rejection emits a structured diagnostic/receipt, not a committed graph event.

## Validation and atomicity

Opportunity admission and activation use fixed validation orders described
above and the same transaction planner as authored Quest operations. A batch
graph admission validates every included `QuestCandidateV1` before constructing
any Quest aggregate; one invalid opportunity rejects the complete patch.

Narrative graph-candidate validation order is fixed:

1. IPC framing, canonical encoding, schema/version, byte/count bounds, request ID/hash and idempotency;
2. completion assignment, cancellation and `deadline_world_tick`;
3. exact project/schema/content/guardrail compatibility;
4. current graph/world/calendar/quest/hook/fact/participant revisions;
5. capability, provenance, redaction and allowed primitive registry;
6. anchor preservation, extension-slot authority and mandatory-path reachability;
7. node/edge identity, cycle, depth, participant/resource and terminal/fallback closure;
8. autonomy policy, quest state-machine and typed effect preconditions;
9. construction of complete `RpgTransactionPlan` plus graph-registry write/event set;
10. atomic Runtime recheck and commit or discard of the complete staging buffer.

No stage may publish a partial graph, Quest instance, hook consumption, effect or narrative fact. A conflict causes one stable rejection; retry is a new request against the current revision unless the exact idempotent receipt already exists.

## Async completion, fallback and time advance

Narrative requests use SPEC-23 `JobClass`/resource bounds and SPEC-21 `CompletionSignalV1`. Result bytes are immutable and revision-bound. Current/next assignment and canonical result merge are independent of worker/provider order.

`deadline_world_tick` is authoritative; provider wall timeout only creates a staged health/deadline signal and cannot choose simulation outcome during replay. A candidate assigned after the deadline is rejected as late even if its contents are valid.

`TemplateNarrativeDirector` is an in-process deterministic producer over the same request schema, authored templates, slots, guards, validators and admission command. At the first narrative decision boundary after deadline with no valid staged candidate, it produces the declared fallback. It MAY create simpler world-local chains; no-op is allowed only when the exact hook policy declares authored closure.

`WorldAdvancePlanV1` treats quest deadline, outcome, hook and graph-decision ticks as observable boundaries. Stepped and bulk execution stop at the same earliest boundary. Bulk advance MUST NOT wait for LLM; without an already assigned valid candidate it uses the same deterministic fallback and continues through ordinary commands. If continuation depth, graph capacity or chain budget is exhausted, the hook commits its authored deterministic closure outcome and schedules no replacement request.

## Persistence, replay and world-local content

Save closure contains:

- current `QuestGraphRevisionV1` and generated definition/text blobs by content hash;
- Quest source/admission lineage, engagement/autonomy/outcome/causal data in RPG segment;
- disclosure policy/history, offer disposition/cooldown and exact recognized prior-fact IDs;
- frozen activation, `ChallengeAssessmentV1`, term variant and `QuestRewardContractV1` revisions;
- open/consumed hook records and continuation depth;
- pending request ID/hash, expected revisions, deadline and cancellation state;
- exact assigned/admitted candidate references and command ledger receipts.

It contains no model weights, provider thread, credentials, worker task handle, mutable cache or project-source path. Missing required generated blob/schema fails load before world mutation and preserves the previous save generation.

Replay supplies exact recorded completion bytes/hashes at their recorded assignment boundaries, then reruns production validation and command paths. It makes zero `ai-host`, network or model calls. First mismatch reports request, assigned tick, graph revision, candidate hash and first differing command/event/graph/state root as `NONDETERMINISTIC_RESULT`.

Generated content never writes authored bundles or packages. A future export may create a bounded `AgentChangeSet`, but automatic source mutation is outside this SPEC.

## Security, privacy and budgets

- Narrative snapshot is capability-filtered and default-redacted; raw save, secrets and unrestricted player text are forbidden.
- Remote provider use remains default-deny and requires existing project/user grant. Revocation prevents new requests and selects local/template fallback.
- Prompt/tool injection has no mutable tool surface. Unknown tool/function output is rejected before command construction.
- Candidate parsing, graph validation and staging use bounded allocation, recursion, node/edge and operation budgets.
- `ai-host` compute is outside authoritative tick budget. Request construction, result validation and commit use existing ADR-016 budget categories; no untracked tick work is allowed.
- Pending request count, due validation work and fallback work use SPEC-23 finite queues. Mandatory due work cannot be silently dropped.

## Stable diagnostics and failure semantics

| Diagnostic | Required outcome |
|---|---|
| `QUEST_CANDIDATE_INVALID` | Reject malformed, unverifiable or unauthorized opportunity; create no Quest instance |
| `QUEST_OPPORTUNITY_NOT_ACTIONABLE` | Keep intent/need as ordinary actor/world state; create no Quest instance |
| `QUEST_DISCLOSURE_NOT_ALLOWED` | Reject channel/trigger/access mismatch; engagement and cooldown unchanged |
| `QUEST_ACTIVATION_STALE` | Reject acceptance without partial reward/resource reservation or state change |
| `QUEST_PRIOR_FACT_NOT_ADMISSIBLE` | Ignore/reject unbound historical fact according to policy; never auto-complete silently |
| `QUEST_REWARD_POLICY_INVALID` | Reject admission/activation; grant no promised or progression reward |
| `QUEST_AUTONOMY_VIOLATION` | Reject forbidden background/terminal transition; quest revision unchanged |
| `QUEST_DEADLINE_INVALID` | Reject definition/transition or fail required load before mutation |
| `NARRATIVE_REQUEST_STALE` | Cancel/discard request; open hook and graph remain unchanged |
| `NARRATIVE_CANDIDATE_SCHEMA_INVALID` | Reject complete candidate; no graph/RPG/event publication |
| `NARRATIVE_GRAPH_GUARDRAIL_VIOLATION` | Reject anchor/slot/reachability/primitive violation atomically |
| `NARRATIVE_GRAPH_BUDGET_EXCEEDED` | Reject oversized/deep/cyclic candidate; use declared template closure |
| `NARRATIVE_DEADLINE_EXCEEDED` | Discard late response and select deterministic fallback at decision boundary |
| `NARRATIVE_RESULT_COLLISION` | Fail closed for conflicting bytes under one result identity; no retry-to-green |
| `NARRATIVE_CONTENT_MISSING` | Fail load before world mutation; preserve previous save generation |
| `NONDETERMINISTIC_RESULT` | Stop replay/product check at first differing compare point; never regenerate model output |

Process absence/crash/bad protocol, queue/resource denial, cancellation, duplicate/late result and model rejection MUST NOT block simulation or repeat a committed graph revision.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| `QUEST-OPPORTUNITY-P1` | `next check QUEST-OPPORTUNITY-P1 --scenario quest-opportunity-contract-v1 --cases 10000` | NPC intents and world needs create no Quest until valid admission; all four disclosure channels resolve to the same Quest ID/terms, stale activation and invalid prior facts/rewards reject atomically, and accepted assessment/reward revisions remain frozen. Keep ordinary AI/world behavior or the prior Quest revision on rejection. |
| `QUEST-AUTONOMY-P1` | `next check QUEST-AUTONOMY-P1 --scenario autonomous-quest-lifecycle-v1 --quests 1000 --ticks 10000 --seeds 100` | Latent/offered/accepted quests produce exact stepped/bulk and residency-tier outcomes; decline cooldown, prior-fact policy, terminal policy and deadlines behave identically across repeats and no continuation is lost or duplicated. Default to `PlayerProtected`, `ProspectiveOnly` and stepped fixed-tick advance. |
| `NARRATIVE-GRAPH-P1` | `next check NARRATIVE-GRAPH-P1 --scenario narrative-graph-admission-v1 --candidates 10000` | Stale, oversized, anchor, slot, cycle, depth, primitive and capability violations reject atomically; accepted graph hashes are exact. Keep the prior graph and use `TemplateNarrativeDirector` on failure. |
| `NARRATIVE-ASYNC-P1` | `next check NARRATIVE-ASYNC-P1 --scenario narrative-director-faults-v1 --injections 1000` | Absence, crash, timeout, cancellation, reordering, duplicate, late, restart and collision never stall a tick, mutate directly or duplicate a revision; disable the `ai-host` route and use the deterministic template fallback. |
| `NARRATIVE-REPLAY-P1` | `next check NARRATIVE-REPLAY-P1 --scenario narrative-save-replay-v1 --runs 100 --modes game,headless,capture-worker` | Save/restart at every async and commit boundary reproduces exact request/candidate/command/outcome/event/graph/state roots without network or model calls; reject divergent load/replay and retain the previous save/candidate. |

## Requirements

| ID | Requirement |
|---|---|
| `REQ-148` | An NPC intent or world need becomes a durable latent Quest only after bounded deterministic opportunity admission; disclosure, activation, decline and autonomous progression of latent/offered/accepted instances use the common RPG command path and explicit policies. |
| `REQ-149` | Committed outcomes open bounded causal hooks/chains without modifying mandatory narrative anchors. |
| `REQ-150` | Narrative Director receives a bounded immutable snapshot and submits only atomic validated graph/quest candidates; it cannot disclose a quest, assign final challenge/XP or mutate RPG state directly. |
| `REQ-151` | Async completion, fallback, save/replay and composition-root parity require no regeneration or network. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| `FAIL-062` | Invalid, stale, unverifiable, oversized, reward-invalid or guardrail-breaking opportunity/graph candidate | Change no graph, Quest, hook, reward or event state; retain ordinary world/agent state or the prior graph and deterministic fallback. |
| `FAIL-063` | Director absent, late, conflicting or restarted | Never block a tick or duplicate a commit; select the deterministic fallback at the declared boundary. |

While SPEC-31 remains Proposed, these schemas and checks do not change Accepted runtime behavior.
