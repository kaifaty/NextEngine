# ADR-041: Owner-thread quiescent GlobalAlloc measurement fast path

| Поле | Значение |
|---|---|
| ID | ADR-041 |
| Статус | Accepted |
| Версия | 1.1 |
| Дата решения | 2026-08-01 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-039](039-tooling-only-process-wide-system-global-allocator-measurement.md), [ADR-040](040-fixed-tls-sharded-global-allocator-measurement.md) |
| Заменяет | Узко заменяет только owner-thread часть active callback и close protocol [ADR-040](040-fixed-tls-sharded-global-allocator-measurement.md): поток, открывший окно и владеющий непередаваемым token, использует exact const-TLS window cookie и owner-only counters без per-call slot admission RMW. Для всех foreign threads fixed-slot registration, один `SeqCst` admission RMW, exact identity postcheck и close handshake ADR-040 остаются обязательными. Crate/unsafe/`System`/count/report boundary ADR-039, process-wide scope, `3%`/`64 MiB` limits и fail-closed fallback не меняются. |
| Заменён | [ADR-042](042-unobserved-deallocation-system-pass-through.md) узко исключает unobserved `dealloc` из owner/foreign state, TLS, slot, fault и close paths после retained implementation overhead `+5.88%`. Owner fast path этого ADR остаётся Accepted без изменений для `alloc`, `alloc_zeroed` и `realloc`. |

## Контекст и retained evidence

> Актуальный `dealloc` pass-through определяется ADR-042. Все упоминания
> deallocation callback ниже сохраняются как historical/evidence context;
> текущие owner/foreign admission и close clauses этого ADR применяются только
> к count-bearing `alloc`, `alloc_zeroed` и `realloc`.

ADR-040 устранил четыре contended global RMW и сохранил один cache-line-local
slot RMW на каждый active allocator callback. Реализация прошла exact-count,
race, recursion, overflow, pointer/data, root-parity, resource и pinned Windows
codegen checks. Однако первый и единственный полный candidate run этого
протокола сохранён как
`allocator-counter-check-candidate-3-20260801/allocator-counter-check-v1.json`
с SHA-256
`E7EC60FD5B84936D7CB46880F5E6F50E4F051B5C6CDE02C14EA357A06EBB46EF` и дал:

- `System` median `1 517 036 100 ns`;
- inactive wrapper median `1 534 132 200 ns`, `+112 bp` (`+1.12%`), `PASS`;
- enabled wrapper median `1 754 392 600 ns`, `+1 564 bp` (`+15.64%`), `FAIL`;
- reserved state `557 096 B`, exact state/archive/identity/kernel roots совпали.

Focused diagnostic того же kernel насчитал `15 253 608` successful
`alloc`/`realloc` calls и подтвердил высокую allocator traffic density.
Release codegen отдельно доказал один locked slot RMW на каждый active call,
включая не входящий в gross counters `dealloc`. Aggregate enabled delta также
содержит TLS, branches, `System` и counter stores, поэтому evidence не выдаёт
его за изолированную стоимость одного RMW. Повтор неизменённого candidate,
повышение `3%` budget или сокращение kernel запрещены. Metric остаётся
unavailable до нового implementation и его единственного полного candidate
check.

Measured kernel выполняет измеряемый runtime work на потоке, который вызывает
`begin` и `finish`. Этот поток имеет более сильный порядок, чем неизвестный
foreign worker: в safe Rust непередаваемый token физически не может вызвать
`finish` параллельно с allocator callback на том же thread. Его counters можно
читать по same-thread sequenced-before, не создавая отдельный admission edge.
Foreign workers по-прежнему могут аллоцировать параллельно и потому сохраняют
полный ADR-040 protocol.

## Решение

### Неизменные границы

- Counter остаётся process-wide: все calls, дошедшие до tooling global wrapper
  в exact active window, принадлежат либо owner fast path, либо foreign
  ADR-040 path. Worker/async allocations не пропускаются и не sampling-уются.
- Каждый contract-valid call делегируется ровно одному matching
  `System::{alloc, alloc_zeroed, realloc, dealloc}` без изменения allocator
  policy, результата или shipping dependency graph.
- Exact successful gross count/bytes semantics, PID/window/report scope,
  overflow/failure semantics и запрет live/peak claims остаются из ADR-039.
- Разрешённый `unsafe` не расширяется. Owner identity, counters и recursion
  реализуются safe `thread_local!`/`Cell`; OS TLS FFI, `ThreadId`, raw pointer к
  shard, `UnsafeCell` и mutable static запрещены.

### Непередаваемый owner token

