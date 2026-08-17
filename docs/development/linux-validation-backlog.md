# Linux validation backlog

| Поле | Значение |
|---|---|
| Статус | Living operational checklist, не нормативная архитектура |
| Последнее обновление | 2026-08-18 |
| Основной developer host | Windows x86_64 MSVC/Vulkan |
| Linux target | Native Linux x86_64 GNU, Ubuntu 22.04 / glibc 2.35 baseline |

## Назначение

Этот документ хранит действия, которые требуют native Linux host. Они
накапливаются во время Windows-first разработки и выполняются отдельными
асинхронными сессиями, когда Linux-машина доступна.

Недоступность Linux не блокирует обычную Windows-разработку и не меняет
результаты независимых Windows/portable checks. Одновременно она не считается
успехом Linux target: невыполненная проверка остаётся `NotRun(reason)`, а
зависящие от неё R1/v1 claims остаются открытыми.

Windows x86_64 и Linux x86_64 по-прежнему являются v1 shipping targets. Этот
порядок выполнения не меняет
[SPEC-12](../architecture/12-vertical-slice-conformance.md),
[ADR-030](../architecture/adr/030-product-first-development-and-lightweight-validation.md)
или [roadmap](../roadmap.md).

## Execution policy

1. Реализация и частые проверки выполняются на Windows.
2. Если work package меняет Linux-specific код либо общий
   renderer/platform/physics/package/performance boundary, в таблицу ниже
   добавляется один Linux item.
3. Linux items не обязаны выполняться параллельно с Windows changes. Несколько
   совместимых items можно закрыть одним более поздним checkpoint run.
4. Каждый Linux checkpoint привязан к одному полному clean commit SHA. Поздняя
   Windows-разработка не делает этот результат ложным, но и не расширяет его на
   более новые commits.
5. Для закрытия stage/shipping criteria выбирается checkpoint, содержащий все
   изменения соответствующего этапа. Нужные native reports и packages
   собираются на этом exact commit.
6. Cross-compilation и WSL-only run полезны для разработки, но не заменяют
   native Linux target evidence.
7. Generated reports, packages, state, logs и caches не коммитятся.

Обычный portable/gameplay change не получает отдельный Linux item, если он не
затронул условие из пункта 2. Полный native gate на checkpoint всё равно
проверит накопленное состояние целиком.

## Текущее evidence

| Checkpoint | Linux environment | Результат | Оставшееся действие |
|---|---|---|---|
| `d15c11a23f9b62a91fa2cc7e400f6ab8409e1766` | Native Linux x86_64, Ubuntu 26.04 LTS / kernel 7.0.0-29, Wayland, NVIDIA RTX 3080, driver 610.43.02, NVIDIA Vulkan ICD; package audited against Ubuntu 22.04 / glibc 2.35 baseline | Full `native-gate-run`: `PASS`; target-report SHA-256 `f22745ba5f9aacf8a3e940c005ddc969e2c1bee9e5a8e63e4dc0c4228f916930`; all eight checks and packaged `game`/`headless` smoke pass; actual maximum GLIBC import is 2.34 | Preserve/transfer the full target bundle; build a matching Windows bundle on this exact commit and run `native-gate-compare`. |
| `f58b2a5557a59aa0e5735844a9cbe927043b314e` | Native Linux x86_64, Ubuntu 22.04 userspace, Vulkan 1.3 Mesa llvmpipe | `native-gate-run` target report и package: `PASS` | Сохранить и перенести полный `targets/x86_64-unknown-linux-gnu/`; matching Windows report и compare выполняются отдельно на том же commit. |

Current `d15c11a…` evidence closes the accumulated hardware-GPU, render-content
and player/session Linux checks; exact decisions and attempts are retained in
the [Linux catch-up task state](task-state/linux-validation-catch-up.md).
Historical `f58b2a5…` still proves only the software Vulkan path. Neither
standalone Linux run sets `native_gate_ready` without a matching Windows report
and successful compare on the same commit.

## Накопительная очередь

