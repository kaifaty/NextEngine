# SPEC-31: Autonomous quest lifecycle, divine agency и narrative director

| Поле | Значение |
|---|---|
| ID | SPEC-31 |
| Статус | Accepted |
| Версия | 1.0 |
| Последняя проверка | 2026-07-26 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](22-schema-registry-compatibility-and-migration.md), [SPEC-23](23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](adr/026-deterministic-work-resource-and-streaming-admission.md), [ADR-029](adr/029-rpg-owned-quest-graph-and-optional-narrative-director.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-031](adr/031-rpg-owned-divine-standing-and-atomic-pantheon-judgment.md) |
| Заменяет | отсутствует |

## История принятия

SPEC-31 принят вместе с ADR-029 и ADR-031 после синхронизации fixed
world-time decision boundary, cross-context atomic publication, complete
offer/covenant lifecycle, epistemic eligibility, reaction lineage и
world-locked pantheon compatibility. Accepted architecture определяет
обязательный контракт, но не утверждает runtime implementation, PASS будущих
gameplay checks, конкретную LLM, provider, model runtime, database или graph
library.

## Назначение и invariants

SPEC-31 определяет автономный lifecycle доступных и принятых квестов, bounded causal continuation после любого исхода и optional asynchronous Narrative Director, который предлагает world-local quest-graph revisions. Та же untrusted director boundary поддерживает authored роли богов: бог оценивает доступные ему факты, выражает отношение к поступку и может предложить ограниченное вмешательство, но не становится владельцем RPG state.

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
- Боги MAY быть объективно существующими участниками мира, но каждый `DivinePatronDefinitionV1` MUST иметь authored `DivineEpistemicPolicyV1`; существование бога не даёт модели доступ ко всему hidden world state.
- Отношение каждого бога к player Character MUST храниться независимо в RPG-owned `DivineStandingV1`. Один поступок MAY повысить standing у одного бога и понизить у другого; конфликт не сворачивается в одну «общую карму».
- Несколько богов MUST оценивать один root event из одного immutable pre-decision snapshot. Completion order, worker/provider traversal и порядок обхода богов MUST NOT менять merge order или commit tick. Принадлежность completion к закрытому ingress batch до decision boundary является записанным external input и MAY выбрать candidate вместо fallback, но MUST NOT переносить саму boundary.
- LLM MAY выбирать только categorical judgement и один eligible authored intervention. Exact standing deltas, effects, covenant conflicts, sanctions and cross-god spillover вычисляются registered deterministic policies.
- `DivineStandingChanged` сам по себе MUST NOT открывать новый divine judgment. Reaction chains открываются только перечисленными semantic events и ограничены causal depth/cooldown, чтобы конфликт пантеона не стал бесконечным каскадом.

## Determinism class и external-effect boundary

Live LLM generation является nondeterministic external effect, а не deterministic system. Authoritative replay-equivalence определяется так:

> Два исполнения эквивалентны, если при одинаковых initial checkpoint, project/content/schema hashes, closed ingress batches, recorded decision-boundary closures и exact recorded narrative/divine completion bytes/assignments они производят одинаковые selected per-role candidates, fallback decisions, command/rejection/outcome/event/graph/standing/state roots в `game`, `headless` и `capture-worker`.

Fresh live runs не обязаны получать одинаковый LLM text или proposal. После назначения result в exact SPEC-21 completion batch canonical bytes, result hash и assigned `SimulationTick` становятся записанным external input. Narrative/divine decision сохраняет отдельную world-time boundary; эти clock domains не сравниваются численно. Compare points находятся:

1. после canonical `NarrativeDirectorRequestV1` snapshot/hash;
2. после current/next completion assignment;
3. после canonical `NarrativeDecisionBoundaryV1` closure;
4. после candidate validation/rejection;
5. после atomic graph revision and RPG outcome commit;
6. после canonical divine batch resolution и atomic multi-standing commit;
7. на save/replay checkpoint boundary.

Worker order, provider session и wall clock не входят в equivalence predicate. Response bytes and their SPEC-21 ingress assignment do enter it as recorded external input; raw latency never becomes a numeric simulation or world-time comparison. Read-only replay поддерживается текущей replay machine; branching/counterfactual regeneration не принимается этим SPEC.

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
| Completion assignment, decision-boundary closure, cross-context command admission and atomic publication | Runtime | worker/provider completion order |
| Generated content/save/replay encoding and closure | Asset & Persistence | mutable provider cache or project source |
| Main-story anchors, protected facts and extension slots | Authored project content resolved by ProjectCompositionLock | generated candidate |
| Generated presentation text | World-local content-addressed candidate after admission | quest state or gameplay identity |
| Player standing with one god, boon/covenant offers, covenant/vow state and bounded causal judgment history | RPG Framework `DivineStanding` aggregate | provider session, Relationship aggregate, UI label, AI memory |
| God identity, persona, domains, values/taboos, epistemic ceiling and intervention catalog | Authored `DivinePatronDefinitionV1` in exact project/content closure | prompt text or mutable model memory |
| Directed god-to-god relations and covenant/intervention spillover rules | Authored content-addressed `PantheonRelationGraphV1` | LLM opinion, completion order or inferred faction relationship |
| Per-god request construction and untrusted judgement candidate | Agent Intelligence `narrative-director` role instance | RPG mutation or exact numeric delta authority |
| Final multi-god judgement vector and atomic publication | RPG/Mechanics/quest-graph owner subplans plus Runtime `CrossContextTransactionPlanV1` commit | individual worker/provider or first completed response |

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
- optional requester, beneficiary and participant bindings plus one typed
  `QuestSponsorV1`; requester MAY be absent only for an authorized world-need
  or divine-sponsored opportunity;
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

