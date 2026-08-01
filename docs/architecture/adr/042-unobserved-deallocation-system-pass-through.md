# ADR-042: Unobserved deallocation System pass-through

| Поле | Значение |
|---|---|
| ID | ADR-042 |
| Статус | Accepted |
| Версия | 1.0 |
| Дата решения | 2026-08-01 |
| Последняя проверка | 2026-08-01 |
| Нормативные зависимости | [SPEC-00](../00-product-contract.md), [SPEC-01](../01-system-architecture.md), [SPEC-09](../09-tooling-sdk-and-observability.md), [SPEC-12](../12-vertical-slice-conformance.md), [SPEC-23](../23-jobs-memory-resource-residency-and-io-backpressure.md), [ADR-030](030-product-first-development-and-lightweight-validation.md), [ADR-036](036-thoth-reference-performance-profile.md), [ADR-039](039-tooling-only-process-wide-system-global-allocator-measurement.md), [ADR-040](040-fixed-tls-sharded-global-allocator-measurement.md), [ADR-041](041-owner-thread-quiescent-global-allocator-measurement.md) |
| Заменяет | Узко заменяет deallocation callback/admission/close/fault/codegen clauses ADR-039, ADR-040 и ADR-041. `dealloc` остаётся exactly-once matching `System::dealloc`, но как явно не наблюдаемая операция `delegated-not-subtracted` больше не читает measurement state, не входит в TLS/slot admission и не задерживает close. Exact process-wide successful `alloc`/`alloc_zeroed`/`realloc` semantics, owner ADR-041 path, foreign ADR-040 path, tooling-only unsafe boundary, report scope, `3%`/`64 MiB` limits и fail-closed fallback не меняются. |
| Заменён | не заменён |

## Контекст и retained evidence

ADR-039 определил публикуемые allocator values как gross successful requested
traffic трёх операций:

```text
allocator_allocation_count = alloc_count + alloc_zeroed_count + realloc_count
allocator_allocated_bytes  = alloc_bytes + alloc_zeroed_bytes + realloc_bytes
```

`dealloc` ничего не вычитает и не имеет отдельно наблюдаемого success result в
`GlobalAlloc`; report уже объявляет его semantics как
`delegated-not-subtracted`. Несмотря на это, ADR-040 и ADR-041 проводили каждый
active `dealloc` через тот же state/TLS/recursion/slot admission, который
защищает counter publication. Этот protocol не меняет ни одного публикуемого
значения при deallocation. `System::realloc` также может освободить прежний
block без отдельного wrapper `dealloc`, поэтому наблюдение explicit deallocation
изначально не могло образовать exact live/net metric.

Первая и единственная полная проверка реализации ADR-041 сохранена как
`allocator-counter-check-candidate-5-20260801/allocator-counter-check-v1.json`
с SHA-256
`2947B89A84EA77801E651E4C156BE59827BB7ED9032E1D311764238A2E19277B` и дала:

- `System` median `1 670 519 800 ns`;
- inactive wrapper median `1 671 316 400 ns`, `+4 bp` (`+0.04%`), `PASS`;
- enabled wrapper median `1 768 755 000 ns`, `+588 bp` (`+5.88%`), `FAIL`;
- reserved state `786 472 B`, exact state/archive/identity/kernel roots совпали.

Pinned Windows release codegen показывает, что normal valid owner `dealloc`
исполняет `59` instructions до/вокруг backend против `25` у текущего
direct-after-state path: `34`-instruction active premium включает native TLS,
recursion flag, exact owner-cookie dispatch и две дополнительные call/return
pairs, хотя counters не записываются. Foreign `dealloc` дополнительно делает
ADR-040 slot admission/identity handshake. Candidate-5 должен вернуть примерно
`48.12 ms` median, чтобы попасть в `3%`; exact dynamic dealloc count schema не
хранит, поэтому aggregate timing и static codegen не доказывают будущий PASS.
Неизменный verdict даёт только один следующий полный candidate. Candidate-5 не
повторяется и не отбрасывается.

## Решение

### Наблюдаемый домен counter

Count-bearing callbacks — только фактически дошедшие до process global wrapper
`alloc`, `alloc_zeroed` и `realloc`. В exact active scenario window каждый их
non-null result обновляет прежние separate count/bytes fields и checked gross
aggregate по ADR-039. Owner использует ADR-041, а все foreign threads — ADR-040;
process-wide scope, exact window identity и close completeness сохраняются.

`dealloc` является unobserved allocator-policy operation:

- она делегируется, но не создаёт count/bytes, live/peak, fault или participant
  evidence;
