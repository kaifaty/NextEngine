# SPEC-06: AI agents, perception и memory

| Поле | Значение |
|---|---|
| ID | SPEC-06 |
| Статус | Accepted |
| Версия | 1.16 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-056](adr/056-deterministic-strategic-agent-and-belief-driven-goap.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md) |
| Заменяет | SPEC-06 1.15; recognizes the current R4b population substrate without promoting R4c cognition |

## Source of truth и ownership

RPG aggregates остаются authoritative вне AI. Agent Runtime владеет working
plan, attention, habits и deterministic decision state. Current World Services
calendar/population state is an immutable input boundary, but no current AI
contract consumes it; beliefs, perception and bounded GOAP remain Proposed R4c
scope in SPEC-20/32. Immutable `AgentArchetypeDefinition` принадлежит cooked content;
Agent Runtime интерпретирует его, но не изменяет. Memory Service владеет durable
episodic/semantic records и relationship recollections/indexes как versioned
save segment; текущие relationship dimensions, quest/dialogue states,
commitments и `SkillProficiency` принадлежат RPG Framework. `ActivePolicyRoute`,
motor transition и joint actions принадлежат Motor Runtime. `ai-host` caches,
prompts и vendor sessions не являются source of truth и могут быть
удалены/rebuilt.

## Public boundary и data flow

Public AI boundary ограничен `AgentArchetypeDefinition`, `PerceptionFrame`,
`MotorCapabilityView`, `AgentIntent`, memory proposal/query values, `ai-host`
handshake/messages и command rejection codes. Vendor
request/session/tokenizer/vector-index types запрещены. Нормативный поток:
`immutable archetype + gameplay facts + motor capabilities → perception/memory
view → optional ai-host proposal → AgentIntent → deterministic
validation/planning → WorldCommand или rejection`.

## Иерархия принятия решений

```text
biography + validated memories + current perception
                ↓ optional ai-host interpretation/dialogue intent
              AgentIntent
                ↓ deterministic drives + Utility + bounded GOAP planner
        strategic goal / executable plan
                ↓ tactical AI
          PhysicalAvatarIntent / command candidates
                ↓ motor controller (no LLM)
              physics
```

Optional LLM улучшает формулировку, long-horizon suggestions и речь, но не заменяет deterministic rule/planner safety. Tactical и motor layers MUST иметь bounded execution time и не ждать `ai-host`.

If human/LLM text requests a physical action, the optional language boundary
must first compile it into typed `AgentIntent` facts: stable entity/skill/
primitive IDs, reference frames, numeric targets, masks, constraints, expiry
and provenance. Raw text, tokens and language embeddings end there and are not
copied into `PhysicalAvatarIntent`, contact/chunk planning, motor observation,
policy state, safety or Physics. Ambiguous compilation is a typed rejection or
clarification, not a free-form motor fallback.

## Agent archetypes и habits

`AgentArchetypeDefinition` является generic immutable asset и MUST содержать behavior traits, routine templates, sensory profile, tactical preferences, memory/relationship priors и initial skill loadout. Definition не содержит motor actions, joint targets, damage formulas или mutable proficiency. Content package может добавить stalking, circling, guarding territory, retreat-when-wounded и preferred-attack rules без native AI code.

Habit/routine/tactical layer создаёт только `AgentIntent` либо validated `InvokeAbility` candidate. Planner читает `MotorCapabilityView` из SPEC-14 и обязан трактовать состояния так:

- `Unavailable` — способность исключается из executable plan;
- `NoviceFallback` — допустима только объявленная безопасная novice/generic route с её performance envelope;
- `PendingActivation` — planner может ждать, выбрать альтернативу или отменить intent, но не посылать joint action;
- `Active` — intent может быть передан Motor Runtime при выполнении остальных preconditions.

