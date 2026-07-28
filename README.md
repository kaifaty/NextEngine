# Next Engine

**Открытый AI-first движок и набор инструментов для системных одиночных RPG.**

Next Engine создаётся для игр, в которых движение, бой, диалоги, квесты,
персонажи и изменения мира работают как части одной системы. Игровые решения
должны оставлять устойчивые последствия, мир — корректно сохраняться и
воспроизводиться, а AI — обогащать игру, не становясь условием её
работоспособности.

> **Статус:** ранний bootstrap. В репозитории уже работает детерминированный
> headless-срез с вводом, командами, RPG-состоянием, эталонной физикой,
> сохранением и replay, а neutral fixture проходит deterministic
> validate/cook/publish/activation и снабжает gameplay root authored
> dialogue/quest/relationship definitions. Portable `game` root уже извлекает
> immutable presentation snapshot и проходит B0 reference renderer; приватный
> SDL3/ash Vulkan adapter компилируется как `Proposed`, но ещё не прошёл
> обязательные Windows/Linux product checks. Первый data-only combat package
> уже проходит общий capability/effect/RPG transaction path. Двухчанковый
> deterministic streaming с отдельным save owner segment и первый
> deterministic NPC planner/procedural avatar fallback работают в cooked
> slice; deterministic Luau package host уже проходит sandbox/budget/state
> checks. Wasm Component host на pinned Wasmtime проходит N/N−1 negotiation,
> fuel/memory/capability/state checks и сохраняет обязательный headless
> fallback. Локальная v1 closure matrix связывает exact project/content/
> mechanics/extension roots; native Windows/Linux shipping evidence пока
> остаётся `NOT_RUN`.
> Название Next Engine временное.

Next Engine — самостоятельный проект. Это не порт OpenGothic и не универсальный
движок для любых жанров.

## Для каких игр создаётся Next Engine

Продукт сфокусирован на системных одиночных RPG: играх, где персонажи,
предметы, фракции, диалоги, квесты, интерактивные объекты и население мира
подчиняются согласованным правилам, а не существуют как набор изолированных
скриптов.

В основе продукта лежат пять идей:

- **Играбельность важнее инфраструктурной церемонии.** Короткий путь от
  изменения к работающему игровому циклу — главный приоритет разработки.
- **Мир помнит действия игрока.** Изменения состояния атомарны, сохранения
  версионируются, а replay позволяет воспроизвести внешние команды и найти
  первую причину расхождения.
- **AI усиливает игру, но не владеет ею.** Агентный AI, модели движения, LLM,
  ASR и TTS подключаются через ограниченные контракты. Без сети, модели или
  отдельного `ai-host` обязательный игровой цикл продолжает работать через
  детерминированный fallback.
- **Модификации — часть продукта, а не надстройка.** Собственные механики
  движка и пакеты сообщества должны использовать один публичный SDK, одни
  проверки полномочий и один путь изменения мира.
- **Физическое воплощение персонажа отделено от его RPG-роли.** Архитектура
  связывает generic Character, физический архетип, навыки движения и AI, не
  превращая конкретную модель, physics backend или ECS в публичный контракт.

## Целевая планка v1

V1 должна дать разработчику возможность:

1. запустить один подготовленный проект на Windows x86_64 и Linux x86_64;
2. пройти связанный цикл движения, взаимодействия, боя, диалога и квеста;
3. сохранять мир, продолжать игру после загрузки и воспроизводить записанный
   поток команд;
4. запускать тот же authoritative runtime интерактивно и без renderer через
   deterministic `headless`;
5. добавлять контент и механики пакетами на data-only, Luau и Wasm уровнях;
6. использовать физических персонажей и опциональные learned motor policies с
   процедурным fallback;
7. разрабатывать, проверять и инспектировать neutral content локальными
   инструментами;
8. запускать обязательный игровой цикл offline — без LLM, сети и внешнего
   AI-процесса.

Полноценный редактор, мультиплеер, консоли, мобильные платформы,
runtime-обучение моделей и
обязательная поставка Gothic importer не входят в v1.

## Что уже работает

Текущий bootstrap реализует фундамент, на котором можно наращивать игровой
цикл:

- Rust workspace с закреплённым toolchain `1.93.0` и
  `unsafe_code = "forbid"`;
