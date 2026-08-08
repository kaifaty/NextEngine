# ADR-051: R3a packaged chunk streaming commit boundary

| Поле | Значение |
|---|---|
| ID | ADR-051 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-08 |
| Последняя проверка | 2026-08-08 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-02](../02-runtime-ecs-and-data.md), [SPEC-03](../03-assets-world-streaming-and-persistence.md), [SPEC-17](../17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](../21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-22](../22-schema-registry-compatibility-and-migration.md), [SPEC-24](../24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](../25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [ADR-026](026-deterministic-work-resource-and-streaming-admission.md), [ADR-046](046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-048](048-direct-exact-project-lock.md) |
| Заменяет | Proposed R3a implementation intent in SPEC-23; не принимает generic scheduler/resource framework |

## Контекст

R2 доказывал канонический переход между `relay-station` и `frontier`, но
activation уже держала весь decoded project в памяти, а World не выполнял
production fetch/decode packaged chunk blobs. Для снятия B-04 нужен один
реальный consumer с точным fixed-stage commit, а не преждевременный общий job,
residency или eviction subsystem.

Frontier в reference package содержит девять blobs общим encoded размером
6 714 байт; максимальный blob — 1 273 байта. Эти данные достаточно малы для
узкого bounded vertical и не обосновывают universal scheduler.

## Решение

1. Assets предоставляет клонируемый `PinnedContentGeneration`. Pin один раз
   фиксирует generation из `CURRENT`, проверяет `INDEX.v1` и complete inventory,
   после чего verified reads не следуют за последующим переключением `CURRENT`.
   Внешний API не раскрывает filesystem path и ограничивает каждый blob.
2. `activate_project_package` возвращает eager `ActivatedProjectV3` вместе с
   pinned generation. R3a повторно читает нужные chunk blobs с диска; весь
   project не становится lazy-loaded.
3. World строит immutable revision-bound request из exact project lock,
   content/schema/partition hashes, topology revision, world generation,
   target binding и канонически упорядоченных chunk/dependency revisions.
4. Приватный профиль R3a ограничивает request 64 assets, blob 1 MiB, группу
   16 MiB, workers четырьмя (production default два), result channel 64 и
   canonical decode одним MiB total bytes. Другие существующие структурные
   decode limits сохраняются.
5. Workers только читают и декодируют `NeutralRecordV1`. Они не получают
   mutable World/Runtime access. Результаты объединяются по `AssetId`;
   каноническая ошибка выбирается по `(AssetId, stable error code)`, независимо
   от completion order, worker count, I/O timing или cache warmth.
6. До publication проверяются file/index/blob hashes, canonical encoding,
   schema ref, semantic class, asset/record identity, exact chunk dependency
   set, external project-global dependencies и уникальность `PersistentId`.
   Приватный result hash связывает request hash, ordered revisions, record IDs
   и logical resource counters.
7. Runtime имеет fixed `WorldStreamingCommit` между `IngressCommit` и
   `PhysicalStep`. `PreparedRuntimeWorldTick` и `ValidatedRuntimeWorldTick`
   валидируют Runtime и World вместе; их final publication не содержит
   fallible work. Ошибка любой стороны отбрасывает обе prepared generations.
8. Переход использует два уже существующих gameplay ticks: первый публикует
   `Requested`, затем simulation advancement ждёт mandatory packaged load, а
   следующий tick атомарно публикует completion. I/O duration меняет только
   wall-clock pause, не выбранный simulation tick или authoritative root.
9. Live snapshot публикует только `Requested` и конечные `Active`/`Unloaded`.
   `Staged`/`Validated` reconstructible. Restore любого pending состояния
   заново строит request и fetch-ит package; staging bytes не сохраняются и не
   считаются authority.
10. Fetch/decode/validation fault сохраняет один declared `Requested` root,
    предыдущие active chunks, generation и decoded cache. Mandatory request
    остаётся доступен для retry/restart.

## Границы решения

R3a не добавляет типов в `crates/contracts` и не меняет wire formats
`ProjectLockV3`, `WorldPartitionManifestV1`, content/index, save, replay или
`WorldStreamingSnapshotV1`. Generic scheduler, cancellation tree, pins/leases,
eviction, residency interests, population tiers и R3b multiregion policy не
принимаются этим ADR. SPEC-23 остаётся Proposed future intent без routing
authority.

## Последствия

- Production construction `WorldStreamerV1` требует package source.
- `game` и `headless` используют один Assets/World/Runtime commit path.
- Reference Save после `Requested` и process restart повторяют fetch и приходят
  к тому же root без доверия к transient staging.
- Performance V4 заполняет существующий
  `logical_resource_charges.required_staging_bytes`; allocator и новый wire
  format не требуются.
- Только smoke scenario hash меняется из-за packaged I/O и Runtime stage.

## Проверка

- focused Assets/Project/World/Runtime/Reference Game/Verification tests;
- `cargo run -p xtask -- host-check`;
- `cargo run -p xtask -- play`;
- `cargo run -p xtask -- persistence-replay`;
- `cargo run -p xtask -- content-package`;
- `cargo run --release -p xtask -- performance --scenario smoke --mode report`.

`platform` не требуется: boundary не меняет platform host, renderer, target
packaging или OS adapter. R3b `r3-multiregion-streaming` остаётся `NOT_RUN`.
