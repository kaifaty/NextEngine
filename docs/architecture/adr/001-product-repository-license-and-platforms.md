# ADR-001: Продукт, репозитории, лицензия и платформы

| Поле | Значение |
|---|---|
| ID | ADR-001 |
| Статус | Accepted |
| Версия | 1.0.2 |
| Дата решения | 2026-07-22 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

## Частичное supersession ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежние process clauses этого ADR. Product scope, Apache-2.0 engine
license, repository/importer boundary и Windows/Linux v1 targets сохраняются.
Release work использует обычные license, provenance и protected-data checks.

## Контекст

Новый runtime должен развиваться независимо от OpenGothic, разрешать коммерческие закрытые игры поверх открытого engine и не тащить legacy formats в доменную модель. Одновременная поддержка desktop, consoles, multiplayer и полного editor размыла бы первый product milestone.

## Решение

- Next Engine MUST быть специализированным AI-first RPG engine, а не general-purpose framework.
- Engine monorepo MUST публиковаться под Apache-2.0. Игры, content packs и внешние сервисы MAY иметь закрытые лицензии и MUST общаться через публичные versioned contracts.
- Gothic importer MUST жить в отдельном репозитории и отдельном distributable. Его license/provenance check независим от лицензии engine.
- V1 MUST поддерживать Windows x86_64 и Linux x86_64. macOS, consoles и mobile отложены до superseding ADR.
- V1 MUST быть single-player. Архитектура MUST использовать stable PersistentId и replay, но не обязана обеспечивать network replication.
- V1 tooling MUST включать CLI cooker, validator, replay runner и inspectors. Полный scene editor не входит в v1.
- До выбора имени в коде и документации используется `Next Engine` и neutral namespaces, не `OpenGothic`.

## Рассмотренные варианты

- Перенос OpenGothic runtime — `Rejected`: это наследует C++/Tempest/Bullet/Daedalus boundaries и противоречит independent product.
- Один репозиторий engine+importer — `Rejected`: усложняет license/provenance boundary и увеличивает риск попадания защищённых данных.
- General-purpose engine — `Rejected`: размывает product focus physical embodiment, agent memory и RPG rules.
- macOS в v1 — `Rejected`: добавляет Metal/portability backend до работающего Windows/Linux baseline.

## Последствия

Engine runtime MUST NOT зависеть от importer packages. Repository scanner MUST блокировать legacy parsers и защищённые data signatures в engine artifacts. Любое добавление платформы, multiplayer или editor требует отдельного ADR и ресурсной оценки.

## Product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| Product boundary | Build engine and importer independently | Engine links no importer/legacy parser and contains no protected data | Block the offending dependency or artifact |
| Shipping baseline | Build representative Windows/Linux roots | Portable contracts and headless loop compile and run on the touched target | Keep the unsupported target outside v1 |

## Supersession

Только новый ADR может изменить лицензию, repository topology, v1 platforms или product scope.
