# ADR-050: Hierarchical NPC cognition and learned behavior-policy boundary

| Поле | Значение |
|---|---|
| ID | ADR-050 |
| Статус | Proposed |
| Версия | 0.1 |
| Дата предложения | 2026-08-08 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-08](../08-audio-navigation-and-world-services.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-16](../16-text-canonical-multimodal-dialogue-and-model-packs.md), [SPEC-19](../19-rpg-domain-and-narrative-state.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-33](../33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

Это consumer-driven предложение для будущего R4 vertical. Оно не изменяет
Accepted runtime, не добавляет текущие public Rust contracts и не означает,
что learned behavior, GPU evaluator или trained bundles реализованы.

По ADR-046 этот ADR, SPEC-32 и SPEC-33 могут перейти в `Accepted` только одним
changeset с production consumer, использующим обе learned policy в одном
player-visible R4 vertical, и с проходящими mapped ProductCheck. До этого
Accepted utility/HTN, authored routine, tactical controller и text-only paths
из SPEC-06/SPEC-16 остаются единственной текущей baseline.

## Контекст

Текущий Agent Runtime уже разделяет high-level `AgentIntent`, deterministic
utility/HTN planning, tactical execution и motor control. Однако он не задаёт
полный контракт для обучаемой модели, которая одновременно должна учитывать
routine, fatigue, relationships, threats, navigation, combat, surrender и
dialogue initiation. Одна монолитная policy смешала бы разные cadence,
observations, failure domains и горизонты решения, а также затруднила бы
обучение и deterministic fallback.

LLM/ASR/TTS решают другую задачу: создают речь или bounded semantic proposal,
но могут отсутствовать, задерживаться и возвращать недоверенный output. Они не
могут владеть NPC goal, tactical state, world mutation или behavior tick.

Нужно зафиксировать границы между двумя learned behavior policies, Agent
executive, существующими subsystem owners, optional `ai-host` и физической
моделью исполнения.

## Предлагаемое решение

### Две независимые learned policy

Agent cognition использует две отдельные роли:

1. **Strategic behavior policy** выбирает bounded long/medium-horizon goal:
   routine activity, sleep/rest, social contact, conversation initiation,
   help seeking и другие closed strategic activities.
2. **Tactical behavior policy** выбирает bounded current-situation decision:
   navigation request, planner-visible combat affordance, fight, flee, yield,
   help-call, conversation request или emergency resolution.

Роли имеют независимые immutable bundles, observation/output/state schemas,
recurrent state, decision cadence, resource envelope и deterministic fallback.
Ни одна policy не получает output tensor или mutable state другой policy.
Strategic context передаётся tactical policy только через engine-owned
revision-bound projection.

Обе роли используют общие foundation models, conditioned на immutable
archetype, personality traits, skills и relationship views. Per-NPC model
weights, runtime fine-tuning и mutable optimizer state не входят в v1.

### Simulation tiers и cadence

Strategic policy исполняется для `Simulated` и `Active` NPC на declared
deterministic cadence. Tactical policy исполняется только для `Active` NPC и
только когда её closed candidate set требует current tactical resolution.
`Abstract` и `Dormant` используют authored schedules и deterministic abstract
rules будущего population owner; learned policy не фабрикует скрытый combat,
contact, navigation или dialogue result.

Tier и cadence выбираются canonical simulation facts и manifests. Camera,
renderer FPS, measured load, evaluator completion order и wall clock не могут
выбирать policy, candidate, deferral или tier.

### Emergency override и intention ownership

Agent Runtime остаётся единственным owner working goals and intentions.
Tactical policy не переписывает strategic goal. При подтверждённой угрозе
executive создаёт один bounded emergency frame, атомарно приостанавливает
current strategic goal и сохраняет его в bounded suspended stack.

После `Resolved`, negotiation completion, исчезновения угрозы либо нового
hostile action executive закрывает или обновляет emergency frame, повторно
проверяет source goal against current facts и либо возобновляет его, либо
отменяет/перепланирует. Стек не растёт безгранично; overflow или invalid cycle
отклоняет transition и выбирает declared safe fallback.

Behavior priority в executive/validator:

1. hard safety, quest и authored non-negotiable constraints;
2. personality и routine consistency;
3. survival;
4. tactical efficiency.

Policy score не может отменить constraint более высокого уровня.

### Exact per-seed decision contract

Behavior model не использует собственный RNG, stochastic inference op,
ambient seed или hidden evaluator state. Для exact одинаковых model bytes,
schema/profile, observations, recurrent state и named RNG state engine обязан
получить одинаковое applied strategic/tactical decision на Windows x86_64 и
Linux x86_64.

Engine строит canonical candidate set, квантует finite model scores exact
versioned integer profile и выполняет canonical selection/sampling через
named authoritative RNG stream SPEC-21. RNG state коммитится атомарно с
decision and next recurrent/intention state. Разные world/stream seeds могут
давать разнообразие; один seed не может зависеть от GPU scheduling, worker
count или evaluator-native random state.

### GPU evaluator и mandatory fallback

GPU evaluator обязателен для закрытия learned R4 gate на обеих shipping
targets. Это требование к доказательству learned vertical, а не минимальное
hardware requirement игры. При отсутствии совместимого GPU/evaluator/model
preflight выбирает declared deterministic utility/HTN behavior profile до
первого affected decision boundary; mandatory gameplay loop остаётся полным.

Evaluator boundary engine-owned и vendor-neutral. Public contracts содержат
closed capabilities, schemas, hashes, limits и resource envelope, но не CUDA,
DirectML, ONNX Runtime provider, Vulkan compute device, tensor-library session
или vendor enum/handle. Выбор конкретного backend требует отдельного bounded
implementation decision, если меняет public semantics.

Logical evaluator error, incompatible result, non-finite score, stale result
или declared injected fault выбирает manifest-bound fallback. Measured
wall-clock miss не становится authoritative failure input и не выбирает другую
decision: exact run получает nonconforming diagnostic/`Fail`, а watchdog может
остановить uncommitted work без записи альтернативной истории.

### LLM, speech и audio boundary

LLM/ASR/TTS остаются optional `ai-host` по ADR-005/SPEC-16. Они могут вернуть:

- bounded speech-act candidate;
- `GoalSuggestionCandidateV1` для следующей declared strategic boundary;
- bounded prosody/emotion candidate.

Они не устанавливают goal, tactical mode или command; не блокируют behavior
tick и не получают direct mutable world access. Strategic policy рассматривает
валидное goal suggestion как один engine-created candidate вместе с authored
activities. Late/stale/invalid suggestion discard-ится и не изменяет уже
закрытый candidate set.

Player↔NPC и NPC↔NPC используют один speech-act protocol. Generated text не
является semantic identity действия. Audio-understanding output проходит
perception validator и может стать только uncertainty-tagged revision-bound
fact; raw embedding/prosody tensor не входит прямо в behavior observation.

### Gameplay и physical mutation boundary

Behavior proposal остаётся untrusted. Normative mutation path:

```text
policy score proposal
  → Agent executive / intention commit
  → AgentIntent or InvokeAbility candidate
  → capability/schema/domain validator
  → WorldCommand transaction
  → committed DomainEvent
```

Navigation policy выбирает bounded goal/query profile либо canonical route
candidate, но `RoutePlan` строит World Services; raw waypoint output
запрещён. Tactical policy выбирает только planner-visible affordance;
Mechanics Runtime повторно проверяет actual ability. Physical model получает
только `PhysicalAvatarIntent`; direct pose, joint action или physics mutation
из behavior policy запрещены.

### Yielding

`Yielding` является Agent-owned tactical/intention state:

- публикуется как revision-bound perception fact;
- не создаёт новый RPG aggregate, Mechanics status или damage immunity;
- другие actors могут уважать либо игнорировать его согласно своим facts,
  rules и policies;
- завершается после negotiation/dialogue resolution, исчезновения угрозы либо
  нового hostile action yielding actor.

Content может отдельно провести relationship/dialogue operations через
существующий WorldCommand/RPG path. Сам state surrender ничего не коммитит в
RPG и не запрещает validated damage.

## Рассмотренные варианты

### Одна монолитная cognition model

`Rejected`: смешивает разные observation horizons, cadence, state и fallback;
тактическая ошибка могла бы повредить routine/social behavior и наоборот.

### LLM напрямую управляет goal или tools

`Rejected`: optional nondeterministic process стал бы source of truth и мог бы
обойти offline, replay, validation и tick deadline.

### Learned policy выдаёт raw waypoint или MotorAction

`Rejected`: дублирует World Services/Motor ownership и обходит route,
capability, safety и physical validation.

### Per-NPC weights и runtime learning

`Rejected` для v1: mutable weights становятся gameplay state, раздувают save,
ломают content addressing и усложняют replay/training provenance. Personality
выражается через conditioning facts и authored state.

### GPU как обязательное hardware requirement игры

`Rejected`: learned quality обязательна для R4 gate, но offline game должна
оставаться playable на complete deterministic planner route.

### Surrender как invulnerability/RPG aggregate

`Rejected`: surrender — намерение, которое другой actor может не уважать.
Защита, relationship или quest consequence являются отдельными validated
gameplay operations.

## Последствия

- R4 получает две обучаемые policy с отдельными quality/failure domains.
- Agent Runtime должен хранить future-affecting goal, emergency, recurrent,
  decision и RNG continuity; save/replay contract расширяется только вместе с
  production consumer.
- Training pipeline производит два independently addressable bundles, но R4
  promotion требует их совместной co-evaluation в одном vertical.
- GPU/provider backend остаётся заменяемым, а game сохраняет complete
  planner-only route.
- `ai-host` улучшает speech/semantic suggestions, но не становится R4 blocker.

## Proposed ProductCheck и promotion

Future implementation должна описать и реально запустить checks из SPEC-32/33:
`BEHAVIOR-SCHEMA-P1`, `BEHAVIOR-DETERMINISM-P1`, `BEHAVIOR-STATE-P1`,
`BEHAVIOR-FALLBACK-P1`, `BEHAVIOR-R4-P1`, `BEHAVIOR-100NPC-P1`,
`BEHAVIOR-COMMS-P1` и `BEHAVIOR-TRAIN-P1`.

В этом docs-only changeset все они `NOT_RUN(NO_PRODUCTION_CONSUMER)`. Ни один
из них не является текущим global gate. Promotion `Proposed → Accepted`
допускается только когда:

1. обе bundles content-addressed и реально активируются в одном production R4
   vertical;
2. deterministic utility/HTN fallback проходит тот же mandatory gameplay loop;
3. Windows/Linux GPU learned route даёт exact applied-decision parity;
4. save/load/replay, 100-NPC cadence, communications и training/runtime parity
   checks проходят;
5. SPEC-06/08/13/19/20/21 и public contract owners синхронно получают только
   минимальные reciprocal Accepted updates, требуемые consumer-ом.

До этого любые model runtime, trainer и bundles являются experiment и не могут
быть представлены как shipped R4 capability.
