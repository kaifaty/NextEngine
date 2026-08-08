# SPEC-09: Tooling, SDK и observability

| Поле | Значение |
|---|---|
| ID | SPEC-09 |
| Статус | Accepted |
| Версия | 2.8 |
| Последняя проверка | 2026-08-06 |
| Нормативные зависимости | [SPEC-02](02-runtime-ecs-and-data.md), [SPEC-03](03-assets-world-streaming-and-persistence.md), [SPEC-13](13-gameplay-mechanics-mod-packages-and-agent-authoring.md), [SPEC-14](14-physical-archetypes-motor-skills-and-policy-lifecycle.md), [SPEC-18](18-player-interaction-ui-camera-localization-and-accessibility.md), [SPEC-29](29-platform-host-and-application-session.md), [SPEC-30](30-presentation-extraction-and-render-content.md), [ADR-011](adr/011-macos-developer-host-local-verification-and-staged-training.md), [ADR-030](adr/030-product-first-development-and-lightweight-validation.md), [ADR-036](adr/036-thoth-reference-performance-profile.md), [ADR-038](adr/038-versioned-production-worker-handoff-diagnostic.md), [ADR-039](adr/039-tooling-only-process-wide-system-global-allocator-measurement.md), [ADR-040](adr/040-fixed-tls-sharded-global-allocator-measurement.md), [ADR-041](adr/041-owner-thread-quiescent-global-allocator-measurement.md), [ADR-042](adr/042-unobserved-deallocation-system-pass-through.md), [ADR-043](adr/043-codegen-proven-non-reentrant-count-bearing-allocator-callbacks.md), [ADR-045](adr/045-low-overhead-hard-performance-evidence.md) |
| Заменяет | SPEC-09 2.7 |

## Technical authority boundary

Каждый subsystem определяет semantics своих diagnostics, metrics и immutable
inspector projections. Tooling определяет versioned CLI grammar, exit codes,
JSON/diagnostic envelopes, optional developer run records, inspectors and
profiling adapters. Tool output, telemetry, screenshots and local run data
never become gameplay authority; disabling them does not change simulation
outcome.

## Public boundary и data flow

Public tool boundary включает versioned CLI arguments/exit codes/JSON,
`ProjectManifest`/`ProjectCompositionLock` resolution reports, cooked/save/
replay/package manifests, engine-owned inspector projections, physical/policy
manifests, diagnostic and telemetry envelopes. Optional MCP or other adapters
project the same application services and cannot create a second source of
truth.

Profiler SDKs, package-manager internals, CI/artifact-store/encoder/training
backend APIs, terminal libraries, OS crash APIs and internal Rust types remain
private. Data flow

```text
subsystem structured signal
  → bounded local collector
  → diagnostic / optional developer run record / inspector
```

is read-only relative to simulation. A mutation-capable development endpoint
requires an explicit local capability and cannot bypass production
command/transaction validation.

## CLI v1

One binary MAY be named `next` until product naming changes. Exit codes are:
`0` success, `2` validation/input failure, `3` incompatible or unsupported
capability, `4` product-check failure and `5` internal tool crash. Human text
may be localized; `--format json` remains stable within a major CLI version.