`QuestSponsorV1` is closed and records attribution without becoming Quest
authority:

```text
QuestSponsorV1 =
  Npc { character_id: PersistentId }
  | Faction { faction_id: PersistentId }
  | WorldNeed {
      owner_context_id: OwnerId,
      need_fact_ref: ImmutableFactRefV1
    }
  | DivinePatron {
      patron_definition_ref: DefinitionRefV1
    }
```

The referenced NPC/faction/fact/definition MUST be authorized and revision/hash
bound by the candidate. Sponsor does not imply requester, beneficiary,
disclosure or acceptance, and unknown sponsor variants reject the candidate.

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

### `NarrativeDecisionBoundaryV1`

Every narrative or divine request binds one immutable world-time decision
boundary:

```text
NarrativeDecisionBoundaryV1 {
  schema_version,
  boundary_id,
  source_event_or_hook_id,
  decision_world_tick,
  base_calendar_revision,
  boundary_policy_hash,
  boundary_hash
}
```

`boundary_id` is runtime-derived causal identity. `decision_world_tick` is a
World Services value and is never compared numerically with SPEC-21
`SimulationTick`. Runtime closes the boundary exactly once when stepped or bulk
`WorldAdvancePlanV1` reaches it. The stage-1 closed ingress batch of that
simulation tick is the last batch whose already assigned completions are
eligible; the boundary closure records its queue generation and
`closing_simulation_tick`.

All valid results already assigned in that closed ingress batch are staged until
the boundary. Results assigned after closure are late even if a provider began
work earlier. Receiving every result early MUST NOT commit or consume the hook
before the boundary. Missing/invalid results select the declared deterministic
fallback at closure, so bulk advance never waits for model or network.

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

## Divine agency и конфликтующий пантеон

### Conceptual boundary

Бог в этом contract — authored narrative actor, который объективно существует
в fiction, но действует через те же proposal/validation boundaries, что и
любой optional Narrative Director. LLM MAY исполнять его роль в live run и
принимать разные решения в зависимости от разрешённых поступков игрока,
текущего мира и истории отношений. Это намеренная вариативность fresh runs, а
не право менять уже записанное прошлое.

Один scalar `karma`, сумма «добра» или implicit faction relationship
запрещены. Для каждого player/god pair существует независимый standing.
Положительное решение одного бога не нормализует и не компенсирует
отрицательное решение другого. Конфликт между богами выражается authored
directed graph и может создавать одновременно положительные и отрицательные
последствия одного root event.

Нормативный поток:

```text
committed semantic event
  → eligible per-god DivineJudgmentHookV1 values
  → one immutable DivineJudgmentBatchBaseV1
  → independent per-god DivineDecisionRequestV1 values
  → recorded candidate or per-god template fallback
  → deterministic PantheonConflictResolverV1
  → WorldCommand::AdmitDivineJudgmentBatch
  → atomic DivineStanding / covenant / effect / quest commit
  → ordered DomainEvent batch
```

Бог MAY предлагать `QuestCandidateV1`, warning, boon, trial or sanction, но
player MUST явно принимать sponsored quest/covenant/voluntary vow through its
ordinary semantic action and validated command. Divine voice, prayer, dream,
omen, priest and temple are presentation/disclosure surfaces, not a second
mutation path.

### `DivinePatronDefinitionV1` и epistemic ceiling

`DivinePatronDefinitionV1` is immutable authored content and contains:

- exact patron `AssetId + ContentHash`, stable role ID, persona/voice and domains;
- typed values, taboos, preferred/forbidden event classes and authored
  categorical judgement policy;
- `DivineEpistemicPolicyV1`;
- standing-band, covenant, warning, intervention, budget, cooldown and fallback
  policy hashes;
- eligible `NarrativeExtensionSlotV1` values for god-sponsored quest branches;
- exact reference to `PantheonRelationGraphV1`.

`DivineEpistemicPolicyV1` is an allowlist, not flavor text. It declares event
schemas/fact classes a god can observe, domain/region/participant/witness
constraints, permitted causal-reference depth, maximum cited facts and
redaction policy. Only committed facts satisfying this policy may appear in a
request or be cited by a candidate. A god does not automatically know a hidden
crime, private dialogue, unrevealed identity, another god's private decision or
complete quest graph merely because it exists.

A candidate citation outside the request fact ceiling rejects that candidate.
The validator MUST NOT repair it by looking up hidden state. Authored omniscience
is allowed only as an explicit bounded policy over named fact classes; wildcard
access to all current/future schema IDs is invalid.

### RPG-owned `DivineStandingV1`

After ADR-031 acceptance, `DivineStanding` becomes a dedicated
`RpgAggregateEnvelopeV1.aggregate_kind`. Its `persistent_id` is runtime-derived
from the subject/patron causal creation identity. It is not a
`Relationship` aggregate and does not require a god to be instantiated as a
Character.

```text
DivineStandingV1 {
  schema_version,
  subject_character_id,
  patron_definition_ref,
  favor_raw,
  attention_raw,
  covenant_state,
  open_offers[],
  recent_offer_history[],
  active_vow_refs[],
  active_mandate_refs[],
  warning_ledger[],
  recent_judgment_refs[],
  last_intervention_world_tick,
  intervention_cooldowns[],
  policy_hashes[]
}
```

`favor_raw` is signed fixed-point in `[-10_000, 10_000]`.
`attention_raw` is unsigned fixed-point in `[0, 10_000]`. Arithmetic uses
checked wide intermediates; overflow or out-of-range result rejects the
complete transaction. Implicit saturation/clamp is forbidden.