- её вызов не делает direct measurement-state update и не получает window
  attribution независимо от observed control phase;
- report сохраняет `deallocation_semantics = "delegated-not-subtracted"`;
- `ProcessAllocationCounterV1` не меняет schema: существующее поле теперь явно
  означает именно unobserved direct pass-through;
- availability/hook proof обязан pre-touch-ить successful count-bearing
  operation; один `dealloc` не считается доказательством установленного hook;
- это не превращает gross traffic в live heap, RSS, leak detector или net
  allocation delta.

### Безусловный direct pass-through

Outer `GlobalAlloc::dealloc(ptr, layout)` обязан безусловно вызвать ровно один
`System::dealloc(ptr, layout)` с теми же pointer/layout и затем вернуть `()`.
На этом path запрещены:

- чтение `STATE`, phase либо window identity;
- TLS access, owner cookie, recursion flag или slot claim/admission/completion;
- atomic load/store/RMW, counter/fault/poison update и close handshake;
- allocation, logging, lock, process API, panic/unwind и wrapper helper,
  способный повторно делегировать call.

Разрешённый `unsafe` не расширяется: это тот же single tooling-only
`GlobalAlloc` boundary ADR-039 и тот же direct `System` call с локальным
`SAFETY` rationale. Contract-valid pointer/layout ownership передаётся ровно
один раз. Invalid pointer/layout остаётся нарушением unsafe precondition
caller; wrapper не пытается диагностировать или исправлять undefined behavior.

### Begin, close и memory model

`begin` и `finish` синхронизируют только callbacks, которые способны изменить
публикуемые counter либо measurement fault state. Поэтому:

1. owner/foreign admission order для `alloc`, `alloc_zeroed` и `realloc`
   полностью сохраняется;
2. сам direct `dealloc` не создаёт missed attribution или measurement-state
   write; отдельный count-bearing callback, если он возник, имеет собственный
   admission;
3. `finish` может линейризоваться и опубликовать текущий gross snapshot
   параллельно с direct `dealloc`, потому что тот не пишет measurement memory;
4. dealloc-only foreign thread не является measurement participant, не claim-ит
   один из `4 096` slots и не может вызвать slot exhaustion;
5. owner pre-registration, foreign close handshake, counter aggregation и
   sticky poison остаются прежними для count-bearing callbacks.

Participant bound `4 096` относится к measurement owners и threads, которые
выполняют count-bearing callbacks. Если dealloc-only thread позднее выполняет
такой callback, он обязан claim-ить slot обычным путём и либо точно учитывается,
либо честно получает существующий `SlotExhausted` fault.

Если во время underlying system deallocation гипотетически возникает отдельный
count-bearing global callback, этот callback самостоятельно проходит обычный
ADR-040/ADR-041 protocol и атрибутируется по собственному admission; outer
`dealloc` не скрывает его и не наследует его state.

Allocator lifetime и завершение workload остаются обязанностью scenario owner:
этот ADR не разрешает публиковать результат до завершения production work и не
меняет fixed commit/checkpoint boundary. Он лишь не использует measurement
state machine как лишний join для операции, которая не входит в snapshot.
Решение не утверждает, что native heap effects `dealloc` коммутируют с будущим
`alloc`/`realloc`: их фактические success outcomes учитываются собственным
admission в том окне, где они произошли.

Recursion, TLS/slot failure, invalid zero-size request, sequence/counter/window
overflow и admitted fault продолжают fail-closed для count-bearing callbacks.
Они не создаются и не диагностируются direct `dealloc`: у него нет
measurement state, partial counter update или отдельного valid success result.

### Bounds, codegen и target admission

Fixed registry, TLS/control state и `reserved_bytes()` сохраняют общий
`<=64 MiB` limit; pass-through не добавляет state. Pinned Rust `1.93.0` release
codegen gate на admitted target обязан отдельно доказать:

- outer deallocation symbol содержит direct matching system deallocation;
- его IR/assembly не содержит `STATE`, TLS, slots, atomics/RMW, active helpers,
  measurement faults, allocation recursion или EH control flow;
- owner/inactive/foreign gates применяются только к трём count-bearing
  operations и сохраняют прежние ADR-041/ADR-040 свойства;
- valid dealloc pointer/layout и exactly-once delegation не изменились;
  pointer/data/null-return checks сохраняются отдельно для count-bearing calls.

Windows unwind metadata/personality без `invoke`, landing, catch или cleanup не
считается EH control flow, как и в inherited ADR-041 gate.

