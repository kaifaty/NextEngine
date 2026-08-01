# Linux validation backlog

| Поле | Значение |
|---|---|
| Статус | Living operational checklist, не нормативная архитектура |
| Последнее обновление | 2026-08-01 |
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
| `f58b2a5557a59aa0e5735844a9cbe927043b314e` | Native Linux x86_64, Ubuntu 22.04 userspace, Vulkan 1.3 Mesa llvmpipe | `native-gate-run` target report и package: `PASS` | Сохранить и перенести полный `targets/x86_64-unknown-linux-gnu/`; matching Windows report и compare выполняются отдельно на том же commit. |

Этот run доказывает software Vulkan path, но не representative hardware-GPU
confidence. Он не выставляет `native_gate_ready` без matching Windows report и
успешного compare.

## Накопительная очередь

| ID | Статус | Когда выполнять | Действие на Linux | Закрывает |
|---|---|---|---|---|
| `LNX-001` | `DONE` | Выполнено на `f58b2a5` | Полный `native-gate-run` на native Ubuntu 22.04 / Mesa llvmpipe. | Linux half текущего B-01 checkpoint. |
| `LNX-002` | `PENDING_TRANSFER_CONFIRMATION` | До paired compare для `f58b2a5` | Передать полный target bundle, включая `checks/`, `package/` и `target-report.json`; один report без package недостаточен. | Делает существующий Linux PASS доступным comparator. |
| `LNX-003` | `PENDING` | Во время ближайшей доступной Linux hardware-GPU сессии | На выбранном актуальном checkpoint выполнить `platform` и package smoke; при накопленных cross-target changes предпочесть полный native gate. | Representative hardware-GPU gap в B-02. |
| `LNX-004` | `PENDING` | Во время ближайшей доступной Linux validation с актуальным render-content checkpoint | Проверить exact render catalog cook/activation, derived meshlet payloads, checked-in SPIR-V, Vulkan B0 CPU visible-list/indexed-indirect draw, declared fallback material и relevant performance/package paths. Можно асинхронно объединить с `LNX-003` одним полным gate на более новом checkpoint; ожидание этого run не блокирует текущие Windows increments. | Linux side первого R2 render-content increment; Windows counterpart проходит локально. |
| `LNX-005` | `PENDING` | Во время ближайшей native Linux validation с актуальным player-action/camera checkpoint | Проверить общий live/headless ActionMap/InputContext path, SDL keyboard/relative-mouse normalization, persisted-ingress `ClosestPoint` targeting, V5 receipt/query/tamper replay, complete in-memory 30 Hz snapshots и parity при 30/60/144 Hz render cadence, bounded catch-up, durable same-session checkpoints на tick `0`/каждые `30` ticks и forced atomic suspend/close publication. Platform matrix должна покрыть per-source sequence gap/duplicate/identity collision, `PLATFORM_EVENT_SEQUENCE_GAP`, illegal combined lifecycle batch и retry после failed preflight без сдвига source cursors/tick/state; stale close обязан отклоняться до forced checkpoint без live/durable mutation. Crash должен восстанавливать тот же active session с rollback не более `29` ticks и без wall-time catch-up; recovery отдельно отклоняет missing declared live payload, а publication/pruning faults проверяют failed-third-publication current+previous retention и repair stale generation на следующем успешном publish. Package smoke запускает скопированный `headless --live-ticks 0` и bounded interactive `game`, сверяет их authoritative state/ledger roots и не публикует disposable smoke state. Также проверить typed integer/fixed-point camera и приватное Vulkan float view-projection/depth преобразование. Можно объединить с `LNX-003`/`LNX-004` одним полным gate на более новом clean commit; само ожидание Linux run не блокирует unrelated Windows work. | Linux side Player action and camera increment; не закрывает R1/B-01 без требуемой same-commit пары и compare. |
| `LNX-006` | `PENDING` | После Windows implementation/overhead PASS ADR-041 на exact clean commit и до использования allocator fields в Linux `REPORT_ONLY` performance profile | На native `x86_64-unknown-linux-gnu` с pinned Rust `1.93.0` выполнить `cargo test --locked -p next_process_allocation_counter`, `cargo run --locked -p xtask -- boundary-scan` и `cargo run --locked -p xtask -- allocator-counter-check --output artifacts/allocator-counter/<commit>`. Добавить/выполнить Linux release assembly+LLVM audit, эквивалентный Windows gate: const no-drop TLS без lazy init/allocation/lock/destructor; inactive path один state load/no TLS; owner success path zero locked/RMW, second control/sequence access и EH control flow; foreign path ровно один slot RMW плюс exact postcheck; direct `System`, bounded frame и unchanged pointer/data/root semantics. Сохранить `allocator-counter-check-v1.json`, его SHA-256, полный raw 2+15 schedule, `rustc -vV`, target triple и codegen audit output; existing output не перезаписывать и failed candidate не повторять. | Native Linux admission для ADR-041 allocator metric. Закрывается только при exact/count/root/codegen PASS, independently `<=3%` inactive/enabled medians и state `<=64 MiB`; Windows PASS не подменяет этот item. |

Новый deferred Linux item добавляется сюда в том же changeset, который
завершает соответствующий Windows work package. Статус `DONE` ставится только
после фактического native run с записанными commit, environment и результатом.

## Focused Linux session

Перед любым run:

- использовать native `x86_64-unknown-linux-gnu`, не WSL и не
  cross-compilation;
- проверить clean worktree, полный 40-символьный commit SHA, Rust `1.93.0` и
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
