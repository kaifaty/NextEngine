# ADR-013: Self-contained physical-avatar authority boundary

| Поле | Значение |
|---|---|
| ID | ADR-013 |
| Статус | Accepted |
| Версия | 1.2 |
| Дата решения | 2026-07-23 |
| Последняя проверка | 2026-08-10 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md), [ADR-066](066-contact-centric-physical-skill-and-morphology-conditioned-motor-architecture.md) |
| Заменяет | [ADR-004](004-physics-avatar-backend-boundary.md) |
| Заменён | process clauses partially by ADR-030; backend-selection/fallback clauses partially by ADR-058 |

## Частичное supersession ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежний формальный lifecycle допуска. Single-writer physics authority,
engine-owned public contracts, deterministic observation/order, LOD state,
safety clamps и bounded fallbacks сохраняются.

## Контекст

ADR-004 правильно фиксировал physics-authoritative avatar и replaceable
backend, но ссылался на внешний исследовательский draft как на нормативную
предпосылку. Accepted boundary должна быть реализуема только по локальным
Accepted документам. Research объясняет происхождение решения, но не определяет
engine contracts, phases или открытые вопросы.

## Решение

### Authority и state ownership

- CPU runtime physics является единственным source of truth для body pose,
  contacts, constraints и topology активного physical avatar.
- Gameplay/RPG изменяет мир только validated `WorldCommand`; physical system
  принимает immutable `PhysicalAvatarIntent` и engine-owned descriptors, а
  authoritative outcomes возвращает через общий stage-9 command path.
- Renderer читает immutable `RenderPose`. Animation, renderer, AI, policy
  runtime и backend adapter не записывают authoritative body transforms.
- Каждый mutable field принадлежит одному engine-owned state object. Backend
  copies являются reconstructible caches и не входят в save/replay как второй
  source of truth.
- Public contracts принадлежат engine: `PhysicsBackend`, `BodyDescriptor`,
  `JointDescriptor`, `ContactRecord`, `PhysicsQuery`, `PhysicalSnapshot`,
  `PhysicalAvatarIntent`, `PhysicalOutcome`, `RenderPose`.
- Vendor handles, backend phases, allocator/task types, raw pointers и
  OpenGothic/Bullet/Jolt/PhysX types запрещены в `crates/contracts`, saves,
  replays, WIT, scripts, AI IPC и package APIs.

### Data flow и deterministic observation

На tick physical flow имеет фиксированный порядок:

1. validated ingress commands изменяют engine-owned intent/config state;
2. snapshot builder создаёт canonical body/constraint input для backend;
3. backend выполняет declared substeps без gameplay callbacks;
4. adapter нормализует contacts, query results и pose в engine-owned units/axes;
5. contacts сортируются по `(tick, substep, min(body_a, body_b),
   max(body_a, body_b), contact_feature_id)`;
6. safety/resolver создаёт outcome proposals; общий validator коммитит их на
   stage 9;
7. presentation строит `RenderPose` только из committed physical snapshot.

Backend worker completion, callback order и native handles не влияют на public
ordering. Numeric projection/tolerance следует ADR-022
`RuntimeDeterminismProfileV1`; gameplay outcome и event classification остаются
exact.

### Physical LOD

Engine-owned LOD state machine содержит четыре tiers:

| Tier | State authority | Required behavior |
|---|---|---|
| Full articulation | Physics pose/contact state | Полные joints, contacts, motor policy и safety supervisor |
| Simplified active ragdoll | Physics pose/contact state | Reduced bodies/constraints с canonical projection |
| Capsule/animation | Capsule collision state; animation is presentation | Нет скрытых articulated gameplay contacts; declared approximate interaction set |
| Abstract simulation | RPG/world-service state | Нет backend body; deterministic aggregate outcomes |

Tier выбирается по deterministic simulation inputs, `PersistentId` и policy
manifest, а не camera, renderer FPS, wall time или worker load. Transition
выполняется только на declared commit point. Downshift сохраняет canonical
momentum/pose summary; upshift проходит overlap validation, penetration recovery
и bounded fallback. Failed transition оставляет last safe tier и выдаёт stable
diagnostic без partial topology mutation.

### Topology, safety и fallback

- Topology mutation является validated atomic operation над engine descriptor
  graph; adapter применяет её all-or-nothing.
- Contact loss, invalid joint mapping, non-finite value, energy/torque limit или
  unsafe recovery отменяет uncommitted proposals и переводит avatar в declared
  safe fallback.
- LLM не участвует в physics/motor tick. Model action является недоверенным
  input для safety supervisor; deterministic fallback policy обязательна.
- Runtime/training observations, action units и compatibility hashes принадлежат
  engine schema. GPU training не становится runtime authority.
- Missing PhysX backend is a typed pre-activation configuration failure under
  ADR-058. Incompatible optional policy still selects the declared procedural
  controller, which executes through PhysX.

### Backend technology

ADR-058 selects PhysX 5.9.0 reduced-coordinate articulations as the sole
production backend. Jolt, Bullet and the old reference solver are not runtime
fallbacks. PhysX remains behind the same engine-owned contract; missing or
failed backend aborts activation/uncommitted work and never weakens it.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Public-boundary и dependency scan | Только local Accepted documents определяют contract; vendor/research types отсутствуют в public API | Исправить dependency/API boundary; research annex остаётся ненормативным |
| Body/axis/limit mapping, contacts и topology mutations проходят representative corpus | Units/axes нормализованы; contact order exact; cycle, invalid mapping и partial topology publication отвергаются | Оставить last safe topology и перейти на simpler backend/tier |
| Одинаковый closed input повторяется на одном platform/profile | Committed outcomes и ordered observations совпадают; non-finite values и unsafe actions отклоняются до publication | Использовать procedural policy или capsule/abstract tier |
| Learned policy проходит export parity, runtime compatibility и recovery scenarios | Observation/action schema и hashes совпадают; safety limits действуют до physical mutation | Использовать deterministic in-process controller |

## Ненормативная provenance

[Frozen physical-avatar research annex](../research/physical-avatar-research-spec.md)
сохранён только для истории происхождения и сравнения вариантов. При конфликте
этот ADR и профильные Accepted SPEC имеют приоритет.

## Последствия

ADR-004 остаётся `Superseded`; SPEC-05 использует этот engine-owned boundary.
PhysX-only cutover and Stage 0 completion remain gated by ADR-058 product
checks; Accepted architecture alone is not an implementation-completion claim.
