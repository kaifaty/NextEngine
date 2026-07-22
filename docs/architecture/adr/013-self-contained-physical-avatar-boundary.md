# ADR-013: Self-contained physical-avatar authority boundary

| Поле | Значение |
|---|---|
| ID | ADR-013 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Physical Embodiment Team |
| Требуемые согласующие | Architecture Working Group, Physical Embodiment Team |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-14](../14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-15](../15-headless-testing-agent-validation-and-human-evidence.md), [ADR-009](009-pretrained-foundation-policies-and-progressive-motor-skills.md) |
| Заменяет | [ADR-004](004-physics-avatar-backend-boundary.md) после принятия packet 1.5 |
| Заменён | не заменён |

## Статус предложения

Этот ADR является self-contained review candidate. Он не меняет Accepted ADR-004 до одобрения и атомарной синхронизации packet 1.5. После принятия внешний research document перестаёт быть нормативной предпосылкой physical architecture.

## Контекст

ADR-004 правильно фиксирует physics-authoritative avatar и replaceable backend, но его normative dependency указывает на внешний исследовательский Draft. Accepted boundary должна быть реализуема только по локальным Accepted документам. Research может объяснять происхождение решения, но не определять engine contracts, phases или открытые вопросы.

## Решение

### Authority и ownership

- CPU runtime physics является единственным source of truth для body pose, contacts, constraints и topology активного physical avatar.
- Gameplay/RPG изменяет мир только validated `WorldCommand`; physical system принимает immutable `PhysicalAvatarIntent` и engine-owned descriptors, а authoritative outcomes возвращает через общий stage-9 command path.
- Renderer читает immutable `RenderPose`. Animation, renderer, AI, policy runtime и backend adapter не записывают authoritative body transforms напрямую.
- Каждый mutable field имеет одного owner. Backend copies — reconstructible caches и не входят в save/replay как второй source of truth.
- Public contracts принадлежат engine: `PhysicsBackend`, `BodyDescriptor`, `JointDescriptor`, `ContactRecord`, `PhysicsQuery`, `PhysicalSnapshot`, `PhysicalAvatarIntent`, `PhysicalOutcome`, `RenderPose`.
- Vendor handles, backend phases, allocator/task types, raw pointers и OpenGothic/Bullet/Jolt/PhysX types запрещены в `crates/contracts`, saves, replays, WIT, scripts, AI IPC и package APIs.

### Data flow и deterministic observation

На tick physical flow имеет фиксированный порядок:

1. validated Ingress commands изменяют engine-owned intent/config state;
2. snapshot builder создаёт canonical body/constraint input для выбранного backend;
3. backend выполняет declared substeps без gameplay callbacks;
4. adapter нормализует contacts, query results и pose в engine-owned units/axes;
5. contacts сортируются по `(tick, substep, min(body_a, body_b), max(body_a, body_b), contact_feature_id)`;
6. safety/resolver создаёт `Outcome` proposals; общий validator коммитит их на stage 9;
7. presentation строит `RenderPose` только из committed physical snapshot.

Backend nondeterministic worker completion, callback order и native handles не влияют на public ordering. Numeric tolerance допускается только по ADR-012 после его принятия; gameplay outcome и event classification остаются exact.

### Physical LOD

Engine-owned LOD state machine содержит четыре tiers:

| Tier | Authority | Required behavior |
|---|---|---|
| Full articulation | Physics pose/contact authority | Полные joints, contacts, motor policy и safety supervisor |
| Simplified active ragdoll | Physics pose/contact authority | Reduced bodies/constraints с canonical projection |
| Capsule/animation | Capsule collision authority, animation presentation | No hidden articulated gameplay contacts; declared approximate interaction set |
| Abstract simulation | RPG/world-service authority | Нет backend body; deterministic aggregate outcomes |

Tier выбирается deterministic simulation inputs, PersistentId и policy manifest, а не camera, renderer FPS, wall time или worker load. Transition выполняется только на declared commit point. Downshift сохраняет canonical momentum/pose summary; upshift проходит overlap validation, penetration recovery и bounded fallback. Failed transition оставляет last safe tier и выдаёт stable diagnostic, без partial topology mutation.

### Topology, safety и fallback

- Topology mutation является validated atomic operation над engine descriptor graph; all-or-nothing adapter application обязательна.
- Contact loss, invalid joint mapping, non-finite value, energy/torque limit или unsafe recovery отменяет uncommitted proposals и переводит avatar в declared safe fallback.
- LLM не участвует в physics/motor tick. Model action является untrusted input safety supervisor; deterministic fallback policy обязателен.
- Runtime/training observations, action units и compatibility hashes принадлежат engine schema. GPU training не становится runtime authority.
- Missing backend/policy/capability обозначается `PrototypeFallback` или `AwaitingCapability`; `PhysicalCertified` запрещён без всех POLICY/PHYS/TRAIN gates, включая `TRAIN-RTX-01`.

## Backend technologies

Статусы backend не меняются этим предложением:

- PhysX reduced-coordinate articulations — `Proposed` primary hypothesis;
- Jolt — `Proposed` first fallback;
- Bullet — `Proposed` second fallback и comparator.

Выбор backend допустим только через один engine-owned contract и существующие measurable gates. Failure primary gate запускает declared fallback; он не разрешает снизить public contract или выдать certification claim.

## Gates

ADR сохраняет gates ADR-004 и профильных SPEC: body/axis/limit mapping, contact delivery/order, topology cycles, leak/sanitizer evidence, same-platform replay tolerance, reference CPU budget, ONNX parity, runtime/training correspondence, safety/recovery и capture evidence. Candidate не объявляет их пройденными.

Дополнительно documentation gate MUST доказывать:

- Accepted normative dependency graph содержит только local Accepted targets;
- public-contract scan не обнаруживает vendor/research types;
- research annex упоминается только в explicitly non-normative provenance context.

## Ненормативная provenance

[Frozen physical-avatar research annex](../research/physical-avatar-research-spec.md) сохранён для исторической проверяемости origin commit/hash и сравнения вариантов. Он не является dependency, не добавляет требований и не может изменять Next Engine contract. При конфликте этот ADR и профильные Accepted SPEC имеют полный приоритет.

## Последствия и синхронизация при принятии

При принятии ADR-004 получает `Superseded` и backlink; SPEC-05, evidence register и traceability одновременно переключаются на ADR-013. PhysX/Jolt/Bullet остаются с текущими `Proposed` gates/fallbacks. До явного approval packet 1.4 и ADR-004 остаются authoritative.