Standing exists independently for every configured god even when no covenant
exists. `favor_raw` expresses approval/disapproval. `attention_raw` is a
deterministic policy input for eligible intervention tier, integer
budget/cooldown and presentation cadence. It MUST NOT select whether an
epistemically eligible patron receives a hook/request, invoke RNG, choose a
winner among gods, bypass the epistemic policy or turn a prohibited fact into
knowledge.

The authoritative standing hash is the containing SPEC-19
`RpgAggregateEnvelopeV1.payload_hash`; `DivineStandingV1` has no nested
self-hash.

After project/content validation and before the first playable tick, one
authenticated `InternalSystem` initialization command creates exactly one
standing aggregate for every patron in the bounded project-locked pantheon,
using authored initial values and causal spawn slots. LLM output and first
contact never lazily create it.

V1 freezes the exact byte-ordered patron definition set and
`PantheonRelationGraphV1.graph_hash` into the world/project closure at creation.
Schema representation migration MAY change record encoding, but MUST preserve
that patron set, every standing `PersistentId`, domain revision and causal
history. Adding, removing or retiring a patron in an existing world is
unsupported; a different set/hash requires a new world or a future superseding
ADR and explicit migration contract.

Player-facing UI receives only `DivineStandingProjectionV1`: an authored
qualitative favor band, qualitative attention band, active covenant/vow
summaries and bounded recent committed reason codes/text references. Exact raw
numbers, hidden thresholds, request prompt and unrevealed taboos are excluded.
Default favor bands are `Condemned | Disfavored | Neutral | Favored | Exalted`;
a project may rename presentation labels, but stable semantic band IDs and
threshold policy remain hash-bound.

### Divine offers, covenants, vows и compatibility

`DivineOfferV1` is a bounded child record of its owning
`DivineStandingV1`:

```text
DivineOfferV1 {
  schema_version,
  offer_id,
  offer_revision,
  offer_kind,               // Boon | Covenant
  definition_ref,
  terms_policy_hash,
  source_judgment_id,
  source_event_id,
  offered_at_world_tick,
  optional_expires_at_world_tick,
  player_disclosure_fact_ref,
  state,                    // Offered | Accepted | Declined | Expired | Superseded
  terminal_command_or_event_id
}
```

`offer_id` is runtime-derived from the judgment command and offer spawn slot.
One standing holds at most 8 open offers and 32 bounded recent terminal offer
records. Exact older terminal history remains recoverable through committed
events/save lineage rather than an unbounded aggregate vector.

Only `Offered → Accepted | Declined | Expired | Superseded` is valid. A terminal
offer cannot reopen; retry returns the original receipt. `Accepted` is not an
effect by itself: the same revision-checked player command revalidates exact
terms, uniqueness/stacking, current budgets/cooldowns, covenant compatibility
and every affected owner subplan before atomic publication.

`DivineCovenantStateV1` is closed:

`None | Offered | Active | RenunciationPending | Broken`.

The exact transitions are:

| From | Trigger | To |
|---|---|---|
| `None` | batch creates one covenant `DivineOfferV1` | `Offered` |
| `Offered` | player accepts the current offer | `Active` |
| `Offered` | decline, expiry or supersession | `None` |
| `Active` | explicit player renunciation request | `RenunciationPending` |
| `Active` | committed authored breach | `Broken` |
| `RenunciationPending` | authored completion boundary and consequences | `None` |
| `Broken` | explicit player restoration after authored prerequisites | `Active` |
| `Broken` | explicit player renunciation and authored consequences | `None` |

There is at most one open covenant offer per patron. Standing is retained
through every covenant transition. A covenant definition declares one
compatibility group and pairwise policy `Compatible | Conditional | Exclusive`
against other active groups. `Conditional` names exact predicates and
consequences. `Exclusive` rejects activation while a conflicting covenant is
active; it never bundles implicit renunciation of another covenant.

For boon/covenant state, `AdmitDivineJudgmentBatch` may create an offer or record
an authored breach already justified by the root event. It cannot accept,
decline, expire, supersede, renounce or restore on the player's behalf. Those
transitions use `AcceptDivineOffer`, `DeclineDivineOffer`,
`ExpireDivineOffer`, `SupersedeDivineOffer`,
`BeginDivineCovenantRenunciation`, `CompleteDivineCovenantRenunciation` or
`RestoreDivineCovenant` through ordinary revision-checked RPG/Runtime command
paths. Expiry and supersession use `InternalSystem`; expiry is scheduled at the
exact World Services boundary. A boon acceptance that changes Mechanics state
uses the existing SPEC-13 `EffectTransaction`; it does not use or extend
`CrossContextTransactionPlanV1` and cannot change the quest graph.

No candidate, dialogue line or boon may silently replace, merge or remove a
covenant. A voluntarily accepted covenant or vow MAY authorize stricter later
sanctions, but the exact promise and penalty policy must have been visible in
the activation projection.

### `PantheonRelationGraphV1`

The pantheon is authored content-addressed directed graph, independent of
current player standing:

```text
PantheonRelationGraphV1 {
  schema_version,
  graph_id,
  patron_definition_refs[],
  directed_edges[],
  covenant_compatibility_policies[],
  conflict_resolver_policy_hash,
  graph_hash
}
```

Each directed edge has one relation
`Allied | Tolerant | Rival | Hostile | Indifferent` and declares:

- exact source and target patron refs;
- semantic root-event and intervention classes to which it applies;
- categorical/standing spillover rules;
- committed covenant acceptance/breach and accepted unique-boon «insult» rules;
- whether a bounded counterquest or countertrial hook is permitted;
- exact cooldown, maximum visits and `reaction_depth` bound.

