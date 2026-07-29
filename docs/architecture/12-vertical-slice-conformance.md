# SPEC-12: Product checks и playable slice

| Поле | Значение |
|---|---|
| ID | SPEC-12 |
| Статус | Accepted |
| Версия | 2.3 |
| Последняя проверка | 2026-07-29 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md) |
| Заменяет | SPEC-12 2.2 |

## Назначение

Product checks дают короткую воспроизводимую обратную связь о том, что
изменение сохраняет работоспособность продукта. Вместо проверки всей
архитектуры выполняются небольшие checks, соответствующие фактически
затронутым областям.

Каждый ProductCheck имеет понятное имя, bounded scenario или command, ожидаемое
поведение и результат `Pass`, `Fail` либо `NotRun(reason)`. Результаты
машиночитаемы и полезны для локальной разработки, но не образуют отдельный
workflow.

## Канонические ProductCheck

| Check | Когда запускать | Что он доказывает |
|---|---|---|
| `fast` | для каждого изменения | workspace собирается; formatting/static analysis/focused tests и boundary scan не находят локальной ошибки |
| `play` | при изменении runtime, gameplay, AI, input, physics, presentation или world behavior | малый neutral RPG scenario запускается offline и остаётся играбельным через production paths |
| `persistence-replay` | при изменении authoritative state, commands/events, schema, save/load, RNG, scheduling или world lifecycle | save/load и replay воспроизводят ожидаемые state/event roots; corrupt input отклоняется без partial mutation |
| `content-package` | при изменении assets, cooker, catalog, bundles, mechanics packages, Luau/Wasm, models, distribution или importer boundary | neutral content валидируется, готовится и загружается; malformed content и forbidden capabilities отклоняются безопасно |
| `platform` | только если затронуты platform host, renderer backend, target packaging, target-specific dependency или launch/session path | изменённый target запускает релевантный smoke scenario без platform-specific утечки в public contracts |
| `performance` | только если затронуты hot path, scheduling, physics/motor, renderer, streaming, memory/resource policy или declared budget | релевантный benchmark не имеет существенной регрессии относительно сохранённого числового результата |

Cross-cutting change запускает объединение соответствующих checks. Если область
неочевидна, выбирается дополнительный релевантный check; не требуется строить
глобальный граф всех требований проекта.

`cargo run -p xtask -- v1-closure` агрегирует реализованные v1 checks, exact
project/content/mechanics/extension roots и target package descriptors. На
developer host недоступный Windows/Linux runtime check или desktop smoke остаётся
`NotRun(reason)` и делает `shipping_ready = false`; portable local success не
подменяет target product evidence.

## Native Windows/Linux gate

R1 native evidence собирается существующими ProductCheck, а не новым видом
глобального допуска. На native Windows x86_64 и Linux x86_64 hosts с
Vulkan-capable driver один и тот же clean commit выполняет:

```text
cargo run --locked -p xtask --features desktop-sdl-ash -- native-gate-run --output artifacts/native-gate/<commit>
```

Команда MUST проверить exact commit, pinned Rust toolchain, `Cargo.lock`, native
target и чистоту worktree до и после run. Она последовательно запускает
`host-check`, `play`, `persistence-replay`, `content-package`, `platform`,
`performance`, `v1-closure` и `v1-package`, изолируя state root каждого check.
Target report, per-check reports и package публикуются атомарно под
`artifacts/native-gate/<commit>/targets/<target-triple>/`; generated artifacts
не коммитятся. Первая ошибка останавливает matrix и атомарно публикует `FAIL`
report с сохранённым пройденным prefix и
`NOT_RUN(PRIOR_CHECK_FAILED)` для оставшихся checks; incomplete package,
smoke-state и другие transient files публиковать запрещено. Существующий output
не перезаписывается.

После ручного переноса обоих target каталогов их сравнивает:

```text
cargo run --locked -p xtask -- native-gate-compare --windows <windows-target-report.json> --linux <linux-target-report.json> --output <cross-target-report.json>
```

Comparator MUST принимать ровно один Windows и один Linux `PASS` report одного
commit/toolchain/lock, проверять target-local package и exact platform-neutral
roots и fail closed при отсутствующем target, несовпадении commit или root.
Windows и Linux package descriptors сравниваются каждый только со своим
одноимённым slot во втором report; descriptors разных OS между собой, binary
hashes, package-manifest hashes и timings не являются cross-target oracle.

Каждый отдельный `v1-closure` сохраняет `PASS` только для native target
текущего host и честный `NotRun(reason)` для другого target; поэтому его
`shipping_ready` остаётся false. Только успешный cross-target compare MAY
выставить `native_gate_ready = true`. Он не переписывает исходные check results
и сам по себе не объявляет весь roadmap stage или v1 завершённым.

## `fast`

Базовая локальная команда проекта:

```text
cargo run -p xtask -- host-check
```

Она проверяет formatting, clippy с warnings denied, workspace tests и
repository boundaries. Пока workspace неполон, реализованные части команды
MAY явно сообщать `NotRun(reason)` вместо ложного `Pass`.

