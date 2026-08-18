# SPEC-12: Product checks и playable slice

| Поле | Значение |
|---|---|
| ID | SPEC-12 |
| Статус | Accepted |
| Версия | 4.4 |
| Последняя проверка | 2026-08-18 |
| Нормативные зависимости | [SPEC-00](00-product-contract.md), [SPEC-01](01-system-architecture.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-07](07-rpg-scripting-and-plugins.md), [SPEC-11](11-security-licensing-and-governance.md), [SPEC-15](15-headless-testing-agent-validation-and-human-evidence.md), [SPEC-17](17-project-composition-configuration-and-application-lifecycle.md), [SPEC-20](20-world-simulation-and-population-lifecycle.md), [SPEC-21](21-deterministic-runtime-primitives-command-ledger-and-causal-identity.md), [SPEC-24](24-content-catalog-bundle-and-neutral-asset-schemas.md), [SPEC-25](25-world-partition-streaming-admission-and-persistent-spatial-objects.md), [SPEC-32](32-npc-cognition-intention-lifecycle-and-deterministic-behavior-inference.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-045](adr/045-low-overhead-hard-performance-evidence.md), [ADR-046](adr/046-consumer-driven-contracts-and-current-only-alpha-formats.md), [ADR-049](adr/049-performance-evidence-without-allocator-instrumentation.md), [ADR-051](adr/051-r3a-packaged-chunk-streaming-commit-boundary.md), [ADR-052](adr/052-derived-world-calendar-and-authored-routine-vertical.md), [ADR-060](adr/060-relaxed-thoth-performance-preflight.md), [ADR-061](adr/061-forty-percent-thoth-load-preflight.md), [ADR-062](adr/062-r5-physx-humanoid-performance-authority.md), [ADR-063](adr/063-run-level-performance-evidence-and-fixed-gate-batches.md), [ADR-072](adr/072-deterministic-population-tier-and-graph-navigation-vertical.md), [ADR-073](adr/073-deterministic-cognition-owner-vertical.md), [ADR-074](adr/074-systemic-strategic-agent-owner-vertical.md), [ADR-082](adr/082-linux-first-development-and-deferred-windows-host.md), [ADR-083](adr/083-public-creator-project-cli-vertical.md), [ADR-084](adr/084-public-creator-run-and-project-package-vertical.md), [ADR-085](adr/085-public-creator-project-inspect-and-diff-vertical.md) |
| Заменяет | SPEC-12 4.3; adds the R6c creator inspect/diff acceptance matrix to existing ProductChecks without creating a new global category |

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
| `fast` | для каждого code/build/schema/generated-data изменения | затронутая область собирается; formatting/static analysis/focused tests и boundary scan не находят локальной ошибки |
| `play` | при изменении runtime, gameplay, AI, input, physics, presentation или world behavior | малый neutral RPG scenario запускается offline и остаётся играбельным через production paths |
| `persistence-replay` | при изменении authoritative state, commands/events, schema, save/load, RNG, scheduling или world lifecycle | save/load и replay воспроизводят ожидаемые state/event roots; corrupt input отклоняется без partial mutation |
| `content-package` | при изменении assets, cooker, catalog, bundles, mechanics packages, Luau/Wasm, models, distribution или importer boundary | neutral content валидируется, готовится и загружается; malformed content и forbidden capabilities отклоняются безопасно |
| `platform` | только если затронуты platform host, renderer backend, target packaging, target-specific dependency или launch/session path | изменённый target запускает релевантный smoke scenario без platform-specific утечки в public contracts |
| `performance` | только если затронуты hot path, scheduling, physics/motor, renderer, streaming, memory/resource policy или declared budget | релевантный benchmark не имеет существенной регрессии относительно сохранённого числового результата |

Cross-cutting change запускает объединение соответствующих checks. Если область
неочевидна, выбирается дополнительный релевантный check; не требуется строить
глобальный граф всех требований проекта.

Milestone-specific checks MAY expose a narrower command without becoming a new
global category. Current `animation-root-motion` executes the fixed
`ANIM-ROOT-MOTION-P1` production matrix and maps to `fast`, `play` and
`persistence-replay`; it proves only the bounded forward R5c/R5e route.
Current `animation-lod` executes the fixed `ANIM-LOD-P1` matrix and maps to
`fast` plus the affected presentation/play boundary; it proves only the exact
bounded R5 profile and does not create a global ProductCheck category or a
creator-authored LOD schema.
Current `physical-character` executes the fixed bounded procedural `PHYS-P6`
scenario and maps to `fast`, `play` and `persistence-replay`; it proves exact
trip, carried-load clearance, contact-driven melee and restart continuation
for the current production reference profile. It is not a general attachment,
articulation, learned-controller, performance or cross-target check.

The R6a public `next project validate/cook` commands map to `fast` plus
`content-package`. Their focused matrix proves exact one-object JSON,
current-only rejection, project-root/output confinement, deterministic repeated
cook and production activation of the independent creator fixture. It is not a
new global ProductCheck and does not prove creator run/package/replay breadth.