The graph MAY be asymmetric. Missing source/target pair resolves only to the
graph's hash-bound default `Indifferent`; unknown patron, relation or policy is
invalid. Edges do not directly mutate standing. They are immutable input to the
deterministic resolver and cannot be invented or rewritten by LLM output.

### Hooks and one shared pre-decision snapshot

`DivineJudgmentHookV1` is opened only by an authored semantic event class whose
committed root event is observable under the target patron's epistemic policy.
Examples include a player action/outcome, accepted or broken vow, accepted
unique boon, covenant transition, completed divine quest, committed sanction or
authored pantheon-conflict outcome.

Every epistemically eligible patron receives its authorized hook and request
from the same batch base regardless of `attention_raw`. Attention may change
only the later deterministic intervention tier, budget/cooldown and
presentation cadence.

Ordinary `DivineStandingChanged`, `DivineAttentionChanged` and presentation
events MUST NOT open another hook. Every hook stores root event ID, target
patron, authorized fact refs, source standing/pantheon/policy revisions,
decision-boundary ref, `reaction_root_event_id`, optional `parent_hook_id`,
cooldown and `reaction_depth`.

An independent player/world command not emitted as part of an existing divine
resolution creates a new reaction root with
`reaction_root_event_id = root_event_id`, no parent and depth `0`. An automatic
counterhook/consequence emitted by `PantheonConflictResolverV1` inherits the
parent root and uses checked `parent_depth + 1`. An explicit later player
acceptance, covenant action or Quest outcome is a new semantic command/root; its
separate generated-quest `continuation_depth` remains governed by the quest
chain bound. Bookkeeping events never reset or advance either counter.

Default maximum automatic reaction depth is 4 and MUST NOT exceed 8. Overflow or
exhaustion commits the authored no-further-reaction closure.

All hooks for one root event form `DivineJudgmentBatchBaseV1`. It freezes one
pre-decision snapshot before any god's result is applied:

```text
DivineJudgmentBatchBaseV1 {
  schema_version,
  batch_id,
  root_event_id,
  reaction_root_event_id,
  optional_parent_hook_id,
  world_tick,
  world_rpg_graph_roots,
  pantheon_graph_hash,
  ordered_eligible_patrons[],
  ordered_hook_hashes[],
  base_standing_revisions[],
  covenant_vow_warning_revisions[],
  decision_boundary_ref,
  reaction_depth,
  base_hash
}
```

Patrons sort by patron `AssetId` bytes and hook ID. All per-god requests cite
this exact `base_hash`; no request sees another god's not-yet-committed
candidate or a standing delta from the same batch. This is a lockstep
snapshot/atomic-merge strategy: worker and completion order are intentionally
unobservable.

### Per-god request and candidate

Each eligible patron gets a separate `DivineDecisionRequestV1`, specialized
from the Narrative Director request protocol. It contains:

- request/idempotency ID, `batch_id`, `base_hash`, root event and exact
  `NarrativeDecisionBoundaryV1` ref/hash;
- exact patron definition, epistemic, standing, pantheon and policy hashes;
- only that patron's bounded authorized facts, current qualitative/typed
  standing/covenant/vow/warning projection and relevant directed graph edges;
- eligible judgement categories, intervention catalog entries, quest extension
  slots, remaining integer budgets and cooldowns;
- deterministic fallback policy and request hash.

One request is limited to 64 KiB, 64 cited facts, 32 relevant pantheon edges,
16 warnings/vows and 32 eligible interventions.

`DivineDecisionCandidateV1` contains request/base hashes, cited fact/revision
set, exactly one judgement
`StrongDisapproval | Disapproval | Ambivalent | Approval | StrongApproval`,
optional one eligible intervention ID/tier, optional bounded sponsored
`QuestCandidateV1`/graph patch, locale-tagged presentation text, provenance and
candidate hash. Sponsored quest/patch payload is permitted only when the chosen
intervention is `Trial` or `SponsoredQuest`. The candidate is limited to
256 KiB.

The candidate cannot provide numeric favor/attention delta, edit another god's
standing, change a covenant, waive a sanction prerequisite, select a runtime
`PersistentId`, emit code/raw effect/raw `WorldCommand` or cite unprovided
facts. Registered policies deterministically convert category and intervention
tier to exact numeric/effect proposals.

### Interventions, rewards and sanction fairness

The closed divine intervention classes are:

| Class | Allowed result |
|---|---|
| `StandingOnly` | exact policy-derived favor/attention change and reason |
| `Warning` | visible causal warning record with taboo/vow/policy reference |
| `Boon` | one eligible authored bounded status/effect/reward tier with exact `Immediate` or `Offered` delivery policy; persistent, unique or covenant-affecting boons MUST be `Offered` |
| `Trial` | one eligible authored trial hook or sponsored quest candidate |
| `MinorSanction` | bounded authored status/effect after a cited taboo already committed as known to the player, or an active visible vow term |
| `MajorSanction` | bounded authored status/effect or quest consequence only after causal warning plus repeated violation, or a voluntarily accepted covenant/vow that explicitly declared the penalty |
| `SponsoredQuest` | bounded `QuestCandidateV1` attached through an eligible divine extension slot |

Every intervention has exact typed preconditions, target classes,
effect/quest primitives, budget charge, cooldown, stacking/conflict policy and
fallback. Candidate text does not grant a reward or apply a sanction.
An unavailable/ineligible intervention, invalid citation or failed sanction/
covenant prerequisite rejects the complete per-god candidate. Runtime MUST NOT
salvage its judgement category or text. It substitutes the patron's one
canonical `DivineDecisionCandidateV1` template result, which MAY itself be an
authored `StandingOnly`, `Warning` or `Ambivalent`-without-intervention choice.

