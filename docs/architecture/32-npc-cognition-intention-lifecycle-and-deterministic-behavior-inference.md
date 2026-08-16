# SPEC-32: Deterministic Strategic Agent cognition and social behavior

| Поле | Значение |
|---|---|
| ID | SPEC-32 |
| Статус | Accepted R4c core; Proposed R4d expansion |
| Lifecycle | Current deterministic cognition owner vertical; social/economy breadth remains consumer-driven |
| Версия | 1.0 |
| Последняя проверка | 2026-08-16 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-06](06-ai-agents-perception-and-memory.md), [SPEC-08](08-audio-navigation-and-world-services.md), [SPEC-09](09-tooling-sdk-and-observability.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-16](16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-19](19-rpg-domain-and-narrative-state.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [ADR-005](adr/005-offline-first-ai-process-boundary.md), [ADR-016](adr/016-compositional-gameplay-budgets.md), [ADR-020](adr/020-rpg-domain-authority-and-extension-boundary.md), [ADR-021](adr/021-deterministic-population-residency-and-time-advance.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-056](adr/056-deterministic-strategic-agent-and-belief-driven-goap.md), [ADR-066](adr/066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md) |
| Заменяет | SPEC-32 0.5; promotes the production R4c cognition core while retaining R4d social/economy sections as Proposed |

## Статус и scope

SPEC-32 теперь разделён по фактической delivery boundary. R4c cognition core
принят через ADR-073 и реализован exact V1 contracts в `crates/contracts`,
deterministic planner/owners в `crates/agent`, fixed-stage consumer в
`crates/runtime`, current content V5/V6 и Replay V8. R4d structured social
behavior, commitments and work/currency/trade/food vertical остаются Proposed
до собственного production consumer по ADR-046.

Current R4c consumer связан с одним существующим population subject. Он
доказывает semantic beliefs/retrieval, health/duty Drive View, canonical goal
Utility/inertia/emergency, bounded GOAP, private task/intention state,
Decision Trace и separate Agent/Memory save/replay. Он не доказывает episodic
memory, generic perception, social communication, economy, physical traversal
или tier-aware cognition всех 100 NPC.

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

Текущий R4c exact cut — `SemanticBeliefV1`, `AgentMemorySnapshotV1` и
`EpistemicViewV1`. Belief хранит stable hash ID, subject/predicate/value,
Q16 confidence, authored-seed или owner-projection provenance, learned tick,
verified revision и contradiction state. Retrieval сортирует consistent
beliefs по confidence, learned tick и belief ID и ограничивается authored
limit. Current view добавляет только own RPG health, population logical
location/revision and a revision-bound engine route. Perception frames,
episodic records, social claims and consolidation are not yet current schemas.

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

R4c реализует минимальные safety/duty pressures и ordinary/emergency goals in
Q16. Canonical order, switch threshold, ordinary inertia and emergency priority
are current. Emergency entry stores the ordinary goal in the bounded suspended
stack; exit produces explicit `EmergencyExitResume`. Personality, hunger,
money, relationships and aspirations in the broader formula remain R4d/future
inputs until consumed.

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

Current `SemanticAffordanceV1` admits only revision-bound logical-route request
and hold-position records. The R4c planner is bounded by authored depth and
expanded-node limits and returns `MissingAffordance` or
`PlanBudgetExhausted` with no proposal. Generic A*/HTN, mechanics work/trade
affordances and physical traversal remain outside the current cut.

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

In R4c the private executive persists one `PrivateTaskStateV1` and emits one
`StrategicAgentIntentV1` (`RequestLogicalRoute` or `HoldPosition`). The
production consumer commits the Agent/Memory decision and diagnostic event but
does not claim that World Services executed the route or that another owner
outcome succeeded. Owner-specific task execution and outcome-driven replanning
are R4d work.

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

## Deterministic social behavior (R4d target; Proposed)

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

R4c consumes the accepted logical location and route for one bound subject at
its authored `1 + 3n` evaluation cadence. It does not yet dispatch cognition by
all four population tiers or run cognition for the full 100-record workload;
the tier table above remains the R4d target.

## Persistence, replay и explainability

Current authoritative snapshot сохраняет по owner segments:

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

R4c exact segments are `AgentCognitionSnapshotV1` under
`nextengine.agent-runtime` and `AgentMemorySnapshotV1` under
`nextengine.memory-service`. They advance as one joint publication. Reference
alpha has eight full-tuple application descriptors; current-only
`ReplayManifestV8` requires both cognition segments and compares their bytes,
commands/events and application root. `SaveManifestV2` and
`WorldCheckpointV4` retain their generic wire shapes.

Decision Trace — bounded immutable diagnostic projection: candidate goals and
score components, switch reason, cited beliefs/revisions, selected plan,
affordances, task outcome and replan reason. Он не является gameplay input,
mutable inspector API или обязательным generic UI. Для одинакового
state/profile trace воспроизводим.

## R4 delivery

SPEC-32 продвигается consumer-driven increments:

1. **R4c cognition core (`COMPLETE`):** Drive View, beliefs/retrieval,
   candidate goals, Utility/inertia/emergency, bounded GOAP, private executive
   and separate Agent/Memory save/replay are current under ADR-073.
2. **R4d systemic vertical:** NPC без еды и денег получает сведения, принимает
   реальную работу, добирается до неё, получает committed currency, покупает и
   ест food; social act, relationship/commitment effect, threat interruption,
   resume/replan и failure path проходят production owners.

R4a calendar/routine и R4b tiers/navigation/100-NPC являются prerequisites.
Расширенные taxes, crime, politics, coalitions и macro-economy — future breadth.
Learned strategic/tactical policies из SPEC-33/34 — optional R8 optimization.

## ProductChecks

| Check | Сценарий и обязательный результат |
|---|---|
| `STRATEGIC-EPISTEMIC-P1` (current focused tests) | Hidden authoritative value is absent from the constructed view and cannot change candidate/plan; semantic belief retrieval is bounded, canonical and provenance-preserving. |
| `STRATEGIC-GOAP-P1` (current focused tests plus `play`) | Fixed-point Utility selects the canonical goal; bounded GOAP builds a stable plan. Missing affordance and expansion cap give typed no-proposal fallback without mutation. |
| `STRATEGIC-STATE-P1` (current production branch plus `persistence-replay`) | Emergency interrupts and suspends the ordinary goal; exact exit resumes it. Save/restart/Replay V8 preserves Agent/Memory segments, decision RNG, commands/events and eight-owner roots. |
| `STRATEGIC-SOCIAL-P1` (Proposed R4d) | NPC-to-NPC `Ask/Inform/Offer/Accept` works without `ai-host`; lie/gossip does not reveal truth; commitment/trade exists only after RPG commit. |
| `STRATEGIC-R4-P1` (Proposed R4d) | R4d `work → currency → trade → food` vertical passes `game` and deterministic `headless`, including no-job/no-route/no-money/stale-revision branches. |
| `STRATEGIC-100NPC-P1` (Proposed R4d) | 100 NPC use declared tiers/cadences/budgets; abstract tiers fabricate no traversal/trade/combat outcome; same inputs give exact applied decisions on Windows/Linux. |

The first three checks are realized by current focused tests and mapped
`play`/`persistence-replay`/`content-package` paths under ADR-073. The R4d rows
remain `NOT_RUN(NO_PRODUCTION_CONSUMER)` and create no current schema or check
mapping. Promotion of another increment updates affected schemas, SPEC/ADR,
routing, traceability and roadmap together.

## Promotion boundary

The R4c subset is Accepted because its production consumer, current-only
schemas and mapped checks exist under ADR-073. R4d clauses become Accepted only
with their own production owners and passing checks under ADR-046. Deterministic
cognition does not depend on SPEC-33, SPEC-34, trained bundles or a GPU route;
any later learned track must preserve exact deterministic compatibility and the
ADR-056 fallback.
