# ADR-018: Authoritative project composition and configuration classes

| Поле | Значение |
|---|---|
| ID | ADR-018 |
| Статус | Accepted |
| Версия | 1.1 |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-25 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-030](030-product-first-development-and-lightweight-validation.md) |
| Заменяет | отсутствует |
| Заменён | частично [ADR-030](030-product-first-development-and-lightweight-validation.md) |

> **Supersession note (2026-08-08):** [ADR-048](048-direct-exact-project-lock.md)
> заменяет resolver/catalog semantics, ProjectManifest→catalog→composition-lock
> pipeline и recovery/shutdown policy closure этого ADR. Сохраняются
> configuration classes, immutable exact activation и parity composition roots.

## Process baseline ADR-030

[ADR-030](030-product-first-development-and-lightweight-validation.md)
заменяет прежний общий admission lifecycle. Authoritative
manifest-to-lock resolution, configuration classes и atomic activation остаются
техническим решением.

## Контекст

Accepted architecture перечисляет composition roots, content manifests, package
locks и save/replay hashes, но без единого public `ProjectManifest`, exact
project lock, однозначной привязки configuration keys и activation transaction
каждый root мог бы получить собственный resolver. User/environment overrides
тогда неявно меняли бы authoritative simulation.

## Решение

1. Authored `ProjectManifest` разрешается до world activation в immutable
   `ProjectCompositionLock`. Resolver принимает только exact immutable
   `ProjectCatalogSnapshot` с verified canonical hash; mutable registry, network
   response, downloader и local cache не являются resolution authority.
2. Candidate release должен быть non-yanked, удовлетворять всем accumulated
   compatibility constraints и сортироваться по SemVer 2.0.0 precedence
   descending. Prerelease допускается только когда exact full prerelease SemVer
   явно разрешён authored/transitive requirement. Equal SemVer precedence
   использует lexicographically smallest canonical catalog-record SHA-256.
3. Dependency identities обрабатываются в canonical order с deterministic
   backtracking до first complete closure. Cycle, incompatible range,
   hash/capability/budget conflict или exhausted required dependency создаёт
   canonical `ProjectResolutionConflictReport`; optional dependency использует
   только declared fallback. Runtime не разрешает floating ranges и не
   подменяет record.
4. Lock связывает exact catalog/resolver trace, engine/schema/runtime profile,
   content, package/script/plugin/model hashes, capability policy, budgets,
   authoritative configuration, migrations, fallbacks и target presentation
   hashes. Save/replay compatibility ссылается на lock hash, а не authored
   ranges или current registry state.
5. `game` и `headless` используют один lock и одинаковую authoritative
   configuration, validation, persistence/replay и domain semantics.
   Presentation adapters являются declared non-authoritative subset.
6. Каждый configuration key принадлежит ровно одному engine-owned
   configuration object и class: `Authoritative`, `PresentationOnly` или
   `DeveloperOnly`. User/environment state не перекрывает authoritative class,
   capability/security policy или budgets. Authoritative изменение создаёт
   новый lock до world activation.
7. Project activation является atomic composition transaction
   `resolve → validate → stage → activate`; required failure discards staging,
   optional failure использует только declared fallback.
8. Composition activation отделена от OS process/window/device/user-session
   lifecycle. Focus, suspend/resume, device recreation, reconnect и OS session
   change не запускают resolution, не меняют lock и не публикуют project state.
9. Runtime public boundary содержит только engine-owned values, canonical hashes
   и immutable manifests, а не filesystem paths, registry objects, OS handles,
   network sessions или vendor config types.
10. Startup, shutdown и recovery сохраняют last published
    project/content/save, не изобретают replacement lock и выдают stable
    structured diagnostics.

## Рассмотренные варианты

- Отдельный resolver в каждом composition root — `Rejected`: создаёт domain
  drift и разные failure semantics.
- Resolution against live registry/network/cache state — `Rejected`: один
  manifest получал бы разные closure без нового exact input hash.
- First/source-order compatible version — `Rejected`: registry ordering
  становится скрытым input.
- Implicit prerelease или automatic use of yanked record — `Rejected`: package
  provenance становится неоднозначной.
- Runtime resolution floating dependencies — `Rejected`: ломает replay/save
  provenance и offline reproducibility.
- Environment/CLI как общий override layer — `Rejected`: скрыто меняет
  authoritative state.
- Один mutable global config object — `Rejected`: создаёт dual state authority
  и hot-change nondeterminism.
- Считать OS/window/user session lifecycle project activation — `Rejected`:
  adapter restart смог бы заменить authoritative composition.
- Best-effort запуск без required package — `Rejected`: приводит к partial
  registries и недостоверному save state.

## Product checks

| Сценарий | Ожидаемый результат | Fallback |
|---|---|---|
| Один manifest разрешается из одинакового catalog snapshot при разном record/source order | Lock hash и conflict report exact; highest compatible version и hash tie-break не меняются | Сохранить previous exact valid lock либо cleanly reject project |
| Required dependency missing, cycle/range/hash/capability conflict или malformed lock | Failure происходит до world mutation; staging не публикуется; diagnostic canonical | Использовать только declared optional fallback; required failure оставляет предыдущую publication |
| Environment/CLI пытается изменить `Authoritative` key после lock creation | Override отвергается; presentation/developer settings меняются только в своих classes | Использовать locked value и declared presentation default |
| OS focus/device/session restart происходит при active project | Active lock и authoritative state не меняются; adapter восстанавливается независимо | Quiesce/recreate adapter либо recover last atomic publication |

## Последствия

- Tooling snapshot-ит bounded catalog input, создаёт и проверяет exact lock и
  canonical conflict report.
- Project authors могут использовать ranges, но явно различают required и
  optional dependencies, prerelease admission и fallback.
- Catalog reorder, network availability и local cache state не меняют result.
- User settings свободны только внутри presentation allowlist.
- Save/replay compatibility строже, но errors возникают до world mutation.
- Изменение resolver ordering, hash tie-break, lock closure, authoritative
  override, root-specific semantics или partial activation требует нового ADR.
