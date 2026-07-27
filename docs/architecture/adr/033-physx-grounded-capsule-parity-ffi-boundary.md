# ADR-033: PhysX grounded-capsule parity и ограниченная FFI-граница

| Поле | Значение |
|---|---|
| ID | ADR-033 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-07-27 |
| Последняя проверка | 2026-07-27 |
| Нормативные зависимости | [SPEC-05](../05-physics-animation-and-motor-control.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-26](../26-physics-world-collision-constraints-queries-and-canonical-snapshots.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-013](013-self-contained-physical-avatar-boundary.md), [ADR-027](027-physics-motor-and-animation-layering.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-032](032-grounded-capsule-physics-checkpoint-version-boundary.md) |
| Частично заменяет | ADR-002: только исходное утверждение об empty FFI allowlist |
| Заменён | не заменён |

## Контекст

Grounded-capsule reference backend уже задаёт переносимую точную семантику:
upright kinematic capsule, static Box colliders, axial sweep, canonical
contacts, `Begin/Persist/End` и `PhysicsWorldCheckpointV1`. Для проверки
replaceable-backend boundary нужен ограниченный PhysX experiment, не
превращающий SDK, native handles или float numerics в публичную authority.

Workspace запрещает `unsafe_code`. PhysX является C++ SDK и требует одного
явного исключения, которое можно проверить автоматически и удалить вместе с
experiment. Silent runtime fallback после начала simulation нарушил бы
transaction и replay semantics.

## Решение

- Эксперимент закрепляет только PhysX `5.9.0`, CPU и static libraries на
  Windows x86_64 MSVC и Linux x86_64 GNU.
- SDK не скачивается и не vendored. Developer задаёт install prefix через
  `NEXTENGINE_PHYSX_SDK_DIR`; build проверяет exact version и required
  headers/libraries до link.
- `crates/physics-physx-ffi` — единственное разрешённое место с `unsafe`.
  Workspace boundary scan содержит exact allowlist
  `next_physics_physx_ffi`; crate manifest связывает исключение с этим ADR.
- FFI использует небольшой versioned C ABI: fixed-width values, opaque
  world pointer, explicit status codes и caller-owned fixed records. C++
  exceptions, STL/container layouts, PhysX types, strings и callbacks не
  пересекают ABI.
- `crates/physics-physx` остаётся полностью safe Rust adapter. Он владеет
  lifecycle native world, проверяет profile/version/capacity, переводит
  diagnostics и реализует engine-owned `PhysicsWorldBackend`.
- Public contracts не содержат PhysX types или native IDs. Shape/feature
  identity, contact sorting/deduplication и `Begin/Persist/End` остаются на
  стороне движка. PhysX face/shape IDs не сериализуются и не входят в hashes.
- V1 experiment поддерживает только upright kinematic capsule против static
  Box colliders. Dynamic bodies, иные shapes, materials/friction/restitution,
  sensors, joints, ragdoll, articulations и GPU PhysX не входят в решение.
- PhysX принимает только
  `nextengine.physics.quantization.grounded-capsule-v2`. Raw finite IEEE-754
  bits проходят exact rational conversion и declared fixed-point bounds.
  Legacy `capsule-reference-v1` остаётся readable только reference backend.
- Reference result является exact compatibility oracle. PhysX подтверждает
  hit, distance, normal и engine token, после чего authoritative result
  формируется из canonical engine projection; correspondence mismatch
  abort-ит незакоммиченный tick.
- Backend выбирается только до world activation политикой
  `ReferenceOnly`, `PreferPhysXThenReference` или `RequirePhysX`.
  `PreferPhysXThenReference` может fallback только при activation failure.
  Ошибка после activation не переключает backend: staging уничтожается,
  предыдущий checkpoint остаётся активным, наружу возвращается stable
  diagnostic.
- Staging и replay создают backend world заново из
  `PhysicsWorldCheckpointV1`; clone native world не является контрактом.

Accepted status этого ADR разрешает узкую FFI-границу и experiment. Сам PhysX
backend остаётся technology status `Proposed`: он не входит в default features,
не является обязательной dependency и не становится backend по умолчанию до
отдельного решения после полной parity-проверки на shipping targets.

## Проверка

- Default `play`, `physics-collision`, `persistence-replay` и `host-check`
  работают без PhysX SDK.
- `physics-collision --backend reference|physx|compare` сравнивает final pose,
  ordered normalized contacts/phases, checkpoint и state/replay compare
  points.
- `persistence-replay --backend reference|physx` проверяет restore и replay
  через тот же выбранный backend.
- `physics-backend-parity` выполняет минимум 100 000 sweep/substep
  comparisons, 10 000 registration-order trials и не менее 1 000
  create/destroy cycles.
- Negative cases покрывают legacy/invalid profile, unsupported shape,
  non-finite/overflow/out-of-range conversion, capacity exhaustion, SDK/ABI
  version mismatch и native failure.
- Native boundary проходит ASan/UBSan на Linux; safe wrapper проходит Miri
  против mock ABI. Windows/Linux platform runs остаются обязательными для
  promotion, но недоступность этих hosts не меняет reference default.

## Последствия и rollback

Default workspace сохраняет portable Rust-only путь. Цена experiment —
дополнительные adapter/FFI crates, внешний SDK setup и platform-specific
native checks.

Rollback удаляет PhysX features и два PhysX crate, убирает
`next_physics_physx_ffi` из allowlist и оставляет `PhysicsWorldBackend`,
checkpoint reconstruction, quantization v2 и reference checks без изменения
durable save semantics.
