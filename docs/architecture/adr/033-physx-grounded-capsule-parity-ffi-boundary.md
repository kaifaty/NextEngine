# ADR-033: PhysX grounded-capsule parity и ограниченная FFI-граница

| Поле | Значение |
|---|---|
| ID | ADR-033 |
| Статус | Superseded |
| Версия | 1.1 |
| Дата решения | 2026-07-27 |
| Последняя проверка | 2026-08-10 |
| Заменён | полностью [ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md) |

ADR-033 ввёл bounded PhysX 5.9 grounded-capsule experiment, единственную
reviewed unsafe FFI boundary и activation-only fallback к reference backend.
[ADR-058](058-physx-only-deterministic-humanoid-training-substrate.md)
заменяет experiment production PhysX-only substrate: SDK bootstrap, real
scenes/articulations, canonical reset/replay, humanoid motor loop и GPU mirror.

После atomic cutover reference solver, `PhysicsBackendPolicy` и fallback
удаляются. Исторические parity fixtures MAY использоваться как regression
inputs, но этот ADR больше не задаёт current backend, schema или check.
