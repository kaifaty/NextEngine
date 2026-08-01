# ADR-040: Fixed TLS-sharded GlobalAlloc measurement protocol

| Поле | Значение |
|---|---|
| ID | ADR-040 |
| Статус | Accepted |
| Версия | 1.1 |
| Дата решения | 2026-08-01 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-039](039-tooling-only-process-wide-system-global-allocator-measurement.md) |
| Заменяет | Узко заменяет в [ADR-039](039-tooling-only-process-wide-system-global-allocator-measurement.md) обязательный per-call checked-CAS/in-flight protocol и запрет любого TLS state в callback. Crate/dependency boundary, `System` delegation, exact gross counting semantics, report schema/scope, shipping prohibition, unsafe scope, 3%/64 MiB limits и fail-closed fallback ADR-039 остаются Accepted. |
| Заменён | Owner-thread active callback/admission и close clauses узко заменены [ADR-041](041-owner-thread-quiescent-global-allocator-measurement.md) после retained enabled overhead `+15.64%`. Для всех foreign threads fixed-slot registration, один `SeqCst` admission RMW, identity postcheck и close handshake этого ADR остаются Accepted. |

## Контекст и измеренное опровержение

> Актуальный owner-thread fast path определяется ADR-041. Описанный ниже
> per-call slot protocol остаётся обязательным для foreign threads и evidence
> context исходного owner implementation.

Первая реализация ADR-039 сохранила один inactive state load и для каждого
active allocation делала checked-CAS admission, повторную exact-window
проверку, atomic count/byte updates и atomic slot release. Она прошла
known-count, multithreaded close, sequential-window, overflow, pointer/data,
root-parity и 64 MiB checks, но не прошла обязательный overhead check.

Один и тот же release kernel
`nextengine-allocator-counter-kernel-v1` (`900` prepared live ticks, два warmup
и `15` retained rounds в rotated order) дал после hot-path inlining:

- `System` median `1 638 965 400 ns`;
- inactive wrapper median `1 596 630 500 ns`, `-258 bp` (`-2.58%`);
- enabled wrapper median `1 900 470 300 ns`, `+1 595 bp` (`+15.95%`);
- reserved state `80 B`, exact state/archive/identity roots совпали.

Это не measurement noise, которое разрешено скрыть повторным запуском:
предыдущий retained run также дал enabled `+17.25%`. Четыре contended locked
RMW на миллионы allocator calls являются причиной, а увеличение `3%` budget,
уменьшение kernel или выбор удачного run запрещены. До новой реализации и её
единственного честного candidate check allocator metric остаётся unavailable.

Реализация этого ADR сохранила exact counts/roots, прошла resource и pinned
Windows codegen gates, но retained candidate-3 снова нарушил overhead limit:
`System 1 517 036 100 ns`, inactive `1 534 132 200 ns` (`+1.12%`, `PASS`),
enabled `1 754 392 600 ns` (`+15.64%`, `FAIL`), reserved state `557 096 B`.
Focused diagnostic насчитал `15 253 608` successful counted calls, а release
codegen доказал один locked slot RMW на каждый active call, включая `dealloc`.
Aggregate enabled delta не изолирует стоимость RMW от TLS/branches/`System` и
counter stores. ADR-041 проверяет сформулированную из этих двух evidence
гипотезу, исключая RMW только для same-thread measurement owner; foreign proof
ниже не меняется.

## Решение

### Неизменные границы

- `next_process_allocation_counter` остаётся std-only internal crate с
  единственным reverse dependency `xtask`; shipping roots его не линкуют.
- Wrapper по-прежнему делегирует каждый contract-valid operation ровно одному
  matching `System::{alloc, alloc_zeroed, realloc, dealloc}` без изменения
  allocator policy или result.
- Разрешённый `unsafe` не расширяется: только `unsafe impl GlobalAlloc` и
  прямые calls к четырём `System` methods. TLS, slot ownership, counters и
  close protocol реализуются safe atomics/`Cell`; `UnsafeCell`, raw pointer к
  shard, platform TLS FFI и mutable static не разрешаются.
- Exact operation semantics и versioned `ProcessAllocationCounterV1` из
  ADR-039 не меняются. Approximate batching, sampling и net/live claims
  запрещены.

### Fixed process slots и TLS selector

Process заранее содержит ровно `4 096` fixed, cache-line-isolated slots. Каждый
slot включает:

- monotonic atomic even/odd `sequence`;
- шесть `AtomicU64` для exact count/bytes по `alloc`, `alloc_zeroed`,
  `realloc`.