Major sanction without the required causal warning/repetition/voluntary promise
MUST reject that patron's complete model candidate even if it strongly
disapproves. Minor sanction without an explicit cited taboo/vow term and its
prior player-disclosure fact does the same. A hidden authored taboo MAY motivate
a categorical judgement, favor change or warning, but redaction still applies
and it cannot authorize a sanction. Mandatory story anchor or ending can never
depend on accepting a divine boon, covenant, generated quest or model-generated
warning.

God-sponsored opportunities use
`QuestSponsorV1::DivinePatron { patron_definition_ref }`.
They reuse existing channels:

- direct divine voice/manifestation — `Direct`;
- prayer or explicit appeal — `Solicited`;
- dream, omen or discovered sign — `Contextual`;
- priest, temple or cult notice — `Public`.

No fifth disclosure channel or auto-accept path is introduced.
An offered covenant, boon or sponsored quest causes no rival spillover merely
because a model proposed it. Explicit player acceptance commits a new semantic
root event; only then may epistemically eligible rivals judge it under authored
edge policy.

### Atomic pantheon conflict resolution

Only when the exact divine `NarrativeDecisionBoundaryV1` closes does Runtime
construct one canonical selected-result set. Receiving every per-god result
early changes only staging and MUST NOT commit early. A missing, late, invalid,
conflicting or cancelled result is replaced independently by that patron's
deterministic template result; it does not discard valid results of other
patrons.

`PantheonConflictResolverV1` then evaluates all selected personal judgements
against the same batch base:

1. map each personal category/intervention to authored exact standing,
   warning/effect/quest proposals;
2. enumerate a directed edge only when both source and target are present in
   `ordered_eligible_patrons` and the target has an authorized hook hash in the
   batch base, then sort by
   `(target_patron_id, source_patron_id, edge_id, consequence_slot)`;
3. compute each final target standing as its base plus personal delta plus all
   applicable incoming allied/rival/hostile deltas in that exact order and,
   when the root event is an
   already committed covenant/boon/quest transition, its declared conflict and
   counterquest consequences with checked integer arithmetic;
4. apply each directed consequence at most once and deduplicate identical
   semantic effects/hooks by causal key;
5. validate budgets, cooldowns, covenant compatibility, sanction fairness,
   graph slots, anchors, aggregate revisions and complete cross-context write
   set;
6. produce one `DivineJudgmentResolutionV1` with exact per-patron before/after
   values, selected candidate/fallback hashes and resolution hash.

Thus one act may directly earn `Approval` from one god and `Disapproval` from
another, and an accepted boon/covenant from the first may add a separately
authored rival spillover against the second. No zero-sum normalization is
performed. A project is expected to make some value systems incompatible; the
player need not and often cannot remain favored by every god.

An edge whose target lacks an authorized hook in the batch causes no standing
or intervention mutation. If that patron later observes an authorized committed
fact, ordinary hook construction may create a future batch from that new
semantic event; resolver lookup never upgrades hidden knowledge.

The only authority path for one pantheon resolution is
`WorldCommand::AdmitDivineJudgmentBatch`. It revision-checks the batch base and
constructs one `CrossContextTransactionPlanV1`. That plan atomically publishes
the complete standing vector, offer/covenant/warning records, typed mechanic
effects, admitted sponsored quest opportunities/hooks and ordered events:

- `DivineJudgmentCommitted`;
- `DivineStandingChanged`;
- `DivineAttentionChanged`;
- `DivineWarningIssued`;
- `DivineInterventionCommitted`;
- `DivineOfferCreated`;
- `DivineCovenantChanged`;
- `DivineJudgmentHookOpened`;
- `DivineJudgmentHookConsumed`.

Events with no changed field are omitted. Inside the RPG owner subplan, event
order is patron ID, typed operation slot and event-local slot, then the existing
SPEC-19/ADR-022 tie-break. `CrossContextTransactionPlanV1` performs the final
owner-vector merge defined below. Any stale revision, checked overflow, invalid effect or failed
cross-context precondition rejects the whole batch: no god is updated
partially. Already committed prior consequences are never hidden or rolled
back; correction is a new causal command/judgment.

`DivineOfferTransitioned` is emitted only by the separate player/internal
transition commands defined above. `DivineCovenantChanged` is emitted by those
commands and MAY also be emitted by a judgment batch only for an authored
breach included in its complete plan. A judgment batch emits
`DivineOfferCreated` for a new offer and never performs an offer terminal
transition.

## Narrative Director contracts

`narrative-director` is one engine-owned protocol role with authored role
instances. A world-graph director receives the schemas below directly; every
god uses the specialized divine request/candidate subset above. The adapter may
route them to one or several models, but adapter/provider identity never merges
the logical requests or creates a «council» response. Separate gods remain
separate requests and candidates.

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
- exact `NarrativeDecisionBoundaryV1` ref/hash and cancellation key;
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
Divine judgement uses the separate
`WorldCommand::AdmitDivineJudgmentBatch`; an individual god candidate cannot be
submitted as a trusted graph or standing command.

## Validation and atomicity

### `CrossContextTransactionPlanV1`

`AdmitQuestGraphRevision` and `AdmitDivineJudgmentBatch` are the only V1
commands permitted to use the Runtime-owned cross-context composition contract:

```text
CrossContextTransactionPlanV1 {
  schema_version,
  causal_command_id,
  canonical_command_body_hash,
  project_composition_lock_hash,
  rpg_transaction_plan,
  optional_mechanics_delta_plan,
  optional_quest_graph_registry_plan,
  ordered_cross_context_preconditions[],
  ordered_merged_event_drafts[],
  plan_hash
}
```

