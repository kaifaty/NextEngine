# ADR-039: Tooling-only process-wide System GlobalAlloc measurement boundary

| Поле | Значение |
|---|---|
| ID | ADR-039 |
| Статус | Accepted |
| Версия | 1.3 |
| Дата решения | 2026-08-01 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-002](002-rust-first-ffi-and-ecs-facade.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-033](033-physx-grounded-capsule-parity-ffi-boundary.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-038](038-versioned-production-worker-handoff-diagnostic.md) |
| Заменяет | Узко заменяет allocator-instrumentation prohibition/measurement gap [ADR-036](036-thoth-reference-performance-profile.md): exact process-wide host allocation counter разрешён только в tooling boundary ниже. Также узко заменяет формулировки [ADR-002](002-rust-first-ffi-and-ecs-facade.md) и [ADR-033](033-physx-grounded-capsule-parity-ffi-boundary.md), по которым любое workspace `unsafe`-исключение обязано быть FFI и `next_physics_physx_ffi` является единственным таким местом. PhysX FFI boundary, запрет недоказанной shipping allocator replacement, THOTH hard gate, budgets, baseline и representative R2–R5 requirements не меняются. |
| Заменён | Runtime in-flight protocol в строках `begin`/callback/`finish` и запрет const destructor-free TLS selector узко заменены [ADR-040](040-fixed-tls-sharded-global-allocator-measurement.md), а same-thread owner path после второго measured enabled failure узко заменён [ADR-041](041-owner-thread-quiescent-global-allocator-measurement.md). [ADR-042](042-unobserved-deallocation-system-pass-through.md) исключает unobserved `dealloc` из measurement state/admission/close/fault domain, сохраняя exactly-once direct `System::dealloc` и `delegated-not-subtracted`. ADR-040/ADR-041 остаются authority для foreign/owner count-bearing callbacks. Crate/dependency, unsafe/System delegation, exact gross counting, report/scope, bounds и fail-closed semantics этого ADR остаются Accepted. |

## Контекст

> Актуальный admission/close protocol определяется ADR-041 для same-thread
> measurement owner и ADR-040 для foreign count-bearing callbacks; direct
> `dealloc` pass-through определяется ADR-042. Описанный ниже checked-CAS
> global in-flight protocol сохраняется как исходное решение и evidence
> context, но не является текущим implementation contract.

ADR-036 требует exact `allocator_allocated_bytes` и
`allocator_allocation_count`, но текущий performance report честно оставляет
их unavailable. End-of-run working set или private bytes не являются
allocator traffic, live heap либо peak RSS и не могут закрыть этот gap.

`GlobalAlloc` является unsafe process-wide boundary. Его нельзя добавлять в
shipping roots, маскировать как FFI или размазывать по runtime crates. Нужен
один изолированный tooling crate, который при выключенном измерении сохраняет
поведение `std::alloc::System`, а при явном scenario window считает только
реально дошедшие до global allocator успешные calls.

## Решение

### Crate и dependency boundary

- Создаётся internal crate `tools/process-allocation-counter` с package name
  `next_process_allocation_counter` и `publish = false`.
- Единственный reverse dependency crate — `tools/xtask`. Crate не экспортируется
  через `crates/contracts` и не входит прямо или транзитивно в `game`,
  `headless`, `next` shipping tools, application/runtime/reference-game либо
  shipping package graph.
- Crate использует только standard library. Его global allocator wrapper
  устанавливается только в `xtask` process и всегда делегирует memory operation
  тому же `std::alloc::System`; allocator policy, pooling, caching, retry,
  padding и failure behavior не меняются.
- Это не FFI. Exact crate записывается в отдельную workspace metadata category
  для tooling global-allocator instrumentation и MUST NOT добавляться в FFI
  allowlist. Existing backend/FFI allowlists и PhysX ABI не меняются.
- Исключение из workspace `unsafe_code = "forbid"` разрешено только внутри
  этого crate для `unsafe impl GlobalAlloc` и минимальных direct calls к
  `System::{alloc, alloc_zeroed, realloc, dealloc}`. Каждый unsafe block имеет
  локальный `SAFETY` rationale; `unsafe_op_in_unsafe_fn` остаётся denied. Public
  measurement API полностью safe. Никакой другой unsafe code этим ADR не
  разрешён.

### Exact counting semantics