- engine-owned идентификаторы, канонические команды и события, command ledger,
  immutable snapshots и versioned manifests;
- production-путь от device-independent `PlayerActionFrameV1` до проверенной
  команды и фиксированного simulation tick;
- транзакционный RPG-срез для диалога и квеста, отношений, передачи предметов,
  изучения навыков и состояния интерактивных объектов;
- целочисленный reference-контроллер grounded capsule, статические Box
  colliders и непрерывность контактов `Begin/Persist/End`;
- optional экспериментальный PhysX 5.9.0 backend для того же
  grounded-capsule сценария за safe engine-owned boundary; reference остаётся
  default и oracle;
- contact-gated interaction: действие игрока выбирает только объект в активном
  физическом контакте и меняет его RPG-состояние через Outcome
  `WorldCommand`;
- contact-gated pickup и equipment: отдельные `Inventory` и `Equipment`
  aggregates атомарно принимают предмет, переводят pickup proxy в collected
  state и назначают main-hand slot; immutable contact fact включается в hash
  RPG transaction plan;
- bounded NPC interaction: тот же semantic `interact` при контакте с authored
  quest-giver атомарно переводит dialogue и quest и обновляет отношение
  NPC→player через Outcome `WorldCommand`;
- canonical project/schema/content/world manifests, deterministic exact
  resolver и CC0 neutral fixture из scene, collider и RPG definition records;
  cooker публикует content-addressed generation атомарно, а production loader
  повторно проверяет lock, hashes, schema/reference closure и blobs;
- content-authored dialogue, quest, relationship и interaction definitions,
  собранные в обычный data-only package с exact mechanics lock и capability
  grants; `headless --project <store> --lock <hash>` запускает тот же
  authoritative сценарий из production activation path без `ai-host`;
- content-authored melee ability в ordinary package
  `org.nextengine.core.combat`: equipped training item публикует
  planner-visible affordance, immutable contact-bound `EffectRequestV1`
  компилируется в `AdjustCharacterResource`, а health commit проходит через
  тот же `RpgCommandV1` и capability grants, что доступны community packages;
- общий `game`/`headless` activation path с exact project lock и совпадающими
  authoritative state/ledger roots; `game` публикует immutable
  `PresentationSnapshotV2`, содержащий floor, capsule, switch, item и NPC;
- engine-owned platform/input lifecycle contracts, детерминированное
  presentation extraction и reference B0 render plan с изоляцией device loss;
  SDL3 `0.18.4` и ash `0.38.0` находятся только в приватном experimental
  adapter crate и не протекают в публичные contracts;
- атомарные поколения сохранений, восстановление, replay и проверка совпадения
  authoritative state roots;
- deterministic two-chunk admission: immutable worker results сходятся к
  canonical staging group, validated group публикуется атомарно, прежний chunk
  проходит `Quiescing → Unloaded`, а pending transition сохраняется отдельным
  `nextengine.world-services` owner segment и реконструируется после load;
- deterministic NPC planner читает immutable revision/hash-bound RPG,
  mechanics и contact projections, выбирает planner-visible affordance в
  canonical order и предлагает обычный `WorldCommand`; при отсутствии
  `ai-host`/model route публикуется procedural idle/locomotion/melee animation
  projection, не имеющая обратной записи в simulation;
- private Luau 728 adapter через pinned `mlua 0.12.0` выполняет package
  callbacks в read-only sandbox: scripts получают только capability-scoped
  immutable queries и typed proposals, а instruction/allocation/live-memory/
  host-call/command budgets, atomic proposal discard, persisted package state
  и three-strike circuit проверяются локальными product checks;
- private Wasmtime `45.0.0` adapter исполняет engine-owned WIT v3/v2
  Component worlds без ambient WASI: exact component hash, N/N−1
  compatibility selection, fuel per call/tick, linear-memory/table/instance
  limits, opaque resource handles, required/optional startup и canonical
  plugin state проверяются до общего mechanics/RPG command path;
- переносимый `headless` composition root и локальные product checks;
- изолированный экспериментальный путь
  `train → ONNX → inference` для проверки локального ML toolchain.

