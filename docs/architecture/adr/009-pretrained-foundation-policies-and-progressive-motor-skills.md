# ADR-009: Pretrained foundation policies и progressive motor skills

| Поле | Значение |
|---|---|
| ID | ADR-009 |
| Статус | Superseded |
| Версия | 1.1 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-08-10 |
| Нормативные зависимости | [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) |
| Заменяет | отсутствует |
| Заменён | полностью [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) |

## Historical summary

ADR-009 ввёл immutable pretrained weights, proficiency-aware deterministic
route selection, bounded conditioned/residual/exclusive composition,
engine-owned safety clamps и mandatory procedural fallback. ADR-030 отдельно
заменил его прежние certification/process clauses.

[ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md)
полностью заменяет оставшуюся technical semantics: сохраняет перечисленные
инварианты, добавляет общий `BodySchema`, explicit skill/contact/motion layers,
dynamics adaptation и несколько policy families, а concrete learned profiles
оставляет Proposed до production consumer.

Этот файл является historical pointer и не регулирует новые изменения.
