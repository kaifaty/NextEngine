# ADR-010: Historical validation workflow

| Поле | Значение |
|---|---|
| ID | ADR-010 |
| Статус | Superseded |
| Заменён | [ADR-030](030-product-first-development-and-lightweight-validation.md) |

ADR-010 описывал единый headless-процесс проверки изменений для ранней
архитектурной baseline. По мере развития проекта этот процесс оказался
несоразмерен задачам bootstrap; ADR-030 заменил его локальными проверками,
выбираемыми по продуктовому риску, сохранив headless-сценарии как обычный
инженерный инструмент.