The R6b `next project run/package` commands map to `fast`, `play`,
`persistence-replay` and `content-package`; changes to their shared
launch/session path additionally run active-host `platform`. The focused matrix
repeats authoring run, builds two byte-identical creator packages, reruns the
published bytes, compares exact runtime/final-save roots and rejects changed
inventory, NOTICE/link and unsafe destination cases. It proves the bounded
one-tick headless creator startup and content distribution envelope, not an
interactive game, native OS bundle, replay inspector or complete R6 SDK.

The R6c `next project inspect/diff` commands map to `fast` plus
`content-package`. The focused matrix compares repeat authoring bytes from two
filesystem locations, authoring against its fully revalidated/rerun package and
one controlled record edit. It requires a path-free deterministic immutable
projection, empty source/package diff, stable-ID localized change categories,
success-on-valid-difference and all-or-none failure for invalid operands. It is
not a live gameplay/replay inspector, scenario runner, editor or complete R6
SDK.

`cargo run -p xtask -- v1-closure` агрегирует реализованные v1 checks, exact
project/content/mechanics/extension roots и target package descriptors. На
developer host недоступный Windows/Linux runtime check или desktop smoke остаётся
`NotRun(reason)` и делает `shipping_ready = false`; portable local success не
подменяет target product evidence.

## Active Linux check policy

Native Linux x86_64 является текущим active development host по ADR-082.
Затронутые Linux-capable checks выполняются в том же work package, а не
переносятся в асинхронный backlog. Desktop renderer/performance check явно
включает `desktop-sdl-ash` и использует подключённый X11/Wayland Vulkan path.

Windows execution отложен до explicit pre-R7 bring-up. Текущий Linux handoff
не запускает и не требует Windows, THOTH calibration или paired comparator;
эти claims остаются открытыми с `NotRun(WindowsHostDeferred)`. Исторический
Windows result не переносится на новый commit. Этот порядок не удаляет Windows
из v1 shipping matrix.

## Deferred native Windows/Linux release gate

После явного Windows bring-up R1/R7 native evidence собирается существующими
ProductCheck, а не новым видом глобального допуска. На native Windows x86_64 и
Linux x86_64 hosts с
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

## Documentation-only cheap path

Изменение квалифицируется как documentation-only, только если все changed
files являются human-readable documentation или agent guidance и diff не
затрагивает Rust/Python/C++, build/configuration, schemas, generated fixtures,
package manifests или runtime-consumed data.

Для такого changeset достаточно:

1. `git diff --check`;
2. direct validation изменённых links, paths, IDs и command examples;
3. честно отметить executable ProductChecks как `NotRun(NoExecutableChange)`.

Cargo, package tests и `host-check` для этого пути по умолчанию не запускаются.
Normative architecture edit всё равно следует ADR/SPEC workflow, но tests не
симулируют executable evidence, которого в diff нет. Если documentation идёт
в одном changeset с implementation/config/generated-data, применяется обычный
code path ниже.

## Commit boundary is not a check boundary

`git commit` сохраняет coherent checkpoint для review, bisect или следующего
шага плана. Он не является ProductCheck, merge/release admission или
readiness claim. Создание commit не требует предварительного запуска
`fast`, `host-check` или любого другого check; checks нельзя запускать только
потому, что следующий шаг — commit.

Relevant checks выполняются до final handoff/completion claim либо по explicit
user/plan request и могут покрывать один или несколько уже созданных commits.
Unverified или failed-check commit остаётся обычным checkpoint; handoff обязан
честно сообщить `Pass`, `Fail` и `NotRun(reason)`. Требование exact clean commit
у native/performance evidence относится к запуску соответствующего gate, а не
к праву создать commit.

## `fast`

`fast` — risk-scoped logical check, а не требование всегда запускать весь
workspace. Для локального code change используются format, lint/typecheck,
focused unit/integration tests затронутого package и repository boundary scan.

Широкая convenience-команда проекта:

```text
cargo run -p xtask -- host-check
```

Она проверяет formatting, clippy с warnings denied, workspace tests и
repository boundaries. `host-check` обязателен для cross-cutting changes,
public contracts, workspace/build configuration, неизвестной области влияния
или явного требования пользователя/плана. Для простой локальной code правки
его MAY заменить эквивалентный focused набор; для documentation-only path он
не применим. Пока workspace неполон, реализованные части команды MAY явно
сообщать `NotRun(reason)` вместо ложного `Pass`.

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

Check использует CC0/engine-owned neutral fixtures и public tooling. The
reference project remains the complete gameplay/package oracle; the independent
`creator-smoke` project additionally proves file-backed validate/cook/activate,
run/package and source-neutral inspect/diff without a Rust fixture constructor:

