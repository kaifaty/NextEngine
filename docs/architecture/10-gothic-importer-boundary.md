# SPEC-10: Граница Gothic importer

| Поле | Значение |
|---|---|
| ID | SPEC-10 |
| Статус | Accepted |
| Версия | 1.3 |
| Владелец | Repository Owner |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-03](03-assets-world-streaming-and-persistence.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-022](adr/022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](adr/023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |

## Public boundary и data flow

Gothic importer MUST быть отдельным Git repository, process и distributable от engine monorepo. На local bootstrap stage его working tree MAY физически находиться в `incubator/gothic-importer/`, только если весь каталог ignored parent repository, содержит собственный `.git`, не входит в parent Cargo workspace/path dependencies и может быть перемещён вместе со своей историей без filtering. Такая co-location не меняет repository/runtime/license boundary. `game`, `headless`, engine SDK и runtime packages MUST NOT link/load Gothic parsers, Daedalus VM, legacy archives или importer libraries. Интеграция происходит только через versioned `NeutralImportModel` (NIM) и provenance manifest/process command configured в ignored local config.

Importer читает только локальную installation, путь к которой явно предоставил пользователь. Он MUST NOT скачивать, искать, bundle или publish игровые данные. Engine repository, CI fixtures и binary packages MUST NOT содержать source/derived protected bytes, если отдельный written legal review явно не разрешил конкретный fixture.

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

## Split-fixture evidence boundary

`vertical-v1-import-smoke` MAY читать только явно переданную user installation во временном isolated root и публикует лишь sanitized provenance/hash metadata, validator result, load projection и source-access trace. NIM, cooked/imported bytes, screenshots, audio/video, source strings/paths и transient projections не входят в publishable evidence. До VS-09 ephemeral root уничтожается либо quarantined вне repository/cache/build/package/evidence roots.

VS-02…VS-15 и human-review media/evidence, bound by current `HumanReviewDecisionV2`/`AttestationEnvelopeV2`, используют только project-generated `vertical-v1-neutral` fixture с recorded CC0-1.0 provenance. Смешение fixture classes даёт `EVIDENCE_FIXTURE_CLASS_MIXED`; V1 review artifacts доступны только historical-audit mode по ADR-023, а технический import smoke не заменяет `LEGAL-IMPORT-01`.

## Whitelist pilot vertical slice

Importer v1 pilot MUST принимать только следующие категории:

| Категория | Разрешённый neutral результат | Явно не входит |
|---|---|---|
| Scene subset | static geometry, transforms, materials/textures references, collision tags, lights, bounded triggers/waypoints | полный world streaming semantics, portals/AI zones без mapping |
| Character | skeleton/skin/animations, generic Character archetype/stats subset, spawn transform | legacy class/runtime/AI VM identity |
| Items | mesh/material и generic Item archetype/instance fields из whitelist | произвольные engine callbacks |
| Scripted behavior | whitelisted declarative mapping в generic interaction/dialogue/quest fixture либо generated Luau stub requiring explicit author review | Daedalus execution, Ikarus/LeGo, arbitrary native/memory behavior |

Unknown/unsupported records MUST выдавать diagnostic с source locator и reason; importer не может silently approximate required record. Whitelist имеет version/hash и входит в NIM manifest.

## Mapping и cooking boundary

Importer не создаёт cooked engine bundles. Engine-side `next inspect import` и `next validate assets` проверяют NIM schema, hashes, provenance, units, references, whitelist и forbidden extensions; затем common cooker преобразует его как любой authoring source. Один и тот же NIM на Win/Linux MUST давать одинаковый platform-neutral ContentManifest.

Символы `GothicNpc`, `DaedalusInstance`, `ZenWorld` и `GothicItem` MAY существовать только в importer implementation, этом ограничивающем RFC или историческом ADR/evidence. Они MUST NOT быть type/schema names в NIM, cooked output, runtime code, WIT, Luau или save.

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
- Protected-data scanner positive → package/repository gate fails and artifact is quarantined, not uploaded.
- Importer path обнаружен в parent index/workspace/dependency graph → boundary gate fails; files удаляются из parent staging/workspace без удаления nested history.

## Legal and distribution gate

Before public importer release Security & Governance MUST obtain written legal review covering code provenance, user-provided installation workflow, derived outputs, trademarks, redistribution notices and sample fixtures. Этот RFC не является юридическим заключением. Без review engine MAY ship neutral demo content, но MUST NOT publicly ship importer; vertical-slice importer release criterion remains unpassed.

## Verification gates

| Gate | Сценарий | Threshold | Evidence | Fallback/rollback |
|---|---|---|---|---|
| IMPORT-P1 | same local fixture import twice Win/Linux | NIM canonical records/output hashes identical; path-independent IDs 100% | provenance/mapping manifests | fix normalization; no publish |
| IMPORT-P2 | whitelist pilot scene/character/items/behavior | 100% required whitelist records mapped/diagnosed; NIM validates/common cooker succeeds | coverage report, inspector output | narrow declared fixture only via reviewed whitelist revision |
| IMPORT-P3 | corrupt/fuzz/archive-bomb corpus 48 CPU-hours | 0 crash/UB/escape; all limits enforced; 0 partial publish | fuzz/sandbox report | quarantine input/block release |
| IMPORT-P4 | runtime binary/SBOM/source scan | 0 importer/legacy parser/VM dependency and 0 forbidden legacy schema symbols | link map/SBOM/rg report | split packages/refactor |
| IMPORT-P5 | protected data scan repo/build/package | 0 matched protected data/signatures/raw imported bytes | scanner manifest | quarantine/delete generated artifact and rotate cache if uploaded |
| PRIVACY-02 | split import-smoke/neutral fixture, cleanup и prohibited-root corpus | 0 protected bytes во всех prohibited roots; 100% mixed/failed-cleanup cases blocked before publication | cleanup/source-access audit, scanner report, neutral fixture provenance | quarantine run; retain no publishable bundle |
| IMPORT-P6 | legal release review | written approval for exact importer release/fixtures or explicit `not approved` | legal record | importer not distributed; neutral content only |
| IMPORT-P7 | NIM → cook cached load | first cook passes; identical second run ≥90% cache hits; runtime scene loads with no source installation access | cook/run manifests | release block |
| IMPORT-P8 | ignored nested repository boundary | parent `git check-ignore` matches entire incubator; parent index/workspace/path-dependency scan contains 0 importer entries; nested `main` history/status valid | parent ignore/index/workspace report + nested Git log/status | remove from parent staging/workspace; retain/move nested repository |