| ID | Статус | Когда выполнять | Действие на Linux | Закрывает |
|---|---|---|---|---|
| `LNX-001` | `DONE / HISTORICAL` | Выполнено на `f58b2a5` | Полный `native-gate-run` на native Ubuntu 22.04 / Mesa llvmpipe. | Historical software-Vulkan checkpoint; current B-01 candidate is `d15c11a…`. |
| `LNX-002` | `READY_FOR_TRANSFER / WINDOWS_PAIR_PENDING` | До paired compare для current `d15c11a…` checkpoint | Передать полный target bundle, включая `checks/`, `package/` и `target-report.json`; один report без package недостаточен. | Делает current Linux `PASS` доступным comparator; Windows report всё ещё отсутствует. |
| `LNX-003` | `DONE` | Выполнено на `d15c11a…` | Full gate прошёл real NVIDIA Vulkan `platform`, current package ABI audit и packaged `game`/`headless` smoke. | Representative hardware-GPU gap в B-02 закрыт; B-01 остаётся отдельным paired gate. |
| `LNX-004` | `DONE` | Выполнено на `d15c11a…` | Current render catalog cook/activation, derived meshlet/SPIR-V B0 route, fallback, performance smoke и package path прошли в полном native gate. | Linux side первого R2 render-content increment. |
| `LNX-005` | `DONE` | Выполнено на `d15c11a…` | Current live/headless input, targeting, camera, session/recovery, platform failure matrix, persistence/replay and packaged launch paths прошли workspace tests и mapped full-gate checks. | Linux side Player action/camera/session increment; R1/B-01 всё ещё требует same-commit Windows pair and compare. |
| `LNX-006` | `CANCELLED / SUPERSEDED_BY_ADR_049` | Не выполнять | ADR-049 удалил allocator-counter subsystem и его target-extension route; retired ADR-043/Rust 1.93 audit больше не является current evidence или future gate. | Ничего не блокирует; новый allocator route потребует отдельного consumer-backed decision, а не возобновления этого item. |

Новый deferred Linux item добавляется сюда в том же changeset, который
завершает соответствующий Windows work package. Статус `DONE` ставится только
после фактического native run с записанными commit, environment и результатом.

## Focused Linux session

Перед любым run:

- использовать native `x86_64-unknown-linux-gnu`, не WSL и не
  cross-compilation;
- проверить clean worktree, полный 40-символьный commit SHA, Rust `1.97.1` и
  неизменённый `Cargo.lock`;
- для package/desktop smoke иметь Ubuntu 22.04 / glibc 2.35 compatible
  userspace, системный `libvulkan.so.1`, Vulkan 1.3 ICD/driver и активную X11
  либо Wayland session;
- выбрать новый output path: существующий gate output не перезаписывается;
- записать host OS/kernel, desktop session, GPU, driver/ICD и Vulkan API.

Для быстрой проверки конкретного накопленного риска запускается только
релевантный набор:

```bash
cargo run --locked -p xtask -- host-check
cargo run --locked -p xtask -- content-package
cargo run --locked -p xtask --features desktop-sdl-ash -- platform
cargo run --locked -p xtask -- performance
cargo run --locked -p xtask -- v1-package --output dist/nextengine-v1-linux
```

Команды выбираются по затронутой области; это не обязательная матрица для
каждой Linux-сессии. Renderer/platform/package change требует `platform`;
renderer hot path также требует `performance`; content/cooker change требует
`content-package`.

## Closure checkpoint

Когда нужен target-level или stage-level вывод, focused checks не подменяют
полный gate. Он выполняется один раз без автоматического retry-to-green:

```bash
cargo run --locked -p xtask --features desktop-sdl-ash -- native-gate-run --output artifacts/native-gate/<commit>
```

После run переносится весь каталог:

```text
artifacts/native-gate/<commit>/targets/x86_64-unknown-linux-gnu/
```

Matching Windows bundle должен быть построен на том же exact commit. Затем на
checkout этого commit запускается:

```bash
cargo run --locked -p xtask -- native-gate-compare \
  --windows artifacts/native-gate/<commit>/targets/x86_64-pc-windows-msvc/target-report.json \
  --linux artifacts/native-gate/<commit>/targets/x86_64-unknown-linux-gnu/target-report.json \
  --output artifacts/native-gate/<commit>/cross-target-report.json
```

Только `PASS` compare с `native_gate_ready = true` закрывает paired target
evidence. Он не объявляет автоматически весь R1 или v1 завершённым.

При `FAIL` сохраняются атомарный failure bundle, первый stable diagnostic и
`NOT_RUN(PRIOR_CHECK_FAILED)` для хвоста. Неполный package не принимается.
После исправления используется новый committed SHA либо явно новый output; уже
опубликованный результат не перезаписывается и не превращается повтором в
`PASS`.

## Как обновлять backlog

После Linux-сессии:

1. записать exact commit, native environment и фактический `PASS`/`FAIL`;
2. отметить, какие IDs покрыты checkpoint;
3. сохранить полный target bundle вне Git;
4. оставить незапущенные пункты без оптимистического статуса;
5. обновить roadmap только если изменился blocker, stage criterion или
   implementation order.

Минимальная запись попытки содержит: ID, scope, full commit SHA, Cargo.lock
SHA-256, Rust release, host/session/GPU/driver profile, run command, target
report path/hash, status или first failure code, Windows companion status,
comparison report и оставшийся незакрытый claim.