Subsystem unit/property tests SHOULD быть достаточно малы для частого запуска.
Негативные tests для public decoders, command validation и capability denial
являются частью `fast`, когда меняется соответствующий boundary.

## `play`

Canonical neutral playable scenario использует independently licensed fixtures
и production composition/input/command paths. Минимальный сценарий:

1. запускает `game` или тот же runtime в headless режиме без network;
2. загружает небольшую scene;
3. принимает movement и interaction через normalized player action path;
4. подбирает или использует item;
5. выполняет bounded NPC dialogue/quest transition;
6. продолжает обязательный outcome при отсутствии optional `ai-host`;
7. завершает сценарий стабильным gameplay result.

Check сравнивает domain outcomes и stable diagnostics, а не screenshots или
неупорядоченный текстовый log. Renderer/audio/UI changes MAY дополнительно
просматриваться через optional capture из SPEC-15.

## `persistence-replay`

Один небольшой deterministic scenario сохраняется в заранее объявленных
точках, закрывает process state, загружается и воспроизводится повторно.

Check MUST проверить:

- exact supported schema/content/project hashes;
- сохранение `PersistentId`, command/event order и named RNG state;
- совпадение authoritative result для original run, loaded continuation и
  replay на одном target/profile;
- fail-closed behavior для truncated, corrupt, hash-mismatched и unsupported
  save/replay;
- отсутствие partial migration или partial world activation.

Raw backend snapshot, wall-clock timing и renderer output не являются
authoritative oracle.

## `content-package`

Check использует малый CC0/engine-owned neutral fixture и public tooling:

Текущий M3/M10 implementation gate покрывает пункты 1–6 для data-only, Luau и
Wasm, storage/schema failure matrix и atomic two-chunk admission тем же
production loader. Wasm component выполняется через pinned private Wasmtime
adapter, engine-owned WIT N/N−1, empty ambient linker и общий mechanics/RPG
proposal path; shipping status всё ещё определяется отдельными platform gates.

1. validate authored manifest/schema/references/bounds;
2. cook immutable content;
3. resolve exact package/content dependencies;
4. load через production registry;
5. выполнить одну first-party mechanic/package action;
6. проверить malformed graph, missing blob/hash, path escape и
   script/plugin capability or budget violation;
7. при изменении distributable проверить отсутствие protected data и наличие
   basic license metadata/notices.

Обычный engine check не требует Gothic installation. Importer smoke запускается
отдельно только при изменении importer mapping или neutral import contract,
использует user-provided local installation во временном root и не копирует
source/imported bytes в engine repository.

## Conditional platform check

Platform check выбирает только реально затронутые shipping targets:

- shared portable/runtime change обычно покрывается `fast`, `play` и
  `persistence-replay`;
- Windows adapter/package change проверяется на Windows;
- Linux adapter/package change проверяется на Linux;
- shared renderer/platform ABI change проверяется на обоих shipping targets;
- macOS developer-host run подтверждает только portable development scope.

Если нужный host недоступен, результат записывается как `NotRun(reason)`, а
утверждение о затронутом target не делается. Это не обесценивает результаты
независимых checks и не блокирует работу в несвязанных областях.

## Conditional performance check

Performance check объявляет scenario, build profile, machine profile, warm-up,
sample count, units и числовой comparison threshold. Один subsystem benchmark
не должен выдавать вывод о другом subsystem или обо всём продукте.

Correctness и determinism имеют приоритет: optimization, меняющая
authoritative outcome, считается failure независимо от скорости. Wall-time
variance MAY привести к повторному измерению по той же declared методике, но
не к retry-to-green функциональных или deterministic failures.

## Product result

Перед handoff перечисляются:

- какие product areas изменены;
- какие ProductCheck запущены и их `Pass`/`Fail`;
- какие conditional checks получили `NotRun(reason)`;
- какой известный продуктовый риск остаётся.

Для обычной разработки не требуется единый root result, фиксированное число
обязательных checks или специальный product status. Release packaging MAY
собрать результаты релевантных checks для удобства, но не меняет их технический
смысл.

## SPEC-31 check composition

When the Accepted autonomous quest/divine contracts are implemented,
`QUEST-*`, `NARRATIVE-*`, `DIVINE-*` and `PANTHEON-*` scenarios are focused
instances of existing ProductCheck kinds, not new global gates:

- `play` covers intent-to-quest, disclosure/activation, autonomous outcomes,
  opposite per-god judgments, qualitative standing UI, offer/covenant choices
  and deterministic template fallbacks offline;
- `persistence-replay` covers recorded external candidates, per-god
  completion/fallback assignment, fixed decision-boundary closure,
  cross-context graph/standing/effect batches and save/restart without
  model/network calls;
- `content-package` covers patron epistemic policies, pantheon relation graphs,
  intervention catalogs, covenant compatibility, anchors/extension slots and
  template definitions;
- `performance` is conditional only when request construction, validation or
  conflict resolution materially changes a declared runtime budget.

No fresh LLM response, screenshot or prose comparison can substitute for
authoritative command/event/graph/standing/state-root assertions.