| Command | Contract |
|---|---|
| `next project resolve <manifest> --catalog <snapshot> --lock <path>` | Deterministic exact-catalog resolution, canonical conflict report and atomic `ProjectCompositionLock` candidate; runtime does not resolve projects. |
| `next project validate\|diff <manifest-or-lock>` | Read-only schema/dependency/config/capability/license/budget/target compatibility and exact lock diff. |
| `next cook --project <manifest> --target <profile> --out <dir>` | Validate, deterministically cook and atomically publish; report cache statistics and manifest hash. |
| `next validate assets\|project\|bundle\|save\|plugin\|model <path>` | Read-only validation with stable diagnostic codes and source locations. |
| `next inspect import <neutral-manifest>` | Show provenance, namespace mappings and unsupported/dropped records without reading proprietary source inside the engine repository. |
| `next replay run <replay> [--headless] [--compare <oracle>]` | Report the first divergent tick/subsystem/schema and return nonzero on mismatch. |
| `next inspect physical <run\|live-endpoint>` | Read pose/joints/contacts/COM/support/LOD/motor/action-safety projections without mutable access. |
| `next inspect ai <run\|live-endpoint>` | Read perception/intents/rejections/plans/memory provenance and optional `ai-host` deadlines. |
| `next inspect mechanics <run\|live-endpoint>` | Read package lock, ability phases, effects/statuses, proposals/rejections, hook order and state revisions. |
| `next inspect player <run\|live-endpoint>` | Read normalized controls, action frames, contexts, semantic UI, camera targets and fallback revisions without native handles. |
| `next inspect rpg <run\|save>` | Read aggregate definitions/revisions, typed operations, transaction results/event order and migration diagnostics. |
| `next inspect world <run\|save>` | Read world calendar, population/schedule/tier revisions, advance-plan results and migration diagnostics. |
| `next physical new\|describe\|validate\|pack <archetype>` | Create, describe, validate and package a neutral physical archetype through public schemas. |
| `next policy inspect\|verify\|route-test\|transition-test <policy-or-project>` | Check compatibility, deterministic route selection, supervisor transitions and declared safety fallback. |
| `lab generate-env\|train\|evaluate\|compare\|export-onnx <config>` | Optional replaceable offline training service; algorithm/backend does not enter the engine ABI. |
| `next package --target windows-x86_64\|linux-x86_64` | Validate content, dependency/license notices and produce a reproducible package manifest. |

Mechanic/mod/agent CLI families defined by SPEC-13 use the same
exit-code/JSON/diagnostic contracts. Tools MUST support `--help`, `--version`,
`--format json`, explicit output paths and non-interactive use. They MUST NOT
upload telemetry or developer output unless the user supplies an explicit
endpoint and consent.

## Optional developer utilities

These utilities shorten debugging and playtesting but are not required for
product correctness:

| Utility | Behavior |
|---|---|
| `next scenario validate\|run\|minimize <input>` | Validate or run a representative scenario and minimize to the same stable failure. |
| `next capture render --input <manifest> --out <dir>` | Run the displayless `capture-worker` for screenshots, video or audio from explicit replay/presentation inputs. |
| `next profile run <command> --out <dir>` | Enable bounded CPU/GPU/platform profiling and write a local report. |
| `cargo run --release -p xtask -- performance --scenario <id> --mode report\|gate [--target ref-win-thoth-v1] [--baseline <file>] [--output <dir>]` | Run repository-owned versioned performance report; incompatible host/baseline/workload returns `NOT_RUN`. |
| `cargo run --release -p xtask -- performance-baseline --runs <ten-run-dir> --output <dir>` | Strictly validate ten clean compatible THOTH reports and atomically publish one `PerformanceBaselineV3`; never overwrite an existing baseline. |
| `next inspect run <run-root>` | Open an optional developer run record and its referenced logs/traces/media. |

Capture, screenshots and profiling are opt-in. Missing GPU, encoder or external
profiler simply means that optional utility was not run; normal game,
headless, save/replay and package work remains available.

## Local bootstrap orchestration

Repository-owned convenience commands MAY combine focused checks:

```text
cargo run -p xtask -- boundary-scan
cargo run -p xtask -- host-check
uv run --project lab python -m next_lab doctor
uv run --project lab python -m next_lab smoke --device auto
```

When present, `host-check` combines formatting, clippy, workspace tests and the
boundary scan. Mac results cover the developer-host portable core/tooling path,
not Windows/Linux game or renderer shipping. Lab smoke may select MPS or a
declared CPU fallback and proves only the exercised local train/export/
inference path. No command requires remote CI, a cloud account or a CI token.

## Diagnostic envelope

Each diagnostic record contains schema version, stable code, severity,
subsystem, message key plus rendered message, process/build identity and, when
applicable, tick, `PersistentId`, `AssetId`, source span, first divergent
tick/event/schema/assertion, typed expected/actual, causal command/event chain,
remediation key, replay/minimization reference and redaction classification.
Raw vendor errors may appear only as a debug field; the stable code remains
engine-owned.

Severity is `info`, `warning`, `error` (operation fails) or `fatal`
(process/world unsafe). Rendered text and unordered logs are never the
correctness oracle.

## Telemetry and optional run records

When telemetry is enabled, metrics/traces use a monotonic timestamp plus
simulation tick where applicable, subsystem, scenario/run ID,
build/config hashes, typed attributes and units. Sampling never changes
control flow or blocks a fixed tick. User paths, dialogue text, prompts,
voices and imported asset bytes are redacted by default.

