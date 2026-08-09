# Strategic Agent architecture integration review

| Поле | Значение |
|---|---|
| Статус | Ненормативный review record |
| Дата | 2026-08-09 |
| Исходник | `docs/Strategic Agent — архитектура NPC, цели, память, планирование и социальное поведение.md` |
| Размер исходника | 38,598 bytes |
| SHA-256 | `035F6008AE827AF1A46B9BEEBE2AAE0EE04ED91B64004AA20445A2C242A2F9B6` |
| Архитектурный результат | [ADR-056](../architecture/adr/056-deterministic-strategic-agent-and-belief-driven-goap.md), [SPEC-32](../architecture/32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md) |

## Назначение

Отчёт фиксирует полный перенос исходного ТЗ в normative architecture Next
Engine. Сам отчёт не задаёт product contract и не имеет приоритета над
Accepted SPEC/ADR. Исходный файл был untracked working material и удаляется
после проверки этой матрицы; его raw text не является второй authority.

Disposition означает:

- `adopted` — рекомендация перенесена как целевая семантика;
- `adapted` — намерение сохранено, но ownership/interface приведены к Next
  Engine;
- `deferred` — записано как future optional/breadth track, не R4/v1 gate;
- `rejected` — конкретная форма противоречит текущим границам, её задача решена
  другим способом.

## Матрица покрытия