Wrapper явно реализует все четыре `GlobalAlloc` operation и для каждого
contract-valid call делегирует ровно один раз соответствующему методу
`System`. Allocation callback не аллоцирует, не логирует, не берёт lock, не
инициализирует thread-local state, не вызывает process API и не unwind-ит.

| Operation | Успешный observation внутри admitted window |
|---|---|
| `alloc(layout)` | non-null result добавляет `1` к `alloc_count` и `layout.size()` к `alloc_bytes` |
| `alloc_zeroed(layout)` | non-null result добавляет `1` к `alloc_zeroed_count` и `layout.size()` к `alloc_zeroed_bytes`; call не проходит через wrapper `alloc` и не double-counted |
| `realloc(ptr, old_layout, new_size)` | non-null result добавляет `1` к `realloc_count` и полный `new_size` к `realloc_bytes`, независимо от in-place/moved result; это не delta к `old_layout.size()` |
| `dealloc(ptr, layout)` | делегируется ровно один раз и ничего не вычитает; `GlobalAlloc` не даёт отдельного observable success result для deallocation |

Null-returning allocation/reallocation не меняет successful counters. Checked
aggregate fields равны:

```text
allocator_allocation_count = alloc_count + alloc_zeroed_count + realloc_count
allocator_allocated_bytes  = alloc_bytes + alloc_zeroed_bytes + realloc_bytes
```

Это gross successful requested allocation traffic, а не live bytes, peak heap,
RSS, leak detector или число source-level allocations. Compiler MAY eliminate
или stack-promote source allocations; exactness относится только к calls,
которые фактически достигли process global allocator.

Rust `GlobalAlloc` требует non-zero `layout.size()` для `alloc`/
`alloc_zeroed` и `new_size > 0` для `realloc`. Safe API crate не создаёт
zero-size calls, а tests не вызывают такой unsafe operation. Wrapper не
нормализует zero к одному byte и не считает его success. Если defensive
non-panicking check всё же наблюдает zero-size request, measurement становится
invalid с `PERF_ALLOCATOR_INVALID_CALL`; это не делает уже нарушившего unsafe
precondition caller sound и не обещает recovery от undefined behavior.

### Runtime-enabled window и in-flight protocol

Inactive wrapper выполняет direct `System` delegation после одного
non-allocating state observation и не обновляет counters. Измерение включает
только явно открытое safe `xtask` scenario window; project bootstrap, unrelated
smoke phases и report serialization выполняются вне него.

`begin()` разрешён только из `Idle` при `in_flight == 0`. Он checked-инкрементит
monotonic `window_id`, сохраняет текущий PID, обнуляет operation counters и
линеаризует `Active(window_id)`. Ровно один window может быть active.

Каждый allocator call использует следующий allocation-free protocol:

1. прочитать `Active(window_id)`; inactive call сразу делегируется без учёта;
2. checked-CAS зарезервировать один `in_flight` slot;
3. повторно проверить тот же `Active(window_id)`; mismatch освобождает slot до
   untracked delegation;
4. при match удерживать slot во время единственного `System` call, checked-add
   successful counters и затем всегда освободить slot non-allocating guard;
5. overflow любого counter, `window_id` или `in_flight` ставит sticky poison;
   allocator result при этом не меняется, но measurement не публикуется.

`finish(token)` CAS-переводит тот же window `Active → Closing`, запрещает новые
attributions, bounded-wait-ит admitted `in_flight == 0`, сверяет start/end PID,
снимает snapshot и только после этого возвращает state в `Idle`. Report
serialization начинается после close. Последовательные корректно завершённые
windows получают новый ID и независимые zero-based counters.

Nested `begin`, wrong/stale token, abandoned guard, PID mismatch, counter/
window/in-flight overflow и bounded close timeout fail closed: process-local
measurement становится poisoned, содержащий performance run получает stable
`NOT_RUN`, а повтор требует нового `xtask` process. Poison никогда не меняет
результат `System` allocation и не превращает неполный snapshot в числа.

### Versioned report, scope и fallback

Успешный report содержит versioned `ProcessAllocationCounterV1` минимум с:

- `schema_version`, `methodology_version`, `counter_scope =
  "xtask-process-scenario-window-v1"`;
- PID, `window_id`, allocator identity `rust-std-system` и exact scenario
  identity;
- отдельными successful bytes/counts для `alloc`, `alloc_zeroed`, `realloc` и
  checked aggregate fields выше;
- explicit `deallocation_semantics = "delegated-not-subtracted"`.

