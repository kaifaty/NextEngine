# ADR-031: RPG-owned divine standing и atomic pantheon judgment

| Поле | Значение |
|---|---|
| ID | ADR-031 |
| Статус | Superseded by ADR-046; future intent is Proposed |
| Версия | 1.0 |
| Дата решения | 2026-07-26 |
| Последняя проверка | 2026-07-26 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-06](../06-ai-agents-perception-and-memory.md), [SPEC-13](../13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-20](../20-world-simulation-and-population-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-005](005-offline-first-ai-process-boundary.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-029](029-rpg-owned-quest-graph-and-optional-narrative-director.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | частично [ADR-020](020-rpg-domain-authority-and-extension-boundary.md), только пункт 1 в части closed RPG aggregate set; RPG authority, typed operations and atomicity остаются без изменений |
| Заменён | fully superseded by [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md) |

## История принятия

ADR-031 принят как extension decision для RPG-owned divine standing и atomic
pantheon judgment. Он расширяет Accepted aggregate enum, но не выбирает
LLM/provider и не утверждает runtime implementation. Detailed schemas, limits,
diagnostics and checks belong to SPEC-31.

## Контекст

SPEC-31 уже предлагает asynchronous Narrative Director, который может
формировать world-local quest branches, но engine не определяет:

- где хранится отношение конкретного бога к игроку;
- является ли «репутация у богов» обычной relationship/faction mechanic или
  самостоятельным RPG fact;
- что именно знает бог о мире и игроке;
- как несколько конфликтующих богов оценивают один поступок;
- может ли один поступок улучшить standing у одного бога и ухудшить у другого;
- как принятие covenant/boon у одного бога влияет на его соперников;
- что LLM вправе решить, а что остаётся authored deterministic policy;
- как предупредить несправедливую major sanction и бесконечные реакционные
  каскады;
- как сохранить live вариативность и при этом точно replay-ить уже случившееся.

Использование одного scalar karma скрывает конфликт ценностей. Обычный
`Relationship` aggregate не подходит: god не обязан быть Character, standing
имеет favor/attention/covenant/warning/intervention semantics и требует
отдельного lifecycle. Последовательное применение ответов богов в completion
order сделало бы OS/provider latency источником gameplay outcome.

## Решение

1. **Dedicated RPG aggregate.** `DivineStanding` добавляется в closed
   `RpgAggregateEnvelopeV1.aggregate_kind`. RPG Framework остаётся единственным
   владельцем favor, attention, covenant/vow/warning history, intervention
   cooldowns and committed judgment lineage.
2. **Standing is a vector.** Каждый player/god pair имеет независимый
   `DivineStandingV1`. Общая karma, автоматическая нормализация и предположение,
   что игрок может быть хорош для всех богов, отсутствуют.
3. **Authored gods and limited knowledge.** `DivinePatronDefinitionV1`
   содержит persona/domains/values/taboos, epistemic allowlist, intervention
   catalog, budgets/cooldowns and deterministic fallback. Реальность бога не
   означает автоматический доступ к hidden world state.
4. **Authored directed pantheon.** Content-addressed
   `PantheonRelationGraphV1` задаёт asymmetric
   `Allied | Tolerant | Rival | Hostile | Indifferent` edges, covenant
   compatibility and bounded spillover/counterquest rules. An edge may affect
   only a target with an authorized hook in the same batch. LLM не меняет этот
   graph.
5. **Independent god roles.** Каждый eligible бог получает отдельный
   `DivineDecisionRequestV1`. Нет model-produced «совета богов», единственного
   победившего бога или shared mutable provider conversation.
6. **One pre-decision snapshot.** Все per-god requests одного root event
   привязаны к одному immutable `DivineJudgmentBatchBaseV1`. Ни один бог не
   видит uncommitted result другого из того же batch.
7. **Categorical LLM authority only.** LLM выбирает одну categorical judgement
   и один eligible authored intervention/quest proposal. Exact favor/attention
   deltas, effects, XP, policy thresholds and cross-god consequences вычисляет
   deterministic validator.
8. **Atomic conflict resolver.** `PantheonConflictResolverV1` канонически
   объединяет personal judgments, directed spillover, covenant/boon conflicts
   and intervention proposals. `AdmitDivineJudgmentBatch` использует
   Runtime-owned `CrossContextTransactionPlanV1` и публикует весь multi-god
   vector, offers/covenants, effects and quest hooks atomically or ничего.
9. **Meaningful conflict is allowed.** Один committed act MAY дать Approval и
   positive standing delta одному богу, Disapproval и negative delta другому,
   а covenant/unique boon MAY вызвать отдельную authored rival reaction.
10. **Explicit covenants.** Standing существует независимо от covenant.
    `Compatible`, `Conditional` and `Exclusive` policies проверяются при
    activation. Incompatible covenant не заменяется без explicit player
    confirmation/renunciation command.
11. **Explicit divine offers.** Offered boon/covenant is an RPG-owned bounded
    `DivineOfferV1`. Only exact player/internal commands may accept, decline,
    expire, supersede, renounce or restore it; model output only proposes
    creation.
12. **Bounded interventions.** Разрешены standing/attention change, warning,
    authored boon/status, trial, minor/major sanction and sponsored quest.
    Candidate не получает raw command/effect/tool access.
13. **Sanction fairness.** Minor sanction требует explicit cited taboo or vow
    term already committed as known to the player. Major sanction требует
    causal warning plus repeated violation либо voluntarily accepted
    covenant/vow with visible penalty policy.
14. **Existing quest disclosure.** Divine voice, prayer, dream/omen and
    priest/temple map to existing `Direct`, `Solicited`, `Contextual` and
    `Public` channels. God sponsorship does not create a fifth channel or
    auto-accept quest.
15. **Bounded reactions.** Standing-change events do not recursively trigger
    judgment. Automatic consequences retain one reaction root and increment
    parent depth; only independent semantic player/world commands create a new
    root. Depth/cooldown and deterministic closure bound pantheon chains.
16. **Fixed world decision boundary.** Completion assignment is recorded
    SPEC-21 input, but every result only stages until one
    `NarrativeDecisionBoundaryV1`. Receiving all results early never moves the
    world-time boundary, and `world_tick` is never numerically compared with
    `SimulationTick`.
17. **Nondeterministic generation, deterministic history.** Fresh live runs
    MAY receive different model choices. Exact completion bytes/assignment and
    boundary closure are recorded external input. Replay uses them and makes
    zero model/network calls.
18. **Whole-candidate per-god fallback.** Missing, late, invalid or conflicting
    result replaces only that god's complete candidate with one canonical
    deterministic template candidate. No category/text salvage is allowed;
    other valid candidates remain usable and final publication is one atomic
    batch.
19. **World-locked pantheon V1.** World creation freezes the exact ordered
    patron set and pantheon graph hash. Representation migration preserves
    standing IDs/revisions/history; adding/removing/retiring patrons in an
    existing world is unsupported.

## Рассмотренные варианты

- **One global karma** — Rejected: erases conflicting values and prevents one
  act from having opposite meanings to different gods.
- **Use ordinary Relationship/Faction only** — Rejected: duplicates or hides
  favor/attention/covenant/warning/intervention semantics and requires god to be
  an ordinary Character/Faction.
- **Let LLM set numeric reputation/effects** — Rejected: prompt wording and
  model drift would directly control progression and sanctions.
- **Only the most attentive god reacts** — Rejected: removes independent
  pantheon agency and makes selection policy an accidental winner-take-all
  council.
- **One LLM call returns the whole council verdict** — Rejected: couples gods,
  hides per-god epistemic scope and makes partial fallback/replay opaque.
- **Commit gods as responses arrive** — Rejected: provider/worker completion
  order would change what later gods see and therefore change gameplay.
- **Commit as soon as every response is ready** — Rejected: provider speed would
  move the gameplay boundary even when merge order remained canonical.
- **Salvage judgement from an invalid intervention** — Rejected: validation
  repair would create an authoritative result that is neither the recorded
  candidate nor the canonical fallback.
- **Automatically balance total standing to zero** — Rejected: invents a
  universal morality that is not authored by any god.
- **Silent covenant replacement** — Rejected: removes player agency and makes
  rewards/penalties surprising.
- **Major punishment on first unseen violation** — Rejected: allows hidden
  policy/model choice to produce unfair irreversible consequence.
- **Every standing delta triggers rival reactions** — Rejected: creates
  unbounded cascades and turns bookkeeping events into new causes.
- **Apply rival spillover to a god without an authorized hook** — Rejected:
  authored relation would bypass that god's epistemic ceiling.
- **Migrate patron additions/removals in V1 saves** — Rejected: lifecycle of
  existing covenants/offers/quests requires a separate explicit migration
  decision; V1 keeps the world-locked set.
- **Regenerate divine decisions during replay** — Rejected: changes historical
  facts and can repeat external effects.

## Последствия

- SPEC-19 gains a dedicated aggregate and typed operations for standing,
  offer, covenant, warning and judgment transitions.
- Authors define patron epistemic scopes, value/taboo categories, standing
  bands, covenant groups, pantheon edges, interventions, safeguards and
  template fallbacks.
- Player UI shows qualitative bands and causal recent reasons, not exact
  optimization numbers, and exposes only committed player-visible offers.
- Agent Intelligence gains several logically independent role instances, while
  authoritative merge remains deterministic and atomic.
- Save/replay grows with divine hook/base/request/candidate/resolution metadata;
  strict per-request/batch/depth limits bound growth.
- Existing worlds remain bound to their creation-time patron set/pantheon hash;
  a different set requires the original closure or a new world in V1.
- Conflicting pantheon design becomes deliberate content: the player may gain
  access to one patron while losing favor or opportunities with another.

## Failure and fallback

- Hidden/stale fact citation, invalid intervention or failed sanction/covenant
  prerequisite rejects only that patron's complete candidate and selects its
  canonical template candidate before resolution.
- Stale/overflowing/cross-context-invalid final batch publishes no standing,
  covenant, quest, effect or event subset.
- Incompatible covenant requires explicit player transition; prior active
  covenant is preserved.
- Sanction without authored prerequisites rejects the complete candidate; its
  canonical template candidate may be warning/standing-only/`Ambivalent`.
- Timeout, crash, restart, duplicate, conflict and response reordering never
  stall simulation, move the fixed boundary or choose commit order.
- Depth/budget exhaustion commits an authored no-further-reaction closure.
- Replay divergence stops at the first batch/candidate/standing root mismatch;
  regeneration is never recovery.
- Patron-set or pantheon-hash mismatch fails before world publication and uses
  the original exact closure or a new world.

## Product checks

| Scenario | Expected result | Fallback |
|---|---|---|
| One act is approved by one god and condemned by another | Independent categorical decisions plus directed spillover produce the exact authored multi-standing vector | Preserve all prior standings if the batch cannot validate |
| Responses finish early, at boundary or after closure in every worker/provider permutation | Every request sees the same base; early results only stage, boundary-batch results are eligible and later results are rejected; selected order and resolution hash are exact | Per-god template at the fixed boundary |
| Covenant/boon offer conflicts with a rival god | Exact offer/covenant transitions and eligible-target spillover apply; no covenant is silently replaced and counterquest depth stays bounded | Reject activation or use authored non-covenant reward |
| First, repeated and covenant-bound violations request sanctions | Minor/major safeguards accept only causally justified sanctions | Warning, standing-only change or `Ambivalent` without intervention |
| Save/restart/replay occurs at every pending, boundary, offer and cross-context commit point | Exact candidates, fallbacks, offers and standing/event/state roots repeat with zero model/network calls | Fail closed before world mutation on missing/corrupt required artifact |

## Supersession

ADR-031 supersedes only ADR-020 decision item 1's closed aggregate enumeration
by adding `DivineStanding`. All ADR-020 ownership, typed-operation,
immutable-plan, atomic-publication, migration and no-direct-AI-mutation rules
remain authoritative.

Moving divine standing into AI memory, provider state, Relationship/UI/package
state; allowing raw LLM deltas/effects, completion-order commits, unbounded
reaction chains, hidden mandatory-story dependency or replay regeneration
requires a future superseding ADR with synchronized SPECs and product checks.
