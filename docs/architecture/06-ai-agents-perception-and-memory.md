# SPEC-06: AI agents, perception и memory

| Поле | Значение |
|---|---|
| ID | SPEC-06 |
| Статус | Accepted |
| Версия | 1.3 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-23 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md) |
| Заменяет | отсутствует |

## Source of truth и ownership

RPG/world state остаётся authoritative вне AI. Agent Runtime владеет working plan, attention, активными habits и deterministic decision state. Immutable `AgentArchetypeDefinition` принадлежит cooked content registry; Agent Runtime интерпретирует его, но не изменяет. Memory Service владеет durable episodic/semantic records и relationship/narrative recollections/indexes как versioned save segment; текущие relationship dimensions, quest/dialogue states, commitments и `SkillProficiency` принадлежат RPG Framework. `ActivePolicyRoute`, motor transition и joint actions принадлежат Motor Runtime. `ai-host` caches, prompts и vendor sessions не являются source of truth и могут быть удалены/rebuilt.

## Public boundary и data flow

Public AI boundary ограничен `AgentArchetypeDefinition`, `PerceptionFrame`, `MotorCapabilityView`, `AgentIntent`, memory proposal/query values, `ai-host` handshake/messages и command rejection codes. Vendor request/session/tokenizer/vector-index types запрещены. Нормативный поток: `immutable archetype + gameplay facts + motor capabilities → perception/memory view → optional ai-host proposal → AgentIntent → deterministic validation/planning → WorldCommand или rejection`; direct reverse mutation edge отсутствует.

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

Health, inventory, quest, faction, relationship и world state MUST NOT изменяться через memory/intent payload напрямую.

## Mechanic affordances

Agent planner MUST получать granted planner-visible `MechanicAffordance` catalog из SPEC-13, а не hardcoded список combat/magic actions. Utility/HTN layer использует declared preconditions, target mode, cost/time/risk и bounded expected outcome; actual availability и effect всегда повторно проверяются WorldCommand/Mechanics Runtime. Новая package ability с valid affordance становится доступна NPC без private AI integration. `ai-host` может предложить affordance, но не подменяет deterministic validation или execution.

## Offline-first behavior

Для каждого AI role project MUST предоставить deterministic fallback: utility/HTN planning, authored dialogue line/template, rule-based memory retrieval и tactical controller. При отсутствии `ai-host` agent продолжает schedules, combat/traversal, interactions и quest dialogue. Допустима только разница качества/разнообразия текста, голоса и high-level proposal.

`ai-host` IPC использует version handshake, request/deadline, cancellation, content/model hash, idempotency key и response provenance. Default deadline: dialogue proposal 2 s, background intent 5 s; project MAY уменьшить, но не блокировать tick. Late response discarded. После restart runtime resends только uncommitted idempotent requests; committed intent IDs хранятся в rolling dedupe ledger.

## Memory model

| Тип | Назначение | Retention/запись |
|---|---|---|
| Working | текущая цель, plan, attention, short context | runtime-owned, bounded; checkpointed только если нужен resume |
| Episodic | произошедшие события с time/place/participants/confidence | создаётся из DomainEvent через policy; append + consolidation |
| Semantic | подтверждённые/выведенные facts и concepts | provenance + confidence + contradiction links |
| Relationship | evidence/recollection о directed agent↔agent events и изменениях | current dimensions принадлежат RPG; memory создаётся из committed DomainEvent |
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

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback |
|---|---|---|---|---|
| AI-01 | same 100 scenarios with ai-host disabled | 100% gameplay/quest completion correctness; no blocked tick > gameplay budget | replay + outcome report | built-in planner/dialogue fallback fix |
| AI-02 | kill/restart/timeout/malformed injection | 1 000 injections; 0 crash, 0 duplicate committed command, fallback selected ≤1 gameplay tick after deadline signal | fault report | circuit-break ai-host |
| AI-03 | adversarial intents/fact staleness | 100% forbidden/stale mutations rejected, 0 direct state writes | validator audit | release block |
| AI-04 | ADR-016 deterministic 100-NPC workload | весь due agent-planning work, queue handling и deterministic deferral помещаются в exclusive row p95 ≤1 250 us / p99 ≤1 500 us; exact membership/phase/due trace; no starvation, dropped work, LLM wait or unowned span | GameplayBudgetMatrix/workload hashes, per-tick due/queue/span trace | reduce deterministic planning cadence/LOD; integrated PERF-01 remains blocking |
| MEMORY-P1 | SQLite candidate crash/compaction/migration corpus | 100% committed records recovered; canonical query parity exact; 1M records, indexed query p95 ≤20 ms; corruption fail-closed | DB fixtures/report | append-only log + compacted indexes |
| AI-05 | restart save/load with and without embeddings | authoritative memory/relationship/narrative hashes exact; outputs remain schema-valid | save/replay report | rebuild/disable embeddings |
| AI-06 | package affordance discovery | все planner-visible abilities exact MechanicsLock доступны по capability; added fixture ability используется без AI code change; invalid/stale affordance 100% rejected | registry/planner/replay report | manual-only ability/fix package |
| AI-07 | AgentArchetype habits + motor capability boundary | reference habits produce only AgentIntent/InvokeAbility; 0 direct motor/gameplay mutation; all four capability states select declared deterministic outcomes with ai-host/adapter disabled | archetype schema audit, planner traces, replay report | disable behavior adapter; authored utility/HTN fallback |
