# ADR-004: Physics-authoritative avatars и replaceable backend

| Поле | Значение |
|---|---|
| ID | ADR-004 |
| Статус | Superseded |
| Версия | 1.0.1 |
| Владелец | Physical Embodiment Team |
| Дата решения | 2026-07-22 |
| Последняя проверка evidence | 2026-07-23 |
| Нормативные зависимости | [SPEC-05](../05-physics-animation-and-motor-control.md) |
| Заменяет | отсутствует |
| Заменён | [ADR-013](013-self-contained-physical-avatar-boundary.md) |

## Контекст

Physical avatar должен реагировать на контакты и perturbations как физическая система, сохраняя заменяемость backend. GPU training может быть эффективен, но runtime gameplay нуждается в доступной контактной телеметрии и корректном authoritative pose.

## Решение

- CPU runtime physics MUST быть source of truth для physical avatar pose, contacts и topology в active simulation tiers.
- Renderer MUST читать immutable `RenderPose`; gameplay/animation/AI MUST NOT записывать body transforms физического avatar напрямую.
- `PhysicsBackend`, body/articulation descriptors, queries, contact stream и snapshot schema MUST принадлежать engine. Vendor types (`Px*`, Bullet/Jolt handles) запрещены вне backend.
- PhysX reduced-coordinate articulations имеют статус primary `Proposed` backend. Jolt является first fallback, Bullet — second fallback и research comparator.
- GPU simulation MAY использоваться для training/batching, но runtime/training correspondence и ONNX parity обязательны. LLM запрещён в motor и physics ticks.
- Physics LOD MUST поддерживать full articulation → simplified active ragdoll → capsule/animation → abstract simulation с безопасными переходами.

## Рассмотренные варианты

- Animation-authoritative locomotion как основной physical tier — `Rejected`: контакты становятся визуальным эффектом, а не причиной движения.
- Прямой PhysX API в gameplay — `Rejected` из-за backend lock-in.
- GPU PhysX как runtime source — `Rejected` до доказательства полного обязательного contact/query contract.

## Последствия

Character gameplay взаимодействует только через `PhysicalAvatarIntent`, normalized contacts и outcomes. Backend migration должна сохранять golden body descriptors и scenario suite. Не прошедший policy/backend не допускается к vertical slice.

## Gate для Proposed частей

| Поле | Требование |
|---|---|
| Владелец | Physical Embodiment Team |
| Сценарий/команда | `cargo xtask gate physical-avatar --suite vertical-v1 --backend physx --seed-set fixed-v1 --render-artifacts` |
| Threshold | 100% body/axis/limit mapping и ≤1% torque conversion error; ≥99.9% required contacts delivered with tick ordering; topology mutation completes without invalid handle/leak in 1 000 cycles; replay final root pose ≤2 cm/1° и event sequence exact within declared floating tolerance on same platform; 16 full avatars ≤4 ms physics+motor p95 on reference 8-core CPU; ONNX actions max abs error ≤1e-5 against training export for 10 000 observations; runtime/training trajectory normalized RMSE ≤0.05 over golden suite; все safety/recovery gates SPEC-05 проходят |
| Evidence | run manifest, metrics, contacts trace, leak/sanitizer report, `baseline.gif`, `selected-policy.gif`, `failures.gif`, `comparison.gif`, MP4 when needed, SHA-256 for every media file |
| Fallback | Повтор того же suite на Jolt; затем Bullet. Если ни один backend не проходит, vertical slice блокируется, capsule demo не считается заменой physical gate |
| Срок повторной проверки | перед M3 physical slice и при backend/policy/model upgrade |

## Supersession

Packet 1.5 заменил это решение self-contained boundary [ADR-013](013-self-contained-physical-avatar-boundary.md). Исторический backend rationale сохранён; новые normative зависимости и изменения physical ownership определяются только ADR-013 либо будущим superseding ADR.
