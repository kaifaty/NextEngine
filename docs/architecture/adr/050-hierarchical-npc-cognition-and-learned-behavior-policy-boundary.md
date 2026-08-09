# ADR-050: Optional learned strategic and tactical behavior-policy boundary

| Поле | Значение |
|---|---|
| ID | ADR-050 |
| Статус | Proposed |
| Lifecycle | Optional R8 quality track |
| Версия | 0.3 |
| Дата предложения | 2026-08-08 |
| Последняя проверка | 2026-08-09 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-27](../27-motor-observation-action-and-deterministic-inference.md), [SPEC-32](../32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [SPEC-33](../33-behavior-policy-training-evaluation-and-deployment-lifecycle.md), [SPEC-34](../34-model-training-environments-trajectories-and-consolidation-lifecycle.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-053](053-engine-native-model-training-and-immutable-artifact-boundary.md), [ADR-054](054-bounded-strategic-adaptation-and-two-tier-sleep.md), [ADR-056](056-deterministic-strategic-agent-and-belief-driven-goap.md) |
| Заменяет | ADR-050 0.2 R4 learned-pair proposal |
| Заменён | не заменён |

## Статус предложения

ADR-056 принял deterministic Strategic Agent как достаточный R4/v1 baseline.
Этот ADR теперь описывает только optional R8 quality track. Он не создаёт
current Rust contracts, model registry, GPU requirement или shipped bundle.
Offline research может идти параллельно после появления canonical data plane,
но production promotion не меняет roadmap exit criteria прошлых этапов.

## Контекст

Learned policy может улучшить разнообразие или качество выбора при большом
candidate set, но не должна скрывать identity, beliefs, needs, goals,
commitments или gameplay rules внутри weights. Strategic cognition и tactical
execution имеют разные cadence, observations, failure domains и evidence, а
Motion Controller остаётся отдельным physical layer.

## Предлагаемое решение

### Две независимые optional роли

- **Strategic scorer** получает bounded canonical goal/affordance candidates
  из deterministic SPEC-32 pipeline и может только ранжировать/выбрать один из
  них. Он не создаёт world facts, goals произвольного вида или commands.
- **Tactical scorer** получает bounded composite candidates, собранные Agent,
  Mechanics, Navigation and Motor capability projections, и выбирает только
  один допустимый semantic action. Он не выдаёт raw waypoint, pose, joint
  target, torque или backend action.

Роли имеют независимые immutable bundles, profile IDs, recurrent state,
cadence, activation and fallback. Shared foundation ancestry допустима только
как provenance; runtime state и failure не объединяются.

### Deterministic substrate и epistemic ceiling

Candidate construction, masks, ordering, fixed-point quantization, sampling,
validation, state commit and fallback принадлежат engine. Strategic input не
может превышать Epistemic View агента. Tactical input не содержит hidden
physics/backend state или critic-only training features.

Model output является untrusted score/selection proposal. Invalid, stale,
missing, non-finite, late или incompatible result отбрасывается целиком до
publication. Deterministic Utility + bounded GOAP/tactical fallback из
ADR-056/SPEC-32 проходит тот же gameplay loop без model, trainer, network или
compatible accelerator.

### State, determinism и activation

Future-affecting recurrent/adaptation state полностью externalized,
versioned, bounded and saved with named RNG continuity. Model weights immutable
в active session; runtime training, optimizer state and in-place hot swap
запрещены. Новая bundle revision активируется только новой session через exact
project lock.

Windows/Linux production evaluator обязан дать exact canonical selected/applied
decision and state-root parity для конкретного bundle/backend pair. Raw tensor
tolerance не может скрывать другое canonical решение. GPU является требованием
evidence выбранного optional route, но не hardware requirement игры.

### Strategic, Tactical и Motion boundary

Strategic scorer выбирает goal/intention candidate. Deterministic executive
строит/проверяет semantic plan. Tactical scorer выбирает допустимый composite
candidate. Motion Controller получает только validated `PhysicalAvatarIntent`
и работает по SPEC-14/27; behavior policy не владеет pose, contacts or motor
recurrent state.

## Failure semantics

| Failure | Результат |
|---|---|
| Missing/incompatible bundle or evaluator | Select deterministic route before affected boundary; no tick stall. |
| Invalid candidate ordering/mask/score/state | Reject whole decision/state pair; retain previous valid state and fallback. |
| Late worker result or stale revisions | Discard by assignment identity; no retry-to-green. |
| GPU/runtime parity mismatch | `NONDETERMINISTIC_RESULT`; quarantine exact artifact/adapter pair. |
| One learned role unavailable | That role falls back independently; optional track remains unpromoted until declared suite passes. |
| Runtime training/hot-swap request | Deny capability; active weights/state unchanged. |

## Рассмотренные варианты

- **Learned pair как R4/v1 gate** — rejected by ADR-056: research quality не
  должна блокировать systemic offline world.
- **Одна end-to-end policy от goal до motor action** — rejected: смешивает
  cadence, ownership, safety and replay boundaries.
- **Model создаёт новые actions/waypoints** — rejected: обходит authored
  capabilities и owner validation.
- **GPU обязателен для игры** — rejected: complete deterministic route является
  baseline.
- **Hidden recurrent state** — rejected: save/replay и cross-target parity
  требуют explicit external state.

## Proposed R8 evidence

Optional promotion требует:

1. production consumer хотя бы одной роли с immutable bundle and current-only
   schema по ADR-046;
2. exact runtime/training candidate, state and applied-decision parity;
3. multi-seed held-out comparison against deterministic baseline and declared
   simple learned comparator;
4. save/load/replay, fault, resource and target checks;
5. deterministic fallback completing the same gameplay outcomes;
6. package/license/provenance and new-session activation closure.

Joint Strategic/Tactical co-evaluation требуется только если конкретный shipped
profile обещает одновременную работу обеих roles. Одна роль не обязана ждать
вторую для independent optional promotion, а ни одна из них не блокирует R4/v1.

## Promotion boundary

ADR-050 остаётся `Proposed` до первого optional R8 production consumer и его
checks. SPEC-32/ADR-056 не зависят от promotion этого ADR. Research prototype,
training report, exported bytes или shadow inference не являются shipped
capability.
