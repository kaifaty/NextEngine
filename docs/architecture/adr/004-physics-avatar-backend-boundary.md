# ADR-004: Physics-authoritative avatars и replaceable backend

| Поле | Значение |
|---|---|
| ID | ADR-004 |
| Статус | Superseded |
| Версия | 1.0.1 |
| Дата решения | 2026-07-22 |
| Заменён | [ADR-013](013-self-contained-physical-avatar-boundary.md) |

Это решение исторически отделило engine-owned physical boundary от конкретного backend: активная симуляция определяет pose, contacts и topology, renderer читает immutable представление, а gameplay и AI не записывают body transforms через vendor API. Backend objects остаются внутри адаптера, поэтому реализацию можно заменить без утечки её типов в публичные контракты.

Действующие правила physical authority, LOD, safety и fallback определяет только [ADR-013](013-self-contained-physical-avatar-boundary.md). Этот файл сохраняет контекст исходного выбора и не устанавливает текущих требований.