`thread_local!` хранит только const-initialized, destructor-free slot index и
recursion flag. Inactive callback TLS не касается. Известный thread MAY
pre-touch/claim slot до window; иначе его первый active callback через один
bounded checked global slot claim получает monotonic index и сохраняет его в
TLS. Это first-touch operation, не steady hot path; slot никогда не
переиспользуется другим thread до завершения process. Такая validated lazy
регистрация сохраняет process-wide coverage для заранее неизвестных threads,
чьи Rust allocations действительно доходят до этого global wrapper. Она не
заявляет учёт приватных native allocations Vulkan driver или иной библиотеки.
Exhaustion, недоступный TLS key либо recursion poison-ит measurement и
делегирует operation без изменения `System` result.

Failure до slot admission, где odd RMW ещё не защищает close (`TLS.try_with`
failure, slot/claim exhaustion либо pre-admission sequence overflow), MUST
линейризоваться exact `SeqCst` transition captured `Active(window_id) →
Poisoned(window_id, stable_code)`. Если `Closing` уже выиграл transition,
callback делегируется untracked без retroactive poison: он не был admitted в
завершившееся окно. Fault после odd admission публикуется до возврата slot в
even; close сначала наблюдает completion, затем fault и не выпускает partial
snapshot. Простого позднего независимого `faults.fetch_or` для pre-admission
failure недостаточно.

Const TLS path MUST быть доказан для pinned Rust `1.93.0` на Windows x86_64 и
Linux x86_64: first touch и steady access не аллоцируют, не берут lock, не
регистрируют destructor и не unwind-ят. Если target/codegen не даёт такого
пути, counter на нём unavailable; platform FFI или `std::thread::current()` из
allocation callback не являются fallback.

### Active callback protocol с одним sharded admission RMW

Для каждого active call:

1. callback читает exact `Active(window_id)`; inactive/closing state сразу
   делегируется тому же `System` без TLS;
2. `TLS.try_with` получает owned slot и отвергает recursion;
3. owner читает свой even `sequence`, checked-вычисляет следующую odd/even
   пару и одним `SeqCst fetch_add(1)` публикует odd; это единственный
   steady-state RMW и он обращается только к cache line owned slot;
4. callback повторно читает тот же exact `Active(window_id)` через `SeqCst`;
   mismatch публикует even и делегирует call untracked;
5. при match выполняется ровно один `System` call; non-null success делает
   checked owner-only atomic load/store соответствующих count и bytes;
6. callback одним explicit non-panicking epilogue публикует even `sequence`
   через `Release` store и очищает recursion flag. Hot helper не полагается на
   unwind/RAII cleanup и не создаёт exception landing pad.

Steady active callback не делает global admission/release либо counter RMW:
остаётся один per-thread sequence `fetch_add` RMW вместо четырёх contended RMW
исходного protocol. Counter atoms используются как safe race-free storage, но их
единственный writer — owning thread. Overflow sequence/counter либо
неожиданная odd/reentrant state ставит sticky poison; partial snapshot не
публикуется.

### Close-side coherence handshake

Простой `Acquire` scan недостаточен: closing thread формально может пропустить
callback, который уже прочитал `Active`, но ещё не создал happens-before edge.
Поэтому expensive RMW переносится за пределы timed workload в `finish`:

1. token `SeqCst` CAS-переводит exact
   `Active(window_id) → Closing(window_id)`;
2. для **каждого** из 4 096 slots, включая ещё не claimed, close делает
   bounded `sequence.fetch_add(0, SeqCst)` handshake;
3. odd value означает admitted call и повторяется до even; counters читаются
   только между двумя одинаковыми even handshake values;
4. RMW читает предшествующую modification того же `sequence`; even `Release`
   publication синхронизирует все предшествующие counter stores. Единый
   `SeqCst` order не допускает both-miss cycle: если callback postcheck ещё
   увидел old Active, его odd admission RMW расположен до Closing CAS и close
   handshake. Изменившийся либо odd sequence заставляет повторить snapshot;
5. checked aggregate, PID/window/fault validation выполняются до перехода в
   `Idle`.

Call, который опубликовал odd до close и успел повторно увидеть old Active,
обязательно наблюдается handshake и завершается до snapshot. Call, который
проснулся только после `Closing` или нового sequential window, не может пройти
exact identity recheck, не обновляет counters и только возвращает свой slot в
even. `begin` обнуляет все fixed counter atoms до `Active(new_window_id)`;
monotonic slot sequence между windows не сбрасывается.

Close handshake, checked aggregation и slot reset выполняются после внешнего
workload timer. Они входят в instrumentation operation, resource/timeout checks
и tests, но не маскируются как engine workload latency.

### Bounds, fallback и admission

- `reserved_bytes()` считает весь fixed slot array и worst-case TLS/control
  state при 4 096 claimed threads и MUST оставаться `<=64 MiB`; 128-byte slot
  layout вместе с TLS/control/report scratch остаётся меньше `1 MiB`.