Текущий M3/M10/R3 implementation gate покрывает пункты 1–6 для data-only, Luau и
Wasm, storage/schema failure matrix и atomic four-region/64-chunk admission тем же
production loader. Wasm component выполняется через pinned private Wasmtime
adapter, exact current engine-owned WIT, empty ambient linker и общий mechanics/RPG
proposal path; shipping status всё ещё определяется отдельными platform gates.

1. validate authored manifest/schema/references/bounds;
2. cook immutable content;
3. validate exact package/content dependencies and direct project lock;
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

Platform check выбирает затронутую границу с учётом active-host policy:

- shared portable/runtime change обычно покрывается `fast`, `play` и
  `persistence-replay`;
- Linux adapter/package и shared renderer/platform change проверяются сейчас
  на active Linux host;
- Windows-only adapter/package result остаётся
  `NotRun(WindowsHostDeferred)` и записывается в Windows backlog;
- paired shared-renderer release claim проверяется на обоих shipping targets
  только после explicit Windows bring-up;
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

Current development timing выполняется на Linux и всегда остаётся
`REPORT_ONLY`; Linux native build/platform/replay/hash correctness обязательны
для затронутой области. Hard timing design следует ADR-036/ADR-045/ADR-049:
только `release`, compatible baseline и полный `ref-win-thoth-v1` могут дать
timing `PASS`/`FAIL`, но ADR-082 откладывает такой run до Windows bring-up и
делает B-12 R7/release blocker. Несовместимый host, driver/BIOS/power plan/toolchain/content/
methodology, недостаточный idle/free-memory/thermal preflight или
неimplemented representative workload возвращает `NOT_RUN`. V5 hard evidence
дополнительно требует canonical logical resource charges, peak working set,
process I/O, device-allocation ceiling, profiler integrity и exact authoritative
roots. Allocator-counter fields/readers отсутствуют.

При возобновлении hard evidence ADR-061 sets the THOTH preflight boundary: CPU/GPU load must be below
40%, free physical RAM must be at least 10 GiB, CPU clock must remain at least
80% of reported maximum and GPU thermal slowdown must be clear.

Two-role streaming, one-agent, статические render fixtures и live-movement checks
являются только `smoke/report`. Отдельный representative `r2-alpha-render.v3`
production workload реализован для `projects/reference-alpha` и активного
Linux desktop host: exploration,
combat и UI/dialogue выполняются отдельно в primary и fallback profiles с
600 warm-up и 3 600 measured frames на каждую из шести пар. Report mode может
дать outer ProductCheck execution `PASS` с вложенным timing verdict
`REPORT_ONLY`, но не закрывает absolute budget или B-12. Он запускается с
`--features desktop-sdl-ash`; disabled feature или недоступная desktop/GPU
capability даёт typed `NOT_RUN`, не Windows-only fallback. Отдельный streaming-only
`r3-multiregion-streaming` выполняет 1 000 production packaged transitions по
canonical four-region/64-chunk route, публикует только `streaming_world` и
заполняет logical `required_staging_bytes`; он также `REPORT_ONLY`. R2/R3 hard
gates требуют clean compatible ten-run THOTH evidence. Реализованный
`r4-100npc.v1` выполняет 1 000 warm-up и 10 000 measured совместных production
ticks над exact 16/32/52 population, проверяет один graph query на каждую due
record, exact four-kind tier-cognition dispatch, zero defer/drop/starvation/
fabricated outcomes и canonical roots. На несовместимом host его
hard verdict остаётся `NOT_RUN`; локальные timings являются только report-only
диагностикой и не закрывают B-12.
`r5-physics-16.v1` реализован по ADR-062: один run выполняет одинаковые
sixteen-slot PhysX 23-DoF trajectories при 1/4/8 workers, требует exact
canonical root parity и отдельно измеряет live cadence, checkpoint/restore and
resources. Clean calibration без compatible ten-run baseline остаётся
`REPORT_ONLY`.

Baseline строится из десяти independent clean runs одного commit, каждый с
полным start/postflight environment pair. Hard gate является одним fixed batch
из трёх independent runs и не разрешает отбрасывать либо выборочно повторять
его members. Raw samples сохраняются с explicit run boundaries. Absolute
budgets применяются к worst per-run p95/p99; relative point estimate сравнивает
median candidate-run p95 с median baseline-run p95, а deterministic 95%
bootstrap resamples whole runs. Compatible relative regression `<2%` считается
noise, `2–5%` — `WARNING`, `>=5%` при нижней границе interval `>=5%` — `FAIL`.
Fallback renderer result всегда сообщается отдельно и не переписывает primary
failure.

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

Future Proposed features receive a check mapping only when a production
consumer and promoting ADR exist. SPEC-20/32 and ADR-073/074 map focused
cognition/systemic/activity/bulk checks plus `play`, `persistence-replay`,
`content-package`, `host-check` and conditional report-only performance for
R4c/R4d. SPEC-23/31 add no current global gates.