Optional learned tactical policy остаётся за границей `AgentIntent`, имеет versioned input/output schema и deterministic Strategic Agent fallback по ADR-056. Отключение `ai-host` или learned behavior adapter MAY ухудшить разнообразие/оптимальность, но MUST NOT менять command validation, motor safety или authoritative gameplay correctness.

### Proposed hierarchical behavior specialization

[ADR-050](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md),
[SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md)
и
[SPEC-33](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md)
предлагают optional R8 specialization: отдельные strategic/tactical learned
policies, intention/recurrent-state lifecycle и offline training/deployment.
Все три документа имеют статус `Proposed`, не добавляют current public schema
или runtime obligation и не заменяют описанные выше perception, memory,
`AgentIntent`, Utility + bounded GOAP и authored tactical fallback. Их promotion
возможна только с optional production consumer и проходящими ProductCheck по
ADR-046; она не является условием R4 или v1.

## Perception contract

Agent получает только capability-filtered `PerceptionFrame`: observer PersistentId, tick, visible/audible/sensed facts, uncertainty, source/occlusion tags и stable fact IDs. Physics/render internals и hidden world state не раскрываются. Perception systems вычисляют gameplay visibility отдельно от renderer culling. Facts имеют TTL/revision; stale fact явно помечается.

## AgentIntent → WorldCommand

`AgentIntent` MUST содержать protocol/schema version, intent ID, agent PersistentId, creation/expiry tick, intent kind, referenced facts/targets, confidence, justification provenance и optional dialogue candidate. Intent является untrusted proposal.

Validation pipeline:

1. protocol/schema/model provenance check;
2. agent identity, capability и current state check;
3. target/fact freshness и reachability precheck;
4. RPG rule, resource, cooldown и narrative constraint check;
5. planner decomposes allowed intent в bounded actions;
6. each mutation becomes canonical WorldCommand for future tick;
7. rejected intent gets stable reason and fallback plan.

Health, inventory, quest, faction, relationship и world state MUST NOT
изменяться через memory/intent payload напрямую.

## Mechanic affordances

Agent planner MUST получать granted planner-visible `MechanicAffordance` catalog из SPEC-13, а не hardcoded список combat/magic actions. Utility + bounded GOAP layer использует declared preconditions, target mode, cost/time/risk и bounded expected outcome; actual availability и effect всегда повторно проверяются WorldCommand/Mechanics Runtime. Новая package ability с valid affordance становится доступна NPC без private AI integration. `ai-host` может предложить affordance, но не подменяет deterministic validation или execution.

## Offline-first behavior

Для каждого AI role project MUST предоставить deterministic fallback:
Utility + bounded GOAP planning, authored dialogue line/template, rule-based memory
retrieval и tactical controller. При отсутствии `ai-host` agent продолжает
authored routines, combat/traversal, interactions и quest dialogue. Допустима
только разница качества/разнообразия текста, голоса и high-level proposal.

`ai-host` IPC использует version handshake, request/deadline, cancellation, content/model hash, idempotency key и response provenance. Default deadline: dialogue proposal 2 s, background intent 5 s; project MAY уменьшить, но не блокировать tick. Late response discarded. После restart runtime resends только uncommitted idempotent requests; committed intent IDs хранятся в rolling dedupe ledger.

## Memory model

| Тип | Назначение | Retention/запись |
|---|---|---|
| Working | текущая цель, plan, attention, short context | runtime-owned, bounded; checkpointed только если нужен resume |
| Episodic | произошедшие события с time/place/participants/confidence | создаётся из DomainEvent через policy; append + consolidation |
| Semantic | подтверждённые/выведенные facts и concepts | provenance + confidence + contradiction links |
| Relationship | observations/recollection о directed agent↔agent events и изменениях | current dimensions принадлежат RPG; memory создаётся из committed DomainEvent |
| Narrative | retrieval/index recollection о quest/dialogue commitments, promises, unresolved hooks | authoritative commitments принадлежат RPG; свободный LLM текст не authority |