`begin()` вызывается единственным measurement-owner thread. `Measurement`
содержит zero-sized `PhantomData<Rc<()>>`, поэтому является `!Send + !Sync` и
не создаёт `Rc` или heap allocation. Window ID остаётся монотонным, ненулевым
exact cookie; bool owner marker недостаточен.

При phase `Starting` owner через const-initialized destructor-free TLS:

1. проверяет отсутствие callback recursion и stale owner cookie;
2. lazily claim-ит один never-reused slot из того же общего реестра `4 096`
   lifetime participants, если этот thread ещё не зарегистрирован;
3. записывает exact `owner_window_id`;
4. обнуляет шесть owner `Cell<u64>` counters;
5. только после этого публикует `Active(window_id)` через `SeqCst`.

Любая ошибка установки marker очищает exact совпавший marker до возврата либо
оставляет process-local measurement `Poisoned`. `mem::forget(token)` не даёт
ложный snapshot: следующий `begin` видит незакрытый `Active` и fail-closed
poison-ит process.

`finish` и `Drop` runtime-проверяют PID и exact current-thread TLS cookie.
Mismatch конкурирует через exact `Active(window_id) -> Poisoned` и никогда не
читает counters другого thread. Это защищает от stale marker и unsafe misuse,
не заменяя статическую `!Send + !Sync` гарантию.

### Owner active callback без admission RMW

После единственного initial `SeqCst` control load с exact
`Active(window_id)` callback читает const TLS:

- exact cookie match выбирает owner fast path;
- mismatch выбирает неизменный foreign path ADR-040;
- TLS failure до admission exact-poison-ит captured Active identity.

Owner fast path:

1. проверяет и устанавливает TLS recursion flag;
2. вызывает ровно один matching `System` method;
3. при non-null success checked-обновляет соответствующие owner-only
   `Cell<u64>` count и bytes;
4. explicit non-panicking epilogue очищает recursion flag.

На owner success path нет slot claim/access, atomic RMW, второго control load
или postcheck. Overflow и recursion exact-poison-ят active measurement, после
чего partial counters не публикуются. Failure helper может содержать cold RMW;
он не является success hot path. Callback не полагается на unwinding/RAII для
очистки флага.

Это корректно, потому что `finish`/`Drop` token нельзя выполнить параллельно на
том же owner thread. Единственный конкурентный control transition от foreign
thread может быть только terminal `Poisoned`, при котором snapshot запрещён.
Same-thread owner counter stores sequenced-before owner `finish` reads и не
нуждаются в межпоточном admission edge.

### Foreign active callback и close

Любой thread без exact current owner cookie остаётся foreign, даже если он был
owner предыдущего окна. Для него без изменений обязательны:

- lazy monotonic claim одного из `4 096` fixed slots;
- odd `SeqCst fetch_add(1)` admission и exact Active identity postcheck;
- один `System` call, owner-only atomic counter load/store и even `Release`;
- pre-admission exact poison, admitted sticky fault и double `SeqCst`
  close-side handshake из ADR-040.

Каждый новый owner заранее регистрируется в том же fixed slot space, чтобы
owner TLS state не обходил process-lifetime participant/resource bound. Этот
slot остаётся dormant в owner window: owner callback не читает его sequence или
counters и не делает slot RMW. Если thread позже станет foreign, он использует
уже принадлежащий ему never-reused slot. Новый owner после исчерпания `4 096`
distinct participating threads fail-closed получает `SlotExhausted` до Active.

`finish` выполняет строго:

1. allocation-free exact PID/TLS cookie validation без очистки marker;
2. `SeqCst` CAS `Active(window_id) -> Closing(window_id)`;
3. чтение same-thread owner counters и очистку exact owner marker;
4. ADR-040 stable scan всех foreign slots и checked aggregation;
5. fault/PID/overflow validation и только затем `Idle` publication.

После `Closing` новые callbacks сразу делегируются в `System` без TLS. Owner
callback не может быть in flight на том же thread; admitted foreign callbacks
обязательно завершаются перед snapshot. Marker обязан быть очищен до `Idle`;
невозможность exact cleanup оставляет measurement poisoned.

`Drop` exact-poison-ит abandoned Active window и очищает только совпавший owner
marker. Successful или уже завершившийся error path подавляет повторное Drop
poisoning. Sequential window может открыться на другом thread: stale TLS cookie
первого owner не совпадает с новым ID и не даёт owner privilege.

### Bounds и target admission

