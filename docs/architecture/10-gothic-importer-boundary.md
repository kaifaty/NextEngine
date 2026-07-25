# SPEC-10: Граница Gothic importer

| Поле | Значение |
|---|---|
| ID | SPEC-10 |
| Статус | Accepted |
| Версия | 1.4 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md) |
| Заменяет | отсутствует |

## Public boundary и data flow

Gothic importer — optional tool, отдельный Git repository, process и distributable от engine monorepo. Его отсутствие, сбой или неподдерживаемая source installation не блокируют build, запуск, сохранения, replay или neutral authoring Next Engine. На local bootstrap stage его working tree MAY физически находиться в `incubator/gothic-importer/`, только если весь каталог ignored parent repository, содержит собственный `.git`, не входит в parent Cargo workspace/path dependencies и может быть перемещён вместе со своей историей без filtering. Такая co-location не меняет repository/runtime/license boundary. `game`, `headless`, engine SDK и runtime packages MUST NOT link/load Gothic parsers, Daedalus VM, legacy archives или importer libraries. Интеграция происходит только через versioned `NeutralImportModel` (NIM) и provenance manifest/process command configured в ignored local config.

Importer читает только локальную installation, путь к которой явно предоставил пользователь. Он MUST NOT скачивать, искать, bundle или publish игровые данные. Engine repository, test fixtures, caches и binary packages MUST NOT содержать source или derived protected bytes.

Нормативный поток однонаправлен: `user-provided installation → sandboxed importer → NIM + provenance → engine validator/cooker → neutral bundles`. Engine runtime не вызывает importer и не передаёт mutable state обратно в legacy formats.

## Source of truth и ownership

До export локальная пользовательская installation является source input, а importer владеет parsing/mapping diagnostics. После успешного export NIM + provenance является единственным boundary artifact; cooker владеет дальнейшей validation/transformation. Imported runtime result после cooking подчиняется ContentManifest и не сохраняет legacy type semantics.

## NeutralImportModel

NIM является строгим подмножеством NeutralAuthoringModel и содержит:

- schema/version и importer build/hash;
- source installation fingerprint без raw absolute path;
- records с generic AssetId/PersistentId, canonical units/coordinates и generic types;
- dependency graph и explicit unsupported/dropped record diagnostics;
- source locator как non-sensitive logical archive/resource identifier + source hash;
- namespace mapping table и provenance/license classification;
- output files list с SHA-256.

Source-specific fields MAY находиться только в `diagnostic.extensions.importer.gothic.*`; cooker MUST удалить extensions из runtime bundle и MUST reject их использование как runtime rule.

## Namespace mapping

PersistentId/AssetId вычисляются детерминированно из importer namespace UUID, normalized source logical identity, record kind и stable instance key. Это explicit-ID boundary ADR-022: importer schema фиксирует namespace/provenance/collision policy, а runtime causal-ID algorithm здесь не применяется. Пользовательский filesystem path, install time и enumeration order не участвуют. Collision/duplicate mapping — fatal export error. Mapping manifest JCS-canonical; изменение algorithm является schema major + migration, не silent remap.

## Temporary output and protected-data boundary

Import run MAY читать user installation только из явно переданного source path и писать только в explicit isolated output root. NIM, cooked/imported bytes, screenshots, audio/video, source strings/paths и transient projections остаются локальными, вне repository, test fixtures, caches, build и package roots. Diagnostic output может содержать sanitized hashes, logical locators, provenance и license notices, но не raw protected bytes или absolute user paths.

Failed or cancelled run очищает incomplete output либо quarantines его в явно выбранном локальном root. Engine tests используют только project-generated neutral fixtures с recorded CC0-1.0 provenance; protected or imported bytes never become engine fixtures.

## Bounded import profile

Importer v1 pilot MUST принимать только следующие категории:

| Категория | Разрешённый neutral результат | Явно не входит |
|---|---|---|
| Scene subset | static geometry, transforms, materials/textures references, collision tags, lights, bounded triggers/waypoints | полный world streaming semantics, portals/AI zones без mapping |
| Character | skeleton/skin/animations, generic Character archetype/stats subset, spawn transform | legacy class/runtime/AI VM identity |
| Items | mesh/material и generic Item archetype/instance fields из whitelist | произвольные engine callbacks |
| Scripted behavior | whitelisted declarative mapping в generic interaction/dialogue/quest fixture либо generated Luau stub requiring explicit author inspection and validation | Daedalus execution, Ikarus/LeGo, arbitrary native/memory behavior |