Containing performance report/methodology version MUST измениться до
публикации этих новых fields; существующие V1 bytes не переопределяются молча.
PID — operational identity, не security credential и не gameplay data.

Если hot workload выполняется в child process, который не содержит этот exact
boundary, parent counter не заявляет process-wide coverage и возвращает
`PERF_ALLOCATOR_COUNTER_UNAVAILABLE`. Unsupported platform, missing hook,
invalid/poisoned или nested window, overflow и process mismatch fail closed:
содержащий run, включая report-only diagnostic, получает `NOT_RUN` со stable
diagnostic и не публикует partial allocator values. Working set/private
bytes/RSS не подставляются в allocator fields. Настоящий live/peak RSS, если
понадобится, получает отдельное имя, scope и sampler.

### Authority и resource bounds

Counter и его window state являются operational tooling data. Они не входят в
command/event/save/replay/state roots, не выбирают work, target tick, fallback,
admission или eviction. Profiler on/off сохраняет exact authoritative hashes.

Вся instrumentation memory preallocated/fixed-size и вместе с остальным
profiling state остаётся `<= 64 MiB`. Enabled и inactive paths проходят
declared overhead check `<= 3%`. Failure этого check отключает allocator metric
и оставляет hard run `NOT_RUN`; он не разрешает shipping allocator replacement,
увеличение budget или ослабление correctness gate.

Accepted status фиксирует boundary, но не утверждает, что crate, counter или
ProductCheck уже реализованы. До реализации и прохождения checks current
allocator fields остаются unavailable, а B-12 — `OPEN`.

## Product impact

Performance tooling получает точный process/scenario-scoped allocator traffic
counter без изменения shipping binaries и без подмены memory semantics.
Gameplay, persistence/replay, PhysX boundary, public contracts и hard timing
thresholds не меняются.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | known-count `alloc`/`alloc_zeroed`/`realloc`, failure, sequential-window reset, multithreaded close race, nested/abandoned/PID/overflow/in-flight faults и strict JSON round-trip | Exact per-operation/aggregate counters; invalid state никогда не публикует partial values; boundary scan показывает единственный reverse dependency `xtask` и отдельную non-FFI unsafe category | Оставить hook unavailable и hard counter `NOT_RUN` |
| `play` / `persistence-replay` | Profiler/counter off/on над одинаковым production scenario | Commands, events, save/replay, state и ledger roots exact | Отключить optional counter; не принимать divergent run |
| `performance --scenario smoke|production-worker-soak|long-session-soak --mode report` | Runtime-enabled window охватывает только фактический in-process workload | Versioned scope/PID/window, exact nonzero breakdown where expected, no bootstrap/report leakage, resource limits соблюдены | Stable unavailable/invalid diagnostic без approximate substitution |
| Targeted overhead | System control против inactive и enabled wrapper на compatible host | Reserved state `<=64 MiB`, overhead `<=3%`, allocator pointer/null/data semantics unchanged for valid calls | Counter unavailable; hard timing gate остаётся `NOT_RUN` |

## Рассмотренные варианты

- **End-of-run RSS/working set как allocator counter.** Отклонено: это другой
  scope и не показывает gross allocation traffic или peak.
- **Global allocator wrapper в `game`/`headless`.** Отклонено: diagnostic не
  должен менять shipping dependency/allocator boundary.
- **Считать только `alloc` и использовать default `realloc`/`alloc_zeroed`.**
  Отклонено: default delegation может double-count и скрывает operation kind.
- **Считать net/live bytes через `dealloc`.** Отклонено: deallocation не имеет
  success result, а cross-window lifetimes делают такой delta ложным peak/live
  claim.
- **Объявить unsafe wrapper FFI crate.** Отклонено: `GlobalAlloc` не является
  foreign interface; такая маркировка разрушает проверяемость allowlist.
- **Заменить System более быстрым allocator.** Отклонено: это shipping policy и
  отдельное product/performance решение, не measurement implementation.

## Последствия

- Следующая code subphase добавляет один internal crate, safe `xtask` API,
  versioned report changes, boundary scan и focused fault/overhead/parity tests.
- ADR-002/ADR-033 сохраняют Rust-first и PhysX FFI semantics; заменены только
  FFI-only/uniqueness формулировки unsafe allowlist.
- ADR-036 сохраняет THOTH authority и запрещает недоказанную глобальную замену
  shipping allocator; закрыт только точный tooling measurement design gap.
- Любой новый consumer, allocator policy, shipping link либо новая unsafe
  operation требует отдельного Accepted ADR.