Это ещё не законченная игра: текущий `play` проверяет ограниченный neutral
сценарий движения, столкновения, pickup/equip, melee damage, активации switch,
один authored NPC dialogue/quest transition и переход во второй chunk с
возвратом. Reference physics profile пока ограничен upright capsule и
статическими Box colliders; desktop adapter пока отображает только
B0-примитивы и остаётся `Proposed`. Локальная closure проходит, но реальные
Windows/Linux platform/package gates ещё не выполнены.

## Быстрый старт

Нужен Rust `1.93.0`; версия закреплена в
[rust-toolchain.toml](rust-toolchain.toml).

Запустить текущий headless-срез:

```bash
cargo run -p next_headless

# Либо активировать exact ранее приготовленный content store
cargo run -p next_headless -- --project <cooked-store> --lock <composition-lock-sha256>
```

Запустить тот же cooked slice через `game` composition root и reference
presentation path:

```bash
cargo run -p next_game
cargo run -p next_game -- --project <cooked-store> --lock <composition-lock-sha256>

# Experimental SDL3/ash Vulkan B0 window; shipped status требует Windows/Linux checks
cargo run -p next_game --features desktop-sdl-ash -- --interactive
```

Запустить основные локальные проверки:

```bash
# Ввод → pickup/equip → player/NPC melee → dialogue/quest → chunk round-trip
cargo run -p xtask -- play

# Гравитация, столкновения с полом/стеной и жизненный цикл контактов
cargo run -p xtask -- physics-collision

# Сохранение → восстановление → replay и fallback повреждённого поколения
cargo run -p xtask -- persistence-replay

# Neutral fixture → deterministic cook → atomic publish → production activation
cargo run -p xtask -- content-package

# game/headless parity, normalized platform events и presentation/device-loss isolation
cargo run -p xtask -- platform

# Форматирование, clippy, тесты workspace и архитектурные границы
cargo run -p xtask -- host-check

# Полная локальная closure matrix с exact hashes и честными target NOT_RUN
cargo run -p xtask -- v1-closure
```

Команды выводят компактный машиночитаемый результат. `host-check` — канонический
широкий локальный `fast` check перед передачей изменения.

На native Windows/Linux target после зелёной matrix собирается атомарный
distribution directory с `game`, `headless`, exact cooked project и
`package.manifest.jcs`. Перед публикацией команда запускает release
`headless` и один bounded interactive Vulkan frame release `game`:

```bash
cargo run -p xtask -- v1-package --output dist/nextengine-v1
```

На macOS команда fail-closed возвращает
`TARGET_PACKAGE_REQUIRES_NATIVE_WINDOWS_OR_LINUX_X86_64`.

Экспериментальный PhysX backend не входит в default features и не нужен этим
командам. На Windows x86_64 или Linux x86_64 локальный SDK 5.9.0 задаётся
через `NEXTENGINE_PHYSX_SDK_DIR`; сборка ничего не скачивает:

```bash
cargo run -p xtask --features physx -- physics-collision --backend compare
cargo run -p xtask --features physx -- persistence-replay --backend physx
cargo run -p xtask --features physx -- physics-backend-parity
```

Для portable проверки safe wrapper без SDK используется
`--features physx-mock`. PhysX остаётся `Proposed`, не поддерживается как
product backend на macOS и не переключается автоматически после activation.

Опциональный ML smoke требует Python 3.12 и `uv`:

```bash
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

Он проверяет только локальную цепочку обучения, экспорта и inference. Это не
проверка качества игровой policy и не замена продуктовым checks. Подробнее:
[локальная проверка ML toolchain](docs/development/training-capability.md).

## Как устроен продукт

Любой источник действия — игрок, NPC, скрипт, плагин, инструмент или AI —
предлагает команду, но не получает прямой доступ к изменяемому миру:

```text
input / AI / package
  → command proposal
  → schema + capability + precondition validation
  → atomic WorldCommand transaction
  → committed DomainEvent
  → immutable gameplay and presentation projections