| № | Раздел исходника | Disposition | Основной архитектурный anchor | Решение |
|---:|---|---|---|---|
| 1 | Цель системы | adopted | SPEC-32: цели и инварианты | Strategic Agent решает `что/почему`, owner skills — `как`. |
| 2 | Общая архитектура | adapted | SPEC-32: Strategic loop | Monolithic diagram разложена по existing owners и fixed stages. |
| 3 | Основные слои | adapted | SPEC-32: ownership | Слои сохранены как responsibilities, не обязательные crates. |
| 4 | Agent State | adapted | SPEC-32: state decomposition | Agent хранит только goal/plan/executive state; RPG/Memory/World fields не дублируются. |
| 5 | Identity | adopted | SPEC-32: ownership | Immutable cooked content + durable `PersistentId`. |
| 6 | Personality | adopted | SPEC-32: drives/goals | Bounded authored traits влияют на utility, но не являются mutation authority. |
| 7 | Needs | adapted | SPEC-32: Drive View | Pressure derived из owner resources; второй mutable needs store запрещён. |
| 8 | Goals | adopted | SPEC-32: goal lifecycle | Stable content-driven kinds, completion/failure state и cited beliefs. |
| 9 | Goal Generation | adopted | SPEC-32: goal lifecycle | Canonical candidates из drives, aspirations, commitments и emergencies. |
| 10 | Utility System | adopted | ADR-056; SPEC-32 | Modular fixed-point utility и canonical tie-break. |
| 11 | Goal Commitment / Inertia | adopted | ADR-056; SPEC-32 | Switch threshold, emergency bands и bounded suspended stack. |
| 12 | Aspirations | adopted | SPEC-32: goals | Long-horizon tendencies создают goals, но не actions. |
| 13 | Commitments | adapted | ADR-056: ownership; SPEC-19 boundary | RPG владеет committed obligation; Agent планирует его выполнение. |
| 14 | World Truth vs Beliefs | adopted | ADR-056; SPEC-32: Epistemic View | Planner не получает authoritative hidden state. |
| 15 | Belief | adopted | SPEC-32: knowledge/memory | Confidence, provenance, revision и contradiction state. |
| 16 | Knowledge и Memory должны быть разными | adopted | SPEC-32: knowledge/memory | Semantic beliefs и episodic recollections имеют разные records/lifecycle. |
| 17 | Episodic Memory | adopted | SPEC-32: knowledge/memory | Bounded importance/emotion/participants/outcome и deterministic retention. |
| 18 | Initial Knowledge | adopted | SPEC-32: knowledge seeding | Initial beliefs приходят только из cooked seed packages. |
| 19 | Knowledge Seeding | adopted | SPEC-32: knowledge seeding | Canonical merge `culture → location → profession → faction → family → backstory → override`. |
| 20 | Perception | adapted | SPEC-06; SPEC-32: Epistemic View | Perception остаётся отдельным immutable producer, не частью world truth query. |
| 21 | Information Retrieval | adopted | SPEC-32: knowledge/memory | Bounded deterministic retrieval; embeddings — rebuildable cache. |
| 22 | Planner | adapted | ADR-056; SPEC-32: GOAP | Utility + bounded GOAP primary; HTN optional. |
| 23 | Planning Actions | adapted | SPEC-32: Semantic Affordance | Action становится owner/revision-bound semantic record, не callback. |
| 24 | Пример планирования лечения | adopted | SPEC-32: GOAP/failure scenarios | Сохраняется как representative multi-step planning case, не hardcoded goal. |
| 25 | Replanning | adopted | SPEC-32: GOAP | Typed triggers, bounded retries and safe fallback. |
| 26 | Task Executor | adapted | SPEC-32: private executive | Private lifecycle создаёт proposals; public mutable task interface отклонён. |
| 27 | Skills | adapted | SPEC-32: skills | Execution остаётся у Navigation/Social/Mechanics/Motor owners. |
| 28 | Affordances | adopted | SPEC-32: Semantic Affordance | Generic semantic planning surface объединяет owner-specific candidates. |
| 29 | Social Layer | adopted | SPEC-32: social behavior | Structured deterministic speech acts и RPG-owned consequences. |
| 30 | Пример NPC-to-NPC interaction | adopted | SPEC-32: social ProductCheck | Ask/Inform/Offer/Accept проходит без LLM. |
| 31 | Trust и Information | adopted | SPEC-32: social behavior | Listener применяет trust/confidence к claim и provenance. |
| 32 | Relationships | adapted | ADR-056: ownership; SPEC-19 boundary | RPG — authority, Agent читает revision-bound projection. |
| 33 | Debts и Obligations | deferred | SPEC-32: social/R4 delivery | Минимальный commitment входит в R4d; развитые debt mechanics — future breadth. |
| 34 | Trade | adapted | SPEC-32: social/R4d | Real currency/items commit-ятся RPG; bargaining breadth не блокирует R4. |
| 35 | Gossip | adopted | SPEC-32: social behavior | Передаётся claim + provenance, не hidden truth. |
| 36 | Lies | adopted | SPEC-32: social behavior | Deception — semantic choice speaker; listener не получает `is_lie`. |
| 37 | LLM Layer | adapted | ADR-056; SPEC-16 | Optional parser/renderer, не planner или authority. |
| 38 | LLM Restrictions | adopted | ADR-056; SPEC-16 | No direct facts, goals, commands, commitments or required network path. |
| 39 | Player Conversation | adapted | SPEC-16 | Canonical text/player parsing converges into bounded semantic candidates. |
| 40 | NPC-to-NPC без LLM | adopted | ADR-056; SPEC-32 | Обязательный deterministic structured protocol. |
| 41 | Conversation Memory | adopted | SPEC-32: memory | Сохраняются semantic acts/claims/outcomes, не generated transcript by default. |
| 42 | AI Level of Detail | adapted | ADR-021; SPEC-20/32 | LOD mapped to `Active/Simulated/Abstract/Dormant`, physical LOD separate. |
| 43 | Location Abstraction | adapted | SPEC-08/20/32 | Logical location/route owned by World Services; physical pose separate. |
| 44 | Update Frequencies | adapted | SPEC-21; SPEC-32 | Manifest integer cadence/phase вместо приблизительных wall-time частот. |
| 45 | Event-Driven Architecture | adopted | SPEC-21; SPEC-32 | Event ставит next-boundary work, same-tick re-entry запрещён. |
| 46 | Debugging | adopted | SPEC-09; SPEC-32 | Stable diagnostics и bounded Decision Trace. |
| 47 | Planner Explainability | adopted | SPEC-32: explainability | Scores, beliefs, plan, failure/replan reason доступны как non-authoritative projection. |
| 48 | Persistence | adapted | SPEC-03/21/32 | State сохраняется owner segments; caches/trace не authority. |
| 49 | Determinism | adopted | ADR-022/056; SPEC-21/32 | Fixed-point, stable ordering, named RNG, replay without model regeneration. |
| 50 | Возможное использование обучаемой модели | deferred | ADR-050/053/054; SPEC-33/34 | Optional R8 quality track после deterministic substrate. |
| 51 | Neural Strategic Policy | deferred | ADR-050; SPEC-33 | Shared policy допустима только как bounded candidate scorer with full fallback. |
| 52 | Training Strategic Policy | deferred | SPEC-33/34 | Offline BC/RL/evaluation; training никогда не происходит в gameplay runtime. |
| 53 | Headless Strategic Simulator | deferred | SPEC-15/34 | Production headless остаётся oracle; training data plane — optional R8 consumer. |
| 54 | Procedural Testing Worlds | deferred | SPEC-34 | R&D corpus/mirror, не current shipped creator capability. |
| 55 | Economy | adapted | SPEC-32: social/R4 delivery | R4d требует real resource flow; macro-economy не R4 gate. |
| 56 | Jobs | adapted | SPEC-20/32 | World Services владеет assignment/schedule, RPG/Mechanics — reward/effect. |
| 57 | Example Emergent Scenario | adopted | SPEC-32: R4d | Сжат до production vertical `work → currency → trade → food` с social/replan. |
| 58 | Crate Architecture | rejected | ADR-046; SPEC-01/32 | Предварительный crate split отклонён; сначала modules в `crates/agent`, public contracts consumer-driven. |
| 59 | Основной Agent Loop | adopted | SPEC-32: Strategic loop | Перенесён в fixed-stage epistemic/goal/plan/task/commit/observe loop. |
| 60 | MVP | adapted | SPEC-32: R4 delivery | Разделён на R4c cognition и R4d systemic vertical после R4a/R4b. |
| 61 | MVP Scenario | adopted | SPEC-32: `STRATEGIC-R4-P1` | Production owners, failures, save/replay и headless добавлены к исходному сценарию. |
| 62 | Этапы реализации | adapted | Roadmap R4a–R4d и R8 | Deterministic stages закрывают R4; learned stage перенесён в optional R8. |
| 63 | Главные архитектурные правила | adopted | ADR-056; SPEC-32 | Правила приняты с Next Engine ownership, determinism and validation constraints. |

