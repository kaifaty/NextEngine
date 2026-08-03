# ADR-044: Neutral text-catalog schema и deterministic locale fallback

| Поле | Значение |
|---|---|
| ID | ADR-044 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-03 |
| Последняя проверка | 2026-08-03 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-18](../18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-019](019-canonical-player-actions-and-presentation-authority.md), [ADR-025](025-schema-content-and-migration-authority.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |

## Контекст

SPEC-18 §Localization требует: durable content references только stable
text/voice/subtitle `AssetId` или namespaced text ID (never rendered text);
locale — валидный BCP-47 tag с deterministic fallback chain, declared by
project content; formatting только через typed arguments с explicit
number/date/unit rules; missing string/format/glyph → stable diagnostic
`LOCALIZATION_RESOURCE_MISSING` плюс declared source/default locale или
readable placeholder; localized resources остаются presentation state и не
могут менять authoritative ordering или outcome (REQ-093).

Semantic UI package (sub-increments 1–3) уже публикует `UiTextRefV1` со
stable text IDs и typed arguments, но SPEC-24 neutral schema catalog не
содержит схемы, способной нести localized string payload: generic
`NeutralRecordV1` properties — это пары `(SchemaId, SchemaId)` без string
данных. Без этого решения text IDs не имеют production content path, а
formatting через host locale/ICU сделал бы rendering зависимым от host
environment и нарушил бы replay-safety REQ-093.

## Решение

### 1. Typed `TextCatalogV1` neutral content record

Новая engine-owned typed neutral content schema
`nextengine.content.text-catalog` (`SchemaRoleV1::NeutralContent`,
`SchemaEncodingV1::CanonicalBinaryV1`). Record содержит:

- `catalog_asset_id: AssetId` и positive monotonic `revision: u32`;
- `locale`: bounded BCP-47 subset tag — ASCII lowercase, hyphen-separated
  alphanumeric subtags, первый subtag alpha 2–8, суммарно не более 35
  символов; canonical stored form lowercase;
- `fallback_locale_or_none`: optional locale tag, образующий declared
  fallback chain (см. §2);
- bounded entries `(text_id: SchemaId, template: NFC string)`, canonical
  sort по `text_id`, без дубликатов. Template placeholders — positional
  `{0}`..`{15}` ровно под typed arguments `UiTextRefV1`
  (`UI_MAX_TEXT_ARGUMENTS = 16`); braces вне placeholder token invalid.

Maximums: 4 096 entries per catalog, 1 024 bytes per template.

### 2. Locale fallback closure при cook

Fallback chain объявляется content: каждый catalog указывает своё fallback
locale. Cook MUST reject project content, если нарушено любое из:

- locale tags уникальны внутри project content set;
- ровно один catalog имеет `fallback_locale_or_none = None` — это declared
  source/default locale;
- каждый declared fallback locale имеет catalog в том же project content
  set;
- fallback pointer chains ацикличны и завершаются в source locale.

Нарушение — fail-closed отказ до content publication под существующими
`CONTENT_SCHEMA_MISMATCH` / `CONTENT_BOUNDS_INVALID`; clamp, implicit default
locale и best-effort chain repair запрещены.

### 3. PresentationOnly semantic class

Text catalogs публикуются в `ContentManifestV1` с
`ContentSemanticClassV1::PresentationOnly`: localized text никогда не входит
в domain closure и не может изменить gameplay hash, command ordering или
replay outcome. Pseudo-locale — это ещё один обычный catalog (например
`qps-ploc` с fallback в source locale), а не особый код.

### 4. Deterministic resolution semantics (presentation boundary)

Resolution — presentation work (SPEC-18 §Scheduling): чистая функция над
cooked catalogs и immutable `UiTextRefV1`, выполняется вне authoritative
tick и не может влиять на commit.

- Chain walk: requested locale catalog → declared fallback pointers →
  source locale; первый catalog, содержащий `text_id`, выигрывает.
- Typed formatting: `SignedInteger` — plain decimal ASCII без grouping
  (explicit number rule V1); `TextId` — recursive resolution в той же
  locale chain, depth ≤ 8, cycle detection через visited set. Date/unit
  rules и plural/gender forms в V1 отсутствуют и появятся только как новая
  schema version.
- Missing entry по всей chain, missing requested-locale catalog, missing
  argument index, TextId cycle/depth overflow — bounded fallback: stable
  diagnostic `LOCALIZATION_RESOURCE_MISSING` плюс readable placeholder
  `[{text_id}]`. Command/content IDs неизменны; resolution никогда не
  паникует и не читает host locale/wall clock.

### Alternatives

- **Generic `NeutralRecordV1` properties для текста** — rejected:
  properties не несут string payload; расширение generic envelope строками
  смешало бы definition records и localization content с разными failure
  semantics.
- **Rendered text как durable identity** — запрещено SPEC-18 §Localization
  и glossary `UiSemanticSnapshot`.
- **Host ICU/CLDR formatting и locale collation** — rejected: host
  environment определял бы presentation bytes и потенциально replay
  evidence; formatting rules обязаны быть explicit и simulation-owned.
- **Compiled text-registry sidecar artifact** — отложено без смены
  контракта: `ContentManifestV1` + blobs уже дают deterministic lookup по
  schema id; compiled form может появиться с widget adapter как private
  presentation cache, не становясь вторым source of truth.

## Последствия

- SPEC-24 обновляется до v1.1: секция `nextengine.content.text-catalog` в
  neutral schema catalog; routing table и traceability map обновлены в том
  же change.
- Contracts получают `TextCatalogV1` с canonical codec, bounds и
  fail-closed validation; project cooker валидирует fallback closure и
  публикует catalogs как PresentationOnly manifest entries + blobs.
- Reference project объявляет source locale `en` и pseudo-locale
  `qps-ploc` catalogs для Semantic UI text IDs.
- Presentation crate получает deterministic resolver; widget adapter
  (Semantic UI sub-increment 5) потребляет его без новых контрактов.
- ACCESS-P1 сценарии source/missing/pseudo locales получают production
  content path.