RPG Framework alone builds `RpgTransactionPlan` and the immutable
`QuestGraphRegistryPlanV1`; Mechanics Runtime alone builds immutable
`MechanicsDeltaPlanV1`. Runtime does not reinterpret owner semantics. It
validates that every embedded subplan binds the same causal command/body,
project lock and cited revisions/hashes, then constructs the wrapper. LLM,
packages, scripts and command callers cannot construct or modify any trusted
subplan.

Each owner preserves its existing local event order. Runtime merges completed
owner event vectors by:

```text
(
  owner_context_tag,          // RPG = 0, Mechanics = 1, QuestGraph = 2
  owner_event_index,
  event_schema_id_nfc_utf8,
  primary_persistent_id_bytes
)
```

Tags are permanent V1 values. An absent optional subplan contributes no event or
index gap. RPG-only transactions do not use this wrapper and retain their exact
SPEC-19 order and event IDs.

Runtime stages every owner write set in one isolated buffer, rechecks the full
cross-context precondition set at the declared commit point and publishes all
state, graph, Mechanics deltas, events and the terminal receipt together.
Failure discards the complete buffer and MUST NOT consume a hook, occupy a graph
slot, transition an offer or expose a partial effect.

Opportunity admission and activation use fixed validation orders described
above and the same transaction planner as authored Quest operations. A batch
graph admission validates every included `QuestCandidateV1` before constructing
any Quest aggregate; one invalid opportunity rejects the complete patch.

Narrative graph-candidate validation order is fixed:

1. IPC framing, canonical encoding, schema/version, byte/count bounds, request ID/hash and idempotency;
2. completion assignment, cancellation and decision-boundary closure;
3. exact project/schema/content/guardrail compatibility;
4. current graph/world/calendar/quest/hook/fact/participant revisions;
5. capability, provenance, redaction and allowed primitive registry;
6. anchor preservation, extension-slot authority and mandatory-path reachability;
7. node/edge identity, cycle, depth, participant/resource and terminal/fallback closure;
8. autonomy policy, quest state-machine and typed effect preconditions;
9. construction of complete `RpgTransactionPlan` and
   `QuestGraphRegistryPlanV1`, followed by their
   `CrossContextTransactionPlanV1` composition;
10. atomic Runtime recheck and commit or discard of the complete staging buffer.

No stage may publish a partial graph, Quest instance, hook consumption, effect or narrative fact. A conflict causes one stable rejection; retry is a new request against the current revision unless the exact idempotent receipt already exists.

## Async completion, fallback and time advance

Narrative requests use SPEC-23 `JobClass`/resource bounds and SPEC-21 `CompletionSignalV1`. Result bytes are immutable and revision-bound. Current/next assignment and canonical result merge are independent of worker/provider order.

`NarrativeDecisionBoundaryV1.decision_world_tick` is authoritative for when a
decision commits. SPEC-21 `CompletionAssignmentV1.assigned_tick` is
authoritative only for which closed ingress batch contains a result. Runtime
MUST NOT compare these nominal values or derive one from the other.

A provider wall timeout creates only a staged health signal. A result is usable
when its assignment is present no later than the stage-1 closed ingress batch
recorded by the boundary closure; any later assignment is rejected as
`NARRATIVE_BOUNDARY_CLOSED`, even if its contents are valid.

`TemplateNarrativeDirector` is an in-process deterministic producer over the same request schema, authored templates, slots, guards, validators and admission command. At the exact narrative decision boundary with no valid staged candidate, it produces the declared fallback. It MAY create simpler world-local chains; no-op is allowed only when the exact hook policy declares authored closure.

For one divine batch, template fallback is selected independently per missing
or invalid patron request at that same boundary. The selected per-god
candidates/fallbacks are then sorted and resolved only from
`DivineJudgmentBatchBaseV1`; a fast result cannot observe a slow result or
preempt the boundary. External generation MAY run concurrently, while
authoritative evaluation uses lockstep snapshot plus canonical atomic merge.

`WorldAdvancePlanV1` treats quest deadline, outcome, hook, graph-decision and
divine-decision ticks as observable boundaries. Stepped and bulk execution stop
at the same earliest boundary. Bulk advance MUST NOT wait for LLM; without an
already assigned valid candidate it uses the same deterministic fallback and
continues through ordinary commands. For a divine boundary it closes the
eligible patron set from the recorded base snapshot, fills missing patrons with
their template results and commits one batch. If continuation/reaction depth,
graph capacity or chain budget is exhausted, the hook commits its authored
deterministic closure outcome and schedules no replacement request.

## Persistence, replay and world-local content

Save closure contains:

- current `QuestGraphRevisionV1` and generated definition/text blobs by content hash;
- Quest source/admission lineage, engagement/autonomy/outcome/causal data in RPG segment;
- disclosure policy/history, offer disposition/cooldown and exact recognized prior-fact IDs;
- frozen activation, `ChallengeAssessmentV1`, term variant and `QuestRewardContractV1` revisions;
- open/consumed hook records and continuation depth;
- pending request ID/hash, expected revisions, decision-boundary ref/hash and
  cancellation state;
- exact assigned/admitted candidate references and command ledger receipts.
- `DivineStandingV1` aggregates, open/terminal `DivineOfferV1` records,
  covenant/vow/warning history, intervention cooldowns and last committed
  judgment refs;
- exact `DivinePatronDefinitionV1`, `DivineEpistemicPolicyV1`,
  byte-ordered world-locked patron set, `PantheonRelationGraphV1`,
  conflict-resolver and standing-band hashes;
