# Историческая ссылка: прежний evidence register

| Поле | Значение |
|---|---|
| ID | EVIDENCE-001 |
| Статус | Superseded |
| Версия | 2.0 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |
| Заменяет | EVIDENCE-001 1.9 active registry |

## Current rule

Next Engine больше не ведёт центральный нормативный реестр внешних
«доказательств», technology candidates или их статусов. Такой реестр быстро
устаревает, дублирует ADR/package metadata и не сообщает, работает ли продукт.

External fact проверяется тогда, когда он нужен конкретному решению или
изменению, и записывается рядом с этим контекстом:

| Информация | Текущее место |
|---|---|
| Выбор архитектурной технологии и fallback | профильный ADR |
| Exact source dependency, version и checksum | workspace lockfile и package manifest |
| License и redistribution metadata | dependency/content/model manifest и generated notices/SBOM |
| Platform support | toolchain/platform configuration и conditional `platform` ProductCheck |
| Runtime behavior, compatibility и performance | релевантный ProductCheck из SPEC-12 |
| Model/content provenance | content manifest или model card рядом с exact artifact |

ProductCheck output является локальным инженерным результатом. Он не
регистрируется здесь и не превращает technology в глобально «одобренную».

## Historical material

Полные таблицы EVIDENCE-001 версий 1.x сохранены в Git history и старых review
records, включая
[packet 1.8](../reviews/architecture/packet-1.8.md). Они объясняют, какие
внешние источники рассматривались при старых architecture packets, но их даты,
версии, maturity claims, old check IDs и статусы не являются current
authority.

Новые technology rows в этот файл не добавляются. Если историческая строка
нужна для нового решения, её факт перепроверяется, а актуальный вывод
фиксируется в новом ADR, lockfile или artifact metadata.
