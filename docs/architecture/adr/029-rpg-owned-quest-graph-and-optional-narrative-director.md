# ADR-029: RPG-owned quest graph и optional narrative director

| Поле | Значение |
|---|---|
| ID | ADR-029 |
| Статус | Superseded by ADR-046; future intent is Proposed |
| Версия | 1.0 |
| Дата решения | 2026-07-26 |
| Последняя проверка | 2026-07-26 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-11](../11-security-licensing-and-governance.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-026](026-deterministic-work-resource-and-streaming-admission.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | fully superseded by [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |

## ADR-030 scope

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses ранней proposal revision этого ADR. Quest/graph authority,
bounded untrusted director proposals, deterministic fallback и
replay-without-regeneration semantics сохраняются.

## История принятия

ADR-029 принят как authority decision для autonomous Quest и narrative
director. Он не выбирает LLM/provider и не утверждает runtime implementation;
accepted detailed contract находится в SPEC-31.

## Контекст

Accepted architecture уже разделяет RPG authority, deterministic population/time advance, optional `ai-host`, async completion staging и replay. Однако она не отвечает:

- является ли частное намерение NPC квестом и кто решает, что его можно делегировать игроку;
- создаёт ли квест реплика NPC/вопрос игрока или отдельный системный admission;
- как direct, solicited, contextual и public discovery приводят к одному Quest contract;
- когда фиксируются сложность, обещанная награда, системный XP и признание уже совершённых действий;
- существует ли offered quest до принятия игроком как authoritative instance;
- может ли accepted quest завершиться по committed событиям мира или deadline;
- как failure/expiry становится причинным входом следующей quest chain;
- где хранится mutable dynamic quest graph;
- может ли LLM добавлять graph topology и prose без превращения в RPG owner;
- как live nondeterministic response становится replayable external input;
- что происходит с generation, когда model/network отсутствует.
- как тот же director boundary может исполнять authored роли богов, не делая
  модель владельцем divine standing, санкций или наград;
- как независимые конфликтующие боги оценивают один поступок из одного
  snapshot, а затем атомарно применяют противоположные последствия.

Без решения quest opportunity может жить только в UI/script, world service может начать менять RPG state, provider thread — стать вторым graph store, а replay — повторно вызвать модель и получить другое прошлое.

## Решение

1. **Intent is not a quest.** Private `AgentIntent`, committed world need, dialogue line and LLM output are inputs/proposals. A Quest begins only when RPG deterministic admission accepts a bounded `QuestCandidateV1`.
2. **Quest exists before disclosure and acceptance.** Successful admission creates one RPG-owned `QuestInstance(Latent)`. `Latent`, `Offered`, `Accepted` and `Resolved` are engagement states of that aggregate.
3. **Conversation reveals; it does not create.** `Direct`, `Solicited`, `Contextual` and `Public` are closed disclosure channels over already admitted opportunities. All commit `Offered` through the same revision-checked RPG command.
4. **Acceptance is system activation.** `AcceptQuest` revalidates current participants, availability, deadline, conflicts and eligibility, then freezes term variant, challenge assessment and reward-contract revision atomically.
5. **Decline is not failure.** It records bounded offer disposition/cooldown while the Quest continues under its autonomy policy. Permanent withdrawal requires an explicit outcome.
6. **Difficulty and rewards remain system-owned.** NPC/LLM may propose bounded terms, but registered validators compute challenge/system progression reward. Promised world reward, system XP and world consequences are separate contracts. Prior actions count only under explicit fact-recognition policy.
7. **Autonomy is explicit per quest.** `PlayerProtected`, `WorldReactive` and `DeadlineBound` define background/terminal authority. Default legacy migration is `PlayerProtected`; new definitions must declare policy.
8. **Outcomes are causal facts.** Success, failure, expiry, cancellation and supersession become facts only after atomic RPG commit and may open bounded `NarrativeHookV1`.
9. **RPG owns the graph.** Current `QuestGraphRevisionV1`, slot occupancy, Quest instances and outcomes belong to RPG Framework. World Services owns only its facts; Agent Intelligence owns only intent/planning and request/candidate routing.
10. **Authored anchors remain mandatory.** LLM/template additions attach only to `NarrativeExtensionSlotV1`. They cannot alter anchors, protected facts, mandatory reachability or endings.
11. **LLM output is external untrusted input.** Optional `ai-host` returns a bounded typed candidate. It receives no mutable tools and cannot emit a trusted command, authoritative ID, XP grant or transaction plan.
12. **Admission is atomic.** `AdmitQuestOpportunity` admits one opportunity. `AdmitQuestGraphRevision` may atomically admit a generated graph batch, but uses the same per-opportunity validator, causal identity path and bounded Runtime-owned `CrossContextTransactionPlanV1`.
13. **Fallback is productive at a fixed boundary.** Deterministic `TemplateNarrativeDirector` uses the same request, slots, policies, validators and command path and may create simpler chains. Results only stage before their `NarrativeDecisionBoundaryV1`; missing model selects fallback at that boundary rather than moving or blocking it.
14. **Replay never regenerates.** Live completion bytes/assignments and decision-boundary closure become recorded external inputs. Save/replay pins candidate and graph artifacts; replay makes zero model/network calls.
15. **Generated content is world-local.** Runtime cannot mutate project/package sources. Перенос результата в authored content выполняется отдельным inspectable `AgentChangeSet` вне runtime.
16. **No new technology decision.** The protocol is engine-owned. Exact local/remote model/provider remains Proposed and is selected only after relevant product checks.
17. **Gods are role instances, not a new authority.** Each authored god MAY use
    an independent `narrative-director` request constrained by its own
    epistemic policy and intervention catalog. LLM output remains a proposal.
18. **Pantheon conflict is resolved atomically.** All gods eligible for one root
    event read one immutable pre-decision base. A deterministic authored
    resolver combines personal judgments and directed pantheon/covenant
    spillover only for epistemically eligible targets into one atomic
    multi-standing/effect/quest command. Invalid per-god candidate is replaced
    wholly by its canonical template result; completion order cannot select a
    merge or commit boundary.
19. **Divine standing is RPG-owned.** The dedicated aggregate and
    supersession of ADR-020's closed list are isolated in ADR-031. This ADR does
    not reinterpret ordinary Relationship as divine reputation.

## Рассмотренные варианты

- **All quests always autonomous** — Rejected: authored main-story protection and player contract would be implicit and unsafe.
- **Every NPC intent becomes a quest** — Rejected: ordinary life and planner goals would flood the quest system and create false player contracts.
- **Dialogue line or “есть работа?” creates the quest** — Rejected: undiscovered opportunities could not live independently, wording/UI would gain domain authority and replay would depend on presentation.
- **Accepted quests always freeze** — Rejected: prevents a living world, deadlines and consequences after player commitment.
- **Create quest only on player acceptance** — Rejected: offered opportunities could not evolve, expire or resolve causally.
- **LLM/NPC assigns final difficulty and XP** — Rejected: untrusted subjective output would control authoritative progression and invite player-level/rewording exploits.
- **World Services owns quest progression** — Rejected: duplicates RPG authority and turns schedule/time into domain mutation.
- **LLM owns or directly patches graph/database** — Rejected: untrusted nondeterministic process becomes authoritative mutable source.
- **LLM may replace mandatory anchors/endings** — Rejected: network/model availability would change mandatory outcome and offline completeness.
- **LLM only writes prose** — Rejected as a product restriction: bounded typed graph proposals are allowed, while authority remains with validators.
- **No generation when ai-host is absent** — Rejected: every AI role needs an in-process deterministic fallback and the requested living-world capability must remain useful offline.
- **Regenerate on replay** — Rejected: provider/model/prompt drift would make past behavior unrecoverable.
- **Automatically save good quests into project assets** — Rejected: runtime would mutate source and bypass the explicit `AgentChangeSet` authoring boundary.
- **One model call acts as a council of gods** — Rejected: merges epistemic
  scopes and prevents independent per-god fallback and causal attribution.
- **Commit god responses as they arrive** — Rejected: provider/worker latency
  would change standing, later context and world outcome.
- **Salvage a valid category from an invalid intervention** — Rejected: the
  authoritative result would depend on validator repair policy rather than one
  recorded candidate or one canonical template candidate.

## Последствия

- Quest definitions and saves gain source/admission lineage, disclosure policy/history, offer cooldown, activation assessment/reward revision, engagement, autonomy, outcome, hook, graph-revision and causal metadata.
- Agent Intelligence gains bounded asynchronous request/candidate work but no RPG or graph mutation authority.
- NPCs may expose fewer or different opportunities as their intent/world facts evolve, while player inquiry remains a disclosure action rather than a content-generation switch.
- Authors define delegation/admission, disclosure, prior-fact, difficulty/reward and no-player policies with a small closed vocabulary instead of bespoke quest scripts.
- World advance gains exact quest/narrative decision boundaries and may perform more bounded stops during large time skips.
- Save closure grows with generated graph/text blobs; strict per-candidate and chain limits bound growth.
- Fresh LLM-assisted runs may differ in optional content, while replay of each recorded run remains exact.
- Authors must declare main-story anchors, extension slots, autonomy/deadline policies and template fallbacks.
- Graph validation and transaction planning consume existing ADR-016 subsystem budgets; no unaccounted span is introduced.
- Authored patron epistemic scopes, pantheon relations and intervention
  catalogs reuse the director protocol while final divine state belongs to the
  RPG extension defined by ADR-031.
- Fresh runs may receive different divine judgments; one recorded run preserves
  exact per-god candidates, fallback choices and the final atomic standing
  vector.

## Failure and fallback

- Invalid/stale/oversized/unsafe candidate is rejected as one unit; previous graph, Quest aggregates and open hooks remain unchanged.
- Missing/crashed/timed-out/restarted director selects `TemplateNarrativeDirector` at the exact fixed narrative decision boundary; early completion only stages.
- Conflicting result bytes for one request identity fail closed and cannot pass by retry.
- Missing generated artifact fails load before world mutation and preserves previous save generation.
- Replay divergence is `NONDETERMINISTIC_RESULT`; model regeneration is never a recovery strategy.
- A committed quest consequence is not silently rolled back. Recovery or compensation is a new causal command/chain.
- Invalid/late divine result falls back only for the affected god. A stale or
  invalid final pantheon batch commits no partial standing, covenant, effect or
  quest state.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| NPC intent/world need is created, updated or resolved before player contact | No Quest exists before admission; admitted latent opportunity follows source facts without waiting for dialogue | Keep it as ordinary AI/world state or resolve the admitted Quest through its authored policy |
| The same eligible opportunity is found by direct contact, inquiry, observation or public notice | Every channel discloses the same Quest ID and terms through the same RPG command; localized text has no authority | Reject stale/ineligible disclosure and retain the latent instance |
| Player accepts, declines or presents prior progress | Acceptance freezes validated challenge/reward terms; decline only applies cooldown; prior facts count only under explicit policy | Reject stale activation/fact and keep the previous Quest revision |
| `Latent`, `Offered`, `Accepted` и `Resolved` transitions проходят через authored autonomy policies | Quest aggregate, outcomes and hooks change atomically through RPG-owned commands; invalid transition mutates no state | Preserve the previous Quest aggregate and emit a stable rejection |
| Template or model candidate proposes bounded additions to declared extension slots | Anchors, protected facts, mandatory reachability and size limits remain valid; one graph revision publishes atomically | Reject the candidate and run `TemplateNarrativeDirector` |
| Director completes early, times out, crashes, restarts or returns conflicting bytes for one request | Early result never commits before the fixed boundary; tick loop does not block and request/assignment/boundary identity stays recorded | Select `TemplateNarrativeDirector` for missing/invalid work at that same boundary |
| Save/replay loads a run containing generated quest graph and text | Exact candidate and graph artifacts reconstruct the committed run with zero model or network calls | Fail closed before world mutation if a referenced artifact is missing |
| Several rival gods judge the same committed player act | Every god reads the same authorized base; only eligible targets receive directed spillover, invalid candidates are wholly replaced, and one exact cross-context standing/effect/quest vector commits at the fixed boundary under every completion permutation | Use per-god template results and preserve all prior standings on final batch rejection |

## Supersession

ADR-029 coexists with ADR-005, ADR-016, ADR-020, ADR-021, ADR-022, ADR-026,
ADR-030 and ADR-031 and does not supersede them. SPEC-31 owns detailed
schemas and diagnostics for this boundary. ADR-031 alone defines the
partial supersession needed to add the dedicated RPG-owned
`DivineStanding` aggregate.

Moving graph/Quest authority into World Services, AI/model/provider/project
source; treating intent/dialogue as a Quest; allowing NPC/LLM-assigned final XP,
direct mutation, model-required mandatory outcomes, wall-clock deadlines, replay
regeneration, unbounded graph growth or removal of authored anchors requires a
future superseding ADR with synchronized affected SPECs, contracts and
lightweight traceability.