- open/consumed divine hook and batch-base records, per-god pending
  request/boundary metadata, boundary-closing ingress generation and
  `SimulationTick`, selected candidate/fallback hashes and committed
  `DivineJudgmentResolutionV1`.

It contains no model weights, provider thread, credentials, worker task handle, mutable cache or project-source path. Missing required generated blob/schema fails load before world mutation and preserves the previous save generation. A save/project closure with another patron set or pantheon graph hash fails as `DIVINE_PANTHEON_WORLD_MISMATCH`; recovery is the original exact content closure or a new world, not inferred add/remove migration.

Replay supplies exact recorded completion bytes/hashes at their recorded
SPEC-21 assignments and replays the recorded decision-boundary closure, then
reruns production validation and command paths. It
makes zero `ai-host`, network or model calls. It MUST NOT ask a god to judge the
same past event again. First mismatch reports request, assigned tick, graph or
standing revision, patron/batch/base ID, candidate/fallback hash and first
differing command/event/graph/standing/state root as
`NONDETERMINISTIC_RESULT`.

Generated content never writes authored bundles or packages. A future export may create a bounded `AgentChangeSet`, but automatic source mutation is outside this SPEC.

## Security, privacy and budgets

- Narrative snapshot is capability-filtered and default-redacted; raw save, secrets and unrestricted player text are forbidden.
- Remote provider use remains default-deny and requires existing project/user grant. Revocation prevents new requests and selects local/template fallback.
- Prompt/tool injection has no mutable tool surface. Unknown tool/function output is rejected before command construction.
- Candidate parsing, graph validation and staging use bounded allocation, recursion, node/edge and operation budgets.
- `ai-host` compute is outside authoritative tick budget. Request construction, result validation and commit use existing ADR-016 budget categories; no untracked tick work is allowed.
- Pending request count, due validation work and fallback work use SPEC-23 finite queues. Mandatory due work cannot be silently dropped.
- A divine request contains only the patron's epistemically authorized fact
  ceiling. Prompt injection cannot grant hidden facts, change pantheon edges,
  lift sanction prerequisites or create new intervention capabilities.
- Per-batch limits are 32 eligible gods, 32 selected per-god results, 64 total
  standing operations, 32 typed interventions and 16 sponsored quest
  opportunities. Larger semantic sets partition deterministically by root
  event and patron ID only when authored policy proves there is no cross-part
  edge. Project/content validation MUST reject a patron/pantheon policy whose
  possible eligible set exceeds the limit without such a proven partition;
  runtime still rejects an incompatible oversized batch before any mutation.

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
| `NARRATIVE_BOUNDARY_CLOSED` | Discard a completion assigned after the recorded boundary-closing ingress batch; never move/reopen the boundary or replace its selected fallback |
| `NARRATIVE_RESULT_COLLISION` | Fail closed for conflicting bytes under one result identity; no retry-to-green |
| `NARRATIVE_CONTENT_MISSING` | Fail load before world mutation; preserve previous save generation |
| `DIVINE_FACT_NOT_AUTHORIZED` | Reject the affected patron candidate; reveal no hidden fact and use only that patron's declared template fallback |
| `DIVINE_CANDIDATE_INVALID` | Reject malformed category/intervention/citation; keep other patron candidates and substitute the affected patron fallback before batch resolution |
| `DIVINE_BATCH_STALE` | Reject the whole batch; publish no standing, covenant, effect, quest or event change |
| `DIVINE_STANDING_OVERFLOW` | Reject the complete batch; never clamp or partially update another god |
| `DIVINE_SANCTION_NOT_JUSTIFIED` | Reject the complete affected patron candidate and select its one canonical template candidate; never retain the rejected category/text |
| `DIVINE_COVENANT_CONFLICT` | Require explicit player renunciation/confirmation; never silently replace an active covenant |
| `DIVINE_OFFER_STALE` | Reject stale/terminal offer transition and return the retained receipt/state without effect, covenant or cooldown change |
| `DIVINE_OFFER_TRANSITION_INVALID` | Reject an unknown/illegal offer or covenant edge atomically; preserve prior standing and offer history |
| `PANTHEON_EDGE_TARGET_NOT_ELIGIBLE` | Do not apply the edge or reveal the root fact; a future authorized observation may open a new hook |
| `PANTHEON_REACTION_DEPTH_EXCEEDED` | Commit the authored no-further-reaction closure and open no recursive standing-change hook |
| `DIVINE_PANTHEON_WORLD_MISMATCH` | Reject project/save activation before world publication; use the original exact content closure or start a new world |
| `NONDETERMINISTIC_RESULT` | Stop replay/product check at first differing compare point; never regenerate model output |