Unknown/unsupported records MUST выдавать diagnostic с source locator и reason; importer не может silently approximate required record. Whitelist имеет version/hash и входит в NIM manifest.

## Mapping и cooking boundary

Importer не создаёт cooked engine bundles. Engine-side `next inspect import` и `next validate assets` проверяют NIM schema, hashes, provenance, units, references, whitelist и forbidden extensions; затем common cooker преобразует его как любой authoring source. Один и тот же NIM на Win/Linux MUST давать одинаковый platform-neutral ContentManifest.

Символы `GothicNpc`, `DaedalusInstance`, `ZenWorld` и `GothicItem` MAY существовать только в importer implementation, этом ограничивающем RFC или исторической документации. Они MUST NOT быть type/schema names в NIM, cooked output, runtime code, WIT, Luau или save.

## Diagnostics contract

Коды имеют namespace `IMPORT_*`: source unavailable/unsupported version/corrupt record, unsupported mapping, provenance missing, namespace collision, protected-output policy, NIM validation. Diagnostic содержит severity, logical source locator, record kind, safe context, suggested user action и whether record was skipped. Raw asset bytes и absolute user path default-redacted.

## Security и resource limits

Source является untrusted. Parser MUST проверять bounds/overflow/recursion/compression ratios, использовать sandboxed process permissions и quotas. Default limits задаются manifest: 4 GiB per source entry, 32 GiB total staged expansion, compression ratio 200:1, nesting depth 64; user может уменьшить. Превышение aborts current import без published NIM. Generated script/text остаётся data и не выполняется importer.

## Failure semantics

- Missing/corrupt/unsupported source → no export publish; staging quarantined/cleaned.
- Unsupported optional record → explicit warning + manifest count; required whitelist record → error.
- Provenance/hash absence → error, even if payload parsable.
- Engine NIM major mismatch → reject before cooking.
- Importer crash/malicious file → engine process unaffected; restart never trusts partial output.
- Protected-data scanner positive → artifact quarantined and import fails; engine product remains unaffected.
- Importer path обнаружен в parent index/workspace/dependency graph → importer setup invalid; path удаляется из parent index/workspace без удаления nested history.

## Distribution and notices

Importer MAY распространяться отдельно только с собственными open-source notices, mappings, provenance и понятным user-provided-installation workflow. Он MUST NOT redistribute source installation, imported output, protected samples, trademarks as product assets или user-specific paths. Если происхождение кода, sample или license classification неясно, importer/sample не распространяется; neutral engine product остаётся доступен.

## Product checks

| ID | Сценарий | Ожидаемый результат | Fallback |
|---|---|---|---|
| IMPORT-P1 | same local neutral fixture imported twice on Windows/Linux | NIM canonical records/output hashes identical; path-independent IDs 100% | fix normalization; publish nothing |
| IMPORT-P2 | bounded scene/character/items/behavior profile | 100% required records mapped or diagnosed; NIM validates and common cooker succeeds | narrow the declared profile; neutral authoring remains available |
| IMPORT-P3 | corrupt/fuzz/archive-bomb corpus | 0 crash, undefined behavior or sandbox escape; all limits enforced; 0 partial publish | quarantine input and disable importer path |
| IMPORT-P4 | runtime binary/source/dependency scan | 0 importer/legacy parser/VM dependency and 0 forbidden legacy schema symbols | split packages or remove importer integration |
| IMPORT-P5 | protected-data scan over repository/build/package/output roots | 0 protected signatures or raw imported bytes | quarantine local output, clear affected cache and publish nothing |
| IMPORT-P7 | NIM → deterministic cook → cached load | first cook passes; identical second run ≥90% cache hits; runtime scene loads without source installation access | reject NIM; engine and neutral content continue |
| IMPORT-P8 | ignored nested repository boundary | parent ignore covers importer; parent index/workspace/path-dependency scan has 0 importer entries; nested history/status intact | remove it from parent index/workspace or move the nested repository |