```

Такой путь нужен не ради абстракции как таковой. Он позволяет:

- одинаково выполнять игру в `game` и `headless`;
- не связывать корректность симуляции с FPS, wall clock, сетью или порядком
  завершения фоновых задач;
- сохранять только engine-owned authoritative state;
- безопасно отклонять повреждённые saves, packages и model output до частичного
  изменения мира;
- заменять renderer, physics backend, AI provider и другие adapters без смены
  продуктовых контрактов.

Публичные контракты живут в `crates/contracts`. ECS storage, OS/window objects,
vendor types, importer structures, database connections и model sessions
остаются деталями реализации.

## Структура репозитория

| Путь | Роль сегодня |
|---|---|
| `crates/contracts` | Публичные ID, commands/events, input, manifests, snapshots, persistence и physics/RPG contracts |
| `crates/runtime` | Admission, command ledger, fixed-stage execution и атомарные tick transactions |
| `crates/rpg` | Generic RPG state и валидируемые доменные переходы |
| `crates/mechanics` | Public-package host: immutable affordances/effect requests → typed RPG commands |
| `crates/script-luau` | Private deterministic Luau sandbox и package-state adapter |
| `crates/plugin-host` | Engine-owned WIT v3/v2 и private bounded Wasmtime Component adapter |
| `crates/agent` | Canonical NPC planner и procedural avatar fallback projection |
| `crates/world` | Deterministic two-chunk staging/admission и world-service save segment |
| `crates/physics-api` | Детерминированный reference physics world и grounded capsule |
| `crates/physics-physx` | Safe optional PhysX adapter за engine-owned physics API |
| `crates/physics-physx-ffi` | Единственная ADR-033 allowlisted native FFI-граница |
| `crates/assets` | Copy-on-write save/content generations, atomic publication и bounded load |
| `crates/project` | Exact resolver, neutral cooker и production project activation |
| `crates/platform` | Backend-free platform capabilities, lifecycle normalization и headless target |
| `crates/presentation` | Immutable revision-bound extraction в `PresentationSnapshotV2` |
| `crates/render` | Backend-neutral B0 render boundary и deterministic reference frame plan |
| `crates/desktop-sdl-ash` | Приватный `Proposed` SDL3/ash Vulkan adapter за ADR-003 boundary |
| `crates/verification` | Neutral fixtures, state roots, headless replay и product scenarios |
| `apps/headless` | Текущий переносимый composition root без окна и renderer |
| `apps/game` | Portable game composition root; optional experimental desktop adapter |
| `tools/xtask` | Локальные product checks и проверка архитектурных границ |
| `lab` | Изолированные ML-эксперименты; не часть игрового runtime |

Целевая архитектура также предусматривает обязательные roots `game`,
`headless` и `tools`, а `ai-host` и displayless capture оставляет
опциональными.

## Платформы

- **Windows x86_64 и Linux x86_64** — целевые shipping-платформы v1.
- **Apple Silicon macOS** — поддерживаемый developer host для portable Rust,
  tooling и ограниченных ML smoke tests, но не обещание поставки игры.

Корректность обязательного gameplay не должна зависеть от конкретной
shipping-платформы, GPU или доступности внешнего сервиса.

## Архитектурные документы

Архитектура — источник продуктовых контрактов, но статус `Accepted` у документа
не означает, что соответствующая возможность уже реализована.

Начать чтение стоит с:

1. [продуктового контракта](docs/architecture/00-product-contract.md);
2. [системной архитектуры](docs/architecture/01-system-architecture.md);
3. [индекса архитектуры и статусов решений](docs/architecture/README.md);
4. [глоссария](docs/architecture/glossary.md);
5. [playable slice и product checks](docs/architecture/12-vertical-slice-conformance.md).

Семантическое изменение принятого решения оформляется коротким ADR и обновляет
затронутые спецификации. Процесс разработки остаётся product-first:
см. [ADR-030](docs/architecture/adr/030-product-first-development-and-lightweight-validation.md).

## Контент, importer и лицензирование

Код нового движка распространяется по Apache-2.0. Происхождение перенесённых
архитектурных документов и их MIT notice описаны в
[истории переноса](MIGRATION_PROVENANCE.md) и
[уведомлениях о сторонних материалах](THIRD_PARTY_NOTICES.md).

Gothic importer может разрабатываться локально в игнорируемом
`incubator/gothic-importer/`, но остаётся отдельным репозиторием и процессом.
Он не является Cargo dependency и взаимодействует с движком только через
versioned neutral artifacts.

Игровые установки, импортированные или защищённые assets, datasets,
checkpoints, сгенерированные модели, captures, caches и secrets не должны
попадать в этот репозиторий.