Windows `x86_64-pc-windows-msvc` admission остаётся локальным. Native Linux
`x86_64-unknown-linux-gnu` evidence остаётся отдельным `LNX-006`; Windows PASS
его не подменяет. Если compiler/target shape не доказан, metric на target
остаётся `NOT_RUN`, а shipping allocator не меняется.

Proof ограничен pinned `rust-std-system` backend без пользовательской allocator
interposition: Windows path делегирует native process heap, Linux path — native
system allocator. Если target/build не доказывает этот shape либо добавляет
reentrant interposition, он не admitted вместо возврата dealloc recursion guard.

## Memory-model invariants

1. Только три count-bearing callbacks читают Active identity и могут писать
   counters/faults.
2. Owner writes этих callbacks остаются sequenced-before same-thread finish по
   ADR-041.
3. Foreign odd admission/postcheck/Closing/handshake остаются единым ADR-040 SC
   order для каждого count-bearing callback.
4. Direct `dealloc` не читает и не пишет measurement memory, поэтому не требует
   attribution/happens-before edge с begin/finish. Это не является заявлением
   о коммутативности его native heap effect с будущими allocator operations.
5. Exactly-once matching `System` delegation сохраняется для всех четырёх
   `GlobalAlloc` operations.

## Relevant product checks

| Check | Scenario | Expected | Fallback |
|---|---|---|---|
| `fast` | known-count owner+foreign operations; valid dealloc во всех control phases; fresh unclaimed dealloc-only thread; synthetic logical capacity cursor; model-only coordinated overlap begin/close/следующего window без production pause hook; hook pre-touch; valid pointer/layout/data parity | Deallocation не делает direct counter/participant/slot/hook/poison update, не задерживает close и делегируется ровно один раз; later counted call проходит normal claim; три counted operations exact | Counter unavailable |
| `fast` fault matrix | recursion/TLS/slot/overflow injection вокруг counted callbacks и direct dealloc | Existing counted faults fail closed; dealloc при активных test hooks не меняет marker/counters/faults и не создаёт/скрывает counter fault | Counter unavailable |
| Boundary/codegen audit | pinned release Windows/Linux | Dealloc direct System, zero STATE/TLS/RMW/helper/fault/EH; остальные три operations сохраняют ADR-040/041 shape | Target `NOT_RUN` |
| `allocator-counter-check` | неизменный rotated 2+15 release kernel, единственный candidate-6 | inactive/enabled независимо `<=3%`, state `<=64 MiB`, exact roots, valid dealloc pointer/data parity и counted-call null semantics | Metric disabled; failed candidate не повторяется |
| `performance` smoke/worker/long | prepared in-process scenario window | Nonzero exact count-bearing process/PID/window evidence; deallocation semantics explicit | Stable unavailable diagnostic |
| `play` / `persistence-replay` | counter/profiler off/on | Exact command/event/save/replay/state/ledger roots | Reject divergent run |

## Рассмотренные варианты

- **Оставить ADR-041 dealloc instrumentation и принять `+5.88%`.** Отклонено:
  нарушает неизменный `3%` gate и сохраняет работу без observable value.
- **Вычитать deallocated bytes или считать dealloc calls.** Отклонено: молча
  меняет report semantics, не даёт live/peak value и не имеет отдельного
  deallocation success signal в `GlobalAlloc`.
- **Оставить initial STATE load, но bypass-ить только active helper.**
  Отклонено: phase не влияет на direct delegation и load не защищает state.
- **Оставить foreign admission как lifetime fence.** Отклонено: dealloc не
  пишет measurement memory; allocator pointer lifetime принадлежит caller, а
  не counter snapshot.
- **Заменить `LocalKey::try_with` на `with`.** Отклонено как измерительная
  гипотеза: pinned release уже свёл normal path к одному direct native TLS
  access без `try_with` runtime call или error branch.
- **Удалить только recursion flag.** Отклонено: codegen убирает лишь четыре hot
  instructions, но ослабляет fail-closed policy всех count-bearing callbacks и
  сохраняет остальную не наблюдаемую dealloc state machine.
- **Ослабить exactness, поднять budget или заменить shipping allocator.**
  Отклонено ADR-036/ADR-039 и retained evidence policy.

## Последствия

- Следующая code subphase удаляет только dealloc dispatch/helpers и обновляет
  exact source/codegen/tests; count-bearing protocol не переписывается.
- Первый полный candidate после implementation сохраняется независимо от
  verdict и не повторяется как retry-to-green.
- До exactness, codegen, profiler parity и overhead `PASS` allocator fields
  остаются unavailable, B-12 — `OPEN`, hard timing calibration не начинается.
- ADR-042 не является shipping allocator optimization и не меняет gameplay,
  persistence, replay, scheduling либо public contracts.