## Конфликты и решения

| Конфликт исходника/репозитория | Решение |
|---|---|
| Learned strategic/tactical pair был R4/v1 gate | ADR-056 делает deterministic Strategic Agent достаточным; learned production переносится в optional R8. |
| Accepted текст называл только Utility/HTN | Utility выбирает goal, bounded GOAP становится primary planner, HTN optional. |
| Monolithic `StrategicAgent` дублировал RPG/Memory/World state | Mutable authority разделена по существующим owners; Agent хранит только cognition/executive state. |
| Planner мог читать полный world state | Введён Epistemic View; authoritative validation не является knowledge channel. |
| Generic `EntityId`, floating scores и mutable task callbacks | Durable `PersistentId`/`AssetId`, fixed-point/stable order и private proposal-only executive. |
| NPC dialogue предполагал LLM layer рядом с semantics | NPC-to-NPC semantics обязательна без LLM; models только optional parse/render adapters. |
| Собственные LOD/frequency правила | Используются accepted population tiers и integer cadence/phase. |
| Предложен преждевременный crate/schema split | Exact crates/types/versions отложены до production consumer по ADR-046. |

## Coverage conclusion

Все 63 нумерованных раздела имеют один primary disposition и архитектурный
anchor. `deferred` элементы сохранены в optional/future документах, а единственный
`rejected` пункт отвергает только преждевременную форму crate split, не исходную
цель модульности. После синхронизации supporting SPEC/ADR, roadmap, routing,
README, glossary и traceability raw source может быть удалён без потери
архитектурного решения.
