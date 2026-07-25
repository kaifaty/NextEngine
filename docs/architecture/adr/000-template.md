# ADR-NNN: Краткое название решения

| Поле | Значение |
|---|---|
| ID | ADR-NNN |
| Статус | Draft |
| Версия | 1.0 |
| Дата решения | YYYY-MM-DD |
| Последняя проверка | YYYY-MM-DD |
| Нормативные зависимости | ссылки на SPEC/ADR либо `отсутствуют` |
| Заменяет | ID либо `отсутствует` |
| Заменён | ID либо `не заменён` |

## Контекст

Опишите product problem, действующие ограничения и причину, по которой решение
нужно сейчас. Отделите проверяемые факты от предпочтений.

## Решение

Сформулируйте одно decision-complete правило с MUST/SHOULD/MAY. Назовите
authoritative state, public boundary, validation и failure/fallback semantics.
Организационные роли и approval workflow не являются частью решения.

## Product impact

Опишите, какой player/developer loop становится лучше, что намеренно остаётся
вне scope и какая совместимость или migration потребуется.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Идентификатор | Короткий воспроизводимый product scenario | Наблюдаемое pass/fail поведение | Безопасное поведение при недоступной optional capability |

Оставляйте только проверки, относящиеся к риску решения: fast local, play loop,
persistence/replay, content/package либо targeted performance/platform.
Отдельные signed artifacts и approval flows не требуются.

## Рассмотренные варианты

Для каждого material варианта укажите причину принятия или отклонения и условия,
при которых решение следует пересмотреть.

## Последствия

Перечислите обязательные implementation changes, ограничения, migration и
failure semantics.

## Supersession

Смысл Accepted ADR меняется новым ADR. Новый документ заполняет `Заменяет`, а
старый становится compact historical stub со статусом `Superseded` и ссылкой
`Заменён`.