Const TLS содержит slot selector/recursion state, exact owner cookie и шесть
owner counters. Owner и foreign используют один process-lifetime реестр ровно
из `4 096` distinct participating threads; owner registration выполняется вне
Active/timer, а callback не обращается к slot. Fixed slots вместе с worst-case
TLS/control state зарегистрированных participants остаются меньше `1 MiB` и
обязаны проходить общий `<=64 MiB` limit.

Pinned Rust `1.93.0` release codegen проверяется отдельно на Windows x86_64
MSVC и native Linux x86_64 GNU. Owner success path обязан иметь direct native
const TLS, ровно один initial control load, direct `System`, zero locked/RMW
instructions, zero second control/sequence access и no EH control-flow/stack
probe. Foreign helper обязан сохранить ровно один owned-slot RMW и exact
postcheck; inactive path — один state load, no TLS/RMW и direct `System`.
Windows unwind metadata/personality без `invoke`/landing/catch/cleanup не
считается EH control flow.

Если compiler/target не доказывает эти свойства, counter на target остаётся
`NOT_RUN`; platform FFI, relaxed exactness или shipping allocator replacement
не являются fallback.

## Memory-model invariants

1. Owner cookie/counter reset sequenced-before `SeqCst Active` publication.
2. Safe `Measurement: !Send + !Sync` и exact TLS cookie обеспечивают, что
   successful Closing CAS выполняет тот же thread, который пишет owner Cells.
3. Owner allocator callback и owner `finish` не исполняются одновременно;
   therefore all owner counter writes happen-before their same-thread reads.
4. Foreign odd admission/postcheck/Closing/handshake сохраняют единый ADR-040
   SC order и не могут одновременно пропустить admitted call.
5. Exact cookie match — единственный owner selector. Old owner нового окна
   является foreign; stale callback не обновляет новый owner counter.
6. Любая recursion, overflow, TLS/slot failure, wrong owner или admitted fault
   предотвращает `Idle` snapshot publication.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | compile-negative `!Send/!Sync`; owner known-count; owner+foreign exact counts; sequential rotating owners; owner/foreign shared capacity `4 096` и distinct-owner `4 097` exhaustion; nested/abandoned/wrong-owner/PID/TLS/recursion/overflow faults | Exact breakdown либо один stable poison; no partial snapshot | Counter unavailable |
| `fast` race matrix | foreign observed-before-odd, odd-before-close, close between odd/postcheck, foreign poison vs owner close, stale old→new window, simultaneous claim, slot exhaustion/timeout | No missed admitted call или cross-window attribution | Counter unavailable |
| Boundary/codegen audit | pinned release Windows/Linux | Inactive one load/no TLS; owner zero RMW/second control/sequence/EH; foreign one RMW+postcheck; direct `System`; exact unsafe/dependency allowlists | Target `NOT_RUN` |
| `allocator-counter-check` | unchanged rotated 2+15 release kernel | inactive/enabled independently `<=3%`, reserved state `<=64 MiB`, pointer/data/null semantics and exact roots unchanged | Metric disabled; no retry-to-green |
| `performance` smoke/worker/long | prepared in-process scenario window | Nonzero exact process/PID/window evidence; bootstrap/report excluded | Stable unavailable diagnostic |
| `play` / `persistence-replay` | counter/profiler off/on | Exact command/event/save/replay/state/ledger roots | Reject divergent run |

## Рассмотренные варианты

- **Оставить ADR-040 и принять `+15.64%`.** Отклонено: нарушает неизменный
  `3%` product check.
- **Убрать RMW у всех threads.** Отклонено: foreign callback может уже увидеть
  Active, а owner finish одновременно Closing; без admission order возникает
  missed call.
- **Считать только owner thread.** Отклонено: сужает обязательный process-wide
  scope и пропускает worker allocations.
- **Передаваемый token с quiescence flag.** Отклонено: возвращает межпоточный
  admission/close race, который и требовал locked protocol.
- **`ThreadId`, OS TLS FFI или platform thread handle на callback.** Отклонено:
  дороже exact const-TLS cookie и может добавить allocation/lock/new unsafe.
- **Approximate batching/sampling или повышение budget.** Отклонено ADR-039 и
  retained evidence policy.

## Последствия

- Следующая code subphase меняет только owner dispatch/counters/close и
  соответствующий codegen gate; foreign ADR-040 proof остаётся review anchor.
- Первый полный candidate после реализации и focused/codegen gates сохраняется
  независимо от verdict и не повторяется как retry-to-green.
- До exactness, codegen, profiler parity и overhead `PASS` allocator fields
  остаются unavailable, B-12 — `OPEN`, hard timing calibration не начинается.
- Native Linux TLS/codegen/performance доказательство записано отдельным
  pending action; Windows result не подменяет его.
