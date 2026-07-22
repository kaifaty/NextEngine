# ADR-018: Authoritative project composition и configuration classes

| Поле | Значение |
|---|---|
| ID | ADR-018 |
| Статус | Proposed |
| Версия | 0.1 |
| Владелец | Core Architecture |
| Требуемые согласующие | Architecture Working Group, Runtime Team, Asset & Persistence Team, Developer Experience Team, Security & Governance Team, Release Engineering |
| Дата предложения | 2026-07-22 |
| Последняя проверка | 2026-07-22 |
| Нормативные зависимости | [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-01](../01-system-architecture.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-11](../11-security-licensing-and-governance.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-007](007-identities-persistence-and-replay.md) |
| Заменяет | отсутствует |
| Заменён | не заменён |

## Статус предложения

ADR входит в foundation-completeness proposal и не изменяет Accepted packet 1.4, remediation candidate 1.5 или dialogue/model proposal. До атомарного promotion всего foundation track lifecycle state — `AwaitingReview`.

## Контекст

Accepted architecture перечисляет composition roots, content/manifests, package locks и save/replay hashes, но не определяет единый public `ProjectManifest`, exact project lock, configuration ownership или activation transaction. Без решения каждый root может получить собственный startup/config resolver, а user/environment overrides — неявно изменить authoritative simulation.

## Решение

При принятии:

1. Authored `ProjectManifest` разрешается до запуска в immutable `ProjectCompositionLock` с exact schema/content/package/script/plugin/model hashes.
2. `game`, `headless` и `capture-worker` используют один lock и одинаковую authoritative configuration; presentation adapters являются declared subset.
3. Каждый key относится к `Authoritative`, `PresentationOnly` или `DeveloperOnly`. User/environment state не перекрывает authoritative class.
4. Authoritative change создаёт новый lock до world activation и становится save/replay compatibility input.
5. Activation является pre-world atomic transaction `resolve → validate → stage → activate`; required failure discards staging, optional failure использует только declared fallback.
6. Runtime public boundary содержит engine values and hashes, а не filesystem paths, package-manager objects, OS handles или vendor config types.
7. Startup, shutdown и recovery сохраняют последний published project/save и выдают stable structured diagnostics.

## Рассмотренные варианты

- Отдельный resolver в каждом composition root — `Rejected`: создаёт domain drift и разные failure semantics.
- Runtime resolution floating dependencies — `Rejected`: ломает replay/save provenance и offline reproducibility.
- Environment/CLI как общий override layer — `Rejected`: скрыто меняет authoritative state и затрудняет evidence.
- Один mutable global config object — `Rejected`: создаёт dual ownership и hot-change nondeterminism.
- Best-effort запуск с отсутствующим required package — `Rejected`: приводит к partial registries и недостоверному save state.

## Последствия

- Tooling обязан создавать/проверять exact lock и объяснять resolution decisions.
- Project authors явно различают required/optional dependencies и fallback.
- User settings остаются свободными только внутри presentation allowlist.
- Save/replay compatibility становится строже, но ошибки возникают до world mutation и имеют воспроизводимый diagnosis.

## Gates и fallback

Acceptance требует полного `PROJECT-P1`, `PROJECT-P2`, `LIFECYCLE-P1` и `CONFIG-P1` набора SPEC-17. Failing lock/config/activation candidate остаётся `Proposed`; fallback — previous exact valid lock либо clean project rejection. Gate retry не может скрыть `NONDETERMINISTIC_RESULT`.

## Promotion и supersession

ADR не supersede ADR-002/006/007/010/011. Он принимается только вместе с SPEC-17…20 и ADR-019…021 после согласований. Изменение exact-lock rule, разрешение authoritative user/env override, root-specific domain semantics или partial activation потребует нового superseding ADR.
