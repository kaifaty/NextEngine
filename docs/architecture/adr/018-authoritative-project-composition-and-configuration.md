# ADR-018: Authoritative project composition и configuration classes

| Поле | Значение |
|---|---|
| ID | ADR-018 |
| Статус | Accepted |
| Версия | 1.0 |
| Владелец | Repository Owner |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Developer Experience Team, Security & Governance Team, Release Engineering |
| Дата решения | 2026-07-24 |
| Последняя проверка | 2026-07-24 |
| Нормативные зависимости | [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-014](014-deterministic-extensions-and-package-trust.md), [ADR-016](016-compositional-gameplay-budgets.md), [ADR-022](022-deterministic-command-identity-ledger-and-causal-identity.md), [ADR-023](023-human-review-decision-v2-and-offline-attestation.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## История принятия

ADR принят в architecture packet 1.7. Он устанавливает authored-manifest → exact-lock boundary, deterministic catalog-snapshot resolution, configuration ownership and atomic project activation. Принятие решения не объявляет runtime implementation, verification gate `PASS` или release conformance.

## Контекст

Accepted architecture перечисляет composition roots, content/manifests, package locks и save/replay hashes, но не определяет единый public `ProjectManifest`, exact project lock, configuration ownership или activation transaction. Без решения каждый root может получить собственный startup/config resolver, а user/environment overrides — неявно изменить authoritative simulation.

## Решение

1. Authored `ProjectManifest` MUST разрешаться до world activation в immutable `ProjectCompositionLock`. Resolver принимает только exact immutable `ProjectCatalogSnapshot` с verified canonical hash; mutable registry, network response, downloader or local cache не являются resolution authority.
2. Candidate release MUST быть non-yanked, satisfy every accumulated compatibility constraint and be ranked by SemVer 2.0.0 precedence descending. Prerelease допускается только когда exact full prerelease SemVer explicitly admitted authored/transitive requirement. Equal SemVer precedence MUST использовать lexicographically smallest canonical catalog-record SHA-256 as tie-break.
3. Dependency identities обрабатываются в canonical order с deterministic backtracking до first complete closure. Cycle, incompatible range, hash/trust/budget conflict or exhausted required dependency produces one canonical `ProjectResolutionConflictReport`; optional dependency MAY использовать только its declared fallback. Runtime MUST NOT resolve floating ranges or silently substitute another record.
4. Lock MUST bind exact catalog/resolver trace; engine/schema/runtime profile; content; package/script/plugin/model; trust/capability; budget; authoritative configuration; migration; fallback; and target presentation hashes. Save/replay compatibility references the lock hash, not authored ranges or current registry state.
5. `game`, `headless` и `capture-worker` MUST использовать один lock и одинаковую authoritative configuration, validation, persistence/replay and domain semantics; presentation adapters являются declared non-authoritative subset.
6. Каждый configuration key MUST принадлежать ровно одному owner и class: `Authoritative`, `PresentationOnly` or `DeveloperOnly`. User/environment state не перекрывает authoritative class, trust policy or budget. Любое authoritative изменение создаёт новый lock до world activation.
7. Project activation является atomic composition transaction `resolve → validate → stage → activate`; required failure discards staging, optional failure использует только declared fallback.
8. Composition activation MUST оставаться отдельной от OS process/window/device/user-session lifecycle. Focus, suspend/resume, device recreation, reconnect or OS session change cannot trigger resolution, change lock or publish project state. Process termination follows quiesce/close or recovers from the last atomic publication.
9. Runtime public boundary содержит only engine-owned values, canonical hashes and immutable manifests, а не filesystem paths, package-manager/registry objects, OS handles, network sessions or vendor config types.
10. Startup, shutdown and recovery MUST preserve the last published project/content/save, never invent a replacement lock and emit stable structured diagnostics.

## Рассмотренные варианты

- Отдельный resolver в каждом composition root — `Rejected`: создаёт domain drift и разные failure semantics.
- Resolution against live registry/network/cache state — `Rejected`: один и тот же manifest получил бы разные closure без нового exact input hash.
- First/source-order compatible version — `Rejected`: registry ordering становится скрытым resolver input; selected release не обязательно highest compatible.
- Implicit prerelease admission или automatic use of yanked record — `Rejected`: authored policy и trust provenance становятся неоднозначными.
- Runtime resolution floating dependencies — `Rejected`: ломает replay/save provenance и offline reproducibility.
- Environment/CLI как общий override layer — `Rejected`: скрыто меняет authoritative state и затрудняет evidence.
- Один mutable global config object — `Rejected`: создаёт dual ownership и hot-change nondeterminism.
- Считать OS/window/user session lifecycle project activation — `Rejected`: adapter restart смог бы незаметно заменить authoritative composition.
- Best-effort запуск с отсутствующим required package — `Rejected`: приводит к partial registries и недостоверному save state.

## Последствия

- Tooling обязан snapshot-ить exact bounded catalog input, создавать/проверять exact lock и canonical conflict report and explain resolution decisions.
- Project authors могут использовать ranges, но явно различают required/optional dependencies, exact prerelease admission and fallback.
- Catalog reorder, network availability or local cache state не меняют result; новый snapshot/yank state создаёт новый resolution input and lock.
- User settings остаются свободными только внутри presentation allowlist.
- Save/replay compatibility становится строже, но ошибки возникают до world mutation и имеют воспроизводимый diagnosis.
- Platform/session adapters могут restart независимо, пока exact active project lock and authoritative state остаются неизменными.

## Gates и fallback

Implementation conformance требует `PROJECT-P1`, `PROJECT-P2`, `LIFECYCLE-P1` и `CONFIG-P1`. Failing resolver/lock/config/activation candidate rejected; fallback — previous exact valid lock либо clean project rejection. Gate retry не может скрыть `NONDETERMINISTIC_RESULT`. Accepted ADR status сам по себе не создаёт gate `PASS`.

## Supersession

ADR не заменяет предыдущее решение. Изменение exact-snapshot/highest-compatible resolver rule, canonical hash tie-break, exact-lock hash closure, разрешение authoritative user/environment override, root-specific domain semantics, OS/session-driven composition change или partial activation потребует нового superseding ADR.