Process absence/crash/bad protocol, queue/resource denial, cancellation, duplicate/late result and model rejection MUST NOT block simulation or repeat a committed graph revision.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| `QUEST-OPPORTUNITY-P1` | `next check QUEST-OPPORTUNITY-P1 --scenario quest-opportunity-contract-v1 --cases 10000` | NPC intents and world needs create no Quest until valid admission; all four disclosure channels resolve to the same Quest ID/terms, stale activation and invalid prior facts/rewards reject atomically, and accepted assessment/reward revisions remain frozen. Keep ordinary AI/world behavior or the prior Quest revision on rejection. |
| `QUEST-AUTONOMY-P1` | `next check QUEST-AUTONOMY-P1 --scenario autonomous-quest-lifecycle-v1 --quests 1000 --ticks 10000 --seeds 100` | Latent/offered/accepted quests produce exact stepped/bulk and residency-tier outcomes; decline cooldown, prior-fact policy, terminal policy and deadlines behave identically across repeats and no continuation is lost or duplicated. Default to `PlayerProtected`, `ProspectiveOnly` and stepped fixed-tick advance. |
| `NARRATIVE-GRAPH-P1` | `next check NARRATIVE-GRAPH-P1 --scenario narrative-graph-admission-v1 --candidates 10000` | Stale, oversized, anchor, slot, cycle, depth, primitive and capability violations reject atomically; accepted graph hashes are exact. Keep the prior graph and use `TemplateNarrativeDirector` on failure. |
| `NARRATIVE-ASYNC-P1` | `next check NARRATIVE-ASYNC-P1 --scenario narrative-director-faults-v1 --injections 1000` | Early completion never commits before the fixed world boundary; completion assigned in the boundary-closing ingress batch is eligible and later completion is rejected. Absence, crash, timeout, cancellation, reordering, duplicate, restart and collision never stall a tick, mutate directly or duplicate a revision; disable the `ai-host` route and use the deterministic template fallback. |
| `NARRATIVE-REPLAY-P1` | `next check NARRATIVE-REPLAY-P1 --scenario narrative-save-replay-v1 --runs 100 --modes game,headless,capture-worker` | Save/restart at every async and commit boundary reproduces exact request/candidate/command/outcome/event/graph/state roots without network or model calls; reject divergent load/replay and retain the previous save/candidate. |
| `DIVINE-JUDGMENT-P1` | `next check DIVINE-JUDGMENT-P1 --scenario divine-standing-and-interventions-v1 --cases 10000` | Epistemic filters, qualitative categories, exact policy mapping, whole-candidate fallback, warning/sanction prerequisites, every offer/covenant transition and intervention budget accept/reject exactly; stale/idempotent offer commands repeat no effect and UI exposes bands/reasons but no raw values. |
| `PANTHEON-CONFLICT-P1` | `next check PANTHEON-CONFLICT-P1 --scenario pantheon-conflict-matrix-v1 --cases 10000 --permutations all` | One root act can raise one standing and lower another; authorized directed spillover is additive in canonical order, a hidden/ineligible target receives no edge mutation, incompatible covenants and boon/counterquest rules resolve from one base snapshot into one exact atomic vector under every patron/worker order. Reject the whole stale/overflowing batch and preserve all prior standings. |
| `DIVINE-ASYNC-P1` | `next check DIVINE-ASYNC-P1 --scenario divine-director-faults-v1 --batches 1000 --faults timeout,crash,reorder,duplicate,late,collision,restart` | Zero early commits, tick stalls, council merges, partial standing updates or duplicate interventions; each missing/invalid patron selects only its own deterministic fallback at the exact fixed boundary. |
| `DIVINE-REPLAY-P1` | `next check DIVINE-REPLAY-P1 --scenario divine-save-replay-v1 --runs 100 --modes game,headless,capture-worker` | Save/restart at hook/base/partial-result/fixed-boundary/offer/resolution/cross-context commit points reproduces candidate selection, fallback, standing/intervention/event/state roots exactly with zero model/network calls. |

## Requirements

| ID | Requirement |
|---|---|
| `REQ-148` | An NPC intent or world need becomes a durable latent Quest only after bounded deterministic opportunity admission; disclosure, activation, decline and autonomous progression of latent/offered/accepted instances use the common RPG command path and explicit policies. |
| `REQ-149` | Committed outcomes open bounded causal hooks/chains without modifying mandatory narrative anchors. |
| `REQ-150` | Narrative Director receives a bounded immutable snapshot and submits only atomic validated graph/quest candidates; it cannot disclose a quest, assign final challenge/XP or mutate RPG state directly. |
| `REQ-151` | Async completion, fallback, save/replay and composition-root parity require no regeneration or network. |
| `REQ-152` | Every player/god relationship uses one RPG-owned `DivineStandingV1`; patron knowledge is capability-filtered by authored epistemic policy and player UI exposes only qualitative bands plus committed reasons. |
| `REQ-153` | Every eligible god judges one root event through an independent request over the same immutable batch base; deterministic pantheon resolution may raise one standing and lower another and commits the complete vector atomically. |
| `REQ-154` | LLM selects only categorical judgement and eligible authored intervention; exact deltas/effects, covenant conflicts and minor/major sanction prerequisites remain deterministic validated policy. |
| `REQ-155` | Divine completions are recorded external inputs with per-god deterministic fallback, bounded reaction chains and exact save/replay behavior that never reissues a model or network call. |

## Failure paths

| ID | Trigger | Required result |
|---|---|---|
| `FAIL-062` | Invalid, stale, unverifiable, oversized, reward-invalid or guardrail-breaking opportunity/graph candidate | Change no graph, Quest, hook, reward or event state; retain ordinary world/agent state or the prior graph and deterministic fallback. |
| `FAIL-063` | Director absent, late, conflicting or restarted | Never block a tick or duplicate a commit; select the deterministic fallback at the declared boundary. |
| `FAIL-064` | Divine candidate cites hidden/stale facts, chooses forbidden intervention, violates covenant/sanction policy or exceeds bounds | Reject that candidate before mutation, select only its patron fallback and reveal/commit no unauthorized fact or effect. |
| `FAIL-065` | Pantheon batch is stale, overflowing, cyclic/deep, completion-order dependent or fails cross-context commit | Reject the complete batch with no partial standing/quest/effect/event change and consume no hook, offer or graph slot; preserve prior standings and use the declared bounded closure/fallback path. |

Эти schemas и scenarios являются Accepted architecture contracts. Их статус не
означает, что runtime уже реализован или что перечисленные ProductCheck
scenarios имеют результат `PASS`.
