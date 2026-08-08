# SPEC-00: Продуктовый контракт

| Поле | Значение |
|---|---|
| ID | SPEC-00 |
| Статус | Accepted |
| Версия | 2.2 |
| Последнее изменение | 2026-08-08 |
| Нормативные зависимости | [INDEX-001](README.md), [ADR-001](adr/001-product-repository-license-and-platforms.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md) |

## Назначение

Next Engine — независимый open-source runtime и toolchain для системных
single-player RPG. Продукт строится вокруг playable loop, generic RPG rules,
иерархического agent AI, физических персонажей, streaming world и расширяемого
контента. Это не OpenGothic port и не универсальный engine для всех жанров.

Главный приоритет v1 — как можно раньше получить игру, в которую можно играть,
которую можно сохранять, воспроизводить, расширять пакетами и запускать offline.
Архитектура не требует отдельной release-процессной системы, не влияющей
непосредственно на этот результат.

## Обязательный продуктовый результат v1

V1 MUST:

1. запускать один cooked project на Windows x86_64 и Linux x86_64;
2. иметь interactive `game`, deterministic `headless` и `tools`;
3. реализовывать generic Character, Item, Quest, Dialogue, Faction,
   InteractiveObject и streamed world без legacy runtime types;
4. поддерживать движение, взаимодействие, combat, dialogue и quest loop;
5. сохранять versioned state, загружать его и воспроизводить внешний command
   stream;
6. оставаться корректной и playable без сети, LLM и `ai-host`;
7. использовать engine-owned physics, motor и animation contracts с
   процедурным fallback для learned policy;
8. выполнять Luau/Wasm только через capabilities, deterministic resource limits
   и общий `WorldCommand` validator;
9. cook, validate и inspect neutral assets и bounded importer output;
10. реализовывать first-party mechanics тем же public package API, который
    доступен community packages;
11. разрешать authored project в immutable exact `ProjectLockV3`;
12. использовать device-independent player actions и не позволять UI,
    camera или renderer менять gameplay state;
13. хранить durable state в versioned engine-owned schemas; несовместимый
    alpha input отклоняется typed до mutation, а migration появляется только
    после первого публично поддерживаемого predecessor;
14. выполнять async jobs, streaming и I/O через bounded staging без partial
    authoritative publication;
15. предоставлять короткие локальные product checks для gameplay,
    persistence/replay и content/package paths;
16. запускать runtime-bearing roots через один application-session lifecycle с
    exactly-once final save, durable close receipt и restart recovery.

`capture-worker`, external `ai-host`, imported Gothic content, policy training,
advanced renderer features и remote CI являются optional. Их отсутствие не
блокирует разработку остальных частей продукта.

## Инварианты продукта

- Gameplay state меняется только принятыми `WorldCommand`.
- У каждого mutable field есть один технический source of truth.
- Committed command публикует либо все свои state changes/events, либо ничего.
- Simulation correctness не зависит от renderer frame rate, wall clock, network
  availability, LLM latency, worker count, cache warmth или I/O completion order.
- Durable/public data использует `PersistentId` и `AssetId`; `RuntimeEntityId`
  остаётся ephemeral.
- Vendor, OS, ECS-backend и importer types не пересекают public contract boundary.
- `game` и `headless` используют одинаковые command validation, system order,
  project activation, application coordination, persistence и replay semantics.
- Verification код только сравнивает production outcomes и не является
  dependency или execution path production composition root.
- Corrupt, oversized, incompatible или ambiguous authoritative input rejected
  до publication; prior complete generation остаётся доступной.
- Presentation, UI, camera, audio и diagnostic state не являются gameplay
  authority.
- Optional AI/model/plugin/backend failure снижает качество либо отключает
  feature через declared fallback, но не повреждает мир и не создаёт partial
  commit.
- Runtime не обучает и не меняет neural weights. Model/policy assets immutable
  и content-addressed.
- First-party content не получает скрытых API.
- Game installations, protected imported data, datasets, checkpoints, secrets и
  generated caches не входят в parent repository или distributable.

## Лёгкая проверка продукта

Обязательная проверка состоит из четырёх независимых действий:

| Check | Что доказывает |
|---|---|
| `fast` | форматирование, compilation, lint, unit/integration tests и dependency boundaries |
| `play` | affected playable loop проходит через production input/command paths |
| `persistence-replay` | save/load и replay сохраняют expected authoritative state |
| `content-package` | affected neutral content/package cook, validation и load работают |

`platform` и `performance` запускаются только для изменения, которое затрагивает
соответствующую область. Failure одного check блокирует только непосредственно
затронутую feature/release claim.

Checks возвращают exit status и понятный diagnostic. Logs, screenshots,
captures, profiles и minimized replay можно сохранить для отладки, но
отдельный validation dossier или serialized approval для этого не нужен.

## Вне v1

- full editor;
- multiplayer/network-authoritative simulation;
- consoles, mobile и macOS shipping;
- universal renderer/backend marketplace;
- runtime model training;
- branching/counterfactual replay; v1 requires read-only replay of recorded
  authoritative input;
- обязательный Gothic importer release;
- формальная compliance/release-process platform.

Такие направления требуют отдельного продуктового решения только когда
появляется реальная потребность.
