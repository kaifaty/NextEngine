# SPEC-32: Deterministic Strategic Agent cognition and social behavior

| Поле | Значение |
|---|---|
| ID | SPEC-32 |
| Статус | Proposed |
| Lifecycle | Consumer-driven R4c/R4d target; no current wire schema |
| Версия | 0.5 |
| Последняя проверка | 2026-08-12 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-056](adr/056-deterministic-strategic-agent-and-belief-driven-goap.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Заменяет | SPEC-32 0.4; makes the typed no-text handoff from the private Task Executive to Physical Embodiment explicit |

## Статус и scope

SPEC-32 задаёт целевую архитектуру deterministic Strategic Agent для R4c/R4d.
Документ не добавляет Rust types, registry entries, save segments или shipped
capability в текущем documentation-only changeset. Названия интерфейсных
семейств ниже концептуальны; exact shape и version появляются только с первым
production consumer по ADR-046.

Current `crates/agent` реализует узкий canonical affordance planner. Это полезный
production substrate, но не доказательство реализации needs, beliefs, memory,
GOAP, social behavior или population LOD из этого SPEC.

## Цели и обязательные инварианты

Strategic Agent отвечает за **что** и **почему** должен делать NPC. Mechanics,
Navigation, Social/RPG, Motor и Physics выполняют **как** через свои authority.

- Planner использует beliefs, а не полный world truth.
- Gameplay меняется только через validated `WorldCommand` transaction.
- Utility выбирает цель; bounded GOAP строит основной semantic plan.
- HTN является optional authored decomposition, а не correctness requirement.
- Goal switching использует inertia, threshold и emergency priority bands.
- Failure является typed observation/reason и ведёт к bounded replan.
- NPC-to-NPC communication не требует LLM.
- Individuality хранится в content и state, а не в neural weights.
- Durable/public identity использует `PersistentId`/`AssetId`; runtime handles,
  ECS rows, backend types и raw pointers не выходят в contract.
- Scores, costs, confidence, time и cadence используют bounded integers или
  fixed-point с canonical tie-break, не unconstrained `f32`.

## Ownership и state decomposition

| Семантика | Owner | Agent читает |
|---|---|---|
| Identity, personality, archetype, backstory, culture/profession/faction/family seed packages | Cooked content | immutable IDs, revisions and bounded traits |
| Health, hunger/fatigue resources, inventory, currency, relationships, faction membership, commitments/debts | RPG Framework | revision-bound projections |
| Beliefs, knowledge provenance, contradictions and episodic recollections | Memory Service | bounded contextual retrieval |
| Calendar, authored routine/job, workplace, logical location, tier and route plan | World Services | derived calendar/location/activity views |
| Abilities, work/gather/craft/trade effects and domain affordances | Mechanics Runtime | capability-filtered semantic affordances |
| Goals, plan, private task lifecycle, interruption stack, hysteresis and decision RNG | Agent Runtime | authoritative local state |
| Pose, traversal, contact and motor outcome | Navigation/Motor/Physics | immutable outcome facts |

Agent Runtime не дублирует mutable owner fields. Derived pressure или candidate
cache reconstructible и не становится вторым источником истины.

## Strategic loop

Canonical evaluation проходит в фиксированных stages и commit points:

1. собрать revision-bound Epistemic View;
2. получить bounded relevant beliefs/recollections;
3. вывести Drive View из owner state и agent hysteresis;
4. создать canonical candidate goals;
5. оценить Utility, inertia и emergency override;
6. сохранить текущую цель либо выбрать новую;
7. построить/починить bounded GOAP plan из semantic affordances;
8. private executive активирует ровно допустимый task step;
9. task создаёт `AgentIntent`/owner proposal;
10. owner валидирует authoritative truth и commit-ит либо возвращает typed
    outcome без скрытого knowledge leak;
11. perception/memory pipeline наблюдает committed result и решает, нужен ли
    следующий replan.

Async query, pathfinding или optional inference возвращают immutable
revision-bound result через staging queue. Mutable ECS access не удерживается
через `await`; late/stale result отбрасывается детерминированно.

## Epistemic View, knowledge и memory

Epistemic View содержит только сведения, которые NPC может обоснованно
использовать:

- текущий `PerceptionFrame` и stable observed facts;
- semantic beliefs с subject/predicate/value, confidence, source, learned tick,
  last verified revision и contradiction state;
- bounded episodic recollections с participants, place, outcome, importance и
  emotional weight;
- разрешённые self projections: собственные resources, inventory, relations,
  commitments, routine/job and logical location;
- capability-filtered affordances и known location/route facts.

Initial knowledge создаётся deterministic merge авторских seed packages в
порядке `culture → home/location → profession → faction → family → backstory →
character overrides`. Duplicate key разрешается более поздним слоем, а exact
provenance сохраняется. Seed package не может раскрыть runtime fact, которого
нет в cooked content.

Semantic knowledge и episodic memory остаются разными. Retrieval bounded по
count/bytes, сортируется stable relevance key и stable ID. Embeddings допустимы
только как rebuildable search cache; authoritative retrieval всегда имеет
deterministic fallback. Retention, importance, decay и consolidation используют
manifest-bound integer rules. Generated conversation text по умолчанию не
сохраняется как memory authority: сохраняются semantic acts, claims и committed
outcomes.

## Drives, aspirations и goal lifecycle

Drive View является derived input, а не новой mutable needs database. Hunger,
fatigue, health, money pressure, safety, social pressure и duty читаются из
своих owners; Agent сохраняет только собственную adaptation/hysteresis state,
когда её нельзя восстановить из owner history.

Aspirations — долгосрочные authored tendencies. Они создают medium-horizon
goal candidates, но никогда не исполняют action напрямую. Goal candidate имеет
stable goal/activity kind, target, cited beliefs, preconditions, completion and
failure conditions, priority band и bounded utility components. Content-specific
goals используют stable IDs/definitions вместо растущего native enum.

Utility вычисляется модульно из drive urgency, personality, relationships,
commitments, risk, expected cost/time, aspiration fit и recency. Все components
fixed-point и имеют declared bounds. Canonical candidate order и stable ID
решают tie. Текущая цель сохраняется, пока новая не превышает switch threshold;
emergency band обходит обычный threshold. Прерванная цель помещается в bounded
suspended stack и затем явно `Resume`, `Replan`, `Complete`, `Fail`, `Impossible`
или `Invalidate`, а не молча исчезает.

## Bounded GOAP и semantic affordances

Semantic Affordance — read-only объединение owner-specific возможностей. Оно
ссылается на stable affordance/action ID, owner revision, target, known
preconditions/effects, deterministic cost/time/risk, outcome range, failure
classes и execution kind. Это не executable callback и не право на mutation.

GOAP state строится из Epistemic View. Planner использует deterministic graph
search с canonical action ordering и manifest-bound limits на depth, expanded
nodes, candidate bytes и replans per boundary. Первым production consumer
выбирается простой bounded A* либо Dijkstra по non-negative fixed-point costs;
эвристика обязана быть deterministic и admissible, иначе используется Dijkstra.
Worst-case time ограничен `O(E log V)` внутри declared node/edge cap, memory —
`O(V)` внутри того же cap. Exhaustion даёт typed `PlanBudgetExhausted` и safe
fallback, а не частичный plan.

HTN может заранее раскрыть authored macro в semantic subgoals или ограничить
candidate set. Оно не обходит GOAP validation и не становится отдельным
mutation path.

Plan пересобирается при invalid goal, changed cited revision, missing/stale
affordance, failed owner validation, unreachable route, task timeout expressed
in simulation ticks, emergency entry/exit или explicit new knowledge. Wall time,
renderer state и retry-to-green не выбирают новый plan.

## Private task executive и skills

Goal описывает desired state, Plan — ordered semantic steps, Task — private
execution lifecycle одного шага, Skill — owner-specific способ исполнения.
Public `AgentTask` trait с `&mut AgentContext` не вводится.

Embodied reasoning is a responsibility of this private Task Executive together
with the Tactical Controller and Physical Embodiment skill boundary, not a new
mutable subsystem or public model-owned authority. For a physical plan step the
executive decomposes the semantic goal into a bounded `PhysicalAvatarIntent`,
observes the Proposed SPEC-14 `SkillProgressProjection` and performs only the
declared complete, retry, fallback or replan transition. The skill/motor layers
do not choose the strategic goal, and the executive does not choose joints,
contacts or torques.

Task state имеет bounded lifecycle `Pending → Active → Succeeded/Failed/
Cancelled/Suspended` и stable reason codes. Navigation task просит `RoutePlan`,
social task создаёт Speech Act, mechanics task ссылается на affordance, motor
task создаёт validated physical intent. Каждый owner повторно проверяет revision,
target, resource cost и capability перед commit.

Any dialogue/LLM text has already been compiled before this handoff. The
executive supplies only stable entity/skill/physical-primitive IDs, reference
frames, numeric targets, masks, constraints and evidence predicates. Raw text,
tokens or language embeddings never enter the physical intent, contact/chunk
planner, motor observation or progress proof.

Every physical progress decision uses only capability-filtered structured
engine projections admitted to the NPC's `Epistemic View`: semantic target
relations, permitted owner state, physical support/contact/outcome facts and
stable revisions. The engine selects this bounded feature set explicitly.
Camera images, rendered frames, depth buffers, visual embeddings and
presentation state are outside this target; the agent never reconstructs a
transform or contact already known to an authoritative owner from pixels.
Hidden world truth remains excluded even when the engine can technically read
it. Missing or stale evidence leads to bounded wait/fallback/replan and cannot
be replaced by a model guess.

## Deterministic social behavior

Speech Act является engine-owned semantic message, а не сгенерированной строкой.
Core taxonomy покрывает `Inform`, `Ask`, `Request`, `Offer`, `CounterOffer`,
`Accept`, `Reject`, `Promise`, `Warn`, `Threaten`, `Thank`, `Apologize`, `Insult`,
`Praise` и `Gossip`. Act содержит stable participants, topic/claim, cited belief
provenance, confidence, optional requested response and expiry.

Listener обновляет belief только по deterministic trust/confidence rules. Он не
получает hidden truth или `is_lie` flag. Deception — явное решение speaker
передать claim, отличающийся от его belief; truth выясняется только обычным
perception/evidence path. Gossip переносит claim и provenance, а не world truth.

Relationships, commitment, debt, currency, inventory и trade outcome принадлежат
RPG Framework. `Promise`/`Accept` создают только proposal; обязательство возникает
после RPG `WorldCommand` commit. Trade резервирует/передаёт реальные ресурсы и не
может создавать скрытую валюту или предмет. Economy safety valve допустим только
как authored observable mechanic.

LLM/ASR/TTS из SPEC-16 могут parse player utterance или render уже выбранный act.
Model output untrusted, не добавляет fact/goal/commitment и не используется для
NPC-to-NPC correctness. Authored text/subtitle остается обязательным fallback.

## Population tiers, location и cadence

Strategic Agent использует accepted World Services tiers:

| Tier | Strategic behavior |
|---|---|
| `Active` | Full perception, strategic evaluation, social/executive and physical tasks |
| `Simulated` | Reduced deterministic cadence, logical navigation/activity and bounded social/economy decisions |
| `Abstract` | Declared aggregate activities/events only; uncertain physical outcome upgrades, defers or blocks |
| `Dormant` | No ordinary cognition; deterministic wake triggers and macro owner updates only |

Tier selection принадлежит simulation и использует region, canonical distance,
importance, capabilities and profile. Renderer camera/frustum/FPS не участвуют.
Physical LOD остаётся отдельной state machine.

Logical LocationNode/region и RoutePlan принадлежат World Services; active pose
принадлежит physical owners. Upgrade/transfer использует validated placement and
traversal handoff, не teleport. Cadence задаётся integer period/phase в immutable
profile. Events лишь ставят evaluation/replan на следующую разрешённую boundary;
same-tick subscriber re-entry запрещён.

## Persistence, replay и explainability

После promotion authoritative snapshot сохраняет по owner segments:

- Agent: active/suspended goals, plan cursor, private task state, hysteresis,
  future-affecting pending decisions and named RNG state;
- Memory: semantic beliefs, contradictions, episodic records and consolidation
  state;
- RPG: resources, relationships, inventory/currency, commitments/debts;
- World Services: tier, logical location, routine/job/activity and transfers.

Planner cache, retrieval index, embedding vectors, optional inference buffers и
Decision Trace не сохраняются как authority. Load проверяет schema/profile/hash
до partial publication. Replay не повторно вызывает LLM/model и сравнивает
canonical goals, plans, proposals, commands/events and owner roots.

Decision Trace — bounded immutable diagnostic projection: candidate goals and
score components, switch reason, cited beliefs/revisions, selected plan,
affordances, task outcome and replan reason. Он не является gameplay input,
mutable inspector API или обязательным generic UI. Для одинакового
state/profile trace воспроизводим.

## R4 delivery

SPEC-32 продвигается consumer-driven increments:

1. **R4c cognition core:** Drive View, beliefs/retrieval, candidate goals,
   Utility/inertia/emergency, bounded GOAP, private executive and save/replay.
2. **R4d systemic vertical:** NPC без еды и денег получает сведения, принимает
   реальную работу, добирается до неё, получает committed currency, покупает и
   ест food; social act, relationship/commitment effect, threat interruption,
   resume/replan и failure path проходят production owners.

R4a calendar/routine и R4b tiers/navigation/100-NPC являются prerequisites.
Расширенные taxes, crime, politics, coalitions и macro-economy — future breadth.
Learned strategic/tactical policies из SPEC-33/34 — optional R8 optimization.

## Proposed ProductChecks

| Check | Сценарий и обязательный результат |
|---|---|
| `STRATEGIC-EPISTEMIC-P1` | Hidden authoritative fact отсутствует в beliefs и не меняет candidate/plan; stale owner rejection не раскрывает его, а normal perception update меняет следующий boundary. |
| `STRATEGIC-GOAP-P1` | Fixed-point utility выбирает canonical goal; bounded GOAP строит stable plan. Missing affordance, expansion cap и unreachable route дают typed fallback/replan без mutation. |
| `STRATEGIC-STATE-P1` | Emergency прерывает ordinary goal; после выхода происходит exact resume/replan. Save/restart/replay сохраняет owner segments, named RNG, command/event and state roots. |
| `STRATEGIC-SOCIAL-P1` | NPC-to-NPC `Ask/Inform/Offer/Accept` работает без `ai-host`; lie/gossip не раскрывает truth; commitment/trade возникает только после RPG commit. |
| `STRATEGIC-R4-P1` | R4d `work → currency → trade → food` vertical проходит `game` и deterministic `headless`, включая no-job/no-route/no-money/stale-revision branches. |
| `STRATEGIC-100NPC-P1` | 100 NPC используют declared tiers/cadences/budgets; abstract tiers не фабрикуют traversal/trade/combat outcomes; same inputs give exact applied decisions on Windows/Linux. |

До production consumers эти checks имеют
`NOT_RUN(NO_PRODUCTION_CONSUMER)` и не создают current check mapping. Promotion
каждого increment обновляет affected schemas, SPEC/ADR, routing, traceability,
roadmap и реальные `fast`/`play`/`persistence-replay`/`content-package` checks в
одном changeset.

## Promotion boundary

SPEC-32 может стать Accepted только с production R4c/R4d consumer, current-only
schemas и passing mapped checks по ADR-046. Promotion deterministic cognition
не зависит от SPEC-33, SPEC-34, trained bundles или GPU route. Optional learned
track может быть promoted позднее и обязан сохранять полную deterministic
совместимость и fallback из ADR-056.
