# Next Engine

**Открытый AI-first движок и набор инструментов для системных одиночных RPG.**

Next Engine создаётся для игр, в которых движение, бой, диалоги, квесты,
персонажи и изменения мира работают как части одной системы. Игровые решения
должны оставлять устойчивые последствия, мир — корректно сохраняться и
воспроизводиться, а AI — обогащать игру, не становясь условием её
работоспособности.

> **Статус:** ранний bootstrap. В репозитории уже работает детерминированный
> headless-срез с вводом, командами, RPG-состоянием, эталонной физикой,
> сохранением и replay. Интерактивная игра, renderer, полный конвейер подготовки
> контента и законченный RPG-цикл остаются целевым результатом v1, а не готовым
> продуктом.
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
- contact-gated core interaction: действие игрока выбирает только объект в
  активном физическом контакте и меняет его RPG-состояние через Outcome
  `WorldCommand`;
- атомарные поколения сохранений, восстановление, replay и проверка совпадения
  authoritative state roots;
- переносимый `headless` composition root и локальные product checks;
- изолированный экспериментальный путь
  `train → ONNX → inference` для проверки локального ML toolchain.

Это ещё не законченная игра: текущий `play` проверяет ограниченный neutral
сценарий движения, столкновения и активации одного `core-switch`. Reference
physics profile пока ограничен upright capsule и статическими Box colliders;
предметы, диалоги и квестовые последствия остаются следующими срезами.

## Быстрый старт

Нужен Rust `1.93.0`; версия закреплена в
[rust-toolchain.toml](rust-toolchain.toml).

Запустить текущий headless-срез:

```bash
cargo run -p next_headless
```

Запустить основные локальные проверки:

```bash
# Ввод игрока → grounded capsule → static collision → contact-gated activation
cargo run -p xtask -- play

# Гравитация, столкновения с полом/стеной и жизненный цикл контактов
cargo run -p xtask -- physics-collision

# Сохранение → восстановление → replay и fallback повреждённого поколения
cargo run -p xtask -- persistence-replay

# Форматирование, clippy, тесты workspace и архитектурные границы
cargo run -p xtask -- host-check
```

Команды выводят компактный машиночитаемый результат. `host-check` — канонический
широкий локальный `fast` check перед передачей изменения.

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
| `crates/physics-api` | Детерминированный reference physics world и grounded capsule |
| `crates/physics-physx` | Safe optional PhysX adapter за engine-owned physics API |
| `crates/physics-physx-ffi` | Единственная ADR-033 allowlisted native FFI-граница |
| `crates/assets` | Copy-on-write поколения сохранений и восстановление |
| `crates/verification` | Neutral fixtures, state roots, headless replay и product scenarios |
| `apps/headless` | Текущий переносимый composition root без окна и renderer |
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