An optional `DeveloperRunRecordV1` may use this local layout:

```text
run-root/
  run.json
  replay/
  metrics/
  traces/
  logs/
  reports/
  media/
  crash/
```

The record may include schema version, run/scenario ID, engine/build/toolchain/
platform/hardware, exact project/content/save/replay/model/plugin/backend
hashes, seeds, tick rates, command, configs and a content-addressed list of
local files. Media additionally records camera, semantic tick window,
resolution/FPS/color/audio settings and encoder identity when those values
matter to reproduction. The record is a debugging convenience, not a package,
merge or release authority.

## Inspectors and live access

Inspectors normally read saves, replays, cooked manifests or optional developer
run records. Live endpoints exist only in development builds over a local
authenticated channel and expose versioned read-only snapshots. A mutating
debug request requires a separate explicit capability and still enters the
production command boundary. Inspector schemas contain no vendor handles,
mutable ECS references or hidden first-party APIs.

## Crash isolation

Each process MAY write a bounded crash capsule containing build/config/content
hashes, last completed tick, recent command/event IDs, subsystem health,
redacted stack/minidump reference and replay checkpoint pointer. A crash
handler never serializes an arbitrary possibly-corrupt world. Tool, importer
or optional `ai-host` crashes cannot partially publish output or crash the
running game process.

## Profiling hooks

CPU spans, schedule stages, allocator counters, GPU timestamps,
physics/motor timings, streaming I/O, script/plugin budgets and optional
`ai-host` latency use stable category names when profiling is enabled.
Profiling off/on MUST produce identical accepted command and authoritative
state hashes. External profilers connect only through private platform
adapters.

Performance tooling serializes `PerformanceRunV3`,
`PerformanceResourceCountersV3`, `PerformanceMetricV1`,
`PerformanceBaselineV3` and `PerformanceVerdict` under ADR-036/ADR-045. Эти schemas
принадлежат tooling и не добавляются в `crates/contracts`.
`PerformanceMethodologyV1` сохраняет прежний field shape, но V3 run требует
значение `nextengine-performance-v3`. V2 run/resource/baseline schemas остаются
strict decode-only historical evidence и не допускаются в current baseline/gate. Raw samples
сохраняются вместе с nearest-rank p50/p95/p99; outliers не удаляются.
Relative comparison хранится fixed-point basis points и deterministic
bootstrap 95% interval.

Stable CPU span categories включают runtime stages, render extraction,
physics/motor, streaming/I/O, agent planning и navigation. Per-thread buffers
preallocated и bounded; overflow, dropped sample или
`UNOWNED_GAMEPLAY_SPAN` invalidates run. Profiler state включается runtime
setting в той же release binary, не feature-specific gameplay build.
Declared overhead bound — 3% и 64 MiB.

Canonical logical resource charges, process peak working set, process/device
residency ceilings, I/O counters и Vulkan timestamp queries помечают unavailable
source явно. Отсутствующий required V3 counter в hard scenario даёт `NOT_RUN`,
но smoke/report MAY сохранить unavailable diagnostic. Canonical charges содержат
hash-bound accounting profile, checked host/device totals и charge root; они
являются budget evidence, но не меняют authoritative scheduling outcome.
Windows process I/O хранится как workload delta между двумя
`GetProcessIoCounters` snapshots. Private Vulkan adapter при runtime-enabled
profiling preallocates a finite frame-sample buffer, writes top/bottom timestamp
queries and pairs each completed GPU duration with CPU extract/submit time.
Overflow or an uncollected query increments a dropped-sample counter. Device
residency reporting uses the conservative sum of engine-owned bound Vulkan
allocations and explicitly excludes driver-owned swapchain storage. Это
presentation/telemetry data и не входит в gameplay authority. Глобальные host
ADR-039/ADR-040/ADR-041/ADR-042/ADR-043 разрешают exact host allocator counter
только как отдельный internal `tools/process-allocation-counter` с единственным
reverse dependency `xtask`, process-wide runtime-enabled scenario window и тем
же `std::alloc::System`. `game`, `headless` и shipping graph его не линкуют.
Успешные `alloc`, `alloc_zeroed` и `realloc` calls хранят отдельные exact
bytes/counts; aggregate является checked gross sum. Unobserved `dealloc`
безусловно делегируется ровно одному matching `System::dealloc`, ничего не
вычитает и не участвует в measurement state/TLS/slot/fault/close protocol.
Scope, PID и window identity входят в versioned report. Это не live heap и не
peak RSS; approximate process-private memory их не подменяет.