Memory proposal содержит source event/fact IDs и не может retroactively менять DomainEvent или authoritative RPG field. Retention quotas и compaction deterministic относительно ordered records. SQLite — `Proposed` storage backend; schema/domain contracts engine-owned. Embedding index является rebuildable cache и optional: canonical memory доступна без embeddings.

## `ai-host` security boundary

Process получает минимальный serialized context, не filesystem paths/saves/credentials. Model adapters network-disabled default; remote provider требует explicit user/project capability и disclosure. Prompt/model output маркируются untrusted, ограничиваются size/UTF-8/schema и не выполняются как Luau/Wasm/code.

## Failure semantics

- Process absent/crash/timeout/protocol mismatch → fallback, request diagnostic, no tick stall.
- Malformed/unsafe intent → stable rejection, planner fallback; repeated failures circuit-break adapter.
- Invalid model/embedding file → quarantine, no load; canonical memory работает.
- Memory storage transaction failure → gameplay command may commit, но corresponding memory DomainEvent остаётся в durable retry queue; relationship/narrative authoritative changes находятся в RPG save, не теряются.
- Retrieval overload → bounded top-k/time budget and rule-based recent/relevant fallback.
- Missing/incompatible AgentArchetype или learned behavior adapter → schema diagnostic и deterministic Utility + bounded GOAP fallback; Agent Runtime не синтезирует motor actions.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| AI-01 | Same 100 scenarios with `ai-host` disabled | Gameplay and quest outcomes remain correct without blocking a tick; use the built-in planner/dialogue fallback. |
| AI-02 | 1,000 kill/restart/timeout/malformed-result injections | No host crash or duplicate committed command; fallback is selected no later than one gameplay tick after the deadline signal. |
| AI-03 | Adversarial intents and stale facts | Every forbidden or stale mutation is rejected and no direct state write is possible. |
| AI-04 | **R4b substrate observation; R4c AI gate remains deferred.** Run current `r4-100npc.v1` for exact population due/query/no-starvation facts; it performs no cognition. | Population cadence and graph queries must retain exact counts and zero defer/drop/starvation. The separate planning p95 ≤1,250 us / p99 ≤1,500 us assertion remains typed `NOT_RUN` until an R4c production cognition consumer exists; R4b timings cannot satisfy it. |
| MEMORY-P1 | SQLite candidate crash, compaction and migration corpus | Every committed record is recovered, canonical queries match, and corruption fails closed; fall back to the append-only log with compacted indexes. |
| AI-05 | Restart/save/load with and without embeddings | Authoritative memory, relationship and narrative state is identical; embeddings may be rebuilt or disabled. |
| AI-06 | Package affordance discovery | Every granted planner-visible ability is discoverable, new fixture abilities need no AI code change, and invalid/stale affordances are rejected; otherwise mark the ability manual-only. |
| AI-07 | Agent archetype habits and motor capability boundary | Habits emit only `AgentIntent`/`InvokeAbility`; every capability state selects its deterministic authored planner fallback without direct motor or gameplay mutation. |

## Future narrative role

Narrative-director and divine-agent roles remain Proposed intent in SPEC-31.
They define no current AI message, schedule, fallback, save/replay or ProductCheck
obligation. Any future model output remains an untrusted proposal through the
same validated command boundary.

## Deterministic Strategic Agent boundary

ADR-056 accepts the architecture-level epistemic boundary and Utility + bounded
GOAP baseline. Detailed Epistemic View, Drive View, goal/plan/task lifecycle,
structured NPC speech acts and Decision Trace remain Proposed in SPEC-32 until
their R4c/R4d production consumers exist. Perception and Memory Service expose
only immutable revision-bound facts/recollections; they do not expose hidden
world truth or mutate RPG/World state. Learned strategic/tactical policies are
optional R8 quality adapters and do not block R4 or v1.
