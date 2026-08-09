# ADR-056: Deterministic Strategic Agent and belief-driven GOAP

| Поле | Значение |
|---|---|
| ID | ADR-056 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-09 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-16](../16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](021-deterministic-population-residency-and-time-advance.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | planner fallback wording ADR-005, где deterministic baseline назван только `utility/HTN`; R4 learned-policy gate из Proposed ADR-050 и living roadmap |
| Не заменяет | offline-first, optional `ai-host`, process isolation и validated proposal boundaries ADR-005 |

## Контекст

Product contract требует иерархический NPC AI, воспроизводимый `headless`,
offline correctness и заменяемые optional AI adapters. Он не требует, чтобы
обучаемая strategic или tactical policy была условием R4 или v1.

Прежний Proposed R4 track связывал целостный NPC vertical с двумя trained
bundles, training data plane и exact Windows/Linux GPU applied-decision parity.
Эта связь ставила исследовательский quality path на критический путь playable
systemic world. Одновременно Accepted текст называл deterministic fallback
`utility/HTN`, хотя для открытого мира с динамическими affordances основной
планировщик должен уметь искать новый bounded plan без полного authored tree.

## Решение

1. **Deterministic Strategic Agent является product baseline.** R4 и v1 не
   зависят от learned strategic/tactical policies, GPU evaluator, training
   platform, remote service, LLM, ASR или TTS.
2. **World truth и beliefs разделены.** Candidate generation, utility и
   planning читают только bounded epistemic view из perception, Memory Service
   и разрешённых immutable owner projections. Authoritative owner может
   отклонить proposal по скрытому или изменившемуся факту, но отказ не раскрывает
   этот факт агенту и не переписывает belief напрямую.
3. **Utility выбирает цель.** Candidate goals строятся из drives, aspirations,
   commitments, perceived emergencies и authored content. Scores используют
   fixed-point arithmetic, canonical ordering, goal inertia и explicit switch
   threshold. Emergency priority bands могут прервать обычную цель.
4. **Bounded GOAP является основным planner.** Он ищет план только по
   engine-owned semantic affordances, объявленным preconditions/effects,
   deterministic cost и лимитам expansion/depth. HTN разрешён как optional
   authored decomposition или bounded optimization, но не является обязательным
   условием корректности.
5. **Executive не мутирует gameplay state.** Goal, plan и private task executor
   создают только `AgentIntent` или owner-specific proposal. Любое gameplay
   изменение проходит common validation и atomic `WorldCommand` commit; только
   committed change создаёт `DomainEvent`.
6. **Индивидуальность является данными.** Identity/personality/archetype,
   knowledge, beliefs, episodic memory, relationships, needs и commitments не
   скрываются в model weights. Общая policy может быть заменяемой оптимизацией,
   но отсутствие модели не меняет доступные gameplay outcomes.
7. **NPC communication семантична и детерминирована.** NPC-to-NPC использует
   bounded structured speech acts, claims, provenance, confidence и trust без
   LLM. Text/audio generation может только parse/render уже допустимую семантику
   и не создаёт факт, обещание, сделку или RPG mutation.
8. **Population LOD использует world tiers.** `Active`, `Simulated`, `Abstract`
   и `Dormant` задают доступную точность и cadence. Tier не зависит от renderer
   camera, frustum, FPS или wall time; abstract work не фабрикует недоказанные
   traversal, combat, trade или quest outcomes.
9. **Learned policies являются optional R8 quality track.** Offline R&D может
   идти параллельно после появления canonical data plane. Production promotion
   требует собственных evidence gates и всегда сохраняет deterministic
   Strategic Agent как полный fallback, но не блокирует R4/v1.
10. **Consumer-driven schemas сохраняются.** SPEC-32 описывает target semantics,
    но exact Rust types, versions, serialization и crate split появляются
    только вместе с production consumer по ADR-046.

## Ownership

| State | Единственный technical owner |
|---|---|
| Active/suspended goals, bounded plan, private task lifecycle, hysteresis, named decision RNG | Agent Runtime |
| Semantic beliefs, episodic recollections, confidence, provenance, retrieval metadata | Memory Service |
| Character resources, inventory, currency, relationships, commitments and debts | RPG Framework |
| Calendar, routine/job assignment, logical location, population tier and route plan | World Services |
| Ability/work/trade effects and mechanic-specific affordances | Mechanics Runtime |
| Physical execution and contacts | Navigation, Motor and Physics owners |
| Immutable identity/personality/archetype/backstory and knowledge seed packages | Cooked content |

Owner state is read through immutable revision-bound projections. Agent-owned
copies may be caches only and are never a second mutable authority.

## Рассмотренные варианты

- **Оставить learned pair условием R4** — отклонено: увеличивает critical path,
  требует GPU/platform evidence для offline gameplay и не следует SPEC-00.
- **Оставить HTN единственным baseline** — отклонено: dynamic affordances и
  failure/replanning требуют authored branch coverage вместо bounded search.
- **Разрешить planner читать authoritative world snapshot** — отклонено:
  мета-знание разрушает perception, gossip, deception и explainability.
- **Сохранить весь NPC state в одном agent aggregate** — отклонено: создаёт
  параллельную authority для RPG, World Services и Memory Service.
- **LLM для NPC-to-NPC** — отклонено как correctness path: provider latency,
  stochastic text и network availability не могут определять simulation state.
- **Сразу принять exact public schemas** — отклонено по ADR-046 до первого
  production consumer.

## Последствия

- SPEC-32 становится главным Proposed design для deterministic NPC cognition.
- SPEC-33, SPEC-34 и ADR-050/053/054 остаются Proposed optional R8 documents.
- R4 состоит из calendar/routine, population/navigation substrate,
  deterministic cognition и одного systemic social/economy vertical.
- Политика, crime, coalition и macro-economy breadth не являются R4 gate.
- Current narrow affordance planner остаётся фактом реализации, но не объявляет
  целевую архитектуру завершённой.

## Проверка

- Unknown authoritative fact не влияет на candidate set до perception/memory
  update; owner rejection не добавляет скрытый belief.
- Fixed seed/state/revisions дают одинаковые goal, GOAP plan, task transition,
  command proposal и replay root в `game` и `headless`.
- Missing/stale affordance, navigation failure и emergency дают bounded typed
  replan без partial mutation.
- NPC-to-NPC scenario проходит без `ai-host`; invalid model/LLM result не меняет
  deterministic outcome.
- `Active`/`Simulated`/`Abstract`/`Dormant` сохраняют declared invariants и не
  фабрикуют недоказанный authoritative outcome.

## Supersession

ADR-056 заменяет только формулировку ADR-005, делающую `utility/HTN`
единственным deterministic planner baseline. Offline-first, optional
`ai-host`, process isolation, bounded proposals и validated `WorldCommand`
границы ADR-005 сохраняются полностью. ADR-050/053/054 имеют статус Proposed и
обновляются как optional R8 track, а не формально supersede-ятся.