Первый global per-call in-flight implementation сохранил exact roots, но
enabled median `+15.95%` нарушил `3%`. ADR-040 заменил его fixed const-TLS
per-thread slots и одним owned-slot RMW; retained enabled result остался
`+15.64%`. ADR-041 убрал owner RMW и получил `+5.88%`. ADR-042 сделал direct
pass-through для unobserved `dealloc`, однако immutable candidate-6 снова
сохранил roots/resource/inactive checks и дал enabled `+4.81%` `FAIL` при
`15 253 608` count-bearing callbacks в diagnostic run.

ADR-043 поэтому разрешает убрать per-call `in_callback` get/set/reject только
на exact pinned target/build, где source, release LLVM IR и assembly доказывают
allocation-free instrumentation, а backend — direct non-interposed System
delegation без обратного входа в Rust global allocator. Owner cookie/counters,
foreign slot admission/postcheck/close handshake, checked overflow и
TLS/slot/window/PID faults сохраняются. Недоказанный либо reentrant target не
открывает measurement window и возвращает `NOT_RUN`. Native Linux требует
отдельного `LNX-006`; Windows proof его не подменяет. ADR-043 реализован и
прошёл focused/source/boundary/codegen/parity admission, однако единственный
immutable candidate-7 дал enabled `+4.30%` `FAIL` при inactive `-0.50%`
`PASS`. ADR-045 сохраняет этот failure immutable, но узко заменяет обязательность
active exact allocator counter в hard run: normal V3 timing window не активирует
его, отсутствие allocator evidence допустимо, а приложенный partial/invalid
counter по-прежнему invalidates run. Hard V3 требует canonical logical charges,
peak working set, process I/O, device-allocation ceiling, Vulkan timestamps,
profiler integrity и exact authoritative roots. Отдельный
`allocator-counter-check` остаётся diagnostic path.
`long-session-soak` дополняет быстрый smoke report-only диагностикой
history-dependent degradation: одинаковые held-movement/periodic-camera inputs
проходят через live driver и interactive application scheduler на `3 600`
ticks. Отдельно сохраняются три окна, mandatory 30-tick durable checkpoint
samples и isolated identity-index/archive-root probes; exact ledger-root
parity обязательна, но этот fixture не становится hard timing authority.
`interactive-frame-soak` отдельно выполняет `240` FIFO-presented кадров при
requested `1920×1080` через production Vulkan path и сохраняет report-only CPU/GPU critical path,
event-polling плюс immutable frame-source update, frame-slot/acquire/image waits,
frame-plan, command-record, submit и present phases вместе с software-pacing и
frame-plan-cache counters. Этот renderer-only fixture использует статические
reference render inputs и не приписывает себе simulation-worker latency.
`production-worker-soak` отдельно выполняет не менее `240` FIFO main callbacks
через production-owned bounded queue, `next-simulation` worker, fixed-step
application advance, shared immutable snapshot publication и main-side read.
Он сохраняет bounded raw queue/send/dequeue, ordinary/checkpoint worker,
publication/read-lock и sequence lag/freshness samples. Diagnostic send и
dequeue observation линеаризованы вокруг того же bounded `sync_channel`,
поэтому high-water означает exact channel occupancy, а не оценку outstanding
work; message/step/publication counts exact и проходят zero-drop/reorder
validation. Оба diagnostics имеют
`REPORT_ONLY`, не подменяют representative R2 alpha project и не закрывают
B-12; wall time и operational sequence metadata не влияют на scheduling или
authoritative state.
`r2-alpha-render` является отдельным production workload: он загружает
`projects/reference-alpha` через public authoring/cook/activation path и
исполняет exploration, combat и UI/dialogue для primary `1920×1080` и
fallback `1280×720`. Каждая из шести независимых пар profile/window использует
600 warm-up и 3 600 measured frames, собственный absolute budget, exact Vulkan
timestamp-query count, canonical logical charges, Windows process/device
counters, profiler integrity и authoritative roots. Это больше не статический
render smoke, но report mode остаётся `REPORT_ONLY`, а hard `PASS` разрешён
только clean release ten-run THOTH evidence по ADR-036/ADR-045. Workloads R3–R5
до их реализации честно возвращают `NOT_RUN`.
Тяжёлые captures, WPA/perf/samply profiles и generated reports остаются
machine-local и не коммитятся.