- Bounded close timeout, slot/sequence/counter/window overflow, wrong token,
  nested begin, recursion, TLS failure и slot exhaustion дают stable
  `PERF_ALLOCATOR_*` poison по описанной выше linearization и `NOT_RUN` без
  partial numbers.
- Enabled и inactive medians обязаны независимо пройти тот же targeted
  overhead check `<=3%`. Первый полный candidate после реального protocol
  change сохраняется независимо от verdict и не повторяется как
  retry-to-green.
- Failure снова оставляет metric disabled. Он не разрешает повышать budget,
  уменьшать representative kernel, линковать wrapper в shipping roots или
  менять allocator.

## Memory-model invariants

1. Begin counter resets happen-before `Active` publication; accepted callback
   sees that identity через `SeqCst` load.
2. Odd sequence RMW precedes exact Active recheck в едином `SeqCst` order. Если
   recheck прочитал old Active, later Closing CAS и slot handshake следуют за
   admission; store-buffering outcome, где callback и close одновременно
   пропускают друг друга, невозможен.
3. Counter stores are sequenced-before even Release; close `SeqCst` RMW that
   observes that even value makes them visible before aggregation.
4. No two threads write one slot. Slot ownership is never recycled, so thread
   exit cannot create ABA ownership.
5. Stale old-window callback can change only its atomic sequence after identity
   mismatch; it cannot update a new window counter. Same-thread recursion
   invalidates the whole measurement before publication.
6. A pre-admission failure and `Closing` compete through the same exact SC
   control transition. Therefore close either observes `Poisoned` and fails
   closed, or wins first and the non-admitted call cannot invalidate an already
   closed snapshot retroactively.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | known-count all operations; pre-touched and lazy first TLS claim; 4 096-slot ownership; slot 4 097 exhaustion; recursion; sequence/counter overflow; nested/wrong/abandoned token | Exact breakdown либо sticky stable poison; no allocation/log/lock/unwind from callback | Counter unavailable |
| `fast` race matrix | odd-before-close, observed-before-odd, close between odd/recheck, pre-admission failure vs close, stale old→new window, thread exit, simultaneous first claim, close timeout | No missed admitted call, no cross-window attribution, no partial snapshot | Counter unavailable |
| Boundary/assembly audit | pinned Rust 1.93.0 Windows/Linux release | Inactive path: one state load/direct `System`; steady active: exactly one owned-slot locked RMW and no global release/counter RMW; TLS first/steady path allocation-free; hot helper has no unwind landing pad and bounded stack frame; unsafe/source/dependency allowlists exact | Target `NOT_RUN` |
| `allocator-counter-check` | unchanged rotated 2+15 release kernel | Both inactive/enabled `<=3%`, state `<=64 MiB`, pointer/data/null semantics and exact roots preserved | Metric disabled; hard timing `NOT_RUN` |
| `performance` smoke/worker/long | prepared in-process window only | Nonzero exact process/PID/window evidence; bootstrap/report excluded | Stable unavailable diagnostic |
| `play` / `persistence-replay` | counter/profiler off/on | Exact command/event/save/replay/state/ledger roots | Reject divergent run |

## Рассмотренные варианты

- **Оставить per-call CAS и принять +15.95%.** Отклонено: нарушает Accepted 3%
  product check.
- **Повысить budget, сократить kernel или перезапускать до PASS.** Отклонено:
  меняет criterion после evidence и создаёт retry-to-green.
- **Unsafe non-atomic shard (`UnsafeCell`).** Отклонено: AtomicU64 owner-only
  load/store даёт тот же hot-path класс без расширения unsafe boundary.
- **Approximate thread-local batches/sampling.** Отклонено: теряет exact gross
  call/byte semantics и close-point completeness.
- **Release odd store + Acquire/AcqRel close scan.** Отклонено: допускает
  store-buffering both-miss, где callback читает old Active, а close — old even.
  `SeqCst` odd RMW/control postcheck/Closing CAS/handshake нужны для единого
  admission order.
- **OS thread ID/TLS FFI или `std::thread::current()` в callback.** Отклонено:
  может добавить allocation/lock/platform unsafe и recursion.
- **Shipping allocator replacement.** Отклонено ADR-036/ADR-039 и не относится
  к measurement.

## Последствия

- Следующая code subphase заменяет только internal window protocol; report и
  gameplay contracts не меняются.
- ADR-039 остаётся authority для crate/unsafe/System/count/report boundary;
  runtime admission/close clauses читаются через этот superseding ADR.
- До implementation, targeted race/assembly/parity checks и overhead PASS
  allocator fields остаются unavailable, а roadmap blocker B-12 — `OPEN`.
