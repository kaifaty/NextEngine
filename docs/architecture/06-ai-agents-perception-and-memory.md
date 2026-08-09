# SPEC-06: AI agents, perception и memory

| Поле | Значение |
|---|---|
| ID | SPEC-06 |
| Статус | Accepted |
| Версия | 1.13 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | SPEC-06 1.12; clarifies that the unimplemented 100-NPC recipe is deferred and creates no current ProductCheck |

## Source of truth и ownership

RPG aggregates остаются authoritative вне AI. Agent Runtime владеет working
plan, attention, habits и deterministic decision state. Future
calendar/population state не имеет current AI contract и остаётся Proposed в
SPEC-20. Immutable `AgentArchetypeDefinition` принадлежит cooked content;
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
                ↓ deterministic policy + utility/HTN planner
        strategic goal / executable plan
                ↓ tactical AI
          PhysicalAvatarIntent / command candidates
                ↓ motor controller (no LLM)
              physics
```

Optional LLM улучшает формулировку, long-horizon suggestions и речь, но не заменяет deterministic rule/planner safety. Tactical и motor layers MUST иметь bounded execution time и не ждать `ai-host`.

## Agent archetypes и habits

`AgentArchetypeDefinition` является generic immutable asset и MUST содержать behavior traits, routine templates, sensory profile, tactical preferences, memory/relationship priors и initial skill loadout. Definition не содержит motor actions, joint targets, damage formulas или mutable proficiency. Content package может добавить stalking, circling, guarding territory, retreat-when-wounded и preferred-attack rules без native AI code.

Habit/routine/tactical layer создаёт только `AgentIntent` либо validated `InvokeAbility` candidate. Planner читает `MotorCapabilityView` из SPEC-14 и обязан трактовать состояния так:

- `Unavailable` — способность исключается из executable plan;
- `NoviceFallback` — допустима только объявленная безопасная novice/generic route с её performance envelope;
- `PendingActivation` — planner может ждать, выбрать альтернативу или отменить intent, но не посылать joint action;
- `Active` — intent может быть передан Motor Runtime при выполнении остальных preconditions.

Optional learned tactical policy остаётся за границей `AgentIntent`, имеет versioned input/output schema и deterministic utility/HTN fallback. Отключение `ai-host` или learned behavior adapter MAY ухудшить разнообразие/оптимальность, но MUST NOT менять command validation, motor safety или authoritative gameplay correctness.

### Proposed hierarchical behavior specialization

[ADR-050](adr/050-hierarchical-npc-cognition-and-learned-behavior-policy-boundary.md),
[SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md)
и
[SPEC-33](33-behavior-policy-training-evaluation-and-deployment-lifecycle.md)
предлагают future R4 specialization: отдельные strategic/tactical learned
policies, intention/recurrent-state lifecycle и offline training/deployment.
Все три документа имеют статус `Proposed`, не добавляют current public schema
или runtime obligation и не заменяют описанные выше perception, memory,
`AgentIntent`, utility/HTN и authored tactical fallback. Их promotion возможна
только вместе с production consumer и проходящими ProductCheck по ADR-046.

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

Agent planner MUST получать granted planner-visible `MechanicAffordance` catalog из SPEC-13, а не hardcoded список combat/magic actions. Utility/HTN layer использует declared preconditions, target mode, cost/time/risk и bounded expected outcome; actual availability и effect всегда повторно проверяются WorldCommand/Mechanics Runtime. Новая package ability с valid affordance становится доступна NPC без private AI integration. `ai-host` может предложить affordance, но не подменяет deterministic validation или execution.

## Offline-first behavior

Для каждого AI role project MUST предоставить deterministic fallback:
utility/HTN planning, authored dialogue line/template, rule-based memory
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
- Missing/incompatible AgentArchetype или learned behavior adapter → schema diagnostic и authored utility/HTN fallback; Agent Runtime не синтезирует motor actions.

## Product checks

| Check ID | Scenario / command | Expected behavior / fallback |
|---|---|---|
| AI-01 | Same 100 scenarios with `ai-host` disabled | Gameplay and quest outcomes remain correct without blocking a tick; use the built-in planner/dialogue fallback. |
| AI-02 | 1,000 kill/restart/timeout/malformed-result injections | No host crash or duplicate committed command; fallback is selected no later than one gameplay tick after the deadline signal. |
| AI-03 | Adversarial intents and stale facts | Every forbidden or stale mutation is rejected and no direct state write is possible. |
| AI-04 | **Deferred R4b recipe; no current gate.** ADR-016 deterministic 100-NPC workload after its production population consumer exists | Due planning stays within p95 ≤1,250 us / p99 ≤1,500 us with deterministic deferral and no starvation, dropped work, LLM wait or unowned span; reduce planning cadence/LOD if needed. Until then the workload remains typed `NOT_RUN`/unavailable and this row creates no completion claim. |
| MEMORY-P1 | SQLite candidate crash, compaction and migration corpus | Every committed record is recovered, canonical queries match, and corruption fails closed; fall back to the append-only log with compacted indexes. |
| AI-05 | Restart/save/load with and without embeddings | Authoritative memory, relationship and narrative state is identical; embeddings may be rebuilt or disabled. |
| AI-06 | Package affordance discovery | Every granted planner-visible ability is discoverable, new fixture abilities need no AI code change, and invalid/stale affordances are rejected; otherwise mark the ability manual-only. |
| AI-07 | Agent archetype habits and motor capability boundary | Habits emit only `AgentIntent`/`InvokeAbility`; every capability state selects its deterministic authored planner fallback without direct motor or gameplay mutation. |

## Future narrative role

Narrative-director and divine-agent roles remain Proposed intent in SPEC-31.
They define no current AI message, schedule, fallback, save/replay or ProductCheck
obligation. Any future model output remains an untrusted proposal through the
same validated command boundary.