## SDK stability

V1 public SDK is limited to documented CLI JSON schemas,
`ProjectManifest`/`ProjectCompositionLock`, player action and semantic
projections, typed RPG/world values, mechanics/package/change schemas, cooked/
save/replay manifests, Luau capability APIs, WIT worlds, optional `ai-host`
IPC and neutral asset/import schemas. Developer capture, profiling, MCP
implementation and live-inspector protocols remain unstable unless a future
ADR promotes a specific contract. SemVer applies to published schemas and
packages; breaking changes include a migration or compatibility note.

## Future CI

Bootstrap MUST NOT depend on `.github/workflows/`, a CI vendor or a remote.
Future CI may run the same repository-owned `fast`, `play`,
`persistence-replay`, `content-package` and conditional `platform` or
`performance` checks described by ADR-030. CI scheduling and stored logs are automation details, not
architectural authority. Optional GPU, capture, profiling or long scenarios
run only when relevant capability is available.

## Failure semantics

- Internal tool crash returns exit `5`; atomic output remains unpublished.
- Disk-full, hash or atomic-publication failure retains the previous complete
  output and returns a stable error.
- Unknown CLI/schema major returns exit `3` before mutation.
- Unavailable telemetry exporter uses a bounded local queue/drop counter and
  never blocks gameplay.
- Corrupt inspector input is a read-only error; repair requires an explicit
  separate command.
- Missing/incompatible MCP or other adapter falls back to the complete
  CLI/JSON path without project/runtime mutation.
- Stale or unsafe `AgentChangeSet` rejects before filesystem mutation with the
  violated precondition/path/capability.
- Missing training backend uses the engine-owned headless-lab fallback; model
  validation and prototype packaging remain available.
- A deterministic retry mismatch is `NONDETERMINISTIC_RESULT`, never
  retry-to-green.

## Product checks

| ID | Scenario | Expected behavior | Fallback |
|---|---|---|---|
| `TOOL-01` | Run golden CLI success/error fixtures on Win/Linux. | Stable exit codes and JSON schemas match; failed commands publish no partial output. | Fix the command contract before distributing the affected tool. |
| `TOOL-02` | Inspect every supported current and previous-major artifact/schema fixture. | Supported data remains readable and incompatible input is rejected with a clear stable diagnostic. | Ship or retain a matching standalone inspector for the older format. |
| `OBS-01` | Replay the same fixtures with profiling/telemetry disabled and at maximum configured sampling. | Accepted commands, events and final authoritative state hashes remain exact; enabled hooks stay within the declared local overhead budget. | Reduce sampling or disable the optional hooks. |
| `OBS-02` | Inject crashes at process and atomic-write boundaries. | Published files are never partial; when safe, the bounded crash capsule identifies the last completed tick and recent causal IDs. | Harden the crash/atomic boundary and retain the previous complete generation. |
| `OBS-03` | Inject representative failures across diagnostic and trace producers. | Records retain stable code, subsystem, stage/tick and causal identity; first divergence is reproducible without using rendered text as an oracle. | Drop the malformed diagnostic record while preserving the original save/replay/input. |
| `OBS-04` | Remove, block, backpressure and crash telemetry exporters. | No gameplay tick blocks and authoritative state/outcome stays unchanged; bounded local drop counters report loss. | Disable the exporter and keep local diagnostics. |
| `PRIVACY-01` | Run sensitive/redaction fixtures and scan developer outputs. | Default output contains no raw user paths, secrets, prompts, voices or imported/protected bytes. | Quarantine, redact and regenerate the affected local output. |
| `TOOL-03` | Compare direct application-service results with CLI/JSON and optional adapters. | Canonical results match for every supported fixture. | Disable the divergent adapter and keep the CLI/JSON path. |
| `TOOL-04` | Build a neutral physical prototype from a cold public-tool workflow. | Public context plus CLI can create, validate and pack it without private crate/backend knowledge. | Fix the context/CLI and keep the authored procedural fallback. |
