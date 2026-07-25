# ADR-006: Luau gameplay и Wasm Component Model plugins

| Поле | Значение |
|---|---|
| ID | ADR-006 |
| Статус | Superseded |
| Версия | 1.0.1 |
| Дата решения | 2026-07-22 |
| Заменён | [ADR-014](014-deterministic-extensions-and-package-trust.md) |

Это решение исторически разделило удобный Luau gameplay scripting и изолированные Wasm plugins. Оба пути ограничивались capabilities, immutable views и validated command sink; trap, overrun или denied operation должны были отбрасывать незавершённые proposals, не открывая прямой mutable доступ к миру.

Действующий контракт расширений, deterministic resource limits и package isolation определяет только [ADR-014](014-deterministic-extensions-and-package-trust.md). Этот файл сохраняет исходную техническую мотивацию и не устанавливает текущих требований.
